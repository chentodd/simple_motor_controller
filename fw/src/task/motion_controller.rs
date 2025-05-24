use embassy_stm32::peripherals::{TIM2, TIM3, TIM8};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_sync::watch::Sender as WatchSender;

use crate::communication::communication::{MotorStatus, CHANNEL_SIZE};
use crate::motion::motion::{Motion, MOTION_CMD_QUEUE_SIZE};
use protocol::*;

pub static TIMER_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

#[embassy_executor::task]
pub async fn motion_task(
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
        TIM8,
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
