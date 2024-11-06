#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use rtt_target::{rprintln, rtt_init_print};

use stm32f4xx_hal as hal;
use crate::hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello, start blinking LED1...");

    let dp = pac::Peripherals::take().unwrap();

    // pin
    let gpioa = dp.GPIOA.split();
    let mut led = gpioa.pa5.into_push_pull_output();

    // delay
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(48.MHz()).freeze();
    let mut delay = dp.TIM5.delay_us(&clocks);
    loop {
        led.toggle();
        delay.delay_ms(1000);
    }
}
