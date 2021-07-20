use super::{NetworkAddress, Packet, SerializationError, TransferError, MAX_SERIALIZE_LENGTH};

pub const ETHERNET_BUFFER_SIZE: usize = MAX_SERIALIZE_LENGTH + 6;

pub fn serialize_packet(
    packet: &Packet,
    from_address: NetworkAddress,
    to_address: NetworkAddress,
    buffer: &mut [u8; ETHERNET_BUFFER_SIZE],
) -> Result<usize, TransferError> {
    for (to, from) in buffer
        .iter_mut()
        .zip(from_address.to_standard_id().to_le_bytes().iter())
        .take(2)
    {
        *to = *from;
    }

    for (to, from) in buffer
        .iter_mut()
        .skip(2)
        .zip(to_address.to_standard_id().to_le_bytes().iter())
        .take(2)
    {
        *to = *from;
    }

    match packet.serialize(&mut buffer[6..]) {
        Ok(len) => {
            for (to, from) in buffer
                .iter_mut()
                .skip(4)
                .zip(len.to_le_bytes().iter())
                .take(2)
            {
                *to = *from;
            }

            Ok(len + 6)
        }
        Err(err) => Err(TransferError::Serialization(err)),
    }
}

pub fn deserialize_packet(
    buffer: &mut [u8; ETHERNET_BUFFER_SIZE],
) -> Result<(Packet, NetworkAddress, NetworkAddress), SerializationError> {
    let from_address = address_from_slice(&buffer[0..2]);
    let to_address = address_from_slice(&buffer[2..4]);
    let len = usize::from(buffer[4]) + (usize::from(buffer[5]) << 8);

    match (from_address, to_address) {
        (Some(from_address), Some(to_address)) => {
            match Packet::deserialize(&mut buffer[6..6 + len]) {
                Ok(packet) => Ok((packet, from_address, to_address)),
                Err(err) => Err(err),
            }
        }
        _ => Err(SerializationError::BadVar),
    }
}

fn address_from_slice(buffer: &[u8]) -> Option<NetworkAddress> {
    if buffer.len() < 2 {
        return None;
    }

    let value = u32::from(buffer[0]) + (u32::from(buffer[1]) << 8);

    NetworkAddress::from_id(value)
}
