#![no_std]
#![no_main]

use core::u16;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::peripherals;
use embassy_stm32::timer::qei::*;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Start encoder test..");

    let p = embassy_stm32::init(Default::default());

    // Test left wheel encoder
    let enc_a: QeiPin<'_, peripherals::TIM2, Ch1> = QeiPin::new_ch1(p.PA0);
    let enc_b: QeiPin<'_, peripherals::TIM2, Ch2> = QeiPin::new_ch2(p.PA1);
    let qei_enc = Qei::new(p.TIM2, enc_a, enc_b);
    
    let mut enc_count: i32 = 0;
    let mut prev_qei_count: i32 = 0;
    loop {
        let curr_qei_count = (qei_enc.count() as i16) as i32;
        let diff = curr_qei_count.wrapping_sub(prev_qei_count);
        enc_count += diff;
        
        println!("{}, {}, {}, {}", enc_count, diff, curr_qei_count, prev_qei_count);
        prev_qei_count = curr_qei_count;
        Timer::after(Duration::from_millis(10)).await;
    }
}