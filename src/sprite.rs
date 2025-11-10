use glam::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct Sprite {
    position: Vec3,
    size: Vec2,
    rotation: f32,
    color: [f32; 4],
    texture_coords: SpriteTextureCoords,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteTextureCoords {
    pub u_min: f32,
    pub v_min: f32,
    pub u_max: f32,
    pub v_max: f32,
}

impl Default for SpriteTextureCoords {
    fn default() -> Self {
        Self {
            u_min: 0.0,
            v_min: 0.0,
            u_max: 1.0,
            v_max: 1.0,
        }
    }
}

impl Sprite {
    pub fn new(position: Vec3, size: Vec2) -> Self {
        Self {
            position,
            size,
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            texture_coords: SpriteTextureCoords::default(),
        }
    }

    pub fn with_color(mut self, color: [f32; 4]) -> Self {
        self.color = color;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_texture_coords(mut self, coords: SpriteTextureCoords) -> Self {
        self.texture_coords = coords;
        self
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_size(&mut self, size: Vec2) {
        self.size = size;
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    pub fn texture_coords(&self) -> SpriteTextureCoords {
        self.texture_coords
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteVertex {
    pub position: [f32; 2],
    pub tex_coords: [f32; 2],
}

impl SpriteVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SpriteInstance {
    pub position: [f32; 3],
    pub size: [f32; 2],
    pub rotation: f32,
    pub color: [f32; 4],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

impl SpriteInstance {
    pub fn from_sprite(sprite: &Sprite) -> Self {
        Self {
            position: sprite.position.to_array(),
            size: sprite.size.to_array(),
            rotation: sprite.rotation,
            color: sprite.color,
            uv_min: [sprite.texture_coords.u_min, sprite.texture_coords.v_min],
            uv_max: [sprite.texture_coords.u_max, sprite.texture_coords.v_max],
        }
    }

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress
                        + mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress
                        + mem::size_of::<[f32; 2]>() as wgpu::BufferAddress
                        + mem::size_of::<f32>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress
                        + mem::size_of::<[f32; 2]>() as wgpu::BufferAddress
                        + mem::size_of::<f32>() as wgpu::BufferAddress
                        + mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress
                        + mem::size_of::<[f32; 2]>() as wgpu::BufferAddress
                        + mem::size_of::<f32>() as wgpu::BufferAddress
                        + mem::size_of::<[f32; 4]>() as wgpu::BufferAddress
                        + mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
