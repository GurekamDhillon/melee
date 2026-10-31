#!/usr/bin/env python3
"""Draw counts per golden frame from the `seed` column: python tools/rust-port/trace/rand_draws.py GOLDEN.csv [CANDIDATE.csv [frames]]
(the seed is the LCG state after the frame; draws counted from MELEE_TEST_SEED 12345). A candidate whose per-frame counts differ
from the golden's makes the same RNG draws in a different frame or order (the engine's `MELEE_RAND_TRACE=<file>` lists each draw
with its callers)."""
import sys,csv
MUL=214013; ADD=2531011
def step(s): return (s*MUL+ADD)&0xFFFFFFFF
def seq(start,n):
    s=start;d={}
    for i in range(1,n+1):
        s=step(s); d[s]=i
    return d
def load(p):
    return [int(r['seed'],16) for r in csv.DictReader(open(p))]
g=load(sys.argv[1]); c=load(sys.argv[2]) if len(sys.argv)>2 else None
N=int(sys.argv[3]) if len(sys.argv)>3 else 70
# map seed -> total draws from test seed 12345
tab=seq(12345,200000)
def total(sd): return tab.get(sd,-1)
prev_g=0;prev_c=0
for f in range(min(N,len(g))):
    tg=total(g[f]); tc=total(c[f]) if c and f<len(c) else None
    print(f, 'golden total',tg,'(+%d)'%(tg-prev_g), '' if tc is None else 'cand total %d (+%d)'%(tc,tc-prev_c))
    prev_g=tg
    if tc is not None: prev_c=tc
