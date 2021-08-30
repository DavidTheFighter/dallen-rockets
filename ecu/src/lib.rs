#![no_std]

pub mod igniter;
pub mod record;

#[macro_use]
extern crate lazy_static;

use hal::{
    comms_hal::{CommsInterface, ECUTelemtryData, NetworkAddress, Packet},
    ecu_hal::{ECUDataFrame, ECUHardware, ECUValve, MAX_ECU_SENSORS, MAX_ECU_VALVES},
};
use igniter::Igniter;
use record::DataRecorder;

pub const DEFAULT_TELEMETRY_RATE: f32 = 0.01;

pub const RECORD_RATE: f32 = 0.001;
pub const RECORD_TRANSMIT_RATE: f32 = 0.001;
pub const RECORD_STORAGE_SIZE: usize = 5000;

pub struct HALs<'a> {
    pub hardware: &'a mut dyn ECUHardware,
    pub comms: &'a mut dyn CommsInterface,
}

pub struct Ecu {
    igniter: Igniter,
    recorder: DataRecorder,
    elapsed_since_last_telemetry: f32,
    enginer_controller_index: u8,
    telemetry_rate: f32,
    max_loop_time_per_telem: f32,
    test_spark_elapsed: f32,
}

impl Ecu {
    pub fn new(enginer_controller_index: u8, hals: &mut HALs) -> Ecu {
        hals.hardware.set_valve(ECUValve::IgniterFuelMain, 0);
        hals.hardware.set_valve(ECUValve::IgniterGOxMain, 0);
        hals.hardware.set_valve(ECUValve::FuelPress, 0);
        hals.hardware.set_valve(ECUValve::FuelVent, 0);

        Ecu {
            igniter: Igniter::new(),
            recorder: DataRecorder::new(),
            elapsed_since_last_telemetry: 0.0,
            enginer_controller_index,
            telemetry_rate: DEFAULT_TELEMETRY_RATE,
            max_loop_time_per_telem: 0.0,
            test_spark_elapsed: -1.0,
        }
    }

    pub fn update(&mut self, hals: &mut HALs, elapsed: f32) {
        self.elapsed_since_last_telemetry += elapsed;
        self.max_loop_time_per_telem = self.max_loop_time_per_telem.max(elapsed);

        while self.elapsed_since_last_telemetry >= self.telemetry_rate {
            self.transmit_telemetry(hals);
            self.elapsed_since_last_telemetry -= self.telemetry_rate;
        }

        if self.test_spark_elapsed > -1e-3 {
            self.test_spark_elapsed -= elapsed;

            if self.test_spark_elapsed <= 1e-3 {
                hals.hardware.set_sparking(false);
                self.test_spark_elapsed = -1.0;
            }
        }

        loop {
            match hals.comms.receive() {
                Some((packet, _from)) => {
                    self.on_packet(hals, &packet);
                }
                None => break,
            }
        }

        self.igniter.update(elapsed, hals);
        self.record_update(elapsed, hals);
    }

    pub fn on_packet(&mut self, hals: &mut HALs, packet: &Packet) {
        match packet {
            Packet::SetValve { valve, state } => hals.hardware.set_valve(*valve, *state),
            Packet::SetSparking(sparking) => {
                self.test_spark_elapsed = *sparking;
                hals.hardware.set_sparking(true)
            }
            Packet::ConfigureSensor { sensor, config } => {
                hals.hardware.configure_sensor(*sensor, config)
            }
            Packet::Abort => self.abort(hals),
            Packet::SetRecording(recording) => self.set_recording(*recording),
            Packet::TransferData => self.transfer_recorded_data(),
            _ => {}
        }

        if !matches!(packet, Packet::Abort) {
            self.igniter.on_packet(packet, hals);
        }
    }

    pub fn abort(&mut self, hals: &mut HALs) {
        self.igniter.on_abort(hals);

        let packet = Packet::ControllerAborted(NetworkAddress::EngineController(
            self.enginer_controller_index,
        ));

        if let Err(err) = hals.comms.transmit(&packet, NetworkAddress::Broadcast) {
            log::error!("Failed to send abort message, got {:?}", err);
        }
    }

    pub fn get_igniter(&mut self) -> &mut Igniter {
        &mut self.igniter
    }

    fn transmit_telemetry(&mut self, hals: &mut HALs) {
        let telem = ECUTelemtryData {
            ecu_data: self.get_ecu_data_frame(hals),
            max_loop_time: self.max_loop_time_per_telem,
        };

        self.max_loop_time_per_telem = 0.0;

        if let Err(err) = hals.comms.transmit(
            &Packet::ECUTelemtry(telem.clone()),
            NetworkAddress::MissionControl,
        ) {
            log::error!("Failed to send packet, got {:?}", err);
        }
    }

    pub fn get_ecu_data_frame(&self, hals: &mut HALs) -> ECUDataFrame {
        let mut frame = ECUDataFrame {
            igniter_state: self.igniter.get_current_state(),
            valve_states: [69_u8; MAX_ECU_VALVES],
            sensor_states: [42_u16; MAX_ECU_SENSORS],
            sparking: hals.hardware.get_sparking(),
        };

        for (telem_valve, valve_state) in frame
            .valve_states
            .iter_mut()
            .zip(hals.hardware.get_valve_states().iter())
        {
            *telem_valve = *valve_state;
        }

        for (telem_sensor, sensor) in frame
            .sensor_states
            .iter_mut()
            .zip(hals.hardware.get_raw_sensor_readings().iter())
        {
            *telem_sensor = *sensor;
        }

        frame
    }
}
