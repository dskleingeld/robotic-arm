type Pin = u8;
type PwmPin = u8;

pub struct MotorConfig {
    pub encoder_fdw: Pin,
    pub encoder_back: Pin,
    pub power_fwd: PwmPin,
    pub power_back: PwmPin,
}

pub struct Motor {
    pins: MotorConfig,
    relative_pos: u16, // degrees
}

impl Motor {
    pub fn from(cfg: MotorConfig) -> Self {
        Self {
            pins: cfg,
            relative_pos: 0,
        }
    }
    pub fn spin() {
    }
}
