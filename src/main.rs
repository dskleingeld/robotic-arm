#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

extern crate nrf52832_hal;
use embassy::executor::Spawner;

mod defmt_setup;
use defmt_setup::*;

use defmt::panic;


#[embassy::main]
async fn main(_spawner: Spawner) {

}
