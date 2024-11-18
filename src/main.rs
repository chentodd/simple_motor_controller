#![no_std]
#![no_main]

use nucleo_f401re_rust::encoder::Encoder;
use nucleo_f401re_rust::motor::BldcMotor24H;
use nucleo_f401re_rust::pid::Pid;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::Timer;

use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::pac;
use embassy_stm32::peripherals::{TIM2, TIM3, TIM4};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, Timer as LLTimer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};

use {defmt_rtt as _, panic_probe as _};

const PERIOD_S: f32 = 0.005;
static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[interrupt]
unsafe fn TIM5() {
    // Trigger the signal to notify the task
    TIMER_SIGNAL.signal(());
    pac::TIM5.sr().modify(|r| r.set_uif(false));
}

#[embassy_executor::task]
async fn run(
    mut left_wheel: BldcMotor24H<'static, TIM2, TIM3>,
    mut right_wheel: BldcMotor24H<'static, TIM4, TIM3>,
) {
    // left_wheel.set_target_velocity(1000.0);
    // right_wheel.set_target_velocity(-1000.0);

    loop {
        TIMER_SIGNAL.wait().await;
        left_wheel.run_pid_velocity_control();
        right_wheel.run_pid_velocity_control();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Init
    let p = embassy_stm32::init(Default::default());

    let left_wheel_enc: Encoder<'_, TIM2, 400> = Encoder::new(p.TIM2, p.PA0, p.PA1);
    let left_wheel_pwm_pin = PwmPin::new_ch3(p.PB0, OutputType::PushPull);
    let left_wheel_dir_pin = Output::new(p.PA4, Level::High, Speed::Low);
    let left_wheel_break_pin = Output::new(p.PC1, Level::High, Speed::Low);
    let left_wheel_pid = Pid::new(0.00006, 0.00124, 0.000000728, 1.0);

    let right_wheel_enc: Encoder<'_, TIM4, 400> = Encoder::new(p.TIM4, p.PB6, p.PB7);
    let right_wheel_pwm_pin = PwmPin::new_ch1(p.PB4, OutputType::PushPull);
    let right_wheel_dir_pin = Output::new(p.PB5, Level::High, Speed::Low);
    let right_wheel_break_pin = Output::new(p.PB3, Level::High, Speed::Low);
    let right_wheel_pid = Pid::new(0.00006, 0.00124, 0.000000728, 1.0);

    let pwm = SimplePwm::new(
        p.TIM3,
        Some(right_wheel_pwm_pin),
        None,
        Some(left_wheel_pwm_pin),
        None,
        Hertz::khz(20),
        Default::default(),
    );

    let pwm_channels = pwm.split();
    let left_wheel_pwm_ch = pwm_channels.ch3;
    let right_wheel_pwm_ch = pwm_channels.ch1;

    // Create wheels
    let left_wheel = BldcMotor24H::new(
        left_wheel_enc,
        left_wheel_pwm_ch,
        left_wheel_dir_pin,
        left_wheel_break_pin,
        left_wheel_pid,
        PERIOD_S,
    );

    let right_wheel = BldcMotor24H::new(
        right_wheel_enc,
        right_wheel_pwm_ch,
        right_wheel_dir_pin,
        right_wheel_break_pin,
        right_wheel_pid,
        PERIOD_S,
    );

    // Create timer
    let low_level_timer = LLTimer::new(p.TIM5);
    low_level_timer.set_counting_mode(CountingMode::EdgeAlignedUp);
    low_level_timer.set_frequency(Hertz::hz(200));
    low_level_timer.set_autoreload_preload(true);
    low_level_timer.enable_update_interrupt(true);
    low_level_timer.start();
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::TIM5);
    }

    // Test
    spawner.spawn(run(left_wheel, right_wheel)).unwrap();
    loop {
        Timer::after_secs(1).await;
    }
}
