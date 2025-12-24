use thiserror::Error;

use crate::text::error::TextError;

#[derive(Error, Debug)]
pub enum RenderError {
    // === Initialization Errors ===
    #[error("Failed to get GPU adapter: {0}")]
    AdapterRequest(#[from] wgpu::RequestAdapterError),

    #[error("Failed to create GPU device: {0}")]
    DeviceCreation(#[from] wgpu::RequestDeviceError),

    #[error("Failed to create surface: {0}")]
    SurfaceCreation(#[from] wgpu::CreateSurfaceError),

    #[error("No compatible surface format found")]
    NoSurfaceFormat,

    // === Runtime Surface Errors ===
    #[error("Surface error: {0}")]
    Surface(#[from] wgpu::SurfaceError),

    // === Device Errors ===
    #[error("GPU device was lost - this can happen after sleep/wake or driver crash")]
    DeviceLost,

    #[error("GPU out of memory")]
    OutOfMemory,

    // === Resource Errors ===
    #[error("Buffer size exceeded: requested {requested} bytes, maximum is {max}")]
    BufferSizeExceeded { requested: u64, max: u64 },

    #[error("Texture size exceeded: requested {width}x{height}, maximum dimension is {max}")]
    TextureSizeExceeded { width: u32, height: u32, max: u32 },

    // === Extra stuff!?!?!?! ===
    #[error("--- Text error ---\n{0}")]
    TextError(#[from] TextError)
}

impl RenderError {
    /// Returns true if this error is recoverable by recreating the surface
    pub fn is_surface_recoverable(&self) -> bool {
        matches!(
            self,
            RenderError::Surface(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated)
        )
    }

    /// Returns true if this error requires full renderer reinitialization
    pub fn requires_reinit(&self) -> bool {
        matches!(self, RenderError::DeviceLost)
    }

    /// Returns true if this is a fatal error that cannot be recovered
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            RenderError::AdapterRequest(_)
                | RenderError::DeviceCreation(_)
                | RenderError::NoSurfaceFormat
                | RenderError::OutOfMemory
        )
    }
}
