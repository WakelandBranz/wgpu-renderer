use wgpu_renderer::renderer::Renderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let sdl_context = sdl3::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("Vulkan Renderer Test", 800, 600)
        .position_centered()
        .vulkan()
        .build()?;

    let window_size = window.size();

    let wgpu_renderer = Renderer::new(&window, window_size.into())?;

    // Keep window open
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        for event in event_pump.poll_iter() {
            if let sdl3::event::Event::Quit { .. } = event {
                break 'running;
            }
        }

        // TODO: Render frame here
        // vulkan_renderer.render_frame()?;

        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    println!("ALL GOOD");
    Ok(())
}
