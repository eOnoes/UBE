# Milestone 1A Checkpoint — DirectX Presentation

## Outcome

A modular DirectX 12 rendering foundation that presents real frames through the existing public engine API while keeping native event handling and GPU ownership in separate crates.

## Implemented

- `wgpu` 30.0.1 with only the standard, DX12, and WGSL features needed by the project.
- High-performance adapter selection and a configured vsync presentation surface.
- Engine-owned platform events for resume, close, resize, redraw, idle, and exit.
- An owned, cloneable native window contract based on raw window handles.
- Runtime-owned renderer initialization and shutdown.
- A clear-frame command encoder and render pass.
- Resize handling, zero-size suspension, and redraw scheduling.
- Recovery for lost, outdated, and suboptimal surfaces; timeout and occlusion handling; fatal validation reporting.
- A repeatable Windows smoke harness that captures actual client pixels and exercises resize and minimize/restore behavior.

## Verified behavior

The renderer selected `NVIDIA GeForce RTX 4080 SUPER` with backend `Dx12` and surface format `Bgra8UnormSrgb`.

The final release smoke test reported:

```text
Initial frame: size=(1264, 681), center=(63, 108, 149)
Resized frame: size=(944, 601), center=(63, 108, 149)
Restored frame: size=(944, 601), center=(63, 108, 149)
Clean shutdown: exit code 0
```

The captured images were inspected and contain the expected uniform blue clear frame. Standard error was empty. Evidence is under `artifacts/render-smoke/`.

## Quality checks

| Check | Result |
| --- | --- |
| Formatting | PASS |
| Workspace compilation | PASS |
| Clippy with warnings denied | PASS |
| Tests | PASS — seven fixed-step tests plus all other targets |
| Release build | PASS |
| Actual DX12 presentation | PASS |
| Resize and restore | PASS |
| Remote GitHub Actions | NOT RUN |

The implementation was adapted to the installed `wgpu` 30 API rather than pinning an older release. Two capture-harness failures were caused by overlapping desktop windows, not rendering defects; the harness now makes the test window topmost during captures.

## Current limitations

- No geometry, shaders, camera, depth buffer, textures, or model loading yet.
- No keyboard or mouse event forwarding yet.
- The fixed-step accumulator is not connected to gameplay updates.
- No frame-time statistics or performance claim.
- Device-loss diagnostics beyond surface statuses remain a later hardening task.
- Owner acceptance has not been recorded.

## Single next step

Add a small WGSL graphics pipeline with vertex/index buffers, a depth buffer, and a camera uniform, then render a known 3D shape before introducing glTF assets.
