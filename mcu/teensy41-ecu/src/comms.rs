use hal::comms_hal::{CommsInterface, NetworkAddress, Packet, TransferError};

#[allow(clippy::module_name_repetitions)]
pub struct Teensy41ECUComms {}

impl Teensy41ECUComms {
    pub fn new() -> Teensy41ECUComms {
        Teensy41ECUComms {}
    }
}

impl CommsInterface for Teensy41ECUComms {
    fn transmit(
        &mut self,
        _packet: &Packet,
        _address: NetworkAddress,
    ) -> Result<(), TransferError> {
        Ok(())
    }

    fn receive(&mut self) -> Option<Packet> {
        None
    }
}
