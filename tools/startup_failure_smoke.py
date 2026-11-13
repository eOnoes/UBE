"""Verify that invalid scene startup fails before renderer initialization."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/startup-failure-smoke"))
    args = parser.parse_args()

    executable = args.executable.resolve()
    root = executable.parent.parent.parent
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    transform = {
        "translation": [0.0, 1.0, 0.0],
        "rotation": [0.0, 0.0, 0.0, 1.0],
        "scale": 1.0,
    }
    cases = {
        "missing-model": (
            {
                "version": 1,
                "entities": [
                    {
                        "id": 0,
                        "parent": None,
                        "transform": transform,
                        "model": "artifacts/startup-failure-smoke/does-not-exist.glb",
                    }
                ],
            },
            "does-not-exist.glb",
        ),
        "unsupported-version": (
            {
                "version": 999,
                "entities": [
                    {"id": 0, "parent": None, "transform": transform, "model": None}
                ],
            },
            "unsupported scene format version 999",
        ),
    }

    for name, (document, expected_error) in cases.items():
        scene_path = output_dir / f"{name}.scene.json"
        scene_path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")
        result = subprocess.run(
            [str(executable), str(scene_path)],
            cwd=root,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        (output_dir / f"{name}-stdout.log").write_text(result.stdout, encoding="utf-8")
        (output_dir / f"{name}-stderr.log").write_text(result.stderr, encoding="utf-8")
        combined = result.stdout + result.stderr
        if result.returncode == 0:
            raise AssertionError(f"{name} unexpectedly started successfully")
        if expected_error not in combined:
            raise AssertionError(f"{name} did not report {expected_error!r}: {combined}")
        if "renderer initialized" in combined:
            raise AssertionError(f"{name} initialized a renderer before failing")
        print(f"{name}: contextual startup failure, no renderer initialized")

    invalid_audio = output_dir / "invalid.wav"
    invalid_audio.write_bytes(b"not a wave file")
    audio_cases = {
        "missing-audio": (output_dir / "does-not-exist.wav", "does-not-exist.wav"),
        "invalid-audio": (invalid_audio, "failed to decode audio clip"),
    }
    for name, (audio_path, expected_error) in audio_cases.items():
        result = subprocess.run(
            [str(executable), str(root / "assets/scenes/orbit.scene.json"), str(audio_path)],
            cwd=root,
            capture_output=True,
            text=True,
            timeout=15,
            check=False,
        )
        (output_dir / f"{name}-stdout.log").write_text(result.stdout, encoding="utf-8")
        (output_dir / f"{name}-stderr.log").write_text(result.stderr, encoding="utf-8")
        combined = result.stdout + result.stderr
        if result.returncode == 0:
            raise AssertionError(f"{name} unexpectedly started successfully")
        if expected_error not in combined:
            raise AssertionError(f"{name} did not report {expected_error!r}: {combined}")
        if "renderer initialized" in combined:
            raise AssertionError(f"{name} initialized a renderer before failing")
        print(f"{name}: contextual startup failure, no renderer initialized")

    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
