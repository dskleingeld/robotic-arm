#![allow(dead_code)]

use core::sync::atomic::{AtomicI8, AtomicU8, Ordering};
use embassy::time::{Duration, Instant, Timer};
use embassy::util::Signal;
use embassy_nrf::gpio;
use nrf52832_hal::pwm::Instance as PwmInstance;
use nrf52832_hal::pwm::PwmChannel;
use pid_lite::Controller as PidController;

pub mod encoder;
mod pwm;
mod driver;
pub use driver::Driver;
pub use encoder::{Encoder, interrupts};
pub use pwm::init as pwm_init;

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
        self.target_speed.load(Ordering::Relaxed)
    }
    pub fn set_speed(&self, speed: i8) {
        self.target_speed.store(speed, Ordering::Relaxed);
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

#[derive(Default, Clone)]
pub struct State {
    relative_pos: encoder::Distance,
    speed: encoder::Speed,
}

impl State {
    pub fn update(&mut self, dist: encoder::Distance, spd: encoder::Speed) {
        self.relative_pos += dist;
        self.speed = spd;
        defmt::info!("rel_pos: {}", self.relative_pos);
    }
}

pub struct Motor<'a, T: PwmInstance> {
    state: State,
    last_update: Instant,
    pid: PidController,
    driver: Driver<'a, T>,
    controls: &'static Controls,
    encoder: Encoder,
}

// 2, 0.5
impl<'a, T: PwmInstance> Motor<'a, T> {
    const P_GAIN: f64 = 1.7;
    const I_GAIN: f64 = 0.5;
    const D_GAIN: f64 = 0.01;

    pub fn from(controls: &'static Controls, encoder: Encoder, driver: Driver<'a, T>) -> Self {
        Self {
            last_update: Instant::now(),
            pid: PidController::new(0.0, Self::P_GAIN, Self::I_GAIN, Self::D_GAIN),
            driver,
            state: State::default(),
            controls,
            encoder,
        }
    }

    pub async fn maintain_forever(&mut self) {
        loop {
            self.maintain().await;
        }
    }

    pub async fn maintain(&mut self) -> State {
        use futures::future::FutureExt;
        use futures::pin_mut;

        let mut changed = self.controls.changed.wait().fuse();
        let encoder = self.encoder.wait().fuse();
        let mut timeout = Timer::after(Duration::from_millis(10)).fuse();

        pin_mut!(encoder);
        futures::select_biased! {
            (dist, spd) = encoder => self.state.update(dist, spd),
            () = changed => {
                let speed = self.controls.get_speed() as f64;
                self.pid.set_target(speed);
            },
            () = timeout => self.state.update(0, 0),
        };

        let duration = self.last_update.elapsed().as_millis();
        let duration = core::time::Duration::from_millis(duration);
        self.last_update = Instant::now();

        defmt::info!("last known speed: {}, pid: {}", self.state.speed, self.pid);
        let power = self.pid.update_elapsed(self.state.speed as f64, duration);
        defmt::info!("pid output: {}", power);

        // NOTE can be negative
        self.driver.set(power);

        self.state.clone()
    }
}
