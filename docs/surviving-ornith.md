# Surviving Ornith — vertical slice checkpoint

Date: 2026-11-13
Engine: **Unbiased Bob Engine (UBE), nerd 0.1**
Status: **IMPLEMENTED AND LOCALLY VERIFIED**
Repository scope: single-player Windows vertical slice on Ornith Island

## Shipped gameplay

- One compact voxel island with grass, beach, water, trees, ore, berry bushes, flowers, clouds, and animals.
- Tree cutting with the axe and ore mining with the pickaxe; depleted objects disappear and their static colliders are removed safely.
- A one-round, accurate, long-range rifle with a 2.4-second reload and a six-round, faster, shorter-range pistol with spread and a 1.05-second reload.
- Wolf, cow, and hawk models with wild movement, sex, health, taming food requirements, and follow/stay commands.
- Tamed wolves and hawks assist against nearby hostile wolves; tamed cows follow without combat assistance.
- Opposite-sex tamed adult pairs consume food and produce live pooled offspring; babies mature after 45 seconds and breeding has a cooldown.
- Meat from hunting, berry foraging, eating, hunger drain, hostile-wolf damage, death, and respawning.
- Grid-snapped walls and foundations with resource costs, placement rejection, fixed object pools, and active collision geometry.
- First-person, third-person, and top-down play, a styled HUD, objective progression, hotbar, prompts, and original sound effects.

Multiplayer is **not implemented** and remains future scope.

## Run

From the repository root:

```sh
cargo run --release --bin surviving-ornith
```

The release executable is `target/release/surviving-ornith.exe`. Launch it with the repository root as the working directory so relative asset paths resolve. `--no-audio` disables output.

## Controls

| Input | Action |
| --- | --- |
| WASD | Move |
| Right mouse | Capture cursor / enable look |
| Left mouse | Use selected weapon or tool; place in build mode |
| 1 / 2 / 3 / 4 | Rifle / pistol / pickaxe / axe |
| R | Reload selected gun |
| E | Forage, tame, or breed with a nearby compatible pair |
| B | Toggle build mode |
| X or Tab | Cycle wall / foundation |
| F | Eat meat first, otherwise berries |
| C | Toggle companion follow / stay |
| V | Cycle first-person, third-person, top-down |
| Escape | Release cursor and clear held input |

## Architecture and ownership

`games/ornith` is a separate package that consumes the public `engine` facade. Ornith-specific rules are not embedded in engine subsystem crates.

UBE owns native input, fixed-step scheduling, player movement, collision-world rebuilding, rendering, HUD presentation, and audio output. `OrnithGame` owns rules and state through `GamePlugin`, `GameAction`, `GameFeedback`, and renderer-neutral `GameHud` contracts. Physics rebuilds occur at engine-controlled safe boundaries.

The scene preallocates **100 model objects**, including hidden resource, building, offspring, avatar, and view-model pools. Hidden objects are translated out of view rather than given invalid zero scale. This keeps renderer capacity fixed at startup and avoids dynamic GPU upload assumptions.

## Original assets and budget

`tools/generate_ornith_assets.py` uses deterministic geometry and audio generation. The island uses 17 original textured GLB models containing 11,844 source triangles across the model set, plus 11 mono PCM WAV clips. No downloaded, hand-sculpted, or third-party game assets are used. These original assets follow the repository's MIT license.

The startup renderer diagnostic confirmed **17 models / 17 primitives / object capacity 100** on the native DirectX 12 path.

## Verification evidence

All checks below ran locally on Windows:

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — **75 tests** |
| `cargo build --release --workspace` | PASS |
| Nine pre-existing Windows engine smoke scripts | PASS |
| `tools/ornith_smoke.py` | PASS |
| `tools/ornith_loop_smoke.py` | PASS |

The Ornith native smoke exercised startup, asset loading, HUD captures, rifle and pistol fire, reload, eating, companion command, all three cameras, build mode, named audio requests, and clean shutdown.

The full-loop native smoke used ordinary Windows keyboard and mouse input to complete this sequence in the real release:

1. Cut a tree and receive 6 wood.
2. Mine ore and receive 4 stone.
3. Place a collision-enabled wall and deduct its cost.
4. Mine a second obstructing ore rock.
5. Tame a female and male cow with berries.
6. Breed the nearby pair and receive a live calf.
7. Request chopping, mining, building, and taming sounds and exit cleanly.

Captured images and logs are written to `artifacts/ornith-smoke/` and `artifacts/ornith-loop-smoke/`; those generated artifacts are intentionally not committed. The captures were visually inspected, and a HUD readability/weapon-framing fix was rerun and verified.

The native smoke recorded steady intervals near 120 FPS at 1280×720 on the RTX 4080 SUPER. This is an observation from the playtest, not a new formal benchmark. Focus-loss clearing is covered by the runtime unit test; the native input smoke explicitly verifies two real cursor captures and two Escape releases, not an operating-system foreground transition.

Remote CI and owner acceptance: **NOT RECORDED** in this checkpoint.

## Deliberate limitations

- Single-player only; no networking, dedicated server, accounts, or online persistence.
- One finite test island; no streaming, procedural infinite terrain, day/night cycle, or large-world simulation.
- Full gameplay-state saving/loading is not implemented. Engine debug scene persistence is not a survival save system.
- Combat uses deterministic aim selection rather than projectile physics or collision-ray occlusion.
- Animal AI is intentionally simple and does not use navigation meshes.
- Resource nodes do not respawn. Building and offspring pools are bounded for this slice.
- Third-person camera obstruction handling and skeletal animation are not implemented.
- This is not feature parity with ARK, Minecraft, or Unreal Engine.

## Next checkpoint candidates

Prioritize owner playtest feedback, then choose one bounded slice: full survival save/load, improved combat/AI feedback, more building pieces, or the separately scoped networking milestone. Do not imply that any of these already exists.
