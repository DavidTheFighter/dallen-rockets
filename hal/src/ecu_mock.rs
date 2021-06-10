use crate::{
    ecu_hal::{ECUHardware, ECUSensor, ECUValve, MAX_ECU_SENSORS, MAX_ECU_VALVES},
    SensorConfig,
};

pub struct ECUHardwareMock {
    pub sparking: bool,
    pub valve_states: [u8; MAX_ECU_VALVES],
    pub sensor_readings: [u16; MAX_ECU_SENSORS],
    pub sensor_configs: [SensorConfig; MAX_ECU_SENSORS],
}

impl ECUHardwareMock {
    pub fn new() -> ECUHardwareMock {
        ECUHardwareMock {
            sparking: false,
            valve_states: [0_u8; MAX_ECU_VALVES],
            sensor_readings: [0_u16; MAX_ECU_SENSORS],
            sensor_configs: [SensorConfig {
                premin: 0.0,
                premax: 0.0,
                postmin: 0.0,
                postmax: 0.0,
            }; MAX_ECU_SENSORS],
        }
    }
}

impl ECUHardware for ECUHardwareMock {
    fn set_valve(&mut self, valve: ECUValve, state: u8) {
        self.valve_states[valve as usize] = state;
    }

    fn set_sparking(&mut self, state: bool) {
        self.sparking = state;
    }

    fn get_sensor_value(&self, sensor: ECUSensor) -> f32 {
        let config = self.sensor_configs[sensor as usize];
        let reading = f32::from(self.sensor_readings[sensor as usize]);
        let normalized = (reading - config.premin) / (config.premax - config.premin);

        normalized * (config.postmax - config.postmin) + config.postmin
    }

    fn get_raw_sensor_readings(&self) -> &[u16] {
        &self.sensor_readings
    }

    fn get_valve_states(&self) -> &[u8] {
        &self.valve_states
    }

    fn get_sparking(&self) -> bool {
        self.sparking
    }

    fn configure_sensor(&mut self, sensor: ECUSensor, config: &SensorConfig) {
        self.sensor_configs[sensor as usize] = *config;
    }
}
