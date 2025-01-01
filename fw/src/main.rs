#![no_std]
#![no_main]

use core::f32;

use fw::encoder::Encoder;
use fw::motion::Motion;
use fw::motor::BldcMotor24H;
use fw::pid::Pid;
use fw::proto::command_::*;
use fw::proto::motor_::{MotorRx, MotorTx};
use fw::rpm_to_rad_s;
use fw::serial::{encode_packet, PacketDecoder};

use s_curve::*;
use utils::MessageId;

use defmt::debug;
use {defmt_rtt as _, panic_probe as _};

use heapless::Vec;
use micropb::{MessageEncode, PbEncoder};

use embassy_executor::Spawner;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::pubsub::{PubSubChannel, WaitResult};
use embassy_sync::signal::Signal;
use embassy_time::Timer;

use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::pac;
use embassy_stm32::peripherals::{self, TIM2, TIM3, TIM4};
use embassy_stm32::{bind_interrupts, usart};

use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, Timer as LLTimer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};

use embassy_stm32::usart::{Uart, UartRx, UartTx};

const PERIOD_S: f32 = 0.001;
const PWM_HZ: u32 = 20_000;
const VEL_LIMIT_RPM: f32 = 4000.0;

static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static LEFT_COMMAND_CHANNEL: PubSubChannel<ThreadModeRawMutex, MotorRx, 16, 1, 1> =
    PubSubChannel::new();
static RIGHT_COMMAND_CHANNEL: PubSubChannel<ThreadModeRawMutex, MotorRx, 16, 1, 1> =
    PubSubChannel::new();
static LEFT_DATA_CHANNEL: PubSubChannel<ThreadModeRawMutex, MotorTx, 16, 1, 1> =
    PubSubChannel::new();
static RIGHT_DATA_CHANNEL: PubSubChannel<ThreadModeRawMutex, MotorTx, 16, 1, 1> =
    PubSubChannel::new();

#[interrupt]
unsafe fn TIM5() {
    // Trigger the signal to notify the task
    TIMER_SIGNAL.signal(());
    pac::TIM5.sr().modify(|r| r.set_uif(false));
}

#[embassy_executor::task]
async fn motion_task(
    mut left_motion_controller: Motion<'static, TIM2, TIM3>,
    mut right_motion_controller: Motion<'static, TIM4, TIM3>,
) {
    let mut left_cmd_subscriber = LEFT_COMMAND_CHANNEL.subscriber().unwrap();
    let mut right_cmd_subscriber = RIGHT_COMMAND_CHANNEL.subscriber().unwrap();

    let left_data_publisher = LEFT_DATA_CHANNEL.publisher().unwrap();
    let right_data_publisher = RIGHT_DATA_CHANNEL.publisher().unwrap();

    let mut left_data = MotorTx::default();
    let mut right_data = MotorTx::default();

    loop {
        TIMER_SIGNAL.wait().await;

        if left_motion_controller.ready() {
            if let Some(left_command) = left_cmd_subscriber.try_next_message() {
                match left_command {
                    WaitResult::Lagged(val) => {
                        #[cfg(feature = "debug-motion")]
                        debug!("left command lag: {}", val);
                    }
                    WaitResult::Message(command) => {
                        left_motion_controller.set_command(command);
                    }
                }
            }
        }

        if right_motion_controller.ready() {
            if let Some(right_command) = right_cmd_subscriber.try_next_message() {
                match right_command {
                    WaitResult::Lagged(val) => {
                        #[cfg(feature = "debug-motion")]
                        debug!("right command lag: {}", val);
                    }
                    WaitResult::Message(command) => {
                        right_motion_controller.set_command(command);
                    }
                }
            }
        }

        left_motion_controller.run();
        right_motion_controller.run();

        left_data.operation_display = left_motion_controller.get_operation();
        left_data.command_buffer_full = left_cmd_subscriber.is_full();
        left_data.set_actual_pos(left_motion_controller.get_actual_position());
        left_data.set_actual_vel(left_motion_controller.get_actual_velocity());

        right_data.operation_display = right_motion_controller.get_operation();
        right_data.command_buffer_full = right_cmd_subscriber.is_full();
        right_data.set_actual_pos(right_motion_controller.get_actual_position());
        right_data.set_actual_vel(right_motion_controller.get_actual_velocity());

        left_data_publisher.publish_immediate(left_data.clone());
        right_data_publisher.publish_immediate(right_data.clone());
    }
}

