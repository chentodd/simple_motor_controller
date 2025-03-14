use crate::{proto::motor_::Operation, UiView, ViewEvent, ViewRequest, DEFAULT_CONTROL_MODE};
use eframe::egui::{Button, ScrollArea, Slider, TextEdit, Ui};

#[derive(Default)]
pub(super) struct CommandWindow {
    curr_control_mode: Operation,
    velocity_command: f32,
    position_command: String,
    position_command_ready: bool,
}

impl CommandWindow {
    pub fn new() -> Self {
        Self {
            curr_control_mode: DEFAULT_CONTROL_MODE,
            ..Default::default()
        }
    }

    fn display_velocity_command_panel(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.velocity_command, -3000.0..=3000.0)
                .text("motor velocity cmd (rpm)"),
        );
    }

    fn display_position_command_panel(&mut self, ui: &mut Ui) {
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
            self.position_command_ready = true;
        }
    }
}

impl UiView for CommandWindow {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("Command setup");
        match self.curr_control_mode {
            Operation::IntpVel => self.display_velocity_command_panel(ui),
            Operation::IntpPos => self.display_position_command_panel(ui),
            _ => (),
        }
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        match self.curr_control_mode {
            Operation::IntpVel => Some(ViewRequest::VelocityControl(self.velocity_command)),
            Operation::IntpPos => {
                let ready = self.position_command_ready;
                self.position_command_ready = false;
                Some(ViewRequest::PositionControl((
                    self.position_command.clone(),
                    ready,
                )))
            }
            _ => None,
        }
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ControlModeUpdate(mode) if mode != Operation::Stop => {
                self.curr_control_mode = mode
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        self.velocity_command = 0.0;
        self.position_command_ready = false;
    }
}
