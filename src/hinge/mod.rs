pub mod motor;
use motor::Motor;
use core::sync::atomic::{AtomicU8, Ordering};
use nrf52832_hal::pwm::PwmChannel;
use nrf52832_hal::pwm::Instance as PwmInstance;
use embassy::util::Unborrow;

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

use embassy_nrf::{gpiote, gpio};
pub struct Hinge<'a, T: PwmInstance, C: gpiote::Channel, P: gpio::Pin+Unborrow> {
    pos: Option<f32>, // degrees
    motor: Motor<'a, T, C, P>,
    controls: &'static Controls,
}

impl<'a, T: PwmInstance, C: gpiote::Channel, P: gpio::Pin+Unborrow> Hinge<'a, T, C, P> {
    // pub fn from(cfg: MotorConfig, controls: &'static Controls, pwm: PwmChannel<'a, T>) -> Self {
    pub fn from(motor: Motor<'a, T, C, P>, controls: &'static Controls) -> Self {
        Self {
            motor,
            pos: None,
            controls,
        }
    }
    pub async fn maintain_hinge(&mut self) {

    }

    pub async fn maintain_forever(&mut self) {
        self.motor.maintain().await;
        self.maintain_hinge().await;
    }
}
