use std::fmt::Display;

use eframe::{
    egui::{self, Button, ComboBox, ScrollArea, Slider, TextEdit, Ui, Vec2},
    App, CreationContext,
};
use egui_plot::{Legend, Line, Plot};

use log::error;

use crate::{
    communication::{Communication, Settings},
    position_command_parser::CommandParser,
    profile_measurement::{MeasurementWindow, ProfileDataType},
    proto::motor_::{MotorRx, Operation},
};

pub struct MainWindow {
    measurement_window: MeasurementWindow,
    communication: Communication,
    position_command_parser: CommandParser,
    error_window: ErrorWindow,
    selected_mode: Operation,
    selected_port: String,
    conn_button_clicked: bool,
    velocity_command: f32,
    position_command: String,
    profile_data_flags: [(ProfileDataType, bool); 6],
    start_showing_profile_data: bool,
}

#[derive(Default, PartialEq, Eq)]
enum ErrorType {
    #[default]
    None,
    StartStopError,
    ParseCommandError,
}

#[derive(Default)]
struct ErrorWindow {
    error_type: ErrorType,
    error_message: String,
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
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(_e) = self.communication.stop() {
            error!("on_exit(), failed, {_e}");
        }
    }

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

        egui::SidePanel::right("profile_data_control_panel").show(ctx, |ui| {
            self.display_profile_control_panel(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.display_profile_data_graph(ui);
            self.display_error_window(ui);
        });

        self.send_motor_command();

        ctx.request_repaint();
    }
}

impl MainWindow {
    pub fn new(_cc: &CreationContext<'_>, window_size: usize) -> Self {
        Self {
            measurement_window: MeasurementWindow::new(window_size),
            communication: Communication::new(),
            position_command_parser: CommandParser::new(),
            error_window: ErrorWindow::default(),
            selected_mode: Operation::IntpVel,
            selected_port: "".to_string(),
            conn_button_clicked: false,
            velocity_command: 0.0,
            position_command: "".to_string(),
            profile_data_flags: [
                (ProfileDataType::IntpPos, false),
                (ProfileDataType::IntpVel, false),
                (ProfileDataType::IntpAcc, false),
                (ProfileDataType::IntpJerk, false),
                (ProfileDataType::ActPos, false),
                (ProfileDataType::ActVel, false),
            ],
            start_showing_profile_data: false,
        }
    }

    fn display_connection_panel(&mut self, ui: &mut Ui) {
        let port_names = Settings::get_port_names();

        ui.horizontal_centered(|ui| {
            let curr_selected = &mut self.selected_port.as_str();
            ComboBox::new("ports", "ports")
                .selected_text(*curr_selected)
                .show_ui(ui, |ui| {
                    for port in &port_names {
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

                let start_stop_result = match self.conn_button_clicked {
                    true => self.communication.start(&self.selected_port),
                    false => {
                        self.measurement_window.reset();
                        self.communication.stop()
                    }
                };

                if let Err(e) = start_stop_result {
                    self.error_window.error_type = ErrorType::StartStopError;
                    self.error_window.error_message = e.to_string();
                }
            }
        });
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
        ui.add(
            Slider::new(&mut self.velocity_command, -3000.0..=3000.0)
                .text("motor velocity cmd (rpm)"),
        );
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
            match self.position_command_parser.parse(&self.position_command) {
                Ok(_) => (),
                Err(e) => {
                    self.error_window.error_type = ErrorType::ParseCommandError;
                    self.error_window.error_message = e.to_string();
                }
            }
        }
    }

    fn display_profile_control_panel(&mut self, ui: &mut Ui) {
        if !self.conn_button_clicked {
            ui.disable();
        }

        let text_in_button = if !self.start_showing_profile_data {
            "▶"
        } else {
            "⏸"
        };

        if ui.button(text_in_button).clicked() {
            self.start_showing_profile_data = !self.start_showing_profile_data;
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

        if let Some(data) = self.communication.get_tx_data() {
            if self.start_showing_profile_data {
                self.measurement_window.update_measurement_window(data);
            }
        }
    }

    fn display_error_window(&mut self, ui: &mut Ui) {
        if self.error_window.error_type == ErrorType::None {
            return;
        }

        egui::Window::new("Error")
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!("❌ {}", self.error_window.error_message));
                if ui.button("Ok").clicked() {
                    self.error_window.error_type = ErrorType::None;
                    self.error_window.error_message.clear();

                    match self.error_window.error_type {
                        ErrorType::StartStopError => {
                            // Clear data when button is clicked
                            self.measurement_window.reset();
                            self.communication.reset();
                        }
                        _ => (),
                    }
                }
            });
    }

    fn send_motor_command(&mut self) {
        let mut motor_commads = Vec::new();

        match self.selected_mode {
            Operation::IntpVel => {
                let mut vel_cmd = MotorRx::default();
                vel_cmd.set_target_vel(self.velocity_command);

                motor_commads.push(vel_cmd);
            }
            Operation::IntpPos => {
                while let Some(cmd) = self.position_command_parser.get_command() {
                    let mut pos_cmd = MotorRx::default();
                    pos_cmd.set_target_dist(cmd.dist);
                    pos_cmd.set_target_vel(cmd.vel);
                    pos_cmd.set_target_vel_end(cmd.vel_end);

                    motor_commads.push(pos_cmd);
                }
            }
            _ => (),
        }

        for motor_cmd in motor_commads {
            self.communication.set_rx_data(motor_cmd);
        }
    }
}
