"""Exercise the Windows renderer and capture presentation evidence."""

from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
from pathlib import Path
import subprocess
import time

from PIL import ImageStat

from window_capture import capture_client_image


WM_CLOSE = 0x0010
SW_MINIMIZE = 6
SW_RESTORE = 9
SWP_NOSIZE = 0x0001
SWP_NOMOVE = 0x0002
SWP_NOZORDER = 0x0004
HWND_TOPMOST = -1

user32 = ctypes.WinDLL("user32", use_last_error=True)
user32.FindWindowW.argtypes = [wintypes.LPCWSTR, wintypes.LPCWSTR]
user32.FindWindowW.restype = wintypes.HWND
user32.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
user32.GetClientRect.restype = wintypes.BOOL
user32.ClientToScreen.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.POINT)]
user32.ClientToScreen.restype = wintypes.BOOL
user32.SetWindowPos.argtypes = [
    wintypes.HWND,
    wintypes.HWND,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_int,
    wintypes.UINT,
]
user32.SetWindowPos.restype = wintypes.BOOL
user32.ShowWindow.argtypes = [wintypes.HWND, ctypes.c_int]
user32.ShowWindow.restype = wintypes.BOOL
user32.BringWindowToTop.argtypes = [wintypes.HWND]
user32.BringWindowToTop.restype = wintypes.BOOL
user32.SetForegroundWindow.argtypes = [wintypes.HWND]
user32.SetForegroundWindow.restype = wintypes.BOOL
user32.PostMessageW.argtypes = [wintypes.HWND, wintypes.UINT, wintypes.WPARAM, wintypes.LPARAM]
user32.PostMessageW.restype = wintypes.BOOL


def wait_for_window(title: str, process: subprocess.Popen[bytes], timeout: float) -> int:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"Sandbox exited early with code {process.returncode}")
        hwnd = user32.FindWindowW(None, title)
        if hwnd:
            return hwnd
        time.sleep(0.1)
    raise TimeoutError(f"Window {title!r} did not appear within {timeout} seconds")


def capture_client(
    hwnd: int, destination: Path, expectation: str
) -> tuple[tuple[int, int], tuple[int, int, int]]:
    if not user32.SetWindowPos(
        hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE
    ):
        raise ctypes.WinError(ctypes.get_last_error())
    user32.BringWindowToTop(hwnd)
    user32.SetForegroundWindow(hwnd)
    time.sleep(0.25)

    image = capture_client_image(hwnd)
    image.save(destination)

    center = image.getpixel((image.width // 2, image.height // 2))
    deviation = ImageStat.Stat(image).stddev
    if expectation == "clear":
        if not (center[2] > center[1] > center[0] and center[2] > 50):
            raise AssertionError(f"Unexpected clear-frame center pixel: {center}")
        if max(deviation) > 20:
            raise AssertionError(f"Clear frame has unexpected visual variation: {deviation}")
    else:
        background = (63, 108, 149)
        pixels = list(image.get_flattened_data())
        changed = sum(
            max(abs(pixel[channel] - background[channel]) for channel in range(3)) > 30
            for pixel in pixels
        ) / (image.width * image.height)
        minimum_coverage = 0.005 if expectation == "textured" else 0.03
        if not minimum_coverage < changed < 0.95:
            raise AssertionError(f"Unexpected geometry coverage: {changed:.1%}")
        minimum_deviation = 5 if expectation == "textured" else 20
        if max(deviation) < minimum_deviation:
            raise AssertionError(f"Geometry frame lacks visual variation: {deviation}")
        if expectation == "textured":
            green_pixels = sum(
                pixel[1] > 40 and pixel[1] > pixel[0] * 1.15 and pixel[1] > pixel[2] * 1.1
                for pixel in pixels
            )
            if green_pixels < 100:
                raise AssertionError(
                    f"Textured model lacks the sample's expected green logo pixels: {green_pixels}"
                )
            print(f"Textured green pixels: {green_pixels}")
        print(f"Geometry coverage: {changed:.1%}")
    return image.size, center


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/render-smoke"))
    parser.add_argument("--expect", choices=["clear", "geometry", "textured"], default="textured")
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
            time.sleep(1)
            if not user32.SetWindowPos(hwnd, 0, 100, 100, 1280, 720, SWP_NOZORDER):
                raise ctypes.WinError(ctypes.get_last_error())
            time.sleep(0.5)
            size, center = capture_client(
                hwnd, output_dir / f"{args.expect}-1280.png", args.expect
            )
            print(f"Initial frame: size={size}, center={center}")

            if not user32.SetWindowPos(hwnd, 0, 180, 120, 960, 640, SWP_NOZORDER):
                raise ctypes.WinError(ctypes.get_last_error())
            time.sleep(0.5)
            size, center = capture_client(
                hwnd, output_dir / f"{args.expect}-resized.png", args.expect
            )
            print(f"Resized frame: size={size}, center={center}")

            user32.ShowWindow(hwnd, SW_MINIMIZE)
            time.sleep(0.4)
            user32.ShowWindow(hwnd, SW_RESTORE)
            time.sleep(0.6)
            size, center = capture_client(
                hwnd, output_dir / f"{args.expect}-restored.png", args.expect
            )
            print(f"Restored frame: size={size}, center={center}")

            if not user32.PostMessageW(hwnd, WM_CLOSE, 0, 0):
                raise ctypes.WinError(ctypes.get_last_error())
            return_code = process.wait(timeout=5)
            if return_code != 0:
                raise RuntimeError(f"Sandbox returned exit code {return_code}")
            print("Clean shutdown: exit code 0")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
