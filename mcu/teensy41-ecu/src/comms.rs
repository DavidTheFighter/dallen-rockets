mod receive;

use cortex_m::interrupt;
use nb::Error::{Other, WouldBlock};

use hal::comms_hal::comms_canfd_hal::{deserialize_packet, serialize_packet, CANFDTransferError};
use hal::comms_hal::{CommsInterface, NetworkAddress, Packet, TransferError, MAX_SERIALIZE_LENGTH};
use teensy4_canfd::can_error::RxTxError;
use teensy4_canfd::{
    config::{Clock, Config, Id, MailboxConfig, RegionConfig, RxMailboxConfig, TimingConfig},
    CANFDBuilder, TxFDFrame, CAN3FD,
};

use self::receive::pop_rx_frame;

#[allow(clippy::module_name_repetitions)]
pub struct Teensy41ECUComms {
    host_address: NetworkAddress,
    canfd: CAN3FD,
}

impl Teensy41ECUComms {
    pub fn new(ecu_index: u8) -> Teensy41ECUComms {
        let host_addr_id = NetworkAddress::EngineController(ecu_index).to_standard_id();

        let host_rx32_mb_config = RxMailboxConfig {
            id: Id::Standard(host_addr_id),
            id_mask: 0b000_0011_1111, // 11-bit ID, 7-11 is src ID, 6 is 32/64-byte, 0-5 are dst ID
        };

        let broadcast_rx32_mb_config = RxMailboxConfig {
            id: Id::Standard(NetworkAddress::Broadcast.to_standard_id()),
            id_mask: 0b000_0011_1111, // 11-bit ID, 7-11 is src ID, 6 is 32/64-byte, 0-5 are dst ID
        };

        let host_rx64_mb_config = RxMailboxConfig {
            id: Id::Standard(host_addr_id + (1 << 5)),
            id_mask: 0b000_0011_1111, // 11-bit ID, 7-11 is src ID, 6 is 32/64-byte, 0-5 are dst ID
        };

        let broadcast_rx64_mb_config = RxMailboxConfig {
            id: Id::Standard(NetworkAddress::Broadcast.to_standard_id() + (1 << 5)),
            id_mask: 0b000_0011_1111, // 11-bit ID, 7-11 is src ID, 6 is 32/64-byte, 0-5 are dst ID
        };

        let region_1_config = RegionConfig::MB32 {
            mailbox_configs: [
                // Host receive mailboxes
                MailboxConfig::Rx {
                    rx_config: host_rx32_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: host_rx32_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: host_rx32_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: host_rx32_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: host_rx32_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: host_rx32_mb_config,
                },
                // Broadcast receive mailboxes
                MailboxConfig::Rx {
                    rx_config: broadcast_rx32_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: broadcast_rx32_mb_config,
                },
                // Outgoing/transfer mailboxes
                MailboxConfig::Tx,
                MailboxConfig::Tx,
                MailboxConfig::Tx,
                MailboxConfig::Tx,
            ],
        };

        let region_2_config = RegionConfig::MB64 {
            mailbox_configs: [
                // Host receive mailboxes
                MailboxConfig::Rx {
                    rx_config: host_rx64_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: host_rx64_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: host_rx64_mb_config,
                },
                // Broadcast receive mailboxes
                MailboxConfig::Rx {
                    rx_config: broadcast_rx64_mb_config,
                },
                MailboxConfig::Rx {
                    rx_config: broadcast_rx64_mb_config,
                },
                // Outgoing/transfer mailboxes
                MailboxConfig::Tx,
                MailboxConfig::Tx,
            ],
        };

        let canfd_config = Config {
            clock_speed: Clock::Clock30Mhz,
            timing_classical: TimingConfig {
                prescalar_division: 1,
                prop_seg: 13,
                phase_seg_1: 3,
                phase_seg_2: 3,
                jump_width: 3,
            },
            timing_fd: TimingConfig {
                prescalar_division: 1,
                prop_seg: 0,
                phase_seg_1: 3,
                phase_seg_2: 2,
                jump_width: 2,
            },
            region_1_config,
            region_2_config,
            transceiver_compensation: Some(3),
        };

        let mut canfd = CANFDBuilder::take().unwrap().build(canfd_config).unwrap();

        interrupt::free(|cs| {
            canfd.set_rx_callback(cs, Some(receive::on_canfd_frame));
        });

        Teensy41ECUComms {
            host_address: NetworkAddress::EngineController(ecu_index),
            canfd,
        }
    }
}

impl CommsInterface for Teensy41ECUComms {
    fn transmit(
        &mut self,
        packet: &Packet,
        address: NetworkAddress,
    ) -> nb::Result<(), TransferError> {
        let mut buffer = [0_u8; MAX_SERIALIZE_LENGTH + 4];

        match serialize_packet(packet, &mut buffer) {
            Ok(data_len) => {
                let tx_frame = TxFDFrame {
                    id: self.get_outgoing_id(address, data_len),
                    buffer: &buffer,
                    priority: None,
                };

                let transfer_result = interrupt::free(|cs| {
                    self.canfd.transfer_nb(cs, &tx_frame)
                });

                match transfer_result {
                    Ok(_) => Ok(()),
                    Err(err) => match err {
                        RxTxError::MailboxUnavailable => Err(WouldBlock),
                        RxTxError::FrameTooBigForRegions => Err(Other(TransferError::CANFDError(
                            CANFDTransferError::FrameTooBigForRegion,
                        ))),
                        RxTxError::Unknown => Err(Other(TransferError::CANFDError(
                            CANFDTransferError::Unknown,
                        ))),
                    },
                }
            }
            Err(err) => Err(Other(err)),
        }
    }

    fn receive(&mut self) -> Option<(Packet, NetworkAddress)> {
        let rx_frame = interrupt::free(|cs| pop_rx_frame(cs));

        match rx_frame {
            Some(mut rx_frame) => {
                let packet = deserialize_packet(&mut rx_frame.buffer);
                let from_id = Teensy41ECUComms::get_incoming_id(rx_frame.id);

                match (packet, from_id) {
                    (Ok(packet), Some(from_addr)) => Some((packet, from_addr)),
                    _ => None, // TODO Throw some kind of error somehow
                }
            }
            None => None,
        }
    }
}

impl Teensy41ECUComms {
    fn get_outgoing_id(&self, target: NetworkAddress, data_len: usize) -> Id {
        let mb_select: u32 = if data_len > 32 { 0b1 } else { 0b0 };

        Id::Standard(
            (target.to_standard_id() as u32)
                + (mb_select << 5)
                + ((self.host_address.to_standard_id() as u32) << 6),
        )
    }

    fn get_incoming_id(id: Id) -> Option<NetworkAddress> {
        NetworkAddress::from_id(match id {
            Id::Standard(val) => val & 0x1F,
            Id::Extended(_val) => 0,
        })
    }
}
