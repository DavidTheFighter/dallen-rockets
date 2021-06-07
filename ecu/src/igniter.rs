use crate::HALs;
use hal::{
    comms_hal::Packet,
    ecu_hal::{IgniterState, IgniterTimingConfig},
    Valve,
};

pub const DEFAULT_IGNITER_PREFIRE_DURATION_MS: u16 = 250;
pub const DEFAULT_IGNITER_FIRE_DURATION_MS: u16 = 1000;
pub const DEFAULT_IGNITER_PURGE_DURATION_MS: u16 = 2000;

pub struct Igniter {
    timing_config: IgniterTimingConfig,
    current_state_enum: IgniterState,
    elapsed_since_state_transition: f32,
}

impl Igniter {
    pub fn new() -> Igniter {
        Igniter {
            timing_config: IgniterTimingConfig {
                prefire_duration_ms: DEFAULT_IGNITER_PREFIRE_DURATION_MS,
                fire_duration_ms: DEFAULT_IGNITER_FIRE_DURATION_MS,
                purge_duration_ms: DEFAULT_IGNITER_PURGE_DURATION_MS,
            },
            current_state_enum: IgniterState::Idle,
            elapsed_since_state_transition: 0.0,
        }
    }

    pub(crate) fn update<'a>(&mut self, elapsed: f32, hals: &'a mut HALs) {
        self.elapsed_since_state_transition += elapsed;

        match self.current_state_enum {
            IgniterState::Idle => {}
            IgniterState::Prefire => self.update_prefire_state(hals),
            IgniterState::Firing => self.update_firing_state(hals),
            IgniterState::Purge => self.update_purging_state(hals),
        }
    }

    pub(crate) fn on_packet<'a>(&mut self, packet: &Packet, hals: &'a mut HALs) {
        match packet {
            Packet::FireIgniter => {
                if self.current_state_enum == IgniterState::Idle {
                    self.transition_state(IgniterState::Prefire, hals);
                }
            }
            Packet::ConfigureIgniterTiming(timing) => self.timing_config = timing.clone(),
            _ => {}
        }
    }

    pub(crate) fn on_abort<'a>(&mut self, hals: &'a mut HALs) {
        self.transition_state(IgniterState::Idle, hals);
    }

    pub fn get_current_state(&self) -> IgniterState {
        self.current_state_enum
    }

    fn transition_state<'a>(&mut self, new_state: IgniterState, hals: &'a mut HALs) {
        self.current_state_enum = new_state;
        self.elapsed_since_state_transition = 0.0;

        match new_state {
            IgniterState::Idle => self.enter_idle_state(hals),
            IgniterState::Prefire => self.enter_prefire_state(hals),
            IgniterState::Firing => self.enter_firing_state(hals),
            IgniterState::Purge => self.enter_purging_state(hals),
        }
    }

    // ----- Idle State ----- //

    #[allow(clippy::unused_self)]
    fn enter_idle_state(&mut self, hals: &mut HALs) {
        hals.hardware.set_sparking(false);
        hals.hardware.set_valve(Valve::IgniterFuelMain, 0);
        hals.hardware.set_valve(Valve::IgniterGOxMain, 0);
        hals.hardware.set_valve(Valve::IgniterFuelPurge, 0);
    }

    // ----- Prefire State ----- //

    #[allow(clippy::unused_self)]
    fn enter_prefire_state(&mut self, hals: &mut HALs) {
        hals.hardware.set_sparking(true);
        hals.hardware.set_valve(Valve::IgniterFuelMain, 0);
        hals.hardware.set_valve(Valve::IgniterGOxMain, 255);
        hals.hardware.set_valve(Valve::IgniterFuelPurge, 0);
    }

    fn update_prefire_state(&mut self, hals: &mut HALs) {
        if self.elapsed_since_state_transition
            > f32::from(self.timing_config.prefire_duration_ms) * 1e-3
        {
            self.transition_state(IgniterState::Firing, hals);
        }
    }

    // ----- Firing State ----- //

    #[allow(clippy::unused_self)]
    fn enter_firing_state(&mut self, hals: &mut HALs) {
        hals.hardware.set_sparking(true);
        hals.hardware.set_valve(Valve::IgniterFuelMain, 255);
        hals.hardware.set_valve(Valve::IgniterGOxMain, 255);
        hals.hardware.set_valve(Valve::IgniterFuelPurge, 0);
    }

    fn update_firing_state(&mut self, hals: &mut HALs) {
        if self.elapsed_since_state_transition
            > f32::from(self.timing_config.fire_duration_ms) * 1e-3
        {
            self.transition_state(IgniterState::Purge, hals);
        }
    }

    // ----- Purging State ----- //

    #[allow(clippy::unused_self)]
    fn enter_purging_state(&mut self, hals: &mut HALs) {
        hals.hardware.set_sparking(false);
        hals.hardware.set_valve(Valve::IgniterFuelMain, 0);
        hals.hardware.set_valve(Valve::IgniterGOxMain, 255);
        hals.hardware.set_valve(Valve::IgniterFuelPurge, 255);
    }

    fn update_purging_state(&mut self, hals: &mut HALs) {
        if self.elapsed_since_state_transition
            > f32::from(self.timing_config.purge_duration_ms) * 1e-3
        {
            self.transition_state(IgniterState::Idle, hals);
        }
    }
}
