use wgpu::{BufferAddress, VertexBufferLayout, VertexStepMode};

pub const U32_SIZE: BufferAddress = std::mem::size_of::<u32>() as BufferAddress;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4], // RGBA
}

// Lets me convert vertices to raw bytes
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: [x, y],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    pub fn with_color(x: f32, y: f32, color: [f32; 4]) -> Self {
        Self {
            position: [x, y],
            color,
        }
    }

    pub const SIZE: BufferAddress = std::mem::size_of::<Self>() as BufferAddress;
    pub const DESC: VertexBufferLayout<'static> = VertexBufferLayout {
        array_stride: Vertex::SIZE,
        step_mode: VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x4
        ],
    };
}

pub const UNBOUNDED_F32: f32 = std::f32::INFINITY;

// TODO! Rename? Might not need to but this might be too all-encompassing if I make a UI system
#[derive(Debug)]
pub struct Text {
    pub text: String,
    pub position: glam::Vec2,
    pub size: f32,
    pub color: glam::Vec4,
    pub bounds: Option<glyphon::TextBounds>,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: String::new(),
            position: (0.0, 0.0).into(),
            size: 16.0,
            color: (1.0, 1.0, 1.0, 1.0).into(),
            bounds: None,
        }
    }
}
