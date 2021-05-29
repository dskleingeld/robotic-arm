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
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};
use embassy_nrf::gpiote;
use embassy_nrf::interrupt;
use embedded_hal::digital::v2::OutputPin;
use nrf52832_hal as hal;
use nrf52832_pac as pac;

use defmt_setup::*;
use hinge::motor::Encoder;

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            b => {
                self.0 &= !(1 << b);
                Some(b)
            }
        }
    }
}

#[interrupt]
unsafe fn GPIOTE() {
    pub const CHANNEL_COUNT: usize = 8;
    let g = &*pac::GPIOTE::ptr();

    for i in 0..CHANNEL_COUNT {
        let event = g.events_in[i].read().bits();
        if event != 0 {
            defmt::info!("channel: {}", i);
            g.intenclr.write(|w| unsafe { w.bits(1 << i) }); // mark interrupt as handled

            // re-enable interrupts
            g.events_in[i].reset();
            g.intenset.write(|w| unsafe { w.bits(1 << i) });
        }
    }
}

fn enable_gpio_interrupts() {
    use embassy::interrupt::{Interrupt, InterruptExt};
    let ports = unsafe { &[&*pac::P0::ptr()] };

    for &p in ports {
        // Enable latched detection
        p.detectmode.write(|w| w.detectmode().ldetect());
        // Clear latch
        p.latch.write(|w| unsafe { w.bits(0xFFFFFFFF) })
    }

    // Enable interrupts
    let irq = unsafe { interrupt::GPIOTE::steal() };
    irq.unpend();
    irq.set_priority(interrupt::Priority::P7);
    irq.enable();

    let g = unsafe { &*pac::GPIOTE::ptr() };
    g.events_port.write(|w| w);
    g.intenset.write(|w| w.port().set());
}

use embassy_nrf::gpio::{Input, Pull};
fn set_pin_interrupt<'d>(pin: Input<'d, impl Pin>, pin_numb: u8, channel_numb: usize) {
   let g = unsafe { &*pac::GPIOTE::ptr() };

    g.config[channel_numb].write(|w| { 
        w.mode().event().polarity().toggle();
        unsafe { w.psel().bits(pin_numb) }
    });

    //enable channel
    g.events_in[channel_numb].reset();
    g.intenset.write(|w| unsafe { w.bits(1 << channel_numb) });
}

#[embassy::main]
async fn main(_spawner: Spawner, ep: embassy_nrf::Peripherals) -> ! {

    // let hp = hal::pac::Peripherals::take().unwrap();
    // let p0 = hal::gpio::p0::Parts::new(hp.P0);
    enable_gpio_interrupts();
    let i30 = Input::new(ep.P0_30, Pull::None);
    let i31 = Input::new(ep.P0_31, Pull::None);

    set_pin_interrupt(i30, 30, 0);
    set_pin_interrupt(i31, 31, 1);

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
