#!/usr/bin/env python3
"""Group 6 round 10 (night 3 round 6) C oracle: the lbrefract texel callbacks
(fn_80021F70, fn_80021FB4, fn_80021FF8, fn_8002206C), lbaudio_ax fn_80025E38, and gm leaves
over MatchEnd / HSD_SObj chains / BracketEntry (fn_80161004, gm_801688AC, fn_8017DE54,
fn_801859C8, gmTournament_SetTripleRightCoords).

  python rust/melee_modes/oracle/gen_oracle_r10.py <outdir>     (from the worktree root)

Reuses gen_oracle_r4.py's machinery unchanged; only the function list, domains and row
builders differ.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_oracle_r4 as g4  # noqa: E402

R = g4.R
R.seed(0x726F756E643130)
NULL = g4.NULL
K, T = g4.K, g4.T

REFRACT = ["fn_80021F70", "fn_80021FB4", "fn_80021FF8", "fn_8002206C"]
g4.FUNCS = {
    "src/melee/lb/lbrefract.c": REFRACT,
    "src/melee/lb/lbaudio_ax.c": ["fn_80025E38"],
    "src/melee/gm/gm_1601.c": ["fn_80161004", "gm_801688AC"],
    "src/melee/gm/gm_17C0.c": ["fn_8017DE54"],
    "src/melee/gm/gm_1832.c": ["fn_801859C8"],
    "src/melee/gm/gmtoulib.c": ["gmTournament_SetTripleRightCoords"],
}
g4.CALLEE_TUS = ["src/melee/lb/lbspdisplay.c"]
g4.EXTRA = {"src/melee/lb/lbaudio_ax.c": ["#define GET_SOUND(x) ((lbAudioAx_UserData*) HSD_GObjGetUserData(x))"]}
RC = [None, list(range(32)), list(range(32))]  # row/col inside an 8x8-tile window
g4.DOMAIN = {n: RC for n in REFRACT}
g4.DOMAIN["fn_8017DE54"] = [list(range(256))]
g4.NULLABLE = {"fn_80021FF8": 1, "fn_8002206C": 1, "fn_80025E38": 0}  # fn_801859C8 reads gobj->hsd_obj unchecked
g4.FORCEBUF = {}
g4.NROWS = {n: 600 for n in REFRACT}
g4.NROWS.update({"fn_80025E38": 600, "fn_80161004": 400, "gm_801688AC": 300, "fn_8017DE54": 600,
                 "fn_801859C8": 400})
g4.SCALAR_ROWS = {}

_SOBJ = None


def sobj_t():
    global _SOBJ
    if _SOBJ is None:
        tu = g4.ci.Index.create().parse("src/melee/gm/gm_1832.c", args=g4.NAT)
        _SOBJ = next(c.type for c in tu.cursor.walk_preorder()
                     if c.kind == K.TYPEDEF_DECL and c.spelling == "HSD_SObj").get_canonical()
    return _SOBJ


def elem_off(t, arr, i, field):
    """Byte offset and size of `arr[i].field` in record type t."""
    t = t.get_canonical()
    a = next(f for f in t.get_fields() if f.spelling == arr).type.get_canonical()
    el = a.element_type.get_canonical()
    return t.get_offset(arr) // 8 + i * el.get_size() + el.get_offset(field) // 8, \
        next(f for f in el.get_fields() if f.spelling == field).type.get_canonical().get_size()


def b_refract(r, a):
    d = a[0]
    r.field(d, "row_stride", R.choice([32, 64, 96, 128, 256]))
    pt = g4.ptype(r.bufs[d][2], "buffer")[1]
    r.link(d, "buffer", to=r.buf(pt, minsz=4096))


def b_match(r, a):
    m = a[0]
    t = r.bufs[m][2]
    r.field(m, "is_teams", R.choice([0, 1, 1, 2]))
    for i in range(5):
        o, s = elem_off(t, "team_standings", i, "active")
        r.fix.append((m, o, s, R.choice([0, 0, 1, 0xFF])))
        o, s = elem_off(t, "team_standings", i, "is_big_loser")
        r.fix.append((m, o, s, R.choice([0, 1, 2, 3, 0x7F, 0x80, 0xFF])))
    for i in range(6):
        o, s = elem_off(t, "player_standings", i, "pkind")
        r.fix.append((m, o, s, R.choice([0, 0, 1, 2, 3, 3, 0xFF])))
        o, s = elem_off(t, "player_standings", i, "is_big_loser")
        r.fix.append((m, o, s, R.choice([0, 1, 2, 3, 0x7F, 0x80, 0xFF])))


def b_ckinds(r, a):
    for i in range(3):
        r.fix.append((a[0], i, 1, R.choice([0x21, 0x21, 0, 0x20, 0x22, 0xFF, R.randrange(256)])))


def b_sobjs(r, a):
    g = a[0]
    if not r.bufs[g][0]:
        return
    st = sobj_t()
    if R.random() < 0.1:
        r.links.append((g, g4.ptype(r.bufs[g][2], "hsd_obj")[0], NULL))
        return
    n = R.randrange(1, 6)
    chain = [r.buf(st, minsz=0) for _ in range(n)]
    nxt = st.get_offset("next") // 8
    for i, s in enumerate(chain):
        r.links.append((s, nxt, chain[i + 1] if i + 1 < n else NULL))
        r.field(s, "x48", R.choice([0, 0, 1, 1, 2, 5, 0xFFFFFFFF]))
        r.field(s, "x40", R.choice([0, 1, 3, 0xFFFFFFFF, R.getrandbits(32)]))
    r.links.append((g, g4.ptype(r.bufs[g][2], "hsd_obj")[0], chain[0]))


g4.BUILD = {n: b_refract for n in REFRACT}
g4.BUILD.update({"fn_80025E38": g4.b_audio, "fn_80161004": b_match, "gm_801688AC": b_match,
                 "fn_8017DE54": b_ckinds, "fn_801859C8": b_sobjs})

if __name__ == "__main__":
    g4.main(sys.argv[1])
