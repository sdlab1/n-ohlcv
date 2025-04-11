use crate::db::Database;
use crate::fetch::KLine;
use reqwest::blocking::Client;
use timeframe::Timeframe;
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::{Duration, Utc};

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
    recent_data: Vec<KLine>,
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
    fn new(db: &Arc<Database>, symbol: &str) -> Self {
        let now = Utc::now().timestamp_millis();
        let start_time = now - Duration::days(7).num_milliseconds();

        // Создаем начальный DataWindow
        let mut data_window = DataWindow {
            bars: Vec::new(),
            visible_range: (0.0, 1.0),
            recent_data: Vec::new(),
            volume_height_ratio: 0.2,
        };

        // Загружаем начальные данные
        if let Err(e) = Timeframe::get_data_window(db, symbol, start_time, now, 15, &mut data_window) {
            eprintln!("Ошибка загрузки начальных данных: {}", e);
        }

        // Возвращаем готовую структуру
        Self {
            db: db.clone(), // Создаем новый Arc с новой ссылкой
            data_window,
            timeframe: 10,
            status_messages: vec![format!("Приложение запущено для {}", symbol)],
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

    fn update_data_window(&mut self) {
        let now = Utc::now().timestamp_millis();
        let start_time = now - Duration::days(7).num_milliseconds();

        if let Err(e) = Timeframe::get_data_window(
            &self.db,
            &self.symbol,
            start_time,
            now,
            self.timeframe,
            &mut self.data_window,
        ) {
            self.log_status(format!("Ошибка обновления данных: {}", e));
        } else {
            self.log_status(format!("Обновлено отображение: {} баров", self.data_window.bars.len()));
        }
    }

    fn log_status(&mut self, message: String) {
        self.status_messages.push(message);
        if self.status_messages.len() > 100 {
            self.status_messages.remove(0);
        }
    }
}

fn main() -> eframe::Result<()> {
    // Создаем Arc с базой данных
    let db = Arc::new(Database::new("ohlcv_db").expect("Ошибка инициализации БД"));

    // Создаем экземпляр TradingApp с владеемым Arc
    let app = Arc::new(Mutex::new(TradingApp::new(&db, "BTCUSDT")));

    // Создаем клон для потока
    let app_clone = app.clone();
        // Clone the Arc<Database> specifically for the background thread
        let db_thread_clone = db.clone();
    // Создаем новый поток
    thread::spawn(move || {
        let client = Client::new();
        let mut app = app_clone.lock().unwrap(); // Блокируем мьютекс
            if let Err(e) = Timeframe::update_loop(&client, &db_thread_clone, "BTCUSDT", &mut app.data_window) {
            eprintln!("Ошибка в цикле обновления: {}", e);
        }
    });

    // Запускаем приложение
    eframe::run_native(
        "BTC/USDT OHLCV",
        gpu_backend::native_options(),
        Box::new(move |_| {
            // Клонируем db для использования в лямбда-функции
            let db_clone = db.clone();
            Box::new(TradingApp::new(&db_clone, "BTCUSDT"))
        }),
    )
}