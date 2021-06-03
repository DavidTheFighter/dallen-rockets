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

pub struct HALs<'a> {
    pub hardware: &'a mut dyn ECUHardware,
    pub comms: &'a mut dyn CommsInterface,
}

pub struct Ecu {
    igniter: Igniter,
    elapsed_since_last_telemetry: f32,
    enginer_controller_index: u8,
    telemetry_rate: f32,
}

impl Ecu {
    pub fn new(enginer_controller_index: u8) -> Ecu {
        Ecu {
            igniter: Igniter::new(),
            elapsed_since_last_telemetry: 0.0,
            enginer_controller_index,
            telemetry_rate: DEFAULT_TELEMETRY_RATE,
        }
    }

    pub fn update(&mut self, hals: &mut HALs, elapsed: f32) {
        self.elapsed_since_last_telemetry += elapsed;

        if self.elapsed_since_last_telemetry > self.telemetry_rate {
            self.transmit_telemetry(hals);
            self.elapsed_since_last_telemetry -= self.telemetry_rate;
        }

        self.igniter.update(elapsed, hals);
    }

    pub fn on_packet(&mut self, hals: &mut HALs, packet: &Packet) {
        match packet {
            Packet::SetValve { valve, state } => hals.hardware.set_valve(*valve, *state),
            Packet::ConfigureSensor { sensor, config } => {
                hals.hardware.configure_sensor(*sensor, config)
            }
            Packet::Abort => self.abort(hals),
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
        if let Err(_err) = hals.comms.transmit(&packet, NetworkAddress::Broadcast) {
            //something
        }
    }

    pub fn get_igniter(&mut self) -> &mut Igniter {
        &mut self.igniter
    }

    fn transmit_telemetry(&mut self, hals: &mut HALs) {
        let mut telem = ECUTelemtryData {
            ecu_data: ECUDataFrame {
                time: 0.0,
                igniter_state: self.igniter.get_current_state(),
                valve_states: [0_u8; MAX_VALVES],
                sensor_states: [0_u16; MAX_SENSORS],
            },
            avg_loop_time_ms: 0.0,
            max_loop_time_ms: 0.0,
        };

        for (telem_valve, valve_state) in telem
            .ecu_data
            .valve_states
            .iter_mut()
            .zip(hals.hardware.get_valve_states().iter())
        {
            *telem_valve = *valve_state;
        }

        for (telem_sensor, sensor) in telem
            .ecu_data
            .sensor_states
            .iter_mut()
            .zip(hals.hardware.get_raw_sensor_readings().iter())
        {
            *telem_sensor = *sensor;
        }

        if let Err(_err) = hals
            .comms
            .transmit(&Packet::ECUTelemtry(telem), NetworkAddress::MissionControl)
        {
            // something
        }
    }
}
