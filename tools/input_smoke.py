"""Exercise camera movement, mouse look, and frame diagnostics on Windows."""

from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
from pathlib import Path
import subprocess
import time

from PIL import Image

from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window


WM_KEYDOWN = 0x0100
WM_KEYUP = 0x0101
VK_D = 0x44
VK_ESCAPE = 0x1B
INPUT_MOUSE = 0
MOUSEEVENTF_MOVE = 0x0001
WM_RBUTTONDOWN = 0x0204
WM_RBUTTONUP = 0x0205


class MouseInput(ctypes.Structure):
    _fields_ = [
        ("dx", wintypes.LONG),
        ("dy", wintypes.LONG),
        ("mouse_data", wintypes.DWORD),
        ("flags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("extra_info", ctypes.c_size_t),
    ]


class InputData(ctypes.Union):
    _fields_ = [("mouse", MouseInput)]


class Input(ctypes.Structure):
    _fields_ = [("type", wintypes.DWORD), ("data", InputData)]


user32.SendInput.argtypes = [wintypes.UINT, ctypes.POINTER(Input), ctypes.c_int]
user32.SendInput.restype = wintypes.UINT
user32.SetCursorPos.argtypes = [ctypes.c_int, ctypes.c_int]
user32.SetCursorPos.restype = wintypes.BOOL


def send_mouse(flags: int, dx: int = 0, dy: int = 0) -> None:
    event = Input(type=INPUT_MOUSE)
    event.data.mouse = MouseInput(dx, dy, 0, flags, 0, 0)
    if user32.SendInput(1, ctypes.byref(event), ctypes.sizeof(Input)) != 1:
        raise ctypes.WinError(ctypes.get_last_error())


def changed_ratio(before_path: Path, after_path: Path) -> float:
    before = Image.open(before_path).convert("RGB")
    after = Image.open(after_path).convert("RGB")
    if before.size != after.size:
        raise AssertionError(f"Input captures changed dimensions: {before.size} -> {after.size}")
    changed = sum(
        max(abs(first[channel] - second[channel]) for channel in range(3)) > 30
        for first, second in zip(before.get_flattened_data(), after.get_flattened_data())
    )
    return changed / (before.width * before.height)


def post_message(hwnd: int, message: int, wparam: int = 0, lparam: int = 0) -> None:
    if not user32.PostMessageW(hwnd, message, wparam, lparam):
        raise ctypes.WinError(ctypes.get_last_error())


def move_cursor_to_client_center(hwnd: int) -> None:
    rectangle = wintypes.RECT()
    if not user32.GetClientRect(hwnd, ctypes.byref(rectangle)):
        raise ctypes.WinError(ctypes.get_last_error())
    point = wintypes.POINT(
        (rectangle.left + rectangle.right) // 2,
        (rectangle.top + rectangle.bottom) // 2,
    )
    if not user32.ClientToScreen(hwnd, ctypes.byref(point)):
        raise ctypes.WinError(ctypes.get_last_error())
    if not user32.SetCursorPos(point.x, point.y):
        raise ctypes.WinError(ctypes.get_last_error())


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/input-smoke"))
    args = parser.parse_args()

    executable = args.executable.resolve()
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    initial_path = output_dir / "input-initial.png"
    moved_path = output_dir / "input-moved.png"
    looked_path = output_dir / "input-looked.png"
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
                raise ctypes.WinError(ctypes.get_last_error())
            time.sleep(0.5)
            capture_client(hwnd, initial_path, "textured")

            post_message(hwnd, WM_KEYDOWN, VK_D)
            time.sleep(0.4)
            post_message(hwnd, WM_KEYUP, VK_D)
            time.sleep(0.3)
            capture_client(hwnd, moved_path, "textured")
            movement_change = changed_ratio(initial_path, moved_path)
            if movement_change < 0.005:
                raise AssertionError(f"Keyboard movement changed only {movement_change:.1%} of pixels")
            print(f"Keyboard movement pixel change: {movement_change:.1%}")

            move_cursor_to_client_center(hwnd)
            post_message(hwnd, WM_RBUTTONDOWN)
            time.sleep(0.1)
            post_message(hwnd, WM_RBUTTONUP)
            for _ in range(7):
                send_mouse(MOUSEEVENTF_MOVE, dx=10, dy=-3)
                time.sleep(0.05)
            time.sleep(0.3)
            capture_client(hwnd, looked_path, "textured")
            mouse_change = changed_ratio(moved_path, looked_path)
            if mouse_change < 0.005:
                raise AssertionError(f"Mouse look changed only {mouse_change:.1%} of pixels")
            print(f"Mouse look pixel change: {mouse_change:.1%}")

            post_message(hwnd, WM_KEYDOWN, VK_ESCAPE)
            post_message(hwnd, WM_KEYUP, VK_ESCAPE)
            time.sleep(0.2)
            post_message(hwnd, WM_RBUTTONDOWN)
            time.sleep(0.1)
            post_message(hwnd, WM_RBUTTONUP)
            time.sleep(0.1)
            post_message(hwnd, WM_KEYDOWN, VK_ESCAPE)
            post_message(hwnd, WM_KEYUP, VK_ESCAPE)
            time.sleep(0.4)

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
    if "frame statistics" not in diagnostics:
        raise AssertionError("Runtime did not emit frame statistics")
    print("Frame statistics: emitted")
    if diagnostics.count("cursor capture changed captured=true") < 2:
        raise AssertionError("Right-click did not capture the cursor for both mouse-look stages")
    if diagnostics.count("cursor capture changed captured=false") < 2:
        raise AssertionError("Escape did not release both cursor captures")
    print("Cursor capture: right-click captured twice; Escape released both")

    stderr = stderr_path.read_text(encoding="utf-8")
    if stderr.strip():
        raise AssertionError(f"Sandbox wrote to standard error: {stderr}")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
