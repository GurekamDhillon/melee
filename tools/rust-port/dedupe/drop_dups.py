#!/usr/bin/env python3
"""Step 23: remove the non-canonical bodies (lint R2) of C-named functions.
A removed body whose name is still mentioned by another non-test source file is replaced by
`pub use <canonical path>::<CName> as <name>;` so the callers keep resolving to the one canonical body.
  python tools/rust-port/dedupe/drop_dups.py [--only SUBSTR] [--methods] [--skip FILE] [--dry] [--noreexport]
Skip file lines: `<rel path>\t<name>` (never touched)."""
import re, sys
from pathlib import Path
from collections import Counter
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import lint_logic as L
args = sys.argv[1:]
def opt(n, d=''):
    return args[args.index(n) + 1] if n in args else d
only = opt('--only'); skipf = opt('--skip'); dry = '--dry' in args
CRATES = ('melee_runtime', 'melee_world', 'melee_items', 'melee_fighter', 'melee_chara', 'melee_modes')
skip = set()
if skipf and Path(skipf).exists():
    for l in Path(skipf).read_text().splitlines():
        p = l.split('\t')
        if len(p) == 2: skip.add((p[0], p[1].lower()))
gskip = {n for (r, n) in skip if r == '*'}
def rd(f):
    return open(f, encoding='utf-8', errors='surrogateescape', newline='').read()
pats = L.allow_patterns()
v, canon, bodies = L.violations(pats)
want = {}
for r, rel, n in v:
    if r == 'R2' and n.lower() not in gskip and rel.split('/')[1] in CRATES[int('--runtime' not in args):] and only in rel and not any(x in rel for x in opt('--exclude').split(',') if x) and (rel, n.lower()) not in skip:
        want.setdefault(rel, set()).add(n.lower())
# names that are `static` in some C file: per-TU, so equal names in two Rust TUs need not be the same function
static_names = set()
for cf in list((L.REPO / 'src/melee').rglob('*.c')) + list((L.REPO / 'src/sysdolphin').rglob('*.c')) + list((L.REPO / 'pc/gameworld').glob('*.c')):
    try: ct = L.strip_c(cf.read_text(errors='replace'))
    except OSError: continue
    for cm_ in L.CDEF.finditer(ct):
        if 'static' in cm_.group('pre').split(): static_names.add(cm_.group('name').lower())
# identifier counts over all non-test, non-raw sources
tok = Counter(); defs = Counter()
texts = {}
for f, rel in L.rust_files():
    t = rd(f); texts[rel] = t
    for m in re.finditer(r'\b[A-Za-z_]\w*\b', t): tok[m.group(0)] += 1
    for m in re.finditer(r'\bfn\s+([A-Za-z_]\w*)', t): defs[m.group(1)] += 1
# hooks someone other than the link layer installs (host seams): never remove the wrapper that consults them
seam_set = set()
for rel, t in texts.items():
    if rel.endswith('gen_table.rs') or '/melee_link/' in rel: continue
    for m in re.finditer(r'(?:H_|HOOK_|hook::)(\w+)\s*=\s*Some', t): seam_set.add(m.group(1).lower())
def sig(t, m):
    """(is_unsafe, param types, is_pub, is_extern, return type) of the fn whose FN match is m in text t."""
    b = L.body_of(t, m.start())
    hdr = t[m.start():b[0]] if b else t[m.start():m.start() + 300]
    i = hdr.index('fn')
    j = hdr.index('(', i)
    d = 0; k = j; cur = ''; params = []
    while k < len(hdr):
        c = hdr[k]
        if c in '(<[':
            d += 1
            if d > 1: cur += c
        elif c in ')>]':
            if c == '>' and hdr[k - 1] == '-': cur += c
            else:
                d -= 1
                if d == 0:
                    params.append(cur); break
                cur += c
        elif c == ',' and d == 1:
            params.append(cur); cur = ''
        else:
            cur += c
        k += 1
    types = []
    for p_ in params:
        if not p_.strip(): continue
        types.append(''.join(p_.split(':', 1)[1].split()) if ':' in p_ else ''.join(p_.split()))
    rest = hdr[k + 1:]
    ret = ''.join(rest.split('->', 1)[1].split()) if '->' in rest else ''
    return ('unsafe' in hdr[:i + 2], tuple(types), bool(re.match(r'\s*pub', hdr)), 'extern' in hdr[:i + 2], ret)
