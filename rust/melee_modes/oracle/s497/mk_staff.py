#!/usr/bin/env python3
"""Fill the @@table block of src/tu/gmstaffroll.rs with the 198 credit entries of gm_803DBFD8,
transcribed from src/melee/gm/gmstaffroll.c. Run from the worktree root."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
src = (ROOT / "src/melee/gm/gmstaffroll.c").read_text(encoding="utf-8")
m = re.search(r"static StaffEntryData gm_803DBFD8\[198\] = \{(.*?)\n\};", src, re.S)
rows = re.findall(r"\{\s*(\d+),\s*\{\s*0\s*\},\s*(\w+),\s*(\d+),\s*\{\s*0\s*\},\s*(\d+),\s*(\d+),\s*\{\s*(\d+),\s*(\d+)\s*\}\s*\}", m.group(1))
assert len(rows) == 198, len(rows)
out = ["/* 3DBFD8 */ pub static mut gm_803DBFD8: [StaffEntryData; 198] = ["]
for x0, chk, arg, alt, has, a, b in rows:
    c = "None" if chk == "NULL" else f"Some({chk})"
    out.append(f"    e({x0}, {c}, {arg}, {alt}, {has}, {a}, {b}),")
out.append("];")
p = ROOT / "rust/melee_modes/src/tu/gmstaffroll.rs"
s = p.read_text(encoding="utf-8")
s = re.sub(r"// @@table-begin.*?// @@table-end", lambda _: "// @@table-begin\n" + "\n".join(out) + "\n// @@table-end", s, flags=re.S)
p.write_text(s, encoding="utf-8", newline="\n")
print(len(rows), "entries")
