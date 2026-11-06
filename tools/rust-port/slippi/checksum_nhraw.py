#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Slippi online's desync checksum (docs/rust-port/notes/slippi-online-patches.md 2.2) computed from the Dolphin
oracle's memory records (.nhraw, tools/rust-port/dolphin), one line per logged frame: "<slippi frame> <checksum hex>".
The same lines as `melee slpcheck --slippi-checksum-out`, so the two files diff directly.

    python -I tools/rust-port/slippi/checksum_nhraw.py dolphin.nhraw out.txt

Limits: the record lists a slot's fighters through `transformed[]` (raw entity order for every character but a
transformed Zelda/Sheik), and an empty slot's stocks byte is not recorded (taken as 0).
"""
import struct
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "dolphin"))
import raw2csv  # noqa: E402


def f32(x):
    return struct.unpack(">f", struct.pack(">f", x))[0]


def checksum(fts):
    cs, fsum = 0, 0.0
    by_slot = {}
    for slot, _pk, stocks, _fp, b in fts:
        by_slot.setdefault(slot // 2, []).append((slot % 2, stocks, b))
    for p in range(4):
        ents = sorted(by_slot.get(p, []))
        for _j, _st, b in ents:
            for o in (0x10, 0xB0, 0xB4, 0x1830, 0x8):
                cs ^= struct.unpack_from(">I", b, o)[0]
            for o in (0xB0, 0xB4, 0x1830):
                fsum = f32(fsum + struct.unpack_from(">f", b, o)[0])
        cs ^= (ents[0][1] & 0xFF) if ents else 0
    hi = (cs >> 16) ^ (cs & 0xFFFF)
    if fsum != fsum:
        v = -2**31
    else:
        v = max(-2**31, min(2**31 - 1, int(fsum)))
    return (hi << 16) | (v & 0xFFFF)


def main():
    data = Path(sys.argv[1]).read_bytes()
    with open(sys.argv[2], "w") as out:
        for frame, _u, _seed, fts, _its in raw2csv.records(data):
            out.write(f"{frame} {checksum(fts):08X}\n")


if __name__ == "__main__":
    main()
