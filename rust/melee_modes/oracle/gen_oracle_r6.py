#!/usr/bin/env python3
"""Group 6 round 6 (night 3 round 2) C oracle: callers of round 5's DevText_StrLen
(textlib_1.c un_80302EA4 / un_80302FFC_maxlen) and leaf functions the earlier rounds left
(my_fmodf, gmMainLib_AdjustNameTags, lb_8000B074, memzero, un_802FFCD0, fn_80194658_get_value).

  python rust/melee_modes/oracle/gen_oracle_r6.py <outdir>     (from the worktree root)

Reuses gen_oracle_r4.py's machinery unchanged (verbatim cut, callee closure, buffers, links,
pointer-normalised hash); only the function list, domains and row builders differ.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gen_oracle_r4 as g4  # noqa: E402

R = g4.R
R.seed(0x726F756E6436)
NULL = g4.NULL

g4.FUNCS = {
    "src/melee/if/textlib_1.c": ["un_80302EA4", "un_80302FFC_maxlen"],
    "src/melee/lb/lbrefract.c": ["my_fmodf"],
    "src/melee/gm/gmmain_lib.c": ["gmMainLib_AdjustNameTags"],
    "src/melee/lb/lb_00B0.c": ["lb_8000B074", "memzero"],
    "src/melee/if/soundtest.c": ["un_802FFCD0"],
    "src/melee/gm/gmtou_0.c": ["fn_80194658_get_value"],
}
# main() reads lbAudioAx_UserData / HSD_JObj / CameraBlurData from these; textdraw.c holds the
# callee DevText_StrLen.
g4.CALLEE_TUS = ["src/melee/lb/lbaudio_ax.c", "src/melee/lb/lbspdisplay.c", "src/melee/if/textdraw.c"]
g4.EXTRA = {}
# -fno-builtin leaves my_fmodf's fabsf unresolved; the game's fabsf is the fabs instruction
# (sign bit cleared), defined here in the dispatcher.
g4.MAIN_C = g4.MAIN_C + ("\nfloat fabsf(float x) { union { float f; unsigned u; } v; "
                         "v.f = x; v.u &= 0x7FFFFFFFu; return v.f; }\n")
_fm = ([0.0, -0.0, 1.0, -1.0, 2.5, -2.5, 7.0, 1e-3, -1e-3, 0.1, 3.0, 1000.0]
       + [round(R.uniform(-60, 60), R.choice([0, 1, 3, 6])) for _ in range(40)])
_fb = [1.0, -1.0, 0.5, -0.5, 2.5, 3.0, 0.1, -0.1, 7.0, 1e-3, 1000.0, 2000.0, -0.0]
g4.DOMAIN = {
    # finite, b != 0 unless a == 0 too would give 0/0: keep |a / b| far inside s64 (gwtool's s64
    # conversion is not modelled by lower.py) and b nonzero
    "my_fmodf": [g4.fdom(-1000, 1000, 300, _fm), g4.fdom(-50, 50, 300, _fb)],
    "gmMainLib_AdjustNameTags": [None, list(range(0, 122)) + [0xFF, 0x80]],
    "memzero": [None, list(range(0, 513))],
    "un_802FFCD0": [list(range(-3, 128))],
}
g4.NULLABLE = {}
g4.FORCEBUF = {"memzero": {"mem"}, "un_802FFCD0": {"data"}, "fn_80194658_get_value": {"ptr"}}
g4.NROWS = {"un_80302EA4": 400, "un_80302FFC_maxlen": 300, "my_fmodf": 1500, "memzero": 300}
g4.SCALAR_ROWS = {}

ENTRY_SZ = 0x38  # native sizeof(struct un_80304138_objalloc_t_x8)


def string(r):
    """A u8 buffer holding a NUL-terminated string of length 0..~40 (or up to the end)."""
    k = r.buf(U8_T)
    sz = r.bufs[k][1]
    r.fix.append((k, sz - 1, 1, 0))
    if R.random() < 0.85:
        r.fix.append((k, R.randrange(0, 40), 1, 0))
    return k


def b_entries(r, a):
    """arg buffer: n entries then one with x0 == 9. x8 a string (sometimes NULL); x0 == 2
    entries get a char* array of (int) x18 strings."""
    k = a[0]
    n = R.randrange(0, 8)
    sz = max(512, (n + 1) * ENTRY_SZ)
    seed, _, pt = r.bufs[k]
    r.bufs[k] = (seed, (sz + 7) // 8 * 8, pt)
    for j in range(n + 1):
        base = j * ENTRY_SZ
        x0 = 9 if j == n else R.choice([-1, 0, 1, 2, 2, 2, 3, 4, 5, 6, 7, 8, 10])
        r.fix.append((k, base + EO["x0"], 4, x0 & 0xFFFFFFFF))
        if j == n:
            continue
        r.links.append((k, base + EO["x8"], NULL if R.random() < 0.1 else string(r)))
        if x0 == 2:
            cnt = R.choice([0.0, 1.0, 2.0, 3.0, 2.5, 1.9, -1.0, 0.5])
            r.fix.append((k, base + EO["x18"], 4, g4.fl(cnt)))
            arr = r.buf(PCHAR_T, 4)
            r.links.append((k, base + EO["xC"], arr))
            for i in range(max(0, int(cnt))):
                r.links.append((arr, 8 * i, NULL if R.random() < 0.1 else string(r)))


def b_jobj(r, a):
    if R.random() < 0.25:
        r.links.append((a[0], JO_AOBJ, NULL))
    else:
        ao = r.buf(AOBJ_T)
        r.links.append((a[0], JO_AOBJ, ao))
        if R.random() < 0.5:
            r.fix.append((ao, 0, 4, R.choice([1 << 30, 0, 0x40000001, 0xBFFFFFFF, 0xFFFFFFFF])))


def b_tags(r, a):
    vmd = a[0]
    for i in range(6):
        off, _ = g4.gp.field_off(VMD_T, "start")
        off += PL_OFF + i * PL_SZ + PL_TAG
        r.fix.append((vmd, off, 1, R.choice([0, 1, 5, 50, 60, 61, 119, 120, 121, 200, 255, R.randrange(256)])))


g4.BUILD = {"un_80302EA4": b_entries, "un_80302FFC_maxlen": b_entries, "lb_8000B074": b_jobj,
            "gmMainLib_AdjustNameTags": b_tags}

_orig_main = g4.main


def main(out):
    global U8_T, PCHAR_T, EO, AOBJ_T, JO_AOBJ, VMD_T, PL_OFF, PL_SZ, PL_TAG
    ci, K, T = g4.ci, g4.K, g4.T
    idx = ci.Index.create()
    tu = idx.parse("src/melee/if/textlib_1.c", args=g4.NAT)
    ET = next(c.type for c in tu.cursor.walk_preorder()
              if c.kind == K.STRUCT_DECL and c.spelling == "un_80304138_objalloc_t_x8"
              and c.is_definition()).get_canonical()
    assert ET.get_size() == ENTRY_SZ, ET.get_size()
    EO = {f.spelling: ET.get_offset(f.spelling) // 8 for f in ET.get_fields()}
    PCHAR_T = next(f.type for f in ET.get_fields() if f.spelling == "x8").get_canonical()
    U8_T = PCHAR_T.get_pointee().get_canonical()
    tu2 = idx.parse("src/melee/lb/lb_00B0.c", args=g4.NAT)
    JT = next(c.type for c in tu2.cursor.walk_preorder()
              if c.kind == K.TYPEDEF_DECL and c.spelling == "HSD_JObj").get_canonical()
    JO_AOBJ = JT.get_offset("aobj") // 8
    AOBJ_T = next(f.type for f in JT.get_fields() if f.spelling == "aobj").get_canonical().get_pointee().get_canonical()
    tu3 = idx.parse("src/melee/gm/gmmain_lib.c", args=g4.NAT)
    VMD_T = next(c.type for c in tu3.cursor.walk_preorder()
                 if c.kind == K.TYPEDEF_DECL and c.spelling == "VsModeData").get_canonical()
    SM = next(f.type for f in VMD_T.get_fields() if f.spelling == "start").get_canonical()
    PL_OFF = SM.get_offset("players") // 8
    PT = next(f.type for f in SM.get_fields() if f.spelling == "players").get_canonical().element_type.get_canonical()
    PL_SZ, PL_TAG = PT.get_size(), PT.get_offset("nametag") // 8
    _orig_main(out)


if __name__ == "__main__":
    main(sys.argv[1])
