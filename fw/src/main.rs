#![no_std]
#![no_main]

use defmt::{info, warn};
use embassy_executor::{InterruptExecutor, Spawner};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::pubsub::{PubSubChannel, Publisher};
use embassy_sync::signal::Signal;
use embassy_sync::watch::{Receiver, Sender as WatchSender, Watch};
use embassy_time::Timer;
use embassy_usb::{Config, UsbDevice};

use embassy_stm32::bind_interrupts;
use embassy_stm32::gpio::{Level, Output, OutputType, Speed};
use embassy_stm32::interrupt;
use embassy_stm32::interrupt::{InterruptExt, Priority};
use embassy_stm32::pac;
use embassy_stm32::peripherals::{self, TIM2, TIM3, TIM4, USB};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::{CountingMode, Timer as LLTimer};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::usb;
use postcard_rpc::server::WireTxErrorKind;

use {defmt_rtt as _, panic_probe as _};

use postcard_rpc::{
    define_dispatch,
    header::VarHeader,
    server::{
        impls::embassy_usb_v0_4::{
            dispatch_impl::{WireRxBuf, WireRxImpl, WireSpawnImpl, WireStorage, WireTxImpl},
            PacketBuffers,
        },
        Dispatch, Sender, Server,
    },
};

use static_cell::ConstStaticCell;

use fw::encoder::Encoder;
use fw::motion::Motion;
use fw::motor::BldcMotor24H;
use fw::pid::Pid;
use fw::rpm_to_rad_s;
use protocol::*;
use s_curve::*;

// control loop
#[derive(Clone, Copy)]
pub struct MotorStatus {
    pub id: MotorId,
    pub is_queue_full: bool,
    pub process_data: MotorProcessData,
}

const PERIOD_S: f32 = 0.005;
const PWM_HZ: u32 = 20_000;
const VEL_LIMIT_RPM: f32 = 4000.0;

// The `CHANNEL_SIZE` is used in `PubSubChannel` and `MOTION_CMD_QUEUE_SIZE` is used
// in motion struct. If the queue in motion struct is full, I want to make sure there
// are spaces in `PubSubChannel`, so `Halt` command can be sent to motion struct.
// And for the other commands, since they don't have the same priority as `Halt`, the
// sender needs to wait until there are spaces in the queue.
const CHANNEL_SIZE: usize = 48;
const MOTION_CMD_QUEUE_SIZE: usize = 32;

static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();
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

// postcard-rpc
pub struct Context {
    pub left_motor_cmd_pub:
        Publisher<'static, CriticalSectionRawMutex, MotorCommand, CHANNEL_SIZE, 1, 2>,
    pub right_motor_cmd_pub:
        Publisher<'static, CriticalSectionRawMutex, MotorCommand, CHANNEL_SIZE, 1, 2>,
    pub left_motor_status: Receiver<'static, CriticalSectionRawMutex, MotorStatus, 2>,
    pub right_motor_status: Receiver<'static, CriticalSectionRawMutex, MotorStatus, 2>,
}

type AppDriver = usb::Driver<'static, USB>;
type AppStorage = WireStorage<ThreadModeRawMutex, AppDriver, 256, 256, 64, 256>;
type BufStorage = PacketBuffers<1024, 1024>;
type AppTx = WireTxImpl<ThreadModeRawMutex, AppDriver>;
type AppRx = WireRxImpl<AppDriver>;
type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

static PBUFS: ConstStaticCell<BufStorage> = ConstStaticCell::new(BufStorage::new());
static STORAGE: AppStorage = AppStorage::new();

