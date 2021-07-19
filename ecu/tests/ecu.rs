use ecu::{Ecu, HALs};
use hal::{comms_mock::CommsMock, ecu_hal::{ECUHardware, ECUValve, ECU_VALVES}, ecu_mock::ECUHardwareMock};

macro_rules! hals {
    ($hardware:ident, $comms:ident) => {
        &mut HALs {
            hardware: &mut $hardware,
            comms: &mut $comms,
        }
    };
}

#[test]
fn test_valve_startup_states() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();

    // Set each valve to a known bad state before initializing the ECU
    // and then testing to make sure that the ECU set the valves to the
    // right state
    for valve in ECU_VALVES.iter() {
        ecu_hardware.set_valve(*valve, 128);
    }

    let _ecu = Ecu::new(0, hals!(ecu_hardware, comms));

    assert!(ecu_hardware.get_valve_states()[ECUValve::IgniterFuelMain as usize] == 0);
    assert!(ecu_hardware.get_valve_states()[ECUValve::IgniterGOxMain as usize] == 0);
    assert!(ecu_hardware.get_valve_states()[ECUValve::FuelPress as usize] == 0);
    assert!(ecu_hardware.get_valve_states()[ECUValve::FuelVent as usize] == 255);
}
