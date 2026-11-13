use bytemuck::{Pod, Zeroable};
use engine_assets::{ModelVertex, TexturedMesh};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device, IndexFormat};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

impl From<ModelVertex> for Vertex {
    fn from(vertex: ModelVertex) -> Self {
        Self {
            position: vertex.position,
            normal: vertex.normal,
            tex_coord: vertex.tex_coord,
        }
    }
}

/// GPU-owned indexed mesh data.
pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub index_format: IndexFormat,
}

impl Mesh {
    pub fn from_asset(device: &Device, asset: &TexturedMesh) -> Self {
        let vertices = asset
            .vertices
            .iter()
            .copied()
            .map(Vertex::from)
            .collect::<Vec<_>>();
        let vertex_label = format!("{} Vertex Buffer", asset.name);
        let index_label = format!("{} Index Buffer", asset.name);
        Self {
            vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&vertex_label),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&index_label),
                contents: bytemuck::cast_slice(&asset.indices),
                usage: BufferUsages::INDEX,
            }),
            index_count: asset.indices.len() as u32,
            index_format: IndexFormat::Uint32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_vertex_layout_is_preserved() {
        let source = ModelVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            tex_coord: [0.25, 0.75],
        };

        let vertex = Vertex::from(source);

        assert_eq!(vertex.position, source.position);
        assert_eq!(vertex.normal, source.normal);
        assert_eq!(vertex.tex_coord, source.tex_coord);
        assert_eq!(size_of::<Vertex>(), 32);
    }
}
