use std::fmt::Display;
use std::sync::mpsc::Receiver;

use eframe::{
    egui::{self, Button, ComboBox, ScrollArea, Slider, TextEdit, Ui, Vec2},
    App, CreationContext,
};
use egui_plot::{Legend, Line, Plot};

use crate::communication::ConnectionSettings;
use crate::profile_measurement::{MeasurementWindow, ProfileData, ProfileDataType};
use crate::proto::motor_::Operation;

pub struct MainWindow {
    conn_settings: ConnectionSettings,
    measurement_window: MeasurementWindow,
    selected_mode: Operation,
    selected_port: String,
    conn_button_clicked: bool,
    velocity_command: f32,
    position_command: String,
    profile_data_flags: [(ProfileDataType, bool); 6],
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Operation::from(self.0) {
            Operation::Unspecified => write!(f, "Unspecified"),
            Operation::IntpPos => write!(f, "IntpPos"),
            Operation::IntpVel => write!(f, "IntpVel"),
            _ => Ok(()),
        }
    }
}

impl App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("funtionality_panel").show(ctx, |ui| {
            ui.columns(2, |col| {
                col[0].allocate_ui(Vec2::new(0.0, 0.0), |ui| {
                    ui.heading("Connection setup");
                    self.display_connection_panel(ui);
                });

                col[1].allocate_ui(Vec2::new(0.0, 0.0), |ui| {
                    ui.heading("Mode setup");
                    self.display_mode_panel(ui);
                });
            });
        });

        egui::TopBottomPanel::top("command_panel").show(ctx, |ui: &mut Ui| {
            ui.heading("Command setup");
            self.display_velocity_command_panel(ui);
            self.display_position_command_panel(ui);
        });

        egui::SidePanel::right("profile_data_selection_panel").show(ctx, |ui| {
            self.display_profile_data_selection_panel(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.display_profile_data_graph(ui);
        });
    }
}

impl MainWindow {
    pub fn new(
        _cc: &CreationContext<'_>,
        window_size: usize,
        data_receiver: Receiver<ProfileData>,
    ) -> Self {
        Self {
            conn_settings: ConnectionSettings::new(),
            measurement_window: MeasurementWindow::new(window_size, data_receiver),
            selected_mode: Operation::IntpVel,
            selected_port: "".to_string(),
            conn_button_clicked: false,
            velocity_command: 0.0,
            position_command: "".to_string(),
            profile_data_flags: [
                (ProfileDataType::IntpPos, false),
                (ProfileDataType::IntpPos, false),
                (ProfileDataType::IntpPos, false),
                (ProfileDataType::IntpPos, false),
                (ProfileDataType::IntpPos, false),
                (ProfileDataType::IntpPos, false),
            ],
        }
    }

    fn display_connection_panel(&mut self, ui: &mut Ui) {
        let port_names = self.conn_settings.get_port_names();

        ui.horizontal_centered(|ui| {
            let curr_selected = &mut self.selected_port.as_str();
            ComboBox::new("ports", "ports")
                .selected_text(*curr_selected)
                .show_ui(ui, |ui| {
                    for port in port_names {
                        ui.selectable_value(curr_selected, port, port);
                    }
                });
            self.selected_port = curr_selected.to_owned();

            let text_in_button = if self.conn_button_clicked {
                "Stop"
            } else {
                "Start"
            };

            let conn_button = Button::new(text_in_button);
            if ui
                .add_enabled(!self.selected_port.is_empty(), conn_button)
                .clicked()
            {
                self.conn_button_clicked = !self.conn_button_clicked;
            }
        });

        // TODO, connect to serial port when `button_clicked` is true
    }

    fn display_mode_panel(&mut self, ui: &mut Ui) {
        if !self.conn_button_clicked {
            ui.disable();
        }
        let curr_selected = &mut self.selected_mode;
        ComboBox::new("control_mods", "control modes")
            .selected_text(format!("{}", curr_selected))
            .show_ui(ui, |ui| {
                ui.selectable_value(curr_selected, Operation::IntpPos, "IntpPos");
                ui.selectable_value(curr_selected, Operation::IntpVel, "IntpVel");
            });

        self.selected_mode = *curr_selected;
    }

    fn display_velocity_command_panel(&mut self, ui: &mut Ui) {
        if self.selected_mode != Operation::IntpVel {
            return;
        }

        if !self.conn_button_clicked {
            ui.disable();
        }
        ui.add(Slider::new(&mut self.velocity_command, 0.0..=100.0).text("motor velocity ratio"));
    }

    fn display_position_command_panel(&mut self, ui: &mut Ui) {
        if self.selected_mode != Operation::IntpPos {
            return;
        }

        if !self.conn_button_clicked {
            ui.disable();
        }

        ScrollArea::vertical().max_height(64.0).show(ui, |ui| {
            ui.add_sized(
                ui.available_size(),
                TextEdit::multiline(&mut self.position_command),
            );
        });

        let send_button = Button::new("Send");
        if ui
            .add_enabled(!self.position_command.is_empty(), send_button)
            .clicked()
        {
            // TODO, send position commands
        }
    }

    fn display_profile_data_selection_panel(&mut self, ui: &mut Ui) {
        if !self.conn_button_clicked {
            ui.disable();
        }

        for item in self.profile_data_flags.iter_mut() {
            ui.checkbox(&mut item.1, item.0.to_string());
        }
    }

    fn display_profile_data_graph(&mut self, ui: &mut Ui) {
        Plot::new("profile_data")
            .legend(Legend::default())
            .show(ui, |plot_ui| {
                for (data_type, enable) in self.profile_data_flags.iter() {
                    if *enable {
                        let data_points = self.measurement_window.get_data(*data_type);
                        plot_ui.line(Line::new(data_points).name(data_type.to_string()));
                    }
                }
            });

        self.measurement_window.update_measurement_window();
    }
}
