use std::io::Write;
use std::thread;
use std::time::Duration;

use micropb::PbEncoder;
use proto::motor_::Operation;
use utils::*;

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}
use proto::{
    command_::*,
    motor_::MotorRx,
};

fn main() {
    let mut _port = serialport::new("/dev/ttyUSB0", 115200)
        .open()
        .expect("Failed to open serial port");

    let mut cloned_port = _port.try_clone().expect("Failed to clone");

    let stream = Vec::<u8>::new();
    let pb_encoder = PbEncoder::new(stream);

    let output_packet_buffer = [0_u8; 128];
    let mut packet_encoder = PacketEncoder::new(output_packet_buffer);

    let mut input_packet_buffer = [0_u8; 128];
    let mut input_packet = Vec::<u8>::new();
    let mut packet_decoder = PacketDecoder::new();

    let mut rx_packet = CommandRx::default();
    let mut tx_packet = CommandTx::default();

    let mut left_motor_command = MotorRx::default();
    let mut right_motor_command = MotorRx::default();
    loop {
        // Test send
        left_motor_command.operation = Operation::IntpVel;
        left_motor_command.set_target_vel(500.0);

        right_motor_command.operation = Operation::IntpVel;
        right_motor_command.set_target_vel(-500.0);

        rx_packet.set_left_motor(left_motor_command.clone());
        rx_packet.set_right_motor(right_motor_command.clone());

        let output_packet = packet_encoder.create_packet(MessageId::CommandRx, pb_encoder.as_writer());
        cloned_port
            .write_all(output_packet)
            .expect("Failed to write to serial port");

        println!("send: {:?}", output_packet);

        // Test get
        input_packet_buffer.fill(0);
        let read_count = cloned_port.read(&mut input_packet_buffer);
        if let Ok(_read_count) = read_count {
            input_packet.extend_from_slice(&input_packet_buffer);
            if packet_decoder.is_packet_valid(&input_packet) {
                if packet_decoder.parse_proto_message(&input_packet, &mut tx_packet) {
                    let left_motor_data = tx_packet.left_motor.clone();
                    let right_motor_data = tx_packet.right_motor.clone();
                    input_packet.clear();

                    println!("get: {:?}", left_motor_data);
                    println!("get: {:?}", right_motor_data);
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}