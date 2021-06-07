use crate::{Sensor, SensorConfig, Valve, MAX_SENSORS, MAX_VALVES};

pub const MAX_ECU_SENSORS: usize = 5;

pub const ECU_SENSORS: [Sensor; MAX_ECU_SENSORS] = [
    Sensor::IgniterThroatTemp,
    Sensor::IgniterFuelInjectorPressure,
    Sensor::IgniterGOxInjectorPressure,
    Sensor::IgniterChamberPressure,
    Sensor::FuelTankPressure,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    fn set_valve(&mut self, valve: Valve, state: u8);
    fn set_sparking(&mut self, state: bool);

    fn get_sensor_value(&self, sensor: Sensor) -> f32;
    fn get_raw_sensor_readings(&self) -> &[u16];
    fn get_valve_states(&self) -> &[u8];
    fn get_sparking(&self) -> bool;

    fn configure_sensor(&mut self, sensor: Sensor, config: &SensorConfig);
}

#[derive(Debug, Clone)]
pub struct ECUDataFrame {
    pub time: f32,
    pub igniter_state: IgniterState,
    pub valve_states: [u8; MAX_VALVES],
    pub sensor_states: [u16; MAX_SENSORS],
    pub sparking: bool,
}

#[derive(Debug, Clone)]
pub struct IgniterTimingConfig {
    pub prefire_duration_ms: u16,
    pub fire_duration_ms: u16,
    pub purge_duration_ms: u16,
}
