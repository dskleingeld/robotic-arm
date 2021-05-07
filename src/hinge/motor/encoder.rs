use embassy::time::{Duration, Instant, Timer};
use nrf52832_hal::gpio;
use rotary_encoder_hal::Rotary;

pub type Speed = i32;
pub type Distance = i8;

type EncoderPin = gpio::Pin<gpio::Input<gpio::Floating>>;
pub struct Encoder {
    inner: Rotary<EncoderPin, EncoderPin>,
    last_spd_update: Option<Instant>,
    last_pin_update: Instant,
}

impl Encoder {
    pub fn from<S>(pin_0: gpio::Pin<S>, pin_1: gpio::Pin<S>) -> Self {
        let pin_0 = pin_0.into_floating_input();
        let pin_1 = pin_1.into_floating_input();
        Self {
            inner: Rotary::new(pin_0, pin_1),
            last_pin_update: Instant::now(),
            last_spd_update: None,
        }
    }
}

impl Encoder {
    const PERIOD: Duration = Duration::from_millis(1); // ms

    pub async fn wait(&mut self) -> (Distance, Speed) {
        use rotary_encoder_hal::Direction::*;

        // defmt::info!("since last pin update: {}", 
            // self.last_pin_update.elapsed().as_millis());

        let distance = loop {
            let dur = self.last_pin_update.elapsed();
            let dt = Self::PERIOD
                .checked_sub(dur)
                .unwrap_or(Duration::from_millis(0));
            Timer::after(dt).await;
            self.last_pin_update = Instant::now();

            match self.inner.update() {
                Err(_) => defmt::panic!("encoder had problem with pins"),
                Ok(None) => continue,
                Ok(Clockwise) => break 1,
                Ok(CounterClockwise) => break -1,
            }
        };

    defmt::info!("distance: {}", distance);
        let speed = self.update(distance);
        (distance, speed)
    }

    fn update(&mut self, distance: Distance) -> Speed {
        let speed = if let Some(t1) = self.last_spd_update {
            let distance = distance as i32;
            let elapsed = t1.elapsed().as_millis() as i32;
            distance / elapsed
        } else {
            0
        };

        self.last_spd_update = Some(Instant::now());
        speed
    }
}
