use core::any::Any;

use crate::{MAX_SENSORS, MAX_VALVES};

#[derive(Debug, Clone, Copy)]
pub struct SensorConfig {
    pub premin: f32,
    pub premax: f32,
    pub postmin: f32,
    pub postmax: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgniterState {
    Idle,
    Prefire,
    Firing,
    Purge,
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

pub trait ECUHardware {
    /// Opens/closes a valve to a particular state. For solenoid valves, 0 is closed and >= 1 is
    /// open. For any other kind of valve, 0 means fully closed and 255 means fully open. Any
    /// value in between means a *linear* increase in open valve area.
    fn set_valve(&mut self, valve: Valve, state: u8);
    fn set_sparking(&mut self, state: bool);

    fn get_sensor_value(&self, sensor: Sensor) -> f32;
    fn get_raw_sensor_readings(&self) -> &[u16];
    fn get_valve_states(&self) -> &[u8];

    fn configure_sensor(&mut self, sensor: Sensor, config: &SensorConfig);
    fn begin_data_logging(&mut self);
    fn end_data_logging(&mut self);
    fn get_next_recorded_data_frame(&mut self) -> Option<ECUDataFrame>;
    fn get_data_collection_rate_hz(&self) -> u16;

    fn any(&self) -> &dyn Any;
}

#[derive(Debug, Clone)]
pub struct ECUDataFrame {
    pub time: f32,
    pub igniter_state: IgniterState,
    pub valve_states: [u8; MAX_VALVES],
    pub sensor_states: [u16; MAX_SENSORS],
}

#[derive(Debug, Clone)]
pub struct IgniterTimingConfig {
    pub prefire_duration_ms: u16,
    pub fire_duration_ms: u16,
    pub purge_duration_ms: u16,
}
