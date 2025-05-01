use std::time::Duration;

use tokio::runtime::Builder;
use tuning_tool::view::main_window::*;

fn main() -> eframe::Result {
    env_logger::init();

    let rt = Builder::new_multi_thread().enable_all().build().unwrap();

    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "MyApp",
        native_options,
        Box::new(|cc| Ok(Box::new(TuningTool::new(cc)))),
    )
}
