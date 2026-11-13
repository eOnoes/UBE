mod camera;
mod debug_overlay;
mod depth;
mod mesh;
mod model;
mod pipeline;

pub use camera::Camera;

use debug_overlay::DebugOverlay;
use depth::DepthTexture;
use engine_assets::StaticModel;
use engine_platform::{NativeWindow, WindowSize};
use model::TexturedModel;
use pipeline::GeometryPipeline;
use thiserror::Error;
use tracing::{info, warn};
use wgpu::{
    Backends, Color, CommandEncoderDescriptor, CurrentSurfaceTexture, Device, DeviceDescriptor,
    Instance, InstanceDescriptor, LoadOp, Operations, PowerPreference, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureViewDescriptor,
};

/// A failure to initialize or execute the renderer.
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("failed to create the graphics surface: {0}")]
    SurfaceCreation(String),
    #[error("failed to select a DirectX 12 adapter: {0}")]
    Adapter(String),
    #[error("failed to create the graphics device: {0}")]
    Device(String),
    #[error("the selected adapter cannot present to the application window")]
    UnsupportedSurface,
    #[error("surface acquisition failed validation")]
    SurfaceValidation,
}

/// Owns the GPU device, presentation surface, model resources, and frame execution.
pub struct Renderer {
    _instance: Instance,
    window: NativeWindow,
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    minimized: bool,
    depth: DepthTexture,
    geometry: GeometryPipeline,
    models: Vec<TexturedModel>,
    debug_overlay: DebugOverlay,
}

/// One scene model instance drawn at an extracted world transform.
#[derive(Clone, Copy, Debug)]
pub struct RenderItem {
    pub model_index: usize,
    pub transform: [[f32; 4]; 4],
}

impl Renderer {
    /// Initializes DirectX 12 rendering and uploads already loaded models.
    pub fn new(
        window: NativeWindow,
        assets: &[StaticModel],
        object_capacity: usize,
    ) -> Result<Self, RenderError> {
        let mut descriptor = InstanceDescriptor::new_without_display_handle();
        descriptor.backends = Backends::DX12;
        let instance = Instance::new(descriptor);
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| RenderError::SurfaceCreation(error.to_string()))?;

        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            ..Default::default()
        }))
        .map_err(|error| RenderError::Adapter(error.to_string()))?;

        let adapter_info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: Some("Engine Device"),
            ..Default::default()
        }))
        .map_err(|error| RenderError::Device(error.to_string()))?;

        let size = window.size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .ok_or(RenderError::UnsupportedSurface)?;
        config.present_mode = PresentMode::AutoVsync;
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);

        let depth = DepthTexture::new(&device, config.width, config.height);
        let geometry = GeometryPipeline::new(&device, config.format, object_capacity);
        let debug_overlay = DebugOverlay::new(&device, config.format);
        let models = assets
            .iter()
            .map(|asset| TexturedModel::from_asset(&device, &queue, &geometry, asset))
            .collect::<Vec<_>>();

        info!(
            adapter = %adapter_info.name,
            backend = ?adapter_info.backend,
            format = ?config.format,
            width = config.width,
            height = config.height,
            models = models.len(),
            primitives = models.iter().map(|model| model.primitives.len()).sum::<usize>(),
            object_capacity,
            "renderer initialized"
        );

        Ok(Self {
            _instance: instance,
            window,
            surface,
            device,
            queue,
            config,
            minimized: size.is_empty(),
            depth,
            geometry,
            models,
            debug_overlay,
        })
    }

    /// Updates the presentation surface after a native window resize.
    pub fn resize(&mut self, size: WindowSize) {
        self.minimized = size.is_empty();
        if self.minimized {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.reconfigure();
        info!(width = size.width, height = size.height, "surface resized");
    }

    /// Requests the next native redraw.
    pub fn request_redraw(&self) {
        if !self.minimized {
            self.window.request_redraw();
        }
    }

    /// Renders model instances and an optional owned debug overlay, then presents the frame.
    pub fn render(
        &mut self,
        camera: &Camera,
        items: &[RenderItem],
        debug_output: Option<(egui::Context, egui::FullOutput)>,
    ) -> Result<(), RenderError> {
        if self.minimized {
            return Ok(());
        }

        let (output, reconfigure_after_present) = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(output) => (output, false),
            CurrentSurfaceTexture::Suboptimal(output) => (output, true),
            CurrentSurfaceTexture::Lost | CurrentSurfaceTexture::Outdated => {
                self.reconfigure();
                return Ok(());
            }
            CurrentSurfaceTexture::Timeout => {
                warn!("surface acquisition timed out; skipping frame");
                return Ok(());
            }
            CurrentSurfaceTexture::Occluded => return Ok(()),
            CurrentSurfaceTexture::Validation => {
                return Err(RenderError::SurfaceValidation);
            }
        };

        let aspect_ratio = self.config.width as f32 / self.config.height as f32;
        let model_matrices = items.iter().map(|item| item.transform).collect::<Vec<_>>();
        self.geometry
            .update_camera(&self.queue, camera, aspect_ratio, &model_matrices);

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Frame Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Geometry Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.05,
                            g: 0.15,
                            b: 0.30,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(self.geometry.pipeline());
            for (index, item) in items.iter().enumerate() {
                render_pass.set_bind_group(
                    0,
                    self.geometry.camera_bind_group(),
                    &[self.geometry.object_offset(index)],
                );
                for primitive in &self.models[item.model_index].primitives {
                    render_pass.set_bind_group(1, &primitive.material, &[]);
                    render_pass.set_vertex_buffer(0, primitive.mesh.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(
                        primitive.mesh.index_buffer.slice(..),
                        primitive.mesh.index_format,
                    );
                    render_pass.draw_indexed(0..primitive.mesh.index_count, 0, 0..1);
                }
            }
        }

        let mut commands = if let Some((context, debug_output)) = debug_output {
            self.debug_overlay.render(
                &self.device,
                &self.queue,
                &mut encoder,
                &view,
                (context, debug_output),
                [self.config.width, self.config.height],
            )
        } else {
            Vec::new()
        };
        commands.push(encoder.finish());
        self.queue.submit(commands);
        self.queue.present(output);
        if reconfigure_after_present {
            self.reconfigure();
        }
        Ok(())
    }

    fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
        self.depth = DepthTexture::new(&self.device, self.config.width, self.config.height);
    }
}
