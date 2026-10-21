#!/usr/bin/env python3
"""Group 6 round 8 (night 3 round 4) C oracle: leftover leaves over nested pointers
(lbColl_CopyHitCapsule, lbColl_8000ACFC, lb_8000B09C, lb_8000B134, lb_80011B74).

  python rust/melee_modes/oracle/gen_oracle_r8.py <outdir>     (from the worktree root)

Reuses gen_oracle_r4.py's machinery unchanged; only the function list, domains and row
builders differ.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_oracle_r4 as g4  # noqa: E402

R = g4.R
R.seed(0x726F756E6438)
NULL = g4.NULL

g4.FUNCS = {
    "src/melee/lb/lbcollision.c": ["lbColl_CopyHitCapsule", "lbColl_8000ACFC"],
    "src/melee/lb/lb_00B0.c": ["lb_8000B09C", "lb_8000B134"],
    "src/melee/lb/lbspdisplay.c": ["lb_80011B74"],
}
g4.CALLEE_TUS = ["src/melee/lb/lbaudio_ax.c"]
g4.EXTRA = {}
VICTIMS = [0, 0x1234, 0x8045_6780, 0xDEAD_BEEF_0000_1000]
g4.DOMAIN = {"lbColl_8000ACFC": [VICTIMS]}
g4.NULLABLE = {"lb_8000B09C": 0, "lb_8000B134": 0}
g4.FORCEBUF = {"lb_80011B74": {"dobj"}, "lbColl_8000ACFC": {"hitbox"},
               "lb_8000B09C": {"jobj"}, "lb_8000B134": {"jobj"}}
g4.NROWS = {"lbColl_8000ACFC": 400, "lb_8000B09C": 500, "lb_8000B134": 500, "lb_80011B74": 300,
            "lbColl_CopyHitCapsule": 250}
g4.SCALAR_ROWS = {}


def b_acfc(r, a):
    hb = a[0]
    t = r.bufs[hb][2]
    base = t.get_offset("victims_1") // 8
    for i in range(12):
        if R.random() < 0.15:
            r.fix.append((hb, base + 16 * i, 8, R.choice(VICTIMS)))


def b_jtree(r, a):
    root = a[0]
    if not r.bufs[root][0]:
        return
    jt = g4.JOBJ_T
    aobj_off = jt.get_offset("aobj") // 8
    aobj_t = next(f for f in jt.get_fields() if f.spelling == "aobj").type.get_canonical().get_pointee().get_canonical()
    n = R.randrange(1, 10)
    nodes = [root] + [r.buf(jt) for _ in range(n - 1)]
    parent = {nodes[0]: None}
    kids = {k: [] for k in nodes}
    for k in nodes[1:]:
        p = R.choice([x for x in nodes if x in parent])
        parent[k] = p
        kids[p].append(k)
    for k in nodes:
        r.field(k, "flags", R.choice([0, 0, 0x1000, 0x1234, 0xFFFFEFFF]))
        ch = kids[k]
        r.links.append((k, g4.CHILD, ch[0] if ch else NULL))
        r.links.append((k, g4.PARENT, parent[k] if parent[k] is not None else NULL))
        if R.random() < 0.25:
            ao = r.buf(aobj_t)
            r.fix.append((ao, 0, 4, R.choice([0, 1 << 30, 1 << 26, (1 << 30) | (1 << 26), 0x3FFFFFFF, 0xFFFFFFFF])))
            r.links.append((k, aobj_off, ao))
        else:
            r.links.append((k, aobj_off, NULL))
    for p in nodes:
        ch = kids[p]
        for i, k in enumerate(ch):
            r.links.append((k, g4.NEXT, ch[i + 1] if i + 1 < len(ch) else NULL))
    r.links.append((root, g4.NEXT, NULL))


def b_dobj(r, a):
    d = a[0]
    dt = r.bufs[d][2]
    nxt = dt.get_offset("next") // 8
    mo = dt.get_offset("mobj") // 8
    mt = next(f for f in dt.get_fields() if f.spelling == "mobj").type.get_canonical().get_pointee().get_canonical()
    n = R.randrange(1, 7)
    chain = [d] + [r.buf(dt) for _ in range(n - 1)]
    for i, k in enumerate(chain):
        r.links.append((k, nxt, chain[i + 1] if i + 1 < n else NULL))
        r.links.append((k, mo, r.buf(mt)))


g4.BUILD = {"lbColl_8000ACFC": b_acfc, "lb_8000B09C": b_jtree, "lb_8000B134": b_jtree,
            "lb_80011B74": b_dobj}

if __name__ == "__main__":
    g4.main(sys.argv[1])
