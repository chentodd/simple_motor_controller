use crate::{UiView, ViewEvent, ViewRequest};
use eframe::egui::{Button, ComboBox, Ui};
use serial_enumerator::get_serial_list;

#[derive(Default)]
pub(super) struct ConnectionWindow {
    selected_port: String,
    curr_flag: bool,
    target_flag: bool,
}

impl ConnectionWindow {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl UiView for ConnectionWindow {
    fn show(&mut self, ui: &mut Ui) {
        let port_names = get_serial_list()
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>();

        ui.heading("Connection setup");
        ui.horizontal_centered(|ui| {
            let curr_selected = &mut self.selected_port.as_str();
            ComboBox::new("ports", "ports")
                .selected_text(*curr_selected)
                .show_ui(ui, |ui| {
                    for port in &port_names {
                        ui.selectable_value(curr_selected, port, port);
                    }
                });
            self.selected_port = curr_selected.to_owned();

            let text_in_button = if self.curr_flag {
                "Stop"
            } else {
                "Start"
            };
            let conn_button = Button::new(text_in_button);

            if ui
                .add_enabled(!self.selected_port.is_empty(), conn_button)
                .clicked()
            {
                self.target_flag = !self.curr_flag;
            }
        });
    }

    fn take_request(&mut self) -> Option<ViewRequest> {
        if self.start {
            Some(ViewRequest::Connection(
                self.start,
                self.selected_port.clone(),
            ))
        } else {
            None
        }
    }

    fn handle_event(&mut self, event: ViewEvent) {
        match event {
            ViewEvent::ConnectionStatusUpdate(x) => self.curr_flag = x,
            _ => (),
        }
    }

    fn reset(&mut self) {
        // Start or stop fails, reset flags, Ex: if user encounters stop failure,
        // 1. target_flag = true, target
        // 2. curr_flag = false, current
        //
        // target_flag will be set to curr_flag when error occurred, so user can 
        // try to start again
        self.target_flag = self.curr_flag
    }
}
