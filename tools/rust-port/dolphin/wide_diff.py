#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Dolphin oracle: compare two by-name state dumps (`melee dolphin-wide`, `melee slpcheck --wide-out`).

    python tools/rust-port/dolphin/wide_diff.py RUST.txt DOLPHIN.txt [--max N] [--all-frames] [--persistent] [--zero-eq]

Lines are `<slippi frame> <instance> <field> <value...>`. Prints, per frame in order, the fields whose values differ.
Many fields never agree between a native build and the console and say nothing about the divergence (heap addresses
held in int fields, the CPU block of a slot the Rust replay player runs as a human, render/HUD-side counters), so by
default a frame lists only the NEW differences: fields that agreed (or were absent) on the previous dumped frame.
Dump a window that starts a few frames before the first NHV1 divergence. --persistent lists every difference.
--zero-eq treats an f32 +0 and -0 as equal (to look past the known sign-of-zero class).
Stops after the first frame with a new difference unless --all-frames. Exit 0 no difference, 1 differences.
"""
import sys
from collections import defaultdict


def load(p):
    d = defaultdict(dict)
    with open(p, encoding="utf8", errors="replace") as f:
        for line in f:
            parts = line.rstrip("\n").split(" ", 3)
            if len(parts) < 4:
                continue
            d[int(parts[0])][(parts[1], parts[2])] = parts[3]
    return d


def main():
    a = sys.argv[1:]
    rust, dol = load(a[0]), load(a[1])
    mx = int(a[a.index("--max") + 1]) if "--max" in a else 40
    every = "--all-frames" in a
    persistent = "--persistent" in a
    zero_eq = "--zero-eq" in a
    ZEROS = {"0 (00000000)", "-0 (80000000)"}

    def same(x, y):
        return x == y or (zero_eq and x in ZEROS and y in ZEROS)
    bad = 0
    prev = None
    for fr in sorted(set(rust) & set(dol)):
        r, d = rust[fr], dol[fr]
        alld = [(k, r.get(k), d.get(k)) for k in sorted(set(r) | set(d)) if not same(r.get(k), d.get(k))]
        keys = {k for k, _, _ in alld}
        first = prev is None
        diffs = alld if (persistent or first) else [x for x in alld if x[0] not in prev]
        prev = keys
        if first and not persistent:
            print(f"frame {fr} (csv frame {fr + 123}): {len(alld)} field(s) differ already (baseline, not listed; --persistent lists them)")
            continue
        if not diffs:
            continue
        bad += 1
        print(f"frame {fr} (csv frame {fr + 123}): {len(diffs)} new difference(s) ({len(alld)} in all)")
        for (inst, field), rv, dv in diffs[:mx]:
            print(f"    {inst:6} {field:48} rust={rv}  dolphin={dv}")
        if len(diffs) > mx:
            print(f"    ... {len(diffs) - mx} more")
        if not every:
            break
    only = sorted(set(rust) ^ set(dol))
    if only:
        print(f"frames on one side only: {len(only)} (first {only[0]})")
    print("WIDE OK" if bad == 0 else f"WIDE MISMATCH: {bad} frame(s) shown")
    return 0 if bad == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
