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
    defmt::info!("hi");
    pub const CHANNEL_COUNT: usize = 8;
    let g = &*pac::GPIOTE::ptr();

    for i in 0..CHANNEL_COUNT {
        if g.events_in[i].read().bits() != 0 {
            g.intenclr.write(|w| unsafe { w.bits(1 << i) });
            defmt::info!("i: {}", i);
        }
    }

    if g.events_port.read().bits() != 0 {
        g.events_port.write(|w| w);

        let ports = &[&*pac::P0::ptr()];

        for (port, &p) in ports.iter().enumerate() {
            let bits = p.latch.read().bits();
            for pin in BitIter(bits) {
                defmt::info!("pin: {}", pin);
                p.pin_cnf[pin as usize].modify(|_, w| w.sense().disabled());
            }
            p.latch.write(|w| w.bits(bits));
        }
    }
}

fn setup_encoder_interrupt() {
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
fn test<'d>(pin: Input<'d, impl Pin>, pin_numb: u8) {
   let g = unsafe { &*pac::GPIOTE::ptr() };
   let num = 0; // TODO interrupt channel

    g.config[num].write(|w| { 
        w.mode().event().polarity().toggle();
        unsafe { w.psel().bits(pin_numb) }
    });

    g.events_in[num].reset();
}


#[embassy::main]
async fn main(_spawner: Spawner, ep: embassy_nrf::Peripherals) -> ! {

    // let hp = hal::pac::Peripherals::take().unwrap();
    // let p0 = hal::gpio::p0::Parts::new(hp.P0);
    setup_encoder_interrupt();
    let i30 = Input::new(ep.P0_30, Pull::None);
    let i31 = Input::new(ep.P0_31, Pull::None);

    test(i30, 30);

    let num = 0;
    let g = unsafe { &*pac::GPIOTE::ptr() };
    g.events_in[num].reset();
    g.intenset.write(|w| unsafe { w.bits(1 << num) });
    // test(i31);

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
