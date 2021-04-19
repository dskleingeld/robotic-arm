#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../defmt_setup.rs"]
mod defmt_setup;
#[path = "../hinge/motor.rs"]
mod motor;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};
use embassy_nrf::Peripherals;
use embedded_hal::digital::v2::OutputPin;

use defmt_setup::*;
use motor::{Motor, MotorConfig, Controls};

static CTRL: Controls = Controls::default();

const TESTCFG: MotorConfig = MotorConfig {
    encoder_fdw: 1,
    encoder_back: 2,
    power_fwd: 3,
    power_back: 4,
};


async fn test(controls: &'static Controls) {
    loop {
        controls.set_speed(-5);
        Timer::after(Duration::from_millis(1000)).await;
        controls.set_speed(5);
        Timer::after(Duration::from_millis(1000)).await;
    }
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
async fn main(_spawner: Spawner) {
    let p = Peripherals::take().unwrap();
    let led = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    let mut motor = Motor::from(TESTCFG, &CTRL);
    info!("Testing motor");

    let test = test(&CTRL);
    let blink = blink(led);
    let motor = motor.maintain();

    futures::join!(test, blink, motor);
}
