use std::error::Error;
use std::process::ExitCode;

use engine::{
    AppConfig, BoxCollider, ENGINE_NAME, ModelReference, RELEASE_VERSION, Scene, Transform,
};

const MODEL_PATH: &str = "assets/models/BoxTextured.glb";

fn gallery_scene() -> Scene {
    let mut scene = Scene::new();
    let model = ModelReference::new(MODEL_PATH).expect("the gallery model path is valid");
    let left = scene.spawn(Transform::default().with_translation([-1.4, 1.4, 0.0]));
    let right = scene.spawn(
        Transform::default()
            .with_translation([1.4, 1.1, -0.5])
            .with_scale(0.75),
    );
    assert!(scene.set_model(left, Some(model.clone())));
    assert!(scene.set_model(right, Some(model)));

    let floor = scene.spawn(Transform::default().with_translation([0.0, -0.1, 0.0]));
    assert!(scene.set_collider(floor, Some(BoxCollider::new([12.0, 0.1, 12.0]).unwrap())));
    let back_wall = scene.spawn(Transform::default().with_translation([0.0, 1.5, -2.0]));
    assert!(scene.set_collider(back_wall, Some(BoxCollider::new([6.0, 1.5, 0.1]).unwrap())));
    scene
}

fn main() -> ExitCode {
    tracing_subscriber::fmt().with_target(false).init();
    let scene = gallery_scene();
    let entities = scene.entities();
    let left = entities[0];
    let right = entities[1];
    let mut elapsed = 0.0_f32;
    let config = AppConfig::new(format!("{ENGINE_NAME} Gallery — {RELEASE_VERSION}"))
        .with_scene(scene)
        .with_scene_update(move |scene, step| {
            elapsed += step.as_secs_f32();
            let bob = (elapsed * 1.4).sin() * 0.35;
            assert!(
                scene.set_local_transform(
                    left,
                    Transform::default()
                        .with_translation([-1.4, 1.4 + bob, 0.0])
                        .with_rotation_y(elapsed * 0.9)
                )
            );
            assert!(
                scene.set_local_transform(
                    right,
                    Transform::default()
                        .with_translation([1.4, 1.1, -0.5])
                        .with_rotation_y(-elapsed * 0.45)
                        .with_scale(0.75)
                )
            );
        })
        .with_workspace_scene_path("artifacts/runtime/gallery.scene.json")
        .without_audio();

    if let Err(error) = engine::run(config) {
        eprintln!("error: {error}");
        let mut source = error.source();
        while let Some(error) = source {
            eprintln!("caused by: {error}");
            source = error.source();
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_owns_two_models_and_independent_collision_geometry() {
        let scene = gallery_scene();
        assert_eq!(scene.entities().len(), 4);
        assert_eq!(
            scene
                .entities()
                .into_iter()
                .filter(|entity| scene.model(*entity).is_some())
                .count(),
            2
        );
        assert_eq!(
            scene
                .entities()
                .into_iter()
                .filter(|entity| scene.collider(*entity).is_some())
                .count(),
            2
        );
        assert_ne!(
            scene.local_transform(scene.entities()[0]),
            scene.local_transform(scene.entities()[1])
        );
    }
}
