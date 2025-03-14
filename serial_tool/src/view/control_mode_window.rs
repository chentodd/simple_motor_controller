use crate::{proto::motor_::Operation, UiView, ViewEvent, ViewRequest, DEFAULT_CONTROL_MODE};
use eframe::egui::{self, ComboBox, Id, ProgressBar, Ui, Widget};

#[derive(Default)]
pub(super) struct ControlModeWindow {
    request: Option<ViewRequest>,
    curr_control_mode: Operation,
    target_control_mode: Operation,
    internal_request: Option<(Operation, String)>,
    mode_switch_progress: Option<f32>,
}

impl ControlModeWindow {
    pub fn new() -> Self {
        Self {
            curr_control_mode: DEFAULT_CONTROL_MODE,
            target_control_mode: DEFAULT_CONTROL_MODE,
            ..Default::default()
        }
    }
}

impl UiView for ControlModeWindow {
    fn show(&mut self, ui: &mut Ui) {
        ui.heading("Control mode setup");
        ComboBox::new("control_modes", "control_modes")
            .selected_text(format!("{}", self.target_control_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.target_control_mode, Operation::IntpPos, "IntpPos");
                ui.selectable_value(&mut self.target_control_mode, Operation::IntpVel, "IntpVel");
            });

        // Target mode is requested by internal functions
        let modal_title = if let Some((mode, title)) = self.internal_request.as_ref() {
            self.target_control_mode = *mode;
            title.to_string()
        } else {
            "Switch control mode".to_string()
        };

        if self.target_control_mode != self.curr_control_mode {
            egui::Modal::new(Id::new(&modal_title)).show(ui.ctx(), |ui| {
                ui.heading(format!("{modal_title}, Are you sure?"));
                egui::Sides::new().show(
                    ui,
                    |_ui| {},
                    |ui| {
                        if ui.button("Yes").clicked() {
                            self.request = Some(ViewRequest::ModeSwitch(self.target_control_mode));
                            self.mode_switch_progress = Some(0.0);
                        }

                        if ui.button("No").clicked() {
                            self.request = Some(ViewRequest::ModeCancel);
                            self.target_control_mode = self.curr_control_mode;

                            self.mode_switch_progress = None;
                            self.internal_request = None;
                        }
                    },
                );
            });
        }

        if let Some(progress) = self.mode_switch_progress {
            egui::Modal::new(Id::new("Switch progress")).show(ui.ctx(), |ui| {
                ui.heading("Switching");

                ProgressBar::new(progress).ui(ui);

                if self.target_control_mode != self.curr_control_mode {
                    self.mode_switch_progress = Some(progress + 0.001);
                    ui.ctx().request_repaint();
                } else {
                    self.mode_switch_progress = None;
                }
            });
        }
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        self.request.take()
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ControlModeUpdate(mode) if mode != Operation::Stop => {
                self.curr_control_mode = mode
            }
            ViewEvent::InternalControlModeRequest(x) => {
                self.internal_request = Some(x);
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        // Fail to switch control mode, reset target_control_mode, Ex: if user fails to
        // switch to velocity mode:
        // 1. target_control_mode = velocity, target
        // 2. curr_control_mode = position, current
        //
        // target_control_mode will be set to curr_control_mode when error occurred,
        // so user can try to switch control mode again again
        self.target_control_mode = self.curr_control_mode;
    }
}
