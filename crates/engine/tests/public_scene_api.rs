use engine::{AppConfig, BoxCollider, ModelReference, Scene, Transform};

#[test]
fn application_builds_and_controls_a_scene_through_the_facade() {
    let mut scene = Scene::new();
    let entity = scene.spawn(Transform::default());
    assert!(scene.set_model(
        entity,
        Some(ModelReference::new("assets/models/BoxTextured.glb").unwrap())
    ));
    scene.set_local_transform(
        entity,
        Transform::default().with_translation([1.0, 0.0, 0.0]),
    );
    let collider = BoxCollider::new([0.5, 0.5, 0.5]).unwrap();
    assert!(scene.set_collider(entity, Some(collider)));
    assert_eq!(scene.collider(entity), Some(collider));

    let config = AppConfig::new("Public API test")
        .with_scene(scene)
        .with_footstep_sound_path("assets/audio/footstep.wav")
        .with_scene_update(move |scene, _| {
            assert!(scene.set_local_transform(
                entity,
                Transform::default().with_translation([2.0, 0.0, 0.0])
            ));
        });

    drop(config);
}