#[embassy_executor::task]
async fn rx_task(mut rx: UartRx<'static, Async>) {
    let left_cmd_publisher = LEFT_COMMAND_CHANNEL.publisher().unwrap();
    let right_cmd_publisher = RIGHT_COMMAND_CHANNEL.publisher().unwrap();

    let mut packet_decoder = PacketDecoder::new();
    let mut command_rx = CommandRx::default();

    loop {
        let mut raw_buffer = [0_u8; 128];
        let read_count = rx.read_until_idle(&mut raw_buffer).await;
        if let Ok(_read_count) = read_count {
            if packet_decoder.is_packet_valid(&raw_buffer) {
                if packet_decoder.parse_proto_message(&raw_buffer, &mut command_rx) {
                    #[cfg(feature = "debug-rx")]
                    debug!("parse ok, command_rx");

                    if !left_cmd_publisher.is_full() {
                        match left_cmd_publisher.try_publish(command_rx.left_motor.clone()) {
                            Ok(()) => (),
                            Err(_) => {
                                #[cfg(feature = "debug-rx")]
                                debug!("left_publisher, fail to publish");
                            }
                        }
                    }

                    if !right_cmd_publisher.is_full() {
                        match right_cmd_publisher.try_publish(command_rx.right_motor.clone()) {
                            Ok(()) => (),
                            Err(_) => {
                                #[cfg(feature = "debug-rx")]
                                debug!("right_publisher, fail to publish");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn tx_task(mut tx: UartTx<'static, Async>) {
    let mut left_data_subscriber = LEFT_DATA_CHANNEL.subscriber().unwrap();
    let mut right_data_subscriber = RIGHT_DATA_CHANNEL.subscriber().unwrap();

    let mut stream = Vec::<u8, 128>::new();
    let mut packet_encoder = PbEncoder::new(&mut stream);
    let mut command_tx = CommandTx::default();

    loop {
        match left_data_subscriber.try_next_message_pure() {
            Some(left_data) => command_tx.set_left_motor(left_data),
            None => command_tx.clear_left_motor(),
        }

        match right_data_subscriber.try_next_message_pure() {
            Some(right_data) => command_tx.set_right_motor(right_data),
            None => command_tx.clear_right_motor(),
        }

        match command_tx.encode(&mut packet_encoder) {
            Ok(()) => {
                let output_packet = encode_packet(MessageId::CommandTx, packet_encoder.as_writer());
                match tx.write(&output_packet).await {
                    Ok(()) => (),
                    Err(err) => {
                        #[cfg(feature = "debug-tx")]
                        debug!("tx_task, fail to write packet, {}", err);
                    }
                }
            }
            Err(_) => {
                #[cfg(feature = "debug-tx")]
                debug!("tx_task, fail to encode packet");
            }
        }
    }
}

bind_interrupts!(struct Irqs {
    USART6 => usart::InterruptHandler<peripherals::USART6>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Init hardware
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
        Hertz::hz(PWM_HZ),
        Default::default(),
    );

    let pwm_channels = pwm.split();
    let left_wheel_pwm_ch = pwm_channels.ch3;
    let right_wheel_pwm_ch = pwm_channels.ch1;

    // Create motors
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

    // Create s_curve interpolator for left, right wheel
    let vel_limit_rad_s = rpm_to_rad_s(VEL_LIMIT_RPM);
    let left_s_curve_intper = SCurveInterpolator::new(
        vel_limit_rad_s,
        vel_limit_rad_s * 10.0,
        vel_limit_rad_s * 100.0,
        PERIOD_S,
    );
    let right_s_curve_intper = left_s_curve_intper.clone();

    // Create motion controller for left, right wheel
    let left_motion_controller = Motion::new(left_s_curve_intper, left_wheel);
    let right_motion_controller = Motion::new(right_s_curve_intper, right_wheel);

    // Create timer
    let low_level_timer = LLTimer::new(p.TIM5);
    low_level_timer.set_counting_mode(CountingMode::EdgeAlignedUp);
    low_level_timer.set_frequency(Hertz::hz((1.0 / PERIOD_S) as u32));
    low_level_timer.set_autoreload_preload(true);
    low_level_timer.enable_update_interrupt(true);
    low_level_timer.start();
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::TIM5);
    }

    // Create USART6 with DMA
    let usart = Uart::new(
        p.USART6,
        p.PA12,
        p.PA11,
        Irqs,
        p.DMA2_CH6,
        p.DMA2_CH1,
        usart::Config::default(),
    )
    .unwrap();

    let (tx, rx) = usart.split();

    // Spawn tasks
    spawner
        .spawn(motion_task(left_motion_controller, right_motion_controller))
        .unwrap();
    spawner.spawn(rx_task(rx)).unwrap();
    spawner.spawn(tx_task(tx)).unwrap();

    loop {
        Timer::after_secs(1).await;
    }
}
