#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Offline compare of one engine-vs-Dolphin run (np_dolphin_pair.sh OUTDIR): every frame both sides finalized.

    python -I np_compare.py OUTDIR [NAME]

Dolphin's own checksums are the TX lines of OUTDIR/checksums_NAME.txt ("cf F cs C": the checksum of frame F it sent);
the engine's are OUTDIR/engine.csv. Prints equal/total, the soft/hard count by the ASM rule, and the first mismatches
(with the engine's trace line from OUTDIR/trace.txt when the run had --trace).
"""
import os
import sys


def s16(v):
    v &= 0xFFFF
    return v - 0x10000 if v & 0x8000 else v


def main():
    o = sys.argv[1]
    name = sys.argv[2] if len(sys.argv) > 2 else "P3A"
    dol = {}
    for ln in open(os.path.join(o, f"checksums_{name}.txt")):
        p = ln.split()
        if len(p) >= 8 and p[0] == "TX" and p[4] == "cf":
            f, c = int(p[5]), int(p[7], 16)
            if f > 1:
                dol[f] = c
    eng = {}
    for ln in open(os.path.join(o, "engine.csv")).read().split("\n")[1:]:
        if ln:
            f, c = ln.split(",")
            eng[int(f)] = int(c, 16)
    tr = {}
    tp = os.path.join(o, "trace.txt")
    if os.path.exists(tp):
        for ln in open(tp):
            f = ln.split(" ", 1)[0]
            tr[int(f)] = ln.rstrip()
    common = sorted(set(dol) & set(eng))
    bad = [f for f in common if dol[f] != eng[f]]
    hard = [f for f in bad if abs(s16(dol[f]) - s16(eng[f])) > 1]
    print(f"{o}: {len(common) - len(bad)}/{len(common)} equal (frames {common[0] if common else '-'}..{common[-1] if common else '-'}), "
          f"{len(bad) - len(hard)} soft, {len(hard)} hard")
    for f in bad[:6]:
        print(f"  frame {f}: dolphin {dol[f]:08X} engine {eng[f]:08X}")
        for g in (f - 1, f):
            if g in tr:
                print(f"    {tr[g]}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
