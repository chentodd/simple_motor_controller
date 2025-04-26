use crate::{ErrorType, UiView, ViewEvent, ViewRequest};
use eframe::egui::{self, Id};

#[derive(Default)]
pub(super) struct ErrorWindow {
    error_type: ErrorType,
    error_message: String,
    request: Option<ViewRequest>,
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
                        self.request = Some(ViewRequest::ErrorDismiss(self.error_type));
                        self.error_type = ErrorType::None;
                        self.error_message.clear();
                    }
                },
            );
        });
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        self.request.take()
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

    fn reset(&mut self) {
        // Do nothing
    }
}
