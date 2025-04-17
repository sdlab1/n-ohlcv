// src/gpu_backend.rs
use eframe;
use egui::Theme;

pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true),
            //.with_inner_size(egui::vec2(1200.0, 800.0))
            //.with_min_inner_size(egui::vec2(300.0, 200.0)),
        // These are the default WGPU settings that eframe uses internally
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        //renderer: eframe::Renderer::Wgpu,
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        //default_theme: Theme::Dark,
        run_and_return: true,
        ..Default::default()
    }
}