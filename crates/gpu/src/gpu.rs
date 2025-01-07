use std::sync::Arc;

use ambient_native_std::asset_cache::SyncAssetKey;
use ambient_settings::RenderSettings;
use anyhow::Context;
use bytemuck::{Pod, Zeroable};
use glam::{uvec2, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use wgpu::{InstanceDescriptor, PresentMode, TextureFormat};
use winit::window::Window;

// #[cfg(debug_assertions)]
pub const DEFAULT_SAMPLE_COUNT: u32 = 1;
// #[cfg(not(debug_assertions))]
// pub const DEFAULT_SAMPLE_COUNT: u32 = 4;

#[derive(Debug)]
pub struct GpuKey;
impl SyncAssetKey<Arc<Gpu>> for GpuKey {}

#[derive(Debug)]
pub struct Gpu {
    pub surface: Option<wgpu::Surface>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swapchain_format: Option<TextureFormat>,
    pub swapchain_mode: Option<PresentMode>,
    pub adapter: wgpu::Adapter,
    /// If this is true, we don't need to use blocking device.polls, since they are assumed to be polled elsewhere
    pub will_be_polled: bool,
}

impl Gpu {
    pub async fn new(window: Option<&Window>) -> anyhow::Result<Self> {
        Self::with_config(window, false, &RenderSettings::default()).await
    }
    pub async fn with_config(
        window: Option<&Window>,
        will_be_polled: bool,
        settings: &RenderSettings,
    ) -> anyhow::Result<Self> {
        let _span = tracing::info_span!("create_gpu").entered();
        // From: https://github.com/KhronosGroup/Vulkan-Loader/issues/552
        #[cfg(not(target_os = "unknown"))]
        {
            std::env::set_var("DISABLE_LAYER_AMD_SWITCHABLE_GRAPHICS_1", "1");
            std::env::set_var("DISABLE_LAYER_NV_OPTIMUS_1", "1");
        }

        let backends = if cfg!(target_os = "windows") {
            wgpu::Backends::VULKAN
        } else if cfg!(target_os = "macos") {
            wgpu::Backends::PRIMARY
        } else if cfg!(target_os = "unknown") {
            wgpu::Backends::BROWSER_WEBGPU
        } else {
            wgpu::Backends::all()
        };

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            // NOTE: Vulkan is used for windows as a non-zero indirect `first_instance` is not supported, and we have to resort direct rendering
            // See: <https://github.com/gfx-rs/wgpu/issues/2471>
            //
            // TODO: upgrade to Dxc? This requires us to ship additionall dll files, which may be
            // possible using an installer. Nevertheless, we are currently using Vulkan on windows
            // due to `base_instance` being broken on windows.
            // https://docs.rs/wgpu/latest/wgpu/enum.Dx12Compiler.html
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            // dx12_shader_compiler: wgpu::Dx12Compiler::Dxc {
            //     dxil_path: Some("./dxil.dll".into()),
            //     dxc_path: Some("./dxcompiler.dll".into()),
            // },
        });

        let surface = window
            .map(|window| unsafe { instance.create_surface(window) })
            .transpose()
            .context("Failed to create surface")?;

        #[cfg(not(target_os = "unknown"))]
        {
            tracing::debug!("Available adapters:");
            for adapter in instance.enumerate_adapters(wgpu::Backends::PRIMARY) {
                tracing::debug!("Adapter: {:?}", adapter.get_info());
            }
        }

        tracing::debug!("Requesting adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface.as_ref(),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find an appopriate adapter")?;

        tracing::info!("Using gpu adapter: {:?}", adapter.get_info());

        tracing::debug!("Adapter features:\n{:#?}", adapter.features());
        let adapter_limits = adapter.limits();
        tracing::debug!("Adapter limits:\n{:#?}", adapter_limits);

        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                // The renderer will dispatch 1 indirect draw command for *each* primitive in the
                // scene, but the data draw data such as index_count, first_instance, etc lives on
                // the gpu
                let features = wgpu::Features::empty();
            } else if #[cfg(target_os = "unknown")] {

                // Same as above, but the *web*gpu target requires a feature flag to be set, or
                // else indirect commands no-op
                let features = wgpu::Features::INDIRECT_FIRST_INSTANCE;
            } else {
                // TODO: make configurable at runtime
                // The renderer will use indirect drawing with the draw commands *and* count
                // fetched from gpu side buffers
                let features =
                wgpu::Features::MULTI_DRAW_INDIRECT | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT;
            }
        };

        tracing::info!("Using device features: {features:?}");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::default()
                        | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                        // | wgpu::Features::POLYGON_MODE_LINE
                        | features,
                    limits: wgpu::Limits {
                        max_bind_groups: 8,
                        max_storage_buffer_binding_size: adapter_limits
                            .max_storage_buffer_binding_size,
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .context("Failed to request a device")?;

        tracing::debug!("Device limits:\n{:#?}", device.limits());

        let swapchain_format = surface
            .as_ref()
            .map(|surface| surface.get_capabilities(&adapter).formats[0]);

        tracing::debug!("Swapchain format: {swapchain_format:?}");

        let swapchain_mode = if surface.is_some() {
            if settings.vsync() {
                // From wgpu docs:
                // "Chooses FifoRelaxed -> Fifo based on availability."
                Some(PresentMode::AutoVsync)
            } else {
                // From wgpu docs:
                // "Chooses Immediate -> Mailbox -> Fifo (on web) based on availability."
                Some(PresentMode::AutoNoVsync)
            }
        } else {
            None
        };

        tracing::debug!("Swapchain present mode: {swapchain_mode:?}");

        if let (Some(window), Some(surface), Some(mode), Some(format)) =
            (window, &surface, swapchain_mode, swapchain_format)
        {
            let size = window.inner_size();
            surface.configure(
                &device,
                &Self::create_sc_desc(format, mode, uvec2(size.width, size.height)),
            );
        }

        Ok(Self {
            device,
            surface,
            queue,
            swapchain_format,
            swapchain_mode,
            adapter,
            will_be_polled,
        })
    }

    pub fn resize(&self, size: winit::dpi::PhysicalSize<u32>) {
        if let Some(surface) = &self.surface {
            if size.width > 0 && size.height > 0 {
                surface.configure(&self.device, &self.sc_desc(uvec2(size.width, size.height)));
            }
        }
    }
    pub fn swapchain_format(&self) -> TextureFormat {
        self.swapchain_format
            .unwrap_or(TextureFormat::Rgba8UnormSrgb)
    }
    pub fn swapchain_mode(&self) -> PresentMode {
        self.swapchain_mode.unwrap_or(PresentMode::Immediate)
    }
    pub fn sc_desc(&self, size: UVec2) -> wgpu::SurfaceConfiguration {
        Self::create_sc_desc(self.swapchain_format(), self.swapchain_mode(), size)
    }
    fn create_sc_desc(
        format: TextureFormat,
        present_mode: PresentMode,
        size: UVec2,
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.x,
            height: size.y,
            present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        }
    }
}

pub trait WgslType: Zeroable + Pod + 'static {
    fn wgsl_type() -> &'static str;
}
impl WgslType for f32 {
    fn wgsl_type() -> &'static str {
        "f32"
    }
}
impl WgslType for i32 {
    fn wgsl_type() -> &'static str {
        "i32"
    }
}
impl WgslType for u32 {
    fn wgsl_type() -> &'static str {
        "u32"
    }
}

impl WgslType for Vec2 {
    fn wgsl_type() -> &'static str {
        "vec2<f32>"
    }
}

impl WgslType for Vec3 {
    fn wgsl_type() -> &'static str {
        "vec3<f32>"
    }
}

impl WgslType for Vec4 {
    fn wgsl_type() -> &'static str {
        "vec4<f32>"
    }
}

impl WgslType for UVec2 {
    fn wgsl_type() -> &'static str {
        "vec2<u32>"
    }
}

impl WgslType for UVec3 {
    fn wgsl_type() -> &'static str {
        "vec3<u32>"
    }
}

impl WgslType for UVec4 {
    fn wgsl_type() -> &'static str {
        "vec4<u32>"
    }
}
