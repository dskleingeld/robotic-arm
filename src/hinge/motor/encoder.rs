use embassy_nrf::gpio::{self, Input, Pull};
use embassy_nrf::gpiote::{self, InputChannel, InputChannelPolarity};
use embassy::time;
use embassy::util::Unborrow;

pub type Speed = i32;
pub type Distance = i8;

#[derive(Default)]
pub struct State {
    last_updated: Option<time::Instant>,
}

impl State {
    fn update(&mut self, distance: Distance) -> Speed {
        let speed = if let Some(t1) = self.last_updated {
            let distance = distance as i32;
            let elapsed = t1.elapsed().as_millis() as i32;
            distance/elapsed
        } else { 
            0
        };

        self.last_updated = Some(time::Instant::now());
        speed
    }
}

pub struct Encoder<'d, C: gpiote::Channel, P: gpio::Pin> {
    ch_0: InputChannel<'d, C, P>,
    ch_1: InputChannel<'d, C, P>,
    state: State,
}

impl<'d, C, P> Encoder<'d, C,P> 
    where 
        C: gpiote::Channel, 
        P: gpio::Pin+Unborrow + Unborrow<Target = P>
    {

    pub fn from(pin_0: P, pin_1: P, gp: gpiote::Initialized, channel_0: C, channel_1: C) -> Self {
        let pin_0 = Input::new(pin_0, Pull::Down);
        let pin_1 = Input::new(pin_1, Pull::Down);
        let ch_0 = InputChannel::new(gp, channel_0, pin_0, InputChannelPolarity::Toggle);
        let ch_1 = InputChannel::new(gp, channel_1, pin_1, InputChannelPolarity::Toggle);

        Self {
            ch_0,
            ch_1,
            state: State::default(),
        }
    }
}

impl<'d, C: gpiote::Channel, P: gpio::Pin> Encoder<'d, C,P> {
    pub async fn wait(&mut self) -> (Distance, Speed) {
        use futures::pin_mut;
        use futures::future::FutureExt;

        let ch_0 = self.ch_0.wait().fuse();
        let ch_1 = self.ch_1.wait().fuse();

        pin_mut!(ch_0);
        pin_mut!(ch_1);
        futures::select_biased! {
            () = ch_0 => (),
            () = ch_1 => (),
        };

        let distance = 1;
        let speed = self.state.update(distance);
        (distance, speed)
    }
}
