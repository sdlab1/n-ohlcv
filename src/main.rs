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
mod crosshair;

struct TradingApp {
    db: Arc<db::Database>,
    data_window: DataWindow,
    timeframe: i32,
    status_messages: Vec<String>,
    symbol: String,
    zoom_sensitivity: f64,
    show_candles: bool,
    crosshair: crosshair::Crosshair,
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

        let data_window = match Timeframe::get_data_window(&db, symbol, start_time, now, 15) {
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
            timeframe: 10,
            status_messages: vec![format!("Application started for {}", symbol)],
            symbol: symbol.to_string(),
            zoom_sensitivity: 0.01,
            show_candles: true,
            crosshair: crosshair::Crosshair::default(),
        }
    }

    fn zoom(&mut self, amount: f64) {
        let zoom_factor = self.zoom_sensitivity;
        let (current_start, current_end) = self.data_window.visible_range;
        let range_width = current_end - current_start;
        
        let new_width = if amount > 0.0 {
            range_width * (1.0 - zoom_factor).max(0.01)
        } else {
            range_width * (1.0 + zoom_factor).max(0.01)
        };
        
        let center = (current_start + current_end) / 2.0;
        let new_start = (center - new_width / 2.0).max(0.0);
        let new_end = (new_start + new_width).min(1.0);
        
        //self.data_window.visible_range = (new_start, new_end);
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