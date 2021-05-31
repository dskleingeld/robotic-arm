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
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive, Pin};
use embassy_nrf::interrupt;
use embedded_hal::digital::v2::OutputPin;
use nrf52832_hal as hal;

use defmt_setup::*;
use hinge::motor::{pwm_init, Controls, Motor, Driver};
use hinge::motor::{encoder, interrupts};
use encoder::{Encoder, ISR_A, ISR_B, ISR_C};


static CTRL_0: Controls = Controls::default();
static CTRL_1: Controls = Controls::default();
static CTRL_2: Controls = Controls::default();

async fn test(controls: &'static Controls) {
    loop {
        controls.set_pos(-10);
        Timer::after(Duration::from_millis(10000)).await;
        controls.set_pos(-30);
        Timer::after(Duration::from_millis(10000)).await;
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
async fn main(_spawner: Spawner, ep: embassy_nrf::Peripherals) -> ! {
    let led = Output::new(ep.P0_17, Level::Low, OutputDrive::Standard);

    let hp = hal::pac::Peripherals::take().unwrap();
    let p0 = hal::gpio::p0::Parts::new(hp.P0);
    let pwm_pins = (p0.p0_04.degrade(), p0.p0_27.degrade(), p0.p0_16.degrade());

    let pwm = pwm_init(hp.PWM0, pwm_pins);
    let (pwm0, pwm1, pwm2, _) = pwm.split_channels();

    interrupts::enable();
    let encoder = Encoder::from(&ISR_A);
    let driver = Driver::from(pwm0, ep.P0_03.degrade(), ep.P0_02.degrade());
    let mut motor_a = Motor::from(&CTRL_0, encoder, driver);

    let encoder = Encoder::from(&ISR_B);
    let driver = Driver::from(pwm1, ep.P0_26.degrade(), ep.P0_25.degrade());
    let mut motor_b = Motor::from(&CTRL_1, encoder, driver);

    let encoder = Encoder::from(&ISR_C);
    let driver = Driver::from(pwm2, ep.P0_19.degrade(), ep.P0_20.degrade());
    let mut motor_c = Motor::from(&CTRL_2, encoder, driver);

    // info!("Testing motor");

    let test = test(&CTRL_0);
    // let blink = blink(led);
    let motor_a = motor_a.maintain_forever();
    // let motor_b = motor_b.maintain_forever();
    // let motor_c = motor_c.maintain_forever();

    // futures::join!(test, blink, motor_a,motor_b,motor_c);
    futures::join!(test, motor_a);
}
