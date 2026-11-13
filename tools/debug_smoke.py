"""Exercise the release inspector, persistent edits, and nonfatal reload failures."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import time

from PIL import ImageChops, ImageStat

from input_smoke import WM_KEYDOWN, WM_KEYUP, post_message
from render_smoke import SWP_NOZORDER, WM_CLOSE, capture_client, user32, wait_for_window


F3, F5, F9 = 0x72, 0x74, 0x78
RIGHT, LEFT, PAGE_UP = 0x27, 0x25, 0x21
BRACKET_RIGHT, EQUAL = 0xDD, 0xBB
SCAN_CODES = {F3: 0x3D, F5: 0x3F, F9: 0x43, RIGHT: 0x4D, LEFT: 0x4B, PAGE_UP: 0x49,
              BRACKET_RIGHT: 0x1B, EQUAL: 0x0D}
EXTENDED_KEYS = {RIGHT, LEFT, PAGE_UP}


def press(hwnd: int, key: int) -> None:
    extended = 0x01000000 if key in EXTENDED_KEYS else 0
    key_data = 1 | (SCAN_CODES[key] << 16) | extended
    post_message(hwnd, WM_KEYDOWN, key, key_data)
    time.sleep(0.03)
    post_message(hwnd, WM_KEYUP, key, key_data | 0xC0000000)
    time.sleep(0.15)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--executable", type=Path, default=Path("target/release/engine-sandbox.exe"))
    parser.add_argument("--output-dir", type=Path, default=Path("artifacts/debug-smoke"))
    args = parser.parse_args()

    executable = args.executable.resolve()
    workspace = executable.parent.parent.parent
    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = output_dir / "stdout.log"
    stderr_path = output_dir / "stderr.log"
    scene_path = workspace / "artifacts/runtime/edited.scene.json"
    scene_path.parent.mkdir(parents=True, exist_ok=True)
    scene_path.unlink(missing_ok=True)

    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        process = subprocess.Popen(
            [str(executable)], cwd=workspace, stdout=stdout, stderr=stderr
        )
        try:
            hwnd = wait_for_window("Unbiased Bob Engine Sandbox — nerd 0.1", process, timeout=20)
            time.sleep(1.2)
            if not user32.SetWindowPos(hwnd, 0, 180, 120, 960, 640, SWP_NOZORDER):
                raise OSError("Could not resize the sandbox window")
            time.sleep(0.4)
            capture_client(hwnd, output_dir / "debug-hidden.png", "textured")

            press(hwnd, F3)
            press(hwnd, F9)
            diagnostics = stdout_path.read_text(encoding="utf-8")
            if "debug scene load failed" not in diagnostics or process.poll() is not None:
                raise AssertionError("Missing scene reload was not reported as a nonfatal failure")
            for key in (RIGHT, PAGE_UP, BRACKET_RIGHT, EQUAL):
                press(hwnd, key)
            press(hwnd, F5)
            if not scene_path.is_file():
                raise AssertionError("F5 did not save the workspace scene")
            saved_bytes = scene_path.read_bytes()
            saved = json.loads(saved_bytes)
            edited = next(entity for entity in saved["entities"] if entity["model"] is not None)
            transform = edited["transform"]
            if abs(transform["translation"][0] - 1.6) > 0.0001:
                raise AssertionError(f"Arrow edit was not saved: {transform}")
            if abs(transform["translation"][1] - 0.1) > 0.0001:
                raise AssertionError(f"Height edit was not saved: {transform}")
            if abs(transform["scale"] - 1.1) > 0.0001:
                raise AssertionError(f"Scale edit was not saved: {transform}")
            if abs(transform["rotation"][1]) < 0.1:
                raise AssertionError(f"Rotation edit was not saved: {transform}")
            print(f"Saved edited local transform: {transform}")

            press(hwnd, LEFT)
            press(hwnd, F9)
            press(hwnd, F5)
            reloaded = json.loads(scene_path.read_text(encoding="utf-8"))
            restored = next(entity for entity in reloaded["entities"] if entity["model"] is not None)
            if restored["transform"] != transform:
                raise AssertionError(
                    f"F9 did not restore the edited entity: {restored['transform']} != {transform}"
                )
            diagnostics = stdout_path.read_text(encoding="utf-8")
            if "debug scene loaded" not in diagnostics or "debug entity edited" not in diagnostics:
                raise AssertionError("Structured edit/reload diagnostics were not emitted")
            print("F9 restored the saved transform after another edit")

            invalid = json.loads(saved_bytes)
            next(entity for entity in invalid["entities"] if entity["model"] is not None)["model"] = (
                "assets/models/NotUploaded.glb"
            )
            scene_path.write_text(json.dumps(invalid), encoding="utf-8")
            press(hwnd, F9)
            diagnostics = stdout_path.read_text(encoding="utf-8")
            if "reload requires already uploaded models" not in diagnostics or process.poll() is not None:
                raise AssertionError("Incompatible model reload did not fail safely")
            scene_path.write_bytes(saved_bytes)
            press(hwnd, F9)
            diagnostics = stdout_path.read_text(encoding="utf-8")
            if diagnostics.count("debug scene loaded") < 2:
                raise AssertionError("Runtime did not recover after an incompatible reload")
            print("Missing and incompatible reloads were nonfatal; valid recovery passed")
            capture_client(hwnd, output_dir / "debug-inspector.png", "textured")
            from PIL import Image
            hidden = Image.open(output_dir / "debug-hidden.png").convert("RGB")
            visible = Image.open(output_dir / "debug-inspector.png").convert("RGB")
            deviation = ImageStat.Stat(ImageChops.difference(hidden, visible)).mean
            if max(deviation) < 2.0:
                raise AssertionError(f"Inspector produced insufficient visual change: {deviation}")
            print(f"Inspector visible; mean pixel difference: {deviation}")

            press(hwnd, F3)
            post_message(hwnd, WM_CLOSE)
            return_code = process.wait(timeout=8)
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
