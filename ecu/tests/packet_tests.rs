use ecu::{Ecu, HALs};
use hal::{comms_hal::Packet, comms_mock::CommsMock, ecu_hal::ECUValve, ecu_mock::ECUHardwareMock};

macro_rules! hals {
    ($hardware:ident, $comms:ident) => {
        &mut HALs {
            hardware: &mut $hardware,
            comms: &mut $comms,
        }
    };
}

#[test]
fn test_set_valve_packet() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();
    let mut ecu = Ecu::new(0);

    let valves = [
        ECUValve::FuelPress,
        ECUValve::FuelVent,
        ECUValve::IgniterFuelMain,
        ECUValve::IgniterGOxMain,
        ECUValve::IgniterFuelPurge,
    ];

    for valve in valves.iter() {
        for valve_state in &ecu_hardware.valve_states {
            assert!(*valve_state == 0);
        }

        ecu.on_packet(
            hals!(ecu_hardware, comms),
            &Packet::SetValve {
                valve: *valve,
                state: 255,
            },
        );

        for (index, valve_state) in ecu_hardware.valve_states.iter().enumerate() {
            if index == *valve as usize {
                assert!(*valve_state == 255);
            } else {
                assert!(*valve_state == 0);
            }
        }

        ecu.on_packet(
            hals!(ecu_hardware, comms),
            &Packet::SetValve {
                valve: *valve,
                state: 0,
            },
        );

        for valve_state in ecu_hardware.valve_states.iter() {
            assert!(*valve_state == 0);
        }
    }
}
