use crate::hinge::MotorConfig;

pub const LOWERMOTOR: MotorConfig = MotorConfig {
    encoder_fdw: 1,
    encoder_back: 2,
    power_fwd: 3,
    power_back: 4,
};

pub const UPPERMOTOR: MotorConfig = LOWERMOTOR;
pub const GRAPPERMOTOR: MotorConfig = LOWERMOTOR;
