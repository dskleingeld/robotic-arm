use embassy::time::{Duration, Instant, Timer};
use nrf52832_hal::gpio;
use rotary_encoder_hal::Rotary;

pub type Speed = i32;
pub type Distance = i8;

type EncoderPin = gpio::Pin<gpio::Input<gpio::PullUp>>;
pub struct Encoder {
    inner: Rotary<EncoderPin, EncoderPin>,
    last_updated: Option<Instant>,
}

impl Encoder {
    pub fn from<S>(pin_0: gpio::Pin<S>, pin_1: gpio::Pin<S>) -> Self {
        let pin_0 = pin_0.into_pullup_input();
        let pin_1 = pin_1.into_pullup_input();
        Self {
            inner: Rotary::new(pin_0, pin_1),
            last_updated: None,
        }
    }
}

impl Encoder {
    const PERIOD: Duration = Duration::from_millis(10); // ms

    pub async fn wait(&mut self) -> (Distance, Speed) {
        use rotary_encoder_hal::Direction::*;

        let dur = self.last_updated
            .map(|t| t.elapsed())
            .unwrap_or(Self::PERIOD);
        Timer::after(dur).await;
        let distance = loop {
            match self.inner.update() {
                Err(_) => defmt::panic!("encoder had problem with pins"),
                Ok(None) => continue,
                Ok(Clockwise) => break 1,
                Ok(CounterClockwise) => break -1,
            }
        };

        let speed = self.update(distance);
        (distance, speed)
    }

    fn update(&mut self, distance: Distance) -> Speed {
        let speed = if let Some(t1) = self.last_updated {
            let distance = distance as i32;
            let elapsed = t1.elapsed().as_millis() as i32;
            distance / elapsed
        } else {
            0
        };

        self.last_updated = Some(Instant::now());
        speed
    }
}
