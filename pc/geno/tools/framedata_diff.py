#!/usr/bin/env python3
"""Compare two Geno Lab frame-data exports and print the changed states and fields.

    python pc/geno/tools/framedata_diff.py <export A> <export B>

An export is a folder the Lab wrote (TOOLS > Frame data export, `lab export`, or a headless run
with MELEE_LAB_BATCH=<version>): scripts-data/geno-lab_lab/framedata/<fighter>/<version>/ holding
framedata.json (plus moves.csv / hitboxes.csv). Either argument may also be the framedata.json
itself. States are matched by name; hitboxes by their order within the state.

Exit status: 0 = no differences, 1 = differences, 2 = bad arguments. The in-game equivalent is
TOOLS > "Diff the last two" / `lab fdiff [fighter verA verB]` (docs/geno.md 14.11).
"""
import json
import os
import sys

MOVE_FIELDS = ["group", "air", "startup", "active", "total", "iasa", "total_script", "landing_lag",
               "lcancel_lag", "autocancel", "landed", "ended", "chain"]
HB_FIELDS = ["id", "damage", "angle", "kbg", "bkb", "wbk", "radius", "bone", "element", "shield_damage",
             "frames"]


def load(path):
    if os.path.isdir(path):
        path = os.path.join(path, "framedata.json")
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def fmt(v):
    if v is None:
        return "-"
    if isinstance(v, float):
        return ("%.4f" % v).rstrip("0").rstrip(".")
    return str(v)


def same(a, b):
    if isinstance(a, float) or isinstance(b, float):
        try:
            return abs(float(a) - float(b)) < 1e-4
        except (TypeError, ValueError):
            return False
    return a == b


def diff(a, b):
    out = []
    ma = {m["name"]: m for m in a["moves"]}
    mb = {m["name"]: m for m in b["moves"]}
    for name in sorted(set(ma) | set(mb)):
        x, y = ma.get(name), mb.get(name)
        if x is None:
            out.append("+ %s (new)" % name)
            continue
        if y is None:
            out.append("- %s (gone)" % name)
            continue
        d = ["%s %s -> %s" % (k, fmt(x.get(k)), fmt(y.get(k))) for k in MOVE_FIELDS
             if not same(x.get(k), y.get(k))]
        hx, hy = x.get("hitboxes", []), y.get("hitboxes", [])
        for i in range(max(len(hx), len(hy))):
            if i >= len(hx) or i >= len(hy):
                d.append("hitbox %d %s" % (i + 1, "added" if i >= len(hx) else "removed"))
                continue
            for k in HB_FIELDS:
                if not same(hx[i].get(k), hy[i].get(k)):
                    d.append("hb%d.%s %s -> %s" % (i + 1, k, fmt(hx[i].get(k)), fmt(hy[i].get(k))))
        if d:
            out.append("~ %s: %s" % (name, "; ".join(d)))
    return out


def main(argv):
    if len(argv) != 3:
        print(__doc__)
        return 2
    a, b = load(argv[1]), load(argv[2])
    lines = diff(a, b)
    print("framedata diff %s: %s -> %s: %d state(s) changed" % (a.get("fighter"), a.get("version"),
                                                                b.get("version"), len(lines)))
    for line in lines:
        print(line)
    return 1 if lines else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
