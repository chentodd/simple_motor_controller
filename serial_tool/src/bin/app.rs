use serial_tool::main_window::*;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MyApp",
        native_options,
        Box::new(|cc| Ok(Box::new(MainWindow::new(cc)))),
    )
}
