#![allow(dead_code)]

use core::sync::atomic::{AtomicI8, AtomicU8, Ordering};
use defmt::info;
use embassy::time::{Duration, Timer};
use embassy::util::{Signal, Unborrow};
use embassy_nrf::gpio;
use nrf52832_hal::pwm::Instance as PwmInstance;
use nrf52832_hal::pwm::PwmChannel;

mod pwm;
mod encoder;
pub use pwm::init as pwm_init;
pub use encoder::Encoder;

type PwmPin = u8;

pub struct MotorConfig {
    pub encoder_fdw: u8,
    pub encoder_back: u8,
    pub power_fwd: PwmPin,
    pub power_back: PwmPin,
}

// Safe to share around as its all atomic, should be declared
// at static. Does not need to be mutable as atomics can be changed
// from & ref
pub struct Controls {
    target_speed: AtomicI8,
    max_torgue: AtomicU8,
    changed: Signal<()>,
}

impl Controls {
    /// by default hold position unless torgue gets
    /// to high
    pub const fn default() -> Self {
        Self {
            target_speed: AtomicI8::new(0),
            max_torgue: AtomicU8::new(50),
            changed: Signal::new(),
        }
    }
}

impl Controls {
    pub fn get_speed(&self) -> i8 {
        self.target_speed.load(Ordering::Release)
    }
    pub fn set_speed(&self, speed: i8) {
        self.target_speed.store(speed, Ordering::Release);
        self.changed.signal(());
    }
    /// change the direction
    pub fn set_dir(&self, dir: i8) {
        self.target_speed
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |s| Some(s * dir))
            .unwrap();
        self.changed.signal(());
    }
    pub fn set_max_torgue(&self, max: u8) {
        self.max_torgue.store(max, Ordering::Release);
        self.changed.signal(());
    }
}

use embassy_nrf::gpiote;
pub struct Motor<'a, T: PwmInstance, C: gpiote::Channel, P: gpio::Pin+Unborrow> {
    pwm: PwmChannel<'a, T>,
    relative_pos: u16, // degrees
    controls: &'static Controls,
    encoder: Encoder<'a, C, P>,
}

impl<'a, T: PwmInstance, C: gpiote::Channel, P: gpio::Pin+Unborrow> Motor<'a, T, C, P> {
    pub fn from(
        cfg: MotorConfig,
        controls: &'static Controls,
        encoder: Encoder<'a, C, P>,
        pwm: PwmChannel<'a, T>,
    ) -> Self {
        Self {
            pwm,
            relative_pos: 0,
            controls,
            encoder,
        }
    }

    pub async fn maintain_forever(&mut self) {
        loop {
            self.maintain().await;
        }
    }

    pub async fn maintain(&mut self) -> encoder::Change {
        // use core::pin::Pin;
        use futures::pin_mut;
        use futures::future::FutureExt;

        let mut changed = self.controls.changed.wait().fuse();
        let encoder = self.encoder.wait().fuse();
        let mut timeout = Timer::after(Duration::from_millis(100)).fuse();

        pin_mut!(encoder);
        futures::select_biased! {
            c = encoder => {
                info!("encoder moved: {}", c);
                c
            },
            () = changed => {
                // self.pwm.set_duty(u16::MAX/i);
                0
            },
            () = timeout => 0,
        }
    }
}
