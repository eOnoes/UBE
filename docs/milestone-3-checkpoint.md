# Milestone 3 Checkpoint — First-Person Collision and Pointer Capture

## Outcome

The Windows release now has a grounded first-person character that moves through fixed simulation steps, collides with scene-designated static geometry, slides along walls, and uses deliberate relative-mouse capture. Collider data is part of the version-1 scene format and is available through the public application API.

## Implemented

- `engine_physics` owns Rapier static colliders, the broad phase, character shape, gravity, and kinematic character movement.
- Static box colliders are constructed from scene entities with composed world transforms; model rendering and collision remain independent.
- `BoxCollider` validates finite, positive half-extents. Scene loading revalidates deserialized collider data, and version-1 documents without colliders remain compatible.
- `BoxCollider` and `ColliderError` are re-exported through `engine_runtime` and the `engine` facade.
- The character uses a capsule with half-height `0.6` and radius `0.3`, gravity `-20.0 m/s²`, and a `40.0 m/s` maximum fall speed.
- Camera WASD velocity is normalized and advanced at the fixed simulation step. Q/E flying was removed.
- Right-click captures/confines the cursor and enables `DeviceEvent::MouseMotion` look. Releasing the button leaves capture active.
- Escape and focus loss clear held input and release capture. Captured cursor-move events are suppressed to avoid doubled deltas.
- The sandbox scene includes floor and wall colliders with transforms independent of the two rendered model entities.

## Verified behavior

- Physics tests cover falling to a grounded floor, forward wall stopping, and diagonal wall sliding: PASS.
- Scene persistence tests cover collider round-trip and rejection of invalid deserialized extents: PASS.
- Public-facade integration test attaches and reads a collider using only `engine`: PASS.
- Real Windows `SendInput` relative mouse look changed 7.1% of pixels: PASS.
- Right-click emitted capture twice; Escape and the native focus-loss event path each emitted release: PASS.
- Keyboard movement changed 6.9% of pixels: PASS.
- Actual release-executable wall contact stopped at `[1.5819, 0.9169499, 3.6996858]`; subsequent forward movement reached `[1.5821537, 0.9180928, 2.198467]`. X stayed bounded while Z decreased by approximately 1.50 m: PASS.
- Eye height remained between 0.85 m and 1.0 m during wall contact and sliding: PASS.
- Inspected render, mouse-look, wall-contact, and sliding captures show coherent textured geometry without presentation corruption: PASS.
- Unattended fixed-step scene pixel change: 10.3% — PASS.
- Resize, minimize/restore, frame diagnostics, clean shutdown, and no remaining sandbox process: PASS.
- Missing-model and unsupported-version startup failures remain contextual and occur before renderer initialization: PASS.

Evidence is under `artifacts/render-smoke/`, `artifacts/input-smoke/`, `artifacts/collision-smoke/`, `artifacts/scene-smoke/`, and `artifacts/startup-failure-smoke/`.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 49 tests total |
| `cargo build --workspace --release` | PASS |
| Python smoke-harness compilation | PASS |
| Actual DX12 rendering and Windows input/collision smoke tests | PASS |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

## Current limitations

- Collision supports static transformed boxes and one kinematic capsule; dynamic rigid bodies, mesh colliders, stepping, slopes, and jumping are not in this release.
- The default character shape and movement constants are engine-owned rather than scene-editable.
- Collider-only scene geometry is not visualized by the renderer.
- The smoke test injects the native focus-loss notification; manual Alt-Tab testing and owner acceptance remain unrecorded.
- Audio, debug inspection/editing, interactive scene save/load, and the second executable are not yet implemented.
- Logged frame statistics are diagnostics, not benchmark results. Remote CI remains NOT RUN.

## Next checkpoint

Add an engine-owned audio subsystem with validated clip loading, explicit gameplay-triggered playback, focused tests, and actual Windows output-device smoke evidence.