define_dispatch! {
    app: MyApp;
    spawn_fn: spawn_fn;
    tx_impl: AppTx;
    spawn_impl: WireSpawnImpl;
    context: Context;

    endpoints: {
        list: ENDPOINT_LIST;

        | EndpointTy                    | kind      | handler                       |
        | ----------                    | ----      | -------                       |
        | SetMotorCommandEndPoint       | async     | set_motor_cmd_handler         |
    };
    topics_in: {
        list: TOPICS_IN_LIST;

        | TopicTy                       | kind      | handler                       |
        | ----------                    | ----      | -------                       |
    };
    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

bind_interrupts!(struct Irqs {
    USB_LP_CAN_RX0 => usb::InterruptHandler<peripherals::USB>;
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
async fn motion_task(
    mut left_motion_controller: Motion<
        'static,
        CriticalSectionRawMutex,
        TIM2,
        TIM3,
        CHANNEL_SIZE,
        MOTION_CMD_QUEUE_SIZE,
    >,
    mut right_motion_controller: Motion<
        'static,
        CriticalSectionRawMutex,
        TIM4,
        TIM3,
        CHANNEL_SIZE,
        MOTION_CMD_QUEUE_SIZE,
    >,
    left_motor_status: WatchSender<'static, CriticalSectionRawMutex, MotorStatus, 2>,
    right_motor_status: WatchSender<'static, CriticalSectionRawMutex, MotorStatus, 2>,
) {
    loop {
        TIMER_SIGNAL.wait().await;

        left_motion_controller.read_cmd_from_queue();
        right_motion_controller.read_cmd_from_queue();

        left_motion_controller.run();
        right_motion_controller.run();

        left_motor_status.send(MotorStatus {
            id: MotorId::Left,
            is_queue_full: left_motion_controller.is_queue_full(),
            process_data: left_motion_controller.get_motor_process_data(),
        });

        right_motor_status.send(MotorStatus {
            id: MotorId::Right,
            is_queue_full: right_motion_controller.is_queue_full(),
            process_data: right_motion_controller.get_motor_process_data(),
        });
    }
}

#[embassy_executor::task]
pub async fn usb_task(mut usb: UsbDevice<'static, AppDriver>) {
    // low level USB management
    info!("USB task started");
    usb.run().await;
}

#[embassy_executor::task]
pub async fn motor_data_publish_task(
    mut left_motor_status: Receiver<'static, CriticalSectionRawMutex, MotorStatus, 2>,
    mut right_motor_status: Receiver<'static, CriticalSectionRawMutex, MotorStatus, 2>,
    app_sender: Sender<AppTx>,
    left_command_pub: Publisher<'static, CriticalSectionRawMutex, MotorCommand, CHANNEL_SIZE, 1, 2>,
    right_command_pub: Publisher<
        'static,
        CriticalSectionRawMutex,
        MotorCommand,
        CHANNEL_SIZE,
        1,
        2,
    >,
) {
    let mut left_motor_topic_seq = 0_u8;
    let mut right_motor_topic_seq = 0_u8;
    let mut connected = false;

    loop {
        let left_motor_status = left_motor_status.get().await;
        let right_motor_status = right_motor_status.get().await;

        // Here, I use the error to check if connection is broken. If the board
        // is previously connected, and `Timeout` error is triggered when
        // publishing the data, then the connection is treated as broken. In
        // this case, I will send a `Halt` command to motion struct to stop
        // motor.
        // Also, because we publish the data on the same communication bus, so
        // I only check the error when publishing left motor topic data.
        if let Err(e) = app_sender
            .publish::<MotorProcessDataTopic>(
                left_motor_topic_seq.into(),
                &(left_motor_status.id, left_motor_status.process_data),
            )
            .await
        {
            match e {
                WireTxErrorKind::Timeout => {
                    if connected {
                        connected = false;
                        let _ = left_command_pub.try_publish(MotorCommand::Halt);
                        let _ = right_command_pub.try_publish(MotorCommand::Halt);
                        warn!("connection is lost, halt motors");
                    }
                }
                _ => (),
            }
        } else {
            connected = true;
        }

        let _ = app_sender
            .publish::<MotorProcessDataTopic>(
                right_motor_topic_seq.into(),
                &(right_motor_status.id, right_motor_status.process_data),
            )
            .await;

        left_motor_topic_seq = left_motor_topic_seq.wrapping_add(1);
        right_motor_topic_seq = right_motor_topic_seq.wrapping_add(1);

        // It might not be a good idea to use 1ms delay here, but I'm using it to prevent
        // the task consumes all the resources and block USB task.
        Timer::after_millis(1).await;
    }
}

async fn set_motor_cmd_handler(
    context: &mut Context,
    _header: VarHeader,
    rqst: (MotorId, MotorCommand),
) -> CommandSetResult {
    // Indicates the internal buffer in motion controller is full or not, need to check
    // this flag before sending commands to motion controller task
    //
    // There are 2 queues in target board:
    // 1. PubSubChannel, publish command to motion controller task
    // 2. Deque, queue used as a backup in motion controller
    //
    // If the Deque in motion controller is full then, this flag will be set to true then
    // the handler will return error.
    // (This could happen if a batch of position commands are pushed to the Deque, and
    // motion controller is still processing it).
    //
    // If the Deque in motion controller is not full but commands are published too fast
    // and all the spaces in PubSubChannel is consumed, then the handler will return error.

    let (queue_status, channel_pub) = match rqst.0 {
        MotorId::Left => (&mut context.left_motor_status, &context.left_motor_cmd_pub),
        MotorId::Right => (
            &mut context.right_motor_status,
            &context.right_motor_cmd_pub,
        ),
    };

    // The `Halt` command has the highest priority, so it can be sent when the queue in motion
    // struct is full.
    let can_push = match rqst.1 {
        MotorCommand::VelocityCommand(_) | MotorCommand::Halt => true,
        MotorCommand::PositionCommand(_) => !queue_status.changed().await.is_queue_full,
    };

    if can_push {
        channel_pub
            .try_publish(rqst.1)
            .map_err(|_e| CommandError::BufferFull(rqst.0))
    } else {
        Err(CommandError::BufferFull(rqst.0))
    }
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
    let left_motion_controller =
        Motion::<CriticalSectionRawMutex, TIM2, TIM3, CHANNEL_SIZE, MOTION_CMD_QUEUE_SIZE>::new(
            left_s_curve_intper,
            left_wheel,
            LEFT_MOTOR_CMD_CHANNEL.subscriber().unwrap(),
        );
    let right_motion_controller =
        Motion::<CriticalSectionRawMutex, TIM4, TIM3, CHANNEL_SIZE, MOTION_CMD_QUEUE_SIZE>::new(
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

    // USB/RPC init
    let driver = usb::Driver::new(p.USB, Irqs, p.PA12, p.PA11);
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
    spawner.must_spawn(usb_task(device));

    // Spawn other tasks
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

    loop {
        let _ = server.run().await;
        Timer::after_millis(1).await;
    }
}
