use core::cell::RefCell;

use cortex_m::interrupt::{CriticalSection, Mutex};
use ringbuffer::{ConstGenericRingBuffer, RingBufferExt, RingBufferWrite};
use teensy4_canfd::RxFDFrame;

const BUFFER_SIZE: usize = 64;

lazy_static! {
    static ref BUFFER: Mutex<RefCell<ConstGenericRingBuffer<RxFDFrame, BUFFER_SIZE>>> =
        Mutex::new(RefCell::new(ConstGenericRingBuffer::new()));
}

#[allow(clippy::needless_pass_by_value)]
pub fn on_canfd_frame(cs: &CriticalSection, rx_frame: RxFDFrame) {
    BUFFER.borrow(cs).borrow_mut().push(rx_frame);
}

pub fn pop_rx_frame(cs: &CriticalSection) -> Option<RxFDFrame> {
    BUFFER.borrow(cs).borrow().get(0).cloned()
}
