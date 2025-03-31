#![no_std]
#![no_main]

use core::f32;
use embassy_stm32::timer::GeneralInstance4Channel;
use fw::encoder::Encoder;
use fw::motion::Motion;
use fw::motor::BldcMotor24H;
use fw::pid::Pid;
use fw::proto::command_::*;
use fw::proto::motor_::{MotorRx, MotorTx, Operation};
use fw::rpm_to_rad_s;

use s_curve::*;
use utils::*;

#[cfg(any(
    feature = "debug-rx",
    feature = "debug-tx",
    feature = "debug-motor",
    feature = "debug-motion"
))]
use defmt::debug;
use {defmt_rtt as _, panic_probe as _};

use heapless::Vec;
use micropb::{MessageEncode, PbEncoder};

use embassy_executor::{InterruptExecutor, Spawner};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::{PubSubChannel, WaitResult};
use embassy_sync::signal::Signal;
use embassy_sync::watch::Watch;
use embassy_time::Timer;

use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_stm32::mode::Async;
use embassy_stm32::pac;
use embassy_stm32::peripherals::{self, TIM2, TIM3, TIM4};
use embassy_stm32::{bind_interrupts, usart};

use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, Timer as LLTimer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};

use embassy_stm32::usart::{Uart, UartRx, UartTx};

const PERIOD_S: f32 = 0.005;
const PWM_HZ: u32 = 20_000;
const VEL_LIMIT_RPM: f32 = 4000.0;

static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static LEFT_COMMAND_CHANNEL: PubSubChannel<CriticalSectionRawMutex, MotorRx, 16, 1, 1> =
    PubSubChannel::new();
static RIGHT_COMMAND_CHANNEL: PubSubChannel<CriticalSectionRawMutex, MotorRx, 16, 1, 1> =
    PubSubChannel::new();
static MOTION_DATA: Watch<CriticalSectionRawMutex, CommandTx, 1> = Watch::new();

static EXECUTOR_TIMER: InterruptExecutor = InterruptExecutor::new();

#[interrupt]
unsafe fn TIM5() {
    let sr = pac::TIM5.sr().read();
    if sr.uif() {
        pac::TIM5.sr().modify(|r| r.set_uif(false));
        EXECUTOR_TIMER.on_interrupt();
        TIMER_SIGNAL.signal(());
    }
}

bind_interrupts!(struct Irqs {
    USART6 => usart::InterruptHandler<peripherals::USART6>;
});

fn set_motion_data_helper<T1: GeneralInstance4Channel, T2: GeneralInstance4Channel>(
    target: &mut MotorTx,
    source: &Motion<'_, T1, T2>,
) {
    target.clear_intp_pos();
    target.clear_intp_vel();
    target.clear_intp_acc();
    target.clear_intp_jerk();

    target.operation_display = source.get_operation();
    if source.get_operation() == Operation::IntpPos {
        let intp_data = source.s_curve_intper.get_intp_data();
        target.set_intp_pos(intp_data.pos);
        target.set_intp_vel(intp_data.vel);
        target.set_intp_acc(intp_data.acc);
        target.set_intp_jerk(intp_data.jerk);
    }
    target.set_actual_pos(source.motor.encoder.get_act_position_in_rad());
    target.set_actual_vel(source.motor.encoder.get_act_velocity_in_rpm());
}

#[embassy_executor::task]
async fn motion_task(
    mut left_motion_controller: Motion<'static, TIM2, TIM3>,
    mut right_motion_controller: Motion<'static, TIM4, TIM3>,
) {
    let mut left_cmd_subscriber = LEFT_COMMAND_CHANNEL.subscriber().unwrap();
    let mut right_cmd_subscriber = RIGHT_COMMAND_CHANNEL.subscriber().unwrap();
    let motion_data_sender = MOTION_DATA.sender();

    let mut left_data = MotorTx::default();
    let mut right_data = MotorTx::default();
    let mut tx_data = CommandTx::default();
    loop {
        TIMER_SIGNAL.wait().await;

        if left_motion_controller.ready() {
            if let Some(left_command) = left_cmd_subscriber.try_next_message() {
                match left_command {
                    WaitResult::Lagged(_val) => {
                        #[cfg(feature = "debug-motion")]
                        debug!("left command lag: {}", _val);
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
                    WaitResult::Lagged(_val) => {
                        #[cfg(feature = "debug-motion")]
                        debug!("right command lag: {}", _val);
                    }
                    WaitResult::Message(command) => {
                        right_motion_controller.set_command(command);
                    }
                }
            }
        }

        left_motion_controller.run();
        right_motion_controller.run();

        left_data.command_buffer_full = left_cmd_subscriber.is_full();
        right_data.command_buffer_full = right_cmd_subscriber.is_full();
        set_motion_data_helper(&mut left_data, &left_motion_controller);
        set_motion_data_helper(&mut right_data, &right_motion_controller);

        tx_data.set_left_motor(left_data.clone());
        tx_data.set_right_motor(right_data.clone());
        motion_data_sender.send(tx_data.clone());
    }
}

#[embassy_executor::task]
async fn rx_task(mut rx: UartRx<'static, Async>) {
    let left_cmd_publisher = LEFT_COMMAND_CHANNEL.publisher().unwrap();
    let right_cmd_publisher = RIGHT_COMMAND_CHANNEL.publisher().unwrap();

    let mut packet_decoder = PacketDecoder::new();
    loop {
        let mut raw_buffer = [0_u8; 128];
        let read_count = rx.read_until_idle(&mut raw_buffer).await;

        if let Ok(_read_count) = read_count {
            let packet_slice = &mut &raw_buffer[..];
            while let Some(good_packet_index) = packet_decoder.get_valid_packet_index(&packet_slice)
            {
                let mut command_rx = CommandRx::default();

                if packet_decoder
                    .parse_proto_message(&packet_slice[good_packet_index..], &mut command_rx)
                {
                    #[cfg(feature = "debug-rx")]
                    debug!("parse ok, command_rx");

                    if command_rx.left_motor.operation == Operation::Stop {
                        left_cmd_publisher.clear();
                    }

                    if command_rx.right_motor.operation == Operation::Stop {
                        right_cmd_publisher.clear();
                    }

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

                    let len = packet_decoder.get_len() as usize;
                    *packet_slice = &packet_slice[len..];
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn tx_task(mut tx: UartTx<'static, Async>) {
    let mut motion_data_receiver = MOTION_DATA.receiver().unwrap();

    let output_packet_buffer = [0_u8; 128];
    let mut packet_encoder = PacketEncoder::new(output_packet_buffer);
    loop {
        let command_tx = motion_data_receiver.get().await;

        let mut stream = Vec::<u8, 128>::new();
        let mut pb_encoder = PbEncoder::new(&mut stream);
        match command_tx.encode(&mut pb_encoder) {
            Ok(()) => {
                let output_packet =
                    packet_encoder.create_packet(MessageId::CommandTx, pb_encoder.as_writer());
                match tx.write(output_packet).await {
                    Ok(()) => (),
                    Err(_err) => {
                        #[cfg(feature = "debug-tx")]
                        debug!("tx_task, fail to write packet, {}", _err);
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
    interrupt::TIM5.set_priority(Priority::P6);
    let timer_spawner = EXECUTOR_TIMER.start(interrupt::TIM5);
    timer_spawner
        .spawn(motion_task(left_motion_controller, right_motion_controller))
        .unwrap();

    spawner.spawn(rx_task(rx)).unwrap();
    spawner.spawn(tx_task(tx)).unwrap();

    loop {
        Timer::after_secs(3).await;
    }
}
