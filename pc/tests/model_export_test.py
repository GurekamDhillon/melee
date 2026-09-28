"""Exporter tests without Blender, asset generation, or a game build.

Run: python pc/tests/model_export_test.py
Only the named pure functions are loaded from the Blender entry points.
"""
import ast
from pathlib import Path
import struct
import tempfile
from types import SimpleNamespace as NS
import unittest

import numpy as np

ROOT = Path(__file__).resolve().parents[2]


def functions(path, names, **namespace):
    tree = ast.parse((ROOT / path).read_text(encoding="utf-8"))
    tree.body = [node for node in tree.body
                 if isinstance(node, ast.FunctionDef) and node.name in names]
    exec(compile(tree, str(path), "exec"), namespace)
    return namespace


class ModelExportTest(unittest.TestCase):
    def test_glass_mip_alpha(self):
        writer = functions("pc/assets_src/bf_platform/export.py", {"write_gxtex"},
                           np=np, struct=struct)["write_gxtex"]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "glass.gxtex"
            writer(path, bytes([50, 100, 200, 26]) * (128 * 128), 128)
            data = path.read_bytes()[64:]
            # GX RGBA8: every tile begins with 16 interleaved alpha/red pairs.
            for start, size in ((0, 128), (128 * 128 * 4, 64)):
                for tile in range(start, start + size * size * 4, 64):
                    self.assertEqual(data[tile:tile + 32:2], bytes([26]) * 16)

    def test_merged_part_and_glass_sidecar(self):
        kit = functions("pc/assets_src/bf_interior/export_kit.py", {"write_part", "sidecar"},
                        UNIT=6.5, HEADER=struct.Struct(">4sIIIfffII"), struct=struct)
        # Two triangles from separate material slots still share one stream.
        co = [NS(x=0, y=0, z=0), NS(x=1, y=0, z=0), NS(x=0, y=0, z=1)]
        mesh = NS(
            uv_layers={"UVBake": NS(data=[NS(uv=NS(x=0, y=0)) for _ in range(6)])},
            attributes={"part_id": NS(data=[NS(value=0), NS(value=0)])},
            loop_triangles=[NS(polygon_index=0, material_index=0, loops=[0, 1, 2]),
                            NS(polygon_index=1, material_index=1, loops=[3, 4, 5])],
            loops=[NS(vertex_index=i % 3) for i in range(6)],
            vertices=[NS(co=v) for v in co],
            corner_normals=[NS(vector=NS(x=0, y=-1, z=0)) for _ in range(6)])
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "part.gxmesh"
            self.assertEqual(kit["write_part"](path, mesh, 0, 0, (4, 1, 1)), (3, 2))
            header = struct.unpack(">4sIIIfffII", path.read_bytes()[:36])
            self.assertEqual(header[:4], (b"GXMS", 2, 3, 6))
        self.assertEqual(kit["sidecar"]([], True),
                         {"version": 1, "atlas": "bf_kit_glass", "lines": [], "alpha": 1})
        self.assertNotIn("alpha", kit["sidecar"]([]))


if __name__ == "__main__":
    unittest.main()
