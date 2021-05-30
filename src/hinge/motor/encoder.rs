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
    static ref ENCODER_A: EncoderISR = unsafe { EncoderISR::<0,1>::from(1,2) };
    // static ref ENCODER_B: EncoderISR = unsafe { EncoderISR::from(3,4,(2,3)) };
    // static ref ENCODER_C: EncoderISR = unsafe { EncoderISR::from(30,31,(4,5)) };
}

type EncoderPin = gpio::Input<'static, AnyPin>;
pub struct EncoderISR<const C1: usize, const C2: usize> {
    inner: UnsafeCell<Rotary<EncoderPin, EncoderPin>>,
    dist: AtomicI16,
}


unsafe impl<const C1: usize, const C2: usize> Sync for EncoderISR<C1, C2> {}

impl<const C1: usize, const C2: usize> EncoderISR<C1, C2> {
    pub unsafe fn from(p0: u8, p1: u8) -> Self {
        let pin_0 = AnyPin::steal(p0);
        let pin_0 = Input::new(pin_0, Pull::None);
        interrupts::set_pin(p0, C1);
        let pin_1 = AnyPin::steal(p1);
        let pin_1 = Input::new(pin_1, Pull::None);
        interrupts::set_pin(p1, C2);
        
        Self {
            inner: UnsafeCell::new(Rotary::new(pin_0, pin_1)),
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

pub struct Encoder<const C1: usize, const C2: usize> {
    isr: &'static EncoderISR<C1, C2>,
    last_spd_update: Option<Instant>,
}

impl Encoder {
    const PERIOD: Duration = Duration::from_millis(1); // ms

    pub async fn wait(&mut self) -> (Distance, Speed) {
        let next = self.last_spd_update.unwrap_or(Instant::now())+Self::PERIOD;
        Timer::at(next).await;

        let distance = self.isr.dist.load(Ordering::Relaxed);
        defmt::info!("distance: {}", distance);
        let speed = self.update(distance);
        (distance, speed)
    }

    fn update(&mut self, distance: Distance) -> Speed {
        let speed = if let Some(t1) = self.last_spd_update {
            let distance = distance as i32;
            let elapsed = t1.elapsed().as_ticks() as i32;
            distance / elapsed
        } else {
            0
        };

        self.last_spd_update = Some(Instant::now());
        speed
    }
}
