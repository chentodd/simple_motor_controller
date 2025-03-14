use std::time::Duration;
use std::{collections::BTreeMap, time::Instant};

use eframe::{
    egui::{self, Ui, Vec2},
    App, CreationContext,
};

use crate::{
    communication::Communication,
    position_command_parser::CommandParser,
    proto::motor_::{MotorRx, MotorTx, Operation},
    view::window_wrapper::{WindowType, WindowWrapper},
    ErrorType, ProfileData, ViewEvent, ViewRequest,
};
use log::error;
use strum::IntoEnumIterator;

#[derive(Default, PartialEq, Eq)]
enum ModeSwitchState {
    #[default]
    Idle,
    Start,
    Wait(Instant),
    Done,
}

pub struct MainWindow {
    communication: Communication,
    position_command_parser: CommandParser,
    window_wrapper: WindowWrapper,

    connection_started: bool,
    view_events: Vec<ViewEvent>,

    // Use ordered map to maintain `Operation`(which is `i32`) in `proto`
    // message. And the commands will be processed in reverse order
    // (from larger one to smaller one)
    requested_mode_map: BTreeMap<i32, ModeSwitchState>,
    requested_mode_finished: bool,
    output_mode: Operation,
    close_event_accepted: bool,
    velocity_command: f32,
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
            if !self.connection_started {
                ui.disable();
            }
            self.window_wrapper
                .get_window(WindowType::CommandWindow)
                .show(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.window_wrapper
                .get_window(WindowType::ProfileWindow)
                .show(ui);
            self.window_wrapper
                .get_window(WindowType::ErrorWindow)
                .show(ui);
        });

        self.handle_close_event(ctx);

        let motor_data_recv = self.collect_motor_data();
        self.handle_ui_request(motor_data_recv.as_ref());
        self.send_ui_event();
        self.process_mode_switch(motor_data_recv.as_ref());
        self.send_motor_command();

        ctx.request_repaint();
    }
}

impl MainWindow {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            communication: Communication::new(),
            position_command_parser: CommandParser::new(),
            window_wrapper: WindowWrapper::new(),
            connection_started: false,
            view_events: Vec::new(),
            requested_mode_map: BTreeMap::new(),
            requested_mode_finished: false,
            output_mode: Operation::default(),
            close_event_accepted: false,
            velocity_command: 0.0,
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

    fn collect_motor_data(&mut self) -> Option<MotorTx> {
        if let Some(data_recv) = self.communication.get_tx_data() {
            let motor_data = &data_recv.left_motor;

            self.view_events
                .push(ViewEvent::ControlModeUpdate(motor_data.operation_display));
            self.view_events
                .push(ViewEvent::ProfileDataUpdate(ProfileData::from(motor_data)));

            Some(motor_data.clone())
        } else {
            None
        }
    }

