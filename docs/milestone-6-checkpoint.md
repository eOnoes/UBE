# Milestone 6 Checkpoint — Public API Reuse

## Outcome

A second executable, `engine-gallery`, constructs and runs a distinct scene using only the public `engine` facade. It proves that application-defined scenes and fixed-step behavior do not require sandbox branches or access to subsystem internals.

## Implemented

- Added `examples/gallery` as a workspace executable depending only on `engine` and process-wide `tracing-subscriber` configuration.
- The gallery programmatically creates two model-bearing entities with independent positions, scales, and Y rotations, plus separate floor and back-wall colliders.
- Its application-owned update callback rotates one cube, counter-rotates the other, and bobs the first cube at each fixed simulation step.
- `AppConfig::with_scene`, `with_scene_update`, `with_workspace_scene_path`, and `without_audio` supply its scene, behavior, persistence path, and deliberate audio choice through the public API.
- The independent workspace save is `artifacts/runtime/gallery.scene.json`; its window title is `Engine Gallery`.
- `tools/gallery_smoke.py` exercises the actual release executable rather than a component mock.

## Verified behavior

- Gallery construction test confirms four entities, exactly two model references, exactly two colliders, and distinct model transforms: PASS.
- Actual DirectX 12 startup logs `Engine Gallery`, `audio disabled`, two model slots, and completed frame statistics: PASS.
- Inspected initial and animated captures show two correctly textured models at different transforms: PASS.
- The distinct fixed-step behavior changed 6.2% of captured pixels between frames: PASS.
- F3/F5 saved a version 1 document containing all four entities and both model references to the gallery-specific workspace path: PASS.
- Clean release shutdown, exit code 0, empty standard error, and process cleanup: PASS.
- No internal engine crate appears in the gallery manifest and no special-case engine changes were made for this example: PASS by manifest and implementation inspection.

Evidence is under `artifacts/gallery-smoke/` and `artifacts/runtime/gallery.scene.json`.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 60 tests total |
| `cargo build --workspace --release` | PASS |
| Actual Windows gallery startup, animation, save, and shutdown | PASS |
| Remote GitHub Actions | NOT RUN |

## Current limitations

- The gallery intentionally reuses the validated textured cube asset; it is an API-reuse proof, not a second game or an asset-diversity benchmark.
- Reloading any example through F9 currently replaces its application update callback with default scene behavior, as recorded in milestone 5.
- Both examples share the same default first-person starting position and controller provided by the runtime.
- Owner acceptance and remote CI remain unrecorded.

## Next checkpoint

Freeze and measure the reference scene on the actual target adapter, then run the complete release regression suite and document final limitations and run commands.
