use eframe::egui::{Button, ScrollArea, Slider, TextEdit, Ui};

use crate::{DEFAULT_CONTROL_MODE, UiView, ViewEvent, ViewRequest};
use protocol::ControlMode;

#[derive(Default)]
pub(super) struct CommandWindow {
    curr_control_mode: ControlMode,
    request: Option<ViewRequest>,
    // velocity command, unit: rpm
    curr_vel_cmd: f32,
    prev_vel_cmd: f32,
    // position command format: '(dist, vel, vel_end);'
    // Input data should be enclosed by parenthesis, and use ';' to indicate the
    // end of one command block, and the unit of each data is as follows:
    // 1. dist: rad
    // 2. vel: rpm
    // 3. vel_end: rpm, the end velocity of position command block, it is optional.
    //    If it is not given, the end velocity will be treated as 0
    pos_cmd: String,
}

impl CommandWindow {
    pub fn new() -> Self {
        Self {
            curr_control_mode: DEFAULT_CONTROL_MODE,
            ..Default::default()
        }
    }

    fn display_position_command_panel(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().max_height(64.0).show(ui, |ui| {
            ui.add_sized(ui.available_size(), TextEdit::multiline(&mut self.pos_cmd));
        });

        let send_button = Button::new("Send");
        if ui
            .add_enabled(!self.pos_cmd.is_empty(), send_button)
            .clicked()
        {
            self.request = Some(ViewRequest::PositionControl(self.pos_cmd.clone()));
        }
    }

    fn display_velocity_command_panel(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.curr_vel_cmd, -3000.0..=3000.0).text("motor velocity cmd (rpm)"),
        );

        if self.curr_vel_cmd != self.prev_vel_cmd {
            self.prev_vel_cmd = self.curr_vel_cmd;
            self.request = Some(ViewRequest::VelocityControl(self.curr_vel_cmd));
        }
    }
}

impl UiView for CommandWindow {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("Command setup");
        match self.curr_control_mode {
            ControlMode::Position => self.display_position_command_panel(ui),
            ControlMode::Velocity => self.display_velocity_command_panel(ui),
            _ => (),
        }
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        self.request.take()
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ControlModeUpdate((ok, mode)) => {
                if ok {
                    self.curr_control_mode = mode;
                    if self.curr_control_mode == ControlMode::Stop {
                        self.reset();
                    }
                }
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        self.request = None;
        self.curr_vel_cmd = 0.0;
        self.prev_vel_cmd = 0.0;
        self.pos_cmd.clear();
    }
}
