use std::{
    io::stdout,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Instant,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use display::ConsoleDisplay;
use hal::{
    comms_hal::{ECUTelemtryData, Packet},
    ecu_hal::{ECUSensor, ECUValve, IgniterState, ECU_SENSORS, ECU_VALVES},
};
use recv::RecvOutput;

pub mod display;
pub mod record;
pub mod recv;

pub(crate) static RUNNING: AtomicBool = AtomicBool::new(true);

pub const WATCHDOG_TIMEOUT_MS: usize = 250;

const DISPLAY_SENSORS: [(ECUSensor, &str, &str); 5] = [
    (ECUSensor::FuelTankPressure, "Fuel Tank", "psi"),
    (ECUSensor::IgniterThroatTemp, "IGN Throat", "\u{b0}C"),
    (
        ECUSensor::IgniterFuelInjectorPressure,
        "IGN Fuel-Inj",
        "psi",
    ),
    (ECUSensor::IgniterGOxInjectorPressure, "IGN GOx-Inj", "psi"),
    (ECUSensor::IgniterChamberPressure, "IGN Chamber", "psi"),
];

const SENSORS_CONFIG: [f32; 5] = [4096.0, 300.0, 200.0, 200.0, 300.0];

const DISPLAY_VALVES: [(ECUValve, &str); 4] = [
    (ECUValve::FuelPress, "Fuel Press"),
    (ECUValve::FuelVent, "Fuel Vent"),
    (ECUValve::IgniterFuelMain, "IGN Fuel Main"),
    (ECUValve::IgniterGOxMain, "IGN GOx Main"),
];

fn main() {
    let mut display = ConsoleDisplay::new(stdout());

    display.set_watchdog("ECU(0)", false);
    display.set_watchdog("CET", false);

    for (sensor, name, units) in &DISPLAY_SENSORS {
        display.set_sensor_full(&format!("{:?}", sensor), *name, -42.0, *units, false);
    }

    for (valve, name) in &DISPLAY_VALVES {
        display.set_valve_full(&format!("{:?}", valve), *name, false);
    }

    display.set_misc("IGN State", &format!("{:?}", IgniterState::Idle));
    display.set_misc("Spark", "Off");
    display.set_misc("ECU Max Δt", "0.0 ms");
    display.set_misc("ECU Loop Freq", "0 Hz");
    display.set_misc("Recv ECU Freq", "0 Hz");
    display.set_misc("Recording", "false");
    display.set_misc("counter", "0");

    let (recv_thread_tx, recv_thread_rx) = mpsc::channel();
    let (record_thread_tx, record_thread_rx) = mpsc::channel();

    let recv_thread = std::thread::spawn(move || {
        recv::packet_loop(recv_thread_tx);
    });

    let record_thread = std::thread::spawn(move || {
        record::record_loop(record_thread_rx);
    });

    let mut ecu_timer_ms: usize = WATCHDOG_TIMEOUT_MS;
    let mut cet_timer_ms: usize = WATCHDOG_TIMEOUT_MS;
    let mut ecu_telem_counter: usize = 0;
    let mut timer: f64 = 0.0;
    let mut counter = 0;
    let mut recording = false;

    loop {
        let start_time = Instant::now();

        display.render();

        if event::poll(std::time::Duration::from_millis(1)).unwrap() {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                if code == KeyCode::Esc {
                    break;
                } else if code == KeyCode::Char('r') {
                    recording = !recording;
                    display.set_misc("Recording", &format!("{}", recording));
                }
            }
        }

        loop {
            match recv_thread_rx.try_recv() {
                Ok(recv_output) => {
                    if let RecvOutput::Packet(packet) = &recv_output {
                        match packet {
                            Packet::RecordedData(data) => {
                                record_thread_tx
                                    .send(ECUTelemtryData {
                                        ecu_data: *data,
                                        max_loop_time: -42.0,
                                    })
                                    .unwrap();
                                counter += 1;
                            }
                            Packet::ECUTelemtry(data) => {
                                ecu_timer_ms = 0;
                                ecu_telem_counter += 1;

                                for ((value, sensor), range) in data
                                    .ecu_data
                                    .sensor_states
                                    .iter()
                                    .zip(ECU_SENSORS.iter())
                                    .zip(SENSORS_CONFIG.iter())
                                {
                                    let voltage = 5.0 * ((*value as f32) / 4095.0);
                                    let lerp = (voltage - 0.5) / 4.0;

                                    display.set_sensor_value(
                                        &format!("{:?}", *sensor),
                                        lerp * (*range),
                                        true,
                                    )
                                }

                                for (value, valve) in
                                    data.ecu_data.valve_states.iter().zip(ECU_VALVES.iter())
                                {
                                    display.set_valve_value(&format!("{:?}", *valve), *value > 0);
                                }

                                display.set_misc(
                                    "IGN State",
                                    &format!("{:?}", data.ecu_data.igniter_state),
                                );
                                display.set_misc(
                                    "Spark",
                                    if data.ecu_data.sparking {
                                        "Sparking"
                                    } else {
                                        "Off"
                                    },
                                );
                                display.set_misc(
                                    "ECU Max Δt",
                                    &format!("{:.3} ms", data.max_loop_time * 1000.0),
                                );
                                display.set_misc(
                                    "ECU Loop Freq",
                                    &format!("{:.1} kHz", 1e-3 / data.max_loop_time),
                                );
                                display.set_misc("counter", &format!("{}", counter));
                            }
                            _ => {}
                        }
                    }

                    if let RecvOutput::CETPulse = &recv_output {
                        cet_timer_ms = 0;
                    }
                }
                Err(err) => {
                    if err == mpsc::TryRecvError::Empty {
                        break;
                    }
                }
            }
        }

        if ecu_timer_ms >= WATCHDOG_TIMEOUT_MS {
            display.set_watchdog("ECU(0)", false);
        } else {
            display.set_watchdog("ECU(0)", true);
        }

        if cet_timer_ms >= WATCHDOG_TIMEOUT_MS {
            display.set_watchdog("CET", false);
        } else {
            display.set_watchdog("CET", true);
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
        ecu_timer_ms += 10;
        cet_timer_ms += 10;

        if timer >= 1.0 {
            display.set_misc("Recv ECU Freq", &format!("{:?} Hz", ecu_telem_counter));
            ecu_telem_counter = 0;
            timer = 0.0;
        }

        let elapsed = Instant::now() - start_time;
        timer += elapsed.as_secs_f64();
    }

    RUNNING.store(false, Ordering::Relaxed);

    display.quit();
    recv_thread.join().unwrap();
    record_thread.join().unwrap();
}
