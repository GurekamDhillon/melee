#!/usr/bin/env python3
"""Lists `pub static mut NAME` data declared inside the `// BEGIN GENERATED EXT` blocks / `pub mod ext` of the tu files
(a private zeroed copy) that are also defined as the real static in another file. Each is a split-brain global: the
engine sets one copy and the reader sees the other. Fix: make the lower crate's copy canonical and `pub use` it.
Usage: audit_ext_statics.py [--all]  (prints crate order hints)."""
import re, sys, pathlib
root = pathlib.Path(__file__).resolve().parents[3] / 'rust'
order = ['melee_core','melee_types','melee_runtime','melee_world','melee_fighter','melee_chara','melee_items','melee_modes']
decl = re.compile(r'^[ \t]*pub static mut (\w+)\s*:\s*([^=;]+?)\s*(?:=|;)', re.M)
ext_defs = {}   # name -> [(file, ty)]
real_defs = {}
for f in root.glob('*/src/**/*.rs'):
    if '/raw/' in f.as_posix() or '/tests/' in f.as_posix(): continue
    t = f.read_text(encoding='latin-1')
    crate = f.relative_to(root).parts[0]
    # ext blocks
    spans = []
    for m in re.finditer(r'pub mod ext\s*\{', t):
        i = m.end(); d = 1
        while d and i < len(t):
            d += (t[i] == '{') - (t[i] == '}'); i += 1
        spans.append((m.start(), i))
    for m in decl.finditer(t):
        name, ty, init = m.group(1), m.group(2).strip(), None
        if 'Option<' in ty and 'fn' in ty: continue
        inext = any(a <= m.start() < b for a, b in spans)
        (ext_defs if inext else real_defs).setdefault(name, []).append((f.relative_to(root).as_posix(), ty, crate))
bad = 0
for name, lst in sorted(ext_defs.items()):
    reals = real_defs.get(name, [])
    if reals:
        bad += 1
        print(f'{name}: ext copies in {[x[0] for x in lst]}; real in {[x[0] for x in reals]}')
print(f'{bad} split statics', file=sys.stderr)
