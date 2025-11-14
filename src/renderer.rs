use std::{iter, sync::Arc};

use wgpu_glyph::{Section, Text, ab_glyph};
use winit::{dpi::PhysicalSize, window::Window};

use crate::types::*;

const FONT_BYTES: &[u8] = include_bytes!("../res/fonts/PressStart2P-Regular.ttf");

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    glyph_brush: wgpu_glyph::GlyphBrush<()>,
    staging_belt: wgpu::util::StagingBelt,
    queued_vertices: Vec<Vertex>,
    queued_indices: Vec<u32>,
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
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

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
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                ..Default::default()
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[],
            push_constant_ranges: &[],
            label: Some("Pipeline Layout"),
        });
        let vert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("vertex shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../res/shaders/textured.vert.wgsl"
            ))),
        });

        let frag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fragment shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "../res/shaders/textured.frag.wgsl"
            ))),
        });

        let pipeline = create_render_pipeline(
            &device,
            &pipeline_layout,
            config.format,
            &[Vertex::DESC],
            vert_shader,
            frag_shader,
        );

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: Vertex::SIZE * 256,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: U32_SIZE * 512,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let font = ab_glyph::FontArc::try_from_slice(FONT_BYTES).unwrap();
        let glyph_brush =
            wgpu_glyph::GlyphBrushBuilder::using_font(font).build(&device, config.format);
        let staging_belt = wgpu::util::StagingBelt::new(1024);

        Self {
            surface,
            adapter,
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
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        // Clamp to wgpu's maximum texture size (2048)
        const MAX_TEXTURE_SIZE: u32 = 2048;
        self.config.width = size.width.min(MAX_TEXTURE_SIZE);
        self.config.height = size.height.min(MAX_TEXTURE_SIZE);
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

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    vs_module: wgpu::ShaderModule,
    fs_module: wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: Some("main"),
            buffers: &vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: color_format,
                blend: Some(wgpu::BlendState {
                    alpha: wgpu::BlendComponent::REPLACE,
                    color: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
        cache: None,
    })
}
