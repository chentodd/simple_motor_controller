#![no_std]
#![no_main]

use core::f32;
use defmt::info;
use fw::encoder::Encoder;
use fw::motion::Motion;
use fw::motor::BldcMotor24H;
use fw::pid::Pid;
use fw::rpm_to_rad_s;

use s_curve::*;
// use utils::*;

#[cfg(any(
    feature = "debug-rx",
    feature = "debug-tx",
    feature = "debug-motor",
    feature = "debug-motion"
))]
use defmt::debug;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::{InterruptExecutor, Spawner};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_sync::pubsub::{PubSubChannel, WaitResult};
use embassy_sync::signal::Signal;
// use embassy_sync::watch::Watch;
use embassy_time::Timer;

use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_stm32::pac;
use embassy_stm32::peripherals::{TIM2, TIM3, TIM4};
// use embassy_stm32::{bind_interrupts, usart};

use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, Timer as LLTimer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
// use embassy_stm32::timer::GeneralInstance4Channel;

const PERIOD_S: f32 = 0.005;
const PWM_HZ: u32 = 20_000;
const VEL_LIMIT_RPM: f32 = 4000.0;

static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static EXECUTOR_TIMER: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn TIM1_BRK_TIM15() {
    let sr = pac::TIM15.sr().read();
    if sr.uif() {
        pac::TIM15.sr().modify(|r| r.set_uif(false));
        EXECUTOR_TIMER.on_interrupt();
        TIMER_SIGNAL.signal(());
    }
}

