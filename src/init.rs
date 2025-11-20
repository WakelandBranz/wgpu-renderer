//! Contains all initialization code. Makes renderer::new() much simpler to read.

use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use wgpu::Backends;
use wgpu::{ Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent, BlendState, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, ColorTargetState, ColorWrites, Device, DeviceDescriptor, Features, FragmentState, FrontFace, Instance, InstanceDescriptor, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, VertexBufferLayout, VertexState };
use wgpu::util::{ BufferInitDescriptor, DeviceExt };
use wgpu_glyph::ab_glyph;
use winit::{ dpi::PhysicalSize, window::Window };

use crate::types::{ Vertex, U32_SIZE };

const FONT_BYTES: &[u8] = include_bytes!("../res/fonts/PressStart2P-Regular.ttf");

pub(crate) fn create_instance() -> Instance {
    Instance::new(
        &(InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: Backends::GL,
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
            &(RequestAdapterOptions {
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
            &(DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
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
) -> SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format = surface_caps.formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

pub(crate) fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(
        &(BindGroupLayoutDescriptor {
            label: Some("Screen Size BGL"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
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
    bind_group_layout: &BindGroupLayout
) -> PipelineLayout {
    device.create_pipeline_layout(
        &(PipelineLayoutDescriptor {
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
            label: Some("Pipeline Layout"),
        })
    )
}

pub(crate) fn create_shader_modules(device: &Device) -> (ShaderModule, ShaderModule) {
    let vert_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("vertex shader"),
        source: ShaderSource::Wgsl(
            std::borrow::Cow::Borrowed(include_str!("../res/shaders/textured.vert.wgsl"))
        ),
    });

    let frag_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("fragment shader"),
        source: ShaderSource::Wgsl(
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
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
    )
}

pub(crate) fn create_vertex_and_index_buffers(device: &Device) -> (Buffer, Buffer) {
    let vertex_buffer = device.create_buffer(
        &(BufferDescriptor {
            label: None,
            size: Vertex::SIZE * 256,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    );

    let index_buffer = device.create_buffer(
        &(BufferDescriptor {
            label: None,
            size: U32_SIZE * 512,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    );

    (vertex_buffer, index_buffer)
}

pub(crate) fn create_bind_group(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    screen_size_buffer: &Buffer
) -> BindGroup {
    device.create_bind_group(
        &(BindGroupDescriptor {
            label: Some("Screen Size BG"),
            layout: bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: screen_size_buffer.as_entire_binding(),
                },
            ],
        })
    )
}

pub(crate) fn create_render_pipeline(
    device: &Device,
    pipeline_layout: &PipelineLayout,
    surface_format: TextureFormat,
    vertex_layouts: &[VertexBufferLayout],
    vert_shader: ShaderModule,
    frag_shader: ShaderModule
) -> RenderPipeline {
    device.create_render_pipeline(
        &(RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(pipeline_layout),
            vertex: VertexState {
                module: &vert_shader,
                entry_point: Some("main"),
                buffers: vertex_layouts,
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &frag_shader,
                entry_point: Some("main"),
                targets: &[
                    Some(ColorTargetState {
                        format: surface_format,
                        blend: Some(BlendState {
                            alpha: BlendComponent::REPLACE,
                            color: BlendComponent::REPLACE,
                        }),
                        write_mask: ColorWrites::ALL,
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
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
    surface_format: TextureFormat
) -> wgpu_glyph::GlyphBrush<()> {
    let font = ab_glyph::FontArc::try_from_slice(FONT_BYTES).unwrap();
    wgpu_glyph::GlyphBrushBuilder::using_font(font).build(device, surface_format)
}
