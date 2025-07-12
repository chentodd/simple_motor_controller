use core::f32;

pub struct Pid {
    kp: f32,
    ki: f32,
    kd: f32,
    // set point (target velocity in RPM)
    set_point: f32,
    // previous process variable (actual velocity in RPM)
    prev_process_variable: f32,
    error_curr: f32,
    error_prev: f32,
    error_sum: f32,
    output_limit: f32,
    // Auto-tuning state. None: not in tuning mode, Some: in tuning mode
    auto_tune: Option<TuningState>,
}

#[derive(Copy, Clone)]
pub struct TuningState {
    // Configuration for the tuning process
    output_high: f32,
    output_low: f32,

    // Internal state for detecting oscillations
    // pv: process variable
    time_last_crossing: f32,
    pv_last_peak: f32,

    // Storage for measured oscillation characteristics
    peak_amplitudes: [f32; 10],
    peak_periods: [f32; 10],
    peak_count: usize,
}

impl Pid {
    pub fn new(kp: f32, ki: f32, kd: f32, output_limit: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            set_point: 0.0,
            prev_process_variable: 0.0,
            error_curr: 0.0,
            error_prev: 0.0,
            error_sum: 0.0,
            output_limit,
            auto_tune: None,
        }
    }

    pub fn is_autotune_running(&self) -> bool {
        self.auto_tune.is_some()
    }

    pub fn start_autotune(&mut self, output_high: f32, output_low: f32) {
        if self.is_autotune_running() {
            return;
        }

        // If any of the parameters are zero, then we ignore the auto tune request
        if output_high == 0.0 || output_low == 0.0 {
            return;
        }

        self.auto_tune = Some(TuningState {
            output_high,
            output_low,
            time_last_crossing: 0.0,
            pv_last_peak: 0.0,
            peak_amplitudes: [0.0; 10],
            peak_periods: [0.0; 10],
            peak_count: 0,
        });
    }

    pub fn cancel_autotune(&mut self) {
        self.auto_tune = None;
        self.reset();
    }

    pub(crate) fn set_target_velocity(&mut self, target_velocity_rpm: f32) {
        self.set_point = target_velocity_rpm;
    }

    pub(crate) fn get_error(&self) -> f32 {
        self.error_curr
    }

    pub(crate) fn run(&mut self, act_velocity_rpm: f32, dt: f32) -> f32 {
        let mut control_effort;

        if let Some(mut tuning_state) = self.auto_tune {
            // If in auto-tuning mode, handle the tuning logic (relay method)

            // Determine relay output
            if act_velocity_rpm > self.set_point {
                control_effort = tuning_state.output_low;
            } else {
                control_effort = tuning_state.output_high;
            }

            // Detect peaks and crossings to measure oscillations
            let crossed_set_point = (self.prev_process_variable < self.set_point
                && act_velocity_rpm >= self.set_point)
                || (self.prev_process_variable > self.set_point
                    && act_velocity_rpm <= self.set_point);

            if crossed_set_point {
                // Half-cycle is completed, measure the period and amplitude
                let period = tuning_state.time_last_crossing * 2.0;
                let amplitude = (act_velocity_rpm - tuning_state.pv_last_peak).abs();

                // Store the measurements
                if period > 0.0 && amplitude > 0.0 {
                    let peak_period_array_len = tuning_state.peak_periods.len();
                    let peek_amplitude_array_len = tuning_state.peak_amplitudes.len();

                    tuning_state
                        .peak_periods
                        .copy_within(0..(peak_period_array_len - 1), 1);
                    tuning_state
                        .peak_amplitudes
                        .copy_within(0..(peek_amplitude_array_len - 1), 1);
                    tuning_state.peak_periods[0] = period;
                    tuning_state.peak_amplitudes[0] = amplitude;
                    tuning_state.peak_count += 1;
                }

                // Reset for the next half-cycle
                tuning_state.time_last_crossing = 0.0;
                tuning_state.pv_last_peak = act_velocity_rpm;
            }
            tuning_state.time_last_crossing += dt;

            // Check if there is enough data to calculate the tuning parameters
            if tuning_state.peak_count >= tuning_state.peak_amplitudes.len() {
                // Calculate average period (Tu) and amplitude (a)
                let avg_period: f32 = tuning_state.peak_periods.iter().sum::<f32>()
                    / tuning_state.peak_periods.len() as f32;
                let avg_amplitude: f32 = tuning_state.peak_amplitudes.iter().sum::<f32>()
                    / tuning_state.peak_amplitudes.len() as f32;

                let tu = avg_period;
                let a = avg_amplitude;
                let d = (tuning_state.output_high - tuning_state.output_low) / 2.0;

                // Calculate Ultimate Gain (Ku) using the describing function method
                let ku = (4.0 * d) / (a * f32::consts::PI);

                // Calculate gains using Ziegler-Nichols "no overshoot" PID tuning rules
                self.kp = 0.2 * ku;
                self.ki = (0.4 * ku) / tu;
                self.kd = 0.066 * ku * tu;

                // Reset the controller and exit tuning mode
                control_effort = 0.0;
                self.auto_tune = None;
                self.reset();

            } else {
                self.auto_tune = Some(tuning_state);
            }
        } else {
            // Normal PID control logic
            self.error_prev = self.error_curr;
            self.error_curr = self.set_point - act_velocity_rpm;
            self.error_sum += self.error_curr * dt;

            // warn!("Set point: {}, Actual: {}, Error: {}", self.set_point, act_velocity_rpm, self.error_curr);
            control_effort = self.kp * self.error_curr
                + self.ki * self.error_sum
                + self.kd * (self.error_curr - self.error_prev) / dt;
        }

        self.prev_process_variable = act_velocity_rpm;

        if control_effort > self.output_limit {
            control_effort = self.output_limit;
        } else if control_effort < -self.output_limit {
            control_effort = -self.output_limit;
        }

        control_effort
    }

    fn reset(&mut self) {
        self.set_point = 0.0;
        self.error_curr = 0.0;
        self.error_prev = 0.0;
        self.error_sum = 0.0;
    }
}
