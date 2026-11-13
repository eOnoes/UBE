# Milestone 1C Checkpoint — Textured Static glTF

## Outcome

An application can select a static textured glTF asset through the public configuration API. The runtime loads and validates it outside the renderer, and the DirectX 12 backend draws its indexed primitives with base-color textures and directional lighting.

## Implemented

- A new `engine_assets` crate with no `wgpu` or platform dependency.
- `gltf` 1.4.1 import and decoding into engine-owned positions, normals, UV0 coordinates, `u32` indices, and RGBA8 texture bytes.
- Support for multiple textured triangle primitives per static model.
- Contextual import and validation errors for missing vertex attributes, indices, textures and images; non-finite vertex values; zero normals; invalid indices; empty textures; unsupported primitive and decoded-image formats; and byte-length inconsistencies.
- Application-configurable model paths through `AppConfig::with_model_path`.
- GPU vertex/index buffers, sRGB base-color textures, a linear sampler, and one material binding per primitive.
- Position/normal/UV vertex layout and WGSL texture sampling.
- A fixed directional light with ambient illumination, interpolated and normalized surface normals, and the existing camera and depth behavior.
- The unchanged Khronos `BoxTextured.glb` fixture, with Cesium attribution and source information under `assets/models/`.
- A textured smoke-test mode that checks model coverage, known green texture pixels, visual variation, resize, minimize/restore, and clean shutdown.

The downloaded GLB is 5,956 bytes with SHA-256 `b510eca2e2ef33f62f9ed57d6e7ce2d10ebb2bdebc4a8e59d347719ba81abdf4`.

## Verified behavior

The renderer selected `NVIDIA GeForce RTX 4080 SUPER` with backend `Dx12`, surface format `Bgra8UnormSrgb`, and one uploaded model primitive.

The final direct per-window release smoke test, `python tools/render_smoke.py --expect textured`, reported:

```text
Textured green pixels: 4730
Geometry coverage: 1.9%
Initial frame: size=(1264, 681), center=(76, 112, 31)
Textured green pixels: 3681
Geometry coverage: 2.3%
Resized frame: size=(944, 601), center=(76, 112, 31)
Textured green pixels: 3681
Geometry coverage: 2.3%
Restored frame: size=(944, 601), center=(76, 112, 31)
Clean shutdown: exit code 0
```

All three captures were inspected. They show the Cesium texture on three visible box faces, with distinct face illumination and correct perspective after resizing and restoring. Standard error was empty. Evidence is under `artifacts/render-smoke/`:

- `textured-1280.png`
- `textured-resized.png`
- `textured-restored.png`

The first two smoke attempts failed old whole-window coverage/variation thresholds designed for the larger built-in colored cube. The captured model was already correct; textured mode now uses thresholds appropriate to the smaller sample and separately checks texture-colored pixels.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 16 tests total |
| `cargo build --workspace --release` | PASS |
| `python -m py_compile tools/render_smoke.py tools/window_capture.py` | PASS |
| Actual DX12 textured rendering | PASS |
| Resize and minimize/restore | PASS |
| `engine_assets` dependency inspection | PASS — no graphics-backend dependency |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

Tests comprise five asset tests, seven fixed-step tests, three renderer tests, and one runtime-configuration test. The asset tests exercise the real GLB, RGB-to-RGBA conversion, malformed indices, non-finite vertices, and missing-file context.

## Current limitations

- The camera and light are static.
- Only base-color texture sampling is implemented; metallic/roughness, normal maps, emissive maps, alpha modes, material factors, and glTF sampler settings are not yet honored.
- Textures have no generated mipmaps.
- Only positions, normals, UV0, and indexed triangles are accepted; unindexed primitives, alternate UV sets, and higher-precision decoded image formats are rejected explicitly.
- Node transforms, scene hierarchies, animation, and skinning are not applied.
- The model is loaded once at renderer initialization; there is no asset cache, hot reload, or asynchronous streaming.
- No keyboard or mouse event forwarding or frame statistics yet.
- The fixed-step accumulator is not connected to gameplay updates.
- Remote CI has not run.
- Owner acceptance has not been recorded.

## Single next step

Forward keyboard and mouse input through engine-owned events, add a movable camera, and expose debug frame statistics without coupling input or timing to GPU internals.
