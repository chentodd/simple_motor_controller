use std::collections::VecDeque;

use crate::{ProfileData, ProfileDataType, UiView, ViewEvent, ViewRequest};
use egui_plot::{Legend, Line, Plot, PlotPoints};

pub struct DataGraph {
    window_values: VecDeque<ProfileData>,
    window_size: usize,
    data_flags: [(ProfileDataType, bool); 6],
    can_update: bool,
}

impl DataGraph {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_values: VecDeque::new(),
            window_size,
            data_flags: [
                (ProfileDataType::IntpPos, false),
                (ProfileDataType::IntpVel, false),
                (ProfileDataType::IntpAcc, false),
                (ProfileDataType::IntpJerk, false),
                (ProfileDataType::ActPos, false),
                (ProfileDataType::ActVel, false),
            ],
            can_update: false,
        }
    }

    fn add_data_point(&mut self, data: ProfileData) {
        if !self.can_update {
            return;
        }

        if self.window_values.len() == self.window_size {
            self.window_values.pop_front();
        }
        self.window_values.push_back(data);
    }

    fn get_data(&self, get_data_type: ProfileDataType) -> PlotPoints {
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

impl UiView for DataGraph {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        let x = ui.available_width();
        let y = ui.available_height();

        ui.horizontal(|ui| {
            Plot::new("profile_data")
                .legend(Legend::default())
                .width(x * 0.85)
                .height(y)
                .show(ui, |plot_ui| {
                    for (data_type, enable) in self.data_flags.iter() {
                        if *enable {
                            let data_points = self.get_data(*data_type);
                            plot_ui.line(Line::new(data_points).name(data_type.to_string()));
                        }
                    }
                });

            ui.vertical(|ui| {
                let text_in_button = if !self.can_update { "▶" } else { "⏸" };

                if ui.button(text_in_button).clicked() {
                    self.can_update = !self.can_update;
                }

                for item in self.data_flags.iter_mut() {
                    ui.checkbox(&mut item.1, item.0.to_string());
                }
            })
        });
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        None
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ProfileDataUpdate(data) => {
                self.add_data_point(data);
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        self.window_values.clear();
    }
}
