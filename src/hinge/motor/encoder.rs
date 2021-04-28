use embassy_nrf::gpio::{self, Input, Pull};
use embassy_nrf::gpiote::{self, InputChannel, InputChannelPolarity};
use embassy::util::Unborrow;

pub type Change = i16;
pub struct Encoder<'d, C: gpiote::Channel, P: gpio::Pin> {
    ch: InputChannel<'d, C, P>,
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
        }
    }
}

impl<'d, C: gpiote::Channel, P: gpio::Pin> Encoder<'d, C,P> {
    pub async fn wait(&mut self) -> Change {
        self.ch.wait().await;
        1i16
    }
}
