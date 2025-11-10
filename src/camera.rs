use glam::{Mat4, Vec3};

#[derive(Debug, Clone)]
pub struct Camera2D {
    position: Vec3,
    width: f32,
    height: f32,
}

impl Camera2D {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vec3::ZERO,
            width,
            height,
        }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(
            self.position + Vec3::new(0.0, 0.0, 1.0),
            self.position,
            Vec3::Y,
        )
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            -self.width / 2.0,
            self.width / 2.0,
            -self.height / 2.0,
            self.height / 2.0,
            0.1,
            100.0,
        )
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn from_camera(camera: &Camera2D) -> Self {
        Self {
            view_proj: camera.view_projection_matrix().to_cols_array_2d(),
        }
    }
}
