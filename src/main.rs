// main.rs
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
mod app_ui;

struct TradingApp {
    db: Arc<db::Database>,
    data_window: DataWindow,
    timeframe: i32,
    status_messages: Vec<String>,
    symbol: String,
    scroll_sensitivity: f32,
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
    fn new(db: Arc<db::Database>, symbol: &str) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let start_time = now - chrono::Duration::days(7).num_milliseconds();

        // Rely on get_data_window to initialize DB if needed
        let data_window = match Timeframe::get_data_window(&db, symbol, start_time, now, 5) {
            Ok(data_window) => data_window,
            Err(e) => {
                eprintln!("Initial data load failed: {}", e);
                DataWindow {
                    bars: Vec::new(),
                    visible_range: (0.0, 1.0),
                    volume_height_ratio: 0.2,
                }
            }
        };

        Self {
            db,
            data_window,
            timeframe: 5,
            status_messages: vec![format!("Application started for {}", symbol)],
            symbol: symbol.to_string(),
            scroll_sensitivity: 0.2,
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
        
        match Timeframe::get_data_window(&self.db, &self.symbol, start_time, now, self.timeframe) {
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

fn main() -> eframe::Result<()> {
    let db = Arc::new(db::Database::new("ohlcv_db").expect("DB init failed"));
    let db_clone = db.clone();

    thread::spawn(move || {
        let client = Client::new();
        timeframe::Timeframe::update_loop(&client, &db_clone, "BTCUSDT");
    });

    eframe::run_native(
        "BTC/USDT OHLCV",
        gpu_backend::native_options(),
        Box::new(|_| Box::new(TradingApp::new(db, "BTCUSDT"))),
    )
}