use embassy_stm32::peripherals::USB;
use embassy_stm32::usb;
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::pubsub::Publisher;
use embassy_sync::watch::Receiver;

use postcard_rpc::{
    define_dispatch,
    header::VarHeader,
    server::{
        impls::embassy_usb_v0_4::{
            dispatch_impl::{WireRxBuf, WireRxImpl, WireSpawnImpl, WireStorage, WireTxImpl},
            PacketBuffers,
        },
        Server,
    },
};
use static_cell::ConstStaticCell;

use protocol::*;

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

pub const CHANNEL_SIZE: usize = 48;

pub static PBUFS: ConstStaticCell<BufStorage> = ConstStaticCell::new(BufStorage::new());
pub static STORAGE: AppStorage = AppStorage::new();

pub type AppDriver = usb::Driver<'static, USB>;
pub type AppStorage = WireStorage<ThreadModeRawMutex, AppDriver, 256, 256, 64, 256>;
pub type BufStorage = PacketBuffers<1024, 1024>;
pub type AppTx = WireTxImpl<ThreadModeRawMutex, AppDriver>;
pub type AppRx = WireRxImpl<AppDriver>;
pub type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

#[derive(Clone, Copy)]
pub struct MotorStatus {
    pub id: MotorId,
    pub is_queue_full: bool,
    pub process_data: MotorProcessData,
}

pub struct Context {
    pub left_motor_cmd_pub:
        Publisher<'static, CriticalSectionRawMutex, MotorCommand, CHANNEL_SIZE, 1, 2>,
    pub right_motor_cmd_pub:
        Publisher<'static, CriticalSectionRawMutex, MotorCommand, CHANNEL_SIZE, 1, 2>,
    pub left_motor_status: Receiver<'static, CriticalSectionRawMutex, MotorStatus, 2>,
    pub right_motor_status: Receiver<'static, CriticalSectionRawMutex, MotorStatus, 2>,
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
