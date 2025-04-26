use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use log::{debug, error};
use nusb;
use tokio::sync::{Notify, watch};

use host::client::{Client, ClientError};
use protocol::*;

pub struct Settings;

impl Settings {
    pub fn get_usb_products() -> Vec<String> {
        let devices = nusb::list_devices().unwrap();
        devices
            .filter(|device| device.product_string().is_some())
            .map(|device| device.product_string().unwrap().to_string())
            .collect()
    }
}

struct MotorCommandQueue {
    queue: Mutex<VecDeque<MotorCommand>>,
    signal: Notify,
}

impl MotorCommandQueue {
    fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            signal: Notify::new(),
        }
    }

    fn send(&self, command: MotorCommand) {
        let mut queue = self.queue.lock().unwrap();

        if command == MotorCommand::Abort {
            queue.clear();
        }
        queue.push_back(command);

        self.signal.notify_one();
    }

    async fn process(&self) -> MotorCommand {
        // Wait until signal gets notified which means there are commands to process
        self.signal.notified().await;

        let mut queue = self.queue.lock().unwrap();
        queue.pop_front().unwrap()
    }
}

pub struct Communication {
    client: Client,
    command_queue: Arc<MotorCommandQueue>,
    process_data_send: Option<watch::Sender<MotorProcessData>>,
    process_data_recv: Option<watch::Receiver<MotorProcessData>>,
    prev_command: Option<MotorCommand>,
}

impl Drop for Communication {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl Communication {
    pub fn new(port_name: &str) -> Self {
        Self {
            client: Client::new(port_name),
            command_queue: Arc::new(MotorCommandQueue::new()),
            process_data_recv: None,
            process_data_send: None,
            prev_command: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), ClientError<Infallible>> {
        // Check `ping` to make sure the device is connected
        let _id = self.client.ping(0).await?;

        // Create motor status channel
        let (process_data_send, process_data_recv) =
            watch::channel::<MotorProcessData>(MotorProcessData::default());
        self.process_data_send = Some(process_data_send);
        self.process_data_recv = Some(process_data_recv);

        Ok(())
    }

    pub fn stop(&mut self) {
        // Drop receiver to stop async task
        self.process_data_recv.take();
    }

    pub fn send_motor_command(&self, data: MotorCommand) {
        if self.prev_command.is_some_and(|x| x == data) {
            return;
        }

        self.command_queue.send(data);
    }

    pub fn get_motor_process_data(&self) -> Option<MotorProcessData> {
        if let Some(recv) = self.process_data_recv.as_ref() {
            return Some(*recv.borrow());
        }

        None
    }

    pub async fn process_motor_command(&self) -> Result<(), ClientError<Infallible>> {
        // This queue is used to store the command from `command_queue`, and it will
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
        let mut buffer_full = false;

        loop {
            let motor_command = self.command_queue.process().await;

            if motor_command == MotorCommand::Abort {
                internal_command_cache.clear();
            }
            internal_command_cache.push_back(motor_command);

            if buffer_full {
                // Buffer is already full, wait until the buffer is empty
                continue;
            }

            if let Some(command) = internal_command_cache.front() {
                debug!("process_motor_command(), command: {command:?}");

                let err = self
                    .client
                    .set_motor_cmd(MotorId::Left, motor_command)
                    .await;

                match err {
                    Ok(_) => {
                        // If the command is sent successfully, pop it from the queue
                        internal_command_cache.pop_front();
                        buffer_full = false;
                    }
                    Err(e) => {
                        match e {
                            ClientError::Endpoint(CommandError::BufferFull(_)) => {
                                buffer_full = true;
                            },
                            ClientError::Comms(e) => {
                                error!("process_motor_command(), unexpected error: {e:?}");
                                break Err(ClientError::Comms(e));
                            },
                        }
                    }
                }
            }
        }
    }

    pub async fn process_motor_data(&self) {
        let sub = self
            .client
            .client
            .subscribe_multi::<protocol::MotorProcessDataTopic>(8)
            .await;

        if let Err(e) = sub {
            error!("process_motor_data(), subscribe error: {e:?}");
            return;
        }

        let mut sub = sub.unwrap();
        loop {
            match sub.recv().await {
                Ok(data) => {
                    if let Some(sender) = self.process_data_send.as_ref() {
                        if let Err(e) = sender.send(data.1) {
                            error!("process_motor_data(), failed to send data: {e}");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("process_motor_data(), recv error: {e:?}");
                    break;
                }
            };
        }
    }
}
