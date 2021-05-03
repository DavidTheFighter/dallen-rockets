#![allow(clippy::missing_panics_doc)]

use std::{
    collections::HashMap,
    io::{Stdout, Write},
};

use std::convert::TryFrom;

use crossterm::style::Colorize;
use crossterm::{cursor, execute, queue, style, terminal};

const SECOND_COL_POS: u16 = 32;

#[allow(clippy::module_name_repetitions)]
pub struct ConsoleDisplay {
    stdout: Stdout,
    watchdogs: HashMap<String, bool>,
    sensors: HashMap<String, (f32, String, bool)>,
}

impl ConsoleDisplay {
    pub fn new(mut stdout: Stdout) -> Self {
        execute!(stdout, terminal::EnterAlternateScreen).unwrap();
        terminal::enable_raw_mode().unwrap();

        Self {
            stdout,
            watchdogs: HashMap::new(),
            sensors: HashMap::new(),
        }
    }

    pub fn render(&mut self) {
        self.clear_terminal();
        queue!(
            self.stdout,
            style::Print("ctrl-view    - Press ESC to exit")
        )
        .unwrap();
        queue!(self.stdout, cursor::MoveToNextLine(2)).unwrap();

        let mut row_num = 0;
        for (name, status) in &self.watchdogs.clone() {
            self.render_watchdog(*status, name);

            if row_num % 2 == 0 {
                queue!(self.stdout, cursor::MoveToColumn(SECOND_COL_POS)).unwrap();
            } else {
                queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            }

            row_num += 1;
        }
        queue!(self.stdout, cursor::MoveToNextLine(2)).unwrap();

        row_num = 0;
        for (name, (value, units, ok)) in &self.sensors.clone() {
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

            if row_num % 2 == 0 {
                queue!(self.stdout, cursor::MoveToColumn(SECOND_COL_POS)).unwrap();
            } else {
                queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            }

            row_num += 1;
        }

        self.stdout.flush().unwrap();
    }

    pub fn set_watchdog(&mut self, name: &str, value: bool) {
        self.watchdogs.insert(String::from(name), value);
    }

    pub fn set_sensor(&mut self, name: &str, value: f32, units: &str, ok: bool) {
        self.sensors
            .insert(String::from(name), (value, String::from(units), ok));
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
