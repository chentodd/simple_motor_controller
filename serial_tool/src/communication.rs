use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

use micropb::{MessageEncode, PbEncoder};
use serial_enumerator::get_serial_list;
use serialport::SerialPort;
use utils::*;

use crate::proto::command_::{CommandRx, CommandTx};

pub struct Settings;

impl Settings {
    pub fn get_port_names() -> Vec<String> {
        let port_info = get_serial_list();
        port_info.iter().map(|x| x.name.clone()).collect()
    }
}

pub struct Communication {
    motor_tx: (Sender<CommandTx>, Receiver<CommandTx>),
    motor_rx: (Sender<CommandRx>, Receiver<CommandRx>),
}

impl Communication {
    const BAUD_RATE: u32 = 115200;

    pub fn new() -> Self {
        Self {
            motor_tx: channel(),
            motor_rx: channel(),
        }
    }

    pub fn start(&mut self, port_name: &str) {
        let port1 = serialport::new(port_name, Self::BAUD_RATE)
            .open()
            .expect("Failed to open serial port");

        let port2 = port1.try_clone().expect("Failed to clone serial port");

        // TODO, check if it is ok to spawn thread like this
        thread::scope(|s| {
            s.spawn(|| {
                self.tx_task(port1);
            });
        });

        thread::scope(|s| {
            s.spawn(|| {
                self.rx_task(port2);
            });
        });
    }

    pub fn stop(&self) {
        // TODO, rember to link close window action to this function
    }

    fn tx_task(&mut self, mut serial_port: Box<dyn SerialPort>) {
        // TODO
        // 1. error handling?
        // 2. a methond to terminate thread
        // ref: https://users.rust-lang.org/t/using-arc-to-terminate-a-thread/81533/9

        let packet_buffer = [0_u8; 128];
        let mut packet_encoder = PacketEncoder::new(packet_buffer);

        loop {
            if let Ok(rx_data) = self.motor_rx.1.try_recv() {
                let stream = Vec::<u8>::new();
                let mut pb_encoder = PbEncoder::new(stream);

                rx_data.encode(&mut pb_encoder).unwrap();
                let output_packet =
                    packet_encoder.create_packet(MessageId::CommandRx, pb_encoder.as_writer());

                serial_port
                    .write_all(output_packet)
                    .expect("Failed to write to serial port");
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    fn rx_task(&mut self, mut serial_port: Box<dyn SerialPort>) {
        let mut packet_buffer = [0_u8; 128];
        let mut packet_decoder = PacketDecoder::new();
        let mut tx_packet = CommandTx::default();

        loop {
            let read_count = serial_port.read(&mut packet_buffer);
            if let Ok(_read_count) = read_count {
                if let Some(good_start_index) =
                    packet_decoder.get_valid_packet_index(&packet_buffer)
                {
                    if packet_decoder
                        .parse_proto_message(&packet_buffer[good_start_index..], &mut tx_packet)
                    {
                        self.motor_tx
                            .0
                            .send(tx_packet.clone())
                            .expect("Fail to send tx packet");
                    }
                }
            }
            serial_port.clear(serialport::ClearBuffer::Input).unwrap();
            thread::sleep(Duration::from_millis(200));
        }
    }
}
