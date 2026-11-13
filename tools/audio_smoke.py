"""Exercise real Windows audio output and gameplay-triggered footstep requests."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import time

from input_smoke import WM_KEYDOWN, WM_KEYUP, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window


VK_W = 0x57


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/audio-smoke"))
    args = parser.parse_args()

    executable = args.executable.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = output_dir / "stdout.log"
    stderr_path = output_dir / "stderr.log"

    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(
            [str(executable)], cwd=executable.parent.parent.parent, stdout=stdout, stderr=stderr
        )
        try:
            hwnd = wait_for_window("Unbiased Bob Engine Sandbox — nerd 0.1", process, timeout=15)
            time.sleep(1.2)
            if not user32.SetWindowPos(hwnd, 0, 180, 120, 960, 640, SWP_NOZORDER):
                raise OSError("Could not resize the sandbox window")
            time.sleep(0.5)
            capture_client(hwnd, output_dir / "audio-initial.png", "textured")
            initial_diagnostics = stdout_path.read_text(encoding="utf-8")
            if "audio initialized" not in initial_diagnostics:
                raise AssertionError("Runtime did not initialize its audio output")
            if "footstep playback requested" in initial_diagnostics:
                raise AssertionError("Footsteps played while the character was idle")
            print("Audio output initialized; idle character is silent")

            post_message(hwnd, WM_KEYDOWN, VK_W)
            time.sleep(1.0)
            post_message(hwnd, WM_KEYUP, VK_W)
            time.sleep(0.6)
            capture_client(hwnd, output_dir / "audio-moved.png", "textured")
            moving_diagnostics = stdout_path.read_text(encoding="utf-8")
            playback_requests = moving_diagnostics.count("footstep playback requested")
            if playback_requests < 2:
                raise AssertionError(f"Expected repeated movement footsteps, got {playback_requests}")
            print(f"Grounded movement playback requests: {playback_requests}")

            post_message(hwnd, WM_CLOSE)
            return_code = process.wait(timeout=5)
            if return_code != 0:
                raise RuntimeError(f"Sandbox returned exit code {return_code}")
            print("Clean shutdown: exit code 0")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    diagnostics = stdout_path.read_text(encoding="utf-8")
    final_requests = diagnostics.count("footstep playback requested")
    if final_requests != playback_requests:
        raise AssertionError(
            f"Idle/shutdown added unexpected playback requests: {playback_requests} -> {final_requests}"
        )
    print("Idle interval and shutdown added no playback requests")

    stderr = stderr_path.read_text(encoding="utf-8")
    if stderr.strip():
        raise AssertionError(f"Sandbox wrote to standard error: {stderr}")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
