# Milestone 2B Checkpoint — Versioned Scene Persistence

## Outcome

The release sandbox loads `assets/scenes/orbit.scene.json` through the public runtime configuration, reconstructs its hierarchy and model reference, and renders the resulting scene. Scene documents have an explicit format version, validated data, deterministic save/load round trips, and path-preserving failures.

## Implemented

- `engine_scene` owns `SceneDocument`, `SceneEntity`, `ModelReference`, `save_scene`, and `load_scene` without depending on the renderer or platform.
- Scene format version 1 stores stable document IDs separately from generational runtime handles.
- Serialization preserves local transforms, parent links, and optional relative model references.
- Loading rejects unsupported versions, duplicate IDs, missing parents, cycles, unsafe model paths, non-finite transforms, non-positive scale, and zero quaternions.
- The runtime's `Simulation::from_scene` selects the first model entity and its parent (or itself when parentless) for the current rotating example.
- Startup loads and validates the scene, reconstructs the simulation, and loads its model before creating the renderer. No partial GPU renderer remains after these startup failures.
- `AppConfig` selects a scene path rather than a model path. The sandbox accepts an optional scene-path argument for deliberate application selection and startup-failure testing.
- The executable prints the full error chain instead of exposing only a debug representation of the top-level error.
- The orbit root is one unit above the origin so the rotating model remains visible throughout the example camera's orbit; this fixes an observed intermittent off-screen sample failure.

## Verified behavior

The scene crate passed 12 focused tests, including hierarchy/model round trips and actual file save/load. Runtime tests verify reconstruction and reject a scene with no model. The workspace passed 41 tests overall.

Actual release-executable checks passed:

- Textured rendering, resize, minimize/restore, and clean shutdown: PASS.
- Keyboard movement pixel change: 3.5% — PASS.
- Right-drag mouse-look pixel change: 3.8% — PASS.
- Unattended fixed-step scene pixel change: 3.2% — PASS.
- Missing model: nonzero exit with model path context and no renderer initialization — PASS.
- Unsupported scene version 999: nonzero exit with explicit version context and no renderer initialization — PASS.

The final rendering, input, and scene-change captures were inspected. They show the textured cube at coherent positions and orientations. Logs contain no runtime renderer initialization in either startup-failure case.

Evidence:

- `artifacts/render-smoke/`
- `artifacts/input-smoke/`
- `artifacts/scene-smoke/`
- `artifacts/startup-failure-smoke/`

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 41 tests total |
| `cargo build --workspace --release` | PASS |
| Python harness compilation | PASS |
| Real DX12 rendering, input, and scene change | PASS |
| Contextual startup failures before renderer initialization | PASS |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

## Current limitations

- Only the first model entity is rendered; M2C generalizes this path.
- The current simulation has one example rotation behavior rather than general gameplay components.
- The scene format is JSON only and has no migration framework beyond explicit version rejection.
- Model references are workspace-relative; there is no packaged asset root or cooked asset database yet.
- Saving through the interactive application belongs to M5.
- No physics, audio, debug inspection/editing, or second example application yet.
- Frame statistics are diagnostics, not benchmark results. No performance target has been established.
- Remote CI and owner acceptance have not been recorded.

## Next checkpoint

Generalize render extraction to all model-bearing entities, share loaded model resources by reference, and expose deliberate application scene controls through the public engine facade.
