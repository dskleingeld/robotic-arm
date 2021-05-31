pub mod motor;
use motor::Motor;
use core::sync::atomic::{AtomicU8, AtomicI16, Ordering};
use nrf52832_hal::pwm::Instance as PwmInstance;
use embassy::time::Instant;
use embassy::util::Signal;
use pid_lite::Controller as PidController;


pub struct Controls {
    pub motor: motor::Controls,
    target: AtomicI16,
    max_spd: AtomicU8,
    changed: Signal<()>,
}



impl Controls {
    pub const fn default() -> Self {
        Self {
            motor: motor::Controls::default(),
            target: AtomicI16::new(0),
            max_spd: AtomicU8::new(1),
            changed: Signal::new(),
        }
    }

    pub fn max_spd(&self) -> u8 {
        self.max_spd.load(Ordering::Relaxed)
    }
    pub fn set_max_spd(&self, max_spd: u8) {
        self.max_spd.store(max_spd, Ordering::Relaxed);
        self.changed.signal(());
    }
    pub fn target(&self) -> i16 {
        self.target.load(Ordering::Relaxed)
    }
    pub fn set_target(&self, target: i16) {
        self.target.store(target, Ordering::Relaxed);
        self.changed.signal(());
    }
}

pub struct Hinge<'a, T: PwmInstance> {
    motor: Motor<'a, T>,
    controls: &'static Controls,
    pid: PidController,
    last_update: Instant,
}

impl<'a, T: PwmInstance> Hinge<'a, T> {
    const P_GAIN: f64 = 0.35;
    const I_GAIN: f64 = 0.0005;
    const D_GAIN: f64 = 1.0;

    pub fn from(motor: Motor<'a, T>, controls: &'static Controls) -> Self {
        Self {
            motor,
            controls,
            pid: PidController::new(0.0, Self::P_GAIN, Self::I_GAIN, Self::D_GAIN),
            last_update: Instant::now(),
        }
    }

    pub async fn maintain(&mut self) {
        use core::f64;
        use ieee754::Ieee754;
        use futures::future::FutureExt;
        use futures::pin_mut;

        let mut state = motor::State::default();

        let mut changed = self.controls.changed.wait().fuse();
        let mut motor = self.motor.maintain2().fuse();

        pin_mut!(motor);
        futures::select_biased! {
            () = changed => {
                let target = self.controls.target();
                self.pid.set_target(target as f64);
            },
            new_state = motor => {
                state = new_state;
            },
        };

        let duration = self.last_update.elapsed().as_millis();
        let duration = core::time::Duration::from_millis(duration);
        self.last_update = Instant::now();
        // defmt::info!("state: {}", state);
        let speed = self.pid.update_elapsed(state.relative_pos as f64, duration);
        // defmt::info!("pid speed: {}, pos: {}, controller: {}", speed, state.relative_pos, self.pid);

        let sign = ((speed*10.0) as i8).signum();
        let speed = if (speed*10.) as i8 == 0 {
            0
        } else if speed.abs() as i8 > 1 {
            speed as i8
        } else {
            (speed*10.) as i8
        };
        let speed = sign * speed.abs().min(5);
        self.controls.motor.set_pos(speed as i16); // DOES NOT WORK IN THIS BRANCH
    }

    pub async fn maintain_forever(&mut self) {
        loop {
            self.maintain().await;
        }
    }
}
