//main.rs
use crate::db::Database;
use crate::fetch::KLine;
use timeframe::Timeframe;
use std::sync::Arc;
use chrono::{Duration, Utc};
use settings::*;

pub mod settings;
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
    show_candles: bool,
    crosshair: crosshair::Crosshair,
}

#[derive(Debug)]
pub struct DataWindow {
    bars: Vec<Bar>,
    visible_range: (i64, i64), // Индексы первого и последнего отображаемого бара
    recent_data: Vec<KLine>,
    timeframe_remainder: Vec<KLine>,
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
    fn new(cc: &eframe::CreationContext<'_>, db: Arc<Database>, symbol: &str) -> Self {
        println!("Создание экземпляра TradingApp...");
        let now = chrono::Utc::now().timestamp_millis();
        let start_time = now - chrono::Duration::days(7).num_milliseconds();

        let mut data_window = DataWindow {
            bars: Vec::new(),
            visible_range: (0, 0),
            recent_data: Vec::new(),
            timeframe_remainder: Vec::new(),
            volume_height_ratio: 0.2,
        };

        // Начальная загрузка данных (может заблокировать первый кадр)
        if let Err(e) = Timeframe::get_data_window(&db, symbol, start_time, now, 15, &mut data_window) {
            eprintln!("Ошибка загрузки начальных данных: {}", e);
        }

        // Настройка стиля (пример)
         let mut style = (*cc.egui_ctx.style()).clone();
         style.visuals.dark_mode = true;
         cc.egui_ctx.set_style(style);

        Self {
           db, // Перемещаем Arc<Database>
           data_window,
           timeframe: 10,
           status_messages: vec![format!("Приложение запущено для {}", symbol)],
           symbol: symbol.to_string(),
           show_candles: true,
           crosshair: crosshair::Crosshair::default(),
           // last_update_time: Instant::now(), // Инициализация таймера
       }
   }


   fn zoom(&mut self, amount: f64) {
    let (mut start_idx, mut end_idx) = self.data_window.visible_range;
    let len = self.data_window.bars.len() as i64;
    if len == 0 || end_idx <= start_idx {
        return;
    }

    let range = end_idx - start_idx;
    let zoom = (range as f64 * ZOOM_SENSITIVITY).max(1.0) as i64; // Минимум 1 бар

    if amount > 0.0 {
        // Zoom in
        start_idx = (start_idx + zoom).min(end_idx - 2);
        end_idx = (end_idx - zoom).max(start_idx + 2).min(len);
    } else {
        // Zoom out
        start_idx = (start_idx - zoom).max(0);
        end_idx = (end_idx + zoom).min(len);
    }

    // Финальная проверка
    start_idx = start_idx.max(0);
    end_idx = end_idx.min(len).max(start_idx + 2); // Минимум 2 бара

    /*println!(
        "Zoom: amount = {}, zoom_step = {}, old_range = ({}, {}), new_range = ({}, {}), bars_len = {}",
        amount, zoom, self.data_window.visible_range.0, self.data_window.visible_range.1, start_idx, end_idx, len
    );*/

    self.data_window.visible_range = (start_idx, end_idx);
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
    println!("Запуск main...");
    // Создаем Arc с базой данных (Arc для DB все еще может быть полезен,
    // т.к. сама DB может быть сложной для Clone)
    let db = Arc::new(Database::new("ohlcv_db").expect("Ошибка инициализации БД"));

    // Опции для окна eframe
    let native_options = gpu_backend::native_options(); // Или NativeOptions::default()

    // Запускаем приложение eframe
    println!("Запуск eframe::run_native...");
    eframe::run_native(
        "BTC/USDT OHLCV (Однопоточный)",
        native_options,
        // Замыкание создает ЕДИНСТВЕННЫЙ экземпляр TradingApp
        Box::new(move |cc| {
            println!("Создание экземпляра TradingApp внутри eframe...");
            // Передаем Arc<Database> в конструктор
            let app = TradingApp::new(cc, db.clone(), "BTCUSDT");
            Box::new(app)
        }),
    )
}