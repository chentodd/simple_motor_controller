pub struct Pid {
    kp: f32,
    ki: f32,
    kd: f32,
    target_velocity_rpm: f32,
    curr_error: f32,
    prev_error: f32,
    accumulated_error: f32,
    output_limit: f32,
}

impl Pid {
    pub fn new(kp: f32, ki: f32, kd: f32, output_limit: f32) -> Self {
        Self {
            kp: kp,
            ki: ki,
            kd: kd,
            target_velocity_rpm: 0.0,
            curr_error: 0.0,
            prev_error: 0.0,
            accumulated_error: 0.0,
            output_limit: output_limit,
        }
    }

    pub(crate) fn set_target_velocity(&mut self, target_velocity_rpm: f32) {
        self.target_velocity_rpm = target_velocity_rpm;
    }

    pub(crate) fn run(&mut self, curr_velocity_prm: f32, period_s: f32) -> f32 {
        self.prev_error = self.curr_error;
        self.curr_error = self.target_velocity_rpm - curr_velocity_prm;
        self.accumulated_error += self.curr_error * period_s;

        let mut control_effort = self.kp * self.curr_error
            + self.ki * self.accumulated_error
            + self.kd * (self.curr_error - self.prev_error) / period_s;

        if control_effort > self.output_limit {
            control_effort = self.output_limit
        } else if control_effort < -self.output_limit {
            control_effort = -self.output_limit;
        }

        control_effort
    }
}
