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
        | SetMotorCommandsEndPoint      | async     | set_motor_cmds_handler        |
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

async fn set_motor_cmd_helper(context: &mut Context, id: MotorId, cmd: MotorCommand) -> CommandSetResult {
    // Indicates the internal buffer in motion controller is full or not, need to check
    // this flag before sending commands to motion controller task
    //
    // There are 2 queues in target board:
    // 1. PubSubChannel, publish command to motion controller task
    // 2. Deque, queue used as a backup in motion controller
    //
    // The `can_push` will be set to false when
    // 1. The Deque in motion controller is full
    // 2. The PubSubChannel is full
    // In these cases, the handler will return `CommandError::BufferFull`.
    //
    // Currently, the size of PubSubChannel is more than Deque, so if user pushes too
    // many position commands, the Dequeu will be full first, and the further commands
    // will not be pushed to PubSubChannel
    let (queue_status, channel_pub) = match id {
        MotorId::Left => (&mut context.left_motor_status, &context.left_motor_cmd_pub),
        MotorId::Right => (
            &mut context.right_motor_status,
            &context.right_motor_cmd_pub,
        ),
    };

    // The `Halt` command has the highest priority, so it can be sent when the queue in motion
    // struct is full.
    let can_push = match cmd {
        MotorCommand::VelocityCommand(_) | MotorCommand::Halt => true,
        MotorCommand::PositionCommand(_) => !queue_status.changed().await.is_queue_full,
    };

    if can_push {
        channel_pub
            .try_publish(cmd)
            .map_err(|_e| CommandError::BufferFull(id as u8))
    } else {
        Err(CommandError::BufferFull(id as u8))
    }
}

async fn set_motor_cmd_handler(
    context: &mut Context,
    _header: VarHeader,
    rqst: (MotorId, MotorCommand),
) -> CommandSetResult {
    set_motor_cmd_helper(context, rqst.0, rqst.1).await
}

async fn set_motor_cmds_handler(
    context: &mut Context,
    _header: VarHeader,
    rqst: [(MotorId, MotorCommand); 2],
) -> CommandSetResult {
    let mut err_motor_id = 0_u8;
    for (id, cmd) in rqst {
        if let Err(e) = set_motor_cmd_helper(context, id, cmd).await {
            match e {
                CommandError::BufferFull(id) => err_motor_id |= id,
            }
        }
    }

    if err_motor_id == 0 {
        Ok(())
    } else {
        Err(CommandError::BufferFull(err_motor_id))
    }
}
