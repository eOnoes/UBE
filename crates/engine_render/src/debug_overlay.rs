use egui_wgpu::{RendererOptions, ScreenDescriptor};
use wgpu::{
    CommandBuffer, CommandEncoder, Device, LoadOp, Operations, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureFormat, TextureView,
};

/// Renderer-owned egui resources and overlay paint execution.
pub struct DebugOverlay {
    renderer: egui_wgpu::Renderer,
}

impl DebugOverlay {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        Self {
            renderer: egui_wgpu::Renderer::new(device, format, RendererOptions::PREDICTABLE),
        }
    }

    /// Uploads and paints one owned egui frame over an already rendered color target.
    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        frame: (egui::Context, egui::FullOutput),
        size: [u32; 2],
    ) -> Vec<CommandBuffer> {
        let (context, output) = frame;
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                self.renderer.update_texture(device, queue, *id, delta);
            }
        }
        let pixels_per_point = output.pixels_per_point;
        let paint_jobs = context.tessellate(output.shapes, pixels_per_point);
        let screen = ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point,
        };
        let commands = self
            .renderer
            .update_buffers(device, queue, encoder, &paint_jobs, &screen);
        {
            let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Debug Inspector Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.renderer
                .render(&mut pass.forget_lifetime(), &paint_jobs, &screen);
        }
        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
        commands
    }
}
