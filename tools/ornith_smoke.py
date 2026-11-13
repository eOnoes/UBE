"""Launch Surviving Ornith, exercise native gameplay input, and capture evidence."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import time

from PIL import ImageStat

from input_smoke import WM_KEYDOWN, WM_KEYUP, move_cursor_to_client_center, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window
from window_capture import capture_client_image

TITLE = "Surviving Ornith — UBE nerd 0.1"
VK = {"R": 0x52, "F": 0x46, "B": 0x42, "V": 0x56, "C": 0x43, "1": 0x31, "2": 0x32}
WM_LBUTTONDOWN, WM_LBUTTONUP = 0x0201, 0x0202
WM_RBUTTONDOWN, WM_RBUTTONUP = 0x0204, 0x0205


def tap(hwnd: int, key: str) -> None:
    post_message(hwnd, WM_KEYDOWN, VK[key])
    time.sleep(0.08)
    post_message(hwnd, WM_KEYUP, VK[key])
    time.sleep(0.15)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/surviving-ornith.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/ornith-smoke"))
    args = parser.parse_args()
    executable = args.executable.resolve()
    output = args.output_dir.resolve()
    output.mkdir(parents=True, exist_ok=True)
    stdout_path, stderr_path = output / "stdout.log", output / "stderr.log"

    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen([str(executable)], cwd=executable.parent.parent.parent, stdout=stdout, stderr=stderr)
        try:
            hwnd = wait_for_window(TITLE, process, timeout=20)
            if not user32.SetWindowPos(hwnd, 0, 100, 80, 1280, 720, SWP_NOZORDER):
                raise OSError("Could not resize Ornith window")
            time.sleep(9)
            capture_client(hwnd, output / "01-island-first-person.png", "geometry")

            move_cursor_to_client_center(hwnd)
            post_message(hwnd, WM_RBUTTONDOWN)
            time.sleep(0.12)
            post_message(hwnd, WM_RBUTTONUP)
            time.sleep(0.12)
            post_message(hwnd, WM_LBUTTONDOWN)
            time.sleep(0.12)
            post_message(hwnd, WM_LBUTTONUP)
            time.sleep(0.4)
            capture_client(hwnd, output / "02-rifle-fired.png", "geometry")
            tap(hwnd, "R")
            time.sleep(2.7)
            tap(hwnd, "2")
            post_message(hwnd, WM_LBUTTONDOWN)
            time.sleep(0.12)
            post_message(hwnd, WM_LBUTTONUP)
            time.sleep(0.4)
            tap(hwnd, "F")
            tap(hwnd, "C")
            tap(hwnd, "V")
            time.sleep(0.5)
            capture_client(hwnd, output / "03-third-person.png", "geometry")
            tap(hwnd, "V")
            time.sleep(0.5)
            top_down = capture_client_image(hwnd)
            if max(ImageStat.Stat(top_down).stddev) < 20:
                raise AssertionError("Top-down capture lacks visual variation")
            top_down.save(output / "04-top-down.png")
            tap(hwnd, "V")
            tap(hwnd, "B")
            time.sleep(0.5)
            capture_client(hwnd, output / "05-build-mode.png", "geometry")

            post_message(hwnd, WM_CLOSE)
            code = process.wait(timeout=8)
            if code != 0:
                raise RuntimeError(f"Ornith returned exit code {code}")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    diagnostics = stdout_path.read_text(encoding="utf-8")
    for required in ["renderer initialized", "models=17", "audio initialized", "frame statistics", "effect=rifle", "effect=pistol", "effect=reload", "effect=eating", "camera mode changed"]:
        if required not in diagnostics:
            raise AssertionError(f"Missing runtime evidence: {required}")
    if diagnostics.count("game sound playback requested") < 4:
        raise AssertionError("Expected at least four named gameplay audio requests")
    stderr = stderr_path.read_text(encoding="utf-8")
    if stderr.strip():
        raise AssertionError(f"Ornith wrote to standard error: {stderr}")
    print("Ornith native smoke: startup, models, HUD captures, guns, reload, eating, companion command, three cameras, build mode, audio requests, clean exit")
    print(f"Evidence written to {output}")


if __name__ == "__main__":
    main()
