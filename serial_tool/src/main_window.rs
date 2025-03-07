use std::collections::BTreeSet;
use std::fmt::Display;

use eframe::{
    egui::{self, Button, ScrollArea, Slider, TextEdit, Ui, Vec2},
    App, CreationContext,
};
use egui_plot::{Legend, Line, Plot};

use crate::{
    communication::Communication,
    position_command_parser::CommandParser,
    profile_measurement::{MeasurementWindow, ProfileDataType},
    proto::motor_::{MotorRx, MotorTx, Operation},
    view::window_wrapper::{WindowType, WindowWrapper},
    ViewEvent, ViewRequest,
};
use log::error;
use strum::IntoEnumIterator;

#[derive(Default, PartialEq, Eq)]
enum ModeSwitchState {
    #[default]
    Idle,
    Start,
    Wait,
}

pub struct MainWindow {
    measurement_window: MeasurementWindow,
    communication: Communication,
    position_command_parser: CommandParser,

    // Windows
    window_wrapper: WindowWrapper,

    // Other members
    connection_started: bool,
    view_events: Vec<ViewEvent>,

    // Use ordered set to maintain `Operation`(which is `i32`) in `proto`
    // message. And the commands will be processed in reverse order
    // (from larger one to smaller one)
    requested_mode_set: BTreeSet<i32>,
    output_mode: Operation,
    mode_switch_state: ModeSwitchState,

    close_event_accepted: bool,

    velocity_command: f32,
    position_command: String,
    profile_data_flags: [(ProfileDataType, bool); 6],
    start_showing_profile_data: bool,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ErrorType {
    #[default]
    None,
    StartStopError,
    ParseCommandError,
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
                    self.window_wrapper
                        .get_window(WindowType::ConnectionWindow)
                        .show(ui);
                });

                col[1].allocate_ui(Vec2::new(0.0, 0.0), |ui| {
                    if !self.connection_started {
                        ui.disable();
                    }
                    self.window_wrapper
                        .get_window(WindowType::ControlModeWindow)
                        .show(ui);
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
            self.window_wrapper
                .get_window(WindowType::ErrorWindow)
                .show(ui);
        });

        self.handle_close_event(ctx);

        let motor_data_recv = self.collect_motor_data();
        self.handle_ui_request();
        self.send_ui_event();
        self.process_mode_switch(motor_data_recv.as_ref());
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

            // Windows
            window_wrapper: WindowWrapper::new(),

            // Other members
            connection_started: false,
            view_events: Vec::new(),

            requested_mode_set: BTreeSet::new(),
            output_mode: Operation::default(),
            mode_switch_state: ModeSwitchState::default(),

            close_event_accepted: false,

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

    fn display_velocity_command_panel(&mut self, ui: &mut Ui) {
        // if self.mode_switch_window.target_mode != Operation::IntpVel {
        //     return;
        // }

        if !self.connection_started {
            ui.disable();
        }
        ui.add(
            Slider::new(&mut self.velocity_command, -3000.0..=3000.0)
                .text("motor velocity cmd (rpm)"),
        );
    }

    fn display_position_command_panel(&mut self, ui: &mut Ui) {
        // if self.mode_switch_window.target_mode != Operation::IntpPos {
        //     return;
        // }

        if !self.connection_started {
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
                    self.view_events.push(ViewEvent::ErrorOccurred(
                        ErrorType::ParseCommandError,
                        e.to_string(),
                    ));
                }
            }
        }
    }

    fn display_profile_control_panel(&mut self, ui: &mut Ui) {
        if !self.connection_started {
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

    fn send_motor_command(&mut self) {
        let mut motor_commads = Vec::new();

        match self.output_mode {
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

    fn handle_ui_request(&mut self) {
        for window_type in WindowType::iter() {
            if let Some(request) = self.window_wrapper.get_window(window_type).take_request() {
                match request {
                    ViewRequest::ErrorDismissed(prev_error_type) => match prev_error_type {
                        ErrorType::StartStopError => {
                            self.measurement_window.reset();
                            self.communication.reset();
                        }
                        _ => (),
                    },
                    ViewRequest::Connection(start, port_name) => {
                        let start_stop_result = match start {
                            true => self.communication.start(&port_name),
                            false => {
                                self.measurement_window.reset();
                                self.communication.stop()
                            }
                        };

                        if let Err(e) = start_stop_result {
                            self.view_events.push(ViewEvent::ErrorOccurred(
                                ErrorType::StartStopError,
                                e.to_string(),
                            ));
                        } else {
                            self.connection_started = start;
                        }
                    }
                    ViewRequest::ModeSwitch(target_mode) => {
                        self.requested_mode_set.insert(target_mode.0);
                        self.requested_mode_set.insert(Operation::Stop.0);
                    }
                }
            }
        }
    }

    fn send_ui_event(&mut self) {
        while let Some(event) = self.view_events.pop() {
            for window_type in WindowType::iter() {
                self.window_wrapper
                    .get_window(window_type)
                    .handle_event(event.clone());
            }
        }
    }

    fn collect_motor_data(&mut self) -> Option<MotorTx> {
        if let Some(data_recv) = self.communication.get_tx_data() {
            let motor_data = &data_recv.left_motor;
            self.view_events
                .push(ViewEvent::ControlModeUpdate(motor_data.operation_display));
            Some(motor_data.clone())
        } else {
            None
        }
    }

    fn process_mode_switch(&mut self, motor_data_recv: Option<&MotorTx>) {
        if let Some(data) = motor_data_recv {
            if let Some(req_mode) = self.requested_mode_set.last() {
                let req_mode = Operation::from(*req_mode);
                match self.mode_switch_state {
                    ModeSwitchState::Idle => {
                        if req_mode != data.operation_display {
                            self.mode_switch_state = ModeSwitchState::Start;
                        }
                    }
                    ModeSwitchState::Start => {
                        self.output_mode = req_mode;
                        self.mode_switch_state = ModeSwitchState::Wait;
                    }
                    ModeSwitchState::Wait => {
                        if req_mode == data.operation_display {
                            self.requested_mode_set.pop_last();
                            self.mode_switch_state = ModeSwitchState::Idle;
                        }
                    }
                }
            }
        }
    }

    fn handle_close_event(&mut self, ctx: &egui::Context) {
        if self.close_event_accepted && self.mode_switch_state == ModeSwitchState::Idle {
            if let Err(e) = self.communication.stop() {
                error!("Fail to stop `communication` {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Handle close event when user clicks 'x' button
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.connection_started {
                // Need to use `Stop` command when connection is started
                self.close_event_accepted = true;
                self.view_events
                    .push(ViewEvent::InternalControlModeRequest((
                        Operation::Stop,
                        "Exit".to_string(),
                    )));

                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }
    }
}
