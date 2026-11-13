use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{BoxCollider, Scene, SceneError, Transform};

pub const SCENE_FORMAT_VERSION: u32 = 1;

/// A validated relative path to a model asset.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModelReference(String);

impl ModelReference {
    pub fn new(path: impl Into<String>) -> Result<Self, SceneDataError> {
        let path = path.into();
        if path.trim().is_empty() {
            return Err(SceneDataError::Invalid(
                "model reference must not be empty".into(),
            ));
        }
        let parsed = Path::new(&path);
        if parsed.is_absolute()
            || parsed.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(SceneDataError::Invalid(format!(
                "model reference {path:?} must be a relative path without parent traversal"
            )));
        }
        Ok(Self(path))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable, serializable scene data independent of runtime slot generations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDocument {
    pub version: u32,
    pub entities: Vec<SceneEntity>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneEntity {
    pub id: u32,
    pub parent: Option<u32>,
    pub transform: Transform,
    pub model: Option<String>,
    #[serde(default)]
    pub collider: Option<BoxCollider>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum SceneDataError {
    #[error("unsupported scene format version {0}; expected {SCENE_FORMAT_VERSION}")]
    UnsupportedVersion(u32),
    #[error("invalid scene data: {0}")]
    Invalid(String),
    #[error("invalid scene hierarchy: {0}")]
    Hierarchy(#[from] SceneError),
}

#[derive(Debug, Error)]
pub enum SceneFileError {
    #[error("failed to read scene {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write scene {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse scene {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to validate scene {path}: {source}")]
    Data {
        path: PathBuf,
        #[source]
        source: SceneDataError,
    },
}

impl Scene {
    pub fn to_document(&self) -> SceneDocument {
        let entities = self.entities();
        let ids = entities
            .iter()
            .enumerate()
            .map(|(id, entity)| (*entity, id as u32))
            .collect::<HashMap<_, _>>();
        let entities = entities
            .into_iter()
            .map(|entity| {
                let node = self.node(entity).expect("live entity must have a node");
                SceneEntity {
                    id: ids[&entity],
                    parent: node.parent.map(|parent| ids[&parent]),
                    transform: node.local,
                    model: node.model.map(|model| model.as_str().to_owned()),
                    collider: node.collider,
                }
            })
            .collect();
        SceneDocument {
            version: SCENE_FORMAT_VERSION,
            entities,
        }
    }

    pub fn from_document(document: &SceneDocument) -> Result<Self, SceneDataError> {
        if document.version != SCENE_FORMAT_VERSION {
            return Err(SceneDataError::UnsupportedVersion(document.version));
        }

        let mut scene = Scene::new();
        let mut entities = HashMap::new();
        let mut unique_ids = HashSet::new();
        for saved in &document.entities {
            if !unique_ids.insert(saved.id) {
                return Err(SceneDataError::Invalid(format!(
                    "duplicate entity id {}",
                    saved.id
                )));
            }
            validate_transform(saved.transform)?;
            let entity = scene.spawn(saved.transform);
            if let Some(model) = &saved.model {
                let model = ModelReference::new(model.clone())?;
                assert!(scene.set_model(entity, Some(model)));
            }
            if let Some(collider) = saved.collider {
                let collider = BoxCollider::new(collider.half_extents())
                    .map_err(|error| SceneDataError::Invalid(error.to_string()))?;
                assert!(scene.set_collider(entity, Some(collider)));
            }
            entities.insert(saved.id, entity);
        }
        for saved in &document.entities {
            if let Some(parent_id) = saved.parent {
                let parent = entities.get(&parent_id).copied().ok_or_else(|| {
                    SceneDataError::Invalid(format!(
                        "entity {} references missing parent {parent_id}",
                        saved.id
                    ))
                })?;
                scene.set_parent(entities[&saved.id], Some(parent))?;
            }
        }
        Ok(scene)
    }
}

pub fn save_scene(path: impl AsRef<Path>, scene: &Scene) -> Result<(), SceneFileError> {
    let path = path.as_ref();
    let json = serde_json::to_string_pretty(&scene.to_document()).map_err(|source| {
        SceneFileError::Write {
            path: path.to_path_buf(),
            source: io::Error::new(io::ErrorKind::InvalidData, source),
        }
    })?;
    fs::write(path, format!("{json}\n")).map_err(|source| SceneFileError::Write {
        path: path.to_path_buf(),
        source,
    })
}

pub fn load_scene(path: impl AsRef<Path>) -> Result<Scene, SceneFileError> {
    let path = path.as_ref();
    let json = fs::read_to_string(path).map_err(|source| SceneFileError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    let document = serde_json::from_str(&json).map_err(|source| SceneFileError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    Scene::from_document(&document).map_err(|source| SceneFileError::Data {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_transform(transform: Transform) -> Result<(), SceneDataError> {
    if !transform
        .translation
        .iter()
        .chain(transform.rotation.iter())
        .all(|value| value.is_finite())
    {
        return Err(SceneDataError::Invalid(
            "transform contains a non-finite value".into(),
        ));
    }
    if !transform.scale.is_finite() || transform.scale <= 0.0 {
        return Err(SceneDataError::Invalid(
            "transform scale must be finite and positive".into(),
        ));
    }
    let rotation_length_squared = transform
        .rotation
        .iter()
        .map(|value| value * value)
        .sum::<f32>();
    if rotation_length_squared <= f32::EPSILON {
        return Err(SceneDataError::Invalid(
            "transform rotation must not be a zero quaternion".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_document() -> SceneDocument {
        SceneDocument {
            version: SCENE_FORMAT_VERSION,
            entities: vec![
                SceneEntity {
                    id: 10,
                    parent: None,
                    transform: Transform::default().with_translation([1.0, 2.0, 3.0]),
                    model: None,
                    collider: Some(BoxCollider::new([5.0, 0.1, 5.0]).unwrap()),
                },
                SceneEntity {
                    id: 20,
                    parent: Some(10),
                    transform: Transform::default().with_translation([1.5, 0.0, 0.0]),
                    model: Some("assets/models/BoxTextured.glb".into()),
                    collider: None,
                },
            ],
        }
    }

    #[test]
    fn document_round_trip_preserves_hierarchy_and_model() {
        let scene = Scene::from_document(&sample_document()).unwrap();
        let loaded = Scene::from_document(&scene.to_document()).unwrap();
        let model_entity = loaded
            .entities()
            .into_iter()
            .find(|entity| loaded.model(*entity).is_some())
            .unwrap();

        assert_eq!(
            loaded.model(model_entity).unwrap().as_str(),
            "assets/models/BoxTextured.glb"
        );
        let position = glam::Mat4::from_cols_array_2d(&loaded.world_matrix(model_entity).unwrap())
            .transform_point3(glam::Vec3::ZERO);
        assert!((position - glam::Vec3::new(2.5, 2.0, 3.0)).length() < 0.0001);
    }

    #[test]
    fn collider_round_trip_and_invalid_deserialized_extents_are_checked() {
        let scene = Scene::from_document(&sample_document()).unwrap();
        let loaded = Scene::from_document(&scene.to_document()).unwrap();
        let floor = loaded
            .entities()
            .into_iter()
            .find(|entity| loaded.collider(*entity).is_some())
            .unwrap();
        assert_eq!(
            loaded.collider(floor).unwrap().half_extents(),
            [5.0, 0.1, 5.0]
        );

        let mut json = serde_json::to_value(scene.to_document()).unwrap();
        json["entities"][0]["collider"]["half_extents"][1] = serde_json::Value::from(-0.1);
        let document: SceneDocument = serde_json::from_value(json).unwrap();
        let error = Scene::from_document(&document).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("half-extents must be finite and positive")
        );
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let mut document = sample_document();
        document.version += 1;

        assert_eq!(
            Scene::from_document(&document).unwrap_err(),
            SceneDataError::UnsupportedVersion(2)
        );
    }

    #[test]
    fn missing_parent_is_rejected() {
        let mut document = sample_document();
        document.entities[1].parent = Some(999);

        let error = Scene::from_document(&document).unwrap_err();

        assert!(error.to_string().contains("missing parent 999"));
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut document = sample_document();
        document.entities[1].id = 10;

        let error = Scene::from_document(&document).unwrap_err();

        assert!(error.to_string().contains("duplicate entity id 10"));
    }

    #[test]
    fn unsafe_model_path_is_rejected() {
        let error = ModelReference::new("../outside.glb").unwrap_err();

        assert!(error.to_string().contains("without parent traversal"));
    }

    #[test]
    fn invalid_transform_is_rejected() {
        let mut document = sample_document();
        document.entities[0].transform.scale = 0.0;

        let error = Scene::from_document(&document).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("scale must be finite and positive")
        );
    }

    #[test]
    fn file_round_trip_and_missing_file_have_path_context() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/test-scenes");
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("round-trip-{}.json", std::process::id()));
        let scene = Scene::from_document(&sample_document()).unwrap();

        save_scene(&path, &scene).unwrap();
        let loaded = load_scene(&path).unwrap();
        assert_eq!(loaded.to_document(), scene.to_document());

        let missing = directory.join("missing.scene.json");
        let error = load_scene(&missing).unwrap_err();
        assert!(error.to_string().contains("missing.scene.json"));
        fs::remove_file(path).unwrap();
    }
}
