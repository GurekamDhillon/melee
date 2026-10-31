#!/usr/bin/env python3
"""Consolidation step 7: make every static of an `ext` block the canonical one.

The generated `ext` blocks of rust/*/src/tu/** declare zeroed copies of the data that another TU owns ("the tests
set them"). On the engine that is a second, never-initialised instance. This rewrites each `pub static mut NAME: T`
inside a `mod ext { }` block to `pub use <crate>::<module>::NAME;` when exactly one canonical top-level
`pub static mut NAME: <same type>` exists in a crate at or below the user's crate (dependency order below).
Run from the repo root; `--check` only reports. Idempotent.
"""
import re, sys, pathlib

sys.path.insert(0, str(pathlib.Path(__file__).parent))
from gen_link import blank, match_close

NL = chr(10)
ORDER = ['melee_core', 'melee_types', 'melee_runtime', 'melee_world', 'melee_items', 'melee_fighter', 'melee_chara', 'melee_modes']
root = pathlib.Path('rust')
check = '--check' in sys.argv
STAT = re.compile(r'^(\s*)(?:/\*.*?\*/\s*)?pub static mut (\w+): (.+?) = (.*);\s*$')
DECL = re.compile(r'^(\s*)(?:/\*.*?\*/\s*)?pub static mut (\w+): (.+?) =( |$)')
EXT = re.compile(r'\bmod\s+ext\s*\{')


def modpath(f):
    parts = f.relative_to(root).parts
    crate = parts[0]
    rest = list(parts[2:])  # drop src
    rest[-1] = rest[-1][:-3]
    if rest[-1] == 'mod':
        rest.pop()
    return crate, '::'.join([crate] + rest)


def norm(t):
    t = re.sub(r'^\[(.+);\s*[^;\]]+\]$', lambda m: '[' + m.group(1) + ';N]', t.strip())
    return re.sub(r'\s+', '', t.replace('melee_types::', '').replace('crate::', ''))


files = [f for c in ORDER for f in (root / c / 'src').rglob('*.rs')]
canon = {}   # name -> list of (crate, path, type)
users = []   # (file, lineno)
for f in files:
    if '/raw/' in f.as_posix() or '/c2r/' in f.as_posix():
        continue
    text = f.read_text(encoding='utf-8')
    lines = text.split(NL)
    bl = blank(text)
    ranges = []
    for m in EXT.finditer(bl):
        e = match_close(bl, m.end() - 1)
        if e > 0:
            ranges.append((bl.count(NL, 0, m.start()), bl.count(NL, 0, e)))
    for i, l in enumerate(lines):
        m = STAT.match(l)
        d = DECL.match(l)
        if not d:
            continue
        if any(a <= i <= b for a, b in ranges):
            if m:
                users.append((f, i))
            continue
        m = d
        if not m.group(1) and 'test' not in f.name:
            crate, p = modpath(f)
            canon.setdefault(m.group(2), []).append((crate, p, norm(m.group(3))))

changed = {}
skipped = []
n = 0
for f, i in users:
    lines = changed.get(f) or f.read_text(encoding='utf-8').split(NL)
    m = STAT.match(lines[i])
    ind, name, ty = m.group(1), m.group(2), norm(m.group(3))
    me = modpath(f)[0]
    cands = [c for c in canon.get(name, []) if ORDER.index(c[0]) <= ORDER.index(me) and c[1] != modpath(f)[1]]
    same = [c for c in cands if c[2] == ty]
    if not same:
        skipped.append((f.as_posix(), name, 'no canonical' if not cands else 'type differs: ' + ' | '.join(c[2] for c in cands)))
        continue
    low = min(ORDER.index(c[0]) for c in same)
    same = [c for c in same if ORDER.index(c[0]) == low]
    if len({c[1] for c in same}) != 1:
        skipped.append((f.as_posix(), name, 'ambiguous: ' + ' '.join(c[1] for c in same)))
        continue
    path = same[0][1]
    if same[0][0] == me:
        path = 'crate' + path[len(me):]
    lines[i] = f'{ind}pub use {path}::{name};'
    changed[f] = lines
    n += 1
if not check:
    for f, lines in changed.items():
        f.write_text(NL.join(lines), encoding='utf-8')
print(f'{n} aliased, {len(skipped)} skipped, {len(users)} total')
for s in skipped:
    print('skip', *s)
