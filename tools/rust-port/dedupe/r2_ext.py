#!/usr/bin/env python3
"""R2 in an ext_fns! default: the entry `fn Name(a: T..) -> R = <typed model call>` duplicates a canonical tu fn of the same C name.
When the canonical fn (pub, in a tu file of this crate or an upstream crate) has the same parameter and return types, the default becomes a
call of it (a forwarder: not a body for the lint). Entries whose types differ are listed with --report and left alone.

  r2_ext.py [--apply] [--crate melee_fighter] [--report]
"""
import os, re, sys, argparse
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(here, '..'))
import lint_logic as L
ap = argparse.ArgumentParser()
ap.add_argument('--apply', action='store_true')
ap.add_argument('--crate')
ap.add_argument('--report', action='store_true')
a = ap.parse_args()
os.chdir(L.REPO)
v, canon, bodies = L.violations(L.allow_patterns())
ORDER = ['melee_core', 'melee_types', 'melee_runtime', 'melee_world', 'melee_fighter', 'melee_chara', 'melee_items', 'melee_modes']

def split_top(s):
    out, d, cur = [], 0, ''
    for ch in s:
        if ch in '<([{': d += 1
        if ch in '>)]}': d -= 1
        if ch == ',' and d == 0:
            out.append(cur); cur = ''
        else:
            cur += ch
    if cur.strip(): out.append(cur)
    return out

def sig(params, ret):
    ps = []
    for p in split_top(params):
        p = p.strip()
        if not p: continue
        ps.append(re.sub(r'\s+', '', p.split(':', 1)[1] if ':' in p else p))
    return ps, re.sub(r'\s+', '', ret or '')

cache = {}
def load(p):
    if p not in cache:
        cache[p] = open(p, encoding='utf-8').read()
    return cache[p]

def crate_of(rel): return rel.split('/')[1]
def canon_sig(path, name):
    t = load(path)
    m = re.search(r'(?m)^pub\s+(unsafe\s+)?(?:extern\s+"C"\s+)?fn\s+' + re.escape(name) + r'\s*(?:<[^>]*>)?\(([^)]*)\)\s*(?:->\s*([^{]+?))?\s*\{', t)
    if not m: return None
    return sig(m.group(2), m.group(3)), bool(m.group(1))

n_ok = n_bad = 0
for r, rel, name in v:
    if r != 'R2': continue
    if a.crate and crate_of(rel) != a.crate: continue
    if '/src/tu/' not in rel: continue
    cname, home = canon[name.lower()] if name.lower() in canon else (name, None)
    if home is None or home == rel: continue
    t = load(rel)
    m = re.search(r'(?m)^(\s+)fn\s+' + re.escape(name) + r'\s*\(([^)]*)\)\s*(?:->\s*([^=]+?))?\s*=\s*', t)
    if not m: continue
    # make sure the entry sits in an ext_fns block
    pre = t[:m.start()]
    if pre.rfind('ext_fns!') < 0 or pre.rfind('ext_fns!') < pre.rfind('\n}\n'): continue
    hc, rc = crate_of(home), crate_of(rel)
    if ORDER.index(hc) > ORDER.index(rc): continue
    cs = canon_sig(home, name)
    es = sig(m.group(2), m.group(3))
    if not cs or cs[0] != es:
        n_bad += 1
        if a.report: print('SIG', rel, name, es, cs[0] if cs else None)
        continue
    # find the end of the rhs: up to ';' at depth 0
    i = m.end(); d = 0
    while i < len(t):
        ch = t[i]
        if ch in '({[': d += 1
        elif ch in ')}]': d -= 1
        elif ch == ';' and d == 0: break
        i += 1
    stem = os.path.splitext(os.path.basename(home))[0]
    path = ('crate' if hc == rc else hc) + '::tu::' + stem + '::' + name
    if '/tu/' in home.split('src/')[1][2:] and home.split('src/tu/')[1].count('/'):
        sub = home.split('src/tu/')[1][:-3].replace('/', '::')
        path = ('crate' if hc == rc else hc) + '::tu::' + sub
        path += '::' + name
    args = ', '.join(re.match(r'\s*(?:mut\s+)?(\w+)', p).group(1) for p in split_top(m.group(2)) if p.strip())
    new = '{ unsafe { %s(%s) } }' % (path, args)
    if cs[1] is False:
        new = '{ %s(%s) }' % (path, args)
    cache[rel] = t[:m.end()] + new + t[i:]
    n_ok += 1
    if a.report: print('OK ', rel, name, '->', path)
print('convertible', n_ok, 'signature mismatch', n_bad)
if a.apply:
    for p, t in cache.items():
        open(p, 'w', encoding='utf-8', newline='\n').write(t)
