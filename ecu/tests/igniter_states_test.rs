use ecu::{
    igniter::{
        DEFAULT_IGNITER_FIRE_DURATION_MS, DEFAULT_IGNITER_PREFIRE_DURATION_MS,
        DEFAULT_IGNITER_PURGE_DURATION_MS,
    },
    Ecu,
};
use hal::{
    comms_hal::Packet,
    comms_mock::CommsMock,
    ecu_hal::{IgniterState, Valve},
    ecu_mock::ECUHardwareMock,
};

#[test]
fn test_startup_state() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();
    let mut ecu = Ecu::initialize(&mut ecu_hardware, &mut comms, 0);

    assert_idle_state(&mut ecu);

    for _ in 0..100 {
        ecu.update(0.01);

        assert_idle_state(&mut ecu);
    }
}

#[test]
fn fire_igniter_test() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();
    let mut ecu = Ecu::initialize(&mut ecu_hardware, &mut comms, 0);

    assert_idle_state(&mut ecu);

    ecu.on_packet(&Packet::FireIgniter);
    assert_prefire_state(&mut ecu);

    for _ in 0..DEFAULT_IGNITER_PREFIRE_DURATION_MS - 1 {
        ecu.update(0.001);
        assert_prefire_state(&mut ecu);
    }

    ecu.update(0.001);
    assert_firing_state(&mut ecu);

    for _ in 0..DEFAULT_IGNITER_FIRE_DURATION_MS {
        ecu.update(0.001);
        assert_firing_state(&mut ecu);
    }

    ecu.update(0.001);
    assert_purging_state(&mut ecu);

    for _ in 0..DEFAULT_IGNITER_PURGE_DURATION_MS - 1 {
        ecu.update(0.001);
        assert_purging_state(&mut ecu);
    }

    for _ in 0..100 {
        ecu.update(0.001);

        assert_idle_state(&mut ecu);
    }
}

fn assert_idle_state(ecu: &mut Ecu) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Idle);

    let ecu_hardware: &ECUHardwareMock = ecu
        .get_ecu_hardware()
        .any()
        .unwrap()
        .downcast_ref()
        .unwrap();
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] == 0);
    assert!(ecu_hardware.sparking == false);
}

fn assert_prefire_state(ecu: &mut Ecu) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Prefire);

    let ecu_hardware: &ECUHardwareMock = ecu
        .get_ecu_hardware()
        .any()
        .unwrap()
        .downcast_ref()
        .unwrap();
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] == 0);
    assert!(ecu_hardware.sparking == true);
}

fn assert_firing_state(ecu: &mut Ecu) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Firing);

    let ecu_hardware: &ECUHardwareMock = ecu
        .get_ecu_hardware()
        .any()
        .unwrap()
        .downcast_ref()
        .unwrap();
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] == 0);
    assert!(ecu_hardware.sparking == true);
}

fn assert_purging_state(ecu: &mut Ecu) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Purge);

    let ecu_hardware: &ECUHardwareMock = ecu
        .get_ecu_hardware()
        .any()
        .unwrap()
        .downcast_ref()
        .unwrap();
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] > 0);
    assert!(ecu_hardware.sparking == false);
}
