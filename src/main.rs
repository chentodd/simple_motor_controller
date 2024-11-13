#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::OutputPolarity;
use embassy_stm32::timer::qei::{Qei, QeiPin};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // left wheel encoder
    let enc_a = QeiPin::new_ch1(p.PA0);
    let enc_b = QeiPin::new_ch2(p.PA1);
    let qei_enc = Qei::new(p.TIM2, enc_a, enc_b);

    // left wheel pwm
    let pwm_ch3_pin = PwmPin::new_ch3(p.PB0, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM3, 
        None, 
        None, 
        Some(pwm_ch3_pin), 
        None, 
        Hertz::khz(20), 
        Default::default());

    let mut pwm_ch3 = pwm.ch3();
    pwm_ch3.set_polarity(OutputPolarity::ActiveLow);
    pwm_ch3.enable();

    // left wheel direction pin
    // HIGH: CCW
    // LOW: CW
    let mut dir_pin = Output::new(p.PA4, Level::High, Speed::Low);
    
    // Test
    let mut enc_count: i32 = 0;
    let mut prev_qei_count: i32 = 0;

    pwm_ch3.set_duty_cycle_percent(10);
    loop {
        let curr_qei_count = (qei_enc.count() as i16) as i32;
        let diff = curr_qei_count.wrapping_sub(prev_qei_count);
        enc_count += diff;
        
        println!("{}, {}, {}, {}", enc_count, diff, curr_qei_count, prev_qei_count);
        prev_qei_count = curr_qei_count;
        Timer::after(Duration::from_millis(10)).await;
    }
}