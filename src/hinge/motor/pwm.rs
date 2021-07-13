use hal::gpio::{Level, Pin};
use hal::pwm::{Channel, Instance, Pwm};
use hal::time::U32Ext;
use nrf52832_hal as hal; //hz()

pub fn init<'a, M, T: Instance>(pwm: T, pins: (Pin<M>, Pin<M>, Pin<M>)) -> Pwm<T> {
    // setup pins
    let pin0 = pins.0.into_push_pull_output(Level::Low);
    let pin1 = pins.1.into_push_pull_output(Level::Low);
    let pin2 = pins.2.into_push_pull_output(Level::Low);

    let pwm = Pwm::new(pwm);
    pwm.set_output_pin(Channel::C0, &pin0)
        .set_output_pin(Channel::C1, &pin1)
        .set_output_pin(Channel::C2, &pin2)
        .set_period(100_000u32.hz()) // experimentally found, lower freq lower motor power
        .enable();

    // let (pwm1, pwm2, pwm3, pwm4) = pwm.split_channels();
    // (pwm, pwm1, pwm2, pwm3)
    pwm
}
