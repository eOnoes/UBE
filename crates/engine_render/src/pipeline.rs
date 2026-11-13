use bytemuck::{Pod, Zeroable};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType,
    BufferDescriptor, BufferSize, BufferUsages, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, Device, FragmentState, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerBindingType, ShaderModuleDescriptor, ShaderSource,
    ShaderStages, StencilState, TextureFormat, TextureSampleType, TextureView,
    TextureViewDimension, VertexBufferLayout, VertexState, VertexStepMode,
};

use crate::camera::Camera;
use crate::depth::DEPTH_FORMAT;
use crate::mesh::Vertex;

const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
}

/// GPU pipeline and bind-group layouts for textured indexed geometry.
pub struct GeometryPipeline {
    pipeline: RenderPipeline,
    camera_buffer: Buffer,
    camera_bind_group: BindGroup,
    material_bind_group_layout: BindGroupLayout,
    object_stride: u64,
    object_capacity: usize,
}

impl GeometryPipeline {
    pub fn new(device: &Device, surface_format: TextureFormat, object_capacity: usize) -> Self {
        let uniform_size = size_of::<CameraUniform>() as u64;
        let alignment = u64::from(device.limits().min_uniform_buffer_offset_alignment);
        let object_stride = uniform_size.next_multiple_of(alignment);
        let object_capacity = object_capacity.max(1);
        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Camera and Model Uniform Buffer"),
            size: object_stride * object_capacity as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(uniform_size),
                    },
                    count: None,
                }],
            });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &camera_buffer,
                    offset: 0,
                    size: BufferSize::new(uniform_size),
                }),
            }],
        });
        let material_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Material Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Geometry Pipeline Layout"),
            bind_group_layouts: &[
                Some(&camera_bind_group_layout),
                Some(&material_bind_group_layout),
            ],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Geometry Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Geometry Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Some(VertexBufferLayout {
                    array_stride: size_of::<Vertex>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &VERTEX_ATTRIBUTES,
                })],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: Some(DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Less),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            material_bind_group_layout,
            object_stride,
            object_capacity,
        }
    }

    pub fn create_material_bind_group(
        &self,
        device: &Device,
        texture_view: &TextureView,
        sampler: &Sampler,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("glTF Material Bind Group"),
            layout: &self.material_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    pub fn update_camera(
        &self,
        queue: &Queue,
        camera: &Camera,
        aspect_ratio: f32,
        model_matrices: &[[[f32; 4]; 4]],
    ) {
        assert!(model_matrices.len() <= self.object_capacity);
        if model_matrices.is_empty() {
            return;
        }
        let uniform_size = size_of::<CameraUniform>();
        let view_projection = camera.view_projection(aspect_ratio);
        let mut bytes = vec![0_u8; self.object_stride as usize * model_matrices.len()];
        for (index, model) in model_matrices.iter().enumerate() {
            let uniform = CameraUniform {
                view_projection,
                model: *model,
            };
            let start = index * self.object_stride as usize;
            bytes[start..start + uniform_size].copy_from_slice(bytemuck::bytes_of(&uniform));
        }
        queue.write_buffer(&self.camera_buffer, 0, &bytes);
    }

    pub fn object_offset(&self, index: usize) -> u32 {
        (index as u64 * self.object_stride) as u32
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn camera_bind_group(&self) -> &BindGroup {
        &self.camera_bind_group
    }
}
