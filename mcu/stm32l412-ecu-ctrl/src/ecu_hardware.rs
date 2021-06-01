use hal::ecu_hal::{ECUDataFrame, ECUHardware, Sensor, SensorConfig, Valve};

use crate::STM32L412ECUController;

impl<'a> ECUHardware for STM32L412ECUController<'a> {
    fn set_valve(&mut self, valve: Valve, state: u8) {
        let valve = match valve {
            Valve::FuelPress => &mut self.sv1_pin,
            Valve::FuelVent => &mut self.sv2_pin,
            Valve::IgniterFuelMain => &mut self.sv3_pin,
            Valve::IgniterGOxMain => &mut self.sv4_pin,
            Valve::IgniterFuelPurge => &mut self.sv5_pin,
        };

        // Both of these results are `Infallible` so unwrapping will never panic
        if state > 0 {
            valve.set_high().unwrap();
        } else {
            valve.set_low().unwrap();
        }
    }

    fn set_sparking(&mut self, state: bool) {
        // Both of these results are `Infallible` so unwrapping will never panic
        if state {
            self.spark_plug_pin.set_high().unwrap();
        } else {
            self.spark_plug_pin.set_low().unwrap();
        }
    }

    fn get_sensor_value(&self, sensor: Sensor) -> f32 {
        self.sensor_values[sensor as usize]
    }

    fn get_raw_sensor_readings(&self) -> &[u16] {
        &self.raw_sensor_values
    }

    fn get_valve_states(&self) -> &[u8] {
        &self.valve_states
    }

    fn configure_sensor(&mut self, sensor: Sensor, config: &SensorConfig) {
        self.sensor_configs[sensor as usize] = *config;
    }

    fn begin_data_logging(&mut self) {
        todo!()
    }

    fn end_data_logging(&mut self) {
        todo!()
    }

    fn get_next_recorded_data_frame(&mut self) -> Option<ECUDataFrame> {
        todo!()
    }

    fn get_data_collection_rate_hz(&self) -> u16 {
        todo!()
    }
}
