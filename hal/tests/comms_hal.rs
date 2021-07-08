use hal::{
    comms_hal::{
        comms_canfd_hal::{self, CANFDPacketMetadata},
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
    let mut buffer = [0_u8; MAX_SERIALIZE_LENGTH + 4];

    for packet in get_all_packets().iter() {
        println!("Before: {:?}", *packet);

        let len = comms_canfd_hal::serialize_packet(packet, &mut buffer).unwrap();
        let metadata = CANFDPacketMetadata::from_byte_slice(&buffer);

        assert!(len == metadata.get_true_data_length());

        let other_packet = comms_canfd_hal::deserialize_packet(&mut buffer).unwrap();

        println!("After {:?}\n", other_packet);

        assert!(*packet == other_packet);
    }
}

fn get_all_packets() -> [Packet; 7] {
    let set_valve = Packet::SetValve {
        valve: ECUValve::FuelPress,
        state: 42,
    };
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
        purge_duration_ms: 49,
    });
    let abort = Packet::Abort;
    let ecu_telemetry = Packet::ECUTelemtry(ECUTelemtryData {
        ecu_data: ECUDataFrame {
            time: 420.69,
            igniter_state: hal::ecu_hal::IgniterState::Idle,
            valve_states: [42_u8; MAX_ECU_VALVES],
            sensor_states: [69_u16; MAX_ECU_SENSORS],
            sparking: false,
        },
        avg_loop_time_ms: 420.420,
        max_loop_time_ms: 69.96,
    });
    let controller_aborted = Packet::ControllerAborted(NetworkAddress::MissionControl);

    match set_valve {
        Packet::SetValve { .. } => println!("test"),
        Packet::FireIgniter => println!("test"),
        Packet::ConfigureSensor { .. } => println!("test"),
        Packet::ConfigureIgniterTiming(_) => println!("test"),
        Packet::Abort => println!("test"),
        Packet::ECUTelemtry(_) => println!("test"),
        Packet::ControllerAborted(_) => println!("test"),
    }

    [
        set_valve,
        fire_igniter,
        configure_sensor,
        configure_igniter_timing,
        abort,
        ecu_telemetry,
        controller_aborted,
    ]
}
