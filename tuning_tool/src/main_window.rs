use eframe::{
    App, CreationContext,
    egui::{self, Ui, Vec2},
};

use crate::{
    ErrorType, ProfileData, ViewEvent, ViewRequest,
    communication::Communication,
    mode_switch::ModeSwitch,
    position_command_parser::CommandParser,
    proto::motor_::{MotorRx, MotorTx, Operation},
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

pub struct MainWindow {
    // Serial communication
    communication: Communication,
    connection_started: bool,

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

        let motor_data_recv = self.collect_motor_data();
        self.collect_conn_status();

        self.handle_close_event(ctx);
        self.handle_ui_request();
        self.send_ui_event();

        let mode_switch_result = self.mode_switch.process(motor_data_recv.as_ref());
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

impl MainWindow {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            communication: Communication::new(),
            connection_started: false,

            mode_switch: ModeSwitch::new(),
            internal_request_state: InternalRequestState::default(),

            window_wrapper: WindowWrapper::new(),
            position_command_parser: CommandParser::new(),
            view_events: Vec::new(),

            velocity_command: 0.0,
        }
    }

    fn reset(&mut self, is_stop_ok: bool) {
        self.internal_request_state = InternalRequestState::Idle;
        self.view_events.clear();
        if is_stop_ok {
            // Clear other data when stop process is succeeded
            self.velocity_command = 0.0;
            self.position_command_parser.reset();
            self.connection_started = false;
        }
    }

    fn send_motor_command(&mut self, output_mode: Operation) {
        if !self.connection_started {
            return;
        }

        let mut motor_commads = Vec::new();
        motor_commads.push(MotorRx::default());

        match output_mode {
            Operation::IntpVel => {
                let first = motor_commads.first_mut().unwrap();
                first.set_target_vel(self.velocity_command);
            }
            Operation::IntpPos => {
                if self.position_command_parser.have_data() {
                    motor_commads.clear();
                    while let Some(cmd) = self.position_command_parser.get_command() {
                        let mut pos_cmd = MotorRx::default();

                        pos_cmd.set_target_dist(cmd.dist);
                        pos_cmd.set_target_vel(cmd.vel);
                        pos_cmd.set_target_vel_end(cmd.vel_end);

                        motor_commads.push(pos_cmd);
                    }
                }
            }
            _ => (),
        }

        for mut motor_cmd in motor_commads {
            motor_cmd.operation = output_mode;
            self.communication.set_rx_data(motor_cmd);
        }
    }

    fn collect_motor_data(&mut self) -> Option<MotorTx> {
        if let Some(motor_data) = self.communication.get_tx_data() {
            self.view_events.push(ViewEvent::ControlModeUpdate((
                self.mode_switch.is_finished(),
                motor_data.operation_display,
            )));
            self.view_events
                .push(ViewEvent::ProfileDataUpdate(ProfileData::from(&motor_data)));

            Some(motor_data.clone())
        } else {
            None
        }
    }

    fn collect_conn_status(&mut self) {
        self.view_events
            .push(ViewEvent::ConnectionStatusUpdate(self.connection_started));
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
                        self.communication.reset();
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
                        if let Err(e) = self.communication.start(&port_name) {
                            self.view_events.push(ViewEvent::ErrorOccurred(
                                ErrorType::StartError,
                                e.to_string(),
                            ));
                        } else {
                            self.connection_started = true;
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
                        self.mode_switch.ignite(Operation::IntpVel);
                        self.velocity_command = cmd;
                    }
                    ViewRequest::PositionControl(cmd) => {
                        if let Err(e) = self.position_command_parser.parse(&cmd) {
                            self.view_events.push(ViewEvent::ErrorOccurred(
                                ErrorType::ParseCommandError,
                                e.to_string(),
                            ));
                        } else {
                            self.mode_switch.ignite(Operation::IntpPos);
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
            if self.connection_started && self.internal_request_state == InternalRequestState::Idle
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
        let internal_request_state = self.internal_request_state;
        match internal_request_state {
            InternalRequestState::Confirm(req_type) => {
                if self.mode_switch.is_finished() {
                    // The internal type is stop connection or close app, and they all need to stop connection
                    let stop_result = self.communication.stop();
                    if let Err(e) = stop_result {
                        error!("Fail to stop `communication` {e}");
                    } else {
                        error!("Stop connection ok");
                        self.reset(true);
                    }

                    match req_type {
                        InternalRequestType::StopConnection => {
                            if let Err(e) = stop_result {
                                self.view_events.push(ViewEvent::ErrorOccurred(
                                    ErrorType::StopError,
                                    e.to_string(),
                                ));
                            }
                        }
                        InternalRequestType::CloseApp => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
