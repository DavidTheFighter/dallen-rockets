use atomic_queue::AtomicQueue;
use hal::{
    comms_hal::{NetworkAddress, Packet},
    ecu_hal::{ECUDataFrame, MAX_ECU_SENSORS, MAX_ECU_VALVES},
};

use crate::{Ecu, HALs, RECORD_RATE, RECORD_STORAGE_SIZE, RECORD_TRANSMIT_RATE};

static mut STORAGE: [ECUDataFrame; RECORD_STORAGE_SIZE] = [ECUDataFrame {
    igniter_state: hal::ecu_hal::IgniterState::Firing,
    sensor_states: [0_u16; MAX_ECU_SENSORS],
    valve_states: [0_u8; MAX_ECU_VALVES],
    sparking: false,
}; RECORD_STORAGE_SIZE];

lazy_static! {
    static ref QUEUE: AtomicQueue<'static, ECUDataFrame> = {
        let m = unsafe { AtomicQueue::new(&mut STORAGE) };
        m
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecorderState {
    Idle,
    Recording,
    Transferring,
}

pub struct DataRecorder {
    state: RecorderState,
    elapsed_since_last_record: f32,
    elapsed_since_last_transfer: f32,
}

impl Ecu {
    pub fn record_update(&mut self, elapsed: f32, hals: &mut HALs) {
        if self.recorder.state == RecorderState::Recording {
            self.recorder.elapsed_since_last_record += elapsed;

            if self.recorder.elapsed_since_last_record >= RECORD_RATE {
                self.recorder.elapsed_since_last_record -= RECORD_RATE;
                self.record_data(hals);
            }
        } else if self.recorder.state == RecorderState::Transferring {
            self.recorder.elapsed_since_last_transfer += elapsed;

            if self.recorder.elapsed_since_last_transfer >= RECORD_TRANSMIT_RATE {
                self.recorder.elapsed_since_last_transfer -= RECORD_TRANSMIT_RATE;

                if let Some(frame) = QUEUE.pop() {
                    if let Err(err) = hals
                        .comms
                        .transmit(&Packet::RecordedData(frame), NetworkAddress::MissionControl)
                    {
                        log::error!("Couldn't send recorded data: {:?}", err);
                    }
                } else {
                    self.recorder.state = RecorderState::Idle;
                    self.recorder.elapsed_since_last_transfer = 0.0;
                }
            }
        }
    }

    pub fn set_recording(&mut self, value: bool) {
        if value && self.recorder.state == RecorderState::Idle {
            self.recorder.state = RecorderState::Recording;
            self.recorder.elapsed_since_last_record = 0.0;
        } else if !value && self.recorder.state == RecorderState::Recording {
            self.recorder.state = RecorderState::Idle;
            self.recorder.elapsed_since_last_record = 0.0;
        }
    }

    pub fn transfer_recorded_data(&mut self) {
        self.recorder.elapsed_since_last_transfer = 0.0;
        self.recorder.elapsed_since_last_record = 0.0;
        self.recorder.state = RecorderState::Transferring;
    }

    fn record_data(&mut self, hals: &mut HALs) {
        if let Err(_) = QUEUE.push(self.get_ecu_data_frame(hals)) {
            self.recorder.state = RecorderState::Idle;
            self.recorder.elapsed_since_last_record = 0.0;
        }
    }
}

impl DataRecorder {
    pub fn new() -> DataRecorder {
        DataRecorder {
            state: RecorderState::Idle,
            elapsed_since_last_record: 0.0,
            elapsed_since_last_transfer: 0.0,
        }
    }
}
