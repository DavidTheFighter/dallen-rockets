use serde::{Deserialize, Serialize};

use crate::SensorConfig;

pub const MAX_ECU_SENSORS: usize = 5;
pub const MAX_ECU_VALVES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ECUSensor {
    IgniterThroatTemp = 0,
    IgniterFuelInjectorPressure = 1,
    IgniterGOxInjectorPressure = 2,
    IgniterChamberPressure = 3,
    FuelTankPressure = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ECUValve {
    IgniterFuelMain = 0,
    IgniterGOxMain = 1,
    IgniterFuelPurge = 2,
    FuelPress = 3,
    FuelVent = 4,
}

pub const ECU_SENSORS: [ECUSensor; MAX_ECU_SENSORS] = [
    ECUSensor::IgniterThroatTemp,
    ECUSensor::IgniterFuelInjectorPressure,
    ECUSensor::IgniterGOxInjectorPressure,
    ECUSensor::IgniterChamberPressure,
    ECUSensor::FuelTankPressure,
];

pub const ECU_VALVES: [ECUValve; MAX_ECU_VALVES] = [
    ECUValve::IgniterFuelMain,
    ECUValve::IgniterGOxMain,
    ECUValve::IgniterFuelPurge,
    ECUValve::FuelPress,
    ECUValve::FuelVent,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgniterState {
    Idle,
    Prefire,
    Firing,
    Purge,
}

pub trait ECUHardware {
    /// Opens/closes a valve to a particular state. For solenoid valves, 0 is closed and >= 1 is
    /// open. For any other kind of valve, 0 means fully closed and 255 means fully open. Any
    /// value in between means a *linear* increase in open valve area.
    fn set_valve(&mut self, valve: ECUValve, state: u8);
    fn set_sparking(&mut self, state: bool);

    fn get_sensor_value(&self, sensor: ECUSensor) -> f32;
    fn get_raw_sensor_readings(&self) -> &[u16];
    fn get_valve_states(&self) -> &[u8];
    fn get_sparking(&self) -> bool;

    fn configure_sensor(&mut self, sensor: ECUSensor, config: &SensorConfig);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUDataFrame {
    pub time: f32,
    pub igniter_state: IgniterState,
    pub valve_states: [u8; MAX_ECU_VALVES],
    pub sensor_states: [u16; MAX_ECU_SENSORS],
    pub sparking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgniterTimingConfig {
    pub prefire_duration_ms: u16,
    pub fire_duration_ms: u16,
    pub purge_duration_ms: u16,
}
