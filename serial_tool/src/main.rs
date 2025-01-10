use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

use micropb::{MessageEncode, PbEncoder};
use utils::*;

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}
use proto::{
    command_::*,
    motor_::{MotorRx, Operation},
};

fn main() {
    let mut port = serialport::new("/dev/ttyUSB0", 115200)
        .open()
        .expect("Failed to open serial port");

    let mut cloned_port = port.try_clone().expect("Failed to clone");

    let output_packet_buffer = [0_u8; 128];
    let mut packet_encoder = PacketEncoder::new(output_packet_buffer);
    let mut packet_decoder = PacketDecoder::new();

    let mut rx_packet = CommandRx::default();
    let mut tx_packet = CommandTx::default();

    let mut left_motor_command = MotorRx::default();
    let mut right_motor_command = MotorRx::default();

    // Test get
    thread::spawn(move || loop {
        let mut input_packet_buffer = [0_u8; 128];
        let read_count = cloned_port.read(&mut input_packet_buffer);
        if let Ok(_read_count) = read_count {
            //println!("{:?}", input_packet_buffer);
            if let Some(good_start_index) =
                packet_decoder.get_valid_packet_index(&input_packet_buffer)
            {
                if packet_decoder
                    .parse_proto_message(&input_packet_buffer[good_start_index..], &mut tx_packet)
                {
                    let _left_motor_data = tx_packet.left_motor.clone();
                    let _right_motor_data = tx_packet.right_motor.clone();
                    println!(
                        "get: {}, {}",
                        _left_motor_data.actual_vel, _right_motor_data.actual_vel
                    );
                }
            }
        }
        cloned_port.flush().unwrap();
        thread::sleep(Duration::from_millis(200));
    });

    let stdin = std::io::stdin();
    loop {
        let mut input_line = String::new();
        stdin.read_line(&mut input_line).expect("Fail to read line");

        let cmd_str_list = input_line
            .trim()
            .split(' ')
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();

        let left_cmd_vel = cmd_str_list[0]
            .parse::<f32>()
            .expect("Fail to pase the value to f32");

        let right_cmd_vel = cmd_str_list[1]
            .parse::<f32>()
            .expect("Fail to pase the value to f32");

        // Test send
        left_motor_command.operation = Operation::IntpVel;
        left_motor_command.set_target_vel(left_cmd_vel);

        right_motor_command.operation = Operation::IntpVel;
        right_motor_command.set_target_vel(right_cmd_vel);

        rx_packet.set_left_motor(left_motor_command.clone());
        rx_packet.set_right_motor(right_motor_command.clone());

        let stream = Vec::<u8>::new();
        let mut pb_encoder = PbEncoder::new(stream);

        rx_packet.encode(&mut pb_encoder).unwrap();
        let output_packet =
            packet_encoder.create_packet(MessageId::CommandRx, pb_encoder.as_writer());

        port.write_all(output_packet)
            .expect("Failed to write to serial port");
        port.flush().unwrap();

        // println!("send: {:?}", output_packet);
        thread::sleep(Duration::from_millis(200));
    }
}
