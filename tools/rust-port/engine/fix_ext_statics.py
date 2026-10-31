#!/usr/bin/env python3
"""Replace the zeroed `pub static mut NAME: T = ...;` copies inside `pub mod ext { }` blocks with
`pub use <canonical path>::NAME;` so the whole port shares one static (see audit_ext_statics.py).
Usage: fix_ext_statics.py NAME CANONICAL_CRATE CANONICAL_MODULE   e.g.  stage_info melee_world tu::ground
The canonical definition must already exist (a `pub static mut NAME` outside any ext block of that module);
copies in crates ABOVE the canonical one are rewritten; a copy in a crate BELOW it is reported (move the
definition down first)."""
import re, sys, pathlib
name, ccrate, cmod = sys.argv[1:4]
order = ['melee_core','melee_types','melee_runtime','melee_world','melee_fighter','melee_chara','melee_items','melee_modes']
root = pathlib.Path(__file__).resolve().parents[3] / 'rust'
decl = re.compile(r'^([ \t]*)pub static mut ' + re.escape(name) + r'\s*:.*;[ \t]*$', re.M)
n = 0
for f in root.glob('*/src/**/*.rs'):
    p = f.as_posix()
    if '/raw/' in p or '/tests/' in p or '/c2r/' in p: continue
    t = f.read_text(encoding='latin-1')
    crate = f.relative_to(root).parts[0]
    if crate not in order: continue
    spans = []
    for m in re.finditer(r'pub mod ext\s*\{', t):
        i = m.end(); d = 1
        while d and i < len(t):
            d += (t[i] == '{') - (t[i] == '}'); i += 1
        spans.append((m.start(), i))
    out = t; shift = 0
    for m in list(decl.finditer(t)):
        if not any(a <= m.start() < b for a, b in spans): continue
        if order.index(crate) < order.index(ccrate):
            print('BELOW', f); continue
        path = ('crate::' if crate == ccrate else ccrate + '::') + cmod + '::' + name
        # file's own module is the canonical one? skip
        rep = f'{m.group(1)}pub use {path};'
        s, e = m.start() + shift, m.end() + shift
        out = out[:s] + rep + out[e:]; shift += len(rep) - (m.end() - m.start())
        n += 1
    if out != t:
        f.write_bytes(out.encode('latin-1'))
print(name, 'rewrote', n)
