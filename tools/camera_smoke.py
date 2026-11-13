"""Exercise UBE's three gameplay cameras in the real Windows release."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import time

from input_smoke import WM_KEYDOWN, WM_KEYUP, changed_ratio, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window


TITLE = "Unbiased Bob Engine Sandbox — nerd 0.1"
VK_V, VK_W = 0x56, 0x57


def key(hwnd: int, virtual_key: int, scan_code: int, pressed: bool) -> None:
    key_data = 1 | (scan_code << 16) | (0 if pressed else 0xC0000000)
    post_message(hwnd, WM_KEYDOWN if pressed else WM_KEYUP, virtual_key, key_data)


def tap_v(hwnd: int) -> None:
    key(hwnd, VK_V, 0x2F, True)
    key(hwnd, VK_V, 0x2F, False)
    time.sleep(0.35)


def move_forward(hwnd: int) -> None:
    key(hwnd, VK_W, 0x11, True)
    time.sleep(0.35)
    key(hwnd, VK_W, 0x11, False)
    time.sleep(0.25)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/camera-smoke"))
    args = parser.parse_args()
    executable = args.executable.resolve()
    workspace = executable.parent.parent.parent
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = output_dir / "stdout.log"
    stderr_path = output_dir / "stderr.log"

    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(
            [str(executable)], cwd=workspace, stdout=stdout, stderr=stderr
        )
        try:
            hwnd = wait_for_window(TITLE, process, timeout=20)
            time.sleep(1.2)
            if not user32.SetWindowPos(hwnd, 0, 180, 120, 960, 640, SWP_NOZORDER):
                raise OSError("Could not resize the UBE sandbox window")
            time.sleep(0.4)
            capture_client(hwnd, output_dir / "first-person.png", "textured")

            tap_v(hwnd)
            third_path = output_dir / "third-person.png"
            capture_client(hwnd, third_path, "textured")
            third_change = changed_ratio(output_dir / "first-person.png", third_path)
            if third_change < 0.01:
                raise AssertionError(f"Third-person camera changed only {third_change:.1%} of pixels")
            move_forward(hwnd)

            tap_v(hwnd)
            top_down_path = output_dir / "top-down.png"
            capture_client(hwnd, top_down_path, "textured")
            top_down_change = changed_ratio(third_path, top_down_path)
            if top_down_change < 0.01:
                raise AssertionError(f"Top-down camera changed only {top_down_change:.1%} of pixels")
            move_forward(hwnd)

            tap_v(hwnd)
            time.sleep(0.2)
            capture_client(hwnd, output_dir / "first-person-restored.png", "textured")
            post_message(hwnd, WM_CLOSE)
            return_code = process.wait(timeout=8)
            if return_code != 0:
                raise RuntimeError(f"UBE sandbox returned exit code {return_code}")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    diagnostics = stdout_path.read_text(encoding="utf-8")
    expected = (
        'engine="Unbiased Bob Engine"',
        'short_name="UBE"',
        'release="nerd 0.1"',
        "mode=ThirdPerson",
        "mode=TopDown",
        "mode=FirstPerson",
    )
    for text in expected:
        if text not in diagnostics:
            raise AssertionError(f"Camera/branding diagnostics did not include {text!r}")
    if diagnostics.count("camera mode changed") != 3:
        raise AssertionError("V did not complete exactly one three-mode camera cycle")
    if stderr_path.read_text(encoding="utf-8").strip():
        raise AssertionError("UBE sandbox wrote to standard error")
    print(f"Third-person pixel change: {third_change:.1%}")
    print(f"Top-down pixel change: {top_down_change:.1%}")
    print("V cycle: first -> third -> top-down -> first; movement exercised in follow views")
    print("UBE name and nerd 0.1 release diagnostics verified")
    print("Clean shutdown: exit code 0")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
