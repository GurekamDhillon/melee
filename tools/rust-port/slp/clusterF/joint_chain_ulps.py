# Exact-f32 model of PSMTXConcat's translation column over a joint chain dumped as "T3191 j<n> ... T x y z ... M <12 hex>"
# (temporary engine dump, not committed); then which ulp shifts reproduce a recorded final position.
# Usage: joint_chain_ulps.py DUMP replay_x replay_y
import struct, sys
from fractions import Fraction as F
def h2f(h): return struct.unpack('>f', bytes.fromhex(h))[0]
def f2h(x): return struct.pack('>f', x).hex()
def r32(q):  # round an exact rational to f32 (nearest even) via double-safe path
    x = float(q)  # may round once; check neighbours exactly
    import numpy as np
    c = np.float32(x)
    best = None
    for cand in [np.nextafter(c, np.float32(-np.inf)), c, np.nextafter(c, np.float32(np.inf))]:
        d = abs(F(float(cand)) - q)
        if best is None or d < best[0] or (d == best[0] and (struct.unpack('>I', struct.pack('>f', float(cand)))[0] & 1) == 0):
            best = (d, float(cand))
    return best[1]
def mul(a,b): return r32(F(a)*F(b))
def fma(a,b,c): return r32(F(a)*F(b)+F(c))
lines = [l.split() for l in open(sys.argv[1]) if l.startswith('T3191 j')]
J = {}
for t in lines:
    name = t[1]; i = t.index('M'); M = [h2f(x) for x in t[i+1:i+13]]
    T = [h2f(x) for x in t[t.index('T')+1:t.index('T')+4]]
    J[name] = dict(M=[M[0:4],M[4:8],M[8:12]], T=T)
p = J['j101']['M']; T = J['j100']['T']
def trans(p, T):
    out = []
    for i in range(3):
        t = mul(T[0], p[i][0]); t = fma(T[1], p[i][1], t); t = fma(T[2], p[i][2], t); t = fma(1.0, p[i][3], t)
        out.append(t)
    return out
got = trans(p, T)
print('recomputed j100 world T', [f2h(v) for v in got], 'dumped', [f2h(J['j100']['M'][i][3]) for i in range(3)])
import numpy as np
def nb(x, k):
    v = np.float32(x)
    for _ in range(abs(k)): v = np.nextafter(v, np.float32(np.inf if k > 0 else -np.inf))
    return float(v)
hd = [l.split() for l in open(sys.argv[1]) if l.startswith('T3191 pos')][0]
x1A70 = [h2f(hd[i]) for i in (5,6,7)]; scale_y = h2f(hd[9]); facing = h2f(hd[11])
rep_x, rep_y = float(np.float32(float(sys.argv[2]))), float(np.float32(float(sys.argv[3])))
def final(px, py):
    return fma(facing, mul(x1A70[2], scale_y), px), fma(x1A70[1], scale_y, py)
print('engine final', final(got[0], got[1]), 'replay', rep_x, rep_y)
okx = [k for k in range(-30,31) if final(nb(got[0],k), got[1])[0] == rep_x]
oky = [k for k in range(-30,31) if final(got[0], nb(got[1],k))[1] == rep_y]
print('pos.x ulp offsets giving replay x:', okx, ' pos.y:', oky)
import itertools
def check(p, T):
    w = trans(p, T); return final(w[0], w[1]) == (rep_x, rep_y)
R = range(-6, 7)
hits = [(a,b,c) for a,b,c in itertools.product(R,R,R) if check(p, [nb(T[0],a), nb(T[1],b), nb(T[2],c)])]
print('T perturbations matching:', sorted(hits, key=lambda h: sum(map(abs,h)))[:10])
import copy
hits = []
for i in range(2):
    for j in range(4):
        for k in range(-8, 9):
            if k == 0: continue
            q = copy.deepcopy(p); q[i][j] = nb(q[i][j], k)
            w = trans(q, T); fx, fy = final(w[0], w[1])
            if fx == rep_x or fy == rep_y: hits.append((i, j, k, fx == rep_x, fy == rep_y))
print('single parent-entry hits (row,col,ulps,x_ok,y_ok):', hits)
print('parent rows', [[f2h(v) for v in r] for r in p[:2]])
