// src/gpu_backend.rs
use eframe::egui;

pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::vec2(800.0, 600.0))
            .with_min_inner_size(egui::vec2(300.0, 200.0)),
        // These are the default WGPU settings that eframe uses internally
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        renderer: eframe::Renderer::Wgpu,
        follow_system_theme: false,
        default_theme: eframe::Theme::Dark,
        run_and_return: true,
        ..Default::default()
    }
}