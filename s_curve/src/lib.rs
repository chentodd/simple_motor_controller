#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use std::io::Write;

#[cfg(not(feature = "std"))]
use num_traits::Float;

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
    pos_offset: f32,
    vel_min: f32,
    acc_start: f32,
    acc_end: f32,
    acc_max: f32,
    acc_min: f32,
    jerk_max: f32,
    jerk_min: f32,
    dir: f32,
}

#[derive(Default, Clone)]
pub struct InterpolationDataOutput {
    pub pos: f32,
    pub vel: f32,
    pub acc: f32,
    pub jerk: f32,
}

#[derive(Default, Clone)]
struct InterpolationData {
    pos: f32,
    dist: f32,
    vel: f32,
    acc: f32,
    jerk: f32,

    ta: [f32; 2],
    tb: [f32; 2],
    td: [f32; 2],
    h: f32,
    steps: usize,
    dec_start_period: usize,
    dec_right_away: bool,
    pos_end: f32,
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
            target_data: TargetData {
                dir: 1.0,
                ..Default::default()
            },
            motion_constraint: SCurveConstraint {
                vel_limit,
                acc_limit,
                jerk_limit,
                sampling_time,
            },
        }
    }

    pub fn get_intp_data(&self) -> InterpolationDataOutput {
        let dir = self.target_data.dir;
        InterpolationDataOutput {
            pos: self.intp_data.pos * dir,
            vel: self.intp_data.vel * dir,
            acc: self.intp_data.acc * dir,
            jerk: self.intp_data.jerk * dir,
        }
    }

    pub fn get_intp_status(&self) -> InterpolationStatus {
        self.intp_status
    }

    pub fn set_target(
        &mut self,
        pos_offset: f32,
        displacement: f32,
        vel_start: f32,
        vel_end: f32,
        vel_max_magnitude: f32,
    ) {
        let t = self.motion_constraint.sampling_time;

        // The interpolation is not needed if distance == 0 or v_max == 0. The `dec_right_away` is a special case for
        // aborting interpolation and 0 displacement is allowed in this case
        if (displacement == 0.0 && !self.intp_data.dec_right_away) || vel_max_magnitude == 0.0 {
            return;
        }

        // Simple protection for v_max, the value should be greater than 0
        let vel_max = vel_max_magnitude.abs();
        let vel_max = if vel_max <= 1e-6 || vel_max > self.motion_constraint.vel_limit {
            self.motion_constraint.vel_limit
        } else {
            vel_max
        };

        // Calculate a_max and j_max from v_max using simple equation
        let acc_max = vel_max / t / 100.0;
        let acc_max = if acc_max <= 1e-6 || acc_max > self.motion_constraint.acc_limit {
            self.motion_constraint.acc_limit
        } else {
            acc_max
        };

        let jerk_max = acc_max / t / 10.0;
        let jerk_max = if jerk_max <= 1e-6 || jerk_max > self.motion_constraint.jerk_limit {
            self.motion_constraint.jerk_limit
        } else {
            jerk_max
        };

        // Calculate dir coefficient
        let dir_prev = self.target_data.dir;
        let dir = if displacement >= 0.0 { 1.0 } else { -1.0 };
        self.target_data.dir = dir;

        // Special case, override displacement if `dec_right_away` is true
        let mut displacement = displacement;
        if self.intp_data.dec_right_away {
            displacement = 0.0;
        }

        // According to the equations on book, the s-curve will always treat the segment as positive which means
        // `q_end > q_start`. If `q_end < q_start` we need to flip velocity and position
        //
        // Override `vel_start` if previous intp vel != 0 to make sure current moves after previous segmenet
        // 1. If previous direction is negative, the sign of previous intp velocity is flipped.
        //    Example:
        //      Test input A:
        //        * q_start: 0
        //        * q_end: -10
        //        * vel_start: 0
        //        * vel_end: -2
        //
        //      This settings will be processed as positive segment:
        //        * q_start: 0
        //        * q_end: 10
        //        * vel_start: 0
        //        * vel_end: 2
        //      with a negative direction settings (-1.0). The direction settings will be used to flip the
        //      interpolated value during the output stage.
        //
        //      After test input A is finished, the intp vel is 2, and this value need to be fliped to prevent
        //      velocity jump errors.
        //
        //      a. New input that moves axis in negative direction:
        //         vel_start = -2 (flipped)
        //         target_data.vel = -1 * vel_start = 2 (flipped)
        //         output vel = -1 * intp vel = -1 * 2 (flipped)
        //         => The first output vel is consistent with previous end vel (-2)
        //
        //      b. New input that moves axis in positive direction:
        //         vel_start = -2 (flipped)
        //         target_data.vel = 1 * vel_start = -2 (no flipped)
        //         output vel = 1 * intp vel = 1 * (-2) (no flipped)
        //         => The first output vel is consistent with previous end vel (-2)
        //
        // 2. If previous direction is positive, the sign of previous intp vel is not flipped.
        //    Example:
        //      Test input A:
        //        * q_start: 0
        //        * q_end: 10
        //        * vel_start: 0
        //        * vel_end: 2
        //
        //    Because it is already a positive segment, the ending intp vel is 2
        //
        //    a. New input that moves axis in negative direction
        //       vel start = 2 (no flipped)
        //       target_data.vel = -1 * vel_start = -2 (flipped)
        //       output vel = -1 * intp vel = 2 (flipped)
        //       => The first output vel is consistent with previous end vel (2)
        //
        //    b. New input that moves axis in positive direction again
        //       vel start = 2 (no flipped)
        //       target_data.vel = 1 * vel_start = 2 (no flipped)
        //       output vel = 1 * intp vel = 2 (no flipped)
        //       => The first output vel is consistent with previous end vel (2)
        //
        let mut vel_start = vel_start;
        if self.intp_data.vel != 0.0 {
            if dir_prev < 0.0 {
                vel_start = -self.intp_data.vel;
            } else {
                vel_start = self.intp_data.vel;
            }
        }

        // Override intp pos end if the direction is revered
        // The decision is similar as above comments, here is the example that explain the decision
        //
        // a. Previous: axis moves in negative direction, Current: axis moves in negative direction
        //    * Previous pos_end is negative
        //    * Current direction is negative, flip previous pos_end, it becomes positive
        //    * Doing interpolation in current direction, the output value is flipped again
        //    => This makes the positive is consistent with previous segment (same for pos_offset)
        let mut pos_offset = pos_offset;
        if dir < 0.0 {
            self.intp_data.pos_end = -self.intp_data.pos_end;
            pos_offset = -pos_offset;
        }

        // Use symmetric settings for min value and update target data struct
        self.target_data.pos_offset = pos_offset;
        self.target_data.dist = dir * displacement;
        self.target_data.vel_start = dir * vel_start;
        self.target_data.vel_end = dir * vel_end;
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

        // Special case, update intp dist according to intp status.
        //
        // 1. If previous intp is finished, the intp dist is set to 0, this makes sure intp can generate correct
        //    distance when running new segment. (combined with `pos_end`, we can get expected position value)
        // 2. If previous intp is not finished, the intp dist is not set to 0. Currently, this is caused by stopping
        //    the interplation in the middle by calling `stop`. Because we need to make sure axis is stopping from
        //    current position, the distance is not reset to 0. (If it is reset to 0, then position will jump during
        //    calculation)
        if self.intp_status == InterpolationStatus::Done {
            self.intp_data.dist = 0.0;
        }

        // Update status
        self.intp_status = InterpolationStatus::Busy;
        self.intp_data.dec_start_period = usize::MIN;
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
        let dir = self.target_data.dir;
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

    pub fn stop(&mut self) {
        // 1. Set `dec_right_away` to true:
        //    * Stop generating acc/vel data
        //    * Run deceleration segment right away
        //
        // 2. Use `set_target` to force deceleration
        //    * Special displacement: 0.0
        //      In general case, 0 displacement will be rejected, but it is allowed when running `Stop`. The 0
        //      displacement makes sure deceleration distance is larger(or equal) to remaing distance which forces
        //      intp to run deceleration calculation (in `generate_jerk_dec_segment`)
        //    * Vel start: use current intp vel times direction.
        //      Because all the calculation is based on positive segment, the intp vel needs to be flipped to get real
        //      value)
        //    * End velocity: 0
        //      Make axis stop at the end and also make sure `calculate_dec_distance` is activated
        self.intp_data.dec_right_away = true;
        self.set_target(
            0.0,
            0.0,
            self.intp_data.vel * self.target_data.dir,
            0.0,
            self.motion_constraint.vel_limit,
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
        if self.intp_data.h >= (self.target_data.dist - self.intp_data.dist)
            || self.intp_data.dec_right_away
        {
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
        if self.intp_data.h < (self.target_data.dist - self.intp_data.dist)
            && !self.intp_data.dec_right_away
        {
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
            self.intp_data.dec_right_away = false;
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
        self.intp_data.pos = self.target_data.pos_offset + self.intp_data.pos_end + dist_next;
        self.intp_data.steps += 1;

        // Store intp pos if the state is Done
        if self.intp_status == InterpolationStatus::Done {
            self.intp_data.pos_end = self.target_data.dir * self.intp_data.pos;
        }
    }
}
