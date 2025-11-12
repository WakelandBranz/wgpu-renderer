use std::sync::Arc;
use wgpu_renderer::renderer::Renderer;
use winit::application::ApplicationHandler;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowAttributes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    let mut app = RenderApp::new();

    event_loop.run_app(&mut app)?;

    println!("ALL GOOD");
    Ok(())
}

struct RenderApp {
    renderer: Option<Renderer>,
    window: Option<Arc<winit::window::Window>>,
}

impl RenderApp {
    fn new() -> Self {
        Self {
            renderer: None,
            window: None,
        }
    }
}

impl ApplicationHandler for RenderApp {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        WindowAttributes::default()
                            .with_title("WGPU Renderer Test")
                            .with_inner_size(winit::dpi::PhysicalSize {
                                width: 800,
                                height: 600,
                            }),
                    )
                    .expect("Failed to create window"),
            );

            let size = window.inner_size();
            let renderer = pollster::block_on(Renderer::new(window.clone(), size));

            self.window = Some(window);
            self.renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(renderer) = &mut self.renderer {
            // Queue some text to render
            renderer.queue_text(
                "Hello, WGPU!",
                (100.0, 100.0),
                32.0,
                [1.0, 1.0, 1.0, 1.0],
            );

            // Render the text
            let _ = renderer.render_text();
        }
    }
}