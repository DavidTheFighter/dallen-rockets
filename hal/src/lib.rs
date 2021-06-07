#![forbid(unsafe_code)]
#![no_std]

pub mod comms_hal;
pub mod comms_mock;
pub mod ecu_hal;
pub mod ecu_mock;

pub const MAX_SENSORS: usize = 12;
pub const MAX_VALVES: usize = 6;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Valve {
    FuelPress = 0,
    FuelVent = 1,
    IgniterFuelMain = 2,
    IgniterGOxMain = 3,
    IgniterFuelPurge = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sensor {
    IgniterThroatTemp = 0,
    IgniterFuelInjectorPressure = 1,
    IgniterGOxInjectorPressure = 2,
    IgniterChamberPressure = 3,
    FuelTankPressure = 4,
}
