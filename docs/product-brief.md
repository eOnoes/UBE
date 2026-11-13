# Unbiased Bob Engine Product Brief

## Purpose

Build **Unbiased Bob Engine (UBE)**, a small Windows-first 3D game engine from the ground up in Rust. The public release label is **nerd 0.1**. UBE is developed alongside real example applications so that its architecture is driven by observed requirements rather than speculative generality. It is not intended to match Unreal Engine's breadth.

"Ground up" means that the engine's runtime, public API, scene model, resource management, renderer, and tools are designed and implemented for this project. Focused third-party Rust crates may provide operating-system integration, graphics APIs, math, physics, audio, and serialization.

The project should remain practical for the team and content-creator collaboration. Ownership, contributor terms, and an explicit project license still require a separate team decision; no license grant is implied by this brief. See `licensing.md`.

## Current product outcome

A developer can run a small 3D scene, switch among first-person, third-person, and top-down gameplay cameras, walk around it, collide with designated geometry, hear basic audio, inspect and modify selected scene properties, save and reload the scene, and produce a Windows release build.

The reusable-engine proof is a second example that uses the same public engine API without special-case changes to subsystem internals. The initial gameplay slice remains local and single-player.

## Initial constraints

- Windows 10/11.
- Rust for engine and gameplay code.
- DirectX 12 through `wgpu` for the initial renderer.
- Single-player and single-threaded engine-owned runtime logic in nerd 0.1.
- First-person, third-person, and top-down cameras sharing one character simulation.
- Static glTF models initially.
- Debug UI before a general-purpose editor.
- Rust gameplay code rather than a scripting layer.
- A 60 FPS target for an agreed reference scene, measured rather than assumed.

## Reference machine

- CPU: AMD Ryzen 9 7900X.
- Memory: 127.1 GB reported by Windows.
- Primary discrete GPU: NVIDIA GeForce RTX 4080 SUPER.
- Additional display adapters are present; renderer tests must identify the adapter actually used.

The reference scene and its complexity budget will be defined before the rendering performance milestone. No performance claim is made by the foundation milestone.

## Milestone 0 outcome

A minimal Cargo workspace with documented architecture and conventions, structured diagnostics, automated quality checks, a fixed-step timing primitive, and a sandbox application that opens and closes a Windows window through the public engine API.

Milestone 0 does not include rendering, physics, audio, an ECS, scene serialization, or an editor.

## Next major product requirement: online multiplayer

Online multiplayer is now an approved product direction for the collaborative game project, but **it is not implemented in nerd 0.1**. It requires its own milestones for transport, authority and security, connection/session flow, deterministic or server-authoritative simulation choices, state replication, prediction/reconciliation, interest management, testing, and deployment. No networking or multiplayer claim is made by the current release.

## Other deferred capabilities

The following remain parked until a milestone or real example requires them:

- Non-Windows platforms.
- Skeletal animation.
- Visual scripting and material graphs.
- Open-world streaming, terrain, and vegetation systems.
- Advanced global illumination.
- Engine-owned multithreaded simulation or rendering.
- Custom physics and audio engines.
- A polished general-purpose editor.

Deferring these capabilities does not require designing their implementations now. New requirements will be reconciled explicitly with the current milestone.

## Overall acceptance evidence

Each milestone must report its checks as PASS, FAIL, or NOT RUN. Component tests alone do not establish end-to-end acceptance. User-facing behavior must be exercised on Windows, and performance claims must identify the scene, hardware, graphics adapter, and measurement method.
