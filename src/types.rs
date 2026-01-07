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