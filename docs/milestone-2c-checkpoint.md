# Milestone 2C Checkpoint — Multi-Entity Rendering and Scene Controls

## Outcome

The release scene renders two independently transformed model entities while loading and uploading their shared glTF asset once. Applications can build a scene and supply fixed-step scene-update logic using only the public `engine` facade.

## Implemented

- `Simulation` extracts every model-bearing entity as a model path and world matrix.
- Runtime startup loads each unique model path once and maps scene references to renderer model indices.
- `Renderer` accepts multiple model assets and a slice of renderer-neutral `RenderItem` values.
- A dynamically indexed, alignment-aware uniform buffer supplies a camera matrix and independent model matrix for each draw item.
- All objects share the depth-aware textured geometry pipeline.
- `AppConfig::with_scene` accepts an application-built scene; `with_scene_update` supplies application logic after each fixed simulation step.
- The public facade exports scene identity, transforms, model references, persistence documents, and save/load functions.
- A consumer-side integration test compiles and links the application scene-construction and fixed-step-update API against `engine` only.
- `assets/scenes/orbit.scene.json` now has two child model entities at opposite offsets.

## Verified behavior

- Workspace tests: PASS — 44 unit tests plus one public-facade integration test.
- Release renderer log: `models=1`, `primitives=1`, `object_capacity=2`, confirming one shared model upload for two objects.
- Inspected captures show two textured cubes with independent perspective transforms and coherent depth.
- Unattended fixed-step pixel change: 7.1% — PASS.
- Keyboard movement pixel change: 8.0% — PASS.
- Right-drag mouse-look pixel change: 8.9% — PASS.
- Resize, minimize/restore, clean shutdown, frame diagnostics: PASS.
- Missing-model and unsupported-version startup failures remain contextual and occur before renderer initialization: PASS.

Evidence is under `artifacts/render-smoke/`, `artifacts/input-smoke/`, `artifacts/scene-smoke/`, and `artifacts/startup-failure-smoke/`.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 45 tests total |
| `cargo build --workspace --release` | PASS |
| Actual DX12 multi-object rendering and inspected captures | PASS |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

## Current limitations

- Model assets and object-uniform capacity are established at startup; runtime spawning does not yet upload new model paths.
- Transform matrices are extracted every frame; there is no dirty-change cache or instancing pass.
- The default scene update rotates the first model's parent; applications can replace it with their own fixed-step callback.
- Scene controls are startup configuration, not yet an interactive editor command stream.
- Camera movement remains collision-unaware and uses frame elapsed time until M3.
- No physics, pointer capture, audio, debug UI, or second executable example yet.
- Logged frame statistics are diagnostics, not benchmark results. Remote CI and owner acceptance remain unrecorded.

## Next checkpoint

Add Rapier-backed scene colliders and a first-person character controller, move camera movement into fixed simulation steps, and implement real pointer capture with Escape/focus-loss release.
