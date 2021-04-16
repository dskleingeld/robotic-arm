mod motor;
use motor::Motor;
use embassy::time::{Duration, Timer};

pub use motor::MotorConfig;


pub struct Hinge {
    pos: Option<f32>, // degrees
    motor: Motor,
}

impl Hinge {
    pub fn from(cfg: MotorConfig) -> Self {
        Self {
            motor: Motor::from(cfg),
            pos: None,
        }
    }
    pub async fn set_target(deg: f32) {
    }

    pub async fn maintain(&mut self) {
        loop {
            Timer::after(Duration::from_millis(1000)).await;
        }
    }
}
