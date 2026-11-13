# Engine Architecture

## Architectural objective

The engine should be replaceable at meaningful subsystem boundaries without requiring a universal abstraction layer. Modularity comes from clear ownership, acyclic dependencies, explicit data flow, and consumers that use the public API.

## Current crate graph

Arrows mean "depends on":

```text
engine-sandbox ---+
                  +--> engine -> engine_runtime -> engine_core
engine-gallery ---+                         |
                                            +----> engine_platform
                                            |
                                            +----> engine_assets
                                            |
                                            +----> engine_scene
                                            |
                                            +----> engine_physics -> engine_scene
                                            |
                                            +----> engine_audio
                                            |
                                            +----> engine_render -> engine_platform
                                                     |
                                                     +----> engine_assets
```

- `engine_core` contains fixed-step timing, renderer-independent frame statistics, and canonical product identity: `ENGINE_NAME`, `ENGINE_SHORT_NAME`, `RELEASE_VERSION`, and Cargo-compatible `CARGO_PACKAGE_VERSION`. It must not depend on another engine crate.
- `engine_platform` owns native window and operating-system event integration. It emits engine-owned events, maps physical keys including V, exposes relative pointer deltas, and owns cursor-capture state without exposing winit to the runtime.
- `engine_assets` loads and validates CPU-side static model data. It has no graphics-backend dependency and performs no GPU resource creation.
- `engine_scene` owns generational entity identity, local transforms, validated static box colliders, acyclic parent/child relationships, and versioned scene-document persistence. Stable document IDs are separate from runtime handles. Model references are validated relative strings; this crate does not load assets or depend on the renderer or platform.
- `engine_physics` owns Rapier static collision geometry, gravity, and the kinematic capsule controller. It reads scene collider data and returns corrected character positions and grounded state without depending on the platform, renderer, or runtime.
- `engine_audio` owns decoded WAV clips, the real output stream, mixer voices, and playback requests. Rodio types do not cross this boundary.
- `engine_render` owns the DirectX 12 device, queue, presentation surface, per-model GPU resources, object transforms, depth, egui overlay resources, and frame execution. It consumes neutral asset data and extracted render items, not scene objects or asset file I/O. Egui stays internal to `engine_render` and `engine_runtime`.
- `engine_runtime` owns application configuration, gameplay camera behavior, asset loading, fixed-step simulation, multi-object extraction, character/audio coordination, debug selection/editing, interactive save/reload, and subsystem lifecycle.
- `engine` is the public facade consumed by applications. It re-exports only deliberate application-facing contracts, including identity constants and `CameraMode`.
- `engine-sandbox` and `engine-gallery` are separate executables that exercise the same public facade with different scenes, update callbacks, audio choices, and workspace-save paths. Their window titles use the shared UBE and nerd 0.1 identity.

Cargo's acyclic crate graph is part of the architecture enforcement mechanism. Internal path dependencies must not be introduced merely to work around an ownership problem.

## Intended subsystem growth

Future crates are added only when their milestone begins:

```text
engine -> engine_runtime +-> engine_physics
                         +-> engine_audio

engine_editor -> public engine and explicitly supported tooling APIs
```

This diagram expresses ownership intent, not permission for arbitrary many-to-many dependencies. Each new subsystem must record its actual inputs, outputs, and dependency direction before implementation. Online multiplayer is a future approved product direction, not an existing subsystem; its transport, authority, replication, and session boundaries must be designed and reviewed before adding crates.

## Runtime and data flow

The current application lifecycle is:

1. The executable configures diagnostics and supplies `AppConfig`: either a versioned scene path or an application-built scene, plus optional fixed-step behavior, audio, title, workspace-save path, and `with_camera_mode(CameraMode)`. It calls `engine::run`. The initial camera defaults to first person.
2. The runtime initializes the native event loop with an event handler and logs the shared engine name, short name, release label, window title, and configured camera mode.
3. The platform creates an owned native window and emits `Resumed`.
4. The runtime validates the selected scene's stable document IDs, hierarchy, transforms, model references, and colliders, then reconstructs generational runtime entities and physics.
5. The runtime loads every distinct referenced glTF model through `engine_assets` and assigns renderer-local model indices. Import failures retain path and validation context.
6. The runtime loads the optional WAV clip and opens audio output before graphics initialization. Startup failures are contextual and no partially initialized renderer is reported.
7. Only after scene, assets, and audio validation succeed, the runtime initializes `engine_render` with the owned window handle, validated models, and object capacity. The renderer uploads meshes, textures, object storage, depth, and egui resources.
8. The platform forwards close, resize, physical-key, right-mouse-button, relative mouse-motion, focus, redraw, idle, and exit events. Native input types do not cross this boundary.
9. Right-click captures the cursor and enables relative mouse look. V cycles `FirstPerson -> ThirdPerson -> TopDown -> FirstPerson` and logs the new mode, camera position, and target. Escape, F3, and focus loss release capture and clear held movement input. While the inspector is visible, its edit/select/save/reload keys take priority over movement controls.
10. A redraw advances at most five fixed simulation steps. Each step updates the one shared character through the physics controller, requests footsteps for grounded horizontal displacement, and invokes application scene logic. Camera selection changes placement and movement interpretation, not player physics.
11. The runtime extracts every model entity's world matrix, pairs it with its uploaded model index, builds optional keyboard-driven egui output, renders geometry plus overlay, and presents one frame. Frame diagnostics are emitted after complete one-second intervals.
12. F5 writes the live versioned scene to the configured workspace path. F9 validates a replacement against loaded assets and object capacity before replacing simulation and physics; failures are nonfatal. Transform edits also rebuild physics.
13. Resize events reconfigure a nonempty surface. Zero-sized windows suspend drawing. Lost/outdated surfaces are reconfigured, occluded/timed-out acquisitions skip the frame, and suboptimal surfaces are reconfigured after presentation.
14. A close request exits the event loop. The runtime explicitly drops renderer, clip, and audio output during exit, then returns success or a contextual error.

