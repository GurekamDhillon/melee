#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Offline compare of a Dolphin-vs-Dolphin run (np_real_dolphin_pair.sh OUTDIR dd): each instance's own sent
checksums (the TX lines of checksums_R1.txt / checksums_R2.txt, "cf F cs C") on every frame both sent, and each
instance's received ones (RX) against the other's TX (the wire carried what was sent).

    python -I dd_compare.py OUTDIR [A B]
"""
import os
import sys


def load(path, kind):
    d = {}
    for ln in open(path):
        p = ln.split()
        if len(p) >= 8 and p[0] == kind and p[4] == "cf":
            d[int(p[5])] = int(p[7], 16)
    return d


def main():
    o = sys.argv[1]
    a, b = (sys.argv[2], sys.argv[3]) if len(sys.argv) > 3 else ("R1", "R2")
    pa, pb = (os.path.join(o, f"checksums_{n}.txt") for n in (a, b))
    ta, tb = load(pa, "TX"), load(pb, "TX")
    common = sorted(set(ta) & set(tb))
    diff = [f for f in common if ta[f] != tb[f]]
    print(f"TX {a} {len(ta)} frames (max {max(ta, default=0)}), TX {b} {len(tb)} (max {max(tb, default=0)})")
    print(f"common {len(common)}, equal {len(common) - len(diff)}, differ {len(diff)}" + (f", first {diff[:5]}" if diff else ""))
    for n, rx, tx in ((a, load(pa, "RX"), tb), (b, load(pb, "RX"), ta)):
        c = [f for f in rx if f in tx]
        print(f"RX {n}: {len(rx)} frames, {sum(rx[f] == tx[f] for f in c)} / {len(c)} equal to the peer's TX")


if __name__ == "__main__":
    main()
