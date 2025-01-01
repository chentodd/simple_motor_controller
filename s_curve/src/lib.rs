#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::io::Write;

#[cfg(not(feature = "std"))]
use num_traits::Float;

#[derive(Default, Clone)]
pub struct InterpolationData {
    pub pos: f32,
    pub dist: f32,
    pub vel: f32,
    pub acc: f32,
    pub jerk: f32,

    ta: [f32; 2],
    tb: [f32; 2],
    td: [f32; 2],
    h: f32,
    steps: usize,
    dec_start_period: usize,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum InterpolationStatus {
    #[default]
    Done,
    Busy,
    Error,
}

#[derive(Default, Clone)]
pub struct TargetData {
    pub dist: f32,
    pub vel_start: f32,
    pub vel_end: f32,
    pub vel_max: f32,
    vel_min: f32,
    acc_start: f32,
    acc_end: f32,
    acc_max: f32,
    acc_min: f32,
    jerk_max: f32,
    jerk_min: f32,
    dir: f32
}

#[derive(Default, Clone)]
struct SCurveConstraint {
    vel_limit: f32,
    acc_limit: f32,
    jerk_limit: f32,
    sampling_time: f32,
}

#[derive(Default, Clone)]
pub struct SCurveInterpolator {
    intp_data: InterpolationData,
    intp_status: InterpolationStatus,
    target_data: TargetData,
    motion_constraint: SCurveConstraint,
}

impl SCurveInterpolator {
    pub fn new(vel_limit: f32, acc_limit: f32, jerk_limit: f32, sampling_time: f32) -> Self {
        Self {
            intp_data: InterpolationData::default(),
            intp_status: InterpolationStatus::default(),
            target_data: TargetData::default(),
            motion_constraint: SCurveConstraint {
                vel_limit,
                acc_limit,
                jerk_limit,
                sampling_time,
            },
        }
    }

    pub fn get_intp_data(&self) -> &InterpolationData {
        &self.intp_data
    }

    pub fn get_intp_status(&self) -> InterpolationStatus {
        self.intp_status
    }

    pub fn set_target(&mut self, dist: f32, vel_start: f32, vel_end: f32, vel_max: f32) {
        let t = self.motion_constraint.sampling_time;

        // Simple protection for v_max, the value should be greater than 0
        let vel_max = vel_max.abs();
        let vel_max = if vel_max <= 1e-6 || vel_max > self.motion_constraint.vel_limit {
            self.motion_constraint.vel_limit
        } else {
            vel_max
        };

        // Calculate a_max and j_max from v_max using simple equation
        let acc_max = vel_max / t;
        let acc_max = if acc_max <= 1e-6 || acc_max > self.motion_constraint.acc_limit {
            self.motion_constraint.acc_limit
        } else {
            acc_max
        };

        let jerk_max = acc_max / t;
        let jerk_max = if jerk_max <= 1e-6 || jerk_max > self.motion_constraint.jerk_limit {
            self.motion_constraint.jerk_limit
        } else {
            jerk_max
        };

        // Calculate dir coefficient
        let dir = if dist >= 0.0 { 1.0 } else { -1.0 };
        self.target_data.dir = dir;

        // Use symmetric settings for min value and update target data struct
        // TODO, verify the correctness of sign changing method
        self.target_data.dist = dir * dist;
        self.target_data.vel_start = vel_start;
        self.target_data.vel_end = vel_end;
        self.target_data.vel_max = vel_max * (dir + 1.0) / 2.0 - vel_max * (dir - 1.0) / 2.0;
        self.target_data.vel_min = -self.target_data.vel_max;
        self.target_data.acc_start = 0.0;
        self.target_data.acc_end = 0.0;
        self.target_data.acc_max = acc_max * (dir + 1.0) / 2.0 - acc_max * (dir - 1.0) / 2.0;
        self.target_data.acc_min = -self.target_data.acc_max;
        self.target_data.jerk_max = jerk_max * (dir + 1.0) / 2.0 - jerk_max * (dir - 1.0) / 2.0;
        self.target_data.jerk_min = -self.target_data.jerk_max;

        // Update current interpolation data based on target start condition
        self.intp_data.vel = self.target_data.vel_start;
        self.intp_data.acc = self.target_data.acc_start;

        // Update status
        self.intp_status = InterpolationStatus::Busy;
        self.intp_data.dec_start_period = usize::MIN;
    }

