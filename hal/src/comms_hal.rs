pub mod comms_canfd_hal;

use postcard::{
    flavors::{Cobs, Slice},
    from_bytes_cobs, serialize_with_flavor, Error,
};
use serde::{Deserialize, Serialize};

use crate::{
    ecu_hal::{ECUDataFrame, ECUSensor, ECUValve, IgniterTimingConfig},
    SensorConfig,
};

pub const MAX_SERIALIZE_LENGTH: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkAddress {
    Broadcast,
    EngineController(u8),
    MissionControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferError {
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationError {
    Unknown,
    PacketTooLong,
    PostcardImplementation,
    SerdeError,
    Corrupted,
}

pub trait CommsInterface {
    /// Attempts to transfer a packet to another controller on the network.
    ///
    /// # Errors
    /// If the transfer fails, it will return a `TransferError` describing what went wrong
    fn transmit(&mut self, packet: &Packet, address: NetworkAddress) -> Result<(), TransferError>;

    /// Attempts to retrieve a packet from an internal FIFO buffer. If there are no incoming
    /// packets stored, then this method will return `None`.
    fn receive(&mut self) -> Option<Packet>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Packet {
    // -- Commands -- //
    SetValve {
        valve: ECUValve,
        state: u8,
    },
    FireIgniter,
    ConfigureSensor {
        sensor: ECUSensor,
        config: SensorConfig,
    },
    ConfigureIgniterTiming(IgniterTimingConfig),
    Abort,

    // -- Telemetry -- //
    ECUTelemtry(ECUTelemtryData),
    ControllerAborted(NetworkAddress),

    // -- Data transfer -- //
    TransferDataLogs,
    ECUDataFrame(ECUDataFrame),
}

impl Packet {
    pub fn serialize(
        &self,
        buffer: &mut [u8; MAX_SERIALIZE_LENGTH],
    ) -> Result<usize, SerializationError> {
        match Cobs::try_new(Slice::new(buffer)) {
            Ok(flavor) => {
                let serialized =
                    serialize_with_flavor::<Packet, Cobs<Slice>, &mut [u8]>(&self, flavor);

                match serialized {
                    Ok(output_buffer) => Ok(output_buffer.len()),
                    Err(err) => match err {
                        Error::WontImplement | Error::NotYetImplemented => {
                            Err(SerializationError::PostcardImplementation)
                        }
                        Error::SerializeBufferFull | Error::SerializeSeqLengthUnknown => {
                            Err(SerializationError::PacketTooLong)
                        }
                        Error::SerdeSerCustom | Error::SerdeDeCustom => {
                            Err(SerializationError::SerdeError)
                        }
                        _ => Err(SerializationError::Unknown),
                    },
                }
            }
            Err(_err) => Err(SerializationError::Unknown),
        }
    }

    pub fn deserialize(buffer: &mut [u8]) -> Result<Packet, SerializationError> {
        match from_bytes_cobs(buffer) {
            Ok(packet) => Ok(packet),
            Err(err) => match err {
                Error::WontImplement | Error::NotYetImplemented => {
                    Err(SerializationError::PostcardImplementation)
                }
                Error::SerdeSerCustom | Error::SerdeDeCustom => Err(SerializationError::SerdeError),
                Error::DeserializeUnexpectedEnd
                | Error::DeserializeBadVarint
                | Error::DeserializeBadBool
                | Error::DeserializeBadChar
                | Error::DeserializeBadUtf8
                | Error::DeserializeBadOption
                | Error::DeserializeBadEnum
                | Error::DeserializeBadEncoding => Err(SerializationError::Corrupted),
                _ => Err(SerializationError::Unknown),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUTelemtryData {
    pub ecu_data: ECUDataFrame,
    pub avg_loop_time_ms: f32,
    pub max_loop_time_ms: f32,
}
