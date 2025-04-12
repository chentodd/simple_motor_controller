use serial_tool::view::main_window::*;

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MyApp",
        native_options,
        Box::new(|cc| Ok(Box::new(MainWindow::new(cc)))),
    )
}
