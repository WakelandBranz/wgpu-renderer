use glyphon::{Cache, FontSystem, Resolution, SwashCache, TextAtlas, Viewport};
use wgpu::MultisampleState;

use crate::RenderError;

const FONT_BYTES: &[u8] = include_bytes!("../../res/fonts/PressStart2P-Regular.ttf");

pub(crate) struct TextRenderer {
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    cache: glyphon::Cache, // Useful if I need multiple viewports
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
    buffers: Vec<glyphon::Buffer>,
}

impl TextRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain_format: wgpu::TextureFormat,
        color_mode: glyphon::ColorMode,
    ) -> Result<Self, RenderError> {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas =
            TextAtlas::with_color_mode(device, queue, &cache, swapchain_format, color_mode);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);
        let mut buffers: Vec<glyphon::Buffer> = Vec::new();
        
        
        font_system.db_mut().load_font_data(FONT_BYTES.to_vec());

        Ok(TextRenderer {
            font_system,
            swash_cache,
            cache,
            viewport,
            atlas,
            renderer,
            buffers,
        })
    }

    pub(crate) fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        let resolution = Resolution { width, height };
        self.viewport.update(queue, resolution);
        println!("resized text renderer")
    }
    
    /// Best for rendering text that updates entire frequently
    pub(crate) fn queue_text(&mut self, ) {
        
    }
    
    /// Best for rendering text that is mostly static (UI text)
    pub(crate) fn queue_cached_text(&mut self, ) {
        
    }
    
    /// Loads a .ttf file from a specific path
    pub(crate) fn load_font_from_path(&mut self, path: &str) -> bool {
        todo!()
    }
}
