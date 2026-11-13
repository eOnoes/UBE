use std::path::{Path, PathBuf};

use gltf::image::Format;
use gltf::mesh::Mode;
use thiserror::Error;

/// CPU-side vertex data shared by asset importers and render backends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

/// Decoded, uncompressed RGBA8 texture data with top-left UV origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

/// One indexed triangle primitive and its base-color texture.
#[derive(Clone, Debug, PartialEq)]
pub struct TexturedMesh {
    pub name: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
    pub base_color: TextureData,
}

/// A static model containing one or more independently renderable primitives.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticModel {
    pub meshes: Vec<TexturedMesh>,
}

/// A contextual failure to import or validate a static textured model.
#[derive(Debug, Error)]
pub enum AssetError {
    #[error("failed to import glTF asset {path}: {source}")]
    Import {
        path: PathBuf,
        #[source]
        source: gltf::Error,
    },
    #[error("invalid textured glTF asset {path}: {details}")]
    Invalid { path: PathBuf, details: String },
}

/// Loads static triangle primitives with positions, normals, UV0, and a base-color texture.
///
/// This function performs file I/O and decoding without depending on a graphics backend.
pub fn load_static_gltf(path: impl AsRef<Path>) -> Result<StaticModel, AssetError> {
    let path = path.as_ref();
    let (document, buffers, images) = gltf::import(path).map_err(|source| AssetError::Import {
        path: path.to_path_buf(),
        source,
    })?;

    let mut meshes = Vec::new();
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let context = format!("mesh {:?}, primitive {}", mesh.name(), primitive.index());
            if primitive.mode() != Mode::Triangles {
                return Err(invalid(
                    path,
                    format!(
                        "{context} uses unsupported mode {:?}; expected triangles",
                        primitive.mode()
                    ),
                ));
            }

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let positions = reader
                .read_positions()
                .ok_or_else(|| invalid(path, format!("{context} has no POSITION attribute")))?;
            let normals = reader
                .read_normals()
                .ok_or_else(|| invalid(path, format!("{context} has no NORMAL attribute")))?;
            let tex_coords = reader
                .read_tex_coords(0)
                .ok_or_else(|| invalid(path, format!("{context} has no TEXCOORD_0 attribute")))?
                .into_f32();
            let indices = reader
                .read_indices()
                .ok_or_else(|| invalid(path, format!("{context} has no indices")))?
                .into_u32()
                .collect::<Vec<_>>();

            let vertices = positions
                .zip(normals)
                .zip(tex_coords)
                .map(|((position, normal), tex_coord)| ModelVertex {
                    position,
                    normal,
                    tex_coord,
                })
                .collect::<Vec<_>>();
            validate_vertices(path, &context, &vertices)?;
            validate_indices(path, &context, &vertices, &indices)?;

            let texture = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .ok_or_else(|| invalid(path, format!("{context} has no base-color texture")))?;
            let image = images
                .get(texture.texture().source().index())
                .ok_or_else(|| invalid(path, format!("{context} references a missing image")))?;
            let base_color = decode_rgba8(path, &context, image)?;

            let name = mesh.name().map(str::to_owned).unwrap_or_else(|| {
                format!("mesh-{}-primitive-{}", mesh.index(), primitive.index())
            });
            meshes.push(TexturedMesh {
                name,
                vertices,
                indices,
                base_color,
            });
        }
    }

    if meshes.is_empty() {
        return Err(invalid(path, "the asset contains no triangle primitives"));
    }
    Ok(StaticModel { meshes })
}

fn validate_vertices(
    path: &Path,
    context: &str,
    vertices: &[ModelVertex],
) -> Result<(), AssetError> {
    if vertices.is_empty() {
        return Err(invalid(path, format!("{context} has no vertices")));
    }
    for (index, vertex) in vertices.iter().enumerate() {
        let finite = vertex
            .position
            .iter()
            .chain(vertex.normal.iter())
            .chain(vertex.tex_coord.iter())
            .all(|value| value.is_finite());
        if !finite {
            return Err(invalid(
                path,
                format!("{context} vertex {index} contains a non-finite value"),
            ));
        }
        let normal_length_squared = vertex.normal.iter().map(|value| value * value).sum::<f32>();
        if normal_length_squared <= f32::EPSILON {
            return Err(invalid(
                path,
                format!("{context} vertex {index} has a zero-length normal"),
            ));
        }
    }
    Ok(())
}

