use std::{env, net::UdpSocket};

use hal::{comms_hal::{NetworkAddress, Packet, comms_ethernet_hal::{self, ETHERNET_BUFFER_SIZE}}, ecu_hal::ECUValve};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("No args!");
        return;
    }

    let socket = UdpSocket::bind("0.0.0.0:25566").unwrap();
    let mut buffer = [0_u8; ETHERNET_BUFFER_SIZE];

    match args[1].as_str() {
        "valve" => {
            if args.len() < 4 {
                println!("Not enough args. Format: valve <name> <value>");
                return;
            }

            let mut valve = match args[2].as_str() {
                "fuel_main" => Some(ECUValve::IgniterFuelMain),
                "gox_main" => Some(ECUValve::IgniterGOxMain),
                "fuel_press" => Some(ECUValve::FuelPress),
                "fuel_vent" => Some(ECUValve::FuelVent),
                _ => None
            };

            let packet = Packet::SetValve {
                valve: valve.unwrap(),
                state: args[3].parse::<u8>().unwrap(),
            };

            let len = comms_ethernet_hal::serialize_packet(
                &packet, 
                NetworkAddress::MissionControl, 
                NetworkAddress::EngineController(0), 
                &mut buffer
            ).unwrap();

            println!("Sending packet {:?}", packet);

            socket.send_to(&buffer[0..len], "10.0.0.5:8888").unwrap();
        },
        "fire" => {
            println!("firing");
        },
        _ => {
            println!("Unknown command");
        }
    }
}
