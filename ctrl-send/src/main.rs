use std::{env, net::UdpSocket, thread, time::Duration};

use hal::{
    comms_hal::{
        comms_ethernet_hal::{self, ETHERNET_BUFFER_SIZE},
        NetworkAddress, Packet,
    },
    ecu_hal::{ECUValve, ECU_VALVES},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("No args!");
        return;
    }

    let socket = UdpSocket::bind("0.0.0.0:25566").unwrap();

    match args[1].as_str() {
        "valve" => {
            if args.len() < 4 {
                println!("Not enough args. Format: valve <name> <value>");
                return;
            }

            let valve = match args[2].as_str() {
                "fuel_main" => Some(ECUValve::IgniterFuelMain),
                "gox_main" => Some(ECUValve::IgniterGOxMain),
                "fuel_press" => Some(ECUValve::FuelPress),
                "fuel_vent" => Some(ECUValve::FuelVent),
                _ => None,
            };

            send_packet(
                &socket,
                Packet::SetValve {
                    valve: valve.unwrap(),
                    state: args[3].parse::<u8>().unwrap(),
                },
            );
        }
        "testvalve" => {
            if args.len() < 4 {
                println!("Not enough args. Format: testvalve <name> <delay>");
                return;
            }

            let valve = match args[2].as_str() {
                "fuel_main" => Some(ECUValve::IgniterFuelMain),
                "gox_main" => Some(ECUValve::IgniterGOxMain),
                "fuel_press" => Some(ECUValve::FuelPress),
                "fuel_vent" => Some(ECUValve::FuelVent),
                _ => None,
            };

            send_packet(
                &socket,
                Packet::SetValve {
                    valve: valve.unwrap(),
                    state: 255,
                },
            );

            std::thread::sleep(std::time::Duration::from_secs_f32(args[3].parse().unwrap()));

            send_packet(
                &socket,
                Packet::SetValve {
                    valve: valve.unwrap(),
                    state: 0,
                },
            );
        }
        "testvalves" => {
            if args.len() != 2 {
                println!("Too many args. Format: testvalves");
            }

            for valve in &ECU_VALVES {
                println!("Testing {:?}", valve);
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0));

                send_packet(
                    &socket,
                    Packet::SetValve {
                        valve: *valve,
                        state: 255,
                    },
                );

                std::thread::sleep(std::time::Duration::from_secs_f32(1.0));

                send_packet(
                    &socket,
                    Packet::SetValve {
                        valve: *valve,
                        state: 0,
                    },
                );
            }
        }
        "testspark" => {
            if args.len() < 3 {
                println!("Not enough args. Format: testspark <duration>");
                return;
            }

            send_packet(&socket, Packet::SetSparking(args[2].parse().unwrap()));
        }
        "fire" => {
            println!("Firing in T-5s");

            for i in 0..5 {
                thread::sleep(Duration::from_secs(1));
                println!("T-{}s", 4 - i);

                if 4 - i == 2 {
                    send_packet(&socket, Packet::SetRecording(true));
                }
            }

            send_packet(&socket, Packet::FireIgniter);
        }
        "transfer" => {
            println!("Transfering!");
            send_packet(&socket, Packet::TransferData);
        }
        _ => {
            println!("Unknown command");
        }
    }
}

fn send_packet(socket: &UdpSocket, packet: Packet) {
    let mut buffer = [0_u8; ETHERNET_BUFFER_SIZE];

    let len = comms_ethernet_hal::serialize_packet(
        &packet,
        NetworkAddress::MissionControl,
        NetworkAddress::EngineController(0),
        &mut buffer,
    )
    .unwrap();

    println!("Sending packet {:?} {:?}", packet, &buffer[6..len]);

    socket.send_to(&buffer[0..len], "10.0.0.5:8888").unwrap();
}
