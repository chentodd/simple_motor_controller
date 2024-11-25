#![no_std]
#![no_main]

use core::str;

use nucleo_f401re_rust::encoder::Encoder;
use nucleo_f401re_rust::motor::BldcMotor24H;
use nucleo_f401re_rust::pid::Pid;

use embedded_io_async::Read;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::signal::Signal;
use embassy_sync::pubsub::PubSubChannel;
use embassy_time::Timer;

use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::pac;
use embassy_stm32::peripherals::{self, TIM2, TIM3, TIM4};
use embassy_stm32::{bind_interrupts, usart};

use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, Timer as LLTimer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};

use embassy_stm32::usart::{BufferedUart, BufferedUartRx};

use heapless::Vec;
use static_cell::StaticCell;

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

const PERIOD_S: f32 = 0.005;
static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static CMD_VEL_CHANNEL: PubSubChannel<ThreadModeRawMutex, f32, 4, 1, 1> = PubSubChannel::new();

static TX_BUFFER: StaticCell<[u8; 32]> = StaticCell::new();
static RX_BUFFER: StaticCell<[u8; 32]> = StaticCell::new();

#[interrupt]
unsafe fn TIM5() {
    // Trigger the signal to notify the task
    TIMER_SIGNAL.signal(());
    pac::TIM5.sr().modify(|r| r.set_uif(false));
}

#[embassy_executor::task]
async fn control_wheel_speed(
    mut left_wheel: BldcMotor24H<'static, TIM2, TIM3>,
    mut right_wheel: BldcMotor24H<'static, TIM4, TIM3>,
) {
    let mut subscriber = CMD_VEL_CHANNEL.subscriber().unwrap();
    loop {
        TIMER_SIGNAL.wait().await;
        if let Some(left_cmd_vel) = subscriber.try_next_message_pure() {
            info!("ctrler, left: {}", left_cmd_vel);
            left_wheel.set_target_velocity(left_cmd_vel);
        }

        if let Some(right_cmd_vel) = subscriber.try_next_message_pure() {
            info!("ctrler, right: {}", right_cmd_vel);
            right_wheel.set_target_velocity(right_cmd_vel);
        }

        left_wheel.run_pid_velocity_control();
        right_wheel.run_pid_velocity_control();
    }
}

#[embassy_executor::task]
async fn read_data(mut rx: BufferedUartRx<'static>) {
    let mut read_data: Vec<u8, 64> = Vec::new();
    let publisher = CMD_VEL_CHANNEL.publisher().unwrap();

    loop {
        let mut raw_buffer: [u8; 8] = [0; 8];
        let read_count = rx.read(&mut raw_buffer).await;
        if let Ok(read_count) = read_count {
            for i in 0..read_count {
                let _ = read_data.push(raw_buffer[i]);
            }
        }

        if let Some(&last_ch) = read_data.last() {
            if last_ch == b'\r' || last_ch == b'\n' {
                {
                    let received_str = str::from_utf8(read_data.as_slice()).unwrap_or("");
                    let parts: Vec<&str, 2> = received_str.trim().split(',').collect();
                    if let (Ok(left_cmd_vel), Ok(right_cmd_vel)) =
                        (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                    {
                        
                        // info!("Get: {}, {}", left_cmd_vel, right_cmd_vel);
                        publisher.publish_immediate(left_cmd_vel);
                        publisher.publish_immediate(right_cmd_vel);
                    }
                }
                read_data.clear();
            }
        }
    }
}

bind_interrupts!(struct Irqs {
    USART6 => usart::BufferedInterruptHandler<peripherals::USART6>;
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

    // Create USART6
    let tx_buffer = TX_BUFFER.init([0; 32]);
    let rx_buffer = RX_BUFFER.init([0; 32]);
    let buffered_uart = BufferedUart::new(
        p.USART6,
        Irqs,
        p.PA12,
        p.PA11,
        tx_buffer,
        rx_buffer,
        usart::Config::default(),
    )
    .unwrap();

    let (_tx, rx) = buffered_uart.split();

    // Test
    spawner
        .spawn(control_wheel_speed(left_wheel, right_wheel))
        .unwrap();
    spawner.spawn(read_data(rx)).unwrap();

    loop {
        Timer::after_secs(1).await;
    }
}
