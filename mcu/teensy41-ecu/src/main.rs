//! The starter code slowly blinks the LED, and sets up
//! USB logging.

#![no_std]
#![no_main]

#[macro_use]
extern crate lazy_static;

use bsp::hal::adc;
use bsp::hal::pit;
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
    let mut led = bsp::configure_led(pins.p13);

    // See the `logging` module docs for more info.
    assert!(logging::init().is_ok());

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
    let mut ecu = Ecu::new(0, &mut HALs {
        hardware: &mut teensy41_hardware,
        comms: &mut teensy41_comms,
    });

    let (_, ipg_hz) = p.ccm.pll1.set_arm_clock(
        bsp::hal::ccm::PLL1::ARM_HZ,
        &mut p.ccm.handle,
        &mut p.dcdc,
    );

    let mut cfg = p.ccm.perclk.configure(
        &mut p.ccm.handle,
        bsp::hal::ccm::perclk::PODF::DIVIDE_3,
        bsp::hal::ccm::perclk::CLKSEL::IPG(ipg_hz),
    );

    let (timer0, timer1, _, _) = p.pit.clock(&mut cfg);
    let mut timer = pit::chain(timer0, timer1);

    log::info!("Completed initialization");

    let mut counter = 0.0;
    let mut last_delta = 0.001;
    loop {
        let (_, period) = timer.time(|| {
            teensy41_hardware.read_sensors();

            ecu.update(
                &mut HALs {
                    hardware: &mut teensy41_hardware,
                    comms: &mut teensy41_comms,
                },
                last_delta,
            );
            
            if counter > 1.0 {
                led.toggle();

                counter = 0.0;
            }
    
            counter += last_delta;
        });

        match period {
            Some(period) => {
                last_delta = period.as_secs_f32();
            }
            None => log::error!("Loop timer expired!")
        }
    }
}
