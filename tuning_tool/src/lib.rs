use std::fmt::Display;

use eframe::egui::Ui;

use protocol::{ControlMode, MotorProcessData};

pub mod controller;
pub mod view;

const DEFAULT_CONTROL_MODE: ControlMode = ControlMode::Velocity;
const DEFAULT_GRAPH_SIZE: usize = 600;
pub trait UiView {
    fn show(&mut self, ui: &mut Ui);
    fn take_request(&mut self) -> Option<ViewRequest>;
    fn handle_event(&mut self, event: ViewEvent);
    fn reset(&mut self);
}

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ErrorType {
    #[default]
    None,
    StartError,
    StopError,
    ModeSwitchTimeout,
    ParseCommandError,
    CommunicationError,
}

#[derive(Default, Clone, Copy)]
pub struct ProfileData {
    intp_pos: f32,
    intp_vel: f32,
    intp_acc: f32,
    intp_jerk: f32,
    act_pos: f32,
    act_vel: f32,
}

impl ProfileData {
    pub fn from(motor_data: &MotorProcessData) -> Self {
        Self {
            intp_pos: motor_data.intp_pos,
            intp_vel: motor_data.intp_vel,
            intp_acc: motor_data.intp_acc,
            intp_jerk: motor_data.intp_jerk,
            act_pos: motor_data.actual_pos,
            act_vel: motor_data.actual_vel,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ProfileDataType {
    IntpPos,
    IntpVel,
    IntpAcc,
    IntpJerk,
    ActPos,
    ActVel,
}

impl Display for ProfileDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileDataType::IntpPos => write!(f, "intp_pos"),
            ProfileDataType::IntpVel => write!(f, "intp_vel"),
            ProfileDataType::IntpAcc => write!(f, "intp_acc"),
            ProfileDataType::IntpJerk => write!(f, "intp_jerk"),
            ProfileDataType::ActPos => write!(f, "act_pos"),
            ProfileDataType::ActVel => write!(f, "act_vel"),
        }
    }
}

#[derive(Debug)]
pub enum ViewRequest {
    // A request that wants to start connection with a port name from connection window
    ConnectionStart(String),
    // A request that wants to stop connection from connection window
    ConnectionStop,
    // A request that wants to clear error from error window
    ErrorDismiss(ErrorType),
    // A request that wants to change to target mode from control mode window
    ModeSwitch(ControlMode),
    // A request that cancels mode switching from control mode window
    ModeCancel,
    // A request that wants to control velocity from command window
    VelocityControl(f32),
    // A request that wants to control position from command window
    PositionControl(String),
}

#[derive(Clone)]
pub enum ViewEvent {
    None,
    // Send error type and error message to error window
    ErrorOccurred(ErrorType, String),
    // Send current connection status to connection windo
    ConnectionStatusUpdate(bool),
    // Send current operation mode to control mode window
    ControlModeUpdate((bool, ControlMode)),
    // Send internal operation mode request to control mode window and update
    // the tile of modal
    InternalStopModeRequest(String),
    // Send motor profile data to profile window to draw the graph
    ProfileDataUpdate(ProfileData),
}
