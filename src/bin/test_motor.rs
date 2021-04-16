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
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;
use embedded_hal::digital::v2::OutputPin;

use defmt_setup::*;
use motor::{Motor, MotorConfig};

const TESTCFG: MotorConfig = MotorConfig {
    encoder_fdw: 1,
    encoder_back: 2,
    power_fwd: 3,
    power_back: 4,
};

async fn watch(motor: Motor) {
    loop {
    }
}

#[embassy::main]
async fn main(spawner: Spawner) {
    let p = Peripherals::take().unwrap();
    let mut led = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    let motor = Motor::from(TESTCFG);
    info!("Testing motor");

    watch(motor).await;

    loop {
        led.set_high().unwrap();
        Timer::after(Duration::from_millis(1000)).await;
        led.set_low().unwrap();
        Timer::after(Duration::from_millis(1000)).await;
    }
}
