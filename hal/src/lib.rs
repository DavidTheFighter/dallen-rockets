#![forbid(unsafe_code)]
#![no_std]

pub const MAX_SERIALIZE_LENGTH: usize = 64;
pub const MAX_SENSORS: usize = 12;
pub const MAX_VALVES: usize = 6;

pub mod comms_hal;
pub mod comms_mock;
pub mod ecu_hal;
pub mod ecu_mock;
