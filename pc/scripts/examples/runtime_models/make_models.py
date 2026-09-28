"""Write original tiny test parts; no Blender, game disc, or third-party art needed.

Run with Python 3 from any directory, then install this folder as scripts/runtime_models.
The art lane can replace these files with production GXMS v2 parts and a shared atlas.
"""
import json
from pathlib import Path
import struct


def write_part(root, name, slope):
    # Six flat-shaded faces, 24 vertices, indexed triangles; X/Y are gameplay axes.
    corners = [(-15, -3, -5), (15, -3 + slope, -5), (15, slope, -5), (-15, 0, -5),
               (-15, -3, 5), (15, -3 + slope, 5), (15, slope, 5), (-15, 0, 5)]
    faces = [(0, 3, 2, 1), (4, 5, 6, 7), (0, 4, 7, 3),
             (1, 2, 6, 5), (3, 7, 6, 2), (0, 1, 5, 4)]
    vertices, indices = [], []
    for face in faces:
        a, b, c = [corners[i] for i in face[:3]]
        u, v = [b[i] - a[i] for i in range(3)], [c[i] - a[i] for i in range(3)]
        normal = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
        length = sum(n*n for n in normal) ** 0.5
        normal = [n / length for n in normal]
        base = len(vertices)
        for index, uv in zip(face, [(0, 0), (1, 0), (1, 1), (0, 1)]):
            vertices.append((*corners[index], *uv, *normal))
        indices.extend(base + i for i in [0, 1, 2, 0, 2, 3])
    header = struct.pack(">4sIIIfffII", b"GXMS", 2, len(vertices), len(indices),
                         30, 10, 3 + slope, 36, 36 + len(vertices) * 32)
    (root / f"{name}.gxmesh").write_bytes(header + b"".join(struct.pack(">8f", *v) for v in vertices)
                                           + struct.pack(f">{len(indices)}H", *indices))
    (root / f"{name}.coll.json").write_text(json.dumps({
        "version": 1, "atlas": "kit", "lines": [["floor", -15, 0, 15, slope, 3]]
    }, indent=2) + "\n", encoding="utf-8")


def main():
    root = Path(__file__).resolve().parent / "models"
    root.mkdir(exist_ok=True)
    # GX RGBA8 is one 4x4 tile: sixteen AR pairs followed by sixteen GB pairs.
    header = struct.pack(">11I", 0x47585458, 1, 6, 4, 4, 0xFFFFFFFF, 0, 64, 0, 64, 0)
    (root / "kit.gxtex").write_bytes(header.ljust(64, b"\0") + bytes([255, 56])*16 + bytes([190, 210])*16)
    write_part(root, "deck", 0)
    write_part(root, "ramp", 8)
    print(f"Wrote two GXMS v2 models, sidecars, and shared atlas to {root}")


if __name__ == "__main__":
    main()
