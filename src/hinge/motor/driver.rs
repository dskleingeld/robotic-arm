use embassy_nrf::gpio::{self, AnyPin};
use nrf52832_hal::pwm::Instance as PwmInstance;
use nrf52832_hal::pwm::PwmChannel;

#[derive(defmt::Format)]
pub enum Direction {
    ClockWise,
    CounterClockWise,
}

impl From<f64> for Direction {
    fn from(spd: f64) -> Direction {
        if spd.is_positive() {
            return Self::ClockWise;
        }
        Self::CounterClockWise
    }
}

pub struct Driver<'a, T: PwmInstance> {
    pwm: PwmChannel<'a, T>,
    en_1: gpio::Output<'a, AnyPin>,
    en_2: gpio::Output<'a, AnyPin>, 
}

impl<'a, T: PwmInstance> Driver<'a, T> {
    pub fn from(pwm: PwmChannel<'a, T>, enable_1: AnyPin, enable_2: AnyPin)  -> Self {
        use embassy_nrf::gpio::{Output, Level, OutputDrive};
        Self {
            pwm,
            en_1: Output::new(enable_1, Level::Low, OutputDrive::Standard),
            en_2: Output::new(enable_2, Level::Low, OutputDrive::Standard),
        }
    }

    pub fn set(&mut self, value: f64) {
        self.set_dir(value);
        self.set_power(value);
    }

    fn set_power(&self, value: f64) {
        use core::f64;
        use ieee754::Ieee754;

        let max_duty = (0.7 * self.pwm.max_duty() as f32) as u16;
        // note 50% duty cycle bad says internet....
        let power: f64 = value.abs();
        let power = power as u16;
        let power = power.min(max_duty); // limit output power
        defmt::info!("setting power: {}, max: {}", power, max_duty);
        self.pwm.set_duty_off(power);
    }
    
    fn set_dir(&mut self, dir: impl Into<Direction>) {
        use embedded_hal::digital::v2::OutputPin;
        let dir = dir.into();
        defmt::info!("dir: {}", dir);
        match dir {
            Direction::ClockWise => {
                self.en_2.set_low().unwrap();
                self.en_1.set_high().unwrap(); 
            }
            Direction::CounterClockWise => {
                self.en_1.set_low().unwrap();
                self.en_2.set_high().unwrap(); 
            }
        }
    }
}
