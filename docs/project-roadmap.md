# Unbiased Bob Engine Completion Roadmap

## Canonical goal

Finish and incrementally improve **Unbiased Bob Engine (UBE)**, a reusable Windows-first 3D engine built in Rust. The original M0–M7 first release is complete; its history is preserved below. The current public release is **nerd 0.1**, adding UBE identity and selectable gameplay cameras. Completion never means feature parity with Unreal Engine or silently including future capabilities.

The current developer outcome is a small 3D scene with first-person, third-person, and top-down cameras, movement and look, collision, audio, property inspection/editing, scene save/reload, and a Windows release build. A second example uses the same public engine API without special-case subsystem changes.

## Working agreement

- Continue through the roadmap in bounded implementation and verification slices.
- Keep subsystem ownership explicit and dependencies acyclic; add abstraction only for a demonstrated consumer.
- Maintain one canonical status document and a separate evidence checkpoint for each completed slice.
- Exercise real Windows behavior where applicable. Component tests alone do not establish end-to-end completion.
- Report checks as PASS, FAIL, or NOT RUN. Do not claim owner acceptance without review.
- Preserve deferred ideas rather than silently expanding scope.
- Remote CI remains NOT RUN until the project is actually pushed and its workflow executes.

## Definition of complete

All items below are required:

- [x] First-person scene with keyboard movement and mouse look.
- [x] Collision against designated scene geometry.
- [x] Static textured models rendered through DirectX 12.
- [x] Fixed-step gameplay and hierarchical scene transforms.
- [x] Versioned scene save/load with asset references and explicit failures.
- [x] Basic audio playback driven by gameplay or scene events.
- [x] Debug inspection and editing of selected scene properties.
- [x] A small, coherent application-facing engine API.
- [x] A second example proving reuse without special-case engine changes.
- [x] An agreed reference scene measured against the 60 FPS target, with hardware and method recorded.
- [x] Windows release build and end-to-end release verification.
- [x] Full regression suite and final limitations documented.

## Checkpoints

| Checkpoint | Observable outcome | Status | Evidence |
| --- | --- | --- | --- |
| M0 | Workspace, architecture, timing primitive, diagnostics, and native window | COMPLETE | `milestone-0-checkpoint.md` |
| M1A | DirectX 12 presentation and surface lifecycle | COMPLETE | `milestone-1a-checkpoint.md` |
| M1B | Indexed 3D geometry, camera, WGSL, and depth | COMPLETE | `milestone-1b-checkpoint.md` |
| M1C | Validated textured static glTF rendering | COMPLETE | `milestone-1c-checkpoint.md` |
| M1D | Interactive camera and frame diagnostics | COMPLETE | `milestone-1d-checkpoint.md` |
| M2A | Fixed-step scene updates, entity handles, and hierarchical transforms | COMPLETE | `milestone-2a-checkpoint.md` |
| M2B | Versioned scene save/load, model references, and failure behavior | COMPLETE | `milestone-2b-checkpoint.md` |
| M2C | General multi-entity render extraction and application scene controls | COMPLETE | `milestone-2c-checkpoint.md` |
| M3 | Physics-backed first-person collision and pointer capture | COMPLETE | `milestone-3-checkpoint.md` |
| M4 | Basic audio playback through an engine-owned subsystem | COMPLETE | `milestone-4-checkpoint.md` |
| M5 | Debug inspection, property editing, and interactive scene save/load | COMPLETE | `milestone-5-checkpoint.md` |
| M6 | Second example and public-API reusability proof | COMPLETE | `milestone-6-checkpoint.md` |
| M7 | Reference-scene performance, release hardening, and final acceptance evidence | COMPLETE | `milestone-7-checkpoint.md` |
| UBE-1 | UBE / nerd 0.1 identity and first-person, third-person, and top-down gameplay cameras | COMPLETE | `nerd-0.1-checkpoint.md` |
| NET-1 | Online multiplayer architecture and first playable network slice | NOT STARTED | Scope required before implementation |

## Current release

**UBE nerd 0.1 is implemented and locally verified.** The M0–M7 acceptance evidence remains intact; UBE-1 adds public product identity, release diagnostics, a configurable initial camera, and runtime cycling among all three views without duplicating player physics or movement state.

### Current verification

- Workspace formatting, all-target compilation, strict Clippy, **65 tests**, and release build: PASS.
- All **nine Windows smoke scripts**, including the new three-camera test: PASS.
- Fresh reference-scene benchmark: **119.966 aggregate FPS**, **119.925 minimum interval FPS**, and **8.336 ms weighted average frame time** over 1,201 frames and 10.011192 seconds at 1280×720 on the RTX 4080 SUPER.
- Architecture, controls, measured results, and limitations: `nerd-0.1-checkpoint.md`.
- Project license: MIT ACTIVE through the owner-provided root `LICENSE` (`Copyright (c) 2026 eOnoes`); all workspace packages inherit MIT. Future team contributor agreements remain OPEN. The verified Windows notice bundle is described in `licensing.md`.
- Remote CI: NOT RUN. Owner acceptance of this new slice: NOT RECORDED.

## Next major scope: online multiplayer

Online multiplayer is a recorded product requirement, but **NET-1 has not started and nerd 0.1 has no networking or multiplayer support**. Before coding, define server authority, transport and security, session flow, replicated state, prediction/reconciliation, testing strategy, and deployment. Do not infer multiplayer readiness from the existing modular crate boundaries.

## Other parked capabilities and limitations

Non-Windows support, skeletal animation, visual scripting, open-world streaming, advanced global illumination, engine-owned multithreading, custom physics/audio engines, and a polished general-purpose editor remain outside the current release.

There is no player avatar, third-person camera obstruction handling, or camera collision. F9 reload still replaces application update callbacks with default scene behavior. Additional limitations are recorded in the release checkpoints.

## Next action

Review UBE nerd 0.1, choose the project license and team ownership/contributor terms, then scope either the multiplayer milestone or the next small gameplay milestone explicitly.
