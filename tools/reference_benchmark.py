"""Measure complete release frame-statistic intervals for the frozen reference scene."""

from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import statistics
import subprocess
import time

from render_smoke import (
    SWP_NOZORDER,
    WM_CLOSE,
    capture_client,
    user32,
    wait_for_window,
)
from input_smoke import post_message


FRAME_PATTERN = re.compile(
    r"frame statistics frames=(\d+) fps=([0-9.]+) "
    r"average_frame_ms=([0-9.]+) maximum_frame_ms=([0-9.]+)"
)
GWL_STYLE = -16
GWL_EXSTYLE = -20

user32.GetWindowLongW.argtypes = [wintypes.HWND, ctypes.c_int]
user32.GetWindowLongW.restype = wintypes.LONG
user32.AdjustWindowRectEx.argtypes = [
    ctypes.POINTER(wintypes.RECT),
    wintypes.DWORD,
    wintypes.BOOL,
    wintypes.DWORD,
]
user32.AdjustWindowRectEx.restype = wintypes.BOOL


def resize_client(hwnd: int, width: int, height: int) -> None:
    style = user32.GetWindowLongW(hwnd, GWL_STYLE)
    extended_style = user32.GetWindowLongW(hwnd, GWL_EXSTYLE)
    rectangle = wintypes.RECT(0, 0, width, height)
    if not user32.AdjustWindowRectEx(ctypes.byref(rectangle), style, False, extended_style):
        raise ctypes.WinError(ctypes.get_last_error())
    if not user32.SetWindowPos(
        hwnd,
        0,
        100,
        80,
        rectangle.right - rectangle.left,
        rectangle.bottom - rectangle.top,
        SWP_NOZORDER,
    ):
        raise ctypes.WinError(ctypes.get_last_error())


def frame_samples(text: str) -> list[dict[str, float | int]]:
    return [
        {
            "frames": int(match.group(1)),
            "fps": float(match.group(2)),
            "average_frame_ms": float(match.group(3)),
            "maximum_frame_ms": float(match.group(4)),
        }
        for match in FRAME_PATTERN.finditer(text)
    ]


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/reference-benchmark"))
    parser.add_argument("--samples", type=int, default=10)
    args = parser.parse_args()
    if not 1 <= args.samples <= 30:
        raise ValueError("Sample count must be between 1 and 30")

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
            hwnd = wait_for_window("Unbiased Bob Engine Sandbox — nerd 0.1", process, timeout=20)
            time.sleep(1.0)
            resize_client(hwnd, 1280, 720)
            time.sleep(0.5)
            size, _ = capture_client(hwnd, output_dir / "reference-scene.png", "textured")
            if size != (1280, 720):
                raise AssertionError(f"Expected a 1280x720 client surface, got {size}")

            time.sleep(3.0)
            baseline = len(frame_samples(stdout_path.read_text(encoding="utf-8")))
            deadline = time.monotonic() + args.samples * 2.5 + 5.0
            while time.monotonic() < deadline:
                diagnostics = stdout_path.read_text(encoding="utf-8")
                if len(frame_samples(diagnostics)) >= baseline + args.samples:
                    break
                if process.poll() is not None:
                    raise RuntimeError(f"Benchmark exited early with code {process.returncode}")
                time.sleep(0.05)
            else:
                raise TimeoutError("Did not collect all complete frame-statistic intervals")

            post_message(hwnd, WM_CLOSE)
            return_code = process.wait(timeout=8)
            if return_code != 0:
                raise RuntimeError(f"Benchmark returned exit code {return_code}")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)

    diagnostics = stdout_path.read_text(encoding="utf-8")
    samples = frame_samples(diagnostics)[baseline : baseline + args.samples]
    if len(samples) != args.samples:
        raise AssertionError("Benchmark did not retain the requested sample count")
    renderer_line = next(line for line in diagnostics.splitlines() if "renderer initialized" in line)
    adapter_match = re.search(r"adapter=(.*?) backend=(\w+)", renderer_line)
    format_match = re.search(r"format=(\w+)", renderer_line)
    if not adapter_match or not format_match:
        raise AssertionError("Could not identify the measured graphics adapter and format")
    adapter, backend = adapter_match.groups()
    if backend != "Dx12":
        raise AssertionError(f"Reference benchmark used unexpected backend {backend}")
    if stderr_path.read_text(encoding="utf-8").strip():
        raise AssertionError("Benchmark wrote to standard error")

    seconds = sum(sample["frames"] / sample["fps"] for sample in samples)
    frames = sum(sample["frames"] for sample in samples)
    average_fps = frames / seconds
    minimum_fps = min(sample["fps"] for sample in samples)
    result = {
        "schema_version": 1,
        "recorded_utc": datetime.now(timezone.utc).isoformat(),
        "method": (
            "Release executable, idle character, inspector hidden, AutoVsync. "
            "Resize to an exact 1280x720 client area, capture evidence, warm up for "
            "3 seconds, then retain complete one-second engine frame-statistic intervals."
        ),
        "executable": str(executable.relative_to(workspace)),
        "executable_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
        "reference_scene": {
            "path": "assets/scenes/orbit.scene.json",
            "entities": 6,
            "rendered_model_instances": 2,
            "uploaded_models": 1,
            "model": "assets/models/BoxTextured.glb",
            "primitives_per_instance": 1,
            "static_box_colliders": 3,
            "simulation": "60 Hz fixed-step parent rotation; idle first-person character",
            "inspector_visible": False,
        },
        "surface": {
            "client_width": 1280,
            "client_height": 720,
            "format": format_match.group(1),
            "present_mode": "AutoVsync",
        },
        "machine": {
            "os": platform.platform(),
            "processor": os.environ.get("PROCESSOR_IDENTIFIER", platform.processor()),
            "adapter": adapter,
            "backend": backend,
        },
        "warmup_seconds": 3.0,
        "complete_interval_count": len(samples),
        "samples": samples,
        "summary": {
            "frames": frames,
            "measured_seconds": seconds,
            "aggregate_fps": average_fps,
            "median_interval_fps": statistics.median(sample["fps"] for sample in samples),
            "minimum_interval_fps": minimum_fps,
            "maximum_interval_fps": max(sample["fps"] for sample in samples),
            "weighted_average_frame_ms": seconds * 1000.0 / frames,
            "worst_recorded_frame_ms": max(sample["maximum_frame_ms"] for sample in samples),
            "target_fps": 60.0,
            "target_met": average_fps >= 60.0 and minimum_fps >= 60.0,
        },
        "limitations": (
            "These are application frame-loop intervals, not GPU timestamps or a percentile "
            "capture of every frame. Results apply only to this scene, machine, and configuration."
        ),
    }
    result_path = output_dir / "results.json"
    result_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(f"Adapter: {adapter}; backend: {backend}; client: 1280x720")
    print(f"Complete one-second intervals: {len(samples)}")
    print(
        f"Aggregate FPS: {average_fps:.2f}; minimum interval FPS: {minimum_fps:.2f}; "
        f"weighted average frame: {result['summary']['weighted_average_frame_ms']:.3f} ms"
    )
    print(f"60 FPS target met: {result['summary']['target_met']}")
    print("Clean shutdown: exit code 0")
    print(f"Evidence written to {output_dir}")


if __name__ == "__main__":
    main()
