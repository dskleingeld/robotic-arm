use embassy::time::{Duration, Instant, Timer};
use core::sync::atomic::{AtomicI16, Ordering};
use embassy_nrf::gpio::{self, AnyPin, Input, Pull};
// use nrf52832_hal as hal;
// use hal::{gpio, pac};
use rotary_encoder_hal::Rotary;
use lazy_static::lazy_static;
use core::cell::UnsafeCell;

pub mod interrupts;

pub type Speed = i32;
pub type Distance = i16;

lazy_static! {
    pub static ref ISR_A: EncoderISR = unsafe { EncoderISR::from((11,0), (12,1)) };
    pub static ref ISR_B: EncoderISR = unsafe { EncoderISR::from((15,2), (16,3)) };
    pub static ref ISR_C: EncoderISR = unsafe { EncoderISR::from((30,4), (31,5)) };
}

type EncoderPin = gpio::Input<'static, AnyPin>;
pub struct EncoderISR {
    channels: [u8;2],
    pins: [u8;2],
    inner: UnsafeCell<Rotary<EncoderPin, EncoderPin>>,
    dist: AtomicI16,
}

unsafe impl Sync for EncoderISR {}
impl EncoderISR {
    pub fn enable(&self) {
        for i in 0..2 {
            interrupts::set_pin(self.pins[i], self.channels[i].into());
        }
    }

    pub unsafe fn from(pc0: (u8, u8), pc1: (u8,u8)) -> Self {
        let (pin_numb1, channel_1) = pc0;
        let pin = AnyPin::steal(pin_numb1);
        let pin0 = Input::new(pin, Pull::None);

        let (pin_numb2, channel_2) = pc1;
        let pin = AnyPin::steal(pin_numb2);
        let pin1 = Input::new(pin, Pull::None);
        
        Self {
            channels: [channel_1, channel_2],
            pins: [pin_numb1, pin_numb2],
            inner: UnsafeCell::new(Rotary::new(pin0, pin1)),
            dist: AtomicI16::new(0),
        }
    }

    /// MUST only be called from isr
    pub unsafe fn update(&self) {
        use rotary_encoder_hal::Direction::*;
        let inner = self.inner.get();
        let inner = inner.as_mut().unwrap();
        match inner.update() {
            Err(_) => defmt::panic!("encoder had problem with pins"),
            Ok(None) => (), 
            Ok(Clockwise) => {self.dist.fetch_add(1, Ordering::SeqCst); ()}
            Ok(CounterClockwise) => {self.dist.fetch_sub(1, Ordering::SeqCst); ()}
        }
    }
}

pub struct Encoder {
    isr: &'static EncoderISR,
    last_spd_update: Option<Instant>,
}

impl Encoder {
    const PERIOD: Duration = Duration::from_millis(10); // ms
    pub fn from(isr: &'static EncoderISR) -> Self {
        Self {
            isr,
            last_spd_update: None,
        }
    }

    pub async fn wait(&mut self) -> (Distance, Speed) {
        loop { 
            let next = self.last_spd_update.unwrap_or(Instant::now())+Self::PERIOD;
            Timer::at(next).await;

            let distance = self.isr.dist.swap(0, Ordering::Relaxed);
            if distance > 0 {
                let speed = self.update(distance);
                return (distance, speed)
            }
        }
    }

    fn update(&mut self, distance: Distance) -> Speed {
        use embassy::time::TICKS_PER_SECOND;

        let speed = if let Some(t1) = self.last_spd_update {
            let distance = distance as i32;
            let elapsed = t1.elapsed().as_ticks() as i32;
            defmt::debug!("elapsed: {}, tps: {}, dist: {}", elapsed, TICKS_PER_SECOND, distance);
            distance * (TICKS_PER_SECOND as i32) / elapsed
        } else {
            0
        };

        self.last_spd_update = Some(Instant::now());
        speed as Speed
    }
}
