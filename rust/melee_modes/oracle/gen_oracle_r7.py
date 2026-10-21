#!/usr/bin/env python3
"""Group 6 round 7 (night 3 round 3) C oracle: callers of round 6's memzero (DevText_Erase,
fn_80168A6C, fn_80169574, gmResult_PcWinner, gm_80177724, gm_8017C984).

  python rust/melee_modes/oracle/gen_oracle_r7.py <outdir>     (from the worktree root)

Reuses gen_oracle_r4.py's machinery unchanged; only the function list, domains and row
builders differ. The C side runs the game's own memzero (and fn_801771C0) from the callee cut.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_oracle_r4 as g4  # noqa: E402

R = g4.R
R.seed(0x726F756E6437)
NULL = g4.NULL

g4.FUNCS = {
    "src/melee/if/textlib.c": ["DevText_Erase"],
    "src/melee/gm/gm_1601.c": ["fn_80168A6C"],
    "src/melee/gm/gm_16A2.c": ["fn_80169574"],
    "src/melee/gm/gmresult.c": ["gmResult_PcWinner"],
    "src/melee/gm/gmresultplayer.c": ["gm_80177724"],
    "src/melee/gm/gm_17C0.c": ["gm_8017C984"],
}
g4.CALLEE_TUS = ["src/melee/lb/lbaudio_ax.c", "src/melee/lb/lbspdisplay.c", "src/melee/lb/lb_00B0.c"]
g4.EXTRA = {}
g4.DOMAIN = {
    "fn_80168A6C": [None, None, list(range(0, 8))],
    "fn_80169574": [list(range(0, 512))],
}
g4.NULLABLE = {}
g4.FORCEBUF = {"fn_80168A6C": {"arg0", "arg1"}, "fn_80169574": {"buf"}, "gm_8017C984": {"arg0"}, "gmResult_PcWinner": {"me"}}
g4.NROWS = {"fn_80169574": 400, "fn_80168A6C": 400}
g4.SCALAR_ROWS = {}


def b_erase(r, a):
    w = R.choice([0, 1, 2, 5, 8, 16, R.randrange(17)])
    h = R.randrange(0, 257 // max(1, 2 * w) + 1) if w else R.randrange(256)
    while 2 * w * h > 512:
        h -= 1
    r.field(a[0], "w", w)
    r.field(a[0], "h", h)
    r.link(a[0], "buf")


def b_168a6c(r, a):
    vt = r.bufs[a[0]][2]
    arr = r.buf(vt)
    r.links.append((a[0], 0, arr))
    for i in range(8):
        r.links.append((arr, 8 * i, NULL if R.random() < 0.3 else r.buf(vt)))
    r.links.append((a[0], 8, NULL if R.random() < 0.3 else r.buf(vt)))


def b_pcwinner(r, a):
    if R.random() < 0.6:
        r.field(a[0], "is_teams", 1)


g4.BUILD = {"DevText_Erase": b_erase, "fn_80168A6C": b_168a6c, "gmResult_PcWinner": b_pcwinner}

if __name__ == "__main__":
    g4.main(sys.argv[1])
