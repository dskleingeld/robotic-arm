#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

mod defmt_setup;
mod hinge;

use defmt::panic;
use nrf52832_hal as hal;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};
use embassy_nrf::gpiote;
use embedded_hal::digital::v2::OutputPin;

use defmt_setup::*;
use hinge::motor::{Motor, Controls, pwm_init, Encoder};

static CTRL: Controls = Controls::default();

async fn test(controls: &'static Controls) {
    // loop {
        controls.set_speed(-5);
        Timer::after(Duration::from_millis(1000)).await;
        controls.set_speed(5);
        Timer::after(Duration::from_millis(1000)).await;
    // }
}

async fn blink<'d>(mut led: Output<'d, impl Pin>) {
    loop {
        led.set_high().unwrap();
        Timer::after(Duration::from_millis(1000)).await;
        led.set_low().unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy::main]
async fn main(_spawner: Spawner) -> ! {
    let ep = embassy_nrf::Peripherals::take().unwrap();
    let led = Output::new(ep.P0_17, Level::Low, OutputDrive::Standard);

    let hp = hal::pac::Peripherals::take().unwrap();
    let p0 = hal::gpio::p0::Parts::new(hp.P0);
    let pwm_pins = (p0.p0_18.degrade(), p0.p0_27.degrade(), p0.p0_26.degrade());

    let pwm = pwm_init(hp.PWM0, pwm_pins); 
    let (pwm0, _pwm1, _pwm2, _) = pwm.split_channels();

    use embassy_nrf::interrupt;
    let gp = gpiote::initialize(ep.GPIOTE, interrupt::take!(GPIOTE));
    let encoder = Encoder::from(ep.P0_11, gp, ep.GPIOTE_CH0);

    let mut motor = Motor::from(&CTRL, encoder, pwm0);
    info!("Testing motor");

    let test = test(&CTRL);
    let blink = blink(led);
    let motor = motor.maintain_forever();

    futures::join!(test, blink, motor);
}
