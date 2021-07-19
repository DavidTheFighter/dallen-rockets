use std::{
    io::stdout,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use display::ConsoleDisplay;
use hal::{
    comms_hal::Packet,
    ecu_hal::{ECUSensor, ECUValve, ECU_SENSORS, ECU_VALVES},
};
use recv::RecvOutput;

pub mod display;
pub mod recv;

pub(crate) static RUNNING: AtomicBool = AtomicBool::new(true);

pub const WATCHDOG_TIMEOUT_MS: usize = 250;

fn main() {
    let mut display = ConsoleDisplay::new(stdout());

    display.set_watchdog("ECU(0)", false);
    display.set_watchdog("CET", false);

    let sensors = [
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

    let valves = [
        (ECUValve::FuelPress, "Fuel Press"),
        (ECUValve::FuelVent, "Fuel Vent"),
        (ECUValve::IgniterFuelMain, "IGN Fuel Main"),
        (ECUValve::IgniterGOxMain, "IGN GOx Main"),
    ];

    for (sensor, name, units) in &sensors {
        display.set_sensor_full(&format!("{:?}", sensor), *name, -42.0, *units, false);
    }

    for (valve, name) in &valves {
        display.set_valve_full(&format!("{:?}", valve), *name, false);
    }

    display.set_misc("Spark", "Off");

    let (recv_thread_tx, recv_thread_rx) = mpsc::channel();

    let recv_thread = std::thread::spawn(move || {
        recv::packet_loop(recv_thread_tx);
    });

    let mut ecu_timer_ms: usize = WATCHDOG_TIMEOUT_MS;
    let mut cet_timer_ms: usize = WATCHDOG_TIMEOUT_MS;

    loop {
        display.render();

        if event::poll(std::time::Duration::from_millis(1)).unwrap() {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                if code == KeyCode::Esc {
                    break;
                }
            }
        }

        loop {
            match recv_thread_rx.try_recv() {
                Ok(recv_output) => {
                    if let RecvOutput::Packet(packet) = &recv_output {
                        match packet {
                            Packet::ECUTelemtry(data) => {
                                ecu_timer_ms = 0;

                                for (value, sensor) in
                                    data.ecu_data.sensor_states.iter().zip(ECU_SENSORS.iter())
                                {
                                    display.set_sensor_value(
                                        &format!("{:?}", *sensor),
                                        *value as f32,
                                        true,
                                    )
                                }

                                for (value, valve) in
                                    data.ecu_data.valve_states.iter().zip(ECU_VALVES.iter())
                                {
                                    display.set_valve_value(&format!("{:?}", *valve), *value > 0);
                                }

                                display.set_misc("Spark", if data.ecu_data.sparking { "Sparking" } else { "Off" });
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
    }

    RUNNING.store(false, Ordering::Relaxed);

    display.quit();
    recv_thread.join().unwrap();
}
