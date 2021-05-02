use crate::{
    ecu_hal::{ECUDataFrame, ECUHardware, Sensor, SensorConfig, Valve},
    MAX_SENSORS, MAX_VALVES,
};

pub struct ECUMock {
    pub sparking: bool,
    pub valve_states: [u8; MAX_VALVES],
    pub sensor_readings: [u16; MAX_SENSORS],
    pub sensor_configs: [SensorConfig; MAX_SENSORS],
    pub logging_data: bool,
}

impl ECUHardware for ECUMock {
    fn set_valve(&mut self, valve: Valve, state: u8) {
        self.valve_states[valve as usize] = state;
    }

    fn set_sparking(&mut self, state: bool) {
        self.sparking = state;
    }

    fn get_sensor_value(&self, _sensor: Sensor) -> f32 {
        0.0
    }

    fn get_raw_sensor_readings(&self) -> &[u16] {
        &self.sensor_readings
    }

    fn get_valve_states(&self) -> &[u8] {
        &self.valve_states
    }

    fn configure_sensor(&mut self, config: &SensorConfig) {
        self.sensor_configs[config.sensor as usize] = config.clone();
    }

    fn begin_data_logging(&mut self) {
        self.logging_data = true;
    }

    fn end_data_logging(&mut self) {
        self.logging_data = false;
    }

    fn get_next_recorded_data_frame(&mut self) -> Option<ECUDataFrame> {
        None
    }

    fn get_data_collection_rate_hz(&self) -> u16 {
        1000
    }
}
