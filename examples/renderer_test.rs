use std::sync::Arc;

use wgpu_renderer::renderer::Renderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::EventLoop,
    window::WindowAttributes,
};

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

            window.request_redraw();
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
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    // Render shapes
                    renderer.queue_rectangle(50.0, 50.0, 100.0, 80.0, [1.0, 0.0, 0.0, 1.0]);
                    renderer.queue_square(300.0, 100.0, 60.0, [0.0, 1.0, 0.0, 1.0]);
                    renderer.queue_circle(600.0, 150.0, 40.0, [0.0, 0.0, 1.0, 1.0]);

                    // Queue text
                    renderer.queue_text("Hello, WGPU!", (100.0, 300.0), 32.0, [1.0, 1.0, 1.0, 1.0]);
                    renderer.queue_text("Rectangle | Square | Circle", (350.0, 350.0), 16.0, [1.0, 1.0, 0.0, 1.0]);

                    // Render frame (which includes shapes and text)
                    let _ = renderer.render_frame();
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

}
