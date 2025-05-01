use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use log::{debug, error, warn};
use postcard_rpc::host_client::{HostErr, MultiSubRxError};
use tokio::select;
use tokio::sync::{Notify, watch};

use host::client::{Client, ClientError};
use protocol::*;

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

struct MotorCommandActor {
    client: Arc<Client>,
    command_queue: Arc<MotorCommandQueue>,
    cancel_actor_recv: watch::Receiver<bool>,
    task_err: watch::Sender<Result<(), String>>,
}

impl MotorCommandActor {
    async fn run(&mut self) {
        let result = self.run_internal().await;
        if let Err(e) = result {
            self.task_err
                .send(Err(format!("{e:?}")))
                .expect("Failed to send result in motor command actor");

            error!("MotorCommandActor error: {e:?}");
        }
    }

    async fn run_internal(&mut self) -> Result<(), ClientError<CommandError>> {
        // This queue is used to store the command from `command_queue`, and it will
        // be used when sending position commands. Ex: The embedded board has 8 buffer
        // limit, but user wants to send 10 position commands:
        // 1. First 8 commands are sent
        // 2. `buffer_full` is set to true, need to wait embedded board to process commands
        // 3. 9th and 10th commands get stored in internal queue when `buffer_full` is true
        // 4. When `buffer_full` is false, the command in the internal queue will be sent to
        //    the board

        let mut buffer_full = false;
        let mut internal_command_cache = VecDeque::new();

        let _id = self
            .client
            .ping(0)
            .await
            .map_err(|_x| ClientError::Comms(HostErr::Closed))?;

        loop {
            select! {
                motor_command = self.command_queue.process() => {
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
                            Err(e) => match e {
                                ClientError::Endpoint(CommandError::BufferFull(_)) => {
                                    buffer_full = true;
                                }
                                ClientError::Comms(e) => {
                                    error!("process_motor_command(), unexpected error: {e:?}");
                                    break Err(ClientError::Comms(e));
                                },
                            },
                        }
                    }
                },
                flag = self.cancel_actor_recv.changed() => {
                    if flag.is_ok() {
                        debug!("process_motor_command(), cancel actor");
                        break Ok(());
                    }
                }
            }
        }
    }
}

struct MotorDataActor {
    client: Arc<Client>,
    data_send: watch::Sender<MotorProcessData>,
    cancel_actor_recv: watch::Receiver<bool>,
    task_err: watch::Sender<Result<(), String>>,
}

impl MotorDataActor {
    async fn run(&mut self) {
        let result = self.run_internal().await;
        if let Err(e) = result {
            self.task_err
                .send(Err(format!("{e:?}")))
                .expect("Failed to send result in motor data actor");

            error!("MotorDataActor error: {e:?}");
        }
    }

    async fn run_internal(&mut self) -> Result<(), ClientError<Infallible>> {
        let mut sub = self
            .client
            .client
            .subscribe_multi::<protocol::MotorProcessDataTopic>(8)
            .await
            .map_err(|_x| ClientError::Comms(HostErr::Closed))?;

        // Check `ping` to make sure the device is connected
        let _id = self.client.ping(0).await?;

        loop {
            select! {
                res = sub.recv() => {
                    match res {
                        Ok(data) => {
                            if data.0 == MotorId::Left {
                                if let Err(e) = self.data_send.send(data.1) {
                                    error!("process_motor_data(), failed to send data: {e}");
                                    break Ok(());
                                }
                            }
                        }
                        Err(e) => {
                            match e {
                                MultiSubRxError::IoClosed => {
                                    error!("process_motor_data(), io closed");
                                    break Err(ClientError::Comms(HostErr::Closed));
                                },
                                MultiSubRxError::Lagged(x) => {
                                    warn!("process_motor_data(), lag: {x}");
                                },
                            }
                        }
                    };
                },
                flag = self.cancel_actor_recv.changed() => {
                    if flag.is_ok() {
                        debug!("process_motor_data(), cancel actor");
                        break Ok(());
                    }
                }
            }
        }
    }
}

pub struct Communication {
    motor_command_queue: Arc<MotorCommandQueue>,
    motor_data_recv: watch::Receiver<MotorProcessData>,
    cancel_actor_send: watch::Sender<bool>,
    motor_command_actor_err_recv: watch::Receiver<Result<(), String>>,
    motor_data_actor_err_recv: watch::Receiver<Result<(), String>>,
    prev_command: Option<MotorCommand>,
}

impl Communication {
    pub fn new(port_name: &str) -> Result<Self, String> {
        let client = Arc::new(Client::new(port_name)?);
        let motor_command_queue = Arc::new(MotorCommandQueue::new());
        let (motor_data_send, motor_data_recv) = watch::channel(MotorProcessData::default());
        let (cancel_actor_send, cancel_actor_recv) = watch::channel(false);
        let (motor_command_actor_err_send, motor_command_actor_err_recv) = watch::channel(Ok(()));
        let (motor_data_actor_err_send, motor_data_actor_err_recv) = watch::channel(Ok(()));

        let mut motor_command_actor = MotorCommandActor {
            client: client.clone(),
            command_queue: motor_command_queue.clone(),
            cancel_actor_recv: cancel_actor_recv.clone(),
            task_err: motor_command_actor_err_send,
        };

        let mut motor_data_actor = MotorDataActor {
            client: client.clone(),
            data_send: motor_data_send,
            cancel_actor_recv: cancel_actor_recv.clone(),
            task_err: motor_data_actor_err_send,
        };

        tokio::spawn(async move { motor_command_actor.run().await });
        tokio::spawn(async move { motor_data_actor.run().await });

        Ok(Self {
            motor_command_queue,
            motor_data_recv,
            cancel_actor_send,
            motor_command_actor_err_recv: motor_command_actor_err_recv,
            motor_data_actor_err_recv: motor_data_actor_err_recv,
            prev_command: None,
        })
    }

    pub fn stop(&self) {
        self.cancel_actor_send
            .send(true)
            .expect("Fail to send stop signal");
    }

    pub fn send_motor_command(&mut self, data: MotorCommand) {
        if self.prev_command.is_some_and(|x| x == data) {
            return;
        }

        self.motor_command_queue.send(data);
        self.prev_command = Some(data);
    }

    pub fn get_motor_process_data(&self) -> MotorProcessData {
        *self.motor_data_recv.borrow()
    }

    pub fn get_motor_command_actor_err(&self) -> Result<(), String> {
        self.motor_command_actor_err_recv.borrow().clone()
    }

    pub fn get_motor_data_actor_err(&self) -> Result<(), String> {
        self.motor_data_actor_err_recv.borrow().clone()
    }
}
