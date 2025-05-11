#![cfg_attr(not(feature = "use-std"), no_std)]

use postcard_rpc::{endpoints, topics, TopicDirection};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

pub type CommandSetResult = Result<(), CommandError>;

endpoints! {
    list = ENDPOINT_LIST;
    omit_std = true;
    | EndpointTy                  | RequestTy                     | ResponseTy          | Path               |
    | ----------                  | ----------                    | ----------          | ----------         |
    | SetMotorCommandEndPoint     | (MotorId, MotorCommand)       | CommandSetResult    | "motor_cmd/set"    |
}

topics! {
    list = TOPICS_IN_LIST;
    direction = TopicDirection::ToServer;
    omit_std = true;
    | TopicTy                     | MessageTy                     | Path          |
    | ----------                  | ----------                    | ----------    |
}


topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    omit_std = true;
    | TopicTy                     | MessageTy                     | Path            | Cfg                |
    | ----------                  | ----------                    | ----------      | ----------         |
    | MotorProcessDataTopic       | (MotorId, MotorProcessData)   | "motor/data"    |                    |
    | Mpu6050MotionDataTopic      | Mpu6050MotionData             | "mpu6050/data"  |                    |
}


#[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Clone, Copy, Default)]
pub enum ControlMode {
    Position,
    #[default]
    Velocity,
    StandStill,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub enum CommandError {
    BufferFull(MotorId),
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Clone, Copy, Default)]
pub struct PositionCommand {
    pub displacement: f32,
    pub vel_max: f32,
    pub vel_end: f32,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Clone, Copy)]
pub enum MotorId {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Clone, Copy)]
pub enum MotorCommand {
    Halt,
    VelocityCommand(f32),
    PositionCommand(PositionCommand)
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Clone, Copy, Default)]
pub struct MotorProcessData {
    pub control_mode_display: ControlMode,
    pub actual_pos: f32,
    pub actual_vel: f32,
    pub intp_pos: f32,
    pub intp_vel: f32,
    pub intp_acc: f32,
    pub intp_jerk: f32,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq, Default)]
pub struct Mpu6050MotionData {
    pub acc_x: i16,
    pub acc_y: i16,
    pub acc_z: i16,
    pub g_x: i16,
    pub g_y: i16,
    pub g_z: i16,
}

#[cfg(feature = "use-std")]
mod display_impl {
    use super::ControlMode;
    use std::fmt::Display;

    impl Display for ControlMode {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ControlMode::Position => write!(f, "Position"),
                ControlMode::Velocity => write!(f, "Velocity"),
                ControlMode::StandStill => write!(f, "StandStill"),
            }
        }
    }
}