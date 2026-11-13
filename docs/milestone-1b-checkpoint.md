# Milestone 1B Checkpoint — Indexed 3D Geometry

## Outcome

The existing DirectX 12 renderer draws a correctly projected, depth-tested colored cube through the engine's normal runtime and public sandbox entry point. This establishes the camera, mesh, pipeline, and depth-resource boundaries before external assets are introduced.

## Implemented

- An engine-owned right-handed, Y-up perspective camera with a 45-degree vertical field of view, `0.1` near plane, and `100.0` far plane.
- Aspect-ratio-aware view-projection uniforms using `glam` 0.34 and `bytemuck` for explicit GPU-compatible data layout.
- A `Mesh` contract that owns its vertex buffer, index buffer, index count, and index format.
- A built-in cube containing 24 position/color vertices and 36 `u32` indices, with distinct face vertices suitable for later normals and texture coordinates.
- A WGSL vertex/fragment pipeline, indexed draw calls, and a `Depth32Float` depth texture.
- Depth-resource recreation whenever a nonempty surface is resized or reconfigured.
- Camera-uniform updates for each rendered frame while retaining the existing presentation recovery behavior.

## Verified behavior

The renderer selected `NVIDIA GeForce RTX 4080 SUPER` with backend `Dx12` and surface format `Bgra8UnormSrgb`.

The final direct per-window release smoke test, `python tools/render_smoke.py --expect geometry`, reported:

```text
Geometry coverage: 8.6%
Initial frame: size=(1264, 681), center=(237, 124, 124)
Geometry coverage: 10.2%
Resized frame: size=(944, 601), center=(237, 124, 124)
Geometry coverage: 10.2%
Restored frame: size=(944, 601), center=(237, 124, 124)
Clean shutdown: exit code 0
```

The captured images were inspected and show a correctly projected colored cube with three visible faces on the blue background. Direct `PrintWindow` capture avoids interference from overlapping desktop windows. Evidence is under `artifacts/render-smoke/`:

- `geometry-1280.png`
- `geometry-resized.png`
- `geometry-restored.png`

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — seven fixed-step tests and three renderer tests |
| `cargo build --workspace --release` | PASS |
| `python -m py_compile tools/render_smoke.py tools/window_capture.py` | PASS |
| Actual DX12 indexed rendering | PASS |
| Resize and minimize/restore | PASS |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

Renderer tests cover finite default-camera matrices, aspect-ratio-dependent projection, and cube vertex/index validation.

## Current limitations

- The camera is static.
- No textures, model loading, or directional lighting yet.
- No keyboard or mouse event forwarding yet.
- No frame-time statistics or performance claim.
- The fixed-step accumulator is not connected to gameplay updates.
- Device-loss diagnostics beyond surface statuses remain a later hardening task.
- Remote CI has not run.
- Owner acceptance has not been recorded.

## Single next step

Add a neutral `engine_assets` crate for validated static textured glTF data, then render a known textured model with UVs, normals, a texture/sampler binding, and directional lighting without moving file loading into the renderer.
