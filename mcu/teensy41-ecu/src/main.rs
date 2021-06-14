//! The starter code slowly blinks the LED, and sets up
//! USB logging.

#![no_std]
#![no_main]

use bsp::hal::adc;
use ecu::Ecu;
use ecu::HALs;
use teensy4_bsp as bsp;
use teensy4_panic as _;

use bsp::hal::adc::AnalogInput;
use bsp::hal::gpio::GPIO;
pub use bsp::t41;

use crate::comms::Teensy41ECUComms;
use crate::ecu_hardware::Teensy41ECUHardware;

mod comms;
mod ecu_hardware;
mod logging;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut p = bsp::Peripherals::take().unwrap();
    let mut _systick = bsp::SysTick::new(cortex_m::Peripherals::take().unwrap().SYST);
    let pins = bsp::t41::into_pins(p.iomuxc);
    let mut _led = bsp::configure_led(pins.p13);

    let (adc1_builder, _) = p.adc.clock(&mut p.ccm.handle);
    let adc1 = adc1_builder.build(adc::ClockSelect::default(), adc::ClockDivision::default());

    let mut teensy41_hardware = Teensy41ECUHardware::new(
        GPIO::new(pins.p2).output(),
        GPIO::new(pins.p3).output(),
        GPIO::new(pins.p4).output(),
        GPIO::new(pins.p5).output(),
        GPIO::new(pins.p6).output(),
        GPIO::new(pins.p7).output(),
        GPIO::new(pins.p8).output(),
        AnalogInput::new(pins.p23),
        AnalogInput::new(pins.p22),
        AnalogInput::new(pins.p41),
        AnalogInput::new(pins.p40),
        AnalogInput::new(pins.p21),
        AnalogInput::new(pins.p20),
        AnalogInput::new(pins.p19),
        AnalogInput::new(pins.p18),
        AnalogInput::new(pins.p17),
        AnalogInput::new(pins.p16),
        AnalogInput::new(pins.p15),
        AnalogInput::new(pins.p14),
        adc1,
    );

    let mut teensy41_comms = Teensy41ECUComms::new(0);

    let mut ecu = Ecu::new(0);

    // See the `logging` module docs for more info.
    assert!(logging::init().is_ok());

    loop {
        teensy41_hardware.read_sensors();

        ecu.update(
            &mut HALs {
                hardware: &mut teensy41_hardware,
                comms: &mut teensy41_comms,
            },
            0.001,
        );
    }
}
