#!/usr/bin/env python3
"""Step 23/24: an R4 ext_fns! default (logic where the link layer already installs the canonical tu function) becomes a
`todo!` default. Only entries whose hook gen_table.rs sets (`<crate>::tu::<stem>::ext::hook::<name> = Some(`) are rewritten.
  python tools/rust-port/dedupe/r4_todo.py [--dry] [--only SUBSTR]"""
import re, subprocess, sys
from pathlib import Path
REPO = Path(__file__).resolve().parents[3]
dry = '--dry' in sys.argv
only = sys.argv[sys.argv.index('--only') + 1] if '--only' in sys.argv else ''
link = (REPO / 'rust/melee_link/src/gen_table.rs').read_text(errors='replace')
installed = set(re.findall(r'(\w+)::tu::(\w+)::ext::hook::(\w+) = Some\(', link))
out = subprocess.check_output([sys.executable, str(REPO / 'tools/rust-port/lint_logic.py'), '--list', 'R4'], text=True)
want = {}
for l in out.splitlines():
    p = l.split('\t')
    if len(p) == 3 and p[0] == 'R4' and only in p[1]:
        want.setdefault(p[1], set()).add(p[2])
tot = skip = 0
def arity(sig):
    s = sig.replace('->', '=>')
    d = 0; n = 0; seen = False
    for ch in s:
        if ch in '(<[': d += 1
        elif ch in ')>]': d -= 1
        elif ch == ',' and d == 0: n += 1
        elif not ch.isspace(): seen = True
    return (n + 1) if seen else 0
for rel, names in want.items():
    f = REPO / rel
    t = f.read_text(errors='replace')
    crate = rel.split('/')[1]
    stem = Path(rel).stem
    changed = False
    for blk in list(re.finditer(r'ext_fns!\s*\{', t))[::-1]:
        d, k = 0, blk.end() - 1
        while k < len(t):
            if t[k] == '{': d += 1
            elif t[k] == '}':
                d -= 1
                if d == 0: break
            k += 1
        text = t[blk.end():k]
        pos = 0
        edits = []
        for m in re.finditer(r'(?:;[ \t]*\n|\Z)', text):
            e = text[pos:m.start()]
            base = pos
            pos = m.end()
            m2 = re.match(r'(\s*(?:fn\s+)?([A-Za-z_]\w*)\s*\((.*?)\)\s*(?:->\s*[^=]+?)?=\s*)(.*)$', e, re.S)
            if not m2: continue
            name = m2.group(2)
            if name not in names: continue
            if (crate, stem, name) not in installed:
                skip += 1; continue
            rhs = m2.group(4).strip()
            if rhs.startswith('|'):
                new = '|' + ', '.join(['_'] * arity(m2.group(3))) + '| todo!("%s: installed by melee_link")' % name
            elif rhs.startswith('{') or rhs.startswith('unsafe'):
                new = '{ todo!("%s: installed by melee_link") }' % name
            else:
                skip += 1; continue
            s0 = base + m2.start(4)
            e0 = base + len(e)
            edits.append((s0, e0, new))
        for s0, e0, new in sorted(edits, reverse=True):
            text = text[:s0] + new + text[e0:]
            tot += 1; changed = True
        t = t[:blk.end()] + text + t[k:]
    if changed and not dry:
        f.write_text(t)
print('rewritten', tot, 'skipped (not installed / other form)', skip)
