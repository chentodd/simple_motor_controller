use std::io::Write;
use std::thread;
use std::time::Duration;

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
    let mut cmd_proto_packet = Command::default();

    // Send out 4 bytes every second
    loop {
        cmd_proto_packet.set_left_wheel_target_vel(0.0);
        cmd_proto_packet.set_right_wheel_target_vel(0.0);
        cmd_proto_packet.encode(&mut encoder).unwrap();

        let send_packet = create_packet(MessageId::CommandVelId, encoder.as_writer());
        cloned_port
            .write_all(&send_packet)
            .expect("Failed to write to serial port");
        thread::sleep(Duration::from_millis(1000));

        println!("send: {:?}", send_packet);
    }
}

pub fn create_packet(message_id: MessageId, proto_message: &[u8]) -> Vec<u8> {
    let mut packet = Vec::<u8>::new();

    packet.push(message_id as u8);
    packet.extend_from_slice(&[0; LENGTH_TYPE_IN_BYTES]);
    packet.extend_from_slice(proto_message);
    packet.push(0);

    let length = packet.len() as u32;
    packet[1..=LENGTH_TYPE_IN_BYTES].copy_from_slice(&length.to_le_bytes());

    let length = length as usize;
    packet[length - 1] = calculate_crc(&packet[0..=length - 2]);

    packet
}
