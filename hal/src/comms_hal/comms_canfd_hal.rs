use super::{NetworkAddress, Packet, SerializationError, TransferError, MAX_SERIALIZE_LENGTH};
use serde::{Deserialize, Serialize};

pub const CANFD_BUFFER_SIZE: usize = MAX_SERIALIZE_LENGTH + 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CANFDTransferError {
    MetadataSerialization,
    FrameTooBigForRegion,
    Unknown,
}

pub struct LengthTooLongError;
pub struct LengthTooShortError;

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

pub fn serialize_packet(
    packet: &Packet,
    buffer: &mut [u8; CANFD_BUFFER_SIZE],
) -> Result<usize, TransferError> {
    match packet.serialize(&mut buffer[4..]) {
        Ok(len) => {
            let mut metadata = CANFDPacketMetadata::new();
            if metadata.set_true_data_length(len).is_err() {
                return Err(TransferError::Serialization(
                    SerializationError::PacketTooLong,
                ));
            }

            if metadata.serialize_to_buffer(&mut buffer[0..4]).is_err() {
                return Err(TransferError::CANFDError(
                    CANFDTransferError::MetadataSerialization,
                ));
            }

            Ok(len + 4)
        }
        Err(err) => Err(TransferError::Serialization(err)),
    }
}

pub fn deserialize_packet(
    buffer: &mut [u8; CANFD_BUFFER_SIZE],
) -> Result<Packet, SerializationError> {
    let metadata = CANFDPacketMetadata::from_byte_slice(buffer);

    Packet::deserialize(&mut buffer[4..(metadata.get_true_data_length() + 4)])
}

pub struct CANFDPacketMetadata {
    /// Bitfield mapping:
    /// [0,5]     - True data length, u8 in range [0, 60]
    /// [6, 31]   - Reserved
    bitfield: u32,
}

impl CANFDPacketMetadata {
    pub fn new() -> CANFDPacketMetadata {
        CANFDPacketMetadata { bitfield: 0 }
    }

    pub fn from_byte_slice(buffer: &[u8]) -> CANFDPacketMetadata {
        let mut inst = Self { bitfield: 0 };
        for (index, byte) in buffer.iter().enumerate().take(4) {
            inst.bitfield |= u32::from(*byte) << (8 * usize_to_u32(index));
        }

        inst
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

    pub fn serialize_to_buffer(&self, buffer: &mut [u8]) -> Result<(), LengthTooShortError> {
        if buffer.len() < 4 {
            Err(LengthTooShortError {})
        } else {
            let bytes = self.bitfield.to_le_bytes();
            for (to, from) in buffer.iter_mut().zip(bytes.iter()) {
                *to = *from;
            }

            Ok(())
        }
    }
}

enum MetadatBitfieldField {
    TrueDataLength,
}

impl MetadatBitfieldField {
    fn mask(&self) -> u32 {
        match self {
            MetadatBitfieldField::TrueDataLength => 0x3F,
        }
    }

    fn shift(&self) -> u32 {
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
