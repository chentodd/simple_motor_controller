use core::f32::consts::PI;

use embassy_stm32::timer::qei::*;
use embassy_stm32::timer::GeneralInstance4Channel;
use embassy_stm32::timer::{Channel1Pin, Channel2Pin};
use embassy_stm32::Peripheral;

pub struct Encoder<'a, T: GeneralInstance4Channel, const COUNTS_PER_REV: u16> {
    qei: Qei<'a, T>,
    act_vel: f32,
    act_pos: f32,
    curr_enc_count: i32,
    prev_enc_count: i32,
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
            act_pos: 0.0,
            curr_enc_count: 0,
            prev_enc_count: 0,
            prev_qei_count: 0,
            curr_qei_count: 0,
        }
    }

    pub fn get_enc_count(&self) -> i32 {
        self.curr_enc_count
    }

    pub fn get_act_position_in_rad(&self) -> f32 {
        self.act_pos
    }

    pub fn get_act_velocity_in_rpm(&self) -> f32 {
        self.act_vel
    }

    pub fn update_act_velocity_in_rpm(&mut self, period_s: f32) {
        self.update_encoder_count();

        let diff_count: f32 = (self.curr_enc_count - self.prev_enc_count) as f32;
        let round_s = diff_count / period_s / (COUNTS_PER_REV as f32);
        self.act_vel = 60.0 * round_s;
        self.act_pos += 2.0 * PI * round_s * period_s;

        self.prev_enc_count = self.curr_enc_count;
    }

    fn update_encoder_count(&mut self) {
        self.curr_qei_count = self.qei.count() as i16;
        self.curr_enc_count += self.curr_qei_count.wrapping_sub(self.prev_qei_count) as i32;
        self.prev_qei_count = self.curr_qei_count;
    }
}
