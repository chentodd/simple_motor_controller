use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::pubsub::Publisher;
use embassy_sync::watch::Receiver;
use embassy_time::Timer;

use defmt::warn;
use postcard_rpc::server::{Sender, WireTxErrorKind};

use crate::communication::communication::{AppTx, MotorStatus, CHANNEL_SIZE};
use protocol::*;

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
