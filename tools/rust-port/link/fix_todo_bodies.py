#!/usr/bin/env python3
"""Rewrite hand-written `pub fn NAME(a: T, ..) [-> R] { todo!("NAME (...)") }` callee stubs of the tu/ files to call the
canonical `NAME` of the SAME crate when exactly one exists with the same arity (arguments and result go through `as _`).
The ext modules of the older TUs spell their cross-TU calls as such stubs; the link layer only reaches hook statics.
Usage: fix_todo_bodies.py [--apply]"""
import os, re, sys
sys.path.insert(0, os.path.dirname(__file__))
import gen_link as g

apply = '--apply' in sys.argv
defs = g.index_defs()
FN = re.compile(r'(?m)^([ \t]*)pub fn (\w+)\(([^)]*)\)\s*(->\s*[^{]+?)?\s*\{\s*(?:unsafe\s*\{\s*)?todo!\("(\w+)[ "][^"]*"\)\s*(?:\})?\s*\}')
n = 0
for crate, f, rel in g.tu_files():
    if not rel.startswith('tu/'):
        continue
    raw = open(f, encoding='latin-1', newline='').read()
    out = raw
    for m in list(FN.finditer(raw))[::-1]:
        name = m.group(2)
        if m.group(5) != name:
            continue
        params = [p.strip() for p in g.split_top(m.group(3), ',') if p.strip()]
        names = [re.sub(r'^muts+', '', p.split(':')[0].strip()) for p in params]
        cands = [c for c in defs.get(name, []) if c[0] == crate and c[2] == len(params) and c[4] != rel]
        if len(cands) != 1:
            continue
        c = cands[0]
        call = f'crate::{c[1]}::{name}({", ".join(x + " as _" for x in names)})'.replace('crate::tu::tu::', 'crate::tu::')
        if m.group(4):
            body = f'unsafe {{ {call} as _ }}'
        else:
            body = f'unsafe {{ let _ = {call}; }}'
        new = f'{m.group(1)}pub fn {name}({m.group(3)}) {m.group(4) or ""} {{ {body} }}'.replace('  {', ' {')
        out = out[:m.start()] + new + out[m.end():]
        n += 1
        print(f'{rel}: {name} -> {c[1]}')
    if apply and out != raw:
        open(f, 'w', encoding='latin-1', newline='').write(out)
print(n, 'stubs' + (' rewritten' if apply else ' (dry run)'))
