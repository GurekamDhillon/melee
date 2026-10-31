#!/usr/bin/env python3
"""Per-chunk call-COUNT diff between the C build and the Rust engine (robust to inlining and phase differences).
  python countdiff.py --c c.seq --map melee-pc.map --rust r.ct --anchor Fighter_8006A1BC --clean A-B --bad C-D [--filter RX] [--top N]
Chunks are cut at every call of --anchor on each side (chunk k of C is compared with chunk k of Rust). Names whose per-chunk
count differs anywhere in the --clean chunk range (frames both sides agree on) are treated as inlining noise; then the
first --bad chunk with a count difference outside the noise set is printed with the names and counts."""
import argparse, re, sys
from collections import Counter
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
src = (Path(__file__).resolve().parent / 'seqdiff.py').read_text().replace('sys.exit(main())', '')
ns = {'__file__': str(Path(__file__).resolve().parent / 'seqdiff.py')}
exec(compile(src, 'seqdiff', 'exec'), ns)
ap = argparse.ArgumentParser()
ap.add_argument('--c', required=True); ap.add_argument('--map', required=True); ap.add_argument('--rust', required=True)
ap.add_argument('--anchor', default='Fighter_8006A1BC'); ap.add_argument('--clean', required=True); ap.add_argument('--bad', required=True)
ap.add_argument('--filter', default=None); ap.add_argument('--top', type=int, default=40)
ap.add_argument("--all", action="store_true")
a = ap.parse_args()
norm = ns['norm']
c, r = ns['read_c'](a.c, a.map), ns['read_rust'](a.rust)
anc = norm(a.anchor)
rx = re.compile(a.filter, re.I) if a.filter else None
def chunks(d):
    out = []
    for f in sorted(d):
        if f < 0: continue
        for n in d[f]:
            if n == anc: out.append((f, Counter()))
            if out and (rx is None or rx.search(n)): out[-1][1][n] += 1
    return out
cc, rc = chunks(c), chunks(r)
cl = tuple(map(int, a.clean.split('-'))); bd = tuple(map(int, a.bad.split('-')))
noise = set()
for (fc, x), (fr, y) in zip(cc, rc):
    if cl[0] <= fc <= cl[1]:
        for n in set(x) | set(y):
            if x[n] != y[n]: noise.add(n)
print('noise names:', len(noise))
for (fc, x), (fr, y) in zip(cc, rc):
    if bd[0] <= fc <= bd[1]:
        d = [(n, x[n], y[n]) for n in set(x) | set(y) if x[n] != y[n] and n not in noise]
        if d:
            print(f'chunk C frame {fc} / Rust frame {fr}: {len(d)} names differ (name, C, Rust)')
            for n, p, q in sorted(d)[:a.top]: print('  ', n, p, q)
            if not a.all: break
else:
    print('no count difference outside noise in the bad range')
