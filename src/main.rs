// main.rs
use eframe::{Frame, egui};
use reqwest::blocking::Client;
use timeframe::Timeframe;
use std::sync::Arc;
use std::thread;
use chrono;

mod axes;
mod hlcbars;
mod volbars;
mod compress;
mod db;
mod fetch;
mod timeframe;
mod gpu_backend;

struct TradingApp {
    db: Arc<db::Database>,
    data_window: DataWindow,
    timeframe: i32,
    status_messages: Vec<String>,
}

#[derive(Debug)]
pub struct DataWindow {
    bars: Vec<Bar>,
    visible_range: (f64, f64),
    volume_height_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct Bar {
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

impl TradingApp {
// In main.rs
fn new(db: Arc<db::Database>, cc: &eframe::CreationContext<'_>) -> Self {
    let now = chrono::Utc::now().timestamp_millis();
    let start_time = now - chrono::Duration::days(7).num_milliseconds();
    
    let data_window = Timeframe::get_data_window(
        &db,
        "BTCUSDT",
        start_time,
        now,
        5 // default 5-minute timeframe
    ).unwrap_or_else(|e| {
        eprintln!("Initial data load failed: {}", e);
        DataWindow {
            bars: Vec::new(),
            visible_range: (0.0, 1.0),
            volume_height_ratio: 0.2,
        }
    });

    Self {
        db,
        data_window,
        timeframe: 5,
        status_messages: vec!["Application started".to_string()],
    }
}

    fn log_status(&mut self, message: String) {
        self.status_messages.push(message);
        if self.status_messages.len() > 100 {
            self.status_messages.remove(0);
        }
    }

    fn update_data_window(&mut self) {
        let now = chrono::Utc::now().timestamp_millis();
        let start_time = now - chrono::Duration::days(7).num_milliseconds();
        
        match Timeframe::get_data_window(
            &self.db,
            "BTCUSDT",
            start_time,
            now,
            self.timeframe
        ) {
            Ok(data_window) => {
                self.data_window = data_window;
                self.log_status(format!("Updated view with {} bars", self.data_window.bars.len()));
            },
            Err(e) => {
                self.log_status(format!("Error updating data: {}", e));
            }
        }
    }
}

impl eframe::App for TradingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("BTC/USDT OHLCV Viewer");

            // Timeframe selection
            ui.horizontal(|ui| {
                for &tf in &[1, 2, 3, 5, 10, 15, 30, 60, 240, 1440] {
                    if ui.button(format!("{}m", tf)).clicked() {
                        self.timeframe = tf;
                        self.update_data_window();
                    }
                }
            });

            // Main chart area
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::drag(),
                );

                // Draw axes
                axes::draw(ui, rect, &self.data_window);

                // Draw HL bars
                hlcbars::draw(ui, rect, &self.data_window);

                // Draw volume bars
                volbars::draw(ui, rect, &self.data_window);
            });

            // Status messages
            egui::ScrollArea::vertical().show(ui, |ui| {
                for msg in &self.status_messages {
                    ui.label(msg);
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let db = Arc::new(db::Database::new("ohlcv_db").expect("DB init failed"));
    
    // Start data updater thread
    let db_clone = db.clone();
    thread::spawn(move || {
        let client = Client::new();
        timeframe::update_loop(&client, &db_clone, "BTCUSDT");
    });

    eframe::run_native(
        "BTC/USDT OHLCV",
        gpu_backend::native_options(),
        Box::new(|cc| Box::new(TradingApp::new(db, cc))),
    )
}