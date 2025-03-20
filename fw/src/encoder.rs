use embassy_stm32::timer::qei::*;
use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_stm32::timer::{Channel1Pin, Channel2Pin};
use embassy_stm32::Peripheral;
use heapless::Vec;

pub struct Encoder<'a, T: GeneralInstance4Channel, const COUNTS_PER_REV: u16> {
    qei: Qei<'a, T>,
    act_vel: f32,
    f_values: Vec<f32, 3>,
    curr_enc_count: i32,
    prev_qei_count: i16,
    curr_qei_count: i16,
}

impl<'a, T: GeneralInstance4Channel, const COUNTS_PER_REV: u16> Encoder<'a, T, COUNTS_PER_REV> {
    pub fn new(
        tim: impl Peripheral<P = T> + 'a,
        enc_a_pin: impl Peripheral<P = impl Channel1Pin<T>> + 'a,
        enc_b_pin: impl Peripheral<P = impl Channel2Pin<T>> + 'a,
    ) -> Self {
        let enc_a_pin = QeiPin::new_ch1(enc_a_pin);
        let enc_b_pin = QeiPin::new_ch2(enc_b_pin);
        Self {
            qei: Qei::new(tim, enc_a_pin, enc_b_pin),
            act_vel: 0.0,
            f_values: Vec::new(),
            curr_enc_count: 0,
            prev_qei_count: 0,
            curr_qei_count: 0,
        }
    }

    pub fn get_enc_count(&self) -> i32 {
        self.curr_enc_count
    }

    pub fn get_act_position_in_rad(&self) -> f32 {
        (self.curr_enc_count as f32) / (COUNTS_PER_REV as f32)
    }

    pub fn get_act_velocity_in_rpm(&self) -> f32 {
        self.act_vel
    }

    pub fn update_act_velocity_in_rpm(&mut self, period_s: f32) {
        self.update_encoder_count();

        let prev_count = self.f_values.last().cloned().unwrap_or(0.0);
        let curr_count = self.curr_enc_count as f32;
        let filtered_enc = 0.8 * prev_count + 0.2 * curr_count;
        let vel_count_s = if !self.f_values.is_full() {
            let _ = self.f_values.push(filtered_enc);
            (filtered_enc - prev_count) as f32 / period_s
        } else {
            let n = self.f_values.len();
            self.f_values.rotate_right(n - 1);
            self.f_values[n - 1] = filtered_enc;

            let f0 = &self.f_values[n - 1];
            let f1 = &self.f_values[n - 2];
            let f2 = &self.f_values[n - 3];
            (3.0 * f0 - 4.0 * f1 + f2) as f32 / (2.0 * period_s)
        };

        self.act_vel = 60.0 * vel_count_s / (COUNTS_PER_REV as f32);
    }

    fn update_encoder_count(&mut self) {
        self.curr_qei_count = self.qei.count() as i16;
        self.curr_enc_count += self.curr_qei_count.wrapping_sub(self.prev_qei_count) as i32;
        self.prev_qei_count = self.curr_qei_count;
    }
}
