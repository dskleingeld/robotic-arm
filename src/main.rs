#![no_std]
#![no_main]
#![feature(impl_trait_in_bindings)]
#![feature(min_type_alias_impl_trait)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

extern crate nrf52832_hal;

mod config;
mod defmt_setup;
mod hinge;

use defmt::panic;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Output, OutputDrive};
use embassy_nrf::Peripherals;
use embedded_hal::digital::v2::OutputPin;

use defmt_setup::*;
// use hinge::Hinge;

use hal::gpio::Level;
use hal::gpiote::{Gpiote, TaskOutPolarity};
use hal::pac::interrupt;
use hal::ppi::{self, ConfigurablePpi, Ppi};
use nrf52832_hal as hal;
// use nrf52832_hal::gpiote::Gpiote;
use core::sync::atomic::{AtomicU8, Ordering};

static TEST: AtomicU8 = AtomicU8::new(0);

#[interrupt]
fn GPIOTE() {
    TEST.fetch_add(1, Ordering::Relaxed);
}

#[embassy::main]
async fn main(_spawner: Spawner) {
    let p = hal::pac::Peripherals::take().unwrap();
    let p0 = hal::gpio::p0::Parts::new(p.P0);
    let btn1 = p0.p0_11.into_pullup_input().degrade();
    let btn2 = p0.p0_12.into_pullup_input().degrade();
    let btn3 = p0.p0_24.into_pullup_input().degrade();
    let btn4 = p0.p0_25.into_pullup_input().degrade();
    let led1 = p0.p0_13.into_push_pull_output(Level::High).degrade();

    let gpiote = Gpiote::new(p.GPIOTE);

    // Set btn1 to generate event on channel 0 and enable interrupt
    gpiote
        .channel0()
        .input_pin(&btn1)
        .hi_to_lo()
        .enable_interrupt();

    // Set both btn3 & btn4 to generate port event
    gpiote.port().input_pin(&btn3).low();
    gpiote.port().input_pin(&btn4).low();
    // Enable interrupt for port event
    gpiote.port().enable_interrupt();

    // PPI usage, channel 2 event triggers "task out" operation (toggle) on channel 1 (toggles led1)
    gpiote
        .channel1()
        .output_pin(led1)
        .task_out_polarity(TaskOutPolarity::Toggle)
        .init_high();
    gpiote.channel2().input_pin(&btn2).hi_to_lo();
    let ppi_channels = ppi::Parts::new(p.PPI);
    let mut ppi0 = ppi_channels.ppi0;
    ppi0.set_task_endpoint(gpiote.channel1().task_out());
    ppi0.set_event_endpoint(gpiote.channel2().event());
    ppi0.enable();

    let p = Peripherals::take().unwrap();
    let mut led = Output::new(
        p.P0_18,
        embassy_nrf::gpio::Level::Low,
        OutputDrive::Standard,
    );
    loop {
        led.set_high().unwrap();
        Timer::after(Duration::from_millis(3000)).await;
        led.set_low().unwrap();
        Timer::after(Duration::from_millis(3000)).await;
        info!(
            "from interrupt handler: value: {}",
            TEST.load(Ordering::Relaxed)
        )
    }
}
