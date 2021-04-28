#![allow(dead_code)]

use defmt::info;
use core::sync::atomic::{AtomicI8, AtomicU8, Ordering};
use embassy::time::{Instant, Duration, Timer};
use embassy::util::{Signal, Unborrow};
use embassy_nrf::gpio;
use nrf52832_hal::pwm::Instance as PwmInstance;
use nrf52832_hal::pwm::PwmChannel;
use pid_lite::Controller as PidController;

mod pwm;
mod encoder;
pub use pwm::init as pwm_init;
pub use encoder::Encoder;

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
        self.target_speed.store(speed, Ordering::Release);
        self.changed.signal(()); //TODO FIXME BUG
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
    }
}

use embassy_nrf::gpiote;
pub struct Motor<'a, T: PwmInstance, C: gpiote::Channel, P: gpio::Pin+Unborrow> {
    state: State,
    last_update: Instant,
    pid: PidController,
    pwm: PwmChannel<'a, T>,
    controls: &'static Controls,
    encoder: Encoder<'a, C, P>,
}

impl<'a, T: PwmInstance, C: gpiote::Channel, P: gpio::Pin+Unborrow> Motor<'a, T, C, P> {
    const P_GAIN: f64 = 10.0;
    const I_GAIN: f64 = 10.0;
    const D_GAIN: f64 = 10.0;

    pub fn from(
        controls: &'static Controls,
        encoder: Encoder<'a, C, P>,
        pwm: PwmChannel<'a, T>,
    ) -> Self {

        Self {
            pwm,
            last_update: Instant::now(),
            pid: PidController::new(0.0, Self::P_GAIN, Self::I_GAIN, Self::D_GAIN),
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
        use futures::pin_mut;
        use futures::future::FutureExt;

        let mut changed = self.controls.changed.wait().fuse();
        let encoder = self.encoder.wait().fuse();
        let mut timeout = Timer::after(Duration::from_millis(1)).fuse();

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

        let power = self.pid.update_elapsed(self.state.speed as f64, duration);
        let power = power as u16; // TODO limit motor power

        info!("setting motor power to: {}", power);
        self.pwm.set_duty(power);


        self.state.clone()
    }
}