    pub fn get_dir(&self) -> f32 {
        self.target_data.dir
    }

    pub fn interpolate(&mut self) {
        if self.intp_status == InterpolationStatus::Done {
            return;
        }

        self.calculate_dec_distance();
        self.generate_jerk_acc_vel_segment();
        self.generate_jerk_dec_segment();
        self.integrate();
    }

    #[cfg(feature = "std")]
    pub fn save_intp_data(&self, file: &mut std::fs::File) {
        let dir = self.get_dir();
        let _ = write!(
            file,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            self.intp_data.pos * dir,
            self.intp_data.vel * dir,
            self.intp_data.acc * dir,
            self.intp_data.jerk * dir,
            self.intp_data.ta[0],
            self.intp_data.tb[0],
            self.intp_data.td[0],
            self.intp_data.h * dir
        );
    }

    fn calculate_dec_distance(&mut self) {
        if self.intp_data.vel < self.target_data.vel_end {
            // In deceleration segment, we expect the intp vel is greater than or
            // equal to target end velocity
            return;
        }

        // Calculate the time in deceleration segment: T_a, T_b, T_d
        let vel_end = self.target_data.vel_end;
        let acc_end = self.target_data.acc_end;
        let acc_min = self.target_data.acc_min;
        let jerk_max = self.target_data.jerk_max;
        let jerk_min = self.target_data.jerk_min;
        let vel_cur = self.intp_data.vel;
        let acc_cur = self.intp_data.acc;

        let mut ta = (acc_min - acc_cur) / jerk_min;
        let mut tb = (acc_end - acc_min) / jerk_max;
        let mut td = ((vel_end - vel_cur) / acc_min)
            + (ta * (acc_min - acc_cur) / (2.0 * acc_min))
            + (tb * (acc_min - acc_end) / (2.0 * acc_min));

        if td < (ta + tb) {
            let acc_cur_square = acc_cur * acc_cur;
            let acc_end_squre = acc_end * acc_end;
            let term1 = acc_cur_square * jerk_max
                - jerk_min * (acc_end_squre + 2.0 * jerk_max * (vel_cur - vel_end));
            let term2 = jerk_max - jerk_min;

            ta = -acc_cur / jerk_min + (term2 * term1).sqrt() / (-term2 * jerk_min);
            tb = acc_end / jerk_max + (term2 * term1).sqrt() / (term2 * jerk_max);
            td = ta + tb;
        }

        // Calculate deceleration distance
        let td_square = td * td;
        let ta_square = ta * ta;
        let tb_cubic = tb * tb * tb;
        let hk = 0.5 * acc_cur * td_square
            + (1.0 / 6.0)
                * (jerk_min * ta * (3.0 * td_square - 3.0 * td * ta + ta_square)
                    + jerk_max * tb_cubic)
            + td * vel_cur;

        // Basic protection of numerical error to prevent negative time
        if ta < 0.0 {
            ta = 0.0;
        }

        if tb < 0.0 {
            tb = 0.0;
        }

        if td < 0.0 {
            td = 0.0;
        }

        self.intp_data.ta[0] = ta;
        self.intp_data.tb[0] = tb;
        self.intp_data.td[0] = td;
        self.intp_data.h = hk;
    }

