#!/usr/bin/env python3
"""Second pass of the fold: every remaining `pub fn` in a `mod <tu> { }` block under <crate>/src/<prefix>
moves to <crate>/src/tu/<tu>.rs (the module name is the C file's stem). A name the tu file already
defines is moved as `<name>_fp` (the typed variant of a gobj callback); the source block keeps a
`pub use crate::tu::<tu>::<new> as <old>;` so every existing path resolves.

  fold_blocks.py --crate rust/melee_chara --prefix chars [--dry]
"""
import re, os, sys, argparse

sys.argv_saved = sys.argv
src = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), 'fold_reexports.py'), encoding='utf-8').read()
exec(src.split('def main():')[0])

ap = argparse.ArgumentParser()
ap.add_argument('--crate', required=True)
ap.add_argument('--prefix', default='chars')
ap.add_argument('--dry', action='store_true')
a = ap.parse_args()
root = os.path.join(a.crate, 'src')
tu_dir = os.path.join(root, 'tu')
tu_files = {f[:-3] for f in os.listdir(tu_dir) if f.endswith('.rs')}
cache = {}


def load(p):
    if p not in cache:
        cache[p] = open(p, encoding='utf-8').read()
    return cache[p]


def defined(tu, name):
    return re.search(r'\bfn\s+' + re.escape(name) + r'\b|\b(?:const|static(?:\s+mut)?|struct|enum|type)\s+' + re.escape(name) + r'\b', tu) is not None


moved = 0
skipped = []
for dp, dn, fns in os.walk(os.path.join(root, a.prefix)):
    for f in sorted(fns):
        if not f.endswith('.rs') or f in ('mod.rs', 'cb.rs'):
            continue
        sp = os.path.join(dp, f)
        s = load(sp)
        pos = 0
        out = []
        changed = False
        for m in re.finditer(r'(?m)^(?:pub(?:\([a-z]+\))?\s+)?mod\s+([A-Za-z0-9_]+)\s*\{', s):
            modname = m.group(1)
            if m.start() < pos:
                continue
            if modname not in tu_files:
                continue
            ob = s.index('{', m.start())
            cb = scan_close(s, ob)
            inner = s[ob + 1:cb]
            items = split_items(inner)
            tp = os.path.join(tu_dir, modname + '.rs')
            tu = load(tp)
            by_name = {}
            for it in items:
                if it[3]:
                    by_name.setdefault(it[3], []).append(it)
            sel = []
            for it in items:
                if it[2] == 'fn' and it[3]:
                    sel.append(it[3])
            selset = set(sel)
            work = list(sel)
            while work:
                w = work.pop()
                for it in by_name.get(w, []):
                    if it[2] == 'use':
                        continue
                    for idn in set(IDENT.findall(it[4])):
                        if idn in by_name and idn not in selset and idn != w and by_name[idn][0][2] != 'use':
                            selset.add(idn)
                            work.append(idn)
            if not selset:
                continue
            movable = [it for it in items if it[3] in selset and it[2] != 'use']
            ren = {}
            texts = []
            for it in movable:
                n = it[3]
                txt = it[4].strip('\n')
                if defined(tu, n):
                    if it[2] == 'fn':
                        ren[n] = n + '_fp'
                    else:
                        skipped.append((sp, modname, n))
                        continue
                texts.append((n, it, txt))
            if not texts:
                continue
            # rename inside moved texts (calls between moved items)
            def rn(t):
                for o, nw in ren.items():
                    t = re.sub(r'(?<![A-Za-z0-9_])' + re.escape(o) + r'(?![A-Za-z0-9_])', nw, t)
                return t
            newtexts = [rn(t) if ren else t for (n, it, t) in texts]
            uses = [it[4].strip() for it in items if it[2] == 'use']
            tu2 = tu.rstrip('\n')
            # imports
            have = set()
            for um in re.finditer(r'(?ms)^[ \t]*(?:pub(?:\([a-z]+\))?\s+)?use\s+.*?;', tu2):
                try:
                    have.update(use_leaves(um.group(0)))
                except Exception:
                    pass
            add = []
            for u in uses:
                u = re.sub(r'^(#\[[^\]]*\]\s*)+', '', u)
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
            lines = tu2.split('\n')
            p0 = 0
            for idx, ln in enumerate(lines):
                if ln.startswith('//!') or ln.startswith('#![') or ln.strip() == '':
                    p0 = idx + 1
                else:
                    break
            lines[p0:p0] = ['#![allow(unused_variables, unused_parens, unused_comparisons, unused_unsafe, unused_mut)]'] + add
            tu2 = '\n'.join(lines) + '\n\n// ---- folded from %s blocks (consolidation step 20) ----\n' % a.prefix + '\n\n'.join(newtexts) + '\n'
            cache[tp] = tu2
            back = []
            for (n, it, t) in texts:
                if n in ren:
                    back.append('pub use crate::tu::%s::%s as %s;' % (modname, ren[n], n))
                else:
                    back.append('pub use crate::tu::%s::%s;' % (modname, n))
            gone = {n for (n, it, t) in texts}
            keep = [it[4] for it in items if not (it[3] in gone and it[2] != 'use')]
            newinner = '\n' + '\n'.join(back) + '\n' + ''.join('\n' + k.strip('\n') + '\n' for k in keep if k.strip()) + '\n'
            s = s[:ob + 1] + newinner + s[cb:]
            cache[sp] = s
            moved += len(texts)
            changed = True
            break_all = True
            # offsets changed: restart scanning this file
            # (simple approach: re-run the whole script until no more moves)
            break
if not a.dry:
    for p, t in cache.items():
        open(p, 'w', encoding='utf-8', newline='\n').write(t)
print('moved', moved)
for x in skipped:
    print('skipped', *x)
