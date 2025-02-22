use egui_plot::PlotPoints;
use std::collections::VecDeque;
use std::fmt::Display;

use crate::proto::command_::CommandTx;

#[derive(Default)]
pub struct ProfileData {
    intp_pos: f32,
    intp_vel: f32,
    intp_acc: f32,
    intp_jerk: f32,
    act_pos: f32,
    act_vel: f32,
}

impl ProfileData {
    pub fn new(
        intp_pos: f32,
        intp_vel: f32,
        intp_acc: f32,
        intp_jerk: f32,
        act_pos: f32,
        act_vel: f32,
    ) -> Self {
        Self {
            intp_pos,
            intp_vel,
            intp_acc,
            intp_jerk,
            act_pos,
            act_vel,
        }
    }
}

#[derive(Clone, Copy)]
pub enum ProfileDataType {
    IntpPos,
    IntpVel,
    IntpAcc,
    IntpJerk,
    ActPos,
    ActVel,
}

impl Display for ProfileDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileDataType::IntpPos => write!(f, "intp_pos"),
            ProfileDataType::IntpVel => write!(f, "intp_vel"),
            ProfileDataType::IntpAcc => write!(f, "intp_acc"),
            ProfileDataType::IntpJerk => write!(f, "intp_jerk"),
            ProfileDataType::ActPos => write!(f, "act_pos"),
            ProfileDataType::ActVel => write!(f, "act_vel"),
        }
    }
}

pub struct MeasurementWindow {
    window_values: VecDeque<ProfileData>,
    window_size: usize,
}

impl MeasurementWindow {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_values: VecDeque::new(),
            window_size,
        }
    }

    pub fn reset(&mut self) {
        self.window_values.clear();
    }

    pub fn update_measurement_window(&mut self, data: CommandTx) {
        if self.window_values.len() == self.window_size {
            self.window_values.pop_front();
        }
        self.window_values.push_back(ProfileData::new(
            data.left_motor.intp_pos,
            data.left_motor.intp_vel,
            data.left_motor.intp_acc,
            data.left_motor.intp_jerk,
            data.left_motor.actual_pos,
            data.left_motor.actual_vel,
        ));
    }

    pub fn get_data(&self, get_data_type: ProfileDataType) -> PlotPoints {
        let iter = self.window_values.iter().enumerate();
        match get_data_type {
            ProfileDataType::IntpPos => iter.map(|(x, y)| [x as f64, y.intp_pos as f64]).collect(),
            ProfileDataType::IntpVel => iter.map(|(x, y)| [x as f64, y.intp_vel as f64]).collect(),
            ProfileDataType::IntpAcc => iter.map(|(x, y)| [x as f64, y.intp_acc as f64]).collect(),
            ProfileDataType::IntpJerk => {
                iter.map(|(x, y)| [x as f64, y.intp_jerk as f64]).collect()
            }
            ProfileDataType::ActPos => iter.map(|(x, y)| [x as f64, y.act_pos as f64]).collect(),
            ProfileDataType::ActVel => iter.map(|(x, y)| [x as f64, y.act_vel as f64]).collect(),
        }
    }
}
