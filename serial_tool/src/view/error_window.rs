use crate::{main_window::ErrorType, UiView, ViewEvent, ViewResponse};
use eframe::egui::{self, Id};

#[derive(Default)]
pub struct ErrorWindow {
    error_type: ErrorType,
    error_message: String,
    error_cleared: bool,
}

impl ErrorWindow {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl UiView for ErrorWindow {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if self.error_type == ErrorType::None {
            return;
        }

        egui::Modal::new(Id::new("Error")).show(ui.ctx(), |ui| {
            ui.heading("Error ❌");
            ui.label(&self.error_message);

            egui::Sides::new().show(
                ui,
                |_ui| {},
                |ui| {
                    if ui.button("Ok").clicked() {
                        self.error_cleared = true;
                    }
                },
            );
        });
    }

    fn take_request(&mut self) -> Option<ViewResponse> {
        if self.error_cleared {
            let prev_error_type = self.error_type;

            self.error_type = ErrorType::None;
            self.error_message.clear();
            self.error_cleared = false;
            
            Some(ViewResponse::ErrorDismissed(prev_error_type))
        } else {
            None
        }
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ErrorOccurred(err, msg) => {
                self.error_type = err;
                self.error_message = msg;
            }
            _ => (),
        }
    }
}
