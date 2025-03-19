use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};

use log::{debug, error};

use micropb::{MessageEncode, PbEncoder};
use serial_enumerator::get_serial_list;
use serialport::SerialPort;
use utils::*;

use crate::proto::command_::{CommandRx, CommandTx};
use crate::proto::motor_::{MotorRx, Operation};

pub struct Settings;

impl Settings {
    pub fn get_port_names() -> Vec<String> {
        let port_info = get_serial_list();
        port_info.iter().map(|x| x.name.clone()).collect()
    }
}

struct CommandRxQueue {
    queue: Mutex<VecDeque<CommandRx>>,
    cond_var: Condvar,
}

impl CommandRxQueue {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            cond_var: Condvar::new(),
        }
    }

    fn send(&self, command: CommandRx) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(command);
        self.cond_var.notify_one();
    }

    fn abort(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();

        let mut motor_command = MotorRx::default();
        motor_command.operation = Operation::Stop;

        let mut special_command = CommandRx::default();
        special_command.set_left_motor(motor_command);

        queue.push_back(special_command);
        self.cond_var.notify_one();
    }

    fn reset(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.clear();
    }

    fn process(&self) -> CommandRx {
        let mut queue = self.queue.lock().unwrap();
        while queue.is_empty() {
            queue = self.cond_var.wait(queue).unwrap();
        }

        queue.pop_front().unwrap()
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
    command_rx_queue: Arc<CommandRxQueue>,
    command_tx_recv: Option<Receiver<CommandTx>>,
    keep_rx_alive: Arc<AtomicBool>,
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
            command_rx_queue: Arc::new(CommandRxQueue::new()),
            command_tx_recv: None,
            keep_rx_alive: Arc::new(AtomicBool::new(false)),
            thread_handles: Vec::new(),
            motor_rx_data: None,
        }
    }

    pub fn reset(&mut self) {
        self.command_rx_queue.reset();
        self.command_tx_recv = None;
        self.keep_rx_alive.store(false, Ordering::SeqCst);
        self.thread_handles.clear();
        self.motor_rx_data = None;
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

        // Create command_tx channel
        let (command_tx_sender, command_tx_recv) = mpsc::channel::<CommandTx>();
        self.command_tx_recv = Some(command_tx_recv);

        // Create atomic bool that stores buffer status
        let buffer_status1 = Arc::new(AtomicBool::new(false));
        let buffer_status2 = buffer_status1.clone();

        // Create `tx_task`
        let command_rx_queue2 = self.command_rx_queue.clone();

        let join_handle = thread::spawn(move || {
            Self::tx_task(command_rx_queue2, buffer_status1, port1);
        });
        self.thread_handles.push(join_handle);

        // Create `rx_task`
        self.keep_rx_alive.store(true, Ordering::SeqCst);
        let keep_rx_alive_clone = self.keep_rx_alive.clone();

        let join_handle = thread::spawn(move || {
            Self::rx_task(
                command_tx_sender,
                buffer_status2,
                keep_rx_alive_clone,
                port2,
            );
        });
        self.thread_handles.push(join_handle);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        if self.thread_handles.is_empty() {
            return Ok(());
        }

        // Stop threads:
        // a. stop `tx_task` by sending a `Unspecified` operation
        // b. stop `rx_task` by setting bool value to false
        let mut rx_data = CommandRx::default();
        rx_data.set_left_motor(MotorRx {
            operation: Operation::Unspecified,
            ..Default::default()
        });

        self.command_rx_queue.send(rx_data);
        self.keep_rx_alive.store(false, Ordering::SeqCst);

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
        if let Some(prev_data) = self.motor_rx_data.as_ref() {
            if *prev_data == data {
                return;
            }
        }

        let operation = data.operation;
        self.motor_rx_data = Some(data.clone());

        let mut rx_data = CommandRx::default();
        rx_data.set_left_motor(data);

        debug!("set_rx_data: {rx_data:?}");
        match operation {
            // `Stop` is the special commands, it will be used when:
            // 1. Switch control modes
            // 2. Stop connection
            //
            // When this command is used, all the commands in `command_rx_queue` will
            // be cleared, and only `Stop` command will be placed in the queue.
            Operation::Stop => self.command_rx_queue.abort(),
            _ => self.command_rx_queue.send(rx_data),
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
        command_rx_queue: Arc<CommandRxQueue>,
        buffer_full: Arc<AtomicBool>,
        mut serial_port: Box<dyn SerialPort>,
    ) {
        let packet_buffer = [0_u8; 512];
        let mut packet_encoder = PacketEncoder::new(packet_buffer);

        // This queue is used to store the command from `command_rx_queue`, and it will
        // be used when sending position commands. Ex: The embedded board has 8 buffer
        // limit, but user wants to send 10 position commands:
        // 1. First 8 commands are sent
        // 2. `buffer_full` is set to true, need to wait embedded board to process commands
        // 3. 9th and 10th commands get stored in internal queue when `buffer_full` is true
        // 4. When `buffer_full` is false, the command in the internal queue will be sent to
        //    the board
        //
        // Also, this can be used when error appeared, and it will re-send the command
        let mut internal_command_cache = VecDeque::new();

        loop {
            let command_rx = command_rx_queue.process();

            internal_command_cache.push_back(command_rx);
            if let Some(command) = internal_command_cache.front() {
                debug!("tx_task(), command: {command:?}");

                match command.left_motor.operation {
                    // Brak the loop when receiving unspecified event
                    Operation::Unspecified => break,
                    // Check `buffer_full`. If it is needed, wait until it is set to false
                    Operation::IntpPos => {
                        if buffer_full.load(Ordering::Acquire) {
                            continue;
                        }
                    }
                    _ => (),
                }

                let stream = Vec::<u8>::new();
                let mut pb_encoder = PbEncoder::new(stream);

                command.encode(&mut pb_encoder).unwrap();
                let output_packet =
                    packet_encoder.create_packet(MessageId::CommandRx, pb_encoder.as_writer());

                if let Err(_e) = serial_port.write_all(output_packet) {
                    error!("tx_task(), fail to write data to serial port, {_e}");
                } else {
                    // Pop command when it is successfully sent to the serial port
                    internal_command_cache.pop_front();
                }
            }
        }

        debug!("tx_task(), end");
    }

    fn rx_task(
        command_tx_sender: Sender<CommandTx>,
        buffer_full: Arc<AtomicBool>,
        keep_rx_alive: Arc<AtomicBool>,
        mut serial_port: Box<dyn SerialPort>,
    ) {
        let mut packet_buffer = [0_u8; 512];
        let mut packet_decoder = PacketDecoder::new();
        let mut tx_packet = CommandTx::default();

        loop {
            if !keep_rx_alive.load(Ordering::SeqCst) {
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
                        buffer_full
                            .store(tx_packet.left_motor.command_buffer_full, Ordering::Release);

                        if let Err(_e) = command_tx_sender.send(tx_packet.clone()) {
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
