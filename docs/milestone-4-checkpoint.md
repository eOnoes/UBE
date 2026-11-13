# Milestone 4 Checkpoint — Engine-Owned Audio Playback

## Outcome

The Windows release initializes a real audio output, validates and decodes a reusable WAV clip, and plays footsteps in response to actual grounded horizontal character movement. Audio is isolated in `engine_audio`; applications configure it through the public `AppConfig` API.

## Implemented

- `engine_audio` owns decoded clip buffers, the operating-system output stream, and the mixer that owns active voices.
- `rodio 0.23.0` is isolated behind engine-owned clip, output-information, playback, and error types. Only playback, WAV, and tracing features are enabled.
- The original deterministic mono footstep asset has 5,292 frames at 44,100 Hz and a duration of 120 ms. `tools/generate_audio_asset.py` reproduces it.
- `AudioClip::load` performs file I/O and decoding without opening an output device, allowing hardware-independent failure and replay tests.
- Runtime startup loads audio and opens output before renderer initialization. Missing, undecodable, empty, and malformed clip data have engine-owned errors.
- Grounded horizontal displacement drives an immediate footstep followed by a 420 ms fixed-step cadence. Idle, airborne, and blocked movement do not request playback.
- `AppConfig::with_footstep_sound_path` selects a clip; `without_audio` deliberately disables output.
- Runtime shutdown drops the clip and output stream explicitly during native event-loop exit.

## Verified behavior

- Focused clip decoding, reusable replay, missing-file, invalid-file, cadence, idle, and airborne tests: PASS.
- Actual Windows output initialized with 2 channels at 48,000 Hz: PASS.
- The initial idle character emitted zero playback requests: PASS.
- One second of grounded forward movement emitted three playback requests: PASS.
- The following idle interval and shutdown emitted no additional requests: PASS.
- Missing and invalid audio both failed contextually before renderer initialization: PASS.
- Inspected initial and moved captures show coherent textured rendering with the audio subsystem active: PASS.
- Post-audio rendering, resize/restore, pointer capture, keyboard movement, wall collision/sliding, fixed-step animation, and clean shutdown regressions: PASS.

Evidence is under `artifacts/audio-smoke/`, `artifacts/startup-failure-smoke/`, `artifacts/render-smoke/`, `artifacts/input-smoke/`, `artifacts/collision-smoke/`, and `artifacts/scene-smoke/`.

## Quality checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo check --workspace --all-targets` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` | PASS — 56 tests total |
| `cargo build --workspace --release` | PASS |
| Actual Windows output-device and gameplay playback smoke test | PASS |
| Remote GitHub Actions | NOT RUN |
| Performance benchmark | NOT RUN |

## Current limitations

- Playback is non-spatial, mono-source audio mixed to the default output. Volume controls, music streaming, positional attenuation, and scene-authored sound sources are deferred.
- The smoke test verifies device initialization and accepted playback requests; it does not claim microphone-recorded or owner-confirmed audible output.
- The default output is opened even when an application never moves unless audio is explicitly disabled.
- Debug inspection/editing, interactive scene save/load, and the second executable are not yet implemented.
- Logged frame statistics are diagnostics, not benchmark results. Remote CI and owner acceptance remain unrecorded.

## Next checkpoint

Add selected-entity inspection and property editing, explicit interactive scene save/load with contextual failures, and release-level proof that edits persist and reload correctly.
