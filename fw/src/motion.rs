use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{Subscriber, WaitResult},
};
use heapless::Deque;
use protocol::{ControlMode, MotorCommand, MotorProcessData, PositionCommand};

use crate::{motor::*, rad_s_to_rpm, rpm_to_rad_s};
use s_curve::*;

enum AbortProcessState {
    Idle,
    Ignite,
    Running,
    Finished,
}

pub struct Motion<
    'a,
    M: RawMutex,
    T1: GeneralInstance4Channel,
    T2: GeneralInstance4Channel,
    const QUEUE_SIZE: usize,
> {
    pub motor: BldcMotor24H<'a, T1, T2>,
    pub s_curve_intper: SCurveInterpolator,
    abort_process_state: AbortProcessState,
    cmd_sub: Subscriber<'a, M, MotorCommand, QUEUE_SIZE, 1, 1>,
    cmd_queue: Deque<MotorCommand, QUEUE_SIZE>,
    control_mode: ControlMode,
}

impl<
        'a,
        M: RawMutex,
        T1: GeneralInstance4Channel,
        T2: GeneralInstance4Channel,
        const QUEUE_SIZE: usize,
    > Motion<'a, M, T1, T2, QUEUE_SIZE>
{
    pub fn new(
        s_curve_intper: SCurveInterpolator,
        motor: BldcMotor24H<'a, T1, T2>,
        cmd_sub: Subscriber<'a, M, MotorCommand, QUEUE_SIZE, 1, 1>,
    ) -> Self {
        Self {
            motor,
            s_curve_intper,
            abort_process_state: AbortProcessState::Idle,
            cmd_sub,
            cmd_queue: Deque::new(),
            control_mode: ControlMode::Velocity,
        }
    }

    pub fn read_cmd_from_queue(&mut self) {
        if let Some(left_cmd) = self.cmd_sub.try_next_message() {
            match left_cmd {
                WaitResult::Message(cmd) => {
                    if cmd == MotorCommand::Abort {
                        self.cmd_queue.clear();
                    }
                    self.set_command(cmd);
                }
                _ => (),
            }
        }
    }

    pub fn is_queue_full(&self) -> bool {
        self.cmd_queue.is_full()
    }

    pub fn get_motor_process_data(&self) -> MotorProcessData {
        let s_curve_intp_data = self.s_curve_intper.get_intp_data();
        MotorProcessData {
            control_mode_display: self.control_mode,
            actual_pos: self.motor.encoder.get_act_position_in_rad(),
            actual_vel: self.motor.encoder.get_act_velocity_in_rpm(),
            intp_pos: s_curve_intp_data.pos,
            intp_vel: s_curve_intp_data.vel,
            intp_acc: s_curve_intp_data.acc,
            intp_jerk: s_curve_intp_data.jerk,
        }
    }

    pub fn run(&mut self) {
        // Process abort process if controller gets abort request
        self.process_abort();

        // Interpolate position command if current operation if IntpPos and update
        // target velocity in pid velocity control loop
        if self.control_mode == ControlMode::Position
            && self.s_curve_intper.get_intp_status() != InterpolationStatus::Done
        {
            self.s_curve_intper.interpolate();

            let intp_vel = rad_s_to_rpm(self.s_curve_intper.get_intp_data().vel);
            self.motor.set_target_velocity(intp_vel);

            #[cfg(feature = "debug-motion")]
            debug!("run, intp pos, {}", intp_vel);
        }

        // The pid velocity control loop will always be run since we need to drive
        // the motor with velocity command.
        // If current operation == `IntPos`, the target velocity will be set by position interpolation
        // If current operation != `IntPos`, the target velocity will be set by `set_command` function
        // Note: only `IntpVel` is handled, and the other operation modes are currently listed as `todo!()`
        self.motor.run_pid_velocity_control();
    }

    fn set_command(&mut self, mut cmd: MotorCommand) {
        if self.cmd_queue.is_full() {
            return;
        }

        // cmd_queue is used as a backup when command can't be set
        let _ = self.cmd_queue.push_back(cmd);
        let ready_to_set = match self.control_mode {
            ControlMode::Velocity | ControlMode::Stop => true,
            ControlMode::Position => self.ready(),
        };

        if ready_to_set {
            // Ok to set command, read cmd from queue
            cmd = self.cmd_queue.pop_front().unwrap();
            match cmd {
                MotorCommand::Abort => {
                    self.abort_process_state = AbortProcessState::Ignite;
                    match self.control_mode {
                        ControlMode::Position => self.s_curve_intper.stop(),
                        ControlMode::Velocity => self.motor.set_target_velocity(0.0),
                        _ => (),
                    }
                }
                MotorCommand::PositionCommand(x) => {
                    self.control_mode = ControlMode::Position;
                    self.set_pos_command(x);
                }
                MotorCommand::VelocityCommand(x) => {
                    self.control_mode = ControlMode::Velocity;
                    self.motor.set_target_velocity(x);
                }
            }
        }
    }

    fn process_abort(&mut self) {
        match self.abort_process_state {
            AbortProcessState::Ignite => self.abort_process_state = AbortProcessState::Running,
            AbortProcessState::Running => {
                if self.ready() {
                    self.abort_process_state = AbortProcessState::Finished;
                }
            }
            AbortProcessState::Finished => {
                // Stop control mode will be set if motor is standstill
                self.abort_process_state = AbortProcessState::Idle;
                self.control_mode = ControlMode::Stop;
            }
            _ => (),
        }
    }

    fn set_pos_command(&mut self, cmd: PositionCommand) {
        let vel_max = rpm_to_rad_s(cmd.vel_max);
        let vel_start = rpm_to_rad_s(self.motor.encoder.get_act_velocity_in_rpm());
        let vel_end = rpm_to_rad_s(cmd.vel_end);

        let pos_offset =
            self.motor.encoder.get_act_position_in_rad() - self.s_curve_intper.get_intp_data().pos;
        self.s_curve_intper
            .set_target(pos_offset, cmd.displacement, vel_start, vel_end, vel_max);

        #[cfg(feature = "debug-motion")]
        debug!(
            "set_pos_command, {}, {}, {}, {}",
            command.target_dist,
            self.motor.encoder.get_act_velocity_in_rpm(),
            vel_end,
            vel
        );
    }

    fn ready(&mut self) -> bool {
        let is_ready = match self.control_mode {
            ControlMode::Position => {
                #[cfg(feature = "debug-motion")]
                debug!(
                    "ready, pos, {}",
                    self.s_curve_intper.get_intp_status() as u8
                );

                self.s_curve_intper.get_intp_status() == InterpolationStatus::Done
            }
            ControlMode::Velocity => {
                #[cfg(feature = "debug-motion")]
                debug!("ready, vel, {}", self.motor.get_error());

                self.motor.pid.get_error().abs() <= 60.0
            }
            ControlMode::Stop => true,
        };

        is_ready
    }
}
