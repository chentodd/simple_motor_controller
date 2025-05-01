use eframe::{
    App, CreationContext,
    egui::{self, Ui, Vec2},
};

use protocol::{ControlMode, MotorCommand, MotorProcessData};

use crate::{
    ErrorType, ProfileData, ViewEvent, ViewRequest,
    controller::communication::Communication,
    controller::mode_switch::ModeSwitch,
    controller::position_command_parser::CommandParser,
    view::window_wrapper::{WindowType, WindowWrapper},
};

use log::error;
use strum::IntoEnumIterator;

#[derive(PartialEq, Eq, Clone, Copy)]
enum InternalRequestType {
    StopConnection,
    CloseApp,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum InternalRequestState {
    #[default]
    Idle,
    Ignite(InternalRequestType),
    Confirm(InternalRequestType),
}

pub struct TuningTool {
    // USB communication using `postcard-rpc`
    communication: Option<Communication>,

    // Mode switch
    mode_switch: ModeSwitch<6>,
    internal_request_state: InternalRequestState,

    // UI
    window_wrapper: WindowWrapper,
    position_command_parser: CommandParser,
    view_events: Vec<ViewEvent>,

    // Others
    velocity_command: f32,
}

impl App for TuningTool {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("funtionality_panel").show(ctx, |ui| {
            ui.columns(2, |col| {
                col[0].allocate_ui(Vec2::new(0.0, 0.0), |ui| {
                    self.window_wrapper
                        .get_window(WindowType::ConnectionWindow)
                        .show(ui);
                });

                col[1].allocate_ui(Vec2::new(0.0, 0.0), |ui| {
                    if self.communication.is_none() {
                        ui.disable();
                    }
                    self.window_wrapper
                        .get_window(WindowType::ControlModeWindow)
                        .show(ui);
                });
            });
        });

        egui::TopBottomPanel::top("command_panel").show(ctx, |ui: &mut Ui| {
            if self.communication.is_none() {
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

        self.handle_communication_error();

        let motor_data_recv = self.collect_motor_data();
        self.collect_conn_status();

        self.handle_close_event(ctx);
        self.handle_ui_request();
        self.send_ui_event();

        let mode_switch_result = self.mode_switch.process(&motor_data_recv);
        if let Err(e) = mode_switch_result {
            self.view_events.push(ViewEvent::ErrorOccurred(
                e,
                "Mode switch failed".to_string(),
            ));
        } else {
            // Send motor command when mode switch gives valud output mode
            self.send_motor_command(mode_switch_result.unwrap());
        }
        self.process_internal_request(&ctx);

        ctx.request_repaint();
    }
}

impl TuningTool {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            communication: None,

            mode_switch: ModeSwitch::new(),
            internal_request_state: InternalRequestState::default(),

            window_wrapper: WindowWrapper::new(),
            position_command_parser: CommandParser::new(),
            view_events: Vec::new(),

            velocity_command: 0.0,
        }
    }

    fn reset(&mut self, communication_stopped: bool) {
        self.internal_request_state = InternalRequestState::Idle;
        self.view_events.clear();
        if communication_stopped {
            // Clear other data when communication is stopped
            self.velocity_command = 0.0;
            self.position_command_parser.reset();
        }
    }

    fn send_motor_command(&mut self, output_mode: ControlMode) {
        if self.communication.is_none() {
            return;
        }

        let communication = self.communication.as_mut().unwrap();
        match output_mode {
            ControlMode::Position => {
                if self.position_command_parser.have_data() {
                    while let Some(cmd) = self.position_command_parser.get_command() {
                        communication.send_motor_command(MotorCommand::PositionCommand(cmd));
                    }
                }
            }
            ControlMode::Velocity => communication
                .send_motor_command(MotorCommand::VelocityCommand(self.velocity_command)),
            _ => (),
        }
    }

    fn collect_motor_data(&mut self) -> MotorProcessData {
        if self.communication.is_none() {
            return MotorProcessData::default();
        }

        let communication = self.communication.as_ref().unwrap();
        let motor_data = communication.get_motor_process_data();

        self.view_events.push(ViewEvent::ControlModeUpdate((
            self.mode_switch.is_finished(),
            motor_data.control_mode_display,
        )));

        self.view_events
            .push(ViewEvent::ProfileDataUpdate(ProfileData::from(&motor_data)));

        motor_data
    }

