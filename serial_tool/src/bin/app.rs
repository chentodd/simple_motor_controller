use eframe::{egui, App};
use egui::ComboBox;

#[derive(Default)]
struct ConnectionWindow {
    port: &'static str,
}

impl ConnectionWindow {
    const PORTS: [&'static str; 2] = ["/dev/ttyUSB0", "/dev/ttyACM0"];
}

impl ConnectionWindow {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            port: Self::PORTS[0],
        }
    }
}

impl App for ConnectionWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { port } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test");
            ComboBox::new("port", "Port")
                .selected_text(*port)
                .show_ui(ui, |ui| {
                    for x in Self::PORTS {
                        ui.selectable_value(port, x, x);
                    }
                });
        });
    }
}

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MyApp",
        native_options,
        Box::new(|cc| Ok(Box::new(ConnectionWindow::new(cc)))),
    )
}
