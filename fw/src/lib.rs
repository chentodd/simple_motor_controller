#![no_std]

pub mod motion;
use core::f32;

pub fn rpm_to_rad_s(val: f32) -> f32 {
    val * 2.0 * f32::consts::PI / 60.0
}

pub fn rad_s_to_rpm(val: f32) -> f32 {
    val * 60.0 / (2.0 * f32::consts::PI)
}
