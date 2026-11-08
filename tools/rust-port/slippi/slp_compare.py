#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Compare two Slippi replays of the same game frame by frame (e.g. Slippi Dolphin's .slp against the engine's
`slippi-replay --slp`): per frame and port, the post-frame action state, position, facing, percent, shield and
stocks, taking the last copy of a frame (a rolled-back frame appears more than once in Dolphin's).
    python -I slp_compare.py A.slp B.slp [--shift N]   (B's frame f is compared with A's frame f + N)"""
import struct
import sys

sys.path.insert(0, __import__("os").path.dirname(__file__))
from slp_rollbacks import events  # noqa: E402


def post(path):
    d = open(path, "rb").read()
    k = d.find(b"raw[$U#l")
    ln = struct.unpack(">I", d[k + 8:k + 12])[0]
    out = {}
    for c, p in events(d[k + 12:k + 12 + ln]):
        if c != 0x38 or p[5] != 0:
            continue
        f = struct.unpack(">i", p[0:4])[0]
        port = p[4]
        # action (0x8..), x, y, facing, percent, shield (0x1A), stocks (0x21): offsets are event-relative minus 1
        act = struct.unpack(">H", p[7:9])[0]
        x, y, fc, pct, sh = struct.unpack(">fffff", p[9:29])
        out[(f, port)] = (act, round(x, 4), round(y, 4), fc, round(pct, 3), round(sh, 3), p[0x20])
    return out


def main():
    a, b = post(sys.argv[1]), post(sys.argv[2])
    shift = int(sys.argv[sys.argv.index("--shift") + 1]) if "--shift" in sys.argv else 0
    common = bad = 0
    first = None
    for (f, port), v in sorted(b.items()):
        w = a.get((f + shift, port))
        if w is None:
            continue
        common += 1
        if w != v:
            bad += 1
            first = first or (f, port, w, v)
    print(f"common port-frames {common}, differing {bad}" + (f", first {first}" if first else ""))
    sys.exit(1 if bad or not common else 0)


if __name__ == "__main__":
    main()
