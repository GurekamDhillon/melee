"""Catalog contract without importing Blender or generating any art/binaries."""
import ast
from pathlib import Path
import re
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
EXPORT = ROOT / "pc/assets_src/bf_interior/export_kit.py"
EDITOR = ROOT / "pc/scripts/examples/map_editor/scripts/main.lua"


def exporter_namespace():
    tree = ast.parse(EXPORT.read_text(encoding="utf-8"))
    nodes = [n for n in tree.body if
             (isinstance(n, ast.Assign) and isinstance(n.targets[0], ast.Name)
              and n.targets[0].id in ("KIT_SCALE", "UNIT")) or
             (isinstance(n, ast.FunctionDef) and n.name == "write_editor_catalog")]
    ns = {}
    exec(compile(ast.Module(body=nodes, type_ignores=[]), str(EXPORT), "exec"), ns)
    return ns


class CatalogTest(unittest.TestCase):
    def test_checked_in_catalog_matches_export(self):
        ns = exporter_namespace()
        source = EDITOR.read_text(encoding="utf-8")
        block = source.split("-- BEGIN GENERATED KIT\n")[1].split("-- END GENERATED KIT")[0]
        self.assertEqual(float(re.search(r"local U = ([\d.]+)", block)[1]), ns["UNIT"])
        expected = sorted(p.name.removesuffix(".coll.json") for p in
                          (ROOT / "pc/scripts/examples/bf_interior_room/models").glob("*.coll.json"))
        self.assertEqual(re.findall(r'"(bf_[a-z0-9_]+)"', block), expected)

    def test_export_updates_scale_and_only_emitted_parts(self):
        ns = exporter_namespace()
        ns["UNIT"] = 13
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "main.lua"
            path.write_text("before\n-- BEGIN GENERATED KIT\nstale\n-- END GENERATED KIT\nafter\n")
            ns["write_editor_catalog"](path, ["bf_wall", "bf_floor"])
            text = path.read_text()
            self.assertIn("local U = 13", text)
            self.assertLess(text.index('"bf_floor"'), text.index('"bf_wall"'))
            self.assertTrue(text.startswith("before\n") and text.endswith("\nafter\n"))
            path.write_text("missing markers")
            with self.assertRaises(RuntimeError):
                ns["write_editor_catalog"](path, [])


if __name__ == "__main__":
    unittest.main()
