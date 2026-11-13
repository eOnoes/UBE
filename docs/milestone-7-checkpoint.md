# Milestone 7 Checkpoint — Reference Performance and First Release

## Outcome

The scoped Windows-first Rust engine release is implemented and locally verified end to end. It includes first-person movement/look, static collision geometry, textured DirectX 12 models, fixed-step scene behavior, versioned persistence, gameplay audio, keyboard-driven inspection/editing, interactive save/reload, and two public-API example executables. The agreed reference scene exceeds its measured 60 FPS target on the reference adapter.

This is the first release defined by `product-brief.md`, not feature parity with Unreal Engine. Owner acceptance has not yet been recorded.

## Run the release

From the workspace root:

```powershell
cargo run --release -p engine-sandbox
cargo run --release -p engine-gallery
```

Or run the already built executables:

```text
target/release/engine-sandbox.exe
target/release/engine-gallery.exe
```

Controls:

| Input | Behavior |
| --- | --- |
| W / A / S / D | Move the first-person character |
| Right-click | Capture cursor and enable relative mouse look |
| Escape | Release cursor and clear held movement |
| F3 | Toggle the inspector and release cursor capture |
| Tab | Select the next entity |
| Arrow keys | Edit local X/Z translation by 0.1 |
| PageUp / PageDown | Edit local Y translation by 0.1 |
| `[` / `]` | Rotate local Y by 15 degrees |
| `-` / `=` | Multiply uniform scale by 1/1.1 or 1.1, clamped to 0.1–10.0 |
| F5 | Save to the application-configured workspace scene |
| F9 | Reload the workspace scene, with nonfatal validation failures |

The sandbox uses `artifacts/runtime/edited.scene.json`; the gallery uses `artifacts/runtime/gallery.scene.json`. Source scene assets are not overwritten.

## Frozen reference scene

- Release `engine-sandbox.exe` loading `assets/scenes/orbit.scene.json`.
- Six scene entities: two rendered textured cube instances sharing one uploaded model and one primitive per instance, plus three static box colliders and one hierarchy root.
- Approximately 60 Hz fixed-step parent rotation, idle first-person character, physics active, inspector hidden.
- Exact 1280×720 client surface, `Bgra8UnormSrgb`, DirectX 12, `AutoVsync`.
- NVIDIA GeForce RTX 4080 SUPER on Windows 11. The result JSON records the reported OS, processor string, executable SHA-256, scene, and full method.

## Measured result

`python tools/reference_benchmark.py` captured evidence, allowed a three-second warm-up, then retained ten complete one-second engine frame-statistic intervals.

| Measure | Result |
| --- | ---: |
| Complete intervals | 10 |
| Frames | 1,200 |
| Measured duration | 10.0027 seconds |
| Aggregate FPS | 119.967 |
| Median interval FPS | 119.966 |
| Minimum interval FPS | 119.957 |
| Maximum interval FPS | 119.979 |
| Weighted average frame time | 8.336 ms |
| Worst recorded frame within reported intervals | 8.911 ms |
| 60 FPS target | PASS — aggregate and every sampled interval exceed 60 FPS |

These are application frame-loop intervals, not GPU timestamps or a percentile capture of every frame. The result applies only to this small scene, machine, resolution, and configuration; it is not a general engine-performance claim. Raw samples and machine details are in `artifacts/reference-benchmark/results.json`, and the inspected 1280×720 capture is `artifacts/reference-benchmark/reference-scene.png`.

## Final release verification

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 60 tests total |
| `cargo build --workspace --release` | PASS |
| Python compilation of all ten smoke/benchmark modules | PASS |
| Rendering, textured geometry, resize, minimize/restore | PASS |
| Keyboard movement, relative mouse look, capture/release/focus loss | PASS |
| Collision stop, wall slide, grounded eye height | PASS |
| Fixed-step scene animation | PASS |
| Real output initialization, idle silence, movement cadence, shutdown silence | PASS |
| Missing model, unsupported scene version, missing/invalid audio startup failures | PASS |
| Inspector visibility/readability, edits, save/reload, nonfatal failures | PASS |
| Second public-API example startup, animation, independent save, shutdown | PASS |
| Frozen-scene performance measurement | PASS |
| Clean application exit codes and standard error | PASS |
| Remote GitHub Actions | NOT RUN — no executed remote workflow is claimed |
| Owner acceptance | NOT RECORDED |

## Final limitations and deferred work

- Windows and DirectX 12 only; no installer, code signing, remote CI result, or portability claim.
- Static textured glTF models with fixed directional/ambient shading; no skeletal animation, material graph, dynamic lighting system, streaming, or advanced global illumination.
- Static box collision plus a kinematic capsule; no general dynamic rigid-body gameplay.
- Basic non-spatial WAV playback and configured movement footsteps; no mixer UI, music streaming, or positional attenuation. Tests verify device initialization and playback requests, not recorded audible output.
- Keyboard-driven local-transform inspector only; no entity creation, asset assignment, general editor layout, or mouse widget interaction.
- GPU model uploads and object capacity are fixed at startup. Reload rejects unknown model paths and excess objects instead of growing renderer resources.
- F9 reload replaces custom application update callbacks with default scene behavior. This affects both examples and is the highest-priority API limitation for a subsequent release.
- Engine-owned logic remains single-threaded. The reference scene is deliberately small and does not establish performance for large worlds or complex rendering.
- Networking, visual scripting, open-world systems, non-Windows support, and other capabilities in the roadmap parking lot remain deferred.

## Completion state

Implementation and local verification for M0–M7 are complete. The canonical roadmap records all acceptance items checked. The next action is owner review of the runnable release; future enhancements should be treated as a new scope rather than silently added to this release.