_csig = {}
def canon_sig(crel, cname):
    key = (crel, cname)
    if key not in _csig:
        t = texts.get(crel) or rd(L.REPO / crel)
        r = None
        for m in L.FN.finditer(t):
            if m.group(1) == cname:
                b = L.body_of(t, m.start())
                if b is not None and not L.trivial(t[b[0] + 1:b[1]], cname):
                    r = sig(t, m); break
        _csig[key] = r
    return _csig[key]
def modpath(rel, from_crate):
    p = Path(rel); parts = list(p.parts)
    crate = parts[1]
    i = parts.index('tu')
    mod = '::'.join([x[:-3] if x.endswith('.rs') else x for x in parts[i:]])
    if mod.endswith('::mod'): mod = mod[:-5]
    return ('crate' if crate == from_crate else crate) + '::' + mod
total = kept = 0
for rel, names in want.items():
    f = L.REPO / rel
    t = texts.get(rel) or rd(f)
    crate = rel.split('/')[1]
    spans = []
    for m in L.FN.finditer(t):
        n = m.group(1)
        if n.lower() not in names: continue
        b = L.body_of(t, m.start())
        if b is None or L.trivial(t[b[0] + 1:b[1]], n): continue
        hdr = t[m.start():b[0]]
        if n in ('GET_ATTRS', 'fn_8022697C') or n.lower() in static_names or '_' not in n: continue
        if not re.match(r'\s*pub', hdr): continue   # private: a C static (same name in several TUs)
        _cn, _cr = canon[n.lower()]
        _cs = canon_sig(_cr, _cn)
        if _cs is None or not _cs[2] or _cs[:2] + _cs[3:] != sig(t, m)[:2] + sig(t, m)[3:]: continue   # the canonical body differs in safety / arity
        if n.lower() in seam_set or n.lower().startswith(('lbaudioax_', 'mex_', 'geno_', 'osreport', 'hsd_dvd')): continue
        if '--methods' not in args and re.search(r"[(,]\s*&?(mut\s+)?self\b", hdr): continue
        if re.search(r'\)\s*(->[^={;]*)?=', hdr): continue   # an ext_fns! / hook entry with a default
        if re.match(r'[ \t]{4,}', hdr) and 'mod' not in hdr and False: continue
        s = t.rfind('\n', 0, m.start()) + 1
        while True:
            p = t.rfind('\n', 0, s - 1) + 1
            line = t[p:s - 1].strip()
            if s > 0 and (line.startswith('///') or line.startswith('#[') or line.startswith('//')) and p < s:
                s = p
            else:
                break
        e = b[1] + 1
        if e < len(t) and t[e] == '\n': e += 1
        if CRATES.index(canon[n.lower()][1].split('/')[1]) > CRATES.index(crate) if canon[n.lower()][1].split('/')[1] in CRATES else True: continue   # canonical lives in a downstream crate
        spans.append((s, e, n))
    spans.sort()
    out = []
    for sp in spans:
        if out and sp[0] < out[-1][1]: continue
        out.append(sp)
    for s, e, n in reversed(out):
        cname, crel = canon[n.lower()]
        used = tok[n] > defs[n] and '--noreexport' not in args
        top = not re.match(r'[ \t]', t[s:e].lstrip('\n')[:1] or 'x') or True
        if used and crel != rel and crel.split('/')[1] in CRATES:
            indent = re.match(r'[ \t]*', t[s:e].split('\n')[-2] if False else '').group(0)
            as_ = '' if cname == n else ' as ' + n
            new = 'pub use %s::%s%s;\n' % (modpath(crel, crate), cname, as_)
            kept += 1
        else:
            new = ''
        t = t[:s] + new + t[e:]
        total += 1
    if out and not dry:
        open(f, 'w', encoding='utf-8', errors='surrogateescape', newline='').write(t)
print('removed', total, 'bodies in', len(want), 'files;', kept, 'replaced by a re-export')
