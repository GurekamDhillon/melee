#!/usr/bin/env python3
"""python gen/mtu/autospec.py <stem> <src.c> [count]  -> writes lbspecs_<stem>.py with pools = every record type
that a pub fn of src/tu/<stem>.rs takes by pointer (a starting point; edit by hand)."""
import sys
from pathlib import Path
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gen_tu
from gen_tu import Tu, canon, T, ROOT
import mk_lb

stem, src = sys.argv[1], sys.argv[2]
cnt = int(sys.argv[3]) if len(sys.argv) > 3 else 3
tu = Tu(src, (), (ROOT / src).parent)
pub = mk_lb.pub_fns(stem)
types = []
for name, cur in tu.defs.items():
    if name not in pub:
        continue
    for p in cur.get_arguments():
        c = canon(p.type)
        if c.kind == T.POINTER:
            pt = canon(c.get_pointee())
            if pt.kind == T.RECORD:
                n = pt.get_declaration().spelling
                if n.startswith("_"): n = n[1:]
                if n not in types and n not in ("Vec", "Vec3") and n in tu.types: types.append(n)
abbr = lambda i: f"P{i}"
pools = [(t, abbr(i), cnt) for i, t in enumerate(types)]
out = f'"""{stem} spec (autospec, then hand edited)."""\nfrom lbspecs import add\n\nPOOLS = {pools!r}\nadd("{stem}", "{src}", dict(pub_only=True, pools=POOLS),\n    pub_only=True, types=[p[0] for p in POOLS], pools=[p[0] for p in POOLS])\n'
(HERE / f"lbspecs_{stem}.py").write_text(out)
print(out)
