use bincode::Encode;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Copy, Encode, Deserialize)]
pub struct MotorData {
    speed: f64,
}

impl fmt::Display for MotorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Speed: {}", self.speed)
    }
}

#[derive(Clone, Copy, Encode, Deserialize)]
pub struct SteeringData {
    steer: f64, // -1.0 G | 0M | D 1.0
}

impl fmt::Display for SteeringData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Steering: {}", self.steer)
    }
}

#[derive(Clone, Copy, Encode, Deserialize)]
pub struct ActuatorData {
    motor: MotorData,
    steering: SteeringData,
}

impl fmt::Display for ActuatorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.motor, self.steering)
    }
}

pub struct Motor {}
impl Motor {
    pub fn empty() -> MotorData {
        MotorData {
            speed: 0.0,
        }
    }
}

pub struct Steering {}
impl Steering {
    pub fn empty() -> SteeringData {
        SteeringData {
            steer: 0.0,
        }
    }
}

pub struct Actuator {}
impl Actuator {
    pub fn empty() -> ActuatorData {
        ActuatorData {
            motor: Motor::empty(),
            steering: Steering::empty(),
        }
    }
}