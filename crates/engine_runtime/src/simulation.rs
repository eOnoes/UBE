use std::time::Duration;

use engine_core::{FixedStep, StepOutcome};
use engine_scene::{ModelReference, Scene, Transform};
use thiserror::Error;

const SIMULATION_STEP: Duration = Duration::from_nanos(16_666_667);
const MAX_STEPS_PER_FRAME: u32 = 5;
const ROOT_ANGULAR_SPEED: f32 = 0.6;
const EXAMPLE_MODEL_PATH: &str = "assets/models/BoxTextured.glb";

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SimulationError {
    #[error("scene must contain an entity with a model reference")]
    NoModel,
}

/// Application-owned fixed-step scene logic.
pub type SceneUpdate = Box<dyn FnMut(&mut Scene, Duration) + 'static>;

/// A renderer-neutral model reference and world transform extracted from the scene.
#[derive(Clone, Copy, Debug)]
pub struct SceneModel<'a> {
    pub model_path: &'a str,
    pub transform: [[f32; 4]; 4],
}

/// Owns deterministic scene updates and render-transform extraction.
pub struct Simulation {
    scene: Scene,
    fixed_step: FixedStep,
    update: SceneUpdate,
}

impl Default for Simulation {
    fn default() -> Self {
        let mut scene = Scene::new();
        let root = scene.spawn(Transform::default().with_translation([0.0, 1.0, 0.0]));
        let model_entity = scene.spawn(Transform::default().with_translation([1.5, 0.0, 0.0]));
        scene
            .set_parent(model_entity, Some(root))
            .expect("a newly created root cannot form a cycle");
        let model = ModelReference::new(EXAMPLE_MODEL_PATH)
            .expect("the built-in example model path must be valid");
        assert!(scene.set_model(model_entity, Some(model)));
        Self::from_scene(scene, None).expect("the built-in example scene must contain a model")
    }
}

impl Simulation {
    pub fn from_scene(scene: Scene, update: Option<SceneUpdate>) -> Result<Self, SimulationError> {
        let model_entity = scene
            .entities()
            .into_iter()
            .find(|entity| scene.model(*entity).is_some())
            .ok_or(SimulationError::NoModel)?;
        let root = scene.parent(model_entity).unwrap_or(model_entity);
        let initial_root = scene
            .local_transform(root)
            .expect("the selected root must exist");
        let mut elapsed_simulation_time = 0.0_f32;
        let update = update.unwrap_or_else(|| {
            Box::new(move |scene, delta| {
                elapsed_simulation_time += delta.as_secs_f32();
                let transform = Transform {
                    rotation: Transform::default()
                        .with_rotation_y(elapsed_simulation_time * ROOT_ANGULAR_SPEED)
                        .rotation,
                    ..initial_root
                };
                assert!(scene.set_local_transform(root, transform));
            })
        });
        Ok(Self {
            scene,
            fixed_step: FixedStep::new(SIMULATION_STEP, MAX_STEPS_PER_FRAME),
            update,
        })
    }

    #[cfg(test)]
    pub fn advance(&mut self, elapsed: Duration) -> StepOutcome {
        self.advance_with(elapsed, |_, _| {})
    }

