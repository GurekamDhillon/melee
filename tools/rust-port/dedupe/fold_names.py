#!/usr/bin/env python3
"""Move C-named fns out of non-tu modules into their canonical tu file (consolidation steps 21/22).

For each non-tu, non-allowlisted file of the crate, every top-level `fn` whose name is a C-defined
function (lint_logic's index) and has a real body moves to `src/tu/<c stem>.rs` (created when missing).
Private items of the source file become `pub(crate)` and the destination glob-imports the source module,
so moved bodies keep resolving their helpers. A name some tu file already defines is moved as `<name>_fp`.
The source keeps `pub use crate::tu::<stem>::<new> as <old>;` so every path resolves.

  fold_names.py --crate rust/melee_fighter [--files a.rs,b.rs] [--dry]
"""
import re, os, sys, argparse, importlib.util

here = os.path.dirname(os.path.abspath(__file__))
src = open(os.path.join(here, 'fold_reexports.py'), encoding='utf-8').read()
exec(src.split('def main():')[0])

spec = importlib.util.spec_from_file_location('lint_logic', os.path.join(here, '..', 'lint_logic.py'))
lint = importlib.util.module_from_spec(spec)
sys.argv_saved = sys.argv
sys.argv = ['x']
spec.loader.exec_module(lint)
sys.argv = sys.argv_saved

ap = argparse.ArgumentParser()
ap.add_argument('--crate', required=True)
ap.add_argument('--files')
ap.add_argument('--dry', action='store_true')
ap.add_argument('--skip', default='')
a = ap.parse_args()
sys.argv = sys.argv_saved

REPO = lint.REPO
cidx = lint.c_index()
pats = lint.allow_patterns()
root = os.path.join(a.crate, 'src')
tu_dir = os.path.join(root, 'tu')
cache = {}


def load(p):
    if p not in cache:
        cache[p] = open(p, encoding='utf-8').read()
    return cache[p]


tu_names = {}
for f in os.listdir(tu_dir):
    if f.endswith('.rs') and f != 'mod.rs':
        tu_names[f[:-3].lower()] = f[:-3]
# every C-named fn defined in any tu file of any crate
tu_defined = set()
for crate in os.listdir(os.path.join(REPO, 'rust')):
    d = os.path.join(REPO, 'rust', crate, 'src', 'tu')
    if os.path.isdir(d):
        for dp, dn, fs in os.walk(d):
            for f in fs:
                if f.endswith('.rs'):
                    t = open(os.path.join(dp, f), encoding='utf-8', errors='replace').read()
                    for m in lint.FN.finditer(t):
                        tu_defined.add(m.group(1))

skipset = set(a.skip.split(',')) if a.skip else set()
files = []
for dp, dn, fs in os.walk(root):
    rel = os.path.relpath(dp, root).replace(os.sep, '/')
    top = rel.split('/')[0]
    if top in ('tu', 'raw', 'c2r', 'types_gen'):
        continue
    for f in fs:
        if not f.endswith('.rs') or f in ('lib.rs', 'gen.rs'):
            continue
        r = os.path.relpath(os.path.join(dp, f), REPO).replace(os.sep, '/')
        if lint.allowed(r, pats):
            continue
        if a.files and f not in a.files.split(','):
            continue
        if f in skipset:
            continue
        files.append(os.path.join(dp, f))

