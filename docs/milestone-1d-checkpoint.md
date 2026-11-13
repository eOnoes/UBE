# Milestone 1D Checkpoint — Interactive Camera and Frame Diagnostics

## Outcome

A user can move the camera through the textured scene using the keyboard and right-drag mouse look. Native input remains behind engine-owned events, and the runtime emits aggregate frame diagnostics without exposing renderer internals.

## Controls

- `W` / `S`: move forward / backward along the horizontal view direction.
- `A` / `D`: strafe left / right.
- `Q` / `E`: move down / up.
- Hold the right mouse button and move the cursor: look around.
- Release the right button: stop looking around.

## Implemented

- Physical-key mapping in `engine_platform` for the six movement keys.
- Engine-owned keyboard, right-mouse-button, cursor-motion, and focus events; no `winit` input types leak into the runtime.
- Window-relative cursor deltas for right-drag look, with first-position and focus-loss handling.
- Runtime-owned yaw/pitch camera state, clamped pitch, normalized diagonal movement, and frame-time-based movement speed.
- A 100 ms cap on camera movement elapsed time to avoid large jumps after long frames.
- Held-input clearing on focus loss.
- Renderer-independent `FrameStats` in `engine_core`, reporting frame count, FPS, average frame time, and maximum frame time once per interval.
- Frame-statistics logging through the existing diagnostics system; no FPS assertion or benchmark claim.
- A Windows input smoke harness that sends native key and mouse window messages, captures actual DX12 client pixels, verifies visible changes from movement and look, checks diagnostics, and closes the application.

Raw pointer-lock mouse input was not included in this slice. Window-relative right-drag is the current contract; raw relative motion belongs with the later first-person pointer-capture work.

## Verified behavior

The final release input test, `python tools/input_smoke.py`, reported:

```text
Keyboard movement pixel change: 4.2%
Mouse look pixel change: 4.8%
Clean shutdown: exit code 0
Frame statistics: emitted
```

The initial, moved, and looked captures were inspected. They show the textured model shifting correctly after strafing and changing orientation after right-drag look while remaining properly lit and depth tested. Standard error was empty. Evidence is under `artifacts/input-smoke/`:

- `input-initial.png`
- `input-moved.png`
- `input-looked.png`
- `stdout.log`

The normal textured rendering smoke test also passed after input integration, including resize, minimize/restore, and clean shutdown.

Early input-harness attempts established that posted keyboard input worked but injected physical mouse events were not delivered to the sandbox's foreground desktop. The engine now uses window-relative cursor messages for this milestone, and the harness exercises that actual native event path. No rendering assertion was weakened to make mouse look pass.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 24 tests total |
| `cargo build --workspace --release` | PASS |
| Python harness compilation | PASS |
| Textured rendering, resize, and restore | PASS |
| Actual keyboard-driven visual change | PASS |
| Actual mouse-driven visual change | PASS |
| Frame-statistics emission | PASS |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

Tests comprise five asset tests, ten core timing/statistics tests, two platform-input tests, three renderer tests, and four runtime tests.

## Current limitations

- No pointer lock, cursor hiding, raw mouse capture, input remapping, gamepad support, or application-facing input API yet.
- Camera movement is not collision-aware.
- Frame statistics are logs, not an on-screen debug overlay.
- Camera movement uses frame elapsed time; the fixed-step accumulator is still not connected to gameplay updates.
- The scene remains one static textured model with a fixed directional light.
- Remote CI has not run.
- Owner acceptance has not been recorded.

## Single next step

Begin Milestone 2 by connecting fixed-step updates to runtime behavior and introducing explicit entity/transform data, while keeping render extraction separate from gameplay state.