fn validate_indices(
    path: &Path,
    context: &str,
    vertices: &[ModelVertex],
    indices: &[u32],
) -> Result<(), AssetError> {
    if indices.is_empty() || !indices.len().is_multiple_of(3) {
        return Err(invalid(
            path,
            format!("{context} indices do not form complete triangles"),
        ));
    }
    if let Some(index) = indices
        .iter()
        .find(|index| (**index as usize) >= vertices.len())
    {
        return Err(invalid(
            path,
            format!("{context} index {index} is outside its vertex buffer"),
        ));
    }
    Ok(())
}

fn decode_rgba8(
    path: &Path,
    context: &str,
    image: &gltf::image::Data,
) -> Result<TextureData, AssetError> {
    if image.width == 0 || image.height == 0 {
        return Err(invalid(path, format!("{context} has an empty texture")));
    }
    let expected_len = (image.width as usize)
        .checked_mul(image.height as usize)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or_else(|| invalid(path, format!("{context} texture dimensions overflow")))?;
    let rgba8 = match image.format {
        Format::R8 => image
            .pixels
            .iter()
            .flat_map(|value| [*value, *value, *value, u8::MAX])
            .collect(),
        Format::R8G8 => image
            .pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        Format::R8G8B8 => image
            .pixels
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], u8::MAX])
            .collect(),
        Format::R8G8B8A8 => image.pixels.clone(),
        format => {
            return Err(invalid(
                path,
                format!("{context} uses unsupported decoded image format {format:?}"),
            ));
        }
    };
    if rgba8.len() != expected_len {
        return Err(invalid(
            path,
            format!(
                "{context} texture has {} decoded bytes; expected {expected_len}",
                rgba8.len()
            ),
        ));
    }
    Ok(TextureData {
        width: image.width,
        height: image.height,
        rgba8,
    })
}

fn invalid(path: &Path, details: impl Into<String>) -> AssetError {
    AssetError::Invalid {
        path: path.to_path_buf(),
        details: details.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/models/BoxTextured.glb")
    }

    #[test]
    fn loads_known_textured_glb() {
        let model = load_static_gltf(fixture_path()).expect("known sample should load");

        assert!(!model.meshes.is_empty());
        for mesh in model.meshes {
            validate_vertices(Path::new("test"), "fixture", &mesh.vertices).unwrap();
            validate_indices(Path::new("test"), "fixture", &mesh.vertices, &mesh.indices).unwrap();
            assert_eq!(
                mesh.base_color.rgba8.len(),
                mesh.base_color.width as usize * mesh.base_color.height as usize * 4
            );
        }
    }

    #[test]
    fn rejects_out_of_bounds_indices() {
        let vertices = [ModelVertex {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            tex_coord: [0.0; 2],
        }];

        let error =
            validate_indices(Path::new("test.glb"), "test", &vertices, &[0, 1, 0]).unwrap_err();

        assert!(error.to_string().contains("index 1 is outside"));
    }

    #[test]
    fn rejects_non_finite_vertex_data() {
        let vertices = [ModelVertex {
            position: [f32::NAN, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tex_coord: [0.0; 2],
        }];

        let error = validate_vertices(Path::new("test.glb"), "test", &vertices).unwrap_err();

        assert!(error.to_string().contains("non-finite"));
    }

    #[test]
    fn converts_rgb_to_rgba() {
        let image = gltf::image::Data {
            pixels: vec![10, 20, 30, 40, 50, 60],
            format: Format::R8G8B8,
            width: 2,
            height: 1,
        };

        let texture = decode_rgba8(Path::new("test.glb"), "test", &image).unwrap();

        assert_eq!(texture.rgba8, [10, 20, 30, 255, 40, 50, 60, 255]);
    }

    #[test]
    fn missing_file_has_path_context() {
        let path = fixture_path().with_extension("missing.glb");

        let error = load_static_gltf(&path).unwrap_err();

        assert!(error.to_string().contains("BoxTextured.missing.glb"));
    }
}
