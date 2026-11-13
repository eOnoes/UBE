"""Generate the original deterministic footstep WAV used by the sandbox."""

from __future__ import annotations

import math
from pathlib import Path
import random
import struct
import wave


SAMPLE_RATE = 44_100
DURATION_SECONDS = 0.12


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    destination = root / "assets" / "audio" / "footstep.wav"
    destination.parent.mkdir(parents=True, exist_ok=True)
    random_source = random.Random(417)
    samples = []
    for index in range(int(SAMPLE_RATE * DURATION_SECONDS)):
        time_seconds = index / SAMPLE_RATE
        envelope = math.exp(-time_seconds * 38.0) * min(1.0, time_seconds * 500.0)
        body = math.sin(2.0 * math.pi * 95.0 * time_seconds) * 0.55
        body += math.sin(2.0 * math.pi * 160.0 * time_seconds) * 0.20
        noise = random_source.uniform(-1.0, 1.0) * 0.25
        value = int(max(-1.0, min(1.0, (body + noise) * envelope * 0.55)) * 32767)
        samples.append(struct.pack("<h", value))

    with wave.open(str(destination), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(SAMPLE_RATE)
        output.writeframes(b"".join(samples))
    print(f"Generated {destination} ({len(samples)} mono samples)")


if __name__ == "__main__":
    main()
