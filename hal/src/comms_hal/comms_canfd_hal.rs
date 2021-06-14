use super::{NetworkAddress, MAX_SERIALIZE_LENGTH};

impl NetworkAddress {
    pub fn to_standard_id(self) -> u32 {
        match self {
            NetworkAddress::Broadcast => 0,
            NetworkAddress::EngineController(index) => 21 + u32::from(index),
            NetworkAddress::MissionControl => 1,
        }
    }

    pub fn from_id(id: u32) -> Option<NetworkAddress> {
        match id {
            0 => Some(NetworkAddress::Broadcast),
            1 => Some(NetworkAddress::MissionControl),
            _ => {
                if (21..=31).contains(&id) {
                    Some(NetworkAddress::EngineController(u32_to_u8(id - 21)))
                } else {
                    None
                }
            }
        }
    }
}

pub struct LengthTooLongError;

pub struct CANFDPacketMetadata {
    /// Bitfield mapping:
    /// [0,5]     - True data length, u8 in range [0, 60]
    /// [6, 31]   - Reserved
    bitfield: u32,
}

impl CANFDPacketMetadata {
    pub fn new(bitfield: u32) -> CANFDPacketMetadata {
        CANFDPacketMetadata { bitfield }
    }

    pub fn get_bitfield(&self) -> u32 {
        self.bitfield
    }

    pub fn set_true_data_length(&mut self, len: usize) -> Result<(), LengthTooLongError> {
        if (0..=MAX_SERIALIZE_LENGTH).contains(&len) {
            self.bitfield &= !MetadatBitfieldField::TrueDataLength.mask(); // Clear bits
            self.bitfield |= usize_to_u32(len) << MetadatBitfieldField::TrueDataLength.shift(); // Set bits

            Ok(())
        } else {
            Err(LengthTooLongError {})
        }
    }

    pub fn get_true_data_length(&self) -> usize {
        ((self.bitfield & MetadatBitfieldField::TrueDataLength.mask())
            >> MetadatBitfieldField::TrueDataLength.shift()) as usize
    }
}

enum MetadatBitfieldField {
    TrueDataLength,
}

impl MetadatBitfieldField {
    pub fn mask(&self) -> u32 {
        match self {
            MetadatBitfieldField::TrueDataLength => 0x1F,
        }
    }

    pub fn shift(&self) -> u32 {
        match self {
            MetadatBitfieldField::TrueDataLength => 0,
        }
    }
}

#[allow(clippy::cast_possible_truncation)]
fn u32_to_u8(val: u32) -> u8 {
    (val & 0xFF) as u8
}

#[allow(clippy::cast_possible_truncation)]
fn usize_to_u32(val: usize) -> u32 {
    (val & 0xFFFF_FFFF) as u32
}
