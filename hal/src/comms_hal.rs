pub mod comms_canfd_hal;
pub mod comms_ethernet_hal;

use postcard::{
    flavors::{Cobs, Slice},
    from_bytes_cobs, serialize_with_flavor, Error,
};
use serde::{Deserialize, Serialize};

use crate::{
    ecu_hal::{ECUDataFrame, ECUSensor, ECUValve, IgniterTimingConfig},
    SensorConfig,
};

use self::comms_canfd_hal::CANFDTransferError;

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
    CANFDError(CANFDTransferError),
    Serialization(SerializationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationError {
    Unknown,
    PacketTooLong,
    PostcardImplementation,
    SerdeError,
    UnexpectedEnd,
    BadVar,
    BadEncoding,
}

pub trait CommsInterface {
    /// Attempts to transfer a packet to another controller on the network.
    ///
    /// # Errors
    /// If the transfer fails, it will return a `TransferError` describing what went wrong
    fn transmit(
        &mut self,
        packet: &Packet,
        address: NetworkAddress,
    ) -> nb::Result<(), TransferError>;

    /// Attempts to retrieve a packet from an internal FIFO buffer. If there are no incoming
    /// packets stored, then this method will return `None`. If there is a packet, then this
    /// will return a pair of that packet and the address it was sent from
    fn receive(&mut self) -> Option<(Packet, NetworkAddress)>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
}

impl Packet {
    pub fn serialize(&self, buffer: &mut [u8]) -> Result<usize, SerializationError> {
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
                Error::DeserializeUnexpectedEnd => Err(SerializationError::UnexpectedEnd),
                Error::DeserializeBadVarint
                | Error::DeserializeBadBool
                | Error::DeserializeBadChar
                | Error::DeserializeBadUtf8
                | Error::DeserializeBadOption
                | Error::DeserializeBadEnum => Err(SerializationError::BadVar),
                Error::DeserializeBadEncoding => Err(SerializationError::BadEncoding),
                _ => Err(SerializationError::Unknown),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ECUTelemtryData {
    pub ecu_data: ECUDataFrame,
    pub avg_loop_time: f32, // In seconds
    pub max_loop_time: f32, // In seconds
}
