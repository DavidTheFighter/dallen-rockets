#![forbid(unsafe_code)]
#![no_std]

pub mod comms_hal;
pub mod comms_mock;
pub mod ecu_hal;
pub mod ecu_mock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SensorConfig {
    pub premin: f32,
    pub premax: f32,
    pub postmin: f32,
    pub postmax: f32,
}

impl Default for SensorConfig {
    fn default() -> SensorConfig {
        SensorConfig {
            premin: 0.0,
            premax: 1.0,
            postmin: 0.0,
            postmax: 1.0,
        }
    }
}
