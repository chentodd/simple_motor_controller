use std::sync::mpsc::channel;

use serial_tool::main_window::*;
use serial_tool::profile_measurement::ProfileData;

fn main() -> eframe::Result {
    let (_tx, rx) = channel::<ProfileData>();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MyApp",
        native_options,
        Box::new(|cc| Ok(Box::new(MainWindow::new(cc, 4000, rx)))),
    )
}
