use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};

use log::{debug, error};

use micropb::{MessageEncode, PbEncoder};
use serial_enumerator::get_serial_list;
use serialport::SerialPort;
use utils::*;

use crate::proto::command_::{CommandRx, CommandTx};
use crate::proto::motor_::MotorRx;

pub struct Settings;

impl Settings {
    pub fn get_port_names() -> Vec<String> {
        let port_info = get_serial_list();
        port_info.iter().map(|x| x.name.clone()).collect()
    }
}

#[derive(Clone, Copy)]
pub enum Error {
    FailToOpenSerialPort,
    FailToCloneSerialPort,
    FailToJoinThread,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FailToOpenSerialPort => write!(f, "Fail to open serial port"),
            Error::FailToCloneSerialPort => write!(f, "Fail to clone serial port"),
            Error::FailToJoinThread => write!(f, "Fail to join thread"),
        }
    }
}

pub struct Communication {
    command_rx_sender: Option<Sender<CommandRx>>,
    command_tx_recv: Option<Receiver<CommandTx>>,
    keep_alive: Arc<AtomicBool>,
    thread_handles: Vec<JoinHandle<()>>,
    motor_rx_data: Option<MotorRx>,
}

impl Drop for Communication {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl Communication {
    const BAUD_RATE: u32 = 115200;

    pub fn new() -> Self {
        Self {
            command_rx_sender: None,
            command_tx_recv: None,
            keep_alive: Arc::new(AtomicBool::new(false)),
            thread_handles: Vec::new(),
            motor_rx_data: None,
        }
    }

    pub fn reset(&mut self) {
        let _ = self.command_rx_sender.take();
        let _ = self.command_tx_recv.take();

        self.keep_alive.store(false, Ordering::Relaxed);
        self.thread_handles.clear();
    }

    pub fn start(&mut self, port_name: &str) -> Result<(), Error> {
        if !self.thread_handles.is_empty() {
            return Ok(());
        }

        let port1 = serialport::new(port_name, Self::BAUD_RATE)
            .open()
            .map_err(|_x| Error::FailToOpenSerialPort)?;

        let port2 = port1
            .try_clone()
            .map_err(|_x| Error::FailToCloneSerialPort)?;

        let (command_rx_sender, command_rx_recv) = mpsc::channel::<CommandRx>();
        self.command_rx_sender = Some(command_rx_sender);

        let (command_tx_sender, command_tx_recv) = mpsc::channel::<CommandTx>();
        self.command_tx_recv = Some(command_tx_recv);

        let (buffer_status_sender, buffer_status_recv) = mpsc::channel::<bool>();

        self.keep_alive.store(true, Ordering::Relaxed);

        let keep_alive = self.keep_alive.clone();
        let join_handle = thread::spawn(move || {
            Self::tx_task(command_rx_recv, buffer_status_recv, keep_alive, port1);
        });
        self.thread_handles.push(join_handle);

        let keep_alive = self.keep_alive.clone();
        let join_handle = thread::spawn(move || {
            Self::rx_task(command_tx_sender, buffer_status_sender, keep_alive, port2);
        });
        self.thread_handles.push(join_handle);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if self.thread_handles.is_empty() {
            return Ok(());
        }

        self.keep_alive.store(false, Ordering::Relaxed);
        while !self.thread_handles.is_empty() {
            let handle = self.thread_handles.remove(0);
            if let Err(_e) = handle.join() {
                error!("stop(), fail to join threads");
                return Err(Error::FailToJoinThread);
            }
        }

        Ok(())
    }

    pub fn set_rx_data(&mut self, data: MotorRx) {
        if let Some(sender) = self.command_rx_sender.as_ref() {
            if let Some(prev_data) = self.motor_rx_data.as_ref() {
                if *prev_data == data {
                    return;
                }
            }

            self.motor_rx_data = Some(data.clone());

            let mut rx_data = CommandRx::default();
            rx_data.set_left_motor(data);

            if             let Err(_e) = sender.send(rx_data) {
                error!("set_rx_data, fail to send data to channel, {_e}");
            }
            debug!("set_rx_data");
        }
    }

    pub fn get_tx_data(&self) -> Option<CommandTx> {
        if let Some(recv) = self.command_tx_recv.as_ref() {
            if let Ok(data) = recv.try_recv() {
                return Some(data);
            }
        }

        None
    }

    fn tx_task(
        cmd_recv: Receiver<CommandRx>,
        buffer_status_recv: Receiver<bool>,
        keep_alive: Arc<AtomicBool>,
        mut serial_port: Box<dyn SerialPort>,
    ) {
        let packet_buffer = [0_u8; 128];
        let mut packet_encoder = PacketEncoder::new(packet_buffer);

        loop {
            if !keep_alive.load(Ordering::Relaxed) {
                debug!("tx_task(), get stopped");
                break;
            }

            if let Ok(buffer_full) = buffer_status_recv.recv() {
                if buffer_full {
                    continue;
                }

                match cmd_recv.try_recv() {
                    Ok(rx_data) => {
                        debug!("tx_task(), rx_data: {rx_data:?}");

                        let stream = Vec::<u8>::new();
                        let mut pb_encoder = PbEncoder::new(stream);

                        rx_data.encode(&mut pb_encoder).unwrap();
                        let output_packet = packet_encoder
                            .create_packet(MessageId::CommandRx, pb_encoder.as_writer());

                        if let Err(_e) = serial_port.write_all(output_packet) {
                            error!("tx_task(), fail to write data to serial port, {_e}");
                        }
                    }
                    Err(_) => (),
                }
            }
        }

        debug!("tx_task(), end");
    }

    fn rx_task(
        cmd_sender: Sender<CommandTx>,
        buffer_status_sender: Sender<bool>,
        keep_alive: Arc<AtomicBool>,
        mut serial_port: Box<dyn SerialPort>,
    ) {
        let mut packet_buffer = [0_u8; 128];
        let mut packet_decoder = PacketDecoder::new();
        let mut tx_packet = CommandTx::default();

        loop {
            if !keep_alive.load(Ordering::Relaxed) {
                debug!("rx_task(), get stopped");
                break;
            }

            let read_count = serial_port.read(&mut packet_buffer);
            if let Ok(_read_count) = read_count {
                if let Some(good_start_index) =
                    packet_decoder.get_valid_packet_index(&packet_buffer)
                {
                    if packet_decoder
                        .parse_proto_message(&packet_buffer[good_start_index..], &mut tx_packet)
                    {
                        let buffer_full = tx_packet.left_motor.command_buffer_full;
                        
                        if let Err(_e) = buffer_status_sender.send(buffer_full) {
                            error!("rx_task(), fail to send bool data to channel, {_e}");
                        }

                        if let Err(_e) = cmd_sender.send(tx_packet.clone()) {
                            error!("rx_task(), fail to send serial data to channel, {_e}");
                        }
                    }
                }
            }

            serial_port.clear(serialport::ClearBuffer::Input).unwrap();
        }

        debug!("rx_task(), end");
    }
}
