"""Exercise jumping, stances, and stamina sprinting in the Windows release."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import time

from input_smoke import WM_KEYDOWN, WM_KEYUP, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window


TITLE = "Unbiased Bob Engine Sandbox — nerd 0.1"
VK_SPACE, VK_CONTROL, VK_Z, VK_SHIFT, VK_W = 0x20, 0x11, 0x5A, 0x10, 0x57


def key(hwnd: int, virtual_key: int, scan_code: int, pressed: bool) -> None:
    key_data = 1 | (scan_code << 16) | (0 if pressed else 0xC0000000)
    post_message(hwnd, WM_KEYDOWN if pressed else WM_KEYUP, virtual_key, key_data)


def tap(hwnd: int, virtual_key: int, scan_code: int) -> None:
    key(hwnd, virtual_key, scan_code, True)
    time.sleep(0.1)
    key(hwnd, virtual_key, scan_code, False)
    time.sleep(0.4)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/movement-smoke"))
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
            time.sleep(1.5)
            if not user32.SetWindowPos(hwnd, 0, 180, 120, 960, 640, SWP_NOZORDER):
                raise OSError("Could not resize the UBE sandbox window")
            time.sleep(0.4)
            capture_client(hwnd, output_dir / "standing.png", "textured")

            tap(hwnd, VK_CONTROL, 0x1D)
            capture_client(hwnd, output_dir / "crouching.png", "textured")
            tap(hwnd, VK_Z, 0x2C)
            capture_client(hwnd, output_dir / "prone.png", "textured")
            tap(hwnd, VK_Z, 0x2C)

            key(hwnd, VK_SPACE, 0x39, True)
            time.sleep(0.12)
            key(hwnd, VK_SPACE, 0x39, False)
            capture_client(hwnd, output_dir / "jumping.png", "textured")
            time.sleep(1.0)

            key(hwnd, VK_W, 0x11, True)
            key(hwnd, VK_SHIFT, 0x2A, True)
            time.sleep(0.7)
            capture_client(hwnd, output_dir / "sprinting.png", "textured")
            key(hwnd, VK_SHIFT, 0x2A, False)
            key(hwnd, VK_W, 0x11, False)
            time.sleep(0.4)

            post_message(hwnd, WM_CLOSE)
            return_code = process.wait(timeout=8)
            if return_code != 0:
                raise RuntimeError(f"UBE sandbox returned exit code {return_code}")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    diagnostics = stdout_path.read_text(encoding="utf-8")
    for stance in ("Crouching", "Prone", "Standing"):
        expected = f"stance={stance}"
        if expected not in diagnostics:
            raise AssertionError(f"Movement diagnostics did not include {expected!r}")
    if diagnostics.count("character stance changed") != 3:
        raise AssertionError("Ctrl/Z did not produce exactly three stance transitions")
    if diagnostics.count("character jumped") != 1:
        raise AssertionError("Space did not produce exactly one grounded jump")
    if "character sprint state changed" not in diagnostics:
        raise AssertionError("Shift+W did not start a sprint")
    if "sprinting=true" not in diagnostics or "sprinting=false" not in diagnostics:
        raise AssertionError("Sprint start/stop diagnostics are incomplete")
    if stderr_path.read_text(encoding="utf-8").strip():
        raise AssertionError("UBE sandbox wrote to standard error")
    print("Windows input: Ctrl crouch, Z prone/stand, Space jump, Shift+W sprint/stop")
    print("Movement diagnostics and five gameplay captures verified")
    print("Clean shutdown: exit code 0")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
