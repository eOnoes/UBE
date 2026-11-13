use std::collections::HashSet;
use std::time::Duration;

use engine_physics::{CharacterMovement, PhysicsWorld};
use engine_platform::{Key, KeyInput};
use engine_render::Camera;

use crate::CameraMode;

const MOVE_SPEED_METERS_PER_SECOND: f32 = 3.0;
const MOUSE_SENSITIVITY_RADIANS_PER_PIXEL: f32 = 0.003;
const PITCH_LIMIT_RADIANS: f32 = 1.54;
const THIRD_PERSON_DISTANCE: f32 = 4.5;
const THIRD_PERSON_TARGET_HEIGHT: f32 = 0.35;
const TOP_DOWN_HEIGHT: f32 = 14.0;

/// Runtime-owned gameplay camera state and held-input tracking.
pub struct CameraController {
    camera: Camera,
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    mode: CameraMode,
    pressed_keys: HashSet<Key>,
    mouse_look_active: bool,
}

impl Default for CameraController {
    fn default() -> Self {
        Self::new(CameraMode::default())
    }
}

impl CameraController {
    pub fn new(mode: CameraMode) -> Self {
        let camera = Camera::default();
        let position = camera.position();
        let direction = normalize(subtract(camera.target(), position));
        let mut controller = Self {
            camera,
            position,
            yaw: direction[0].atan2(-direction[2]),
            pitch: direction[1].asin(),
            mode,
            pressed_keys: HashSet::new(),
            mouse_look_active: false,
        };
        controller.rebuild_camera();
        controller
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn cycle_camera_mode(&mut self) -> CameraMode {
        self.mode = self.mode.next();
        self.rebuild_camera();
        self.mode
    }

    pub fn handle_key(&mut self, input: KeyInput) {
        if input.pressed {
            self.pressed_keys.insert(input.key);
        } else {
            self.pressed_keys.remove(&input.key);
        }
    }

    pub fn set_mouse_look(&mut self, active: bool) {
        self.mouse_look_active = active;
    }

    pub fn handle_mouse_motion(&mut self, delta_x: f32, delta_y: f32) {
        if !self.mouse_look_active {
            return;
        }
        self.yaw += delta_x * MOUSE_SENSITIVITY_RADIANS_PER_PIXEL;
        self.pitch = (self.pitch - delta_y * MOUSE_SENSITIVITY_RADIANS_PER_PIXEL)
            .clamp(-PITCH_LIMIT_RADIANS, PITCH_LIMIT_RADIANS);
        self.rebuild_camera();
    }

    pub fn clear_input(&mut self) {
        self.pressed_keys.clear();
        self.mouse_look_active = false;
    }

    pub fn update(&mut self, elapsed: Duration, physics: &mut PhysicsWorld) -> CharacterMovement {
        let movement_yaw = match self.mode {
            CameraMode::TopDown => 0.0,
            CameraMode::FirstPerson | CameraMode::ThirdPerson => self.yaw,
        };
        let forward = [movement_yaw.sin(), 0.0, -movement_yaw.cos()];
        let right = [movement_yaw.cos(), 0.0, movement_yaw.sin()];
        let mut movement = [0.0; 3];
        for key in self.pressed_keys.iter().copied() {
            match key {
                Key::W => add_assign(&mut movement, forward),
                Key::S => subtract_assign(&mut movement, forward),
                Key::D => add_assign(&mut movement, right),
                Key::A => subtract_assign(&mut movement, right),
                _ => {}
            }
        }

        let length_squared = dot(movement, movement);
        let velocity = if length_squared > 0.0 {
            let scale = MOVE_SPEED_METERS_PER_SECOND / length_squared.sqrt();
            movement.map(|component| component * scale)
        } else {
            [0.0; 3]
        };
        let movement = physics.move_character(velocity, elapsed);
        self.position = movement.position;
        self.rebuild_camera();
        movement
    }

    fn rebuild_camera(&mut self) {
        self.camera = match self.mode {
            CameraMode::FirstPerson => {
                Camera::new(self.position, add(self.position, self.direction()))
            }
            CameraMode::ThirdPerson => {
                let target = add(self.position, [0.0, THIRD_PERSON_TARGET_HEIGHT, 0.0]);
                let offset = self
                    .direction()
                    .map(|component| component * THIRD_PERSON_DISTANCE);
                Camera::new(subtract(target, offset), target)
            }
            CameraMode::TopDown => Camera::new(
                add(self.position, [0.0, TOP_DOWN_HEIGHT, 0.01]),
                self.position,
            ),
        };
    }

    fn direction(&self) -> [f32; 3] {
        [
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        ]
    }

    pub(crate) fn position(&self) -> [f32; 3] {
        self.position
    }

    pub(crate) fn player_snapshot(&self) -> crate::PlayerSnapshot {
        crate::PlayerSnapshot {
            position: self.position,
            yaw: self.yaw,
            pitch: self.pitch,
            camera_mode: self.mode,
        }
    }

    pub(crate) fn respawn(&mut self, position: [f32; 3]) {
        self.position = position;
        self.clear_input();
        self.rebuild_camera();
    }

    #[cfg(test)]
    fn pitch(&self) -> f32 {
        self.pitch
    }
}

fn add(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn subtract(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn add_assign(target: &mut [f32; 3], value: [f32; 3]) {
    for axis in 0..3 {
        target[axis] += value[axis];
    }
}

fn subtract_assign(target: &mut [f32; 3], value: [f32; 3]) {
    for axis in 0..3 {
        target[axis] -= value[axis];
    }
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn normalize(value: [f32; 3]) -> [f32; 3] {
    let inverse_length = dot(value, value).sqrt().recip();
    value.map(|component| component * inverse_length)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: Key, pressed: bool) -> KeyInput {
        KeyInput { key, pressed }
    }

    fn grounded_physics() -> PhysicsWorld {
        let mut scene = engine_scene::Scene::new();
        let floor = scene.spawn(engine_scene::Transform::default());
        scene.set_collider(
            floor,
            Some(engine_scene::BoxCollider::new([20.0, 0.1, 20.0]).unwrap()),
        );
        PhysicsWorld::from_scene(&scene, CameraController::default().position())
    }

    #[test]
    fn diagonal_movement_is_normalized() {
        let mut forward = CameraController::default();
        let mut diagonal = CameraController::default();
        let start = forward.position();
        forward.handle_key(key(Key::W, true));
        diagonal.handle_key(key(Key::W, true));
        diagonal.handle_key(key(Key::D, true));

        let mut forward_physics = grounded_physics();
        let mut diagonal_physics = grounded_physics();
        forward.update(Duration::from_secs(1), &mut forward_physics);
        diagonal.update(Duration::from_secs(1), &mut diagonal_physics);

        let forward_distance = dot(
            subtract(forward.position(), start),
            subtract(forward.position(), start),
        )
        .sqrt();
        let diagonal_distance = dot(
            subtract(diagonal.position(), start),
            subtract(diagonal.position(), start),
        )
        .sqrt();
        assert!((forward_distance - MOVE_SPEED_METERS_PER_SECOND).abs() < 0.0001);
        assert!((diagonal_distance - MOVE_SPEED_METERS_PER_SECOND).abs() < 0.0001);
    }

    #[test]
    fn mouse_look_requires_button_and_clamps_pitch() {
        let mut controller = CameraController::default();
        let initial_pitch = controller.pitch();
        controller.handle_mouse_motion(0.0, 100.0);
        assert_eq!(controller.pitch(), initial_pitch);

        controller.set_mouse_look(true);
        controller.handle_mouse_motion(0.0, -100_000.0);
        assert_eq!(controller.pitch(), PITCH_LIMIT_RADIANS);
    }

    #[test]
    fn gameplay_cameras_have_distinct_positions_and_targets() {
        let first = CameraController::new(CameraMode::FirstPerson);
        assert_eq!(first.camera().position(), first.position());

        let third = CameraController::new(CameraMode::ThirdPerson);
        assert!(
            (third.camera().position()[2] - (third.position()[2] + THIRD_PERSON_DISTANCE)).abs()
                < 0.0001
        );
        assert!(
            (third.camera().target()[1] - (third.position()[1] + THIRD_PERSON_TARGET_HEIGHT)).abs()
                < 0.0001
        );

        let top_down = CameraController::new(CameraMode::TopDown);
        assert_eq!(top_down.camera().target(), top_down.position());
        assert!(
            (top_down.camera().position()[1] - (top_down.position()[1] + TOP_DOWN_HEIGHT)).abs()
                < 0.0001
        );
    }

    #[test]
    fn top_down_movement_uses_world_axes_and_camera_follows() {
        let mut controller = CameraController::new(CameraMode::TopDown);
        let initial_position = controller.position();
        controller.handle_mouse_motion(0.0, 0.0);
        controller.handle_key(key(Key::W, true));
        let mut physics = grounded_physics();
        controller.update(Duration::from_secs(1), &mut physics);

        assert!(controller.position()[2] < initial_position[2] - 2.9);
        assert!(
            (controller.camera().position()[1] - (controller.position()[1] + TOP_DOWN_HEIGHT))
                .abs()
                < 0.0001
        );
        assert_eq!(controller.camera().target(), controller.position());
    }

    #[test]
    fn focus_loss_clears_held_movement() {
        let mut controller = CameraController::default();
        controller.handle_key(key(Key::W, true));
        controller.set_mouse_look(true);
        controller.clear_input();
        let position = controller.position();

        let mut physics = grounded_physics();
        controller.update(Duration::from_secs(1), &mut physics);
        controller.handle_mouse_motion(100.0, 100.0);

        let settled = controller.position();
        assert!((settled[0] - position[0]).abs() < 0.0001);
        assert!((settled[2] - position[2]).abs() < 0.0001);
    }
}
