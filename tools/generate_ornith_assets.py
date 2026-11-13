"""Generate original voxel GLB models and PCM WAV sounds for Surviving Ornith."""

from __future__ import annotations

import json
import math
from pathlib import Path
import random
import struct
import wave

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
MODEL_DIR = ROOT / "assets" / "ornith" / "models"
AUDIO_DIR = ROOT / "assets" / "ornith" / "audio"
SEED = 0x0B17_2026


class VoxelMesh:
    def __init__(self, palette: list[tuple[int, int, int]]) -> None:
        if not 1 <= len(palette) <= 16:
            raise ValueError("A model palette must contain 1..16 colors")
        self.palette = palette
        self.positions: list[tuple[float, float, float]] = []
        self.normals: list[tuple[float, float, float]] = []
        self.uvs: list[tuple[float, float]] = []
        self.indices: list[int] = []

    def box(self, center, size, color: int = 0) -> None:
        cx, cy, cz = center
        x, y, z = (component / 2 for component in size)
        faces = [
            ([(-x,-y,z),(x,-y,z),(x,y,z),(-x,y,z)], (0,0,1)),
            ([(x,-y,-z),(-x,-y,-z),(-x,y,-z),(x,y,-z)], (0,0,-1)),
            ([(x,-y,z),(x,-y,-z),(x,y,-z),(x,y,z)], (1,0,0)),
            ([(-x,-y,-z),(-x,-y,z),(-x,y,z),(-x,y,-z)], (-1,0,0)),
            ([(-x,y,z),(x,y,z),(x,y,-z),(-x,y,-z)], (0,1,0)),
            ([(-x,-y,-z),(x,-y,-z),(x,-y,z),(-x,-y,z)], (0,-1,0)),
        ]
        tile_x, tile_y = color % 4, color // 4
        u0, v0 = (tile_x * 16 + 0.5) / 64, (tile_y * 16 + 0.5) / 64
        u1, v1 = (tile_x * 16 + 15.5) / 64, (tile_y * 16 + 15.5) / 64
        for corners, normal in faces:
            start = len(self.positions)
            self.positions.extend((cx+a, cy+b, cz+c) for a,b,c in corners)
            self.normals.extend([normal] * 4)
            self.uvs.extend([(u0,v1),(u1,v1),(u1,v0),(u0,v0)])
            self.indices.extend([start,start+1,start+2,start,start+2,start+3])

    def save(self, name: str) -> None:
        rng = random.Random(SEED + sum(map(ord, name)))
        pixels = np.zeros((64, 64, 4), dtype=np.uint8)
        pixels[:, :, 3] = 255
        for index, color in enumerate(self.palette):
            tx, ty = index % 4 * 16, index // 4 * 16
            pixels[ty:ty+16, tx:tx+16, :3] = color
            # Seeded, subtle face grain keeps the flat-color voxel surfaces tactile.
            for _ in range(22):
                px, py = tx + rng.randrange(16), ty + rng.randrange(16)
                delta = rng.choice((-7, -4, 4, 7))
                pixels[py, px, :3] = np.clip(np.array(color) + delta, 0, 255)
        image = Image.fromarray(pixels, "RGBA")
        from io import BytesIO
        png_buffer = BytesIO()
        image.save(png_buffer, format="PNG")
        png = png_buffer.getvalue()

        arrays = [self.positions, self.normals, self.uvs]
        binary = bytearray()
        views, accessors = [], []
        for array, kind, accessor_type in zip(arrays, ["POSITION", "NORMAL", "TEXCOORD_0"], ["VEC3", "VEC3", "VEC2"]):
            while len(binary) % 4:
                binary.append(0)
            offset = len(binary)
            flat = [component for vertex in array for component in vertex]
            binary.extend(struct.pack("<" + "f" * len(flat), *flat))
            view = len(views)
            views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(flat)*4, "target": 34962})
            accessor = {"bufferView": view, "componentType": 5126, "count": len(array), "type": accessor_type}
            if kind == "POSITION":
                accessor["min"] = [min(v[i] for v in array) for i in range(3)]
                accessor["max"] = [max(v[i] for v in array) for i in range(3)]
            accessors.append(accessor)
        while len(binary) % 4:
            binary.append(0)
        index_offset = len(binary)
        binary.extend(struct.pack("<" + "I" * len(self.indices), *self.indices))
        views.append({"buffer": 0, "byteOffset": index_offset, "byteLength": len(self.indices)*4, "target": 34963})
        index_accessor = len(accessors)
        accessors.append({"bufferView": len(views)-1, "componentType": 5125, "count": len(self.indices), "type": "SCALAR"})
        while len(binary) % 4:
            binary.append(0)
        image_offset = len(binary)
        binary.extend(png)
        views.append({"buffer": 0, "byteOffset": image_offset, "byteLength": len(png)})
        document = {
            "asset": {"version": "2.0", "generator": "Surviving Ornith original procedural voxel generator"},
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"mesh": 0, "name": name}],
            "meshes": [{"name": name, "primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1, "TEXCOORD_0": 2}, "indices": index_accessor, "material": 0}]}],
            "materials": [{"name": f"{name} palette", "pbrMetallicRoughness": {"baseColorTexture": {"index": 0}, "metallicFactor": 0.0, "roughnessFactor": 1.0}}],
            "textures": [{"source": 0, "sampler": 0}],
            "samplers": [{"magFilter": 9729, "minFilter": 9729, "wrapS": 33071, "wrapT": 33071}],
            "images": [{"bufferView": len(views)-1, "mimeType": "image/png"}],
            "buffers": [{"byteLength": len(binary)}],
            "bufferViews": views,
            "accessors": accessors,
        }
        encoded = json.dumps(document, separators=(",", ":")).encode()
        encoded += b" " * ((-len(encoded)) % 4)
        while len(binary) % 4:
            binary.append(0)
        glb = struct.pack("<III", 0x46546C67, 2, 12 + 8 + len(encoded) + 8 + len(binary))
        glb += struct.pack("<II", len(encoded), 0x4E4F534A) + encoded
        glb += struct.pack("<II", len(binary), 0x004E4942) + binary
        (MODEL_DIR / f"{name}.glb").write_bytes(glb)


def models() -> dict[str, int]:
    made = {}
    def save(name, mesh):
        mesh.save(name)
        made[name] = len(mesh.indices) // 3

    grass = [(80,132,67),(95,145,73),(111,154,80),(189,169,112),(203,184,128),(113,86,58),(105,111,105)]
    terrain = VoxelMesh(grass)
    for x in range(-15, 15):
        for z in range(-15, 15):
            edge = max(abs(x), abs(z))
            color = 3 + ((x-z) & 1) if edge >= 13 else (x*7+z*11) % 3
            height = -0.75 - (0.25 if edge == 14 else 0)
            terrain.box((x*1.4+0.7,height,z*1.4+0.7),(1.4,1.5,1.4),color)
    save("terrain", terrain)
    water = VoxelMesh([(39,117,145)])
    water.box((0,-0.15,0),(150,0.3,150))
    save("water", water)

    tree = VoxelMesh([(105,72,42),(132,91,50),(44,104,54),(58,125,59),(76,143,66)])
    tree.box((0,1.45,0),(0.65,2.9,0.65),0)
    tree.box((0.12,2.2,0),(0.42,1.1,0.42),1)
    tree.box((0,3.2,0),(2.6,1.35,2.4),2)
    tree.box((-0.65,3.8,0.15),(1.5,1.1,1.6),3)
    tree.box((0.7,3.75,-0.25),(1.5,1.2,1.5),4)
    tree.box((0,4.45,0),(1.3,0.75,1.3),3)
    save("tree", tree)
    ore = VoxelMesh([(91,98,99),(115,123,119),(50,181,174),(100,224,199),(210,137,64)])
    ore.box((0,0.5,0),(1.5,1.0,1.35),0)
    ore.box((-0.4,1.0,0.1),(0.8,0.55,0.85),1)
    ore.box((0.55,0.55,0.69),(0.3,0.35,0.08),2)
    ore.box((-0.4,1.3,0.1),(0.32,0.12,0.35),3)
    ore.box((0.75,0.55,-0.25),(0.08,0.3,0.32),4)
    ore.box((-0.76,0.45,0.2),(0.08,0.26,0.3),2)
    save("ore", ore)
    berry = VoxelMesh([(47,111,59),(70,138,67),(193,61,91),(238,104,118)])
    berry.box((0,0.45,0),(1.35,0.9,1.2),0)
    berry.box((-0.3,0.95,0),(0.8,0.55,0.75),1)
    for center in [(-.5,.55,.61),(.35,.7,.61),(-.25,1.05,.39),(.52,.4,.4)]:
        berry.box(center,(.18,.18,.12),2)
    save("berry", berry)
    flower = VoxelMesh([(57,126,62),(239,202,89),(238,121,141),(185,216,230)])
    flower.box((0,.25,0),(.08,.5,.08),0)
    flower.box((0,.52,0),(.38,.14,.38),1)
    flower.box((0,.58,0),(.16,.12,.16),2)
    save("flower", flower)
    cloud = VoxelMesh([(218,235,230),(238,244,229)])
    cloud.box((0,0,0),(4.5,1.1,1.8),0)
    cloud.box((-1.0,.7,0),(2.1,1.0,1.6),1)
    cloud.box((1.2,.45,.1),(2.2,.8,1.5),1)
    save("cloud", cloud)

    wolf = VoxelMesh([(105,116,119),(137,148,148),(66,77,82),(220,224,207),(232,176,67)])
    wolf.box((0,.85,0),(0.85,.75,1.65),0)
    wolf.box((0,1.25,-.85),(.78,.75,.75),1)
    wolf.box((0,1.12,-1.3),(.5,.35,.45),2)
    for x in [-.28,.28]:
        for z in [-.52,.52]: wolf.box((x,.35,z),(.22,.7,.24),2)
        wolf.box((x,1.77,-.85),(.22,.45,.25),2)
        wolf.box((x*.65,1.38,-1.24),(.1,.1,.06),4)
    wolf.box((0,1.05,1.05),(.25,.25,.9),1)
    save("wolf", wolf)
    cow = VoxelMesh([(218,210,176),(245,234,202),(104,76,58),(69,60,50),(226,160,146)])
    cow.box((0,1.0,0),(1.2,1.05,2.0),0)
    cow.box((0,1.35,-1.12),(1.0,.95,.85),1)
    cow.box((0,1.12,-1.58),(.72,.42,.28),4)
    cow.box((.42,1.25,.45),(.45,.55,.65),2)
    cow.box((-.45,.85,-.45),(.4,.45,.55),2)
    for x in [-.42,.42]:
        for z in [-.65,.65]: cow.box((x,.35,z),(.25,.7,.28),3)
        cow.box((x,1.94,-1.1),(.35,.16,.18),1)
        cow.box((x*.55,1.52,-1.55),(.1,.1,.06),3)
    save("cow", cow)
    hawk = VoxelMesh([(111,75,46),(157,111,59),(225,192,120),(235,215,163),(49,49,43)])
    hawk.box((0,.35,0),(.55,.65,1.0),0)
    hawk.box((0,.75,-.48),(.48,.5,.48),2)
    hawk.box((0,.68,-.82),(.22,.18,.3),1)
    hawk.box((-.65,.42,.05),(.85,.18,1.25),1)
    hawk.box((.65,.42,.05),(.85,.18,1.25),1)
    hawk.box((0,.25,.72),(.42,.16,.55),3)
    for x in [-.13,.13]: hawk.box((x,.85,-.71),(.07,.07,.04),4)
    save("hawk", hawk)

    survivor = VoxelMesh([(47,91,94),(65,122,118),(220,178,126),(57,55,51),(224,194,74)])
    survivor.box((0,1.25,0),(.75,.85,.45),0)
    survivor.box((0,1.95,0),(.58,.58,.55),2)
    for x in [-.23,.23]: survivor.box((x,.45,0),(.27,.85,.3),3)
    for x in [-.52,.52]: survivor.box((x,1.25,0),(.25,.8,.28),1)
    survivor.box((0,1.45,.3),(.55,.55,.22),4)
    save("survivor", survivor)

    rifle = VoxelMesh([(87,57,35),(133,88,46),(58,65,67),(117,132,128)])
    rifle.box((0,0,.15),(.18,.22,1.2),0)
    rifle.box((0,.04,-.7),(.09,.09,1.0),2)
    rifle.box((0,.16,-.25),(.12,.14,.45),3)
    rifle.box((0,-.18,.42),(.13,.35,.25),1)
    save("rifle", rifle)
    pistol = VoxelMesh([(62,71,75),(116,132,133),(87,57,35)])
    pistol.box((0,0,-.25),(.18,.2,.65),0)
    pistol.box((0,-.25,.02),(.16,.45,.24),2)
    pistol.box((0,.08,-.58),(.1,.08,.18),1)
    save("pistol", pistol)
    pickaxe = VoxelMesh([(122,80,43),(75,88,91),(142,157,153)])
    pickaxe.box((0,0,0),(.12,1.0,.12),0)
    pickaxe.box((0,.48,0),(.95,.16,.18),1)
    pickaxe.box((-.48,.37,0),(.16,.35,.16),2)
    save("pickaxe", pickaxe)
    axe = VoxelMesh([(122,80,43),(130,143,139),(75,88,91)])
    axe.box((0,0,0),(.13,1.0,.13),0)
    axe.box((.23,.37,0),(.55,.45,.16),1)
    axe.box((.5,.37,0),(.08,.5,.18),2)
    save("axe", axe)
    wall = VoxelMesh([(135,91,49),(157,107,57),(99,68,42)])
    for x in [-.9,0,.9]: wall.box((x,1.2,0),(.84,2.4,.24),int((x+1)*2)%2)
    for y in [.25,2.15]: wall.box((0,y,.15),(2.9,.18,.14),2)
    save("wall", wall)
    foundation = VoxelMesh([(135,91,49),(157,107,57),(99,68,42)])
    for x in [-1.05,-.35,.35,1.05]: foundation.box((x,.15,0),(.65,.3,2.8),int(x*10)&1)
    save("foundation", foundation)
    return made


def write_sound(name: str, samples: np.ndarray, rate: int = 22050) -> None:
    samples = np.clip(samples, -0.95, 0.95)
    with wave.open(str(AUDIO_DIR / f"{name}.wav"), "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(rate)
        output.writeframes((samples * 32767).astype("<i2").tobytes())


def sounds() -> None:
    rate = 22050
    rng = np.random.default_rng(SEED)
    def timeline(seconds): return np.arange(int(rate*seconds))/rate
    def noise_burst(seconds, decay, tone=0):
        t=timeline(seconds)
        return (rng.uniform(-1,1,len(t))*.6 + np.sin(2*np.pi*tone*t)*.4)*np.exp(-t*decay)
    write_sound("rifle", noise_burst(.7,7,72)*.9)
    write_sound("pistol", noise_burst(.28,17,145)*.7)
    t=timeline(.32); write_sound("reload", np.sin(2*np.pi*880*t)*(np.sin(2*np.pi*8*t)>.75)*.22)
    write_sound("mining", noise_burst(.22,19,620)*.55)
    write_sound("chopping", noise_burst(.25,15,105)*.65)
    write_sound("building", noise_burst(.32,11,180)*.5)
    write_sound("eating", noise_burst(.38,10,85)*.35)
    write_sound("hurt", noise_burst(.4,8,58)*.5)
    write_sound("footstep", noise_burst(.13,28,92)*.24)
    t=timeline(.65); melody=np.zeros_like(t)
    for start,frequency in [(0,523),(.14,659),(.28,784)]:
        local=np.maximum(0,t-start); envelope=(t>=start)*np.exp(-local*7)
        melody += np.sin(2*np.pi*frequency*local)*envelope*.22
    write_sound("taming", melody)
    t=timeline(.22); write_sound("empty", np.sin(2*np.pi*190*t)*np.exp(-t*28)*.3)


def main() -> None:
    MODEL_DIR.mkdir(parents=True, exist_ok=True)
    AUDIO_DIR.mkdir(parents=True, exist_ok=True)
    triangles = models()
    sounds()
    manifest = {"seed": SEED, "origin": "Original procedural assets for Surviving Ornith", "models": triangles, "audio": sorted(p.stem for p in AUDIO_DIR.glob("*.wav"))}
    (MODEL_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Generated {len(triangles)} GLB models ({sum(triangles.values()):,} triangles) and {len(manifest['audio'])} WAV clips")


if __name__ == "__main__":
    main()
