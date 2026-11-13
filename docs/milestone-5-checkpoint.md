# Milestone 5 Checkpoint — Debug Inspection and Scene Editing

## Outcome

The Windows release now provides an on-screen, keyboard-driven scene inspector. A developer can select entities, edit local transforms, save the live scene to an application-owned workspace path, and reload it without terminating on missing or incompatible data. `egui` and its GPU integration remain inside the engine rather than leaking through the public application facade.

## Implemented

- F3 toggles the inspector and releases cursor capture. Tab cycles through live entities; initial selection prefers a model-bearing entity.
- Arrow keys change X/Z translation by 0.1; PageUp/PageDown change Y by 0.1. Brackets rotate around local Y by 15 degrees. Minus/equal multiply positive uniform scale by 1/1.1 or 1.1, clamped to 0.1–10.0.
- The overlay displays entity and parent identity, model reference, collider presence/extents, local translation, quaternion rotation, and uniform scale.
- F5 saves through the existing versioned scene serializer to the path configured by `AppConfig::with_workspace_scene_path`. The default is `artifacts/runtime/edited.scene.json`, leaving source assets untouched.
- F9 parses and validates the saved document, checks uploaded-model compatibility and renderer object capacity, then replaces simulation and reconstructs physics. Failures leave the current scene running and display contextual status.
- Edits also reconstruct physics, keeping designated collision geometry synchronized with the edited scene.
- `engine_render` owns egui texture upload, tessellation, buffers, and a color-target overlay pass after geometry. It consumes owned frame output and processes all texture deltas.
- The inspector embeds IBM Plex Mono under the included SIL Open Font License (`assets/fonts/OFL.txt`), so readable text does not depend on operating-system fonts.

## Verified behavior

- Selection, cycling, translation, Y rotation, bounded scaling, pressed-key gating, and hidden-inspector gating tests: PASS.
- Actual DX12 overlay visible with readable entity values and inspected captures: PASS.
- ArrowRight and PageUp edits saved translation `[1.6, 0.1, 0.0]`; right bracket saved Y rotation quaternion component `0.1305262`; equal saved scale `1.1`: PASS.
- A further edit followed by F9 and F5 restored the original edited entity transform. The animated parent is allowed to advance, so persistence is asserted for the edited entity rather than byte-identical whole-file output: PASS.
- Missing workspace scene and a valid document referencing an unuploaded model both reported nonfatal errors; valid reload afterward recovered successfully: PASS.
- Native Windows key messages explicitly include navigation scancodes and extended-key flags, avoiding synthetic-input ambiguity and foreground dependence: PASS twice consecutively.
- Rendering/resize/restore, keyboard and pointer capture, wall collision/sliding, fixed-step scene behavior, audio output/cadence, and startup failure regressions: PASS.
- Clean release shutdown, empty standard error, and process cleanup: PASS.

Evidence is under `artifacts/debug-smoke/`, with regression evidence under the existing render, input, collision, scene, audio, and startup-failure artifact directories.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 59 tests total |
| `cargo build --workspace --release` | PASS |
| Python smoke-script compilation | PASS |
| All seven Windows smoke scripts | PASS (inspector repeated twice after harness hardening) |
| Remote GitHub Actions | NOT RUN |
| Reference-scene performance benchmark | NOT RUN |

## Current limitations

- This is a debug inspector, not a mouse-driven or polished editor. It edits local transforms only and cannot create entities or assign new models/materials.
- Reload currently restores default scene behavior rather than the application's custom fixed-step update callback. Applications needing persistent custom behavior after reload must provide that capability in a future API revision.
- Runtime model uploads and object capacity are fixed at startup. Reload rejects unknown model paths and excess objects rather than uploading resources dynamically.
- The surface uses `Bgra8UnormSrgb`; egui warns that it prefers a non-sRGB target. The inspected text and scene output are coherent, and the warning is nonfatal.
- Logged frame statistics remain diagnostics, not benchmark results. Remote CI and owner acceptance remain unrecorded.

## Next checkpoint

Build a second executable with its own programmatic scene, fixed-step behavior, and workspace-save path using only the `engine` facade; prove release startup, rendering, and shutdown without special-case subsystem changes.
