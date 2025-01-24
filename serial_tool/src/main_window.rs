use eframe::{egui, App, CreationContext};
use egui::{Ui, ComboBox};

use serial_enumerator::{get_serial_list, SerialInfo};

pub struct ConnectionWindow {
    serial_ports: Vec<SerialInfo>,
    selected_port: String,
}

impl App for ConnectionWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Connection");
            self.display(ui);
        });
    }
}

impl ConnectionWindow {
    pub fn new(_cc: &CreationContext<'_>) -> Self {
        Self {
            serial_ports: get_serial_list(),
            selected_port: "".to_owned(),
        }
    }

    pub fn display(&mut self, ui: &mut Ui) {
        let curr_selected = &mut self.selected_port.as_str();
        ComboBox::new("ports", "ports")
            .selected_text(*curr_selected)
            .show_ui(ui, |ui| {
                for port in self.serial_ports.iter() {
                    ui.selectable_value(curr_selected, &port.name, &port.name);
                }
            });
        
        self.selected_port = curr_selected.to_owned();
    }
}
