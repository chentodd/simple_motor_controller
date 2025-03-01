pub mod communication;
pub mod main_window;
pub mod position_command_parser;
pub mod profile_measurement;
pub mod view;

use eframe::egui::Ui;
use main_window::ErrorType;

pub mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}

pub trait UiView {
    fn show(&mut self, ui: &mut Ui);
    fn take_request(&mut self) -> Option<ViewResponse>;
    fn handle_event(&mut self, event: ViewEvent);
}

pub enum ViewResponse {
    ConnectionStart(String),
    ConnectionStop,
    ErrorDismissed(ErrorType),
}

pub enum ViewEvent {
    None,
    ErrorOccurred(ErrorType, String),
    // add other event variants here if needed
}