    pub fn advance_with(
        &mut self,
        elapsed: Duration,
        mut before_scene_update: impl FnMut(Duration, &mut Scene),
    ) -> StepOutcome {
        let outcome = self.fixed_step.advance(elapsed);
        for _ in 0..outcome.steps {
            before_scene_update(SIMULATION_STEP, &mut self.scene);
            (self.update)(&mut self.scene, SIMULATION_STEP);
        }
        outcome
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn model_transforms(&self) -> Vec<SceneModel<'_>> {
        self.scene
            .entities()
            .into_iter()
            .filter_map(|entity| {
                let model_path = self.scene.model(entity)?.as_str();
                let transform = self
                    .scene
                    .world_matrix(entity)
                    .expect("a live model entity must have a world transform");
                Some(SceneModel {
                    model_path,
                    transform,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconstructs_hierarchy_and_model_reference_from_scene() {
        let mut scene = Scene::new();
        let root = scene.spawn(Transform::default().with_translation([2.0, 0.0, 0.0]));
        let model_entity = scene.spawn(Transform::default().with_translation([1.5, 0.0, 0.0]));
        scene.set_parent(model_entity, Some(root)).unwrap();
        scene.set_model(
            model_entity,
            Some(ModelReference::new(EXAMPLE_MODEL_PATH).unwrap()),
        );

        let simulation = Simulation::from_scene(scene, None).unwrap();
        let models = simulation.model_transforms();
        let matrix = models[0].transform;

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_path, EXAMPLE_MODEL_PATH);
        assert!((matrix[3][0] - 3.5).abs() < 0.0001);
        assert!(matrix[3][1].abs() < 0.0001);
        assert!(matrix[3][2].abs() < 0.0001);
    }

    #[test]
    fn extracts_every_model_with_its_own_transform() {
        let mut scene = Scene::new();
        let first = scene.spawn(Transform::default().with_translation([1.0, 0.0, 0.0]));
        let second = scene.spawn(Transform::default().with_translation([-1.0, 0.0, 0.0]));
        let model = ModelReference::new(EXAMPLE_MODEL_PATH).unwrap();
        scene.set_model(first, Some(model.clone()));
        scene.set_model(second, Some(model));

        let simulation = Simulation::from_scene(scene, None).unwrap();
        let models = simulation.model_transforms();

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].transform[3][0], 1.0);
        assert_eq!(models[1].transform[3][0], -1.0);
    }

    #[test]
    fn application_update_controls_the_selected_entity_at_fixed_steps() {
        let mut scene = Scene::new();
        let model_entity = scene.spawn(Transform::default());
        scene.set_model(
            model_entity,
            Some(ModelReference::new(EXAMPLE_MODEL_PATH).unwrap()),
        );
        let mut update_steps = 0;
        let update = Box::new(move |scene: &mut Scene, _| {
            update_steps += 1;
            let transform = Transform::default().with_translation([update_steps as f32, 0.0, 0.0]);
            assert!(scene.set_local_transform(model_entity, transform));
        });
        let mut simulation = Simulation::from_scene(scene, Some(update)).unwrap();

        let outcome = simulation.advance(SIMULATION_STEP * 3);

        assert_eq!(outcome.steps, 3);
        assert_eq!(simulation.model_transforms()[0].transform[3][0], 3.0);
    }

    #[test]
    fn rejects_a_scene_without_a_model() {
        let mut scene = Scene::new();
        scene.spawn(Transform::default());

        assert!(matches!(
            Simulation::from_scene(scene, None),
            Err(SimulationError::NoModel)
        ));
    }

    #[test]
    fn fixed_steps_change_the_child_world_transform() {
        let mut simulation = Simulation::default();
        let initial = simulation.model_transforms()[0].transform;

        let outcome = simulation.advance(SIMULATION_STEP * 2);

        assert_eq!(outcome.steps, 2);
        assert_ne!(simulation.model_transforms()[0].transform, initial);
    }

    #[test]
    fn batched_and_split_time_produce_the_same_transform() {
        let mut batched = Simulation::default();
        let mut split = Simulation::default();

        batched.advance(SIMULATION_STEP * 4);
        for _ in 0..4 {
            split.advance(SIMULATION_STEP);
        }

        assert_eq!(
            batched.model_transforms()[0].transform,
            split.model_transforms()[0].transform
        );
    }

    #[test]
    fn fractional_time_does_not_advance_the_scene() {
        let mut simulation = Simulation::default();
        let initial = simulation.model_transforms()[0].transform;

        let outcome = simulation.advance(SIMULATION_STEP / 2);

        assert_eq!(outcome.steps, 0);
        assert_eq!(simulation.model_transforms()[0].transform, initial);
    }
}
