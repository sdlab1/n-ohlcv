// main.rs - Application entry point, initializes eframe with InteractiveGui
// See CONVENTIONS.md for project structure and workflow

use crate::interactivegui::InteractiveGui;

mod axes;
mod axes_util;
mod compress;
mod crosshair;
mod datawindow;
mod db;
mod drawing_util;
mod fetch;
mod gpu_backend;
mod gui;
mod hlcbars;
mod interactivegui;
mod performance;
mod rsi;
mod settings;
mod timeframe;
mod volbars;

fn main() -> eframe::Result<()> {
    // Запускаем приложение eframe
    println!("Running eframe::run_native");
    eframe::run_native(
        "n-ohlc",
        gpu_backend::native_options(),
        Box::new(|cc| Ok(Box::new(InteractiveGui::new(cc, "BTCUSDT", 15)))),
    )
    .unwrap();
    Ok(())
}
