#!/usr/bin/env python3
"""Prints the Rust initializer of `gmClassic_803DDEC8` (src/melee/gm/gmclassic.c), read from the C
source, for src/tu/gmclassic.rs (python rust/melee_modes/gen/cref/classic_tables.py > fragment)."""
import re
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
src = (ROOT / "src/melee/gm/gmclassic.c").read_bytes().decode("utf-8").replace(chr(13) + chr(10), chr(10))
a = src.index("static gmClassic_803DDEC8Data gmClassic_803DDEC8 = {")
b = src.index(chr(10) + "};", a)
body = src[a + len("static gmClassic_803DDEC8Data gmClassic_803DDEC8 = {"):b]
groups, depth, cur = [], 0, ""
for ch in body:
    if ch == "{":
        depth += 1
        if depth == 1:
            cur = ""
            continue
    if ch == "}":
        depth -= 1
        if depth == 0:
            groups.append(cur)
            continue
    if depth >= 1:
        cur += ch
assert len(groups) == 9
row = re.compile(r"\{\s*(\d+),\s*\{\s*(\d+),\s*(\d+),\s*(\d+)\s*\},\s*(\d+)\s*\}")
sizes = {1: 2, 2: 39, 4: 30, 5: 11, 7: 10}
out = []
rs = re.findall(r"\{([^{}]*)\}", groups[0])
assert len(rs) == 12
out.append("        x00: [")
for r in rs:
    v = [int(x, 0) for x in r.split(",")]
    out.append(f"            gm_803DDEC8Struct {{ x0: {v[0]}, x1: {v[1]}, x2: {v[2]}, pad03: {v[3]}, x4: {v[4]}, x6: {v[5]}, x8: {v[6]}, pad09: [0; 3], xC: core::ptr::null_mut() }},")
out.append("        ],")
names = {1: "x0C0", 2: "x0CC", 4: "x1B8", 5: "x26C", 7: "x2B0"}
for gi in (1, 2, 4, 5, 7):
    ms = row.findall(groups[gi])
    n = sizes[gi]
    assert len(ms) <= n, (gi, len(ms), n)
    out.append(f"        {names[gi]}: [")
    for m in ms:
        out.append(f"            mt({m[0]}, [{m[1]}, {m[2]}, {m[3]}], {m[4]}),")
    for _ in range(n - len(ms)):
        out.append("            mt(0, [0, 0, 0], 0),")
    out.append("        ],")
    if gi == 2:
        out.append("        pad_1B6: [0; 2],")
    if gi == 5:
        out.append("        pad_2AE: [0; 2],")
out.append("        pad_2EC: [0; 4],")
print("\n".join(out))
