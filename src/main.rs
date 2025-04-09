use eframe::{Frame, egui};
use reqwest::blocking::Client;
use std::sync::Arc;
use std::thread;

mod compress;
mod db;
mod fetch;
mod timeframe;
mod gpu_backend;

struct TradingApp {
    db: Arc<db::Database>,
    symbols: Vec<String>,
    selected_symbol: String,
    status_messages: Vec<String>,
}

impl TradingApp {
    fn new(db: Arc<db::Database>) -> Self {
        let symbols = vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "BNBUSDT".to_string(),
        ];
        
        Self {
            db,
            symbols: symbols.clone(),
            selected_symbol: symbols[0].clone(),
            status_messages: Vec::new(),
        }
    }

    fn log_status(&mut self, message: String) {
        self.status_messages.push(message);
        if self.status_messages.len() > 100 {
            self.status_messages.remove(0);
        }
    }
}

impl eframe::App for TradingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("n-ohlcv Multi-Ticker Monitor");

            egui::ComboBox::from_label("Select Ticker")
                .selected_text(&self.selected_symbol)
                .show_ui(ui, |ui| {
                    for symbol in &self.symbols {
                        ui.selectable_value(&mut self.selected_symbol, symbol.clone(), symbol);
                    }
                });

            if let Ok(last_block) = self.db.get_last_block(&self.selected_symbol) {
                ui.label(format!(
                    "Last block for {}: {}",
                    self.selected_symbol,
                    chrono::DateTime::from_timestamp_millis(last_block)
                        .unwrap()
                        .format("%Y-%m-%d %H:%M:%S")
                ));
            }

            if ui.button("Force Update").clicked() {
                let client = Client::new();
                let db = Arc::clone(&self.db);
                let symbol_clone = self.selected_symbol.clone();
                
                thread::spawn(move || {
                    if let Err(e) = timeframe::Timeframe::fetch_and_process(&client, &db, &symbol_clone) {
                        eprintln!("Error: {}", e);
                    }
                });
                
                self.log_status(format!("Manual update triggered for {}", self.selected_symbol));
            }

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

    let symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];
    let client = Client::new();

    for symbol in symbols {
        let client = client.clone();
        let db = Arc::clone(&db);
        
        thread::spawn(move || {
            if let Err(e) = timeframe::Timeframe::run_forever(&client, &db, symbol) {
                eprintln!("Processor for {} crashed: {}", symbol, e);
            }
        });
    }

    eframe::run_native(
        "n-ohlcv",
        gpu_backend::native_options(),
        Box::new(|_cc| Box::new(TradingApp::new(db))),
    )
}