#[embassy_executor::task]
async fn motion_task(mut left_motion_controller: Motion<'static, TIM2, TIM3>) {
    // let mut left_cmd_subscriber = LEFT_COMMAND_CHANNEL.subscriber().unwrap();
    // let mut right_cmd_subscriber = RIGHT_COMMAND_CHANNEL.subscriber().unwrap();
    // let motion_data_sender = MOTION_DATA.sender();

    // let mut left_cmd_queue = Deque::<MotorRx, 32>::new();
    // let mut right_cmd_queue = Deque::<MotorRx, 32>::new();

    // let mut left_data = MotorTx::default();
    // let mut right_data = MotorTx::default();
    // let mut tx_data = CommandTx::default();
    loop {
        TIMER_SIGNAL.wait().await;

        // if let Some(left_cmd) = left_cmd_subscriber.try_next_message() {
        //     match left_cmd {
        //         WaitResult::Lagged(_val) => {
        //             #[cfg(feature = "debug-motion")]
        //             debug!("left command lag: {}", _val);
        //         }
        //         WaitResult::Message(cmd) => {
        //             if cmd.operation == Operation::Stop {
        //                 left_cmd_queue.clear();
        //             }
        //             let _ = left_cmd_queue.push_back(cmd);
        //         }
        //     }
        // }

        // if let Some(right_cmd) = right_cmd_subscriber.try_next_message() {
        //     match right_cmd {
        //         WaitResult::Lagged(_val) => {
        //             #[cfg(feature = "debug-motion")]
        //             debug!("right command lag: {}", _val);
        //         }
        //         WaitResult::Message(cmd) => {
        //             if cmd.operation == Operation::Stop {
        //                 right_cmd_queue.clear();
        //             }
        //             let _ = right_cmd_queue.push_back(cmd);
        //         }
        //     }
        // }

        // let first_mode = left_cmd_queue.front().and_then(|x| Some(x.operation));
        // if left_motion_controller.can_send_cmd(first_mode) {
        //     if let Some(cmd) = left_cmd_queue.pop_front() {
        //         left_motion_controller.set_command(cmd);
        //     }
        // }

        // let first_mode = right_cmd_queue.front().and_then(|x| Some(x.operation));
        // if right_motion_controller.can_send_cmd(first_mode) {
        //     if let Some(cmd) = right_cmd_queue.pop_front() {
        //         right_motion_controller.set_command(cmd);
        //     }
        // }

        left_motion_controller.set_command(-300.0);
        left_motion_controller.run();
        // right_motion_controller.run();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Init hardware
    info!("Start");
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: embassy_stm32::time::mhz(8),
            mode: HseMode::Bypass,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }
    let p = embassy_stm32::init(config);

    let left_wheel_enc: Encoder<'_, TIM2, 400> = Encoder::new(p.TIM2, p.PA0, p.PA1);
    let left_wheel_pwm_pin = PwmPin::new_ch3(p.PB0, OutputType::PushPull);
    let left_wheel_dir_pin = Output::new(p.PA4, Level::High, Speed::Low);
    let left_wheel_break_pin = Output::new(p.PC1, Level::High, Speed::Low);
    let left_wheel_pid = Pid::new(0.00006, 0.00124, 0.000000728, 1.0);

    let _right_wheel_enc: Encoder<'_, TIM4, 400> = Encoder::new(p.TIM4, p.PB6, p.PB7);
    let right_wheel_pwm_pin = PwmPin::new_ch1(p.PB4, OutputType::PushPull);
    let _right_wheel_dir_pin = Output::new(p.PB5, Level::High, Speed::Low);
    let _right_wheel_break_pin = Output::new(p.PB3, Level::High, Speed::Low);
    let _right_wheel_pid = Pid::new(0.00006, 0.00124, 0.000000728, 1.0);

    let pwm = SimplePwm::new(
        p.TIM3,
        Some(right_wheel_pwm_pin),
        None,
        Some(left_wheel_pwm_pin),
        None,
        Hertz::hz(PWM_HZ),
        Default::default(),
    );

    let pwm_channels = pwm.split();
    let left_wheel_pwm_ch = pwm_channels.ch3;
    let _right_wheel_pwm_ch = pwm_channels.ch1;

    // Create motors
    let left_wheel = BldcMotor24H::new(
        left_wheel_enc,
        left_wheel_pwm_ch,
        left_wheel_dir_pin,
        left_wheel_break_pin,
        left_wheel_pid,
        PERIOD_S,
    );

    // let right_wheel = BldcMotor24H::new(
    //     right_wheel_enc,
    //     right_wheel_pwm_ch,
    //     right_wheel_dir_pin,
    //     right_wheel_break_pin,
    //     right_wheel_pid,
    //     PERIOD_S,
    // );

    // Create s_curve interpolator for left, right wheel
    let vel_limit_rad_s = rpm_to_rad_s(VEL_LIMIT_RPM);
    let left_s_curve_intper = SCurveInterpolator::new(
        vel_limit_rad_s,
        vel_limit_rad_s * 10.0,
        vel_limit_rad_s * 100.0,
        PERIOD_S,
    );
    // let right_s_curve_intper = left_s_curve_intper.clone();

    // Create motion controller for left, right wheel
    let left_motion_controller = Motion::new(left_s_curve_intper, left_wheel);
    // let right_motion_controller = Motion::new(right_s_curve_intper, right_wheel);

    // Create timer
    let low_level_timer = LLTimer::new(p.TIM15);
    low_level_timer.set_counting_mode(CountingMode::EdgeAlignedUp);
    low_level_timer.set_frequency(Hertz::hz((1.0 / PERIOD_S) as u32));
    low_level_timer.set_autoreload_preload(true);
    low_level_timer.enable_update_interrupt(true);
    low_level_timer.start();

    // Spawn tasks
    interrupt::TIM1_BRK_TIM15.set_priority(Priority::P6);
    let timer_spawner = EXECUTOR_TIMER.start(interrupt::TIM1_BRK_TIM15);
    timer_spawner
        .spawn(motion_task(left_motion_controller))
        .unwrap();

    let mut led = Output::new(p.PE8, Level::Low, Speed::Medium);
    loop {
        led.set_high();
        Timer::after_secs(1).await;
        led.set_low();
        Timer::after_secs(1).await;
    }
}
