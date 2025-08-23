use crate::crosshair;
use crate::datawindow::DataWindow;
use crate::db::Database;
use crate::gpu_backend;
use crate::performance::FrameInfo;
use crate::settings::*;
use chrono::{Duration, Utc};
use std::time::Instant;

pub struct InteractiveGui {
    db: Database,
    pub data_window: DataWindow,
    pub timeframe: i32,
    pub status_messages: Vec<String>,
    pub status_messages_last_ts: Option<Instant>,
    pub symbol: String,
    pub show_candles: bool,
    pub measure_frame_time: bool,
    pub crosshair: crosshair::Crosshair,
    pub frame_info: FrameInfo,
}

impl InteractiveGui {
    pub fn new(cc: &eframe::CreationContext<'_>, symbol: &str, timeframe: i32) -> Self {
        println!("Creating InteractiveGui object");

        let future = gpu_backend::log_gpu_api();
        pollster::block_on(future);
        /*if let Some(_render_state) = &cc.wgpu_render_state {
        // just to know where it's at
        }*/
        // dark theme
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.dark_mode = true;
        cc.egui_ctx.set_style(style);

        let mut data_window = DataWindow {
            bars: Vec::new(),
            visible_range: (0, 0),
            price: (0.0, 0.0),
            recent_data: Vec::new(),
            timeframe_remainder: Vec::new(),
            volume_height_ratio: 0.2,
            pixel_offset: 0.0,
            min_indexes: None,
            max_indexes: None,
            cached_visible_range: None,
            cached_max_volume: None,
        };
        let now = chrono::Utc::now().timestamp_millis();
        let start_time = now - chrono::Duration::days(INITIAL_LOAD_DAYS).num_milliseconds();
        let db = Database::new("ohlcv_db").expect("Error initializing DB");
        // loading initial data window
        if let Err(e) =
            DataWindow::get_data_window(&db, symbol, start_time, now, timeframe, &mut data_window)
        {
            eprintln!("Unable to get data window: {}", e);
        }
        Self {
            db,
            data_window,
            timeframe,
            status_messages: Vec::new(),
            status_messages_last_ts: None,
            symbol: symbol.to_string(),
            show_candles: true,
            measure_frame_time: false,
            crosshair: crosshair::Crosshair::default(),
            frame_info: FrameInfo::default(),
        }
    }
    fn message_add(&mut self, new_message: String) {
        self.status_messages.push(new_message);
        self.status_messages_last_ts = Some(Instant::now());
        if self.status_messages.len() > STATUS_MESSAGE_MAX_COUNT {
            self.status_messages.remove(0);
        }
    }

    pub fn zoom(&mut self, amount: f64) {
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

        self.data_window.visible_range = (start_idx, end_idx);
    }

    pub fn update_data_window(&mut self) {
        let now = Utc::now().timestamp_millis();
        let start_time = now - Duration::days(INITIAL_LOAD_DAYS).num_milliseconds();
        use DataWindow;
        if let Err(e) = DataWindow::get_data_window(
            &self.db,
            &self.symbol,
            start_time,
            now,
            self.timeframe,
            &mut self.data_window,
        ) {
            self.message_add(format!("Ошибка обновления данных: {}", e));
        } else {
            self.message_add(format!(
                "Обновлено отображение: {} баров",
                self.data_window.bars.len()
            ));
        }
    }
}
