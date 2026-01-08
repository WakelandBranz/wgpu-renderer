use wgpu::{BindGroupLayout, Device, RenderPipeline, TextureFormat, VertexBufferLayout};

use crate::init::{
    create_camera_bind_group_layout, create_pipeline_layout, create_render_pipeline,
    create_screen_size_bind_group_layout, create_shader_modules, create_texture_bind_group_layout,
};

// Thank you egor
// Centralizes rendering pipeline resources
pub struct Pipelines {
    pub primitive: RenderPipeline,
    pub screen_size_bind_group_layout: BindGroupLayout,
    pub camera_bind_group_layout: BindGroupLayout,
    pub texture_bind_group_layout: BindGroupLayout,
}

impl Pipelines {
    pub(crate) fn init(
        device: &Device,
        surface_format: TextureFormat,
        vertex_layouts: &[VertexBufferLayout],
    ) -> Self {
        let screen_size_bind_group_layout = create_screen_size_bind_group_layout(device);
        let camera_bind_group_layout = create_camera_bind_group_layout(device);
        let texture_bind_group_layout = create_texture_bind_group_layout(device);

        let primitive_pipeline_layout = create_pipeline_layout(
            device,
            "Primitive Pipeline Layout",
            &[
                &screen_size_bind_group_layout,
                &camera_bind_group_layout,
                &texture_bind_group_layout,
            ],
        );

        let (vert_shader, frag_shader) = create_shader_modules(device);

        let primitive = create_render_pipeline(
            device,
            "Primitive Pipeline",
            &primitive_pipeline_layout,
            surface_format,
            vertex_layouts,
            vert_shader,
            frag_shader,
        );

        Pipelines {
            primitive,
            screen_size_bind_group_layout,
            camera_bind_group_layout,
            texture_bind_group_layout,
        }
    }
}
