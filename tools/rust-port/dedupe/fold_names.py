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

def use_pairs(u):
    m = re.match(r'\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([a-z]+\))?\s+)?use\s+(.*?);', u, re.S)
    p = m.group(1)
    out = []

    def rec(prefix, t):
        t = t.strip()
        if t.startswith('{') and t.endswith('}'):
            depth = 0
            cur = ''
            parts = []
            for ch in t[1:-1]:
                if ch == '{':
                    depth += 1
                if ch == '}':
                    depth -= 1
                if ch == ',' and depth == 0:
                    parts.append(cur)
                    cur = ''
                else:
                    cur += ch
            parts.append(cur)
            for q in parts:
                if q.strip():
                    rec(prefix, q)
            return
        if '{' in t:
            a, b = t.split('{', 1)
            rec(prefix + a, '{' + b)
            return
        full = (prefix + t).strip()
        if full.endswith('::self'):
            full = full[:-6]
        if full.endswith('*'):
            out.append((full, full))
        elif ' as ' in full:
            pth, al = full.split(' as ')
            out.append((al.strip(), pth.strip()))
        else:
            out.append((full.split('::')[-1].strip(), full))

    rec('', p)
    return out


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
tu_paths = {}
for dp_, dn_, fs_ in os.walk(tu_dir):
    for f in fs_:
        if f.endswith('.rs') and f != 'mod.rs':
            if f[:-3].lower() not in tu_names or dp_ == tu_dir:
                tu_names[f[:-3].lower()] = f[:-3]
                tu_paths[f[:-3].lower()] = os.path.join(dp_, f)
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
                        bb = lint.body_of(t, m.start())
                        if bb is None or lint.trivial(t[bb[0] + 1:bb[1]]):
                            continue
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


def process(sp, items, modpath, parentpath, modname):
    global moved
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
        return None
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
        tp = tu_paths.get(key.lower(), os.path.join(tu_dir, key + '.rs'))
        if key not in tu_names.values() and key.lower() not in tu_names:
            cstem = cidx[its[0][3].lower()][1]
            with open(tp, 'w', encoding='utf-8', newline=chr(10)) as nf:
                nf.write('//! ' + cstem + '.c: ports folded from the hand-written modules (consolidation steps 21/22).' + chr(10)
                         + '#![allow(non_snake_case, non_upper_case_globals, unused_unsafe, static_mut_refs)]' + chr(10))
            with open(os.path.join(tu_dir, 'mod.rs'), 'a', encoding='utf-8', newline=chr(10)) as mf:
                mf.write('pub mod ' + key + ';' + chr(10))
            tu_names[key.lower()] = key
            tu_paths[key.lower()] = tp
        tu = load(tp)
        texts = []
        for it in its:
            n = it[3]
            new = n
            if n in tu_defined or re.search(r'\bfn\s+' + re.escape(n) + r'\b', tu):
                new = n + '_fp'
                while re.search(r'\bfn\s+' + re.escape(new) + r'\b', tu) or any(v[1] == new and v[0] == key for v in ren.values()):
                    new += '2'
            ren[n] = (key, new)
            txt = it[4].strip('\n')
            txt = re.sub(r'(?m)^//!', '//', txt)
            if not re.search(r'(?m)^pub\b', txt):
                txt = re.sub(r'(?m)^((?:unsafe |extern "C" )*fn )', r'pub \1', txt, count=1)
            texts.append((n, new, txt))
        # rename inside texts (self-calls among moved)
        out_t = []
        for (n, new, txt) in texts:
            for (o, (k2, nw)) in ren.items():
                if o != nw and k2 == key:
                    txt = re.sub(r'(?<![A-Za-z0-9_:.])' + re.escape(o) + r'(?![A-Za-z0-9_])', nw, txt)
            out_t.append(txt)
        have = set()
        have_map = {}
        for um in re.finditer(r'(?ms)^(?:pub(?:\([a-z]+\))?\s+)?use\s+.*?;', tu):
            try:
                for lf, fp_ in use_pairs(um.group(0)):
                    have.add(lf)
                    have_map.setdefault(lf, fp_)
            except Exception:
                pass
        roots0 = set()
        for u0 in uses:
            for m0 in re.finditer(r'use\s+([A-Za-z_][A-Za-z0-9_]*)::', u0):
                roots0.add(m0.group(1))
            for m0 in re.finditer(r'[{,]\s*([A-Za-z_][A-Za-z0-9_]*)::', u0):
                roots0.add(m0.group(1))
        used0 = set()
        for t0 in out_t:
            used0.update(IDENT.findall(t0))
        add = []
        for u in uses:
            u = re.sub(r'^(#\[[^\]]*\]\s*)+', '', u)
            mu = re.search(r'(?m)^(?:pub(?:\([a-z]+\))?\s+)?use\s', u)
            u = u[mu.start():] if mu else u
            if re.match(r'pub use crate::tu::', u):
                continue
            try:
                prs = use_pairs(u)
            except Exception:
                continue
            def absol(q):
                if q.startswith('super::'):
                    return parentpath + '::' + q[7:]
                if q.startswith('self::'):
                    return modpath + '::' + q[6:]
                return q
            prs = [(lf if not lf.startswith(('super::', 'self::')) else absol(lf), absol(fp_)) for lf, fp_ in prs]
            for lf, fp_ in prs:
                if not lf.endswith('*') and lf.split('::')[-1] not in used0 and lf not in roots0:
                    continue
                defd = (not lf.endswith('*')) and lf not in roots0 and re.search(r'(?<![A-Za-z0-9_])' + re.escape(lf.split('::')[-1]) + r'(?![A-Za-z0-9_])', tu)
                if lf in have_map or defd:
                    if lf in have_map and have_map[lf] == fp_:
                        continue
                    if lf.endswith('*'):
                        continue
                    al = lf + '_' + modname
                    out_t = [re.sub(r'(?<![A-Za-z0-9_:.])' + re.escape(lf) + r'(?![A-Za-z0-9_])', al, t) for t in out_t]
                    have_map[al] = fp_
                    have.add(al)
                    add.append('#[allow(unused_imports)]\nuse %s as %s;' % (fp_, al))
                else:
                    have_map[lf] = fp_
                    have.add(lf)
                    if lf.endswith('*'):
                        add.append('#[allow(unused_imports)]\nuse %s;' % fp_)
                    elif fp_.split('::')[-1] == lf:
                        add.append('#[allow(unused_imports)]\nuse %s;' % fp_)
                    else:
                        add.append('#[allow(unused_imports)]\nuse %s as %s;' % (fp_, lf))
        srcnames = {it2[3] for it2 in items if it2[3] and it2[2] in ('fn', 'const', 'static', 'struct', 'enum', 'type', 'union', 'mod', 'trait') and it2[3] not in {x[3] for x in its}}
        used = set()
        for txt in out_t:
            used.update(IDENT.findall(txt))
        def dest_defines(n):
            return re.search(r'(?<![A-Za-z0-9_])' + re.escape(n) + r'(?![A-Za-z0-9_])', tu) or n in have
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
            add.append('#[allow(unused_imports)]\nuse %s::{%s};' % (modpath, ', '.join(need)))
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
        if it[2] in ('fn', 'const', 'static', 'struct', 'enum', 'type', 'union', 'mod', 'trait') and it[3]:
            # make private top-level items visible to the destination
            t = re.sub(r'(?m)^(\s*)((?:unsafe\s+|extern\s+"C"\s+)*(?:fn|const|static\s+mut|static|struct|enum|type|union|mod|trait)\s)', lambda m: m.group(1) + 'pub(crate) ' + m.group(2), t, count=1) if not re.match(r'(?s)\s*(?:(?:///[^\n]*|//[^\n]*|#\[[^\]]*\])\s*)*pub', t) else t
        keep.append(t)
    back = []
    for o, (key, nw) in sorted(ren.items()):
        back.append('pub use crate::tu::%s::%s%s;' % (key, nw, '' if nw == o else ' as ' + o))
    return keep, back


