use std::io::Write;
use std::time::Duration;
use std::thread;

use micropb::{MessageEncode, PbEncoder};
use utils::*;

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}
use proto::command_::*;

fn main() {
    let mut port = serialport::new("/dev/ttyUSB0", 115200)
        .open()
        .expect("Failed to open serial port");

    let mut cloned_port = port.try_clone().expect("Failed to clone");

    // Encode protobuf message
    let stream = Vec::<u8>::new();
    let mut encoder = PbEncoder::new(stream);
    let cmd_proto_packet = Command {
        left_wheel_target_vel: 100.0,
        right_wheel_target_vel: -100.0
    };
    cmd_proto_packet.encode(&mut encoder).unwrap();

    // Send out 4 bytes every second
    thread::spawn(move || loop {
        let send_packet = create_packet(MessageId::CommandVelId, encoder.as_writer());
        cloned_port
            .write_all(&send_packet)
            .expect("Failed to write to serial port");
        thread::sleep(Duration::from_millis(1000));
    });
}

pub fn create_packet(message_id: MessageId, proto_message: &[u8]) -> Vec<u8> {
    let mut packet = Vec::<u8>::new();

    packet.push(message_id as u8);
    packet.extend_from_slice(&[0; LENGTH_TYPE_IN_BYTES]);
    packet.extend_from_slice(proto_message);
    packet.push(calculate_crc(&packet));

    let length = packet.len() as u32;
    packet[1..=LENGTH_TYPE_IN_BYTES].copy_from_slice(&length.to_le_bytes());

    packet
}