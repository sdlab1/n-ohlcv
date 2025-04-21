use crate::interactivegui::InteractiveGui;

mod timeframe;
mod datawindow;
mod settings;
mod axes;
mod axes_util;
mod hlcbars;
mod volbars;
mod compress;
mod db;
mod fetch;
mod gpu_backend;
mod gui;
mod interactivegui;
mod crosshair;
mod performance;
mod rsi;

fn main() -> eframe::Result<()> {
    

    // Запускаем приложение eframe
    println!("Running eframe::run_native");
    eframe::run_native(
        "n-ohlc",
        gpu_backend::native_options(),
        Box::new(|cc|{
             Ok(Box::new(InteractiveGui::new(cc, "BTCUSDT", 15)))
        })
    ).unwrap(); 
    Ok(())
}