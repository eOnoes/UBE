# UBE nerd 0.1 Checkpoint

## Outcome

**Unbiased Bob Engine (UBE), public release nerd 0.1**, is implemented and locally verified on Windows. This slice adds the agreed product identity and selectable first-person, third-person, and top-down gameplay cameras to the completed M0–M7 engine foundation.

`CameraMode` and the identity constants are re-exported through the public engine facade. Cargo's internal package version remains `0.1.0` because Cargo requires semver; `nerd 0.1` is the public release label.

Online multiplayer is a recorded next major requirement but is **not implemented**. There are no networking, remote-session, replication, prediction, or multiplayer claims in this checkpoint.

## Run and control

From the workspace root:

```powershell
cargo run --release -p engine-sandbox
cargo run --release -p engine-gallery
```

Built executables:

```text
target/release/engine-sandbox.exe
target/release/engine-gallery.exe
```

Press **V** to cycle `FirstPerson -> ThirdPerson -> TopDown -> FirstPerson`. Right-click captures the mouse; WASD moves the same physics-controlled character in all three views. Top-down movement uses fixed world axes; first/third-person movement is yaw-relative.

An application can select the initial view through the public API:

```rust
use engine::{AppConfig, CameraMode};

let config = AppConfig::new("My UBE Game")
    .with_camera_mode(CameraMode::ThirdPerson);
```

The default is first person. Existing movement, mouse look, collision, audio, inspector, and save/reload controls remain as documented in `milestone-7-checkpoint.md`.

## Verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — **65 tests** |
| `cargo build --workspace --release` | PASS |
| Python compilation of tools | PASS |
| Rendering, resize, minimize/restore smoke | PASS |
| Movement, mouse look, cursor capture/release/focus smoke | PASS |
| Collision and grounded character smoke | PASS |
| Fixed-step scene animation smoke | PASS |
| Audio initialization and playback-request smoke | PASS |
| Four contextual startup-failure cases | PASS |
| Inspector, editing, save/reload, and nonfatal failure smoke | PASS |
| Independent gallery animation/save and branding smoke | PASS |
| Three-camera cycle, follow-view movement, branding, and shutdown smoke | PASS |
| Remote CI | NOT RUN |
| Owner acceptance of this new slice | NOT RECORDED |

The camera smoke exercised the real release executable, captured all three modes, moved in third-person and top-down, returned to first person, checked all three mode-change logs and UBE/nerd 0.1 diagnostics, and exited with code 0. The inspected final captures contain coherent textured geometry. Evidence is in `artifacts/camera-smoke` and `artifacts/gallery-smoke`.

## Fresh nerd 0.1 reference benchmark

`python tools/reference_benchmark.py --output-dir artifacts/nerd-0.1-reference-benchmark` measured the actual UBE release executable at an exact 1280×720 client area on Windows 11, DirectX 12, `AutoVsync`, NVIDIA GeForce RTX 4080 SUPER. The idle character, hidden inspector, six-entity orbit scene, two rendered model instances, and fixed-step parent rotation match the frozen reference setup.

| Measure | Result |
| --- | ---: |
| Complete one-second intervals | 10 |
| Frames | 1,201 |
| Measured duration | 10.011192 seconds |
| Aggregate FPS | **119.966** |
| Median interval FPS | 119.969 |
| Minimum interval FPS | **119.925** |
| Maximum interval FPS | 119.995 |
| Weighted average frame time | **8.336 ms** |
| Worst recorded frame within reported intervals | 9.936 ms |
| 60 FPS target | PASS — aggregate and every complete interval exceed 60 FPS |

Executable SHA-256: `f1b7111988c9aa0848fde6a5f0ff6f484400a099f52f52b92d53b78c7894cb9e`.

These are application frame-loop intervals, not GPU timestamps or a percentile capture of every frame. Results apply only to this scene, machine, resolution, and configuration. Raw samples, full method, machine details, and capture are in `artifacts/nerd-0.1-reference-benchmark`.

## Identity and licensing

Both executable titles include “Unbiased Bob Engine” and “nerd 0.1.” Runtime startup logs include engine name, short name, release, title, and configured camera mode. Product, architecture, and roadmap documents use UBE identity while retaining internal Cargo executable names.

The project license and team/content-creator ownership and contributor agreements remain **undecided**. A dependency metadata inventory exists, but no legal-clearance or complete redistribution-notice claim is made. See `licensing.md`.

## Current limitations

- No multiplayer or networking implementation.
- No visible player avatar, third-person camera obstruction handling, or camera collision.
- Windows and DirectX 12 only; no installer, code signing, or remote CI result.
- Static textured glTF, fixed directional/ambient shading, static box collision, and a kinematic capsule.
- Keyboard-driven local-transform inspector rather than a general editor.
- F9 reload substitutes default scene behavior for application-provided update callbacks.
- The small reference scene does not establish performance for large worlds or complex games.

## Completion state

The UBE branding/camera slice is complete with local regression and benchmark evidence. Future multiplayer, camera, editor, and gameplay work should proceed as separately scoped checkpoints.
