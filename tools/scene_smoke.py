"""Verify that the rendered scene changes without user input."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import time

from input_smoke import changed_ratio, post_message
from render_smoke import (
    SWP_NOZORDER,
    WM_CLOSE,
    capture_client,
    user32,
    wait_for_window,
)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/scene-smoke"))
    args = parser.parse_args()

    executable = args.executable.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    first_path = output_dir / "scene-first.png"
    second_path = output_dir / "scene-second.png"
    stdout_path = output_dir / "stdout.log"
    stderr_path = output_dir / "stderr.log"

    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(
            [str(executable)], cwd=executable.parent.parent.parent, stdout=stdout, stderr=stderr
        )
        try:
            hwnd = wait_for_window("Unbiased Bob Engine Sandbox — nerd 0.1", process, timeout=15)
            time.sleep(1)
            if not user32.SetWindowPos(hwnd, 0, 180, 120, 960, 640, SWP_NOZORDER):
                raise OSError("Could not resize the sandbox window")
            time.sleep(0.3)
            capture_client(hwnd, first_path, "textured")
            time.sleep(0.5)
            capture_client(hwnd, second_path, "textured")

            scene_change = changed_ratio(first_path, second_path)
            if scene_change < 0.005:
                raise AssertionError(f"Unattended scene changed only {scene_change:.1%} of pixels")
            print(f"Fixed-step scene pixel change: {scene_change:.1%}")

            post_message(hwnd, WM_CLOSE)
            return_code = process.wait(timeout=5)
            if return_code != 0:
                raise RuntimeError(f"Sandbox returned exit code {return_code}")
            print("Clean shutdown: exit code 0")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    stderr = stderr_path.read_text(encoding="utf-8")
    if stderr.strip():
        raise AssertionError(f"Sandbox wrote to standard error: {stderr}")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
