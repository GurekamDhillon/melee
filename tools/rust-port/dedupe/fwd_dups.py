#!/usr/bin/env python3
"""Step 23: a non-canonical body (lint R2) of a C-named fn becomes a forwarder to the canonical body when the canonical fn
lives in a crate at or below this one in the dependency order. Params are passed with `as _`; compile errors are bisected per fn
(the fn is restored and remembered in fwd_skip.txt).
  fwd_dups.py [--crate melee_fighter] [--only SUBSTR] [--dry] [--check]    (--check = cargo check loop, reverting failing fns)
"""
import os, re, sys, subprocess, json
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(here, '..'))
import lint_logic as L
args = sys.argv[1:]
def opt(n, d=''): return args[args.index(n) + 1] if n in args else d
os.chdir(L.REPO)
ORDER = ['melee_runtime', 'melee_world', 'melee_items', 'melee_fighter', 'melee_chara', 'melee_modes']
skipf = L.REPO / 'tools/rust-port/dedupe/fwd_skip.txt'
skip = set(skipf.read_text().split('\n')) if skipf.exists() else set()
def rd(p): return open(p, encoding='utf-8', errors='surrogateescape', newline='').read()
def wr(p, t): open(p, 'w', encoding='utf-8', errors='surrogateescape', newline='').write(t)
def modpath(rel, from_crate):
    parts = rel.split('/'); crate = parts[1]; i = parts.index('tu')
    mod = '::'.join([x[:-3] if x.endswith('.rs') else x for x in parts[i:]])
    if mod.endswith('::mod'): mod = mod[:-5]
    return ('crate' if crate == from_crate else crate) + '::' + mod
def split_top(s):
    out, d, cur = [], 0, ''
    for ch in s:
        if ch in '<([{': d += 1
        if ch in '>)]}' and not (ch == '>' and cur.endswith('-')): d -= 1
        if ch == ',' and d == 0: out.append(cur); cur = ''
        else: cur += ch
    if cur.strip(): out.append(cur)
    return out
v, canon, bodies = L.violations(L.allow_patterns())
want = {}
for r, rel, n in v:
    if r != 'R2' or '/src/tu/' not in rel: continue
    if opt('--crate') and rel.split('/')[1] != opt('--crate'): continue
    if opt('--only') and opt('--only') not in rel: continue
    want.setdefault(rel, set()).add(n.lower())
def canon_info(crel, cname):
    t = rd(L.REPO / crel)
    m = re.search(r'(?m)^pub\s+(unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+' + re.escape(cname) + r'\s*[(<]', t)
    if not m: return None
    return bool(m.group(1))
edits = []   # (rel, name, old_body_text)
total = 0
for rel, names in sorted(want.items()):
    crate = rel.split('/')[1]
    t = rd(L.REPO / rel)
    spans = []
    for m in L.FN.finditer(t):
        n = m.group(1)
        if n.lower() not in names or (rel + ':' + n) in skip: continue
        b = L.body_of(t, m.start())
        if b is None or L.trivial(t[b[0] + 1:b[1]], n): continue
        cname, crel = canon[n.lower()]
        if crel == rel: continue
        cc = crel.split('/')[1]
        if cc not in ORDER or crate not in ORDER or ORDER.index(cc) > ORDER.index(crate): continue
        if '/src/tu/' not in crel: continue
        ci = canon_info(crel, cname)
        if ci is None: continue
        hdr = t[m.start():b[0]]
        if re.search(r'\bself\b', hdr) or re.search(r'\bfn\s+\w+\s*<', hdr) or ' = ' in hdr or '=>' in hdr: continue
        j = hdr.index('(', hdr.index('fn'))
        # parameter list
        d = 0; k = j; 
        while True:
            c = hdr[k]
            if c in '([<': d += 1
            elif c in ')]>' and not (c == '>' and hdr[k-1] == '-'):
                d -= 1
                if d == 0: break
            k += 1
        plist = hdr[j+1:k]; rest = hdr[k+1:]
        names_ = []
        okp = True
        for p in split_top(plist):
            p = p.strip()
            if not p: continue
            mm = re.match(r'(?:mut\s+)?([A-Za-z_]\w*)\s*:', p)
            if not mm: okp = False; break
            names_.append(mm.group(1))
        if not okp: continue
        has_ret = '->' in rest
        call = '%s::%s(%s)' % (modpath(crel, crate), cname, ', '.join(x + ' as _' for x in names_))
        newb = ' unsafe { %s as _ } ' % call if has_ret else ' unsafe { %s; } ' % call
        spans.append((b[0] + 1, b[1], n, t[b[0]+1:b[1]], newb))
    for s, e, n, old, newb in reversed(spans):
        t = t[:s] + newb + t[e:]
        edits.append((rel, n, old, newb)); total += 1
    if spans and '--dry' not in args: wr(L.REPO / rel, t)
print('forwarders:', total, 'in', len(set(e[0] for e in edits)), 'files')
json.dump([list(e) for e in edits], open(os.environ.get('TEMP', '/tmp') + '/fwd_edits.json', 'w'))
