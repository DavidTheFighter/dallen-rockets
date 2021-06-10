use std::io::stdout;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use display::ConsoleDisplay;
use hal::ecu_hal::ECUSensor;

pub mod display;

fn main() {
    let mut display = ConsoleDisplay::new(stdout());

    display.set_watchdog("ECU(0)", true);
    display.set_watchdog("TCU", false);

    let sensors = [
        (ECUSensor::IgniterThroatTemp, "IGN Throat", "\u{b0}C"),
        (
            ECUSensor::IgniterFuelInjectorPressure,
            "IGN Fuel-Inj",
            "psi",
        ),
        (ECUSensor::IgniterGOxInjectorPressure, "IGN GOx-Inj", "psi"),
        (ECUSensor::IgniterChamberPressure, "IGN Chamber", "psi"),
        (ECUSensor::FuelTankPressure, "Fuel Tank", "psi"),
    ];

    for (_sensor, name, units) in &sensors {
        display.set_sensor(*name, 42.0, *units, false);
    }

    loop {
        display.render();

        if event::poll(std::time::Duration::from_millis(1)).unwrap() {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                if code == KeyCode::Esc {
                    break;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    display.quit();
}
