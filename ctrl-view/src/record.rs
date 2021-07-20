use std::{fs::OpenOptions, sync::{atomic::Ordering, mpsc::Receiver}, time::Duration};
use std::io::prelude::*;

use hal::comms_hal::ECUTelemtryData;

use crate::{RUNNING, recv};

pub fn record_loop(recv_telem: Receiver<ECUTelemtryData>) {
    while RUNNING.load(Ordering::Relaxed) {
        match recv_telem.recv_timeout(Duration::from_millis(10)) {
            Ok(telem_data) => {
                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open("telem-data.log")
                    .unwrap();

                    writeln!(file, "{:?}", telem_data).unwrap();
            },
            Err(err) => {
                panic!("record_loop: {:?}", err);
            }
        }
    }
}
