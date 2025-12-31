use std::{iter, sync::Arc};

use winit::{dpi::PhysicalSize, window::Window};

use crate::RenderError;
use crate::init::*;
use crate::text::renderer::*;
use crate::text::types::FontHandle;
use crate::text::types::TextHandle;
use crate::types::*;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    text_renderer: TextRenderer,
    queued_vertices: Vec<Vertex>,
    queued_indices: Vec<u32>,
    // Passed into shaders
    screen_size_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Renderer {
    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    pub fn height(&self) -> f32 {
        self.config.height as f32
    }

    pub async fn new(
        window: Arc<Window>,
        size: PhysicalSize<u32>,
    ) -> Result<Renderer, RenderError> {
        tracing::debug!("size: {:?}", size);

        // Create core wgpu components
        let instance = create_instance();
        let surface = create_surface(&instance, window)?;
        let adapter =
            create_adapter(&instance, wgpu::PowerPreference::HighPerformance, &surface).await?;
        let (device, queue) = create_device_and_queue(&adapter).await?;

        let config = create_surface_config(&surface, &adapter, size)?;

        let bind_group_layout = create_bind_group_layout(&device);
        let pipeline_layout = create_pipeline_layout(&device, &bind_group_layout);

        let (vert_shader, frag_shader) = create_shader_modules(&device);

        let screen_size_buffer = create_screen_size_buffer(&device, size);
        let (vertex_buffer, index_buffer) = create_vertex_and_index_buffers(&device);

        let bind_group = create_bind_group(&device, &bind_group_layout, &screen_size_buffer);

        let pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            &[Vertex::DESC],
            vert_shader,
            frag_shader,
        );

        surface.configure(&device, &config);

        let text_renderer = create_text_renderer(&device, &queue, config.format)?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            text_renderer,
            queued_vertices: Vec::new(),
            queued_indices: Vec::new(),
            screen_size_buffer,
            bind_group,
        })
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        // Clamp to device's max 2d texture size
        let max_texture_size = self.device.limits().max_texture_dimension_2d;
        self.config.width = size.width.min(max_texture_size);
        self.config.height = size.height.min(max_texture_size);

        self.queue.write_buffer(
            &self.screen_size_buffer,
            0,
            bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
        );
        self.surface.configure(&self.device, &self.config);
        self.text_renderer.resize(&self.queue, self.config.width, self.config.height);
    }

    // === TEXT RENDERING ===

    pub fn queue_text(&mut self, text: &str, position: (f32, f32), size: f32, color: [f32; 4]) {
        self.text_renderer.queue_text(text, glam::Vec2::new(position.0, position.1), size, color, None);
    }

    pub fn queue_text_ex(&mut self, text: &str, position: (f32, f32), size: f32, color: [f32; 4], scale: Option<f32>, font: Option<&FontHandle>,) {
        self.text_renderer.queue_text_ex(text, glam::Vec2::new(position.0, position.1), size, color, scale, font);
    }

    pub fn create_cached_text(&mut self, text: &str, size: f32) -> TextHandle {
        self.text_renderer.create_cached_text(text, size)
    }

    pub fn create_cached_text_ex(&mut self, text: &str, size: f32, font: Option<&FontHandle>) -> TextHandle {
        self.text_renderer.create_cached_text_ex(text, size, font)
    }

    pub fn queue_cached_text(&mut self, handle: TextHandle, position: (f32, f32), color: [f32; 4], scale: f32) {
        self.text_renderer.queue_cached_text(handle, glam::Vec2::new(position.0, position.1), color, scale);
    }

    pub fn update_cached_text(
        &mut self,
        text_handle: TextHandle,
        new_text: &str,
        new_size: Option<f32>,
        new_font: Option<&FontHandle>,
    ) {
        self.text_renderer.update_cached_text(text_handle, new_text, new_size, new_font);
    }

    pub fn load_ttf_font_from_path(&mut self, path: &std::path::Path) -> Result<FontHandle, RenderError> {
        Ok(self.text_renderer.load_ttf_font_from_path(path)?)
    }

    pub fn load_ttf_font_from_bytes(&mut self, bytes: Vec<u8>) -> Result<FontHandle, RenderError> {
        Ok(self.text_renderer.load_ttf_font_from_bytes(bytes)?)
    }
    
    pub fn set_default_font(&mut self, font: FontHandle) {
        self.text_renderer.set_default_font(font)
    }

    // === PRIMITIVE SHAPE RENDERING ===

    pub fn queue_rectangle(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        let vertex_offset = self.queued_vertices.len() as u32;

        self.queued_vertices.extend_from_slice(&[
            Vertex::with_color(x, y, color),
            Vertex::with_color(x + width, y, color),
            Vertex::with_color(x + width, y + height, color),
            Vertex::with_color(x, y + height, color),
        ]);

        self.queued_indices.extend_from_slice(&[
            vertex_offset + 2,
            vertex_offset + 1,
            vertex_offset,
            vertex_offset + 3,
            vertex_offset + 2,
            vertex_offset,
        ]);
    }

    pub fn queue_square(&mut self, x: f32, y: f32, size: f32, color: [f32; 4]) {
        self.queue_rectangle(x, y, size, size, color)
    }

    pub fn queue_circle(&mut self, center_x: f32, center_y: f32, radius: f32, color: [f32; 4]) {
        const SEGMENTS: usize = 32;
        let vertex_offset = self.queued_vertices.len() as u32;

        // Center vertex
        self.queued_vertices
            .push(Vertex::with_color(center_x, center_y, color));

        for i in 0..SEGMENTS {
            let angle = (2.0 * std::f32::consts::PI * (i as f32)) / (SEGMENTS as f32);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            self.queued_vertices.push(Vertex::with_color(x, y, color));
        }

        for i in 0..SEGMENTS {
            let next = if i == SEGMENTS - 1 { 1 } else { i + 2 };
            self.queued_indices.push(vertex_offset + (next as u32));
            self.queued_indices.push(vertex_offset + ((i + 1) as u32));
            self.queued_indices.push(vertex_offset);
        }
    }

    pub fn begin_frame(&mut self) -> Result<(), RenderError> {
        self.surface.get_current_texture()?;
        Ok(())
    }

    pub fn draw_shape(
        &mut self,
        num_indices: u32,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("Shape Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations::default(),
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            }),
        );

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    pub fn render_frame(&mut self) -> Result<(), RenderError> {
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&Default::default());

                // Handle buffer uploads
                if !self.queued_vertices.is_empty() {
                    self.queue.write_buffer(
                        &self.vertex_buffer,
                        0,
                        bytemuck::cast_slice(&self.queued_vertices),
                    );
                    self.queue.write_buffer(
                        &self.index_buffer,
                        0,
                        bytemuck::cast_slice(&self.queued_indices),
                    );
                }

                let mut encoder = self
                    .device
                    .create_command_encoder(&(wgpu::CommandEncoderDescriptor { label: None }));

                // Create render pass with clear and render shapes
                {
                    let mut render_pass = encoder.begin_render_pass(
                        &(wgpu::RenderPassDescriptor {
                            label: Some("Shape Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        }),
                    );

                    if !self.queued_vertices.is_empty() {
                        render_pass.set_pipeline(&self.pipeline);
                        render_pass.set_bind_group(0, &self.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            self.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.draw_indexed(0..self.queued_indices.len() as u32, 0, 0..1);
                    }
                }

                // Prepare text (upload glyphs to atlas)
                self.text_renderer.prepare(&self.device, &self.queue)?;

                // Render text on top
                {
                    let mut render_pass = encoder.begin_render_pass(
                        &(wgpu::RenderPassDescriptor {
                            label: Some("Text Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,  // Keep shapes!
                                    store: wgpu::StoreOp::Store,
                                },
                                depth_slice: None,
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                            multiview_mask: None,
                        }),
                    );

                    self.text_renderer.render(&mut render_pass)?;
                }

                // After submit, clear text buffers
                self.text_renderer.clear_frame();

                self.queue.submit(iter::once(encoder.finish()));
                frame.present();

                // Clear queued data for next frame
                self.queued_vertices.clear();
                self.queued_indices.clear();

                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}
