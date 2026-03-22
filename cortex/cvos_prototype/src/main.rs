use bevy_ecs::prelude::*;
use loro::LoroDoc;
use std::sync::{Arc, Mutex};
use winit::{
    event::{Event as WinitEvent, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod deliberation;

// Component: A position in the infinite canvas
#[derive(Component, Debug)]
struct PositionData {
    x: f32,
    y: f32,
}

// Component: Represents a node synced with Loro
#[derive(Component)]
struct LoroSynced {
    id: String,
}

struct AppState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    // ECS World
    world: World,
    // Loro Doc (Arc Mutex for thread safety if needed, though single thread here)
    loro: Arc<Mutex<LoroDoc>>,
}

impl AppState {
    async fn new(window: Arc<winit::window::Window>) -> Self {
        let size = window.inner_size();

        // 1. WGPU Setup
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Safety: Window is Arc, so it lives long enough
        // Removed unnecessary unsafe block per warning
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // 2. ECS Setup
        let mut world = World::new();
        // Spawn 10k entities to test "Vertical Slice" perf requirement
        for i in 0..10_000 {
            world.spawn((
                PositionData {
                    x: i as f32,
                    y: i as f32,
                },
                LoroSynced {
                    id: format!("node_{}", i),
                },
            ));
        }

        // 3. Loro Setup
        let loro = Arc::new(Mutex::new(LoroDoc::new()));

        Self {
            surface: unsafe { std::mem::transmute(surface) }, // Hack for static lifetime in struct constraint, careful in real prod
            device,
            queue,
            config,
            size,
            world,
            loro,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update(&mut self) {
        // ECS System: Update positions based on Loro (Simulated)
        // In real app, we'd apply Loro deltas here
        let mut query = self.world.query::<(&mut PositionData, &LoroSynced)>();
        for (mut pos, _synced) in query.iter_mut(&mut self.world) {
            pos.x += 0.1; // Animation proof
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // Draw calls would go here (using pipeline)
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    // Initialize Async State
    let state = pollster::block_on(AppState::new(window.clone()));
    let mut state = state;

    event_loop
        .run(move |event, elwt| {
            match event {
                WinitEvent::WindowEvent { event, window_id } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                                Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
                WinitEvent::AboutToWait => {
                    // Request redraw
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
