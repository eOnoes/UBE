"""Exercise first-person wall collision and sliding in the Windows release build."""

from __future__ import annotations

import argparse
from pathlib import Path
import re
import subprocess
import time

from input_smoke import VK_D, WM_KEYDOWN, WM_KEYUP, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window
from window_capture import capture_client_image


VK_W = 0x57


def character_positions(diagnostics: str) -> list[list[float]]:
    positions = []
    for line in diagnostics.splitlines():
        if "character position" not in line:
            continue
        match = re.search(r"position=\[([^\]]+)\]", line)
        if match:
            positions.append([float(value.strip()) for value in match.group(1).split(",")])
    return positions


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/collision-smoke"))
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
            capture_client(hwnd, output_dir / "collision-initial.png", "textured")

            post_message(hwnd, WM_KEYDOWN, VK_W)
            time.sleep(0.8)
            post_message(hwnd, WM_KEYUP, VK_W)
            time.sleep(0.2)

            post_message(hwnd, WM_KEYDOWN, VK_D)
            time.sleep(1.2)
            post_message(hwnd, WM_KEYUP, VK_D)
            time.sleep(0.3)
            capture_client_image(hwnd).save(output_dir / "collision-wall.png")

            post_message(hwnd, WM_KEYDOWN, VK_W)
            time.sleep(0.55)
            post_message(hwnd, WM_KEYUP, VK_W)
            time.sleep(0.3)
            capture_client_image(hwnd).save(output_dir / "collision-slide.png")

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
    positions = character_positions(diagnostics)
    if len(positions) < 2:
        raise AssertionError(f"Expected wall and slide positions, got {positions}")
    wall, slide = positions[-2:]
    if not 1.0 < wall[0] < 1.75:
        raise AssertionError(f"Character did not stop at the expected wall boundary: {wall}")
    if abs(slide[0] - wall[0]) > 0.08:
        raise AssertionError(f"Character crossed or departed the wall while sliding: {wall} -> {slide}")
    if wall[2] - slide[2] < 0.5:
        raise AssertionError(f"Forward input did not slide along the wall: {wall} -> {slide}")
    for position in (wall, slide):
        if not 0.85 < position[1] < 1.0:
            raise AssertionError(f"Character did not remain at grounded eye height: {position}")
    print(f"Wall contact position: {wall}")
    print(f"Wall sliding position: {slide}")
    print("Grounded eye height: stable")

    stderr = stderr_path.read_text(encoding="utf-8")
    if stderr.strip():
        raise AssertionError(f"Sandbox wrote to standard error: {stderr}")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
