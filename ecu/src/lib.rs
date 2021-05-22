#![forbid(unsafe_code)]
#![no_std]

use hal::{
    comms_hal::{CommsInterface, ECUTelemtryData, NetworkAddress, Packet},
    ecu_hal::{ECUDataFrame, ECUHardware},
    MAX_SENSORS, MAX_VALVES,
};
use igniter::Igniter;

pub const DEFAULT_TELEMETRY_RATE: f32 = 0.01;

pub mod igniter;

pub(crate) struct HALs<'a> {
    pub hardware: &'a mut dyn ECUHardware,
    pub comms: &'a mut dyn CommsInterface,
}

pub struct Ecu<'a> {
    hals: HALs<'a>,
    igniter: Igniter,
    elapsed_since_last_telemetry: f32,
    enginer_controller_index: u8,
    telemetry_rate: f32,
}

impl<'a> Ecu<'a> {
    pub fn initialize<'b>(
        hardware: &'b mut dyn ECUHardware,
        comms: &'b mut dyn CommsInterface,
        enginer_controller_index: u8,
    ) -> Ecu<'b> {
        Ecu {
            hals: HALs { hardware, comms },
            igniter: Igniter::new(),
            elapsed_since_last_telemetry: 0.0,
            enginer_controller_index,
            telemetry_rate: DEFAULT_TELEMETRY_RATE,
        }
    }

    pub fn update(&mut self, elapsed: f32) {
        self.elapsed_since_last_telemetry += elapsed;

        if self.elapsed_since_last_telemetry > self.telemetry_rate {
            self.transmit_telemetry();
            self.elapsed_since_last_telemetry -= self.telemetry_rate;
        }

        self.igniter.update(elapsed, &mut self.hals);
    }

    pub fn on_packet(&mut self, packet: &Packet) {
        match packet {
            Packet::SetValve { valve, state } => self.hals.hardware.set_valve(*valve, *state),
            Packet::ConfigureSensor { sensor, config } => self.hals.hardware.configure_sensor(*sensor, config),
            Packet::BeginDataLogging => self.hals.hardware.begin_data_logging(),
            Packet::EndDataLogging => self.hals.hardware.end_data_logging(),
            Packet::Abort => self.abort(),
            _ => {}
        }

        if !matches!(packet, Packet::Abort) {
            self.igniter.on_packet(packet, &mut self.hals);
        }
    }

    pub fn abort(&mut self) {
        let packet = Packet::ControllerAborted(NetworkAddress::EngineController(
            self.enginer_controller_index,
        ));
        if let Err(_err) = self.hals.comms.transmit(&packet, NetworkAddress::Broadcast) {
            //something
        }

        self.igniter.abort(&mut self.hals);
    }

    pub fn get_igniter(&mut self) -> &mut Igniter {
        &mut self.igniter
    }

    pub fn get_ecu_hardware(&mut self) -> &mut dyn ECUHardware {
        self.hals.hardware
    }

    pub fn get_comms(&mut self) -> &mut dyn CommsInterface {
        self.hals.comms
    }

    fn transmit_telemetry(&mut self) {
        let mut telem = ECUTelemtryData {
            ecu_data: ECUDataFrame {
                time: 0.0,
                igniter_state: self.igniter.get_current_state(),
                valve_states: [0_u8; MAX_VALVES],
                sensor_states: [0_u16; MAX_SENSORS],
            },
            avg_loop_time_ms: 0.0,
            max_loop_time_ms: 0.0,
            data_collection_rate_hz: self.hals.hardware.get_data_collection_rate_hz(),
        };

        for (telem_valve, valve_state) in telem
            .ecu_data
            .valve_states
            .iter_mut()
            .zip(self.hals.hardware.get_valve_states().iter())
        {
            *telem_valve = *valve_state;
        }

        for (telem_sensor, sensor) in telem
            .ecu_data
            .sensor_states
            .iter_mut()
            .zip(self.hals.hardware.get_raw_sensor_readings().iter())
        {
            *telem_sensor = *sensor;
        }

        if let Err(_err) = self
            .hals
            .comms
            .transmit(&Packet::ECUTelemtry(telem), NetworkAddress::MissionControl)
        {
            // something
        }
    }
}
