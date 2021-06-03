use crate::ecu_hal::{ECUDataFrame, IgniterTimingConfig, Sensor, SensorConfig, Valve};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub enum Packet {
    // -- Commands -- //
    SetValve {
        valve: Valve,
        state: u8,
    },
    FireIgniter,
    ConfigureSensor {
        sensor: Sensor,
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

#[derive(Debug, Clone)]
pub struct ECUTelemtryData {
    pub ecu_data: ECUDataFrame,
    pub avg_loop_time_ms: f32,
    pub max_loop_time_ms: f32,
}
