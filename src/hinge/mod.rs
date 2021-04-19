mod motor;
use motor::Motor;
use embassy::time::{Duration, Timer};
use core::sync::atomic::{AtomicU8, Ordering};

pub use motor::MotorConfig;

pub struct Controls {
    motor: motor::Controls,
    target: AtomicU8,
    pos: AtomicU8,
}

impl Controls {
    fn pos(&self) -> u8 {
        self.pos.load(Ordering::Relaxed)
    }
    pub fn set_speed(&self, speed: u8) { 
        let dir = self.motor.get_speed().signum();
        self.motor.set_speed(dir * speed as i8);
    }
    pub fn set_max_torgue(&self, max: u8) {
        self.motor.set_max_torgue(max);
    }
    pub fn set_target(&self, target: u8) {
        self.target.store(target, Ordering::Relaxed);
        let dir = if self.pos() > target {
            -1
        } else {
            1
        };
        self.motor.set_dir(dir);
    }
}

pub struct Hinge {
    pos: Option<f32>, // degrees
    motor: Motor,
    controls: &'static Controls,
}

impl Hinge {
    pub fn from(cfg: MotorConfig, controls: &'static Controls) -> Self {
        Self {
            motor: Motor::from(cfg, &controls.motor),
            pos: None,
            controls,
        }
    }
    pub async fn maintain_hinge(&mut self) {

    }

    pub async fn maintain(&mut self) {
        self.motor.maintain().await;
    }
}
