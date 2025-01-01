#![no_std]

pub mod encoder;
pub mod motion;
pub mod motor;
pub mod pid;
pub mod serial;

pub mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!("proto_packet.rs");
}

use core::f32;

pub fn rpm_to_rad_s(val: f32) -> f32 {
    val * 2.0 * f32::consts::PI / 60.0
}

pub fn rad_s_to_rpm(val: f32) -> f32 {
    val * 60.0 / (2.0 * f32::consts::PI)
}
