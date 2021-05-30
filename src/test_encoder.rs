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
use hinge::motor::{encoder, interrupts};
use encoder::{Encoder, ISR_A, ISR_B, ISR_C};


#[embassy::main]
async fn main(_spawner: Spawner, ep: embassy_nrf::Peripherals) -> ! {

    interrupts::enable();

    let mut encoder_a = Encoder::from(&ISR_A);
    let mut encoder_b = Encoder::from(&ISR_B);
    let mut encoder_c = Encoder::from(&ISR_C);

    let mut pos = 0;
    loop {
        let (dist, spd) = encoder_c.wait().await;
        pos += (dist as i16);
        defmt::info!("\rpos: {}, dist: {}, spd: {}", pos, dist, spd);
    }
    pending::<()>().await;
}