### Gameplay camera boundaries

- First person places the camera at the physics-controlled character position and looks along the yaw/pitch direction.
- Third person targets 0.35 m above that position and offsets the camera 4.5 m backward along the look direction.
- Top down targets the character position and places the camera 14 m above it with a 0.01 m Z offset to avoid a singular look-at matrix.
- WASD is yaw-relative in first/third person and fixed to world axes in top down, so screen movement does not rotate with the mouse while overhead.
- All modes retain the same position, grounded state, capsule collision, footsteps, speed, and fixed-step updates.

There is no visible player avatar, third-person obstruction handling, or camera collision. These are explicit limitations rather than implicit completed character-camera features. The fixed-step timing primitive and renderer-independent frame-statistics collector live in `engine_core`. The simulation runs at approximately 60 Hz with bounded catch-up. The dedicated reference benchmark records complete frame-statistic intervals separately from smoke-test diagnostics.

## Current rendering resources

- The runtime supplies the camera and a vector of plain render items, each containing a model index and extracted world matrix. Scene objects and gameplay state do not live in GPU resources.
- `engine_render` owns the camera uniform, dynamically offset object-transform storage, mesh vertex/index buffers, base-color textures, sampler, material bindings, WGSL pipeline, depth texture, egui renderer, and presentation resources.
- A mesh owns its index format and index count; the pipeline does not impose a mesh-specific index format.
- Each textured primitive has an independent material bind group. The current vertex contract is position, normal, and UV0; the current fragment shader samples base color and applies fixed directional and ambient illumination.
- The camera uses a right-handed, Y-up coordinate system with a 45-degree vertical field of view. Projection uses the current nonzero surface aspect ratio.
- The depth texture uses `Depth32Float` and is recreated with the surface after resize or reconfiguration.
- The optional debug pass loads the completed geometry color target, uploads all egui texture deltas, tessellates owned output, paints, and presents.
- `PrintWindow`-based per-window capture is the rendering smoke-test strategy. It captures this DX12 window directly and avoids assertions against unrelated overlapping desktop content.

The current frame order is:

```text
Input -> fixed character/physics steps -> audio requests -> application scene updates
      -> transform propagation -> multi-object render extraction -> geometry
      -> optional debug overlay -> presentation
```

Renderer, physics, and audio consume explicit extracted data or requests rather than owning or searching application gameplay state. Runtime reload currently substitutes default scene behavior for an application-provided update callback; this is a recorded API limitation, not hidden callback preservation.

## Public API policy

- Applications depend on `engine`, not internal crates, for engine functionality.
- The facade exposes small application-facing types and functions.
- Native window, event-loop, GPU, physics, and audio implementation types stay behind subsystem boundaries.
- Third-party math types may become public later if a deliberate decision determines that they are part of the engine's data contract.
- Internal APIs can change while the project is small, but changes must be driven by an actual consumer and checked against all examples.

The sandbox may use a logging-subscriber crate to configure process-wide diagnostics; this does not grant it access to engine internals.

## Resource ownership and failures

- Initialization and shutdown order belong to the runtime.
- Native resources have one clear owner.
- Cross-subsystem references use explicit data or handles rather than hidden shared mutable state.
- Expected platform failures are represented by engine-owned errors with context.
- Library crates do not install global logging subscribers or terminate the process.
- The executable decides how to report a fatal startup error and which exit code to return.

## Dependency decisions

Decisions are validated by implementation and focused spikes rather than treated as permanent commitments:

| Area | Current decision | Revisit trigger |
| --- | --- | --- |
| Window and events | `winit` | A demonstrated Windows requirement that it cannot support cleanly |
| Graphics | `wgpu` with DirectX 12 | Measured rendering or backend limitations |
| Math | `glam` | A demonstrated missing capability or interoperability problem |
| Models | Rust `gltf` loader | Required unsupported formats or an asset-cooking pipeline |
| Physics | `rapier3d` behind `engine_physics` | Gameplay requirements that expose an integration or capability mismatch |
| Audio | `rodio` behind `engine_audio`, with WAV playback | Required streaming, spatialization, or output-control capabilities |
| Debug UI | `egui` plus `egui-wgpu`, with bundled IBM Plex Mono | Editor requirements that exceed keyboard-driven immediate-mode tooling |
| Serialization | `serde` | A measured format or compatibility requirement |
| Diagnostics | `tracing` | A demonstrated diagnostics limitation |

Replacing a third-party crate should not require changes to unrelated subsystems. This does not justify wrapping every third-party type; isolation is applied where replacement or ownership risk is real.

## Verification strategy

Milestone 0 checks:

- Workspace formatting, compilation, linting, and tests.
- Focused fixed-step timing tests, including long-frame handling.
- A native Windows startup and shutdown smoke test.
- Debug and release compilation.
- Inspection of the actual dependency graph and public consumer.

Later milestones must add integrated checks appropriate to their behavior. GPU tests, rendering correctness, and performance evidence require the actual target adapter and cannot be replaced by headless logic tests.

## Change control

When a new feature or abstraction is proposed, record whether it:

1. Blocks the current acceptance condition.
2. Fixes an observed defect.
3. Addresses a concrete safety or integrity risk.

If none applies, park it with a revisit trigger and continue the current milestone. Architecture documents must be updated when implemented boundaries diverge from the documented ones.
