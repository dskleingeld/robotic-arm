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
// use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;

// use hinge::{MotorConfig, Hinge};
use defmt_setup::*;

/* const TESTCFG: MotorConfig = MotorConfig {
    encoder_fdw: 1,
    encoder_back: 2,
    power_fwd: 3,
    power_back: 4,
};
 */

/* async fn test(hinge: Hinge) {
    Timer::after(Duration::from_millis(1000)).await;
    // hinge.set_target(5);
} */


#[embassy::main]
async fn main(spawner: Spawner, p: embassy_nrf::Peripherals) {
    let led = Output::new(p.P0_18, Level::Low, OutputDrive::Standard);
    info!("Testing hinge");
    /* let mut hinge = Hinge::from(TESTCFG);

    let control = hinge.maintain();
    let test = test(hinge);

    futures::join!(control, test); */
}

