//! Physics-backed scene collision and first-person character movement.

use std::time::Duration;

use engine_scene::Scene;
use glam::Mat4;
use rapier3d::control::KinematicCharacterController;
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;

const PLAYER_RADIUS: f32 = 0.3;
const STANDING_HALF_HEIGHT: f32 = 0.6;
const CROUCHING_HALF_HEIGHT: f32 = 0.35;
const PRONE_HALF_HEIGHT: f32 = 0.05;
const GRAVITY: f32 = -20.0;
const MAX_FALL_SPEED: f32 = 40.0;

/// Collision and movement posture for a player character.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlayerStance {
    #[default]
    Standing,
    Crouching,
    Prone,
}

impl PlayerStance {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standing => "Standing",
            Self::Crouching => "Crouching",
            Self::Prone => "Prone",
        }
    }

    fn capsule(self) -> Capsule {
        let half_height = match self {
            Self::Standing => STANDING_HALF_HEIGHT,
            Self::Crouching => CROUCHING_HALF_HEIGHT,
            Self::Prone => PRONE_HALF_HEIGHT,
        };
        Capsule::new_y(half_height, PLAYER_RADIUS)
    }

    fn center_height(self) -> f32 {
        (match self {
            Self::Standing => STANDING_HALF_HEIGHT,
            Self::Crouching => CROUCHING_HALF_HEIGHT,
            Self::Prone => PRONE_HALF_HEIGHT,
        }) + PLAYER_RADIUS
    }
}

/// Result of a collision-aware character movement step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterMovement {
    pub position: [f32; 3],
    pub grounded: bool,
}

/// Owns static scene collision geometry and a kinematic first-person character.
pub struct PhysicsWorld {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
    controller: KinematicCharacterController,
    character_shape: Capsule,
    stance: PlayerStance,
    position: Vector,
    vertical_velocity: f32,
    grounded: bool,
    static_collider_count: usize,
}

