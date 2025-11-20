use std::{iter, sync::Arc};

use wgpu::{BindGroup, Buffer};
use wgpu_glyph::{Section, Text};
use winit::{dpi::PhysicalSize, window::Window};

use crate::init::*;
use crate::types::*;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
    queued_vertices: Vec<Vertex>,
    queued_indices: Vec<u32>,
    // Passed into shaders
    screen_size_buffer: Buffer,
    bind_group: BindGroup,
}

impl Renderer {
    pub fn width(&self) -> f32 {
        self.config.width as f32
    }

    pub fn height(&self) -> f32 {
        self.config.height as f32
    }

    pub async fn new(window: Arc<Window>, size: PhysicalSize<u32>) -> Renderer {
        log::warn!("size: {:?}", size);

        // Create core wgpu components
        let instance = create_instance();
        let surface = create_surface(&instance, window);
        let adapter = create_adapter(&instance, wgpu::PowerPreference::HighPerformance, &surface).await;
        let (device, queue) = create_device_and_queue(&adapter).await;

        let config = create_surface_config(&surface, &adapter, size);

        let bind_group_layout = create_bind_group_layout(&device);
        let pipeline_layout = create_pipeline_layout(&device, &bind_group_layout);

        let (vert_shader, frag_shader) = create_shader_modules(&device);

        let screen_size_buffer = create_screen_size_buffer(&device, size);
        let (vertex_buffer, index_buffer) = create_vertex_and_index_buffers(&device);

        let bind_group = create_bind_group(&device, &bind_group_layout, &screen_size_buffer);

        let pipeline = create_render_pipeline(&device, &pipeline_layout, config.format, &[Vertex::DESC], vert_shader, frag_shader);

        let glyph_brush = create_glyph_brush(&device, config.format);
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            vertex_buffer,
            index_buffer,
            glyph_brush,
            staging_belt,
            queued_vertices: Vec::new(),
            queued_indices: Vec::new(),
            screen_size_buffer,
            bind_group,
        }
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
    }

    pub fn queue_text(&mut self, text: &str, position: (f32, f32), size: f32, color: [f32; 4]) {
        let section = Section {
            screen_position: position,
            bounds: (self.config.width as f32, self.config.height as f32),
            layout: wgpu_glyph::Layout::default().h_align(wgpu_glyph::HorizontalAlign::Left),
            ..Section::default()
        }
        .add_text(Text::new(text).with_color(color).with_scale(size));

        self.glyph_brush.queue(section);
    }

    pub fn render_text(&mut self) -> Result<(), wgpu::SurfaceError> {
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&Default::default());
                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                self.glyph_brush
                    .draw_queued(
                        &self.device,
                        &mut self.staging_belt,
                        &mut encoder,
                        &view,
                        self.config.width,
                        self.config.height,
                    )
                    .unwrap();

                self.staging_belt.finish();
                self.queue.submit(iter::once(encoder.finish()));
                frame.present();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

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
            let angle = 2.0 * std::f32::consts::PI * (i as f32) / (SEGMENTS as f32);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            self.queued_vertices.push(Vertex::with_color(x, y, color));
        }

        for i in 0..SEGMENTS {
            let next = if i == SEGMENTS - 1 { 1 } else { i + 2 };
            self.queued_indices.push(vertex_offset + next as u32);
            self.queued_indices.push(vertex_offset + (i + 1) as u32);
            self.queued_indices.push(vertex_offset);
        }
    }

    pub fn begin_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.surface.get_current_texture()?;
        Ok(())
    }

    pub fn draw_shape(
        &mut self,
        num_indices: u32,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }

    pub fn render_frame(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                // Create render pass with clear and render shapes
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    });

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

                // Render text on top
                self.glyph_brush
                    .draw_queued(
                        &self.device,
                        &mut self.staging_belt,
                        &mut encoder,
                        &view,
                        self.config.width,
                        self.config.height,
                    )
                    .unwrap();

                self.staging_belt.finish();
                self.queue.submit(iter::once(encoder.finish()));
                frame.present();

                // Clear queued data for next frame
                self.queued_vertices.clear();
                self.queued_indices.clear();

                // Reclaim staging belt memory
                // If we don't do this, we get a memory leak.
                self.staging_belt.recall();

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
