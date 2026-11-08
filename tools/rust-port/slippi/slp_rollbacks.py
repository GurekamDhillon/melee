#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Rollback count of a Slippi replay: Slippi Dolphin writes a frame's events again each time it resimulates it, so a
frame number that repeats in the Frame Start (0x3A) / Pre-Frame (0x37) stream is a rolled-back frame.
    python -I slp_rollbacks.py FILE.slp   -> frames, unique frames, rolled frames, rollbacks (runs of a frame going back), max depth
Only the raw event stream is read (no metadata)."""
import struct
import sys


def events(raw):
    i = 0
    if raw[0] != 0x35:
        raise SystemExit("no Event Payloads")
    n = raw[1]
    sizes = {}
    for k in range((n - 1) // 3):
        c, s = raw[2 + 3 * k], struct.unpack(">H", raw[3 + 3 * k:5 + 3 * k])[0]
        sizes[c] = s
    i = 1 + n
    while i < len(raw):
        c = raw[i]
        if c not in sizes:
            break
        yield c, raw[i + 1:i + 1 + sizes[c]]
        i += 1 + sizes[c]


def main():
    d = open(sys.argv[1], "rb").read()
    k = d.find(b"raw[$U#l")
    ln = struct.unpack(">I", d[k + 8:k + 12])[0]
    raw = d[k + 12:k + 12 + ln]
    seen, frames, rolls, depth, last = set(), 0, 0, 0, None
    rolled = set()
    port = None
    for c, p in events(raw):
        if c != 0x37 or p[5] != 0:  # count one pre-frame per frame: the leader of the first port seen
            continue
        if port is None:
            port = p[4]
        if p[4] != port:
            continue
        f = struct.unpack(">i", p[0:4])[0]
        frames += 1
        if last is not None and f <= last:
            rolls += 1
            depth = max(depth, last - f + 1)
        if f in seen:
            rolled.add(f)
        seen.add(f)
        last = f
    print(f"frames {frames} unique {len(seen)} rolled_frames {len(rolled)} rollbacks {rolls} max_depth {depth}")


if __name__ == "__main__":
    main()
