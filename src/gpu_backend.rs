// src/gpu_backend.rs
use eframe;
use eframe::wgpu::Instance;

pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions { // eframe = "0.31.1"
        viewport: egui::ViewportBuilder::default()
        .with_inner_size([1920.0, 1080.0])  // Принудительный размер
        .with_position([0.0, 0.0])          // В верхний левый угол
        .with_decorations(false),           // Скрыть рамки
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
pub async fn log_gpu_api() {
    let instance = Instance::default();
    let adapter = instance
        .request_adapter(&eframe::wgpu::RequestAdapterOptions::default())
        .await
        .expect("Failed to find a suitable GPU adapter!");
    println!("[GPU] Backend: {:?}", adapter.get_info().backend);
}
