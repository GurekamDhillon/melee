#!/usr/bin/env python3
"""Group 6 round 9 (night 3 round 5) C oracle: lb_80011C18 (caller of round 8's lb_80011B74),
fn_80021C18, fn_8018FBD8, and getters over records the .c files define (getStKind,
gm_801AB200_HasCheck, gm_801BC00C_GetCharacterKind, mnVibration_GetCursorRow,
lbSnap_GetSaveDataOffset).

  python rust/melee_modes/oracle/gen_oracle_r9.py <outdir>     (from the worktree root)

Reuses gen_oracle_r4.py's machinery unchanged; only the function list, domains and row
builders differ.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_oracle_r4 as g4  # noqa: E402

R = g4.R
R.seed(0x726F756E6439)
NULL = g4.NULL

g4.FUNCS = {
    "src/melee/lb/lbspdisplay.c": ["lb_80011C18"],
    "src/melee/lb/lb_0219.c": ["fn_80021C18"],
    "src/melee/lb/lbsnap.c": ["lbSnap_GetSaveDataOffset"],
    "src/melee/gm/gmtoulib.c": ["fn_8018FBD8"],
    "src/melee/gm/gm_1879.c": ["getStKind"],
    "src/melee/gm/gmstaffroll.c": ["gm_801AB200_HasCheck"],
    "src/melee/gm/gmevent.c": ["gm_801BC00C_GetCharacterKind"],
    "src/melee/mn/mnvibration.c": ["mnVibration_GetCursorRow"],
}
g4.CALLEE_TUS = ["src/melee/lb/lbaudio_ax.c"]
# StaffEntryData uses this typedef, which the cut does not reach (it copies direct references only)
g4.EXTRA = {"src/melee/gm/gmstaffroll.c": ["typedef bool (*StaffEntryCheck)(s16);"]}
g4.DOMAIN = {}
g4.NULLABLE = {}
g4.FORCEBUF = {"lb_80011C18": {"jobj"}, "fn_8018FBD8": {"arg0"}}
g4.NROWS = {"lb_80011C18": 500}
g4.SCALAR_ROWS = {}


def b_jtree_dobj(r, a):
    """A random JObj tree (child/next/parent), each joint's u.dobj a DObj chain (every DObj with
    a live MObj) or NULL; flags include JOBJ_PTCL (1<<5) and JOBJ_SPLINE (1<<14)."""
    root = a[0]
    jt = g4.JOBJ_T
    u_off = jt.get_offset("u") // 8
    ut = next(f for f in jt.get_fields() if f.spelling == "u").type.get_canonical()
    dt = next(f for f in ut.get_fields() if f.spelling == "dobj").type.get_canonical().get_pointee().get_canonical()
    nxt = dt.get_offset("next") // 8
    mo = dt.get_offset("mobj") // 8
    mt = next(f for f in dt.get_fields() if f.spelling == "mobj").type.get_canonical().get_pointee().get_canonical()
    n = R.randrange(1, 6)  # main.c holds 32 buffers: 5 joints x (1 + 2 DObjs + 2 MObjs) + 1
    nodes = [root] + [r.buf(jt, minsz=0) for _ in range(n - 1)]
    parent = {nodes[0]: None}
    kids = {k: [] for k in nodes}
    for k in nodes[1:]:
        p = R.choice([x for x in nodes if x in parent])
        parent[k] = p
        kids[p].append(k)
    for k in nodes:
        r.field(k, "flags", R.choice([0, 0, 0x20, 0x4000, 0x4020, 0x1234, 0xFFFFBFDF]))
        ch = kids[k]
        r.links.append((k, g4.CHILD, ch[0] if ch else NULL))
        r.links.append((k, g4.PARENT, parent[k] if parent[k] is not None else NULL))
        if R.random() < 0.8:
            m = R.randrange(1, 3)
            chain = [r.buf(dt, minsz=0) for _ in range(m)]
            for i, d in enumerate(chain):
                r.links.append((d, nxt, chain[i + 1] if i + 1 < m else NULL))
                r.links.append((d, mo, r.buf(mt, minsz=0)))
            r.links.append((k, u_off, chain[0]))
        else:
            r.links.append((k, u_off, NULL))
    for p in nodes:
        ch = kids[p]
        for i, k in enumerate(ch):
            r.links.append((k, g4.NEXT, ch[i + 1] if i + 1 < len(ch) else NULL))
    r.links.append((root, g4.NEXT, NULL))


g4.BUILD = {"lb_80011C18": b_jtree_dobj}

if __name__ == "__main__":
    g4.main(sys.argv[1])
