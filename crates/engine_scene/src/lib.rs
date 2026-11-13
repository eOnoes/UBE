mod persistence;

pub use persistence::{
    ModelReference, SCENE_FORMAT_VERSION, SceneDataError, SceneDocument, SceneEntity,
    SceneFileError, load_scene, save_scene,
};

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A generational handle to an entity stored in a scene.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Entity {
    index: u32,
    generation: u32,
}

impl Entity {
    pub fn index(self) -> u32 {
        self.index
    }

    pub fn generation(self) -> u32 {
        self.generation
    }
}

/// Local translation, quaternion rotation (`x, y, z, w`), and positive uniform scale.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: f32,
}

impl Transform {
    pub fn with_translation(mut self, translation: [f32; 3]) -> Self {
        self.translation = translation;
        self
    }

    pub fn with_rotation_y(mut self, radians: f32) -> Self {
        self.rotation = Quat::from_rotation_y(radians).to_array();
        self
    }

    pub fn rotated_y(mut self, radians: f32) -> Self {
        self.rotation = (Quat::from_array(self.rotation) * Quat::from_rotation_y(radians))
            .normalize()
            .to_array();
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        assert!(
            scale.is_finite() && scale > 0.0,
            "scale must be finite and positive"
        );
        self.scale = scale;
        self
    }

    fn matrix(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::splat(self.scale),
            Quat::from_array(self.rotation).normalize(),
            Vec3::from(self.translation),
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: 1.0,
        }
    }
}

/// A validated static box collider attached to a scene entity.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoxCollider {
    half_extents: [f32; 3],
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
#[error("collider half-extents must be finite and positive")]
pub struct ColliderError;

impl BoxCollider {
    pub fn new(half_extents: [f32; 3]) -> Result<Self, ColliderError> {
        if !half_extents
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
        {
            return Err(ColliderError);
        }
        Ok(Self { half_extents })
    }

    pub fn half_extents(self) -> [f32; 3] {
        self.half_extents
    }
}

#[derive(Clone, Debug)]
struct Node {
    parent: Option<Entity>,
    local: Transform,
    model: Option<ModelReference>,
    collider: Option<BoxCollider>,
}

#[derive(Clone, Debug)]
struct Slot {
    generation: u32,
    node: Option<Node>,
}

/// Owns entity identity, local transforms, and acyclic parent/child relationships.
#[derive(Clone, Debug, Default)]
pub struct Scene {
    slots: Vec<Slot>,
    vacant: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SceneError {
    #[error("child entity does not exist")]
    MissingChild,
    #[error("parent entity does not exist")]
    MissingParent,
    #[error("parent assignment would create a cycle")]
    Cycle,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, local: Transform) -> Entity {
        if let Some(index) = self.vacant.pop() {
            let slot = &mut self.slots[index as usize];
            slot.node = Some(Node {
                parent: None,
                local,
                model: None,
                collider: None,
            });
            Entity {
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot {
                generation: 0,
                node: Some(Node {
                    parent: None,
                    local,
                    model: None,
                    collider: None,
                }),
            });
            Entity {
                index,
                generation: 0,
            }
        }
    }

