pub mod communication;
pub mod main_window;
pub mod position_command_parser;
pub mod profile_measurement;
pub mod view;

use eframe::egui::Ui;
use main_window::ErrorType;
use proto::motor_::Operation;

const DEFAULT_CONTROL_MODE: Operation = Operation::IntpVel;

pub mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}

pub trait UiView {
    fn show(&mut self, ui: &mut Ui);
    fn take_request(&mut self) -> Option<ViewRequest>;
    fn handle_event(&mut self, event: ViewEvent);
    fn reset(&mut self);
}

pub enum ViewRequest {
    // A request that wants to start connection with a port name from connection window
    StartConnection(String),
    // A request that wants to stop connection from connection window
    StopConnection,
    // A request that wants to clear error from error window
    ErrorDismissed(ErrorType),
    // A request that wants to change to target mode from control mode window
    ModeSwitch(Operation),
}

#[derive(Clone)]
pub enum ViewEvent {
    None,
    // Send error type and error message to error window
    ErrorOccurred(ErrorType, String),
    // Send current connection status to connection windo
    ConnectionStatusUpdate(bool),
    // Send current operation mode to control mode window
    ControlModeUpdate(Operation),
    // Send internal operation mode request to control mode window and update
    // the tile of modal
    InternalControlModeRequest((Operation, String)),
    // add other event variants here if needed
}
