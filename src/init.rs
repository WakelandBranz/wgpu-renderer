//! Contains all initialization code. Makes renderer::new() much simpler to read.

use std::sync::Arc;
use wgpu::{ Adapter, Buffer, Device, Instance, PowerPreference, Queue, Surface };
use wgpu::util::{ BufferInitDescriptor, DeviceExt };
use wgpu_glyph::ab_glyph;
use winit::{ dpi::PhysicalSize, window::Window };

use crate::types::{ Vertex, U32_SIZE };

const FONT_BYTES: &[u8] = include_bytes!("../res/fonts/PressStart2P-Regular.ttf");

pub(crate) fn create_instance() -> Instance {
    Instance::new(
        &(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        })
    )
}

pub(crate) fn create_surface(instance: &Instance, window: Arc<Window>) -> Surface<'static> {
    instance.create_surface(window).unwrap()
}

pub(crate) async fn create_adapter(
    instance: &Instance,
    power_preference: PowerPreference,
    surface: &Surface<'_>
) -> Adapter {
    instance
        .request_adapter(
            &(wgpu::RequestAdapterOptions {
                power_preference,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
        ).await
        .unwrap()
}

pub(crate) async fn create_device_and_queue(adapter: &Adapter) -> (Device, Queue) {
    adapter
        .request_device(
            &(wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: adapter.limits(),
                ..Default::default()
            })
        ).await
        .unwrap()
}

pub(crate) fn create_surface_config(
    surface: &Surface<'_>,
    adapter: &Adapter,
    size: PhysicalSize<u32>
) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format = surface_caps.formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

pub(crate) fn create_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(
        &(wgpu::BindGroupLayoutDescriptor {
            label: Some("Screen Size BGL"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    )
}

pub(crate) fn create_pipeline_layout(
    device: &Device,
    bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(
        &(wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
            label: Some("Pipeline Layout"),
        })
    )
}

pub(crate) fn create_shader_modules(device: &Device) -> (wgpu::ShaderModule, wgpu::ShaderModule) {
    let vert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("vertex shader"),
        source: wgpu::ShaderSource::Wgsl(
            std::borrow::Cow::Borrowed(include_str!("../res/shaders/textured.vert.wgsl"))
        ),
    });

    let frag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fragment shader"),
        source: wgpu::ShaderSource::Wgsl(
            std::borrow::Cow::Borrowed(include_str!("../res/shaders/textured.frag.wgsl"))
        ),
    });

    (vert_shader, frag_shader)
}

pub(crate) fn create_screen_size_buffer(device: &Device, size: PhysicalSize<u32>) -> Buffer {
    device.create_buffer_init(
        &(BufferInitDescriptor {
            label: Some("Screen Size Buffer"),
            contents: bytemuck::cast_slice(&[size.width as f32, size.height as f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    )
}

pub(crate) fn create_vertex_and_index_buffers(device: &Device) -> (Buffer, Buffer) {
    let vertex_buffer = device.create_buffer(
        &(wgpu::BufferDescriptor {
            label: None,
            size: Vertex::SIZE * 256,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    );

    let index_buffer = device.create_buffer(
        &(wgpu::BufferDescriptor {
            label: None,
            size: U32_SIZE * 512,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    );

    (vertex_buffer, index_buffer)
}

pub(crate) fn create_bind_group(
    device: &Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    screen_size_buffer: &Buffer
) -> wgpu::BindGroup {
    device.create_bind_group(
        &(wgpu::BindGroupDescriptor {
            label: Some("Screen Size BG"),
            layout: bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: screen_size_buffer.as_entire_binding(),
                },
            ],
        })
    )
}

pub(crate) fn create_render_pipeline(
    device: &Device,
    pipeline_layout: &wgpu::PipelineLayout,
    surface_format: wgpu::TextureFormat,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    vert_shader: wgpu::ShaderModule,
    frag_shader: wgpu::ShaderModule
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(
        &(wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader,
                entry_point: Some("main"),
                buffers: vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader,
                entry_point: Some("main"),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState {
                            alpha: wgpu::BlendComponent::REPLACE,
                            color: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        })
    )
}

pub(crate) fn create_glyph_brush(
    device: &Device,
    surface_format: wgpu::TextureFormat
) -> wgpu_glyph::GlyphBrush<()> {
    let font = ab_glyph::FontArc::try_from_slice(FONT_BYTES).unwrap();
    wgpu_glyph::GlyphBrushBuilder::using_font(font).build(device, surface_format)
}
