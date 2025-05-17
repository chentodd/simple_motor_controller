use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::Arc;

use log::{debug, error, warn};
use postcard_rpc::host_client::{HostErr, MultiSubRxError};
use tokio::select;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, watch};

use host::client::{Client, ClientError};
use protocol::*;

struct MotorCommandActor {
    client: Arc<Client>,
    halt_command_recv: mpsc::Receiver<()>,
    command_queue_send_internal: UnboundedSender<MotorCommand>,
    command_queue_recv: mpsc::UnboundedReceiver<MotorCommand>,
    cancel_actor_recv: watch::Receiver<bool>,
    task_err_send: watch::Sender<Result<(), String>>,
}

impl MotorCommandActor {
    async fn run(&mut self) {
        let result = self.run_internal().await;
        if let Err(e) = result {
            self.task_err_send
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

        let mut internal_command_cache = VecDeque::<MotorCommand>::new();

        let _id = self
            .client
            .ping(0)
            .await
            .map_err(|_x| ClientError::Comms(HostErr::Closed))?;

        loop {
            select! {
                biased;

                flag = self.cancel_actor_recv.changed() => {
                    if flag.is_ok() {
                        debug!("process_motor_command(), cancel actor");
                        break Ok(());
                    }
                },
                Some(()) = self.halt_command_recv.recv() => {
                    debug!("process_motor_command(), halt");
                    while let Ok(_) = self.command_queue_recv.try_recv() {
                        // Consume all the commands in the queue
                    }

                    // Ignore the error because the receiver is held by the actor
                    let _ = self.command_queue_send_internal.send(MotorCommand::Halt);
                },
                Some(motor_command) = self.command_queue_recv.recv() => {
                    debug!("receive, command: {motor_command:?}");
                    if motor_command == MotorCommand::Halt {
                        internal_command_cache.clear();
                    }
                    internal_command_cache.push_back(motor_command);
                },
                result = async {
                    if let Some(command) = internal_command_cache.front() {
                        self
                            .client
                            .set_motor_cmd(MotorId::Left, command.clone())
                            .await?;
                    }

                    Ok(())
                } => {
                    match result {
                            Ok(_) => {
                                // If the command is sent successfully, pop it from the queue
                                internal_command_cache.pop_front();
                            }
                            Err(e) => match e {
                                ClientError::Comms(e) => {
                                    error!("process_motor_command(), unexpected error: {e:?}");
                                    break Err(ClientError::Comms(e));
                                },
                                _ => (),
                            },
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
    task_err_send: watch::Sender<Result<(), String>>,
}

impl MotorDataActor {
    async fn run(&mut self) {
        let result = self.run_internal().await;
        if let Err(e) = result {
            self.task_err_send
                .send(Err(format!("{e:?}")))
                .expect("Failed to send result in motor data actor");

            error!("MotorDataActor error: {e:?}");
        }
    }

    async fn run_internal(&mut self) -> Result<(), ClientError<Infallible>> {
        let mut motor_data_sub = self
            .client
            .client
            .subscribe_multi::<protocol::MotorProcessDataTopic>(8)
            .await
            .map_err(|_x| ClientError::Comms(HostErr::Closed))?;

        // let mut mpu6050_data_sub = self
        //     .client
        //     .client
        //     .subscribe_multi::<protocol::Mpu6050MotionDataTopic>(8)
        //     .await
        //     .map_err(|_x| ClientError::Comms(HostErr::Closed))?;

        // Check `ping` to make sure the device is connected
        let _id = self.client.ping(0).await?;

        loop {
            select! {
                biased;

                flag = self.cancel_actor_recv.changed() => {
                    if flag.is_ok() {
                        debug!("process_motor_data(), cancel actor");
                        break Ok(());
                    }
                },
                res = motor_data_sub.recv() => {
                    match res {
                        Ok(data) => {
                            if data.0 == MotorId::Left {
                                if let Err(e) = self.data_send.send(data.1) {
                                    error!("process_motor_data(), failed to send data: {e}");
                                    // I borrow the error type from HostError (it might be a bad idea, and this can be
                                    // impproved). When this error is triggered, it means when receiver is dropped
                                    // unexpected, and it should not happen, but I add this error and log message to
                                    // debug it incase it appears.
                                    break Err(ClientError::Comms(HostErr::Closed));
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
                // res = mpu6050_data_sub.recv() => {
                //     match res {
                //         Ok(data) => {
                //             debug!("{:?}", data);
                //         },
                //         _ => (),
                //     }
                // }
            }
        }
    }
}

impl Drop for Communication {
    fn drop(&mut self) {
        debug!("Communication actor is dropped");
        let _ = self.stop();
    }
}

pub struct Communication {
    halt_command_send: mpsc::Sender<()>,
    command_queue_send: mpsc::UnboundedSender<MotorCommand>,
    data_recv: watch::Receiver<MotorProcessData>,
    cancel_actor_send: watch::Sender<bool>,
    command_actor_err_recv: watch::Receiver<Result<(), String>>,
    data_actor_err_recv: watch::Receiver<Result<(), String>>,
    prev_command: Option<MotorCommand>,
}

impl Communication {
    pub fn new(port_name: &str) -> Result<Self, String> {
        let client = Arc::new(Client::new(port_name)?);
        let (halt_command_send, halt_command_recv) = mpsc::channel::<()>(1);
        let (command_queue_send, command_queue_recv) = mpsc::unbounded_channel::<MotorCommand>();
        let (data_send, data_recv) = watch::channel(MotorProcessData::default());
        let (cancel_actor_send, cancel_actor_recv) = watch::channel(false);
        let (command_actor_err_send, command_actor_err_recv) = watch::channel(Ok(()));
        let (data_actor_err_send, data_actor_err_recv) = watch::channel(Ok(()));

        let mut motor_command_actor = MotorCommandActor {
            client: client.clone(),
            halt_command_recv,
            command_queue_send_internal: command_queue_send.clone(),
            command_queue_recv,
            cancel_actor_recv: cancel_actor_recv.clone(),
            task_err_send: command_actor_err_send,
        };

        let mut motor_data_actor = MotorDataActor {
            client: client.clone(),
            data_send,
            cancel_actor_recv: cancel_actor_recv.clone(),
            task_err_send: data_actor_err_send,
        };

        tokio::spawn(async move { motor_command_actor.run().await });
        tokio::spawn(async move { motor_data_actor.run().await });

        Ok(Self {
            halt_command_send,
            command_queue_send,
            data_recv,
            cancel_actor_send,
            command_actor_err_recv,
            data_actor_err_recv,
            prev_command: None,
        })
    }

    pub fn stop(&self) -> Result<(), String> {
        self.cancel_actor_send
            .send(true)
            .map_err(|x| format!("Failed to send cancel signal, msg: {x}"))
    }

    pub fn send_motor_command(&mut self, data: MotorCommand) {
        match data {
            MotorCommand::VelocityCommand(_) | MotorCommand::Halt => {
                if self.prev_command.is_some_and(|x| x == data) {
                    return;
                }
            }
            _ => (),
        }

        // The error arise when the receiver is dropped which means the actor has
        // communcation error, and we need restart the actor again. When this
        // happens, I think we don't need to handle the error as we are going to
        // restart the actor anyway.
        let _ = self.command_queue_send.send(data.clone());

        if data == MotorCommand::Halt {
            // Same as above, we don't need to handle the error
            let _ = self.halt_command_send.try_send(());
        }
        self.prev_command = Some(data);
    }

    pub fn get_motor_process_data(&self) -> MotorProcessData {
        *self.data_recv.borrow()
    }

    pub fn get_motor_command_actor_err(&self) -> Result<(), String> {
        self.command_actor_err_recv.borrow().clone()
    }

    pub fn get_motor_data_actor_err(&self) -> Result<(), String> {
        self.data_actor_err_recv.borrow().clone()
    }
}
