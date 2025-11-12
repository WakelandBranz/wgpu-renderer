use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Failed to initialize renderer: {0}")]
    InitializationError(String),
    #[error("Surface error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Device lost error")]
    DeviceLost,
    #[error("Render pass error: {0}")]
    RenderPassError(String),
    #[error("Shader compilation error: {0}")]
    ShaderError(String),
    #[error("Resource creation error: {0}")]
    ResourceError(String),
}
