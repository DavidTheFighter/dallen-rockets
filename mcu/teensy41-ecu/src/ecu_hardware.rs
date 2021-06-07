use embedded_hal::adc::OneShot;
use hal::{
    ecu_hal::{ECUHardware, ECU_SENSORS},
    Sensor, SensorConfig, Valve, MAX_SENSORS, MAX_VALVES,
};
use teensy4_bsp::{
    hal::{
        adc::{AnalogInput, ADC},
        gpio::{Output, GPIO},
        iomuxc::adc::ADC1,
    },
    t41,
};

pub struct Teensy41ECUHardware {
    sv1_pin: GPIO<t41::P2, Output>,
    sv2_pin: GPIO<t41::P3, Output>,
    sv3_pin: GPIO<t41::P4, Output>,
    sv4_pin: GPIO<t41::P5, Output>,
    sv5_pin: GPIO<t41::P6, Output>,
    sv6_pin: GPIO<t41::P7, Output>,
    spark_pin: GPIO<t41::P8, Output>,
    t1_pin: AnalogInput<ADC1, t41::P23>,
    t2_pin: AnalogInput<ADC1, t41::P22>,
    t3_pin: AnalogInput<ADC1, t41::P41>,
    t4_pin: AnalogInput<ADC1, t41::P40>,
    p1_pin: AnalogInput<ADC1, t41::P21>,
    p2_pin: AnalogInput<ADC1, t41::P20>,
    p3_pin: AnalogInput<ADC1, t41::P19>,
    p4_pin: AnalogInput<ADC1, t41::P18>,
    p5_pin: AnalogInput<ADC1, t41::P17>,
    p6_pin: AnalogInput<ADC1, t41::P16>,
    p7_pin: AnalogInput<ADC1, t41::P15>,
    p8_pin: AnalogInput<ADC1, t41::P14>,
    adc1: ADC<ADC1>,
    sensor_configs: [SensorConfig; MAX_SENSORS],
    valve_states: [u8; MAX_VALVES],
    raw_sensor_readings: [u16; MAX_SENSORS],
    sensor_readings: [f32; MAX_SENSORS],
    sparking: bool,
}

impl Teensy41ECUHardware {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sv1_pin: GPIO<t41::P2, Output>,
        sv2_pin: GPIO<t41::P3, Output>,
        sv3_pin: GPIO<t41::P4, Output>,
        sv4_pin: GPIO<t41::P5, Output>,
        sv5_pin: GPIO<t41::P6, Output>,
        sv6_pin: GPIO<t41::P7, Output>,
        spark_pin: GPIO<t41::P8, Output>,
        t1_pin: AnalogInput<ADC1, t41::P23>,
        t2_pin: AnalogInput<ADC1, t41::P22>,
        t3_pin: AnalogInput<ADC1, t41::P41>,
        t4_pin: AnalogInput<ADC1, t41::P40>,
        p1_pin: AnalogInput<ADC1, t41::P21>,
        p2_pin: AnalogInput<ADC1, t41::P20>,
        p3_pin: AnalogInput<ADC1, t41::P19>,
        p4_pin: AnalogInput<ADC1, t41::P18>,
        p5_pin: AnalogInput<ADC1, t41::P17>,
        p6_pin: AnalogInput<ADC1, t41::P16>,
        p7_pin: AnalogInput<ADC1, t41::P15>,
        p8_pin: AnalogInput<ADC1, t41::P14>,
        adc1: ADC<ADC1>,
    ) -> Teensy41ECUHardware {
        Teensy41ECUHardware {
            sv1_pin,
            sv2_pin,
            sv3_pin,
            sv4_pin,
            sv5_pin,
            sv6_pin,
            spark_pin,
            t1_pin,
            t2_pin,
            t3_pin,
            t4_pin,
            p1_pin,
            p2_pin,
            p3_pin,
            p4_pin,
            p5_pin,
            p6_pin,
            p7_pin,
            p8_pin,
            adc1,
            sensor_configs: [SensorConfig::default(); MAX_SENSORS],
            valve_states: [0_u8; MAX_VALVES],
            raw_sensor_readings: [0_u16; MAX_SENSORS],
            sensor_readings: [0.0_f32; MAX_SENSORS],
            sparking: false,
        }
    }

    pub fn read_sensors(&mut self) {
        for (index, ecu_sensor) in ECU_SENSORS.iter().enumerate() {
            self.raw_sensor_readings[index] = self.read_sensor(*ecu_sensor);
        }
    }

    fn read_sensor(&mut self, sensor: Sensor) -> u16 {
        match sensor {
            Sensor::IgniterThroatTemp => self.adc1.read(&mut self.t1_pin).unwrap(),
            Sensor::IgniterFuelInjectorPressure => self.adc1.read(&mut self.p1_pin).unwrap(),
            Sensor::IgniterGOxInjectorPressure => self.adc1.read(&mut self.p2_pin).unwrap(),
            Sensor::IgniterChamberPressure => self.adc1.read(&mut self.p3_pin).unwrap(),
            Sensor::FuelTankPressure => self.adc1.read(&mut self.p4_pin).unwrap(),
        }
    }
}

macro_rules! set_gpio {
    ($pin:expr, $state:expr) => {
        if $state {
            $pin.set();
        } else {
            $pin.clear();
        }
    };
}

impl ECUHardware for Teensy41ECUHardware {
    fn set_valve(&mut self, valve: Valve, state: u8) {
        match valve {
            Valve::FuelPress => set_gpio!(self.sv1_pin, state > 0),
            Valve::FuelVent => set_gpio!(self.sv2_pin, state > 0),
            Valve::IgniterFuelMain => set_gpio!(self.sv3_pin, state > 0),
            Valve::IgniterGOxMain => set_gpio!(self.sv4_pin, state > 0),
            Valve::IgniterFuelPurge => set_gpio!(self.sv5_pin, state > 0),
        };

        self.valve_states[valve as usize] = state;
    }

    fn set_sparking(&mut self, state: bool) {
        set_gpio!(self.spark_pin, state);
    }

    fn get_sensor_value(&self, sensor: Sensor) -> f32 {
        self.sensor_readings[sensor as usize]
    }

    fn get_raw_sensor_readings(&self) -> &[u16] {
        &self.raw_sensor_readings[0..5]
    }

    fn get_valve_states(&self) -> &[u8] {
        &self.valve_states[0..5]
    }

    fn get_sparking(&self) -> bool {
        self.sparking
    }

    fn configure_sensor(&mut self, sensor: Sensor, config: &SensorConfig) {
        self.sensor_configs[sensor as usize] = *config;
    }
}
