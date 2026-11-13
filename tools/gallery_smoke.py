"""Prove the second public-API example with its real Windows release binary."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import time

from input_smoke import WM_KEYDOWN, WM_KEYUP, changed_ratio, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window


F3, F5 = 0x72, 0x74


def press(hwnd: int, key: int, scan_code: int) -> None:
    post_message(hwnd, WM_KEYDOWN, key, 1 | (scan_code << 16))
    post_message(hwnd, WM_KEYUP, key, 1 | (scan_code << 16) | 0xC0000000)
    time.sleep(0.2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-gallery.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/gallery-smoke"))
    args = parser.parse_args()
    executable = args.executable.resolve()
    workspace = executable.parent.parent.parent
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = output_dir / "stdout.log"
    stderr_path = output_dir / "stderr.log"
    scene_path = workspace / "artifacts/runtime/gallery.scene.json"
    scene_path.unlink(missing_ok=True)

    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(
            [str(executable)], cwd=workspace, stdout=stdout, stderr=stderr
        )
        try:
            hwnd = wait_for_window("Unbiased Bob Engine Gallery — nerd 0.1", process, timeout=20)
            time.sleep(1.0)
            if not user32.SetWindowPos(hwnd, 0, 180, 120, 960, 640, SWP_NOZORDER):
                raise OSError("Could not resize the gallery window")
            time.sleep(0.3)
            capture_client(hwnd, output_dir / "gallery-initial.png", "textured")
            time.sleep(0.65)
            capture_client(hwnd, output_dir / "gallery-animated.png", "textured")
            change = changed_ratio(
                output_dir / "gallery-initial.png", output_dir / "gallery-animated.png"
            )
            if change < 0.005:
                raise AssertionError(f"Gallery fixed-step behavior changed only {change:.1%} of pixels")
            print(f"Independent gallery animation pixel change: {change:.1%}")

            press(hwnd, F3, 0x3D)
            press(hwnd, F5, 0x3F)
            if not scene_path.is_file():
                raise AssertionError("Gallery did not save to its own workspace scene path")
            document = json.loads(scene_path.read_text(encoding="utf-8"))
            if document["version"] != 1 or len(document["entities"]) != 4:
                raise AssertionError("Gallery saved an unexpected scene document")
            if sum(entity["model"] is not None for entity in document["entities"]) != 2:
                raise AssertionError("Gallery did not persist both model references")
            print("Gallery saved four entities and two models to its independent workspace path")

            press(hwnd, F3, 0x3D)
            post_message(hwnd, WM_CLOSE)
            return_code = process.wait(timeout=8)
            if return_code != 0:
                raise RuntimeError(f"Gallery returned exit code {return_code}")
            print("Clean shutdown: exit code 0")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    diagnostics = stdout_path.read_text(encoding="utf-8")
    for expected in (
        'title="Unbiased Bob Engine Gallery — nerd 0.1"',
        'engine="Unbiased Bob Engine"',
        'release="nerd 0.1"',
        "audio disabled",
        "object_capacity=2",
        "frame statistics",
    ):
        if expected not in diagnostics:
            raise AssertionError(f"Gallery diagnostics did not include {expected!r}")
    if stderr_path.read_text(encoding="utf-8").strip():
        raise AssertionError("Gallery wrote to standard error")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
