use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_sync::{
    blocking_mutex::raw::RawMutex,
    pubsub::{Subscriber, WaitResult},
};

#[cfg(feature = "debug-motion")]
use defmt::{debug, Debug2Format};

use heapless::Deque;
use protocol::{ControlMode, MotorCommand, MotorProcessData, PositionCommand};

use crate::{motor::*, rad_s_to_rpm, rpm_to_rad_s};
use s_curve::*;

#[derive(PartialEq)]
enum HaltProcessState {
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
    const CHANNEL_SIZE: usize,
    const MOTION_QUEUE_SIZE: usize,
> {
    pub motor: BldcMotor24H<'a, T1, T2>,
    pub s_curve_intper: SCurveInterpolator,
    halt_process_state: HaltProcessState,
    cmd_sub: Subscriber<'a, M, MotorCommand, CHANNEL_SIZE, 1, 2>,
    cmd_queue: Deque<MotorCommand, MOTION_QUEUE_SIZE>,
    control_mode: ControlMode,
}

impl<
        'a,
        M: RawMutex,
        T1: GeneralInstance4Channel,
        T2: GeneralInstance4Channel,
        const CHANNEL_SIZE: usize,
        const MOTION_QUEUE_SIZE: usize,
    > Motion<'a, M, T1, T2, CHANNEL_SIZE, MOTION_QUEUE_SIZE>
{
    pub fn new(
        s_curve_intper: SCurveInterpolator,
        motor: BldcMotor24H<'a, T1, T2>,
        cmd_sub: Subscriber<'a, M, MotorCommand, CHANNEL_SIZE, 1, 2>,
    ) -> Self {
        Self {
            motor,
            s_curve_intper,
            halt_process_state: HaltProcessState::Idle,
            cmd_sub,
            cmd_queue: Deque::new(),
            control_mode: ControlMode::Velocity,
        }
    }

    pub fn read_cmd_from_queue(&mut self) {
        if self.cmd_queue.is_full() {
            return;
        }

        if let Some(cmd) = self.cmd_sub.try_next_message() {
            match cmd {
                WaitResult::Message(cmd) => {
                    if cmd == MotorCommand::Halt {
                        self.cmd_queue.clear();
                    }

                    // cmd_queue is used as a cache to hold commands from host
                    let _ = self.cmd_queue.push_back(cmd);
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
        // Process that reads command from queue and set command if it is ok
        if let Some(&cmd) = self.cmd_queue.front() {
            let mut ready_to_set = match cmd {
                MotorCommand::VelocityCommand(_) | MotorCommand::Halt => true,
                MotorCommand::PositionCommand(_) => self.ready(),
            };

            if self.halt_process_state != HaltProcessState::Idle {
                // Halt process is running, do not set command
                ready_to_set = false;
            }

            if ready_to_set {
                match cmd {
                    MotorCommand::Halt => {
                        self.halt_process_state = HaltProcessState::Ignite;
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

                // Command is set, pop it from queue
                self.cmd_queue.pop_front();
            }
        }

        // Process halt if controller gets halt request
        self.process_halt();

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

    fn process_halt(&mut self) {
        match self.halt_process_state {
            HaltProcessState::Ignite => self.halt_process_state = HaltProcessState::Running,
            HaltProcessState::Running => {
                if self.ready() {
                    self.halt_process_state = HaltProcessState::Finished;
                }
            }
            HaltProcessState::Finished => {
                // Standstill control mode will be set when halt process is finished
                self.halt_process_state = HaltProcessState::Idle;
                self.control_mode = ControlMode::StandStill;
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
            cmd.displacement,
            self.motor.encoder.get_act_velocity_in_rpm(),
            vel_end,
            vel_max
        );
    }

    fn ready(&self) -> bool {
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
            ControlMode::StandStill => true,
        };

        is_ready
    }
}
