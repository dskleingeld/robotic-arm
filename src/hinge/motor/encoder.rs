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
    ch: InputChannel<'d, C, P>,
    state: State,
}

impl<'d, C, P> Encoder<'d, C,P> 
    where 
        C: gpiote::Channel, 
        P: gpio::Pin+Unborrow + Unborrow<Target = P>
    {

    pub fn from(pin: P, gp: gpiote::Initialized, channel: C) -> Self {
        let pin = Input::new(pin, Pull::Down);
        let ch = InputChannel::new(gp, channel, pin, InputChannelPolarity::Toggle);

        Self {
            ch,
            state: State::default(),
        }
    }
}

impl<'d, C: gpiote::Channel, P: gpio::Pin> Encoder<'d, C,P> {
    pub async fn wait(&mut self) -> (Distance, Speed) {
        self.ch.wait().await;
        let distance = 1;
        let speed = self.state.update(distance);
        (distance, speed)
    }
}
