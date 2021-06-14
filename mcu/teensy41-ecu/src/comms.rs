mod transfer;

use hal::comms_hal::{CommsInterface, NetworkAddress, Packet, TransferError};
use teensy4_canfd::{
    config::{Clock, Config, Id, MailboxConfig, RegionConfig, RxMailboxConfig, TimingConfig},
    CANFDBuilder,
};

#[allow(clippy::module_name_repetitions)]
pub struct Teensy41ECUComms {}

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

        let _canfd = CANFDBuilder::take().unwrap().build(canfd_config);

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