    fn collect_conn_status(&mut self) {
        self.view_events.push(ViewEvent::ConnectionStatusUpdate(
            self.communication.is_some(),
        ));
    }

    fn handle_communication_error(&mut self) {
        if self.communication.is_none() {
            return;
        }

        let communication = self.communication.as_ref().unwrap();
        if let Err(e) = communication.get_motor_command_actor_err() {
            self.view_events
                .push(ViewEvent::ErrorOccurred(ErrorType::CommunicationError, e));
        }

        if let Err(e) = communication.get_motor_data_actor_err() {
            self.view_events
                .push(ViewEvent::ErrorOccurred(ErrorType::CommunicationError, e));
        }
    }

    fn handle_ui_request(&mut self) {
        // Handle error first, because we need to reset UI if error appears
        if let Some(request) = self
            .window_wrapper
            .get_window(WindowType::ErrorWindow)
            .take_request()
        {
            match request {
                ViewRequest::ErrorDismiss(prev_error_type) => match prev_error_type {
                    ErrorType::StartError | ErrorType::StopError => {
                        self.communication.take();
                        self.reset(false);

                        for window_type in WindowType::iter() {
                            self.window_wrapper.get_window(window_type).reset();
                        }
                    }
                    ErrorType::ModeSwitchTimeout => {
                        error!("handle mode switch error");
                        self.mode_switch.reset();
                        self.reset(false);

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
                        match Communication::new(&port_name) {
                            Ok(comm) => {
                                self.communication = Some(comm);
                                self.view_events
                                    .push(ViewEvent::ConnectionStatusUpdate(true));
                            }
                            Err(e) => {
                                self.view_events
                                    .push(ViewEvent::ErrorOccurred(ErrorType::StartError, e));
                            }
                        }
                    }
                    ViewRequest::ConnectionStop => {
                        // Send internal request to control mode window, ask user to confirm this operation
                        self.internal_request_state =
                            InternalRequestState::Ignite(InternalRequestType::StopConnection);
                        self.view_events.push(ViewEvent::InternalStopModeRequest(
                            "Stop connection".to_string(),
                        ));
                    }
                    ViewRequest::ModeSwitch(target_mode) => {
                        self.mode_switch.ignite(target_mode);

                        self.internal_request_state = match &self.internal_request_state {
                            InternalRequestState::Ignite(x) => InternalRequestState::Confirm(*x),
                            _ => InternalRequestState::Idle,
                        };
                    }
                    ViewRequest::ModeCancel => {
                        self.mode_switch.reset();
                        self.reset(false);
                    }
                    ViewRequest::VelocityControl(cmd) => {
                        self.mode_switch.ignite(ControlMode::Velocity);
                        self.velocity_command = cmd;
                    }
                    ViewRequest::PositionControl(cmd) => {
                        if let Err(e) = self.position_command_parser.parse(&cmd) {
                            self.view_events.push(ViewEvent::ErrorOccurred(
                                ErrorType::ParseCommandError,
                                e.to_string(),
                            ));
                        } else {
                            self.mode_switch.ignite(ControlMode::Position);
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

    fn handle_close_event(&mut self, ctx: &egui::Context) {
        // Handle close event when user clicks 'x' button
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.communication.is_some()
                && self.internal_request_state == InternalRequestState::Idle
            {
                // When connection is started and user wants to close UI, an internal request is used.
                // This internal request will update `target_mode` in `control_mode_window`, create
                // an effect that user wants to switch to `Stop` mode.
                self.internal_request_state =
                    InternalRequestState::Ignite(InternalRequestType::CloseApp);
                self.view_events
                    .push(ViewEvent::InternalStopModeRequest("Exit".to_string()));

                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }
    }

    fn process_internal_request(&mut self, ctx: &egui::Context) {
        if self.communication.is_none() {
            return;
        }

        let internal_request_state = self.internal_request_state;
        let communication = self.communication.as_ref().unwrap();
        match internal_request_state {
            InternalRequestState::Confirm(req_type) => {
                if self.mode_switch.is_finished() {
                    // The internal type is stop connection or close app, and they all need to stop connection
                    communication.stop();

                    match req_type {
                        InternalRequestType::CloseApp => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
}