    fn generate_jerk_acc_vel_segment(&mut self) {
        if self.intp_data.h >= (self.target_data.dist - self.intp_data.dist) {
            // Check decelerate distance, do acceleration only when decelerate
            // distance is less than remaining distance
            return;
        }

        let t = self.motion_constraint.sampling_time;

        let vel_max = self.target_data.vel_max;
        let acc_max = self.target_data.acc_max;
        let jerk_min = self.target_data.jerk_min;
        let jerk_max = self.target_data.jerk_max;

        let vel_cur = self.intp_data.vel;
        let acc_cur = self.intp_data.acc;

        // Check if we can continue using jMax to accelerate
        // TODO, pay attension on the effect on dir coefficient
        let end_vel_cur = vel_cur - (acc_cur * acc_cur / (2.0 * jerk_min));
        if end_vel_cur < vel_max && acc_cur < acc_max {
            let jerk_temp = (acc_max - acc_cur) / t;
            self.intp_data.jerk = jerk_max.min(jerk_temp);
        } else if end_vel_cur < vel_max && acc_cur >= acc_max {
            self.intp_data.acc = acc_max;
            self.intp_data.jerk = 0.0;
        } else if end_vel_cur >= vel_max && acc_cur > 0.0 {
            let jerk_temp = (0.0 - acc_cur) / t;
            self.intp_data.jerk = jerk_min.max(jerk_temp);
        } else if end_vel_cur >= vel_max && acc_cur <= 0.0 {
            self.intp_data.acc = 0.0;
            self.intp_data.jerk = 0.0;
        }
    }

    fn generate_jerk_dec_segment(&mut self) {
        if self.intp_data.h < (self.target_data.dist - self.intp_data.dist) {
            return;
        }

        // Need to decelerate, record the period when decelerating phase takes control
        if self.intp_data.dec_start_period == usize::MIN {
            self.intp_data.dec_start_period = self.intp_data.steps;
            self.intp_data.ta[1] = self.intp_data.ta[0];
            self.intp_data.tb[1] = self.intp_data.tb[0];
            self.intp_data.td[1] = self.intp_data.td[0];
        }

        let t = self.motion_constraint.sampling_time;

        let first_stage_start_period = 0_usize;
        let first_stage_end_period = (self.intp_data.ta[1] / t) as usize;

        let second_stage_start_period = first_stage_end_period;
        let second_stage_end_period = ((self.intp_data.td[1] - self.intp_data.tb[1]) / t) as usize;

        let third_stage_start_period = second_stage_end_period;
        let third_stage_end_period = (self.intp_data.td[1] / t) as usize;

        let elapsed_period = self.intp_data.steps - self.intp_data.dec_start_period;
        if first_stage_start_period <= elapsed_period && elapsed_period <= first_stage_end_period {
            let jerk_temp = (self.target_data.acc_min - self.intp_data.acc) / t;
            self.intp_data.jerk = self.target_data.jerk_min.max(jerk_temp);
        } else if second_stage_start_period <= elapsed_period
            && elapsed_period <= second_stage_end_period
        {
            self.intp_data.jerk = 0.0;
            self.intp_data.acc = self.target_data.acc_min;
        } else if third_stage_start_period <= elapsed_period
            && elapsed_period <= third_stage_end_period
        {
            let jerk_temp = (self.target_data.acc_end - self.intp_data.acc) / t;
            self.intp_data.jerk = self.target_data.jerk_max.min(jerk_temp);
        } else {
            self.intp_data.vel = self.target_data.vel_end;
            self.intp_data.acc = 0.0;
            self.intp_data.jerk = 0.0;

            // set finished status
            self.intp_status = InterpolationStatus::Done;
            self.intp_data.dec_start_period = usize::MIN;
        }
    }

    fn integrate(&mut self) {
        let jerk = self.intp_data.jerk;
        let acc = self.intp_data.acc;
        let vel = self.intp_data.vel;
        let dist = self.intp_data.dist;
        let t = self.motion_constraint.sampling_time;

        let acc_next = acc + t * jerk;
        let vel_next = vel + (t / 2.0) * (acc + acc_next);
        let dist_next = dist + (t / 2.0) * (vel + vel_next);

        self.intp_data.acc = acc_next;
        self.intp_data.vel = vel_next;
        self.intp_data.dist = dist_next;
        self.intp_data.pos = dist_next;
        self.intp_data.steps += 1;
    }
}
