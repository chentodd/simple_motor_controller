use std::fmt::Display;

use eframe::{
    egui::{self, Button, ComboBox, ScrollArea, Slider, TextEdit, Ui, Vec2},
    App, CreationContext,
};

use serial_enumerator::{get_serial_list, SerialInfo};

use crate::proto::motor_::Operation;

#[derive(Default)]
struct ConnectionSettings {
    serial_ports: Vec<SerialInfo>,
    selected_port: String,
    button_clicked: bool,
}

#[derive(Default)]
struct ProfileDataSelection {
    intp_pos: bool,
    intp_vel: bool,
    intp_acc: bool,
    intp_jerk: bool,
    act_pos: bool,
    act_vel: bool,
}

#[derive(Default)]
pub struct MainWindow {
    conn_settings: ConnectionSettings,
    selected_mode: Operation,
    velocity_command: f32,
    position_command: String,
    profile_data_selection: ProfileDataSelection,
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
    }
}

impl MainWindow {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            conn_settings: ConnectionSettings {
                serial_ports: get_serial_list(),
                ..Default::default()
            },
            selected_mode: Operation::IntpVel,
            ..Default::default()
        }
    }

    fn display_connection_panel(&mut self, ui: &mut Ui) {
        let ConnectionSettings {
            serial_ports,
            selected_port,
            button_clicked,
        } = &mut self.conn_settings;

        ui.horizontal_centered(|ui| {
            let curr_selected = &mut selected_port.as_str();
            ComboBox::new("ports", "ports")
                .selected_text(*curr_selected)
                .show_ui(ui, |ui| {
                    for port in serial_ports.iter() {
                        ui.selectable_value(curr_selected, &port.name, &port.name);
                    }
                });
            *selected_port = curr_selected.to_owned();

            let text_in_button = if *button_clicked { "Stop" } else { "Start" };
            let conn_button = Button::new(text_in_button);
            if ui
                .add_enabled(!selected_port.is_empty(), conn_button)
                .clicked()
            {
                *button_clicked = !*button_clicked;
            }
        });

        // TODO, connect to serial port when `button_clicked` is true
    }

    fn display_mode_panel(&mut self, ui: &mut Ui) {
        if !self.conn_settings.button_clicked {
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

        if !self.conn_settings.button_clicked {
            ui.disable();
        }
        ui.add(Slider::new(&mut self.velocity_command, 0.0..=100.0).text("motor velocity ratio"));
    }

    fn display_position_command_panel(&mut self, ui: &mut Ui) {
        if self.selected_mode != Operation::IntpPos {
            return;
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
        if !self.conn_settings.button_clicked {
            ui.disable();
        }

        ui.checkbox(&mut self.profile_data_selection.intp_pos, "Intp pos");
        ui.checkbox(&mut self.profile_data_selection.intp_vel, "Intp vel");
        ui.checkbox(&mut self.profile_data_selection.intp_acc, "Intp acc");
        ui.checkbox(&mut self.profile_data_selection.intp_jerk, "Intp jerk");
        ui.checkbox(&mut self.profile_data_selection.act_pos, "Actual pos");
        ui.checkbox(&mut self.profile_data_selection.act_vel, "Actual vel");
    }
}
