#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

mod defmt_setup;
mod hinge;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};
use embassy_nrf::gpiote;
use embassy_nrf::interrupt;
use embedded_hal::digital::v2::OutputPin;
use nrf52832_hal as hal;

use defmt_setup::*;
use hinge::motor::{pwm_init, Controls, Encoder, Motor};

static CTRL_0: Controls = Controls::default();
static CTRL_1: Controls = Controls::default();
static CTRL_2: Controls = Controls::default();

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
async fn main(_spawner: Spawner, ep: embassy_nrf::Peripherals) -> ! {
    let led = Output::new(ep.P0_17, Level::Low, OutputDrive::Standard);

    let hp = hal::pac::Peripherals::take().unwrap();
    let p0 = hal::gpio::p0::Parts::new(hp.P0);
    let pwm_pins = (p0.p0_04.degrade(), p0.p0_27.degrade(), p0.p0_16.degrade());

    let pwm = pwm_init(hp.PWM0, pwm_pins);
    let (pwm0, pwm1, pwm2, _) = pwm.split_channels();

    let encoder = Encoder::from(
        p0.p0_31.degrade(),
        p0.p0_30.degrade(),
    );
    let mut motor_a = Motor::from(&CTRL_0, encoder, pwm0);
    // TODO hinge

    let encoder = Encoder::from(
        p0.p0_11.degrade(),
        p0.p0_12.degrade(),
    );
    let mut _motor_b = Motor::from(&CTRL_1, encoder, pwm1);
    // TODO hinge

    let encoder = Encoder::from(
        p0.p0_22.degrade(),
        p0.p0_23.degrade(),
    );
    let mut _motor_c = Motor::from(&CTRL_2, encoder, pwm2);
    // TODO hinge

    info!("Testing motor");

    let test = test(&CTRL_0);
    let blink = blink(led);
    let motor = motor_a.maintain_forever();

    futures::join!(test, blink, motor);
}
