#!/usr/bin/env python3
"""python rust/melee_items/gen/kinds/splice.py <stem>...: replaces the //@EXT@ marker (or an existing generated EXT block)
of src/tu/<stem>.rs by the output of gen_tu.py --ext (gen/kinds/tu/<stem>_ext.rs.txt)."""
import re, subprocess, sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
for s in sys.argv[1:]:
    subprocess.run([sys.executable, str(HERE / "gen_tu.py"), s, "--ext"], cwd=ROOT, check=True, capture_output=True)
    ext = (HERE / "tu" / f"{s}_ext.rs.txt").read_text(encoding="utf-8").rstrip("\n")
    p = ROOT / "rust/melee_items/src/tu" / f"{s}.rs"
    t = p.read_text(encoding="utf-8")
    if "//@EXT@" in t:
        t = t.replace("//@EXT@", ext)
    else:
        t = re.sub(r"(?://\s*)?// BEGIN GENERATED EXT.*?// END GENERATED EXT", lambda m: ext, t, flags=re.S)
    p.write_text(t, encoding="utf-8", newline="\n")
    print("spliced", s)
