// src/gpu_backend.rs
use eframe;

pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions { // eframe = "0.31.1"
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true),
            renderer: eframe::Renderer::Wgpu,
            hardware_acceleration: eframe::HardwareAcceleration::Preferred,
            vsync: true,
            multisampling: 0,
            depth_buffer: 0,
            stencil_buffer: 0,
            run_and_return: true,
            ..Default::default()
    }
}