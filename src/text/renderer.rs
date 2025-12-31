use std::f32;

use glyphon::{
    Attrs, Cache, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextAtlas, Viewport,
};
use wgpu::MultisampleState;

use crate::{
    text::{
        error::TextError,
        types::{BufferRef, CachedTextEntry, FontHandle, QueuedText, TextHandle},
    },
    RenderError,
};

pub(crate) struct TextRenderer {
    // Glyphon infrastructure
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    cache: glyphon::Cache, // Useful if I need multiple viewports
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,

    // Text storage
    cached_buffers: Vec<CachedTextEntry>, // Persists across many frames
    immediate_buffers: Vec<glyphon::Buffer>, // Cleared each frame
    queued_renders: Vec<QueuedText>,      // Cleared each frame

    default_font: FontHandle,
}

impl TextRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        swapchain_format: wgpu::TextureFormat,
        color_mode: glyphon::ColorMode,
    ) -> Result<Self, RenderError> {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(&device);
        let viewport = Viewport::new(device, &cache);
        tracing::debug!("color_mode: {:?}", color_mode);
        tracing::debug!("swapchain_format: {:?}", swapchain_format);
        let mut atlas =
            TextAtlas::with_color_mode(device, queue, &cache, swapchain_format, color_mode);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        // Load system fonts and set a default
        let default_font = font_system
            .db()
            .faces()
            .next()
            .map(FontHandle::from)
            .ok_or_else(|| RenderError::TextError(TextError::FontNotFound))?;

        tracing::debug!(
            "Loaded default system font: {}",
            default_font.display_name()
        );

        Ok(TextRenderer {
            font_system,
            swash_cache,
            cache,
            viewport,
            atlas,
            renderer,

            cached_buffers: Vec::new(),
            immediate_buffers: Vec::new(),
            queued_renders: Vec::new(),

            default_font,
        })
    }

    pub(crate) fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
        let resolution = Resolution { width, height };
        self.viewport.update(queue, resolution);
        tracing::trace!("resized text renderer")
    }

    /// Loads a .ttf file from a relative path
    pub(crate) fn load_ttf_font_from_path(
        &mut self,
        path: &std::path::Path,
    ) -> Result<FontHandle, TextError> {
        self.font_system
            .db_mut()
            .load_font_file(path)
            .map_err(TextError::FontLoad)?;

        let font: FontHandle = self
            .font_system
            .db()
            .faces()
            .last()
            .ok_or(TextError::FontNotFound)?
            .into();
        Ok(font)
    }

    /// Loads a .ttf file from loaded bytes
    pub(crate) fn load_ttf_font_from_bytes(
        &mut self,
        bytes: Vec<u8>,
    ) -> Result<FontHandle, TextError> {
        self.font_system.db_mut().load_font_data(bytes);
        let font: FontHandle = self
            .font_system
            .db()
            .faces()
            .last()
            .ok_or(TextError::FontNotFound)?
            .into();
        Ok(font)
    }

    pub(crate) fn set_default_font(&mut self, font: FontHandle) {
        self.default_font = font
    }

    // TODO: Implement line height enum
    pub(crate) fn create_buffer(
        &mut self,
        text: &str,
        size: f32,
        font: Option<&FontHandle>,
    ) -> glyphon::Buffer {
        let family = if let Some(family) = font {
            Family::Name(&family.family_name)
        } else {
            Family::Name(&self.default_font.family_name)
        };
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, Metrics::new(size, size * 1.2));
        buffer.set_size(
            &mut self.font_system,
            Some(f32::INFINITY),
            Some(f32::INFINITY),
        );
        buffer.set_text(
            &mut self.font_system,
            text,
            &Attrs::new().family(family),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    /// Immediate mode
    /// Best for rendering text that does not persist and updates constantly
    pub(crate) fn queue_text(
        &mut self,
        text: &str,
        pos: glam::Vec2,
        size: f32,
        color: [f32; 4],
        scale: Option<f32>,
    ) {
        let buffer = self.create_buffer(text, size, None);
        let index = self.immediate_buffers.len();
        self.immediate_buffers.push(buffer);
        self.queued_renders.push(QueuedText {
            buffer_ref: BufferRef::Immediate(index),
            position: pos,
            color: convert_color(color),
            scale: scale.unwrap_or(1.0),
            bounds: None, // CHANGE IN THE FUTURE!?!?!?!
        });
    }

    /// Immediate mode
    /// Best for rendering text that does not persist and updates constantly
    pub(crate) fn queue_text_ex(
        &mut self,
        text: &str,
        pos: glam::Vec2,
        size: f32,
        color: [f32; 4],
        scale: Option<f32>,
        font: Option<&FontHandle>,
    ) {
        let buffer = self.create_buffer(text, size, font);
        let index = self.immediate_buffers.len();
        self.immediate_buffers.push(buffer);
        self.queued_renders.push(QueuedText {
            buffer_ref: BufferRef::Immediate(index),
            position: pos,
            color: convert_color(color),
            scale: scale.unwrap_or(1.0),
            bounds: None, // CHANGE IN THE FUTURE!?!?!?!
        });
    }

    /// Cached mode
    /// Create text to cache and render later
    /// MUST EXPLICITLY QUEUE THE PROVIDED HANDLE TO RENDER!
    pub(crate) fn create_cached_text(&mut self, text: &str, size: f32) -> TextHandle {
        let buffer = self.create_buffer(text, size, None);
        let index = self.cached_buffers.len();
        self.cached_buffers.push(CachedTextEntry { buffer, size });
        TextHandle(index)
    }

    /// Cached mode
    /// Create text to cache and render later
    /// MUST EXPLICITLY QUEUE THE PROVIDED HANDLE TO RENDER!
    pub(crate) fn create_cached_text_ex(
        &mut self,
        text: &str,
        size: f32,
        font: Option<&FontHandle>,
    ) -> TextHandle {
        let buffer = self.create_buffer(text, size, font);
        let index = self.cached_buffers.len();
        self.cached_buffers.push(CachedTextEntry { buffer, size });
        TextHandle(index)
    }

    /// Cached mode
    /// Best for rendering text that is mostly static (UI text)
    pub(crate) fn queue_cached_text(
        &mut self,
        text_handle: TextHandle,
        pos: glam::Vec2,
        color: [f32; 4],
        scale: f32,
    ) {
        self.queued_renders.push(QueuedText {
            buffer_ref: BufferRef::Cached(text_handle),
            position: pos,
            color: convert_color(color),
            scale,
            bounds: None,
        })
    }

    /// Modify cached text to re-render
    pub(crate) fn update_cached_text(
        &mut self,
        text_handle: TextHandle,
        new_text: &str,
        new_size: Option<f32>,
        new_font: Option<&FontHandle>,
    ) {
        // TODO: Rewrite this function to take in advanced attributes
        if let Some(entry) = self.cached_buffers.get_mut(text_handle.0) {
            if let Some(size) = new_size {
                entry.buffer.set_metrics_and_size(
                    &mut self.font_system,
                    Metrics::new(size, size * 1.2),
                    None,
                    None,
                );
            }

            let family = if let Some(family) = new_font {
                Family::Name(&family.family_name)
            } else {
                Family::Name(&self.default_font.family_name)
            };

            entry.buffer.set_text(
                &mut self.font_system,
                new_text,
                &Attrs::new().family(family),
                Shaping::Advanced,
                None,
            );
            entry
                .buffer
                .shape_until_scroll(&mut self.font_system, false);
        }
    }

    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<(), TextError> {
        let text_areas: Vec<glyphon::TextArea> = self
            .queued_renders
            .iter()
            .filter_map(|queued| {
                let buffer = match &queued.buffer_ref {
                    BufferRef::Cached(handle) => {
                        self.cached_buffers.get(handle.0).map(|entry| &entry.buffer)
                    }
                    BufferRef::Immediate(index) => self.immediate_buffers.get(*index),
                }?;

                Some(glyphon::TextArea {
                    buffer,
                    left: queued.position.x,
                    top: queued.position.y,
                    scale: queued.scale,
                    bounds: queued.bounds.unwrap_or(glyphon::TextBounds {
                        left: 0,
                        top: 0,
                        right: i32::MAX,
                        bottom: i32::MAX,
                    }),
                    default_color: queued.color,
                    custom_glyphs: &[],
                })
            })
            .collect();

        self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        )?;

        Ok(())
    }

    pub(crate) fn render<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), TextError> {
        Ok(self.renderer.render(&self.atlas, &self.viewport, pass)?)
    }

    pub(crate) fn clear_frame(&mut self) {
        self.immediate_buffers.clear();
        self.queued_renders.clear();
    }
}

/// Converts to a glyphon::Color struct
fn convert_color(color: [f32; 4]) -> glyphon::Color {
    glyphon::Color::rgba(
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8,
    )
}
