use hal::{
    comms_hal::{
        comms_canfd_hal::{self, CANFDPacketMetadata, CANFD_BUFFER_SIZE},
        comms_ethernet_hal::{self, ETHERNET_BUFFER_SIZE},
        ECUTelemtryData, NetworkAddress, Packet, MAX_SERIALIZE_LENGTH,
    },
    ecu_hal::{
        ECUDataFrame, ECUSensor, ECUValve, IgniterTimingConfig, MAX_ECU_SENSORS, MAX_ECU_VALVES,
    },
    SensorConfig,
};

#[test]
fn serialize_packet_size_check() {
    let mut buffer = [0_u8; 1024];

    for packet in get_all_packets().iter() {
        let len = packet.serialize(&mut buffer).unwrap();
        println!("{:?} is length {}", packet, len);
        assert!((1..MAX_SERIALIZE_LENGTH).contains(&len));
    }
}

#[test]
fn serialize_deserialize_eq() {
    let mut buffer = [0_u8; MAX_SERIALIZE_LENGTH];

    for packet in get_all_packets().iter() {
        let len = packet.serialize(&mut buffer).unwrap();
        let other_packet = Packet::deserialize(&mut buffer[0..len]).unwrap();

        println!("Before: {:?}", *packet);
        println!("After {:?}\n", other_packet);

        assert!(*packet == other_packet);
    }
}

#[test]
fn canfd_serialize_deserialize_eq() {
    let mut buffer = [0_u8; CANFD_BUFFER_SIZE];

    for packet in get_all_packets().iter() {
        println!("Before: {:?}", *packet);

        let len = comms_canfd_hal::serialize_packet(packet, &mut buffer).unwrap();
        let metadata = CANFDPacketMetadata::from_byte_slice(&buffer);

        assert!((len - 4) == metadata.get_true_data_length());

        let other_packet = comms_canfd_hal::deserialize_packet(&mut buffer).unwrap();

        println!("After {:?}\n", other_packet);

        assert!(*packet == other_packet);
    }
}

#[test]
fn ethernet_serialize_deserialize_eq() {
    let mut buffer = [0_u8; ETHERNET_BUFFER_SIZE];

    for packet in get_all_packets().iter() {
        let from_address = NetworkAddress::MissionControl;
        let to_address = NetworkAddress::EngineController(4);

        println!("Before: {:?}", *packet);
        println!("\t{:?} -> {:?}\n", from_address, to_address);

        comms_ethernet_hal::serialize_packet(packet, from_address, to_address, &mut buffer)
            .unwrap();

        let (des_packet, des_from, des_to) =
            comms_ethernet_hal::deserialize_packet(&mut buffer).unwrap();

        println!("After {:?}", des_packet);
        println!("\t{:?} -> {:?}\n", des_from, des_to);

        assert!(*packet == des_packet);
        assert!(from_address == des_from);
        assert!(to_address == des_to);
    }
}

fn get_all_packets() -> [Packet; 11] {
    let set_valve = Packet::SetValve {
        valve: ECUValve::FuelPress,
        state: 42,
    };
    let set_sparking = Packet::SetSparking(0.42);
    let fire_igniter = Packet::FireIgniter;
    let configure_sensor = Packet::ConfigureSensor {
        sensor: ECUSensor::FuelTankPressure,
        config: SensorConfig {
            premin: 42.69,
            premax: 420.42,
            postmin: 69.420,
            postmax: 42.42,
        },
    };
    let configure_igniter_timing = Packet::ConfigureIgniterTiming(IgniterTimingConfig {
        prefire_duration_ms: 42,
        fire_duration_ms: 420,
    });
    let abort = Packet::Abort;
    let ecu_telemetry = Packet::ECUTelemtry(ECUTelemtryData {
        ecu_data: ECUDataFrame {
            igniter_state: hal::ecu_hal::IgniterState::Idle,
            sensor_states: [69_u16; MAX_ECU_SENSORS],
            valve_states: [42_u8; MAX_ECU_VALVES],
            sparking: false,
        },
        max_loop_time: 69.96,
    });
    let controller_aborted = Packet::ControllerAborted(NetworkAddress::MissionControl);
    let set_recording = Packet::SetRecording(true);
    let transfer_data = Packet::TransferData;
    let recorded_data = Packet::RecordedData(ECUDataFrame {
        igniter_state: hal::ecu_hal::IgniterState::Idle,
        sensor_states: [69_u16; MAX_ECU_SENSORS],
        valve_states: [42_u8; MAX_ECU_VALVES],
        sparking: false,
    });

    // Remember to create a packet so its serialization/deserialization can be tested
    match set_valve {
        Packet::SetValve { .. } => println!("test"),
        Packet::SetSparking(_) => println!("test"),
        Packet::FireIgniter => println!("test"),
        Packet::ConfigureSensor { .. } => println!("test"),
        Packet::ConfigureIgniterTiming(_) => println!("test"),
        Packet::Abort => println!("test"),
        Packet::ECUTelemtry(_) => println!("test"),
        Packet::ControllerAborted(_) => println!("test"),
        Packet::SetRecording(_) => println!("test"),
        Packet::TransferData => println!("test"),
        Packet::RecordedData(_) => println!("test"),
    }

    [
        set_valve,
        set_sparking,
        fire_igniter,
        configure_sensor,
        configure_igniter_timing,
        abort,
        ecu_telemetry,
        controller_aborted,
        set_recording,
        transfer_data,
        recorded_data,
    ]
}
