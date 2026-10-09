"""Behavior tests for the coordinator's confirmed-frame diagnostic comparator."""
import contextlib
import csv
import io
import tempfile
import unittest
from pathlib import Path

from compare_rb_hash_detail import compare, load


class HashDetailTest(unittest.TestCase):
    def data(self, frames, changed=None):
        return {
            (1, frame): {
                ("hash", "match", 0, -1, "RB_GameHashForFrame"): "ABCD0001" if frame != changed else "ABCD0003",
                ("word", "fighter", 0, 1, "fp->motion_id"): "0000000E" if frame != changed else "0000000F",
            }
            for frame in frames
        }

    def test_negative_frame_reports_field(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            self.assertFalse(compare(self.data(range(-123, 2)), self.data(range(-123, 2), -80), require_prematch=True))
        self.assertIn("frame -80", out.getvalue())
        self.assertIn("fighter[0] word +1 fp->motion_id", out.getvalue())

    def test_missing_overlap_cannot_pass(self):
        with self.assertRaisesRegex(ValueError, "missing frame"):
            compare(self.data([-123, -122, -121]), self.data([-123, -121]))

    def test_no_prematch_cannot_certify_entry(self):
        with self.assertRaisesRegex(ValueError, "no shared pre-match"):
            compare(self.data([0, 1]), self.data([0, 1]), require_prematch=True)

    def test_equal_with_shutdown_skew(self):
        self.assertTrue(compare(self.data([-123, -122]), self.data([-123, -122, -121])))

    def test_raw_seed_is_diagnostic_only(self):
        a, b = self.data([-80]), self.data([-80])
        field = ("seed", "match", 0, -1, "arrived_rng")
        a[1, -80][field], b[1, -80][field] = "AAAAAAAA", "BBBBBBBB"
        self.assertTrue(compare(a, b))

    def test_truncation_and_missing_history_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "detail.csv"
            for kind in ("overflow", "missing"):
                path.write_text("kind,epoch,frame,region,slot,offset,symbol,value\n"
                                f"{kind},1,-80,match,0,-1,history,1\n")
                with self.assertRaisesRegex(ValueError, kind):
                    load(path)

    def test_csv_symbol_with_comma_and_duplicate_detection(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "detail.csv"
            with path.open("w", newline="") as stream:
                w = csv.writer(stream)
                w.writerow(["kind", "epoch", "frame", "region", "slot", "offset", "symbol", "value"])
                w.writerow(["hash", 1, -80, "match", 0, -1, "RB_GameHashForFrame", "ABCD0001"])
                w.writerow(["word", 1, -80, "fighter", 0, 1, "func(a, b)", "00000001"])
            self.assertIn(("word", "fighter", 0, 1, "func(a, b)"), load(path)[1, -80])
            with path.open("a") as stream:
                stream.write("hash,1,-80,match,0,-1,RB_GameHashForFrame,ABCD0001\n")
            with self.assertRaisesRegex(ValueError, "duplicate"):
                load(path)


if __name__ == "__main__":
    unittest.main()
