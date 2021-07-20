#![allow(clippy::missing_panics_doc)]

use std::io::{Stdout, Write};

use std::convert::TryFrom;

use crossterm::style::Colorize;
use crossterm::{cursor, execute, queue, style, terminal};

const SECOND_COL_POS: u16 = 32;

#[allow(clippy::module_name_repetitions)]
pub struct ConsoleDisplay {
    stdout: Stdout,
    watchdogs: Vec<(String, bool)>,
    sensors: Vec<(String, String, f32, String, bool)>,
    valves: Vec<(String, String, bool)>,
    misc: Vec<(String, String)>,
}

impl ConsoleDisplay {
    pub fn new(mut stdout: Stdout) -> Self {
        execute!(stdout, terminal::EnterAlternateScreen).unwrap();
        terminal::enable_raw_mode().unwrap();

        Self {
            stdout,
            watchdogs: Vec::new(),
            sensors: Vec::new(),
            valves: Vec::new(),
            misc: Vec::new(),
        }
    }

    pub fn render(&mut self) {
        self.clear_terminal();
        queue!(
            self.stdout,
            style::Print("ctrl-view".blue()),
            style::Print(" - Press 'ESC' to exit. Press 'r' to toggle telemetry recording")
        )
        .unwrap();
        queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();

        let mut row_num = 0;
        for (name, status) in &self.watchdogs.clone() {
            if row_num % 2 == 1 {
                queue!(self.stdout, cursor::MoveToColumn(SECOND_COL_POS)).unwrap();
            } else {
                queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            }

            self.render_watchdog(*status, name);

            row_num += 1;
        }
        queue!(self.stdout, cursor::MoveToNextLine(2)).unwrap();

        row_num = 0;
        for (_id, name, value, units, ok) in &self.sensors.clone() {
            if row_num % 2 == 1 {
                queue!(self.stdout, cursor::MoveToColumn(SECOND_COL_POS)).unwrap();
            } else {
                queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            }
            
            queue!(self.stdout, style::Print(format!("{}: ", name))).unwrap();

            let value_string = format!("{:.1}", value);
            let value_string_len = u16::try_from(value_string.len() & 0xFFFF).unwrap();
            queue!(
                self.stdout,
                cursor::MoveToColumn(
                    SECOND_COL_POS as u16 - 8 - value_string_len + SECOND_COL_POS * (row_num % 2)
                )
            )
            .unwrap();

            if *ok {
                queue!(self.stdout, style::Print(value_string.green())).unwrap();
            } else {
                queue!(self.stdout, style::Print(value_string.red())).unwrap();
            }

            queue!(self.stdout, style::Print(format!(" {}", units))).unwrap();

            row_num += 1;
        }
        queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();

        row_num = 0;
        for (_id, name, value) in &self.valves.clone() {
            if row_num % 2 == 1 {
                queue!(self.stdout, cursor::MoveToColumn(SECOND_COL_POS)).unwrap();
            } else {
                queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            }

            queue!(self.stdout, style::Print(format!("{}: ", name))).unwrap();

            let value_string = String::from(if *value { "Open" } else { "Closed" });
            let value_string_len = u16::try_from(value_string.len() & 0xFFFF).unwrap();
            queue!(
                self.stdout,
                cursor::MoveToColumn(
                    SECOND_COL_POS as u16 - 8 - value_string_len / 2
                        + SECOND_COL_POS * (row_num % 2)
                )
            )
            .unwrap();

            if *value {
                queue!(self.stdout, style::Print(value_string.red())).unwrap();
            } else {
                queue!(self.stdout, style::Print(value_string.green())).unwrap();
            }

            row_num += 1;
        }

        queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();

        row_num = 0;
        for (name, value) in &self.misc.clone() {
            if row_num % 2 == 1 {
                queue!(self.stdout, cursor::MoveToColumn(SECOND_COL_POS)).unwrap();
            } else {
                queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            }

            queue!(self.stdout, style::Print(format!("{}: ", name))).unwrap();

            let value_string = value.clone();
            let value_string_len = u16::try_from(value_string.len() & 0xFFFF).unwrap();
            queue!(
                self.stdout,
                cursor::MoveToColumn(
                    SECOND_COL_POS as u16 - 8 - value_string_len / 2
                        + SECOND_COL_POS * (row_num % 2)
                )
            )
            .unwrap();

            queue!(self.stdout, style::Print(value_string.blue())).unwrap();

            row_num += 1;
        }

        self.stdout.flush().unwrap();
    }

    pub fn set_watchdog(&mut self, name: &str, value: bool) {
        let mut found = false;
        for watchdog in self.watchdogs.iter_mut() {
            if watchdog.0 == name {
                *watchdog = (String::from(name), value);
                found = true;
                break;
            }
        }

        if !found {
            self.watchdogs.push((String::from(name), value));
        }
    }

    pub fn set_sensor_full(&mut self, id: &str, name: &str, value: f32, units: &str, ok: bool) {
        let mut found = false;
        for sensor in self.sensors.iter_mut() {
            if sensor.0 == id {
                *sensor = (
                    String::from(id),
                    String::from(name),
                    value,
                    String::from(units),
                    ok,
                );
                found = true;
                break;
            }
        }

        if !found {
            self.sensors.push((
                String::from(id),
                String::from(name),
                value,
                String::from(units),
                ok,
            ));
        }
    }

    pub fn set_sensor_value(&mut self, id: &str, value: f32, ok: bool) {
        for sensor in self.sensors.iter_mut() {
            if sensor.0 == id {
                sensor.2 = value;
                sensor.4 = ok;
                break;
            }
        }
    }

    pub fn set_valve_full(&mut self, id: &str, name: &str, value: bool) {
        let mut found = false;
        for valve in self.valves.iter_mut() {
            if valve.0 == id {
                *valve = (String::from(id), String::from(name), value);
                found = true;
                break;
            }
        }

        if !found {
            self.valves
                .push((String::from(id), String::from(name), value));
        }
    }

    pub fn set_misc(&mut self, name: &str, value: &str) {
        let mut found = false;
        for misc in self.misc.iter_mut() {
            if misc.0 == name {
                misc.1 = String::from(value);
                found = true;
                break;
            }
        }

        if !found {
            self.misc
                .push((String::from(name), String::from(value)));
        }
    }

    pub fn set_valve_value(&mut self, id: &str, value: bool) {
        for valve in self.valves.iter_mut() {
            if valve.0 == id {
                valve.2 = value;
                break;
            }
        }
    }

    fn render_watchdog(&mut self, watchdog: bool, name: &str) {
        queue!(self.stdout, style::Print(format!("{}: ", name))).unwrap();
        if watchdog {
            queue!(self.stdout, style::Print("Alive".green())).unwrap();
        } else {
            queue!(self.stdout, style::Print("Timed out".red())).unwrap();
        }
    }

    fn clear_terminal(&mut self) {
        queue!(
            self.stdout,
            style::ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide,
        )
        .unwrap();
    }

    pub fn quit(mut self) {
        execute!(
            self.stdout,
            style::ResetColor,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .unwrap();

        terminal::disable_raw_mode().unwrap();
    }
}