impl PhysicsWorld {
    pub fn from_scene(scene: &Scene, eye_position: [f32; 3]) -> Self {
        let bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut static_collider_count = 0;
        for entity in scene.entities() {
            let Some(collider) = scene.collider(entity) else {
                continue;
            };
            let matrix = Mat4::from_cols_array_2d(
                &scene
                    .world_matrix(entity)
                    .expect("live collider entity must have a world transform"),
            );
            let (scale, rotation, translation) = matrix.to_scale_rotation_translation();
            let extents = collider.half_extents();
            let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
                rotation.w, rotation.x, rotation.y, rotation.z,
            ));
            let pose = Pose::from_parts(
                Vector::new(translation.x, translation.y, translation.z),
                rotation.into(),
            );
            colliders.insert(
                ColliderBuilder::cuboid(
                    extents[0] * scale.x,
                    extents[1] * scale.y,
                    extents[2] * scale.z,
                )
                .position(pose),
            );
            static_collider_count += 1;
        }

        let mut broad_phase = BroadPhaseBvh::new();
        let narrow_phase = NarrowPhase::new();
        let modified = colliders
            .iter()
            .map(|(handle, _)| handle)
            .collect::<Vec<_>>();
        broad_phase.update(
            &IntegrationParameters::default(),
            &colliders,
            &bodies,
            &modified,
            &[],
            &mut Vec::new(),
        );

        Self {
            bodies,
            colliders,
            broad_phase,
            narrow_phase,
            controller: KinematicCharacterController::default(),
            character_shape: PlayerStance::Standing.capsule(),
            stance: PlayerStance::Standing,
            position: Vector::from(eye_position),
            vertical_velocity: 0.0,
            grounded: false,
            static_collider_count,
        }
    }

    /// Changes capsule height while preserving foot position and rejecting occupied space.
    pub fn set_stance(&mut self, stance: PlayerStance) -> bool {
        if self.stance == stance {
            return true;
        }
        let target =
            self.position + Vector::Y * (stance.center_height() - self.stance.center_height());
        let shape = stance.capsule();
        let queries = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            QueryFilter::default(),
        );
        if queries
            .intersect_shape(Pose::from_translation(target + Vector::Y * 0.02), &shape)
            .next()
            .is_some()
        {
            return false;
        }
        self.character_shape = shape;
        self.stance = stance;
        self.position = target;
        self.vertical_velocity = self.vertical_velocity.min(0.0);
        true
    }

    /// Starts a standing jump. Returns false while airborne, crouching, or prone.
    pub fn jump(&mut self, speed: f32) -> bool {
        if !self.grounded || self.stance != PlayerStance::Standing || speed <= 0.0 {
            return false;
        }
        self.vertical_velocity = speed;
        self.grounded = false;
        true
    }

    pub fn grounded(&self) -> bool {
        self.grounded
    }

    pub fn stance(&self) -> PlayerStance {
        self.stance
    }

    pub fn move_character(
        &mut self,
        horizontal_velocity: [f32; 3],
        elapsed: Duration,
    ) -> CharacterMovement {
        let dt = elapsed.as_secs_f32();
        if self.grounded && self.vertical_velocity < 0.0 {
            self.vertical_velocity = 0.0;
        }
        self.vertical_velocity = (self.vertical_velocity + GRAVITY * dt).max(-MAX_FALL_SPEED);
        let desired = Vector::new(
            horizontal_velocity[0] * dt,
            self.vertical_velocity * dt,
            horizontal_velocity[2] * dt,
        );
        let queries = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.bodies,
            &self.colliders,
            QueryFilter::default(),
        );
        let movement = self.controller.move_shape(
            dt,
            &queries,
            &self.character_shape,
            &Pose::from_translation(self.position),
            desired,
            |_| {},
        );
        self.position += movement.translation;
        self.grounded = movement.grounded;
        if self.grounded {
            self.vertical_velocity = 0.0;
        }
        CharacterMovement {
            position: self.position.into(),
            grounded: self.grounded,
        }
    }

    pub fn position(&self) -> [f32; 3] {
        self.position.into()
    }

    pub fn static_collider_count(&self) -> usize {
        self.static_collider_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_scene::{BoxCollider, Transform};

    const STEP: Duration = Duration::from_nanos(16_666_667);

    fn collision_scene() -> Scene {
        let mut scene = Scene::new();
        let floor = scene.spawn(Transform::default().with_translation([0.0, -0.1, 0.0]));
        scene.set_collider(floor, Some(BoxCollider::new([10.0, 0.1, 10.0]).unwrap()));
        let wall = scene.spawn(Transform::default().with_translation([1.0, 1.0, 0.0]));
        scene.set_collider(wall, Some(BoxCollider::new([0.1, 1.0, 2.0]).unwrap()));
        scene
    }

    #[test]
    fn character_falls_onto_floor_and_stays_grounded() {
        let scene = collision_scene();
        let mut physics = PhysicsWorld::from_scene(&scene, [0.0, 2.0, 0.0]);
        assert_eq!(physics.static_collider_count(), 2);

        let mut movement = physics.move_character([0.0; 3], STEP);
        for _ in 0..120 {
            movement = physics.move_character([0.0; 3], STEP);
        }

        assert!(movement.grounded);
        assert!((movement.position[1] - 0.9).abs() < 0.03);
    }

    #[test]
    fn wall_stops_forward_movement_without_blocking_floor_collision() {
        let scene = collision_scene();
        let mut physics = PhysicsWorld::from_scene(&scene, [0.0, 0.91, 0.0]);
        for _ in 0..10 {
            physics.move_character([0.0; 3], STEP);
        }

        let mut movement = physics.move_character([3.0, 0.0, 0.0], STEP);
        for _ in 0..120 {
            movement = physics.move_character([3.0, 0.0, 0.0], STEP);
        }

        assert!(movement.grounded);
        assert!(movement.position[0] < 0.62, "{:?}", movement.position);
    }

    #[test]
    fn diagonal_motion_slides_along_wall() {
        let scene = collision_scene();
        let mut physics = PhysicsWorld::from_scene(&scene, [0.0, 0.91, -1.0]);
        for _ in 0..10 {
            physics.move_character([0.0; 3], STEP);
        }

        let mut movement = physics.move_character([3.0, 0.0, 2.0], STEP);
        for _ in 0..60 {
            movement = physics.move_character([3.0, 0.0, 2.0], STEP);
        }

        assert!(movement.position[0] < 0.62);
        assert!(movement.position[2] > -0.5);
    }

    #[test]
    fn stance_changes_preserve_feet_and_restore_standing_height() {
        let scene = collision_scene();
        let mut physics = PhysicsWorld::from_scene(&scene, [0.0, 0.91, -1.0]);
        for _ in 0..10 {
            physics.move_character([0.0; 3], STEP);
        }

        assert!(physics.set_stance(PlayerStance::Crouching));
        assert!((physics.position()[1] - 0.65).abs() < 0.03);
        assert!(physics.set_stance(PlayerStance::Prone));
        assert!((physics.position()[1] - 0.35).abs() < 0.03);
        assert!(physics.set_stance(PlayerStance::Standing));
        assert!((physics.position()[1] - 0.9).abs() < 0.03);
    }

    #[test]
    fn jumping_requires_standing_ground_contact() {
        let scene = collision_scene();
        let mut physics = PhysicsWorld::from_scene(&scene, [0.0, 0.91, -1.0]);
        for _ in 0..10 {
            physics.move_character([0.0; 3], STEP);
        }
        assert!(physics.set_stance(PlayerStance::Crouching));
        assert!(!physics.jump(7.0));
        assert!(physics.set_stance(PlayerStance::Standing));

        let start_height = physics.position()[1];
        assert!(physics.jump(7.0));
        let movement = physics.move_character([0.0; 3], Duration::from_millis(100));
        assert!(movement.position[1] > start_height + 0.4);
        for _ in 0..120 {
            physics.move_character([0.0; 3], STEP);
        }
        assert!(physics.grounded());
    }

    #[test]
    fn standing_is_rejected_under_a_low_ceiling() {
        let mut scene = collision_scene();
        let ceiling = scene.spawn(Transform::default().with_translation([0.0, 1.5, -1.0]));
        scene.set_collider(ceiling, Some(BoxCollider::new([2.0, 0.1, 2.0]).unwrap()));
        let mut physics = PhysicsWorld::from_scene(&scene, [0.0, 0.91, -1.0]);
        assert!(physics.set_stance(PlayerStance::Crouching));
        assert!(!physics.set_stance(PlayerStance::Standing));
        assert_eq!(physics.stance(), PlayerStance::Crouching);
    }
}
