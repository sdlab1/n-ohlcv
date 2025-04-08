// src/debug_red.rs
use std::sync::Arc;

use egui_wgpu::wgpu;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

struct App<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
}

impl<'a> App<'a> {
    fn new(window: &'a Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        println!("Using adapter: {}", adapter.get_info().name);

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ))
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        println!("Surface capabilities: {:#?}", surface_caps);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        println!("Selected surface format: {:?}", surface_format);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        Self {
            surface,
            device,
            queue,
            surface_config,
        }
    }

    fn redraw(&mut self, _window: &Window) {
        println!("Attempting to get surface texture...");
        let output = match self.surface.get_current_texture() {
            Ok(frame) => {
                println!("Surface texture acquired");
                frame
            }
            Err(wgpu::SurfaceError::Lost) => {
                println!("Surface lost, reconfiguring...");
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("Surface out of memory!"),
            Err(e) => {
                eprintln!("Dropped frame: {:?}", e);
                return;
            }
        };

        println!("Creating texture view...");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("debug encoder"),
            });

        {
            println!("Starting render pass...");
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("debug render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.0,
                            b: 0.5,
                            a: 1.0,
                        }),//debug red+blue=violet screen so you can see it works
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        println!("Submitting encoder...");
        self.queue.submit(std::iter::once(encoder.finish()));
        println!("Presenting output...");
        output.present();

        println!("Frame rendered");
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }
}

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("n-ohlcv [Rust+wgpu]")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0))
            .with_transparent(false)
            .build(&event_loop)
            .unwrap(),
    );

    let mut app = App::new(&window);
    window.request_redraw();

    let window_clone = window.clone();
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                window_id,
                event,
            } if window_id == window_clone.id() => match event {
                WindowEvent::CloseRequested => {
                    println!("Close requested, exiting...");
                    elwt.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    app.resize(physical_size);
                    window_clone.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    println!("Redraw requested event triggered");
                    app.redraw(&window_clone);
                }
                _ => {}
            },
            Event::AboutToWait => {
                window_clone.request_redraw();
                elwt.set_control_flow(ControlFlow::Poll);
            },
            Event::LoopExiting => {
                println!("Exiting event loop.");
            },
            _ => {}
        })
        .unwrap();
}