use embedded_hal::adc::OneShot;
use hal::{SensorConfig, ecu_hal::{ECUHardware, ECUSensor, ECUValve, ECU_SENSORS, ECU_VALVES, MAX_ECU_SENSORS, MAX_ECU_VALVES}};
use teensy4_bsp::{hal::{adc::{ADC, AnalogInput, ResolutionBits}, gpio::{Output, GPIO}, iomuxc::adc::ADC1}, t41};

pub struct Teensy41ECUHardware {
    sv1_pin: GPIO<t41::P2, Output>,
    sv2_pin: GPIO<t41::P3, Output>,
    sv3_pin: GPIO<t41::P4, Output>,
    sv4_pin: GPIO<t41::P5, Output>,
    sv5_pin: GPIO<t41::P6, Output>,
    _sv6_pin: GPIO<t41::P7, Output>,
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
        let mut inst = Teensy41ECUHardware {
            sv1_pin,
            sv2_pin,
            sv3_pin,
            sv4_pin,
            sv5_pin,
            _sv6_pin: sv6_pin,
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

            log::info!("{}: {}", index, raw_reading);
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

    pub fn flip_valves(&mut self) {
        let val = if self.valve_states[0] > 0 { 0 } else { 255 };
        
        for valve in &ECU_VALVES {
            self.set_valve(*valve, val);
        }

        log::info!("Setting all valves to {}", val);
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
            ECUValve::FuelPress => set_gpio!(self.sv1_pin, state > 0),
            ECUValve::FuelVent => set_gpio!(self.sv2_pin, state > 0),
            ECUValve::IgniterFuelMain => set_gpio!(self.sv3_pin, state > 0),
            ECUValve::IgniterGOxMain => set_gpio!(self.sv4_pin, state > 0),
            ECUValve::IgniterFuelPurge => set_gpio!(self.sv5_pin, state > 0),
        };

        self.valve_states[valve as usize] = state;
    }

    fn set_sparking(&mut self, state: bool) {
        set_gpio!(self.spark_pin, state);
        self.sparking = state;
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
