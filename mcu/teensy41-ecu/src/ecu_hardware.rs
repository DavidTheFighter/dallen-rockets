use embedded_hal::adc::OneShot;
use hal::{
    ecu_hal::{ECUHardware, ECUSensor, ECUValve, ECU_SENSORS, MAX_ECU_SENSORS, MAX_ECU_VALVES},
    SensorConfig,
};
use teensy4_bsp::{hal::{adc::{AnalogInput, ResolutionBits, ADC}, gpio::{Output, GPIO}, iomuxc::adc::ADC1}, t41};

pub struct Teensy41ECUHardware {
    sv1_pin: GPIO<t41::P2, Output>,
    sv2_pin: GPIO<t41::P3, Output>,
    sv3_pin: GPIO<t41::P4, Output>,
    sv4_pin: GPIO<t41::P6, Output>,
    spark_pin: GPIO<t41::P8, Output>,
    t1_pin: AnalogInput<ADC1, t41::P23>,
    _t2_pin: AnalogInput<ADC1, t41::P22>,
    _t3_pin: AnalogInput<ADC1, t41::P41>,
    _t4_pin: AnalogInput<ADC1, t41::P40>,
    p1_pin: AnalogInput<ADC1, t41::P21>,
    p2_pin: AnalogInput<ADC1, t41::P20>,
    p3_pin: AnalogInput<ADC1, t41::P19>,
    p4_pin: AnalogInput<ADC1, t41::P18>,
    _p5_pin: AnalogInput<ADC1, t41::P17>,
    _p6_pin: AnalogInput<ADC1, t41::P16>,
    _p7_pin: AnalogInput<ADC1, t41::P15>,
    _p8_pin: AnalogInput<ADC1, t41::P14>,
    adc1: ADC<ADC1>,
    sensor_configs: [SensorConfig; MAX_ECU_SENSORS],
    valve_states: [u8; MAX_ECU_VALVES],
    raw_sensor_readings: [u16; MAX_ECU_SENSORS],
    sensor_readings: [f32; MAX_ECU_SENSORS],
    sparking: bool,
    time_since_last_spark: f32,
    spark_frequency: f32,
    spark_duty_cycle: f32,
}

impl Teensy41ECUHardware {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sv1_pin: GPIO<t41::P2, Output>,
        sv2_pin: GPIO<t41::P3, Output>,
        sv3_pin: GPIO<t41::P4, Output>,
        sv4_pin: GPIO<t41::P6, Output>,
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
        let mut inst = Teensy41ECUHardware {
            sv1_pin,
            sv2_pin,
            sv3_pin,
            sv4_pin,
            spark_pin,
            t1_pin,
            _t2_pin: t2_pin,
            _t3_pin: t3_pin,
            _t4_pin: t4_pin,
            p1_pin,
            p2_pin,
            p3_pin,
            p4_pin,
            _p5_pin: p5_pin,
            _p6_pin: p6_pin,
            _p7_pin: p7_pin,
            _p8_pin: p8_pin,
            adc1,
            sensor_configs: [SensorConfig::default(); MAX_ECU_SENSORS],
            valve_states: [0_u8; MAX_ECU_VALVES],
            raw_sensor_readings: [0_u16; MAX_ECU_SENSORS],
            sensor_readings: [0.0_f32; MAX_ECU_SENSORS],
            sparking: false,
            time_since_last_spark: 0.0,
            spark_frequency: 0.01,
            spark_duty_cycle: 0.5,
        };

        inst.adc1.set_resolution(ResolutionBits::Res12);

        inst
    }

    pub fn read_sensors(&mut self) {
        for (index, ecu_sensor) in ECU_SENSORS.iter().enumerate() {
            let raw_reading = self.read_sensor(*ecu_sensor);
            let config = &self.sensor_configs[index];
            let reading = (f32::from(raw_reading) - config.premin)
                / (config.premax - config.premin)
                * (config.postmax - config.postmin)
                + config.postmin;

            self.raw_sensor_readings[index] = raw_reading;
            self.sensor_readings[index] = reading;
        }
    }

    pub fn update_spark(&mut self, elapsed: f32) {
        if self.sparking {
            self.time_since_last_spark += elapsed;

            if self.time_since_last_spark > self.spark_frequency {
                self.spark_pin.set();
                self.time_since_last_spark -= self.spark_frequency;
            } else if self.time_since_last_spark > self.spark_frequency * self.spark_duty_cycle {
                self.spark_pin.clear();
            }
        }
    }

    fn read_sensor(&mut self, sensor: ECUSensor) -> u16 {
        match sensor {
            ECUSensor::IgniterThroatTemp => self.adc1.read(&mut self.t1_pin).unwrap(),
            ECUSensor::IgniterFuelInjectorPressure => self.adc1.read(&mut self.p1_pin).unwrap(),
            ECUSensor::IgniterGOxInjectorPressure => self.adc1.read(&mut self.p2_pin).unwrap(),
            ECUSensor::IgniterChamberPressure => self.adc1.read(&mut self.p3_pin).unwrap(),
            ECUSensor::FuelTankPressure => self.adc1.read(&mut self.p4_pin).unwrap(),
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
    fn set_valve(&mut self, valve: ECUValve, state: u8) {
        match valve {
            ECUValve::IgniterFuelMain => set_gpio!(self.sv1_pin, state > 0),
            ECUValve::IgniterGOxMain => set_gpio!(self.sv2_pin, state > 0),
            ECUValve::FuelPress => set_gpio!(self.sv3_pin, state > 0),
            ECUValve::FuelVent => set_gpio!(self.sv4_pin, state > 0),
        };

        self.valve_states[valve as usize] = state;
    }

    fn set_sparking(&mut self, state: bool) {
        self.sparking = state;
        self.time_since_last_spark = 0.0;

        set_gpio!(self.spark_pin, state);
    }

    fn get_sensor_value(&self, sensor: ECUSensor) -> f32 {
        self.sensor_readings[sensor as usize]
    }

    fn get_raw_sensor_readings(&self) -> &[u16] {
        &self.raw_sensor_readings
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
