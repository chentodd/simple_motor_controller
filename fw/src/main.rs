#![no_std]
#![no_main]

use embassy_executor::{InterruptExecutor, Spawner};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_sync::watch::Watch;
use embassy_usb::{Config, UsbDevice};

use embassy_stm32::bind_interrupts;
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::i2c::{self};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_stm32::pac;
use embassy_stm32::peripherals::{self, TIM2, TIM3, TIM8};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, Timer as LLTimer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::usb;

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

use postcard_rpc::server::{Dispatch, Server};

use fw::{
    communication::communication::*,
    motion::{
        encoder::Encoder,
        motion::{Motion, MOTION_CMD_QUEUE_SIZE},
        motor::BldcMotor24H,
        pid::Pid,
    },
    rpm_to_rad_s,
    task::{
        motion_controller::{motion_task, TIMER_SIGNAL},
        motion_data_publisher::motor_data_publish_task,
        mpu6050_data_publisher::mpu6050_data_publish_task,
    },
};
use protocol::*;
use s_curve::*;

const PERIOD_S: f32 = 0.005;
const PWM_HZ: u32 = 20_000;
const VEL_LIMIT_RPM: f32 = 4000.0;

static EXECUTOR_TIMER: InterruptExecutor = InterruptExecutor::new();
static LEFT_MOTOR_CMD_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    MotorCommand,
    CHANNEL_SIZE,
    1,
    2,
> = PubSubChannel::new();
static RIGHT_MOTOR_CMD_CHANNEL: PubSubChannel<
    CriticalSectionRawMutex,
    MotorCommand,
    CHANNEL_SIZE,
    1,
    2,
> = PubSubChannel::new();
static LEFT_MOTOR_STATUS_WATCH: Watch<CriticalSectionRawMutex, MotorStatus, 2> = Watch::new();
static RIGHT_MOTOR_STATUS_WATCH: Watch<CriticalSectionRawMutex, MotorStatus, 2> = Watch::new();

bind_interrupts!(struct UsbIrqs {
    USB_LP_CAN_RX0 => usb::InterruptHandler<peripherals::USB>;
});

bind_interrupts!(struct I2cIrqs {
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
});

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
pub async fn usb_task(mut usb: UsbDevice<'static, AppDriver>) {
    // low level USB management
    info!("USB task started");
    usb.run().await;
}

fn usb_config() -> Config<'static> {
    let mut config = Config::new(0x16c0, 0x27DD);
    config.manufacturer = Some("tchen");
    config.product = Some("stm32-discovery");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    // config.device_class = 0xEF;
    // config.device_sub_class = 0x02;
    // config.device_protocol = 0x01;
    // config.composite_with_iads = true;

    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // System init
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

    let left_wheel_enc: Encoder<'_, TIM2, 400> = Encoder::new(p.TIM2, p.PD3, p.PD4);
    let left_wheel_pwm_pin = PwmPin::new_ch3(p.PB0, OutputType::PushPull);
    let left_wheel_dir_pin = Output::new(p.PA4, Level::High, Speed::Low);
    let left_wheel_break_pin = Output::new(p.PC1, Level::High, Speed::Low);
    let left_wheel_pid = Pid::new(0.00006, 0.00124, 0.000000728, 1.0);

    let right_wheel_enc: Encoder<'_, TIM8, 400> = Encoder::new(p.TIM8, p.PC6, p.PC7);
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
    let left_motion_controller =
        Motion::<CriticalSectionRawMutex, TIM2, TIM3, CHANNEL_SIZE, MOTION_CMD_QUEUE_SIZE>::new(
            left_s_curve_intper,
            left_wheel,
            LEFT_MOTOR_CMD_CHANNEL.subscriber().unwrap(),
        );
    let right_motion_controller =
        Motion::<CriticalSectionRawMutex, TIM8, TIM3, CHANNEL_SIZE, MOTION_CMD_QUEUE_SIZE>::new(
            right_s_curve_intper,
            right_wheel,
            RIGHT_MOTOR_CMD_CHANNEL.subscriber().unwrap(),
        );

    // Create timer
    let low_level_timer = LLTimer::new(p.TIM15);
    low_level_timer.set_counting_mode(CountingMode::EdgeAlignedUp);
    low_level_timer.set_frequency(Hertz::hz((1.0 / PERIOD_S) as u32));
    low_level_timer.set_autoreload_preload(true);
    low_level_timer.enable_update_interrupt(true);
    low_level_timer.start();

    // Create I2C
    let i2c = i2c::I2c::new(
        p.I2C1,
        p.PB6,
        p.PB7,
        I2cIrqs,
        p.DMA1_CH6,
        p.DMA1_CH7,
        Hertz(400_000),
        Default::default(),
    );

    // USB/RPC init
    let driver = usb::Driver::new(p.USB, UsbIrqs, p.PA12, p.PA11);
    let pbufs = PBUFS.take();
    let config = usb_config();

    let context = Context {
        left_motor_cmd_pub: LEFT_MOTOR_CMD_CHANNEL.publisher().unwrap(),
        right_motor_cmd_pub: RIGHT_MOTOR_CMD_CHANNEL.publisher().unwrap(),
        left_motor_status: LEFT_MOTOR_STATUS_WATCH.receiver().unwrap(),
        right_motor_status: RIGHT_MOTOR_STATUS_WATCH.receiver().unwrap(),
    };
    let (device, tx_impl, rx_impl) = STORAGE.init(driver, config, pbufs.tx_buf.as_mut_slice());

    let dispatcher = MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();
    let mut server: AppServer = Server::new(
        tx_impl,
        rx_impl,
        pbufs.rx_buf.as_mut_slice(),
        dispatcher,
        vkk,
    );

    // Spawn tasks
    spawner.must_spawn(usb_task(device));

    // Trigger motion task with timer interrupt
    interrupt::TIM1_BRK_TIM15.set_priority(Priority::P6);
    let timer_spawner = EXECUTOR_TIMER.start(interrupt::TIM1_BRK_TIM15);
    timer_spawner
        .spawn(motion_task(
            left_motion_controller,
            right_motion_controller,
            LEFT_MOTOR_STATUS_WATCH.sender(),
            RIGHT_MOTOR_STATUS_WATCH.sender(),
        ))
        .unwrap();

    spawner.must_spawn(motor_data_publish_task(
        LEFT_MOTOR_STATUS_WATCH.receiver().unwrap(),
        RIGHT_MOTOR_STATUS_WATCH.receiver().unwrap(),
        server.sender(),
        LEFT_MOTOR_CMD_CHANNEL.publisher().unwrap(),
        RIGHT_MOTOR_CMD_CHANNEL.publisher().unwrap(),
    ));

    spawner.must_spawn(mpu6050_data_publish_task(server.sender(), i2c));

    loop {
        let _ = server.run().await;
    }
}
