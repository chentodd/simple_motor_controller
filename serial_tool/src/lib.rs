pub mod communication;
pub mod main_window;
pub mod position_command_parser;
pub mod profile_measurement;

pub mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}
