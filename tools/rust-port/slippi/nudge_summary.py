#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Summarise nudge_sweep.py's rows.csv (docs/rust-port/notes/nudge.md): python -I nudge_summary.py rows.csv"""
import csv
import statistics
import sys
from collections import Counter, defaultdict

rows = list(csv.DictReader(open(sys.argv[1])))
ctl = [r for r in rows if r["when"] == "control"]
rows = [r for r in rows if r["when"] != "control" and r["class"] != "error"]
print("control (0u) runs:", len(ctl), Counter(r["class"] for r in ctl))
print("errors:", sum(1 for r in csv.DictReader(open(sys.argv[1])) if r["class"] == "error"))
CL = ["invisible", "healed", "soft", "hard"]


def table(title, key, sel=lambda r: True):
    g = defaultdict(list)
    for r in rows:
        if sel(r):
            g[key(r)].append(r)
    print(f"\n{title}")
    print(f"{'group':24} {'n':>4} " + " ".join(f"{c:>9}" for c in CL) + "  med_frames_to_hard  heal_med")
    for k in sorted(g):
        v = g[k]
        c = Counter(r["class"] for r in v)
        hd = [int(r["first_hard"]) - int(r["frame"]) for r in v if r["first_hard"]]
        hl = [int(r["healed_at"]) - int(r["first_soft"]) for r in v if r["healed_at"]]
        print(f"{str(k):24} {len(v):4} " + " ".join(f"{100 * c[x] / len(v):8.1f}%" for x in CL)
              + f"  {statistics.median(hd) if hd else '-':>10}  {statistics.median(hl) if hl else '-':>8}")


nz = lambda r: r["from_zero"] == "0"
table("all runs", lambda r: "all")
table("non-zero start value only", lambda r: "all", nz)
table("by field (non-zero start)", lambda r: r["field"], nz)
table("by field and amount (non-zero start)", lambda r: (r["field"], r["amount"]), nz)
table("by amount (non-zero start)", lambda r: r["amount"], nz)
table("by when (non-zero start)", lambda r: r["when"], nz)
print("\nfrom-zero runs:", sum(1 for r in rows if not nz(r)), Counter((r["field"], r["class"]) for r in rows if not nz(r)))
print("\n1-ULP runs that became Hard (replay index, when, field, frame, action, frames to hard, from_zero):")
for r in rows:
    if r["amount"] == "1u" and r["class"] == "hard":
        print(" ", r["replay"], r["when"], r["field"], r["frame"], r["action"], int(r["first_hard"]) - int(r["frame"]), r["from_zero"])
