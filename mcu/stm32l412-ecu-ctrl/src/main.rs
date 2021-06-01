//! Blinks an LED

#![deny(unsafe_code)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate stm32l4xx_hal;

pub mod ecu_hardware;

use ecu::{Ecu, HALs};
use enc28j60::Enc28j60;
use hal::{comms_mock::CommsMock, ecu_hal::SensorConfig, MAX_SENSORS, MAX_VALVES};
use stm32l4xx_hal::delay::Delay;
use stm32l4xx_hal::hal::spi::{Mode, Phase, Polarity};
use stm32l4xx_hal::rcc::{PllConfig, PllDivider, PllSource};
use stm32l4xx_hal::spi::Spi;

use crate::rt::entry;
use crate::rt::ExceptionFrame;
use crate::stm32l4xx_hal::prelude::*;

use core::{convert::Infallible, panic::PanicInfo};

pub struct STM32L412ECUController<'a> {
    sv1_pin: &'a mut dyn OutputPin<Error = Infallible>,
    sv2_pin: &'a mut dyn OutputPin<Error = Infallible>,
    sv3_pin: &'a mut dyn OutputPin<Error = Infallible>,
    sv4_pin: &'a mut dyn OutputPin<Error = Infallible>,
    sv5_pin: &'a mut dyn OutputPin<Error = Infallible>,
    spark_plug_pin: &'a mut dyn OutputPin<Error = Infallible>,
    valve_states: [u8; MAX_VALVES],
    raw_sensor_values: [u16; MAX_SENSORS],
    sensor_values: [f32; MAX_SENSORS],
    sensor_configs: [SensorConfig; MAX_SENSORS],
}

pub const MODE: Mode = Mode {
    phase: Phase::CaptureOnFirstTransition,
    polarity: Polarity::IdleLow,
};

#[entry]
fn main() -> ! {
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32l4xx_hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    let clocks = rcc
        .cfgr
        .pll_source(PllSource::HSI16)
        .sysclk_with_pll(80.mhz(), PllConfig::new(2, 20, PllDivider::Div2))
        .pclk1(80.mhz())
        .pclk2(80.mhz())
        .freeze(&mut flash.acr, &mut pwr);

    let mut gpio_a = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpio_b = dp.GPIOB.split(&mut rcc.ahb2);

    let mut sv1_pin = gpio_a
        .pa1
        .into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);

    let mut sv2_pin = gpio_a
        .pa2
        .into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);

    let mut sv3_pin = gpio_a
        .pa3
        .into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);

    let mut sv4_pin = gpio_b
        .pb0
        .into_push_pull_output(&mut gpio_b.moder, &mut gpio_b.otyper);

    let mut sv5_pin = gpio_b
        .pb1
        .into_push_pull_output(&mut gpio_b.moder, &mut gpio_b.otyper);

    let mut spark_plug_pin = gpio_a
        .pa15
        .into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);

    let mut nss = gpio_a
        .pa4
        .into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);    

    let sck = gpio_a.pa5.into_af5(&mut gpio_a.moder, &mut gpio_a.afrl);
    let miso = gpio_a.pa6.into_af5(&mut gpio_a.moder, &mut gpio_a.afrl);
    let mosi = gpio_a.pa7.into_af5(&mut gpio_a.moder, &mut gpio_a.afrl);

    nss.set_high().unwrap();

    let mut spi = Spi::spi1(
        dp.SPI1,
        (sck, miso, mosi),
        MODE,
        1.mhz(),
        clocks,
        &mut rcc.apb2,
    );

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut reset = gpio_a.pa11.into_push_pull_output(&mut gpio_a.moder, &mut gpio_a.otyper);
    reset.set_high().unwrap();

    let mut enc28j60 = Enc28j60::new(
        spi,
        nss,
        enc28j60::Unconnected,
        enc28j60::Unconnected,
        &mut delay,
        7 * 1024,
        [0x20, 0x18, 0x03, 0x01, 0x00, 0x00]
    );

    let mut ecu_hardware = STM32L412ECUController::new(
        &mut sv1_pin,
        &mut sv2_pin,
        &mut sv3_pin,
        &mut sv4_pin,
        &mut sv5_pin,
        &mut spark_plug_pin,
    );

    let mut comms = CommsMock::new();

    let mut ecu = Ecu::new(0);

    loop {
        ecu.update(
            &mut HALs {
                hardware: &mut ecu_hardware,
                comms: &mut comms,
            },
            0.001,
        );
    }
}

impl<'a> STM32L412ECUController<'a> {
    pub fn new(
        sv1_pin: &'a mut dyn OutputPin<Error = Infallible>,
        sv2_pin: &'a mut dyn OutputPin<Error = Infallible>,
        sv3_pin: &'a mut dyn OutputPin<Error = Infallible>,
        sv4_pin: &'a mut dyn OutputPin<Error = Infallible>,
        sv5_pin: &'a mut dyn OutputPin<Error = Infallible>,
        spark_plug_pin: &'a mut dyn OutputPin<Error = Infallible>,
    ) -> STM32L412ECUController<'a> {
        STM32L412ECUController {
            sv1_pin,
            sv2_pin,
            sv3_pin,
            sv4_pin,
            sv5_pin,
            spark_plug_pin,
            valve_states: [0_u8; MAX_VALVES],
            raw_sensor_values: [0_u16; MAX_SENSORS],
            sensor_values: [0.0_f32; MAX_SENSORS],
            sensor_configs: [SensorConfig {
                premin: 0.0,
                premax: 1024.0,
                postmin: 0.0,
                postmax: 1.0,
            }; MAX_SENSORS],
        }
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
