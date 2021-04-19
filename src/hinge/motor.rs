use core::sync::atomic::{AtomicI8, AtomicU8, Ordering};
use embassy::time::{Duration, Timer};
use embassy::util::Signal;
use defmt::info;

type Pin = u8;
type PwmPin = u8;

pub struct MotorConfig {
    pub encoder_fdw: Pin,
    pub encoder_back: Pin,
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
        self.target_speed.fetch_update(Ordering::Relaxed, 
            Ordering::Relaxed, 
            |s| Some(s*dir)).unwrap();
        self.changed.signal(());
    }
    pub fn set_max_torgue(&self, max: u8) {
        self.max_torgue.store(max, Ordering::Release);
        self.changed.signal(());
    }
}

pub struct Motor {
    pins: MotorConfig,
    relative_pos: u16, // degrees
    controls: &'static Controls,
}

use futures::future::{select, Either};
impl Motor {
    pub fn from(cfg: MotorConfig, controls: &'static Controls) -> Self {
        Self {
            pins: cfg,
            relative_pos: 0,
            controls,
        }
    }
    pub async fn maintain(&mut self) {
        loop {
            let changed = self.controls.changed.wait();
            let timeout = Timer::after(Duration::from_millis(1000));
            // TODO event
            match select(changed, timeout).await {
                Either::Left(_) => info!("changed"),
                Either::Right(_) => info!("timeout"),
            };
        }
    }
}
