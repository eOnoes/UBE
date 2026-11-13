"""Exercise Ornith's harvest-build-tame-breed loop through native Windows input."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import time

from input_smoke import WM_KEYDOWN, WM_KEYUP, move_cursor_to_client_center, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window

TITLE = "Surviving Ornith — UBE nerd 0.1"
VK = {
    "W": 0x57, "A": 0x41, "S": 0x53, "D": 0x44, "E": 0x45, "B": 0x42,
    "2": 0x32, "3": 0x33, "4": 0x34,
}
WM_LBUTTONDOWN, WM_LBUTTONUP = 0x0201, 0x0202


def tap(hwnd: int, key: str, pause: float = 0.18) -> None:
    post_message(hwnd, WM_KEYDOWN, VK[key])
    time.sleep(0.08)
    post_message(hwnd, WM_KEYUP, VK[key])
    time.sleep(pause)


def hold(hwnd: int, key: str, seconds: float) -> None:
    post_message(hwnd, WM_KEYDOWN, VK[key])
    time.sleep(seconds)
    post_message(hwnd, WM_KEYUP, VK[key])
    time.sleep(0.2)


def use(hwnd: int) -> None:
    post_message(hwnd, WM_LBUTTONDOWN)
    time.sleep(0.1)
    post_message(hwnd, WM_LBUTTONUP)
    time.sleep(0.42)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/surviving-ornith.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/ornith-loop-smoke"))
    args = parser.parse_args()
    executable = args.executable.resolve()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    stdout_path, stderr_path = output / "stdout.log", output / "stderr.log"

    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen([str(executable)], cwd=executable.parent.parent.parent, stdout=stdout, stderr=stderr)
        try:
            hwnd = wait_for_window(TITLE, process, timeout=20)
            if not user32.SetWindowPos(hwnd, 0, 120, 90, 1280, 720, SWP_NOZORDER):
                raise OSError("Could not resize Ornith window")
            time.sleep(2)
            move_cursor_to_client_center(hwnd)

            # Face north throughout. Align with the first tree using movement alone.
            hold(hwnd, "A", 1.34)
            hold(hwnd, "W", 0.65)
            tap(hwnd, "4")
            for _ in range(3):
                use(hwnd)
            capture_client(hwnd, output / "01-tree-harvested.png", "geometry")

            # The nearest ore is straight ahead from this lane after strafing east.
            hold(hwnd, "D", 2.34)
            tap(hwnd, "3")
            for _ in range(3):
                use(hwnd)
            tap(hwnd, "B")
            use(hwnd)
            capture_client(hwnd, output / "02-wall-built.png", "geometry")
            tap(hwnd, "B")

            # Approach the cow lane, clear the blocking ore, then tame the nearby cow.
            hold(hwnd, "A", 3.0)
            hold(hwnd, "W", 2.4)
            tap(hwnd, "3")
            for _ in range(3):
                use(hwnd)
            hold(hwnd, "W", 0.65)
            tap(hwnd, "E", 0.4)

            # Approach the bull from the south so the follower stays behind the camera.
            hold(hwnd, "A", 1.67)
            hold(hwnd, "S", 2.75)
            tap(hwnd, "E", 0.5)
            tap(hwnd, "E", 0.7)
            capture_client(hwnd, output / "03-cow-pair-and-calf.png", "geometry")

            post_message(hwnd, WM_CLOSE)
            code = process.wait(timeout=8)
            if code != 0:
                raise RuntimeError(f"Ornith returned exit code {code}")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    diagnostics = stdout_path.read_text(encoding="utf-8")
    expected = ["Timber!", "Ore recovered", "Wall placed", "Cow tamed", "A baby Cow was born"]
    for text in expected:
        if text not in diagnostics:
            raise AssertionError(f"Native gameplay loop did not reach: {text}")
    if diagnostics.count("Cow tamed") != 2:
        raise AssertionError("Expected two real cow-taming interactions")
    for effect in ["effect=chopping", "effect=mining", "effect=building", "effect=taming"]:
        if effect not in diagnostics:
            raise AssertionError(f"Missing gameplay audio request: {effect}")
    stderr = stderr_path.read_text(encoding="utf-8")
    if stderr.strip():
        raise AssertionError(f"Ornith wrote to standard error: {stderr}")
    print("Ornith full-loop native smoke: tree cut, ore mined, wall built, two cows tamed, calf bred, audio requested, clean exit")
    print(f"Evidence written to {output}")


if __name__ == "__main__":
    main()
