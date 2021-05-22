use ecu::Ecu;
use hal::{comms_hal::Packet, comms_mock::CommsMock, ecu_hal::Valve, ecu_mock::ECUHardwareMock};

#[test]
fn test_data_logging() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();
    let mut ecu = Ecu::initialize(&mut ecu_hardware, &mut comms, 0);

    assert!(
        ecu.get_ecu_hardware()
            .any()
            .unwrap()
            .downcast_ref::<ECUHardwareMock>()
            .unwrap()
            .logging_data
            == false
    );
    ecu.on_packet(&Packet::BeginDataLogging);
    assert!(
        ecu.get_ecu_hardware()
            .any()
            .unwrap()
            .downcast_ref::<ECUHardwareMock>()
            .unwrap()
            .logging_data
            == true
    );
    ecu.on_packet(&Packet::EndDataLogging);
    assert!(
        ecu.get_ecu_hardware()
            .any()
            .unwrap()
            .downcast_ref::<ECUHardwareMock>()
            .unwrap()
            .logging_data
            == false
    );
}

#[test]
fn test_set_valve_packet() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();
    let mut ecu = Ecu::initialize(&mut ecu_hardware, &mut comms, 0);

    let valves = [
        Valve::FuelPress,
        Valve::FuelVent,
        Valve::IgniterFuelMain,
        Valve::IgniterGOxMain,
        Valve::IgniterFuelPurge,
    ];

    for valve in valves.iter() {
        for valve_state in &ecu
            .get_ecu_hardware()
            .any()
            .unwrap()
            .downcast_ref::<ECUHardwareMock>()
            .unwrap()
            .valve_states
        {
            assert!(*valve_state == 0);
        }

        ecu.on_packet(&Packet::SetValve {
            valve: *valve,
            state: 255,
        });

        for (index, valve_state) in ecu
            .get_ecu_hardware()
            .any()
            .unwrap()
            .downcast_ref::<ECUHardwareMock>()
            .unwrap()
            .valve_states
            .iter()
            .enumerate()
        {
            if index == *valve as usize {
                assert!(*valve_state == 255);
            } else {
                assert!(*valve_state == 0);
            }
        }

        ecu.on_packet(&Packet::SetValve {
            valve: *valve,
            state: 0,
        });

        for valve_state in ecu
            .get_ecu_hardware()
            .any()
            .unwrap()
            .downcast_ref::<ECUHardwareMock>()
            .unwrap()
            .valve_states
            .iter()
        {
            assert!(*valve_state == 0);
        }
    }
}
