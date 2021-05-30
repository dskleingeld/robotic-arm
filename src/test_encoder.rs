#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

mod defmt_setup;
mod hinge;

use core::future::pending;
use defmt::panic;
use embassy::executor::Spawner;
use embassy_nrf::gpio::{Input, Pull};
use embassy_nrf::interrupt;
// pub use nrf52832_hal as hal;

use defmt_setup::*;
use hinge::motor::{Encoder, interrupts};


#[embassy::main]
async fn main(_spawner: Spawner, ep: embassy_nrf::Peripherals) -> ! {

    interrupts::enable();
    // let i30 = Input::new(ep.P0_30, Pull::None);
    // let i31 = Input::new(ep.P0_31, Pull::None);

    // interrupts::set_pin(30, 0);
    // interrupts::set_pin(31, 1);

    // let mut encoder = Encoder::from(
    //     p0.p0_31.degrade(),
    //     p0.p0_30.degrade(),
    // );

    // let mut pos = 0i16;
    // loop {
    //     let (dist, spd) = encoder.wait().await;
    //     pos += (dist as i16);
    //     defmt::info!("\rpos: {}, dist: {}, spd: {}", pos, dist, spd);
    // }
    pending::<()>().await;
}