    fn handle_ui_request(&mut self, motor_data_recv: Option<&MotorTx>) {
        // Handle error first, because we need to reset UI if error appears
        if let Some(request) = self
            .window_wrapper
            .get_window(WindowType::ErrorWindow)
            .take_request()
        {
            match request {
                ViewRequest::ErrorDismiss(prev_error_type) => match prev_error_type {
                    ErrorType::StartError | ErrorType::StopError => {
                        self.communication.reset();
                        self.close_event_accepted = false;

                        for window_type in WindowType::iter() {
                            self.window_wrapper.get_window(window_type).reset();
                        }
                    }
                    ErrorType::ModeSwitchTimeout => {
                        self.close_event_accepted = false;

                        self.window_wrapper
                            .get_window(WindowType::ControlModeWindow)
                            .reset();
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        // Handle other requests and create view events is needed
        for window_type in WindowType::iter() {
            let a = self.window_wrapper.get_window(window_type).take_request();
            if let Some(request) = a {
                match request {
                    ViewRequest::ConnectionStart(port_name) => {
                        if let Err(e) = self.communication.start(&port_name) {
                            self.view_events.push(ViewEvent::ErrorOccurred(
                                ErrorType::StartError,
                                e.to_string(),
                            ));
                        } else {
                            self.connection_started = true;
                            self.view_events
                                .push(ViewEvent::ConnectionStatusUpdate(self.connection_started));
                        }
                    }
                    ViewRequest::ConnectionStop if self.requested_mode_finished => {
                        if let Err(e) = self.communication.stop() {
                            self.view_events.push(ViewEvent::ErrorOccurred(
                                ErrorType::StartError,
                                e.to_string(),
                            ));
                        } else {
                            self.connection_started = false;
                            self.view_events
                                .push(ViewEvent::ConnectionStatusUpdate(self.connection_started));
                        }
                    }
                    ViewRequest::ConnectionStop if !self.requested_mode_finished => {
                        if let Some(data) = motor_data_recv {
                            // When user asks to stop connection, we will do:
                            // 1. Send stop mode to the board, make sure motor is not moving
                            // 2. Send current operation mode, make sure motor stays in current operation mode
                            self.requested_mode_map
                                .entry(data.operation_display.0)
                                .or_insert(ModeSwitchState::Idle);
                            self.requested_mode_map
                                .entry(Operation::Stop.0)
                                .or_insert(ModeSwitchState::Idle);
                        }
                    }
                    ViewRequest::ModeSwitch(target_mode) if !self.requested_mode_finished => {
                        // When user asks to stop connection, we will do:
                        // 1. Send stop mode to the board, make sure motor is not moving
                        // 2. Send target operation mode
                        self.requested_mode_map
                            .entry(target_mode.0)
                            .or_insert(ModeSwitchState::Idle);
                        self.requested_mode_map
                            .entry(Operation::Stop.0)
                            .or_insert(ModeSwitchState::Idle);
                    }
                    ViewRequest::ModeCancel => {
                        self.requested_mode_finished = false;
                        self.close_event_accepted = false;
                    }
                    ViewRequest::VelocityControl(cmd) => {
                        self.output_mode = Operation::IntpVel;
                        self.velocity_command = cmd;
                    }
                    ViewRequest::PositionControl((cmd, ready)) => {
                        if ready {
                            if let Err(e) = self.position_command_parser.parse(&cmd) {
                                self.view_events.push(ViewEvent::ErrorOccurred(
                                    ErrorType::ParseCommandError,
                                    e.to_string(),
                                ));
                            } else {
                                self.output_mode = Operation::IntpPos;
                            }
                        }
                    }
                    _ => (),
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

    fn process_mode_switch(&mut self, motor_data_recv: Option<&MotorTx>) {
        let mut finished_states = 0_usize;

        if let Some(data) = motor_data_recv {
            for (request_mode, switch_state) in self.requested_mode_map.iter_mut().rev() {
                if *switch_state == ModeSwitchState::Done {
                    continue;
                }

                let request_mode = Operation::from(*request_mode);
                match switch_state {
                    ModeSwitchState::Idle => {
                        self.requested_mode_finished = false;

                        if request_mode != data.operation_display {
                            *switch_state = ModeSwitchState::Start;
                        } else {
                            *switch_state = ModeSwitchState::Done;
                        }
                    }
                    ModeSwitchState::Start => {
                        self.output_mode = request_mode;
                        *switch_state = ModeSwitchState::Wait(Instant::now());
                    }
                    ModeSwitchState::Wait(prev) => {
                        // TODO, replace hard-coded time limit
                        let now = Instant::now();
                        if now.duration_since(*prev) >= Duration::from_secs(6) {
                            // Timeout error, clear the map and create error event
                            self.view_events.push(ViewEvent::ErrorOccurred(
                                ErrorType::ModeSwitchTimeout,
                                "Fail to switch mode in given time".to_string(),
                            ));
                            self.requested_mode_map.clear();
                            return;
                        }

                        if request_mode == data.operation_display {
                            *switch_state = ModeSwitchState::Done;
                        }
                    }
                    ModeSwitchState::Done => {
                        finished_states += 1;
                    }
                }
            }
        }

        // If `requested_mode_map` is empty, it is not treated as `Done`
        if !self.requested_mode_map.is_empty() && finished_states == self.requested_mode_map.len() {
            self.requested_mode_finished = true;
            self.requested_mode_map.clear();
        }
    }

    fn handle_close_event(&mut self, ctx: &egui::Context) {
        if self.close_event_accepted && self.requested_mode_finished {
            if let Err(e) = self.communication.stop() {
                error!("Fail to stop `communication` {e}");
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Handle close event when user clicks 'x' button
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.connection_started {
                // When connection is started and user wants to close UI, an internal request is used.
                // This internal request will update `target_mode` in `control_mode_window`, create
                // an effect that user wants to switch to `Stop` mode.
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
