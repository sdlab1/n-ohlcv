use eframe::{egui, Frame, NativeOptions};
use std::sync::Arc;
use wgpu::{Backends, DeviceDescriptor, Features, Limits, SurfaceError};
use crate::timeframe::Timeframe;
use crate::db::Database;
mod timeframe;
mod fetch;
mod compress;
mod db;

struct App {
    timeframe: Option<Timeframe>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("n-ohlcv");

            if let Some(tf) = &self.timeframe {
                ui.label(format!("Data ready from {} to {}", tf.start_date, tf.end_date));
                ui.label(format!("Total minutes: {}", tf.total_minutes));
                ui.label(format!("Current price: {}", tf.current_price));
            } else {
                ui.label("Preparing data...");
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let client = reqwest::blocking::Client::new();
    let db = Database::new("n-ohlcv-sled-db").expect("Failed to initialize database");

    // Проверка последнего сохраненного времени
    if let Ok(Some(last_time)) = db.get_last_time() {
        println!("Resuming from last saved time: {}", last_time);
    }
    // Загружаем данные
    let timeframe = match Timeframe::fetch_and_store(&client, &db) {
        Ok(tf) => {
            println!("Data preparation completed successfully");
            Some(tf)
        }
        Err(e) => {
            eprintln!("Data preparation failed: {}", e);
            None
        }
    };

    let options = NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            device_descriptor: Arc::new(|_adapter| DeviceDescriptor {
                label: Some("egui wgpu device"),
                required_features: Features::empty(),
                required_limits: Limits::downlevel_webgl2_defaults(),
            }),
            supported_backends: Backends::all(),
            power_preference: wgpu::PowerPreference::HighPerformance,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: Some(2),
            on_surface_error: Arc::new(|err| {
                eprintln!("WGPU surface error: {:?}", err);
                match err {
                    SurfaceError::Timeout => eframe::egui_wgpu::SurfaceErrorAction::SkipFrame,
                    SurfaceError::Outdated | SurfaceError::Lost => eframe::egui_wgpu::SurfaceErrorAction::RecreateSurface,
                    SurfaceError::OutOfMemory => {
                        eprintln!("WGPU out of memory! Exiting.");
                        std::process::exit(-1);
                    }
                }
            }),
            ..Default::default()
        },
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([300.0, 200.0]),
        ..Default::default()
    };

    eframe::run_native(
        "n-ohlcv [Rust+egui]",
        options,
        Box::new(|_cc| Box::new(App { timeframe })),
    )
}