    /// Removes an entity and detaches its direct children into root entities.
    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.contains(entity) {
            return false;
        }
        let children = self
            .slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                let node = slot.node.as_ref()?;
                (node.parent == Some(entity)).then_some(Entity {
                    index: index as u32,
                    generation: slot.generation,
                })
            })
            .collect::<Vec<_>>();
        for child in children {
            self.slots[child.index as usize]
                .node
                .as_mut()
                .unwrap()
                .parent = None;
        }

        let slot = &mut self.slots[entity.index as usize];
        slot.node = None;
        slot.generation = slot.generation.wrapping_add(1);
        self.vacant.push(entity.index);
        true
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.node(entity).is_some()
    }

    pub fn set_parent(&mut self, child: Entity, parent: Option<Entity>) -> Result<(), SceneError> {
        let Some(child_node) = self.node(child) else {
            return Err(SceneError::MissingChild);
        };
        if let Some(parent) = parent {
            if !self.contains(parent) {
                return Err(SceneError::MissingParent);
            }
            if child == parent || self.is_ancestor(child, parent) {
                return Err(SceneError::Cycle);
            }
        }
        self.slots[child.index as usize]
            .node
            .as_mut()
            .unwrap()
            .parent = parent;
        let _ = child_node;
        Ok(())
    }

    pub fn set_local_transform(&mut self, entity: Entity, local: Transform) -> bool {
        if let Some(node) = self.node_mut(entity) {
            node.local = local;
            true
        } else {
            false
        }
    }

    pub fn local_transform(&self, entity: Entity) -> Option<Transform> {
        self.node(entity).map(|node| node.local)
    }

    pub fn entities(&self) -> Vec<Entity> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| {
                slot.node.as_ref().map(|_| Entity {
                    index: index as u32,
                    generation: slot.generation,
                })
            })
            .collect()
    }

    pub fn parent(&self, entity: Entity) -> Option<Entity> {
        self.node(entity).and_then(|node| node.parent)
    }

    pub fn set_model(&mut self, entity: Entity, model: Option<ModelReference>) -> bool {
        if let Some(node) = self.node_mut(entity) {
            node.model = model;
            true
        } else {
            false
        }
    }

    pub fn model(&self, entity: Entity) -> Option<&ModelReference> {
        let slot = self.slots.get(entity.index as usize)?;
        if slot.generation != entity.generation {
            return None;
        }
        slot.node.as_ref()?.model.as_ref()
    }

    pub fn set_collider(&mut self, entity: Entity, collider: Option<BoxCollider>) -> bool {
        if let Some(node) = self.node_mut(entity) {
            node.collider = collider;
            true
        } else {
            false
        }
    }

    pub fn collider(&self, entity: Entity) -> Option<BoxCollider> {
        self.node(entity).and_then(|node| node.collider)
    }

    /// Computes a world matrix by composing the entity's valid ancestor chain.
    pub fn world_matrix(&self, entity: Entity) -> Option<[[f32; 4]; 4]> {
        let node = self.node(entity)?;
        let local = node.local.matrix();
        let world = match node.parent {
            Some(parent) => self.resolve_matrix(parent) * local,
            None => local,
        };
        Some(world.to_cols_array_2d())
    }

    fn resolve_matrix(&self, entity: Entity) -> Mat4 {
        let node = self
            .node(entity)
            .expect("valid parent links must reference live entities");
        let local = node.local.matrix();
        match node.parent {
            Some(parent) => self.resolve_matrix(parent) * local,
            None => local,
        }
    }

    fn is_ancestor(&self, possible_ancestor: Entity, mut descendant: Entity) -> bool {
        while let Some(parent) = self.node(descendant).and_then(|node| node.parent) {
            if parent == possible_ancestor {
                return true;
            }
            descendant = parent;
        }
        false
    }

    fn node(&self, entity: Entity) -> Option<Node> {
        let slot = self.slots.get(entity.index as usize)?;
        if slot.generation != entity.generation {
            return None;
        }
        slot.node.clone()
    }

    fn node_mut(&mut self, entity: Entity) -> Option<&mut Node> {
        let slot = self.slots.get_mut(entity.index as usize)?;
        if slot.generation != entity.generation {
            return None;
        }
        slot.node.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_transform_propagates_to_child() {
        let mut scene = Scene::new();
        let root = scene.spawn(Transform::default().with_translation([1.0, 2.0, 3.0]));
        let child = scene.spawn(Transform::default().with_translation([4.0, 0.0, 0.0]));
        scene.set_parent(child, Some(root)).unwrap();

        let world = Mat4::from_cols_array_2d(&scene.world_matrix(child).unwrap());
        let position = world.transform_point3(Vec3::ZERO);

        assert!((position - Vec3::new(5.0, 2.0, 3.0)).length() < 0.0001);
    }

    #[test]
    fn parent_rotation_affects_child_offset() {
        let mut scene = Scene::new();
        let root = scene.spawn(Transform::default().with_rotation_y(90.0_f32.to_radians()));
        let child = scene.spawn(Transform::default().with_translation([1.0, 0.0, 0.0]));
        scene.set_parent(child, Some(root)).unwrap();

        let world = Mat4::from_cols_array_2d(&scene.world_matrix(child).unwrap());
        let position = world.transform_point3(Vec3::ZERO);

        assert!(position.x.abs() < 0.0001);
        assert!((position.z + 1.0).abs() < 0.0001);
    }

    #[test]
    fn cyclic_parenting_is_rejected() {
        let mut scene = Scene::new();
        let root = scene.spawn(Transform::default());
        let child = scene.spawn(Transform::default());
        scene.set_parent(child, Some(root)).unwrap();

        assert_eq!(scene.set_parent(root, Some(child)), Err(SceneError::Cycle));
        assert_eq!(scene.set_parent(root, Some(root)), Err(SceneError::Cycle));
    }

    #[test]
    fn stale_handle_cannot_access_reused_slot() {
        let mut scene = Scene::new();
        let stale = scene.spawn(Transform::default());
        assert!(scene.despawn(stale));
        let replacement = scene.spawn(Transform::default());

        assert_eq!(stale.index(), replacement.index());
        assert_ne!(stale.generation(), replacement.generation());
        assert!(!scene.contains(stale));
        assert!(scene.world_matrix(stale).is_none());
    }

    #[test]
    fn despawning_parent_detaches_children() {
        let mut scene = Scene::new();
        let parent = scene.spawn(Transform::default().with_translation([10.0, 0.0, 0.0]));
        let child = scene.spawn(Transform::default().with_translation([1.0, 0.0, 0.0]));
        scene.set_parent(child, Some(parent)).unwrap();

        assert!(scene.despawn(parent));
        let world = Mat4::from_cols_array_2d(&scene.world_matrix(child).unwrap());
        let position = world.transform_point3(Vec3::ZERO);

        assert!((position - Vec3::X).length() < 0.0001);
    }
}
