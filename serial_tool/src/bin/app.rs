use serial_tool::main_window::ConnectionWindow;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MyApp",
        native_options,
        Box::new(|cc| Ok(Box::new(ConnectionWindow::new(cc)))),
    )
}
