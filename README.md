# Unbiased Bob Engine (UBE)

**UBE nerd 0.1** is a small, modular, Windows-first 3D engine written in Rust. It includes native DirectX 12 rendering, fixed-step scenes, collision, audio, three gameplay cameras, scene persistence, debug inspection, and an application-facing gameplay API.

The repository also contains **Surviving Ornith**, an original single-player survival vertical slice built on that public API.

## Play Surviving Ornith

From the repository root:

```sh
cargo run --release --bin surviving-ornith
```

The working directory must remain the repository root so the game can find `assets/ornith/`. Add `-- --no-audio` to run without audio output.

### Controls

| Input | Action |
| --- | --- |
| WASD | Move |
| Right mouse / mouse | Capture cursor and look |
| Left mouse | Fire, harvest, or place the selected structure |
| 1 / 2 | Rifle / pistol |
| 3 / 4 | Pickaxe / axe |
| R | Reload |
| E | Forage, tame, or breed |
| B | Toggle build mode |
| X or Tab | Cycle wall / foundation |
| F | Eat meat or berries |
| C | Companions follow / stay |
| V | First-person / third-person / top-down |
| Escape | Release cursor and clear held input |

Start with 4 meat and 4 berries. Wolves and hawks eat meat; cows eat berries. A tamed adult needs a nearby, tamed adult of the opposite sex and 2 additional food to breed. Babies mature after 45 seconds.

The rifle is accurate and long-ranged but holds one round and reloads slowly. The pistol fires faster, holds six rounds, and has more spread with a shorter effective range.

## Engine examples

```sh
cargo run --release --bin engine-sandbox
cargo run --release --bin engine-gallery
```

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
python tools/ornith_smoke.py
python tools/ornith_loop_smoke.py
```

See `docs/surviving-ornith.md` for release evidence and limitations, and `docs/project-roadmap.md` for the engine roadmap.

## Original assets

Ornith's 17 GLB models and 11 WAV clips are generated locally by `tools/generate_ornith_assets.py`; no external game assets are used. Regenerate them with:

```sh
python tools/generate_ornith_assets.py
```

## Scope

This is a compact, polished test island—not an Unreal Engine replacement, a complete ARK-scale survival game, or a multiplayer release. Networking, persistence of full gameplay state, advanced animation, and large-world streaming remain future work.

## License

MIT. The owner-provided root `LICENSE` is preserved. Third-party notices are in `THIRD_PARTY_NOTICES.html`.