stems_lc = {v[1].lower() for v in cidx.values()}
for sp in sorted(files):
    s = load(sp)
    modname0 = os.path.splitext(os.path.basename(sp))[0]
    relp = os.path.relpath(sp, root).replace(os.sep, '/')[:-3].split('/')
    if relp[-1] == 'mod':
        relp = relp[:-1]
    fpath = 'crate' + ''.join('::' + x for x in relp)
    fparent = 'crate' + ''.join('::' + x for x in relp[:-1])
    if modname0 == 'mod' and relp:
        modname0 = relp[-1]
    # nested `mod <c stem> { ... }` blocks first
    pos = 0
    block_moved = False
    while True:
        m = re.compile(r'(?m)^(?:pub(?:\([a-z]+\))?\s+)?mod\s+([A-Za-z0-9_]+)\s*\{').search(s, pos)
        if not m:
            break
        pos = m.end()
        bname = m.group(1)
        if bname.lower() not in stems_lc:
            continue
        ob = s.index('{', m.start())
        cb = scan_close(s, ob)
        binner = s[ob + 1:cb]
        bitems = split_items(binner)
        res = process(sp, bitems, fpath + '::' + bname, fpath, bname)
        if res is None:
            continue
        keep, back = res
        block_moved = True
        newinner = chr(10) + chr(10).join(back) + chr(10) + ''.join((chr(10) + k.strip(chr(10)) + chr(10)) for k in keep if k.strip()) + chr(10)
        s = s[:ob + 1] + newinner + s[cb:]
        pos = ob + 1 + len(newinner)
    if block_moved:
        for it in reversed(split_items(s)):
            if it[2] in ('fn', 'const', 'static', 'struct', 'enum', 'type', 'union', 'trait') and it[3]:
                t = it[4]
                if not re.match(r'(?s)\s*(?:(?:///[^\n]*|//[^\n]*|#\[[^\]]*\])\s*)*pub', t):
                    t = re.sub(r'(?m)^(\s*)((?:unsafe\s+|extern\s+"C"\s+)*(?:fn|const|static\s+mut|static|struct|enum|type|union|trait)\s)', lambda m: m.group(1) + 'pub(crate) ' + m.group(2), t, count=1)
                    s = s[:it[0]] + t + s[it[1]:]
    cache[sp] = s
    items = split_items(s)
    res = process(sp, items, fpath, fparent, modname0)
    if res is None:
        continue
    keep, back = res
    new_s = ''.join((chr(10) + k.strip(chr(10)) + chr(10)) for k in keep if k.strip()) + chr(10) + chr(10).join(back) + chr(10)
    header = s[:items[0][0]] if items else ''
    cache[sp] = header + new_s

if not a.dry:
    for p, t in cache.items():
        open(p, 'w', encoding='utf-8', newline=chr(10)).write(t)
print('moved', moved)
