use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

use crate::{camera::Camera2D, error::RenderError};

pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: (u32, u32),
    is_surface_configured: bool,
}

impl Renderer {
    pub async fn new(
        window: &(impl HasWindowHandle + HasDisplayHandle + Send + Sync),
        window_size: [u32; 2],
    ) -> Result<Self, RenderError> {
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance.create_surface(window).map_err(|e| {
            RenderError::InitializationError(format!("Failed to create surface: {}", e))
        })?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| {
                RenderError::InitializationError("Failed to find suitable GPU adapter".to_string())
            })?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: Some("renderer_device"),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                RenderError::InitializationError(format!("Failed to create device: {}", e))
            })?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size[0],
            height: window_size[1],
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size: window_size.into(),
            is_surface_configured: false,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        if width > 0 && height > 0 {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            Ok(())
        } else {
            Err(RenderError::InitializationError(
                "Invalid window dimensions".to_string(),
            ))
        }
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, RenderError> {
        self.surface.get_current_texture().map_err(Into::into)
    }

    pub fn present(&self) {
        // Frame presentation is handled by drop(SurfaceTexture)
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size.0 as f32 / self.size.1 as f32
    }

    pub fn create_camera(&self) -> Camera2D {
        Camera2D::new(self.size.0 as f32, self.size.1 as f32)
    }
}
