#!/usr/bin/env python3
"""Fields that rust/melee_types/src/gen.rs decodes as a pointer while the decomp header now declares a scalar.

The GcDecoder reads an unrelocated non-zero word in a pointer slot as null, so a data-file int or float that gen.rs still
types `void *` (an older header) comes out 0 in the Rust engine while the console (and the C build) read the value.
Found: `_ftSamusAttributes.xD0` (Samus's grapple hang time, 180 frames on the console, 0 here: AirCatchHit ended at once,
the action 38-vs-358 slpcheck class; notes/fp-audit.md "Generated-type drift").

  python tools/rust-port/fp_audit/gen_type_drift.py      (exit 1 when a pointer-vs-scalar field is found)
"""
import re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[3]
gen = (ROOT / "rust/melee_types/src/gen.rs").read_text(errors="replace")
# record -> header path and field -> ctype, from the gen.rs doc line and GcField tables
recs = {}
cur = None
for l in gen.splitlines():
    m = re.match(r"/// C `(\w+)` \((src/[^:]+\.h):\d+\)", l)
    if m:
        cur = (m.group(1), m.group(2)); recs.setdefault(cur, {}); continue
    m = re.search(r'GcField::new\("(\w+)", 0x[0-9A-Fa-f]+, \d+, "([^"]*)"\)', l)
    if m and cur:
        recs[cur][m.group(1)] = m.group(2)
SCALAR = re.compile(r"^(?:const\s+)?(int|s32|u32|s16|u16|s8|u8|f32|float|f64|double|bool|unsigned int|signed int)$")
# checked by hand, not drift: a nested struct's field of the same name (ftData.x10/x14), a GcField table of the next record
ALLOW = {("ftData", "x10"), ("ftData", "x14"), ("CrowdConfig", "x24"),
         # Jigglypuff xE8/xEC: never read (the C has no use); retyping them changes the native record size
         ("_ftPurinAttributes", "xE8"), ("_ftPurinAttributes", "xEC")}
hdrs = {}
bad = 0
for (name, path), fields in recs.items():
    p = ROOT / path
    if not p.exists():
        continue
    txt = hdrs.setdefault(path, p.read_text(errors="replace"))
    m = re.search(r"(?:struct\s+%s\s*\{|typedef\s+struct\s+%s\s*\{)(.*?)\}" % (name, name), txt, re.S)
    if not m:
        continue
    body = m.group(1)
    for f, ct in fields.items():
        if "*" not in ct or (name, f) in ALLOW:
            continue
        fm = re.search(r"(?m)^\s*(?:/\*.*?\*/\s*)?([A-Za-z_][\w ]*?)\s+%s\s*;" % re.escape(f), body)
        if fm and SCALAR.match(fm.group(1).strip()):
            print(f"{path}: {name}.{f}: gen.rs '{ct}', header '{fm.group(1).strip()}'")
            bad += 1
print(f"{bad} pointer-vs-scalar field(s)")
sys.exit(1 if bad else 0)
