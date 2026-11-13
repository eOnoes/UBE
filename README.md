# Unbiased Bob Engine (UBE)

**UBE nerd 0.1** is a small, modular, Windows-first 3D engine written in Rust. This repository contains the engine, generic examples, and engine development tools—not a game project.

## Engine capabilities

- Native DirectX 12 rendering and validated textured GLB models
- First-person, third-person, and top-down cameras
- Keyboard movement and mouse look
- Fixed-step simulation and hierarchical scene transforms
- Physics-backed character collision
- WAV audio playback and named gameplay sound effects
- Versioned scene save/load and debug property inspection
- An application-facing `GamePlugin` API for game-owned rules and HUD data

UBE is an early, compact engine—not an Unreal Engine replacement. Networking, skeletal animation, large-world streaming, a polished general-purpose editor, and non-Windows support remain future scope.

## Run the generic examples

From this repository root:

```sh
cargo run --release --bin engine-sandbox
cargo run --release --bin engine-gallery
```

The sandbox can open a scene and an optional footstep WAV passed as command-line arguments. The gallery constructs its own scene entirely through the public engine API.

## Use UBE in your own game

Create a separate Rust project and add the public `engine` facade as a dependency.

For local development, with `UBE` and `my-game` as sibling directories:

```toml
[dependencies]
engine = { path = "../UBE/crates/engine" }
```

Or consume the published repository directly:

```toml
[dependencies]
engine = { git = "https://github.com/eOnoes/UBE.git", package = "engine" }
```

Then construct a scene, install an optional gameplay plugin, and run the engine:

```rust
use engine::{AppConfig, CameraMode, GameHud, GamePlugin};

struct MyGame;

impl GamePlugin for MyGame {
    fn hud(&self) -> GameHud {
        GameHud {
            title: "My Game".into(),
            ..GameHud::default()
        }
    }
}

fn main() {
    let scene = my_game::build_scene();
    let config = AppConfig::new("My Game — powered by UBE")
        .with_scene(scene)
        .with_game_plugin(MyGame)
        .with_camera_mode(CameraMode::ThirdPerson);

    engine::run(config).expect("failed to run the game");
}
```

Your game owns its assets, world construction, rules, AI, progression, and save data. UBE owns the native window, input, rendering, cameras, collision world, fixed-step scheduling, scene services, and audio output. See `docs/creating-a-game.md` for the project layout and API walkthrough.

## Engine architecture

The public `engine` crate re-exports the supported application API. Internal crates separate core timing and identity, assets, audio, physics, platform input, rendering, scenes, and runtime orchestration. Games should depend on `engine`, not reach into internal subsystem crates.

## Verify

```sh
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release --workspace
```

Windows native checks:

```sh
python tools/render_smoke.py
python tools/input_smoke.py
python tools/camera_smoke.py
python tools/collision_smoke.py
python tools/audio_smoke.py
python tools/scene_smoke.py
python tools/debug_smoke.py
python tools/startup_failure_smoke.py
python tools/gallery_smoke.py
```

See `docs/project-roadmap.md` for current scope and evidence.

## License

MIT. The owner-provided root `LICENSE` is preserved. Third-party notices are in `THIRD_PARTY_NOTICES.html`.
