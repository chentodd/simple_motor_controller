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
    | TopicTy                     | MessageTy                     | Path          |
    | ----------                  | ----------                    | ----------    |
}


topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy                     | MessageTy                     | Path          | Cfg                |
    | ----------                  | ----------                    | ----------    | ----------         |
    | MotorProcessDataTopic       | (MotorId, MotorProcessData)   | "motor/data"  |                    |
}


#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub enum ControlMode {
    Position,
    Velocity,
    Stop,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub enum CommandError {
    PositionBufferFull,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub enum MotorId {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub enum MotorCommand {
    ControlMode(ControlMode),
    VelocityCommand(f32),
    PositionCommand(PositionCommand)
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct PositionCommand {
    pub displacement: f32,
    pub max_vel: f32,
    pub end_vel: f32,
}

#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct MotorProcessData {
    pub control_mode_display: ControlMode,
    pub actual_pos: f32,
    pub actual_vel: f32,
    pub intp_pos: f32,
    pub intp_vel: f32,
    pub intp_acc: f32,
    pub intp_jerk: f32,
}