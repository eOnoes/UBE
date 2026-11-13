# Creating a game with UBE

UBE is intended to be consumed as a Rust dependency from an independent game repository. Do not copy the engine sources into the game and do not place the game inside the UBE workspace.

## 1. Create the game project

```sh
cargo new my-game
cd my-game
```

A typical layout is:

```text
my-game/
├── Cargo.toml
├── assets/
│   ├── models/
│   └── audio/
└── src/
    ├── lib.rs
    └── main.rs
```

The game owns this directory, its assets, and its gameplay-specific save data.

## 2. Depend on the public engine facade

For local engine and game development in sibling directories:

```toml
[dependencies]
engine = { path = "../UBE/crates/engine" }
tracing-subscriber = { version = "0.3", features = ["fmt"] }
```

For a game that only consumes UBE from GitHub:

```toml
[dependencies]
engine = { git = "https://github.com/eOnoes/UBE.git", package = "engine" }
tracing-subscriber = { version = "0.3", features = ["fmt"] }
```

For reproducible releases, pin the Git dependency to a reviewed commit with `rev = "..."` or use a release tag. Use the public `engine` crate rather than internal crates such as `engine_render` or `engine_physics`.

## 3. Construct a scene

The public scene API supports entities, local and hierarchical transforms, GLB model references, and box colliders:

```rust
use engine::{BoxCollider, ModelReference, Scene, Transform};

pub fn build_scene() -> Scene {
    let mut scene = Scene::new();

    let model = ModelReference::new("assets/models/MyModel.glb")
        .expect("the model path must be valid");
    let entity = scene.spawn(Transform::default().with_translation([0.0, 1.0, 0.0]));
    assert!(scene.set_model(entity, Some(model)));

    let floor = scene.spawn(Transform::default().with_translation([0.0, -0.1, 0.0]));
    let floor_shape = BoxCollider::new([12.0, 0.1, 12.0]).expect("valid floor extents");
    assert!(scene.set_collider(floor, Some(floor_shape)));

    scene
}
```

Run the game with its own project root as the working directory so relative asset paths resolve to the game's `assets` directory.

## 4. Add game-owned rules

Implement `GamePlugin` to respond to engine-owned actions, advance gameplay at fixed steps, and supply renderer-neutral HUD data:

```rust
use std::time::Duration;
use engine::{
    GameAction, GameFeedback, GameHud, GamePlugin, PlayerSnapshot, Scene,
};

struct MyGame {
    score: u32,
}

impl GamePlugin for MyGame {
    fn on_action(
        &mut self,
        action: GameAction,
        scene: &mut Scene,
        player: PlayerSnapshot,
    ) -> GameFeedback {
        // Interpret actions using the game's own rules.
        GameFeedback::default()
    }

    fn fixed_update(
        &mut self,
        scene: &mut Scene,
        elapsed: Duration,
        player: PlayerSnapshot,
    ) -> GameFeedback {
        // Advance AI, progression, cooldowns, and other game state.
        GameFeedback::default()
    }

    fn hud(&self) -> GameHud {
        GameHud {
            title: "My Game".into(),
            status: format!("Score: {}", self.score),
            crosshair: true,
            ..GameHud::default()
        }
    }
}
```

`GameFeedback` lets the game request physics rebuilds, player respawning, and named sound effects without taking ownership of engine subsystems. Unused callback parameters can be prefixed with an underscore.

## 5. Configure and run

```rust
use engine::{AppConfig, CameraMode};

fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    let config = AppConfig::new("My Game — powered by UBE")
        .with_scene(my_game::build_scene())
        .with_game_plugin(my_game::MyGame { score: 0 })
        .with_camera_mode(CameraMode::FirstPerson)
        .without_audio();

    engine::run(config).expect("failed to run the game");
}
```

Register game audio with `with_sound_effect` and footsteps with `with_footstep_sound_path` when audio is enabled. See `examples/gallery` for a complete scene-construction example without a gameplay plugin.

## Character movement

UBE owns the reusable character controller and exposes its current state through `PlayerSnapshot`:

- **Space** jumps when the standing character is grounded.
- **Left Ctrl** toggles crouching and standing; from prone it enters crouching.
- **Z** toggles prone and standing.
- **Left Shift** sprints while held with movement input. Sprinting consumes energy and only works while standing; energy recovers when sprinting stops.
- Standing is rejected when the larger capsule would intersect a low ceiling.

Use `snapshot.stance`, `snapshot.sprint_energy`, `snapshot.sprinting`, and `snapshot.grounded` for game rules, animation state, and HUD presentation. Set `GameHud.energy` to `Some(snapshot.sprint_energy)` to display the built-in energy meter, or `None` for games without sprint energy. Tool swings, weapon recoil, game-specific sounds, and other presentation remain game-owned.

## Ownership boundary

UBE provides the native window, physical input mapping, rendering, cameras, character movement, collision world, fixed-step scheduling, scene services, HUD presentation, and audio output.

The game provides models and audio, world layout, weapons, AI, quests, resources, progression, HUD contents, and full gameplay-state persistence. Engine debug scene save/load is not a substitute for a game's own save system.
