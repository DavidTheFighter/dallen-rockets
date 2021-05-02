use crate::comms_hal::{CommsInterface, NetworkAddress, Packet, TransferError};

pub struct CommsMock {}

impl CommsMock {
    pub fn new() -> CommsMock {
        CommsMock {}
    }
}

impl CommsInterface for CommsMock {
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
