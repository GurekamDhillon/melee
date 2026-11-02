# SPDX-License-Identifier: MIT
# Copyright (c) 2026 GurekamDhillon
"""Reads the GBLIVE lines pad_behaviours.lua wrote to melee-pc.log and checks the live run of the
Geno v5.1 behaviours against the acceptance table (docs/specs/geno-v5.1-behaviours.md).

Each row prints  LIVE <row> PASS|FAIL <measured>.  Exit status 1 when a row fails.
The state numbers are the order of the states in tests/mods/behaviour-test/geno.json plus 1024.
"""
import re
import sys

TOR, DR, CP, GS, G, GA, GE = (1024 + i for i in range(7))
LINE = re.compile(
    r"GBLIVE (\d+) ph (\d+) act (-?\d+) x (-?[\d.]+) y (-?[\d.]+) vx (-?[\d.]+) vy (-?[\d.]+) f (-?\d+)")


def main(path):
    rows = []
    with open(path, errors="replace") as fh:
        for line in fh:
            m = LINE.search(line)
            if m:
                n, ph, act = int(m[1]), int(m[2]), int(m[3])
                rows.append((n, ph, act, float(m[4]), float(m[5]), float(m[6]), float(m[7]), int(m[8])))
    if not rows:
        print("LIVE no GBLIVE lines in", path)
        return 1
    runs = []  # (act, first index, length)
    for i, r in enumerate(rows):
        if runs and runs[-1][0] == r[2]:
            runs[-1][2] += 1
        else:
            runs.append([r[2], i, 1])
    fails = 0

    def report(name, ok, meas):
        nonlocal fails
        print("LIVE %s %s %s" % (name, "PASS" if ok else "FAIL", meas))
        if not ok:
            fails += 1

    def run_of(act, nth=0):
        hits = [r for r in runs if r[0] == act]
        return hits[nth] if len(hits) > nth else None

    # tornado from the ground, no mashing: start-up 12 + 47 loop + 29 end = 88 frames (spec 3)
    r = run_of(TOR)
    report("tornado-frames", r is not None and 87 <= r[2] <= 89, r[2] if r else None)  # 12 + 47 + 29
    if r:
        seg = rows[r[1]:r[1] + r[2]]
        report("tornado-ground-no-rise", all(abs(s[6]) < 0.01 for s in seg), max(abs(s[6]) for s in seg))
    # drill: 96 frames (spec 4); heading dive -20 degrees has no vertical part on the ground
    r = run_of(DR)
    report("drill-frames", r is not None and 95 <= r[2] <= 97, r[2] if r else None)
    # cape: 40 frames (spec 5)
    r = run_of(CP)
    report("cape-frames", r is not None and 39 <= r[2] <= 41, r[2] if r else None)
    # glide: GlideStart 12 frames, then Glide (spec 2.3), then GlideAttack on the attack press
    r = run_of(GS)
    report("glide-start-12", r is not None and 11 <= r[2] <= 13, r[2] if r else None)
    r = run_of(G)
    report("glide-fly-entered", r is not None and r[2] >= 1, r[2] if r else None)
    r = run_of(GA)
    report("glide-attack-28", r is not None and 27 <= r[2] <= 29, r[2] if r else None)  # ATTACK_FRAMES 28
    r = run_of(GE)
    report("glide-end-entered", r is not None, r[2] if r else None)
    print("LIVE runs", " ".join("%d:%d" % (a, l) for a, _, l in runs if a >= 1024))
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
