#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

mod defmt_setup;
mod hinge;
mod config;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;
use embedded_hal::digital::v2::OutputPin;

use defmt_setup::*;
// use hinge::Hinge;

#[embassy::main]
async fn main(spawner: Spawner) {
    let p = Peripherals::take().unwrap();
    let mut led = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    info!("reading...");
    /* let hinge_lower = Hinge::from(config::LOWERMOTOR);
    let hinge_upper = Hinge::from(config::LOWERMOTOR);
    let hinge_grapper = Hinge::from(config::LOWERMOTOR); */

    loop {
        led.set_high().unwrap();
        Timer::after(Duration::from_millis(3000)).await;
        led.set_low().unwrap();
        Timer::after(Duration::from_millis(3000)).await;
    }
}

