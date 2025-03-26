use gnuplot::Coordinate::Graph;
use gnuplot::{AxesCommon, Caption, Figure};
use s_curve::*;
use std::fs::File;

const T: f32 = 0.001;

fn main() {
    let mut s_curve_interpolator = SCurveInterpolator::new(10.0, 10.0, 30.0, T);
    s_curve_interpolator.set_target(0.0, -10.0, 1.0, 0.0, 5.0);

    let mut time = 0.0;
    let mut time_stamps = Vec::new();
    let mut jerk = Vec::new();
    let mut acc = Vec::new();
    let mut vel = Vec::new();
    let mut pos = Vec::new();

    let mut file = File::create("./record.txt").unwrap();

    while s_curve_interpolator.get_intp_status() != InterpolationStatus::Done {
        s_curve_interpolator.interpolate();

        let s_curve_intp_data = s_curve_interpolator.get_intp_data();
        let dir = s_curve_interpolator.get_dir();

        time += T;
        time_stamps.push(time);
        jerk.push(s_curve_intp_data.jerk * dir);
        acc.push(s_curve_intp_data.acc * dir);
        vel.push(s_curve_intp_data.vel * dir);
        pos.push(s_curve_intp_data.pos * dir);

        s_curve_interpolator.save_intp_data(&mut file);
    }

    // plot
    let mut fg = Figure::new();
    fg.axes2d()
        .set_title("S-Curve Velocity Motion Profile", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .set_x_label("time in seconds", &[])
        .set_y_label("Position derivatives m, m/s, m/s², m/s³", &[])
        .lines(time_stamps.clone(), pos.clone(), &[Caption("Position")])
        .lines(time_stamps.clone(), vel.clone(), &[Caption("Velocity")])
        .lines(time_stamps.clone(), acc.clone(), &[Caption("Acceleration")])
        .lines(time_stamps.clone(), jerk.clone(), &[Caption("Jerk")]);
    fg.show().unwrap();
}
