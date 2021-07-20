use core::cell::RefCell;

use cortex_m::interrupt::{CriticalSection, Mutex};
use teensy4_canfd::{RxFDFrame, config::Id};
use atomic_queue::AtomicQueue;

#[derive(Debug, Clone, Copy)]
struct FrameWrapper {
    pub id: Id,
    pub buffer_len: u32,
    pub buffer: [u8; 64],
    pub timestamp: u16,
    pub error_state: bool,
}

fn empty_rx_frame() -> FrameWrapper {
    FrameWrapper {
        id: Id::Standard(0),
        buffer_len: 0,
        buffer: [0_u8; 64],
        timestamp: 0,
        error_state: false,
    }
}

fn from_rx_frame(rx_frame: &RxFDFrame) -> FrameWrapper {
    let mut frame = FrameWrapper {
        id: rx_frame.id,
        buffer_len: rx_frame.buffer_len,
        buffer: [0_u8; 64],
        timestamp: rx_frame.timestamp,
        error_state: rx_frame.error_state
    };

    for (to, from) in frame.buffer.iter_mut().zip(rx_frame.buffer.iter()) {
        *to = *from;
    }

    frame
}

fn from_frame(frame: &FrameWrapper) -> RxFDFrame {
    let mut rx_frame = RxFDFrame {
        id: frame.id,
        buffer_len: frame.buffer_len,
        buffer: [0_u8; 64],
        timestamp: frame.timestamp,
        error_state: frame.error_state
    };

    for (to, from) in rx_frame.buffer.iter_mut().zip(frame.buffer.iter()) {
        *to = *from;
    }

    rx_frame
}

// This is the static storage we use to back our queue
static mut STORAGE: [FrameWrapper; 64] = [FrameWrapper {
    id: Id::Standard(0),
    buffer_len: 0,
    buffer: [0_u8; 64],
    timestamp: 0,
    error_state: false,
}; 64];

// This is our queue. We need `lazy_static` because we can't refer to the storage above at compile time.
lazy_static! {
    static ref QUEUE: AtomicQueue<'static, FrameWrapper> = {
        let m = unsafe { AtomicQueue::new(&mut STORAGE) };
        m
    };
}

#[allow(clippy::needless_pass_by_value)]
pub fn on_canfd_frame(_cs: &CriticalSection, rx_frame: RxFDFrame) {
    QUEUE.push(from_rx_frame(&rx_frame));

    log::info!("CAN FRAME");
}

pub fn pop_rx_frame(_cs: &CriticalSection) -> Option<RxFDFrame> {
    let frame = QUEUE.pop();

    match frame {
        Some(frame) => Some(from_frame(&frame)),
        None => None
    }
}
