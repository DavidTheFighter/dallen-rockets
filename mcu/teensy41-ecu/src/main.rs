//! The starter code slowly blinks the LED, and sets up
//! USB logging.

#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use bsp::hal::gpio::{GPIO, Output};
pub use bsp::t41;

mod logging;

const LED_PERIOD_MS: u32 = 1_000;

pub(crate) struct Teensy41ECU {
    pub sv1_pin: GPIO<t41::P2, Output>,
    pub sv2_pin: GPIO<t41::P3, Output>,
    pub sv3_pin: GPIO<t41::P4, Output>,
    pub sv4_pin: GPIO<t41::P5, Output>,
    pub sv5_pin: GPIO<t41::P6, Output>,
    pub sv6_pin: GPIO<t41::P7, Output>,
}

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = bsp::Peripherals::take().unwrap();
    let mut systick = bsp::SysTick::new(cortex_m::Peripherals::take().unwrap().SYST);
    let pins = bsp::t41::into_pins(p.iomuxc);
    let mut led = bsp::configure_led(pins.p13);

    // See the `logging` module docs for more info.
    assert!(logging::init().is_ok());

    loop {
        led.toggle();
        systick.delay(LED_PERIOD_MS);
        log::info!("Hello world");
    }
}