moved = 0
for sp in sorted(files):
    s = load(sp)
    items = split_items(s)
    sel = []
    for it in items:
        if it[2] != 'fn' or not it[3]:
            continue
        if it[3].lower() not in cidx:
            continue
        # skip trivial / declaration bodies
        b = lint.body_of(it[4], it[4].index('fn ' + it[3]))
        if b is None or lint.trivial(it[4][b[0] + 1:b[1]]):
            continue
        sel.append(it)
    if not sel:
        continue
    modname = os.path.splitext(os.path.basename(sp))[0]
    groups = {}
    for it in sel:
        cname, stem = cidx[it[3].lower()]
        key = tu_names.get(stem.lower())
        if key is None:
            key = stem.lower()
        groups.setdefault(key, []).append(it)
    uses = [it[4].strip() for it in items if it[2] == 'use']
    gone_all = {it[3] for it in sel}
    ren = {}
    for key, its in groups.items():
        tp = os.path.join(tu_dir, key + '.rs')
        if key not in tu_names.values() and key.lower() not in tu_names:
            cstem = cidx[its[0][3].lower()][1]
            with open(tp, 'w', encoding='utf-8', newline=chr(10)) as nf:
                nf.write('//! ' + cstem + '.c: ports folded from the hand-written modules (consolidation steps 21/22).' + chr(10)
                         + '#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]' + chr(10))
            with open(os.path.join(tu_dir, 'mod.rs'), 'a', encoding='utf-8', newline=chr(10)) as mf:
                mf.write('pub mod ' + key + ';' + chr(10))
            tu_names[key.lower()] = key
        tu = load(tp)
        texts = []
        for it in its:
            n = it[3]
            new = n
            if n in tu_defined or re.search(r'\bfn\s+' + re.escape(n) + r'\b', tu):
                new = n + '_fp'
            ren[n] = (key, new)
            txt = it[4].strip('\n')
            if not re.search(r'(?m)^pub\b', txt):
                txt = re.sub(r'(?m)^((?:unsafe |extern "C" )*fn )', r'pub \1', txt, count=1)
            texts.append((n, new, txt))
        # rename inside texts (self-calls among moved)
        out_t = []
        for (n, new, txt) in texts:
            for (o, (k2, nw)) in ren.items():
                if o != nw and k2 == key:
                    txt = re.sub(r'(?<![A-Za-z0-9_:])' + re.escape(o) + r'(?![A-Za-z0-9_])', nw, txt)
            out_t.append(txt)
        have = set()
        for um in re.finditer(r'(?ms)^(?:pub(?:\([a-z]+\))?\s+)?use\s+.*?;', tu):
            try:
                have.update(use_leaves(um.group(0)))
            except Exception:
                pass
        add = []
        for u in uses:
            u = re.sub(r'^(#\[[^\]]*\]\s*)+', '', u)
            mu = re.search(r'(?m)^(?:pub(?:\([a-z]+\))?\s+)?use\s', u)
            u = u[mu.start():] if mu else u
            if re.match(r'pub use crate::tu::', u):
                continue
            try:
                lv = use_leaves(u)
            except Exception:
                continue
            fresh = [l for l in lv if l not in have]
            if not fresh:
                continue
            mb = re.match(r'((?:pub(?:\([a-z]+\))?\s+)?use\s+[A-Za-z0-9_:]+::)\{(.*)\};', u, re.S)
            if mb and len(fresh) != len(lv):
                u = mb.group(1) + '{' + ', '.join(fresh) + '};'
            have.update(fresh)
            add.append('#[allow(unused_imports)]\n' + re.sub(r'^(?:pub(?:\([a-z]+\))?\s+)?use', 'use', u))
        srcnames = {it2[3] for it2 in items if it2[3] and it2[2] in ('fn', 'const', 'static', 'struct', 'enum', 'type', 'union') and it2[3] not in gone_all}
        used = set()
        for txt in out_t:
            used.update(IDENT.findall(txt))
        def dest_defines(n):
            return re.search(r'(?m)^(?:pub(?:\([a-z]+\))?\s+)?(?:unsafe\s+)?(?:extern\s+"C"\s+)?(?:fn|const|static(?:\s+mut)?|struct|enum|type|union)\s+' + re.escape(n) + r'\b', tu) or n in have
        need = []
        for n in sorted(used & srcnames):
            if not dest_defines(n):
                need.append(n)
            else:
                # the destination has its own item of that name: bind the source's under an alias
                al = n + '_' + modname
                out_t = [re.sub(r'(?<![A-Za-z0-9_:.])' + re.escape(n) + r'(?![A-Za-z0-9_])', al, t) for t in out_t]
                need.append(n + ' as ' + al)
        if need:
            add.append('#[allow(unused_imports)]\nuse crate::%s::{%s};' % (modname, ', '.join(need)))
        lines = tu.rstrip('\n').split('\n')
        p0 = 0
        for idx, ln in enumerate(lines):
            if ln.startswith('//!') or ln.startswith('#![') or ln.strip() == '':
                p0 = idx + 1
            else:
                break
        lines[p0:p0] = ['#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]'] + add
        tu = '\n'.join(lines) + '\n\n// ---- folded from %s.rs (consolidation steps 21/22) ----\n' % modname + '\n\n'.join(out_t) + '\n'
        cache[tp] = tu
        moved += len(its)
    # rewrite the source: drop moved fns, make the rest pub(crate), add back-references
    gone = {it[3] for it in sel}
    keep = []
    for it in items:
        if it[3] in gone and it[2] == 'fn':
            continue
        t = it[4]
        if it[2] in ('fn', 'const', 'static', 'struct', 'enum', 'type', 'union') and it[3]:
            # make private top-level items visible to the destination
            t = re.sub(r'(?m)^(\s*)((?:unsafe\s+|extern\s+"C"\s+)*(?:fn|const|static\s+mut|static|struct|enum|type|union)\s)', lambda m: m.group(1) + 'pub(crate) ' + m.group(2), t, count=1) if not re.match(r'(?s)\s*(?:(?:///[^\n]*|//[^\n]*|#\[[^\]]*\])\s*)*pub', t) else t
        keep.append(t)
    back = []
    for o, (key, nw) in sorted(ren.items()):
        back.append('pub use crate::tu::%s::%s%s;' % (key, nw, '' if nw == o else ' as ' + o))
    first_item = items[0][0] if items else 0
    # put back-references after the leading inner attrs/docs: simply append at the end of file
    new_s = ''.join(('\n' + k.strip('\n') + '\n') for k in keep if k.strip()) + '\n' + '\n'.join(back) + '\n'
    # preserve the file header (anything before the first item)
    header = s[:items[0][0]] if items else ''
    cache[sp] = header + new_s

if not a.dry:
    for p, t in cache.items():
        open(p, 'w', encoding='utf-8', newline=chr(10)).write(t)
print('moved', moved)
