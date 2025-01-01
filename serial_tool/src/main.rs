use std::io::Write;
use std::thread;
use std::time::Duration;

use micropb::{MessageEncode, PbEncoder};
use proto::motor_::Operation;
use utils::*;

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}
use proto::{command_::*, motor_::{MotorRx, MotorTx}};

fn main() {
    let mut port = serialport::new("/dev/ttyUSB0", 115200)
        .open()
        .expect("Failed to open serial port");

    let mut cloned_port = port.try_clone().expect("Failed to clone");

    // Encode protobuf message
    let stream = Vec::<u8>::new();
    let mut encoder = PbEncoder::new(stream);

    let mut rx_packet = CommandRx::default();
    let mut tx_packet = CommandTx::default();

    let mut left_motor_command = MotorRx::default();
    let mut right_motor_command = MotorRx::default();

    let mut left_motor_data = MotorTx::default();
    let mut right_motor_data = MotorTx::default();

    loop {
        // Test
        left_motor_command.operation = Operation::IntpVel;
        left_motor_command.set_target_vel(500.0);

        right_motor_command.operation = Operation::IntpVel;
        right_motor_command.set_target_vel(-500.0);

        rx_packet.set_left_motor(left_motor_command.clone());
        rx_packet.set_right_motor(right_motor_command.clone());

        let send_packet = create_packet(MessageId::CommandRx, encoder.as_writer());
        cloned_port
            .write_all(&send_packet)
            .expect("Failed to write to serial port");

        println!("send: {:?}", send_packet);

        thread::sleep(Duration::from_millis(100));
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
