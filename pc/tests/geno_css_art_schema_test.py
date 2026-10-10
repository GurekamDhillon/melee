"""Validate the game-owned presentation contract with synthetic names only."""
import json
from pathlib import Path
import unittest

from jsonschema import Draft202012Validator

SCHEMA = Path(__file__).resolve().parents[1] / "geno" / "presentation.schema.json"


class PresentationSchemaTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(cls.schema)
        cls.validator = Draft202012Validator(cls.schema)

    def test_all_art_slots_and_255_costumes(self):
        self.validator.validate({
            "icon": "icon.gxtex", "portrait": ["p.gxtex"] * 255,
            "stock": ["s.gxtex"] * 255, "results": ["r.gxtex"] * 255,
            "emblem": "emblem.gxtex",
        })
        self.validator.validate({"results": "card.GXTEX"})
        self.validator.validate({})

    def test_bad_names_lists_and_unknown_fields(self):
        for field in ("icon", "portrait", "stock", "results", "emblem"):
            for value in ("", "a.png", "../a.gxtex", "a..b.gxtex",
                          "sub/a.gxtex", "sub\\a.gxtex", "C:a.gxtex",
                          "with space.gxtex", "x" * 58 + ".gxtex",
                          "a.gxtex\n", [], ["a.gxtex"] * 256, 7):
                with self.subTest(field=field, value=value):
                    self.assertFalse(self.validator.is_valid({field: value}))
        self.assertFalse(self.validator.is_valid({"icon": ["a.gxtex"]}))
        self.assertFalse(self.validator.is_valid({"emblem": ["a.gxtex"]}))
        self.assertFalse(self.validator.is_valid({"banner": "a.gxtex"}))



class WorkspacePresentationTest(unittest.TestCase):
    """Run from the workspace with PYTHONPATH=. after applying the handoff patch."""
    def test_generator_imports_game_contract(self):
        from tools.geno import schema
        generated = schema.build_schema()
        actual = generated["properties"]["fighters"]["items"]["properties"]["presentation"]
        self.assertEqual(actual, json.loads(SCHEMA.read_text(encoding="utf-8")))
        self.assertIn("pc/geno/presentation.schema.json", generated["x-source-contract"])

    def test_results_file_validation(self):
        import struct
        import tempfile
        from tools.geno import check
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp) / "files"
            folder.mkdir()
            # Synthetic GXTX v1: a 4x4 RGB5A3 tile made entirely here.
            header = b"GXTX" + struct.pack(">10I", 1, 5, 4, 4, 0, 0, 32, 0, 64, 0)
            blob = header.ljust(64, b"\0") + b"\xfc\0" * 16
            (folder / "r.gxtex").write_bytes(blob)
            errors = []
            check.presentation_checks({"presentation": {"results": "r.gxtex"}}, "fighters[0]", temp, errors)
            self.assertEqual(errors, [])
            (folder / "r.gxtex").write_bytes(b"malformed")
            check.presentation_checks({"presentation": {"results": "r.gxtex"}}, "fighters[0]", temp, errors)
            self.assertTrue(any("not a v1 .gxtex" in str(e) for e in errors), errors)

if __name__ == "__main__":
    unittest.main()
