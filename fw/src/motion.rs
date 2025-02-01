#[cfg(feature = "debug-motion")]
use defmt::debug;

use embassy_stm32::timer::GeneralInstance4Channel;

use crate::proto::motor_::{MotorRx, Operation};
use crate::{motor::*, rad_s_to_rpm, rpm_to_rad_s};

use s_curve::*;

pub struct Motion<'a, T1: GeneralInstance4Channel, T2: GeneralInstance4Channel> {
    pub motor: BldcMotor24H<'a, T1, T2>,
    s_curve_intper: SCurveInterpolator,
    operation: Operation,
}

impl<'a, T1: GeneralInstance4Channel, T2: GeneralInstance4Channel> Motion<'a, T1, T2> {
    pub fn new(s_curve_intper: SCurveInterpolator, motor: BldcMotor24H<'a, T1, T2>) -> Self {
        Self {
            motor,
            s_curve_intper,
            operation: Operation::default(),
        }
    }

    pub fn ready(&self) -> bool {
        match self.operation {
            Operation::IntpPos => {
                #[cfg(feature = "debug-motion")]
                debug!(
                    "ready, pos, {}",
                    self.s_curve_intper.get_intp_status() as u8
                );

                return self.s_curve_intper.get_intp_status() == InterpolationStatus::Done;
            }
            Operation::IntpVel => {
                #[cfg(feature = "debug-motion")]
                debug!("ready, vel, {}", self.motor.get_error());

                return self.motor.pid.get_error().abs() <= 60.0;
            }
            _ => (),
        }

        true
    }

    pub fn get_operation(&self) -> Operation {
        self.operation
    }

    pub fn set_command(&mut self, command: MotorRx) {
        // Record operation
        self.operation = command.operation;

        match command.operation {
            Operation::IntpPos => {
                self.set_pos_command(&command);
            }
            Operation::IntpVel => {
                self.motor.set_target_velocity(command.target_vel);

                #[cfg(feature = "debug-motion")]
                debug!("set_command, intp vel, {}", command.target_vel);
            }
            Operation::PidVel => todo!(),
            Operation::PidTune => todo!(),
            _ => (),
        }
    }

    pub fn run(&mut self) {
        // Interpolate position command if current operation if IntpPos and update
        // target velocity in pid velocity control loop
        if self.operation == Operation::IntpPos
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

    fn set_pos_command(&mut self, command: &MotorRx) {
        let vel = rpm_to_rad_s(command.target_vel);
        let vel_start = rpm_to_rad_s(self.motor.encoder.get_act_velocity_in_rpm());
        let vel_end = rpm_to_rad_s(command.target_vel_end);

        self.s_curve_intper
            .set_target(command.target_dist, vel_start, vel_end, vel);

        #[cfg(feature = "debug-motion")]
        debug!(
            "set_pos_command, {}, {}, {}, {}",
            command.target_dist,
            self.motor.get_current_velocity(),
            vel_end,
            vel
        );
    }
}
