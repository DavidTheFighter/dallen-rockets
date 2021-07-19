use core::time;
use std::{
    io::ErrorKind,
    net::UdpSocket,
    sync::{atomic::Ordering, mpsc::Sender},
};

use hal::comms_hal::comms_ethernet_hal::ETHERNET_BUFFER_SIZE;
use hal::comms_hal::{comms_ethernet_hal, Packet};

use crate::RUNNING;

pub enum RecvOutput {
    Packet(Packet),
    CETPulse,
}

pub fn packet_loop(recv_thread_tx: Sender<RecvOutput>) {
    let socket = UdpSocket::bind("0.0.0.0:25565").unwrap();
    socket
        .set_read_timeout(Some(time::Duration::from_millis(10)))
        .unwrap();

    let mut buffer = [0_u8; ETHERNET_BUFFER_SIZE];
    while RUNNING.load(Ordering::Relaxed) {
        match socket.recv_from(&mut buffer) {
            Ok((_packet_size, _addr)) => {
                if buffer[0] == 255 && buffer[2] == 255 {
                    recv_thread_tx.send(RecvOutput::CETPulse).unwrap();
                } else {
                    let packet = comms_ethernet_hal::deserialize_packet(&mut buffer);

                    if packet.is_ok() {
                        recv_thread_tx
                            .send(RecvOutput::Packet(packet.unwrap().0))
                            .unwrap();
                    }
                }
            }
            Err(err) => {
                if err.kind() != ErrorKind::WouldBlock && err.kind() != ErrorKind::TimedOut {
                    panic!("{:?}", err);
                }
            }
        }
    }
}
