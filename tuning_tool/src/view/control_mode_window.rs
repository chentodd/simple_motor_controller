use eframe::egui::{self, ComboBox, Id, ProgressBar, Ui, Widget};
use log::debug;

use crate::{DEFAULT_CONTROL_MODE, UiView, ViewEvent, ViewRequest};
use protocol::ControlMode;

#[derive(Default)]
pub(super) struct ControlModeWindow {
    request: Option<ViewRequest>,
    target_control_mode: ControlMode,
    curr_control_mode: ControlMode,
    backup_control_mode: ControlMode,
    internal_request: Option<String>,
    wait_mode_switch: bool,
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
                ui.selectable_value(&mut self.target_control_mode, ControlMode::Position, "Position");
                ui.selectable_value(&mut self.target_control_mode, ControlMode::Velocity, "IntpVel");
            });

        // Check if we need to do mode switch
        let modal_title = if let Some(title) = self.internal_request.take() {
            self.target_control_mode = ControlMode::Stop;
            title.to_string()
        } else {
            "Switch control mode".to_string()
        };

        if self.target_control_mode != self.curr_control_mode {
            if !self.wait_mode_switch {
                self.backup_control_mode = self.curr_control_mode;
                self.wait_mode_switch = true;
            }
        } else {
            self.wait_mode_switch = false;
        }

        if self.wait_mode_switch {
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
                            self.target_control_mode = self.backup_control_mode;
                            self.wait_mode_switch = false;
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

                if self.wait_mode_switch {
                    self.mode_switch_progress = Some(progress + 0.001);
                    ui.ctx().request_repaint();
                } else {
                    self.mode_switch_progress = None;
                }
            });
        }
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        if self.request.is_some() {
            debug!("{:?}", self.request);
        }
        self.request.take()
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ControlModeUpdate((ok, mode)) => {
                if ok {
                    self.curr_control_mode = mode;
                }
            }
            ViewEvent::InternalStopModeRequest(x) => {
                self.internal_request = Some(x);
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        // Fail to switch control mode, reset target_control_mode, Ex: if user fails to
        // switch to velocity mode:
        // 1. target_control_mode = velocity, target
        // 2. backup_control_mode = position, current
        //
        // target_control_mode will be set to backup_control_mode when error occurred,
        // so user can try to switch control mode again again
        self.target_control_mode = self.backup_control_mode;
    }
}
