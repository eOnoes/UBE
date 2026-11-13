# Milestone 2A Checkpoint — Fixed-Step Scene and Entity Transforms

## Outcome

The runtime advances a small scene through bounded fixed simulation steps. Generational entity handles and acyclic parent/child transforms live in a dedicated scene crate, and the renderer consumes an extracted world matrix rather than reading scene objects.

## Implemented

- A new `engine_scene` crate with no platform, renderer, or asset dependency.
- Generational `Entity` handles, slot reuse, stale-handle rejection, spawn/despawn, and local-transform access.
- Translation, quaternion rotation, and positive uniform scale in local `Transform` values.
- Parent/child relationships with cycle rejection and deterministic world-matrix composition.
- Parent despawn behavior that detaches direct children as roots rather than leaving invalid links.
- A runtime `Simulation` owner that uses the existing `FixedStep` accumulator at approximately 60 Hz with at most five catch-up steps per render frame.
- A rotating root entity and a child model entity offset by 1.5 units, demonstrating actual hierarchy propagation.
- Batched-versus-split time equivalence tests and fractional-frame no-update tests.
- Model transform extraction as a plain matrix; `engine_render` has no `engine_scene` dependency.
- A model matrix in the camera/frame uniform and WGSL world-position/normal transformation.
- A no-input scene smoke harness that compares real DX12 frames over time.

## Verified behavior

The final release scene test, `python tools/scene_smoke.py`, reported:

```text
Fixed-step scene pixel change: 3.2%
Clean shutdown: exit code 0
```

Both captures were inspected. They show the textured child model at different root-relative positions and orientations with coherent perspective and lighting, without any user input. Standard error was empty. Evidence is under `artifacts/scene-smoke/`:

- `scene-first.png`
- `scene-second.png`

The normal rendering and interactive-input smoke tests also passed after the transform integration:

- Textured rendering, resize, minimize/restore, and clean shutdown: PASS.
- Keyboard movement visual change: 3.5% — PASS.
- Right-drag mouse look visual change: 3.7% — PASS.
- Frame-statistics emission and clean shutdown: PASS.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 32 tests total |
| `cargo build --workspace --release` | PASS |
| Python harness compilation | PASS |
| Fixed-step scene visual change without input | PASS |
| Existing rendering and input regressions | PASS |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

Tests comprise five asset tests, ten core tests, two platform tests, three renderer tests, seven runtime tests, and five scene tests.

## Current limitations

- The public application API still runs the built-in simulation rather than exposing scene mutation to games.
- Only one model entity is rendered; general render extraction, multiple render items, instancing, and per-object materials remain future work.
- Transform extraction recomposes ancestor matrices on demand; there is no cached change-propagation system.
- Scale is uniform only, and normal transformation assumes that positive uniform-scale contract.
- No scene serialization, versioned save/load, asset references, prefabs, or component storage yet.
- Camera movement remains frame-time-based and collision-unaware; only scene behavior currently uses fixed steps.
- No physics, audio, editor, or second example application yet.
- Remote CI has not run.
- Owner acceptance has not been recorded.

## Single next step

Define versioned scene asset references and save/load behavior, then use them to reconstruct the current entity hierarchy and model assignment with explicit failure reporting.
