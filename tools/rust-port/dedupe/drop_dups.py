#!/usr/bin/env python3
"""Step 23: delete the non-canonical bodies (lint R2, and R1 that are also R2) of C-named functions, then let the
compiler tell which were still used (restore those via skip.txt).
  python tools/rust-port/dedupe/drop_dups.py [--only SUBSTR] [--rule R2] [--skip FILE] [--dry]
Skip file lines: `<rel path>\t<name>` (never deleted). Output: number of function bodies removed."""
import re, sys, subprocess
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import lint_logic as L
args = sys.argv[1:]
def opt(n, d=''):
    return args[args.index(n) + 1] if n in args else d
only = opt('--only'); skipf = opt('--skip'); dry = '--dry' in args
skip = set()
if skipf and Path(skipf).exists():
    for l in Path(skipf).read_text().splitlines():
        p = l.split('\t')
        if len(p) == 2: skip.add((p[0], p[1].lower()))
pats = L.allow_patterns()
v, canon, bodies = L.violations(pats)
want = {}
for r, rel, n in v:
    if r == 'R2' and only in rel and (rel, n.lower()) not in skip:
        want.setdefault(rel, set()).add(n.lower())
total = 0
for rel, names in want.items():
    f = L.REPO / rel
    t = f.read_text(errors='replace')
    spans = []
    for m in L.FN.finditer(t):
        n = m.group(1)
        if n.lower() not in names: continue
        b = L.body_of(t, m.start())
        if b is None or L.trivial(t[b[0] + 1:b[1]], n): continue
        if '--methods' not in args and re.search(r"[(,]\s*&?(mut\s+)?self\b", t[m.start():b[0]]): continue   # a trait / impl method
        s = t.rfind('\n', 0, m.start()) + 1
        # extend over preceding doc comments / attributes
        while True:
            p = t.rfind('\n', 0, s - 1) + 1
            line = t[p:s - 1].strip()
            if s > 0 and (line.startswith('///') or line.startswith('#[') or line.startswith('//')) and p < s:
                s = p
            else:
                break
        e = b[1] + 1
        if e < len(t) and t[e] == '\n': e += 1
        spans.append((s, e))
    # drop nested spans
    spans.sort()
    out = []
    for s, e in spans:
        if out and s < out[-1][1]: continue
        out.append((s, e))
    for s, e in reversed(out):
        t = t[:s] + t[e:]
        total += 1
    if out and not dry:
        f.write_text(t)
print('removed', total, 'bodies in', len(want), 'files')
