use ecu::{
    igniter::{
        DEFAULT_IGNITER_FIRE_DURATION_MS, DEFAULT_IGNITER_PREFIRE_DURATION_MS,
        DEFAULT_IGNITER_PURGE_DURATION_MS,
    },
    Ecu, HALs,
};

use hal::{
    comms_hal::Packet, comms_mock::CommsMock, ecu_hal::IgniterState, ecu_mock::ECUHardwareMock,
    Valve,
};

macro_rules! hals {
    ($hardware:ident, $comms:ident) => {
        &mut HALs {
            hardware: &mut $hardware,
            comms: &mut $comms,
        }
    };
}

#[test]
fn test_startup_state() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();
    let mut ecu = Ecu::new(0);

    assert_idle_state(&mut ecu, &mut ecu_hardware);

    for _ in 0..100 {
        ecu.update(hals!(ecu_hardware, comms), 0.01);

        assert_idle_state(&mut ecu, &mut ecu_hardware);
    }
}

#[test]
fn fire_igniter_test() {
    let mut ecu_hardware = ECUHardwareMock::new();
    let mut comms = CommsMock::new();
    let mut ecu = Ecu::new(0);

    assert_idle_state(&mut ecu, &mut ecu_hardware);

    ecu.on_packet(hals!(ecu_hardware, comms), &Packet::FireIgniter);
    assert_prefire_state(&mut ecu, &mut ecu_hardware);

    for _ in 0..DEFAULT_IGNITER_PREFIRE_DURATION_MS - 1 {
        ecu.update(hals!(ecu_hardware, comms), 0.001);
        assert_prefire_state(&mut ecu, &mut ecu_hardware);
    }

    ecu.update(hals!(ecu_hardware, comms), 0.001);
    assert_firing_state(&mut ecu, &mut ecu_hardware);

    for _ in 0..DEFAULT_IGNITER_FIRE_DURATION_MS {
        ecu.update(hals!(ecu_hardware, comms), 0.001);
        assert_firing_state(&mut ecu, &mut ecu_hardware);
    }

    ecu.update(hals!(ecu_hardware, comms), 0.001);
    assert_purging_state(&mut ecu, &mut ecu_hardware);

    for _ in 0..DEFAULT_IGNITER_PURGE_DURATION_MS - 1 {
        ecu.update(hals!(ecu_hardware, comms), 0.001);
        assert_purging_state(&mut ecu, &mut ecu_hardware);
    }

    for _ in 0..100 {
        ecu.update(hals!(ecu_hardware, comms), 0.001);

        assert_idle_state(&mut ecu, &mut ecu_hardware);
    }
}

fn assert_idle_state(ecu: &mut Ecu, ecu_hardware: &mut ECUHardwareMock) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Idle);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] == 0);
    assert!(ecu_hardware.sparking == false);
}

fn assert_prefire_state(ecu: &mut Ecu, ecu_hardware: &mut ECUHardwareMock) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Prefire);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] == 0);
    assert!(ecu_hardware.sparking == true);
}

fn assert_firing_state(ecu: &mut Ecu, ecu_hardware: &mut ECUHardwareMock) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Firing);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] == 0);
    assert!(ecu_hardware.sparking == true);
}

fn assert_purging_state(ecu: &mut Ecu, ecu_hardware: &mut ECUHardwareMock) {
    assert!(ecu.get_igniter().get_current_state() == IgniterState::Purge);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelMain as usize] == 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterGOxMain as usize] > 0);
    assert!(ecu_hardware.valve_states[Valve::IgniterFuelPurge as usize] > 0);
    assert!(ecu_hardware.sparking == false);
}
