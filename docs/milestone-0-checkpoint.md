# Milestone 0 Checkpoint

## Agreed outcome

A minimal Windows-first Rust engine foundation with documented boundaries, a public application API, a native window lifecycle, structured diagnostics, a tested fixed-step timing primitive, automated quality gates, and debug/release builds.

## Implemented and independently checked

- Five workspace members: `engine_core`, `engine_platform`, `engine_runtime`, `engine`, and `engine-sandbox`.
- The sandbox consumes the public `engine` facade rather than internal engine crates.
- Native window creation and close handling use `winit` 0.30.13.
- Runtime and platform errors cross engine-owned error boundaries.
- The fixed-step accumulator preserves fractional time and bounds catch-up work.
- Windows CI configuration covers formatting, checking, Clippy, tests, and release builds.
- Product, conventions, and architecture documents reflect the implemented foundation.

## Verification evidence

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS | Completed without differences. |
| `cargo check --workspace --all-targets` | PASS | All five workspace members checked. |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS | Completed without warnings. |
| `cargo test --workspace` | PASS | Seven fixed-step tests passed; all other test and doc-test targets passed. |
| `cargo build --workspace` | PASS | Debug sandbox executable produced. |
| `cargo build --workspace --release` | PASS | Release sandbox executable produced. |
| Windows window smoke test | PASS | Two consecutive release runs displayed `Engine Sandbox`, accepted a normal close request, and exited with code 0. |
| Runtime diagnostics | PASS | Logs showed startup, window creation, close request, event-loop exit, and runtime stop; stderr was empty. |
| Dependency inspection | PASS | Cargo dependency output confirmed the sandbox depends on `engine` and `tracing-subscriber`, not internal engine crates. |
| GitHub Actions execution | NOT RUN | CI is configured locally; no remote repository or workflow run exists. |
| Rendering, input behavior, physics, audio, scenes, editor | NOT RUN | These are outside Milestone 0. |

One initial test expectation was incorrect: it failed to include a 5 ms remainder from the preceding frame. The implementation was correct; the expected values were corrected and the complete suite was rerun successfully.

## Current limitations

- The workspace is not a Git repository, so no Git diff or commit exists.
- Rust is installed under the user's standard `.cargo/bin` directory, but that directory was not initially present in this shell's PATH.
- The fixed-step primitive is tested but is not yet connected to gameplay updates.
- The native event loop currently waits for events; continuous rendering and simulation scheduling belong to later milestones.
- No performance claim has been made or measured.
- Owner acceptance of Milestone 0 has not yet been recorded.

## Deferred ideas

Networking, additional platforms, skeletal animation, visual scripting, material graphs, streaming, terrain, advanced lighting, engine-owned multithreading, custom physics/audio engines, and a general-purpose editor remain parked under the product brief. No placeholder crates were created for them.

## Single next step

Begin Milestone 1 with a bounded rendering design spike: validate `wgpu` device initialization and DirectX 12 adapter selection on the reference machine, then define the smallest render-context boundary before integrating presentation into the existing window lifecycle.
