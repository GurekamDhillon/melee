#!/usr/bin/env python3
"""Prints the Rust data of src/melee/gm/gmevent.c's tables (gm_803DF918, the gm_804D43xx callback
records and the gm_803DF94C pointer array), read from the C source, for src/tu/gmevent.rs."""
import re
from pathlib import Path
ROOT = Path(__file__).resolve().parents[4]
src = (ROOT / "src/melee/gm/gmevent.c").read_bytes().decode("utf-8").replace(chr(13) + chr(10), chr(10))
out = []
m = re.search(r"static u8 gm_803DF918\[\] = \{(.*?)\};", src, re.S)
vals = [int(x) for x in re.findall(r"\d+", m.group(1))]
assert len(vals) == 51
out.append("/// `static u8 gm_803DF918[]` (803DF918): event index -> level table index.")
out.append("pub static gm_803DF918: [u8; 51] = [")
for i in range(0, 51, 17):
    out.append("    " + ", ".join(str(v) for v in vals[i:i + 17]) + ",")
out.append("];")
out.append("")
recs = re.findall(r"static gm_803DF94C_t (gm_804D\w+) = \{\s*(gm_\w+),\s*(\(void \(\*\)\(int\)\)\(void\*\) )?(NULL|gm_\w+),?\s*\};", src)
recs = [(a, c, b, d) for a, b, c, d in recs]
assert len(recs) == 51, len(recs)
for name, cast, f0, f1 in recs:
    if f1 == "NULL":
        x4 = "None"
    elif cast:
        x4 = f"unsafe {{ core::mem::transmute::<unsafe extern \"C\" fn(i32, i32), Option<unsafe extern \"C\" fn(i32)>>({f1}) }}"
    else:
        x4 = f"Some({f1})"
    out.append(f"static mut {name}: gm_803DF94C_t = gm_803DF94C_t {{ x0: Some({f0}), x4: {x4} }};")
out.append("")
m = re.search(r"gm_803DF94C_t\* gm_803DF94C\[\] = \{(.*?)\};", src, re.S)
names = re.findall(r"&(gm_804D\w+)", m.group(1))
assert len(names) == 51 and set(names) == {r[0] for r in recs}
out.append("/// `gm_803DF94C_t* gm_803DF94C[]` (803DF94C): per-event {frame proc, player-setup callback}.")
out.append("pub static mut gm_803DF94C: [*mut gm_803DF94C_t; 51] = unsafe {")
out.append("    [")
for i in range(0, 51, 4):
    out.append("        " + ", ".join(f"&raw mut {n}" for n in names[i:i + 4]) + ",")
out.append("    ]")
out.append("};")
print("\n".join(out))
