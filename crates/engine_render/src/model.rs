use engine_assets::{StaticModel, TextureData};
use wgpu::{
    AddressMode, BindGroup, Device, Extent3d, FilterMode, Queue, Sampler, SamplerDescriptor,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
};

use crate::mesh::Mesh;
use crate::pipeline::GeometryPipeline;

/// GPU resources for all primitives in a static model.
pub struct TexturedModel {
    pub primitives: Vec<ModelPrimitive>,
}

pub struct ModelPrimitive {
    pub mesh: Mesh,
    pub material: BindGroup,
    _texture: Texture,
    _texture_view: TextureView,
    _sampler: Sampler,
}

impl TexturedModel {
    pub fn from_asset(
        device: &Device,
        queue: &Queue,
        geometry: &GeometryPipeline,
        asset: &StaticModel,
    ) -> Self {
        let primitives = asset
            .meshes
            .iter()
            .map(|mesh| {
                let (texture, texture_view, sampler) =
                    create_texture(device, queue, &mesh.base_color);
                let material = geometry.create_material_bind_group(device, &texture_view, &sampler);
                ModelPrimitive {
                    mesh: Mesh::from_asset(device, mesh),
                    material,
                    _texture: texture,
                    _texture_view: texture_view,
                    _sampler: sampler,
                }
            })
            .collect();
        Self { primitives }
    }
}

fn create_texture(
    device: &Device,
    queue: &Queue,
    data: &TextureData,
) -> (Texture, TextureView, Sampler) {
    let size = Extent3d {
        width: data.width,
        height: data.height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("glTF Base Color Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &data.rgba8,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * data.width),
            rows_per_image: Some(data.height),
        },
        size,
    );
    let texture_view = texture.create_view(&TextureViewDescriptor::default());
    let sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("glTF Base Color Sampler"),
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });
    (texture, texture_view, sampler)
}
