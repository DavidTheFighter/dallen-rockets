//! Blinks an LED

#![deny(unsafe_code)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use]
extern crate cortex_m_rt as rt;
extern crate stm32l4xx_hal;

pub mod ecu_hardware;

use hal::{ecu_hal::SensorConfig, MAX_SENSORS, MAX_VALVES};
use stm32l4xx_hal::rcc::{PllConfig, PllDivider, PllSource};

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

#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32l4xx_hal::stm32::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    let _clocks = rcc
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

    let mut ecu_hardware = STM32L412ECUController::new(
        &mut sv1_pin,
        &mut sv2_pin,
        &mut sv3_pin,
        &mut sv4_pin,
        &mut sv5_pin,
        &mut spark_plug_pin,
    );

    loop {}
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
