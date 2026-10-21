#!/usr/bin/env python3
"""Group 6 round 4 C oracle: callers of rounds 1-3 (and of other groups' ported code), with
pointer arguments that may lead to further objects (gobj -> user_data, spline -> cv, a JObj tree).

  python rust/melee_modes/oracle/gen_oracle_r4.py <outdir>     (from the worktree root)

As in gen_oracle_ptr.py each function is cut verbatim and renamed o_<name>. New here:
- Every function a cut function calls that is DEFINED in a scanned .c (CALLEE_TUS) is cut too,
  transitively, renamed o_<name> with external linkage, and calls are renamed to match, so the
  oracle runs the game's own callee code (not the Rust port of it).
- Buffers: `b:<seed>:<size>:<sizeof>` as before. The first NARGS buffers are the pointer
  arguments in order; any further ones are objects reachable only through links.
- Links: `l:<from>:<off>:<to>` stores the address of buffer <to> (ffff = NULL) as a 64-bit
  pointer at byte <off> of buffer <from>, after the fill and the `f:` fixups.
- Hash: FNV-1a 64 over every buffer, where each 8-byte-aligned word that is an address inside
  some buffer k is replaced by 0x5EED000000000000 | k<<24 | offset (both sides), so stored
  pointers compare by target, not by address.
"""
import os, random, re, struct, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gen_oracle_ptr as gp  # noqa: E402  (chdirs to the worktree root, loads gen_types)

ci, K, T = gp.ci, gp.K, gp.T
NAT = gp.NAT

FUNCS = {
    "src/melee/gm/gmmain_lib.c": ["bitset_set", "bitset_clear", "bitset_test", "bitset_test_word"],
    "src/melee/gm/gm_1B03.c": ["gm_SetupHumanPlayer", "gm_SetupCpuPlayer"],
    "src/melee/gm/gm_1601.c": ["gm_MatchHasMultipleWinners"],
    "src/melee/gm/gmtrainingmode.c": ["gm_801B07E8_layer"],
    "src/melee/gm/gmvsmelee.c": ["gmVsMelee_UpdateKOCounts"],
    "src/melee/gm/gmresult.c": ["fn_801771C0"],
    "src/melee/if/textlib.c": ["DevText_SetCursorX"],
    "src/melee/lb/lb_00CE.c": ["lb_8000D148"],
    "src/melee/lb/lb_00F9.c": ["groundHeight"],
    "src/melee/lb/lb_020A.c": ["calc_acos"],
    "src/melee/lb/lbshadow.c": ["lbShadow_8000E9F0"],
    "src/melee/lb/lbaudio_ax.c": ["fn_800253D8", "fn_800256BC", "fn_80025B44", "fn_80025CBC", "fn_800259EC",
                                  "fn_80025A98", "lbAudioAx_800264E4"],
    "src/melee/lb/lbspdisplay.c": ["lb_800138CC", "lb_800138D8", "lb_8001204C"],
    "src/melee/mn/mndiagram3.c": ["mnDiagram3_GetPopupSpacing", "mnDiagram3_GetPopupY", "mnDiagram3_GetRowSpacing"],
    "src/melee/mn/mnname.c": ["mnName_TextWidth"],
    "src/melee/mn/mnvibration.c": ["mnVibration_GetCursorYSpacing"],
}
# TUs whose definitions may be cut as callees (besides FUNCS' own TUs).
CALLEE_TUS = ["src/melee/lb/lbtrigf.c", "src/melee/lb/lbtime.c"]
EXTRA = {"src/melee/lb/lbaudio_ax.c": ["#define GET_SOUND(x) ((lbAudioAx_UserData*) HSD_GObjGetUserData(x))"],
         "src/melee/lb/lbtrigf.c": ["#define BITWISE(f) (*(u32*) &f)", "#define SIGNED_BITWISE(f) ((s32) BITWISE(f))"]}

R = random.Random(0x726F756E6434)
NULL = 0xFFFF


def fl(x):
    return struct.unpack("<I", struct.pack("<f", x))[0]


def fdom(lo, hi, n, extra=()):
    return [fl(x) for x in extra] + [fl(R.uniform(lo, hi)) for _ in range(n)]


# Scalar domains (None = default for the kind).
DOMAIN = {
    "bitset_set": [None, list(range(0, 4096))],
    "bitset_clear": [None, list(range(0, 4096))],
    "bitset_test": [None, list(range(0, 4096))],
    "bitset_test_word": [None, list(range(0, 4096))],
    "DevText_SetCursorX": [None, list(range(-300, 300, 7)) + [-0x80000000, 0x7FFFFFFF]],
    "calc_acos": [fdom(-1.2, 1.2, 300, [0.0, -0.0, 1.0, -1.0, 0.5, -0.5, 1.0000001, 0.9999999])],
    "lbShadow_8000E9F0": [None, None, fdom(-0.2, 1.2, 200, [0.0, -0.0, 1.0, 0.5, 1e-7, 0.9999999, -1e-7, 1.0000001])],
    "lb_800138D8": [None, list(range(-128, 128))],
    "mnDiagram3_GetPopupY": [None, fdom(-60, 60, 100, [0.0, 1.0, -1.0, 7.5]), list(range(256))],
    "mnName_TextWidth": [fdom(-500, 500, 100, [0.0]), None],
    "mnVibration_GetCursorYSpacing": [fdom(-500, 500, 100, [0.0]), None],
}
NULLABLE = {"gm_801B07E8_layer": 1, "fn_800253D8": 0, "fn_800256BC": 0, "fn_80025B44": 0, "fn_80025CBC": 0,
            "fn_800259EC": 0, "fn_80025A98": 0, "lbAudioAx_800264E4": 0, "lb_8001204C": 0}
BUFN = {}
# Pointer arguments the body dereferences only through a local copy (derefd cannot see it).
FORCEBUF = {"lb_8001204C": {"root", "result", "indices"}}
NROWS = {"lb_8000D148": 6000, "lb_8001204C": 600, "lbShadow_8000E9F0": 800, "fn_801771C0": 400}


def ptype(rec, path):
    """(offset, pointee type) of a pointer field `a.b` or array element `a#i` in record type rec."""
    t = rec.get_canonical()
    off = 0
    for p in path.split("."):
        i = None
        if "#" in p:
            p, i = p.split("#"); i = int(i)
        o = t.get_offset(p)
        assert o >= 0, (t.spelling, p)
        off += o // 8
        f = next(c for c in t.get_fields() if c.spelling == p)
        t = f.type.get_canonical()
        if i is not None:
            el = t.element_type.get_canonical()
            off += i * el.get_size()
            t = el
    assert t.kind == T.POINTER, (path, t.spelling)
    return off, t.get_pointee().get_canonical()


class Row:
    """One input row under construction: buffers (seed, size, sizeof, type), fixups, links."""

    def __init__(self):
        self.bufs, self.fix, self.links = [], [], []

    def buf(self, pt, n=1, null=False, minsz=512):
        sz = max(minsz, max(pt.get_size(), 0) * n)
        sz = (sz + 7) // 8 * 8
        seed = 0 if null else (R.getrandbits(64) | 1)
        self.bufs.append((seed, sz, pt))
        return len(self.bufs) - 1

    def link(self, frm, path, to=None, n=1, null_p=0.0):
        """Point field `path` of buffer frm at buffer `to`, or at a new buffer of the pointee."""
        if not self.bufs[frm][0]:
            return None
        off, pt = ptype(self.bufs[frm][2], path)
        if R.random() < null_p:
            self.links.append((frm, off, NULL))
            return None
        if to is None:
            to = self.buf(pt, n)
        self.links.append((frm, off, to))
        return to

    def field(self, k, path, val):
        if not self.bufs[k][0]:
            return
        off, s = gp.field_off(self.bufs[k][2], path)
        self.fix.append((k, off, s, val & ((1 << (8 * s)) - 1)))

    def tokens(self):
        t = [f"b:{s:x}:{z:x}:{max(pt.get_size(), 0):x}" for s, z, pt in self.bufs]
        t += ["f:%x:%x:%x:%x" % f for f in self.fix]
        t += ["l:%x:%x:%x" % l for l in self.links]
        return t


# ---- per-function row builders: (row, arg buffer indices) -> None (adds links/fixups)
def b_spline(r, a):
    sp = a[1]
    numcv = R.choice([2, 3, 4, 5, 8, 10])  # numcv 1 with u == 1 reads cv[-1] (type 0)
    r.field(sp, "type", R.choice([0, 1, 2, 3, 3, 4]))
    r.field(sp, "numcv", numcv)
    r.link(sp, "cv", n=3 * numcv + 4)


def b_gobj_ud(r, a, ud=None, null_p=0.0):
    """gobj->user_data -> a new buffer of type ud (the C reads it without a NULL check unless null_p)."""
    if a and r.bufs[a[0]][0]:
        r.link(a[0], "user_data", to=r.buf(ud), null_p=null_p)


def b_blur(r, a):
    b_gobj_ud(r, a, BLUR_T)


def b_audio(r, a):
    b_gobj_ud(r, a, UD_T, 0.15)
    if len(r.bufs) > 1 and r.bufs[-1][0]:
        u = len(r.bufs) - 1
        # x3C (float) == 1.0 sometimes; frames small; x44 a bool
        if R.random() < 0.4:
            r.fix.append((u, AUD["x3C"], 4, fl(1.0)))
        r.fix.append((u, AUD["current_frame"], 4, R.choice([-5, 0, 1, 3, 10, 30, 60, 200]) & 0xFFFFFFFF))
        r.fix.append((u, AUD["end_frame"], 4, R.choice([-3, 0, 1, 5, 30, 60, 100]) & 0xFFFFFFFF))
        r.fix.append((u, AUD["pan_left"], 4, R.choice([0, 0x40, 0x7F, 10, 90, -20]) & 0xFFFFFFFF))
        r.fix.append((u, AUD["pan_right"], 4, R.choice([0, 0x40, 0x7F, 10, 90, 300]) & 0xFFFFFFFF))
        r.fix.append((u, AUD["x44"], 4, R.choice([0, 0, 1, 2])))


def b_results(r, a):
    me = r.link(a[0], "x94")
    if me is not None and R.random() < 0.6:
        r.field(me, "is_teams", 1)


def b_diagram(r, a):
    d = a[-1]
    for i in (8, 9):
        r.link(d, f"jobjs#{i}")


def b_jtree(r, a):
    root, res, ind = a
    if r.bufs[root][0]:
        n = R.randrange(1, 9)
        nodes = [root] + [r.buf(JOBJ_T) for _ in range(n - 1)]
        parent = {nodes[0]: None}
        kids = {k: [] for k in nodes}
        for k in nodes[1:]:
            p = R.choice([x for x in nodes if x in parent])
            parent[k] = p
            kids[p].append(k)
        for k in nodes:
            r.field(k, "flags", R.choice([0, 0, 0x1000, 0x1234, 0xFFFFEFFF]))
            ch = kids[k]
            r.links.append((k, CHILD, ch[0] if ch else NULL))
            r.links.append((k, PARENT, parent[k] if parent[k] is not None else NULL))
        for p in nodes:
            ch = kids[p]
            for i, k in enumerate(ch):
                r.links.append((k, NEXT, ch[i + 1] if i + 1 < len(ch) else NULL))
        r.links.append((root, NEXT, NULL))
    if r.bufs[ind][0]:
        for i in range(16):
            r.fix.append((ind, 2 * i, 2, R.choice(list(range(0, 11)) + [0xFFFF])))


BUILD = {"lbShadow_8000E9F0": b_spline, "lb_800138CC": b_blur, "lb_800138D8": b_blur,
         "fn_801771C0": b_results, "mnDiagram3_GetPopupSpacing": b_diagram, "mnDiagram3_GetRowSpacing": b_diagram,
         "lb_8001204C": b_jtree}
for _n in FUNCS["src/melee/lb/lbaudio_ax.c"]:
    BUILD[_n] = b_audio
SCALAR_ROWS = {"lb_8001204C": [list(range(-1, 17))]}


def lb_8000D148_rows():
    out = []
    for _ in range(NROWS["lb_8000D148"]):
        mode = R.random()
        if mode < 0.3:
            v = [fl(float(R.randrange(-4, 5))) for _ in range(6)] + [fl(R.choice([0.5, 1.0, 2.0, 3.0]))]
        elif mode < 0.4:
            p = R.uniform(-5, 5)
            v = [fl(p), fl(p), fl(p + R.choice([0, 1e-4, 1e-3])), fl(p)] + [fl(R.uniform(-5, 5)) for _ in range(2)] + [fl(1.0)]
        elif mode < 0.55:
            v = [fl(R.uniform(-10, 10)) for _ in range(6)] + [fl(R.uniform(0, 6))]
        else:
            v = boundary_row()
        out.append(v)
    return out


# ---- boundary rows for lb_8000D148: put `threshold` where a fused and an unfused evaluation of
# one of the 5 fused sites disagree, so the vectors tell fused from unfused (a bool result
# otherwise almost never depends on the last bit).
def r32(x):
    return struct.unpack("<f", struct.pack("<f", x))[0]


def fmadds(a, b, c):
    return r32(a * b + c)  # f64 product (exact for f32 inputs) + f64 add, one f32 rounding


def nb(x, k):
    """x moved k f32 ulps."""
    u = fl(x)
    u = u + k if x >= 0 else u - k
    return struct.unpack("<f", struct.pack("<I", u & 0xFFFFFFFF))[0]


def boundary_row():
    import math
    while True:
        p = [r32(R.uniform(-8, 8)) for _ in range(4)]
        dx, dy = r32(p[2] - p[0]), r32(p[1] - p[3])
        d01 = fmadds(dx, dx, r32(dy * dy))
        if d01 < 1e-4:
            continue
        which = R.random()
        if which < 0.5:
            # near the line: threshold at |cross + inner| / |p0p1|, fused vs unfused
            t = R.uniform(0, 1)
            q = [r32(p[0] + t * (p[2] - p[0]) + R.uniform(-1, 1)), r32(p[1] + t * (p[3] - p[1]) + R.uniform(-1, 1))]
            cr = r32(p[1] * p[2])
            crf = fmadds(p[0], p[3], -cr)
            cru = r32(r32(p[0] * p[3]) - cr)
            vf = abs(r32(crf + fmadds(dx, q[0], r32(dy * q[1]))))
            vu = abs(r32(cru + r32(r32(dx * q[0]) + r32(dy * q[1]))))
            d = r32(math.sqrt(d01))
            th = R.choice([r32(vf / d), r32(vu / d)])
        else:
            # near an endpoint: search for a threshold where the fused and the unfused
            # |p_k - p2|^2 fall on different sides of (or onto) threshold^2
            k = R.choice([0, 2])
            for _ in range(200):
                q = [r32(p[k] + R.uniform(-0.3, 0.3)), r32(p[k + 1] + R.uniform(-0.3, 0.3))]
                a, b = r32(p[k] - q[0]), r32(p[k + 1] - q[1])
                df, du = fmadds(a, a, r32(b * b)), r32(r32(a * a) + r32(b * b))
                if df == du:
                    continue
                th0 = r32(math.sqrt(df))
                for j in range(-3, 4):
                    th = nb(th0, j)
                    ts = r32(th * th)
                    if (df < ts) != (du < ts) or (df > ts) != (du > ts):
                        return [fl(x) for x in p + q] + [fl(th)]
            continue
        th = nb(th, R.choice([-2, -1, 0, 0, 1, 2]))
        return [fl(x) for x in p + q] + [fl(th)]


def main(out):
    global UD_T, AUD, JOBJ_T, CHILD, NEXT, PARENT, BLUR_T
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    idx = ci.Index.create()
    tus = {}
    defs = {}  # name -> (tu_path, cursor)
    for tp in list(FUNCS) + CALLEE_TUS:
        tu = idx.parse(tp, args=NAT)
        tus[tp] = tu
        for c in tu.cursor.get_children():
            if (c.kind == K.FUNCTION_DECL and c.is_definition() and c.location.file
                    and Path(str(c.location.file)).resolve() == Path(tp).resolve()):
                if c.spelling not in defs or tp in FUNCS and c.spelling in FUNCS[tp]:
                    defs[c.spelling] = (tp, c)
    # transitive callee closure
    cut = {tp: list(ns) for tp, ns in FUNCS.items()}
    todo = [n for ns in FUNCS.values() for n in ns]
    seen = set(todo)
    calls_of = {}
    while todo:
        n = todo.pop()
        tp, c = defs[n]
        cs = []
        for d in c.walk_preorder():
            if d.kind == K.CALL_EXPR and d.referenced is not None and d.referenced.spelling in defs \
                    and d.referenced.spelling != n:
                cs.append(d.referenced.spelling)
        calls_of[n] = cs
        for x in cs:
            if x not in seen:
                seen.add(x)
                todo.append(x)
                cut.setdefault(defs[x][0], []).append(x)
    allnames = sorted(seen, key=len, reverse=True)
    ident = re.compile(r"(?<![A-Za-z0-9_.])(?<!->)(" + "|".join(allnames) + r")(?![A-Za-z0-9_])")

    # record types the row builders need
    ud_tu = tus["src/melee/lb/lbaudio_ax.c"]
    UD_T = next(c.type for c in ud_tu.cursor.walk_preorder()
                if c.kind == K.TYPEDEF_DECL and c.spelling == "lbAudioAx_UserData").get_canonical()
    AUD = {f.spelling: UD_T.get_offset(f.spelling) // 8 for f in UD_T.get_fields()}
    sp_tu = tus["src/melee/lb/lbspdisplay.c"]
    JOBJ_T = next(c.type for c in sp_tu.cursor.walk_preorder()
                  if c.kind == K.TYPEDEF_DECL and c.spelling == "HSD_JObj").get_canonical()
    BLUR_T = next(c.type for c in sp_tu.cursor.walk_preorder()
                  if c.kind == K.STRUCT_DECL and c.spelling == "CameraBlurData" and c.is_definition()).get_canonical()
    CHILD, NEXT, PARENT = (JOBJ_T.get_offset(f) // 8 for f in ("child", "next", "parent"))

    inputs, sigs, stems = [], {}, []
    for tp, names in cut.items():
        src = open(tp, "rb").read()
        tu = tus[tp]
        here = Path(tp).resolve()
        found = {n: defs[n][1] for n in names}
        stem = Path(tp).stem
        lines = src.decode("utf-8", "replace").splitlines()
        pre = []
        for ln in lines:
            s = ln.strip()
            if s.startswith('#include') and not s.rstrip('">').endswith('.h'):
                continue
            if s.startswith('#include "'):
                inc = s.split('"')[1]
                if (Path(tp).parent / inc).exists():
                    s = '#include "' + (Path(tp).parent / inc).as_posix() + '"'
                pre.append(s)
            elif s.startswith("#include") or (s.startswith("#define") and not s.endswith("\\")
                                            and "(" not in s.split()[1]):
                pre.append(s)
        body = ["/* generated by gen_oracle_r4.py from " + tp + "; do not edit */"] + pre + \
            EXTRA.get(tp, []) + ["#include <string.h>", ""]
        # types and file-scope variables the cut functions use that the .c itself defines
        decls = {}
        for n in names:
            for d in found[n].walk_preorder():
                r = d.referenced
                if d.kind in (K.TYPE_REF, K.DECL_REF_EXPR) and r is not None and r.location.file \
                        and Path(str(r.location.file)).resolve() == here \
                        and (r.kind in (K.TYPEDEF_DECL, K.STRUCT_DECL, K.ENUM_DECL)
                             or (r.kind == K.VAR_DECL and r.semantic_parent.kind == K.TRANSLATION_UNIT)):
                    decls[r.extent.start.offset] = r
        for o in sorted(decls):
            r = decls[o]
            body.append(src[r.extent.start.offset:r.extent.end.offset].decode("utf-8") + ";")
        # prototypes of every cut function this file calls
        called = sorted({x for n in names for x in calls_of.get(n, [])})
        for x in called:
            c = defs[x][1]
            def ct(t):
                return "int" if t.get_canonical().kind == T.ENUM else t.spelling
            a = ", ".join(ct(y.type) for y in c.get_arguments()) or "void"
            body.append(f"{ct(c.result_type)} o_{x}({a});")
        disp = []
        for n in names:
            c = found[n]
            text = src[c.extent.start.offset:c.extent.end.offset].decode("utf-8")
            head, rest = text.split("{", 1)
            i = [m.start() for m in re.finditer(r"(?<![A-Za-z0-9_])" + n + r"\s*\(", head)][-1]
            head = head[:i] + "o_" + n + head[i + len(n):]
            head = head.replace("static ", "").replace("inline ", "")
            rest = ident.sub(lambda m: "o_" + m.group(1), rest)
            body.append(head + "{" + rest + "\n")
            if n not in FUNCS.get(tp, []):
                continue
            bf = gp.bitfields(c)
            assert not bf, (n, bf)
            args = list(c.get_arguments())
            pnames = {a.spelling for a in args}
            dr = gp.derefd(c, pnames)
            # a pointer passed on to a callee (which dereferences it) is a buffer too
            for d in c.walk_preorder():
                if d.kind == K.CALL_EXPR:
                    for a in list(d.get_arguments()):
                        e = gp.strip(a)
                        if e.kind == K.DECL_REF_EXPR and e.spelling in pnames:
                            dr.add(e.spelling)
            dr |= FORCEBUF.get(n, set())
            kinds = []
            for a in args:
                t = a.type.get_canonical()
                if t.kind == T.POINTER and t.get_pointee().get_canonical().kind == T.FUNCTIONPROTO:
                    k = "ptr"
                else:
                    k = gp.kind_of(a.type)
                if k == "ptr" and a.spelling in dr:
                    k = "buf"
                kinds.append(k)
            ret = c.result_type.get_canonical()
            rk = "void" if ret.kind == T.VOID else gp.kind_of(c.result_type)
            sigs[n] = (kinds, rk)
            dl = [f'    if (!strcmp(name, "{n}")) {{']
            call, js, jb = [], 0, 0
            for k, (a, kd) in enumerate(zip(args, kinds)):
                ts = a.type.spelling
                if kd == "buf":
                    call.append(f"({ts}) b[{jb}]"); jb += 1
                elif kd == "ptr":
                    call.append(f"({ts}) (unsigned long long) s[{js}]"); js += 1
                elif kd == "f32":
                    dl.append(f"        float x{k} = bf(s[{js}]);"); call.append(f"x{k}"); js += 1
                else:
                    call.append(f"({ts}) ({gp.CT[kd]}) s[{js}]"); js += 1
            cs = f"o_{n}({', '.join(call)})"
            if rk == "void":
                dl.append(f"        {cs}; *out = 0; return 0;")
            elif rk == "f32":
                dl.append(f"        *out = fb({cs}); return 0;")
            else:
                dl.append(f"        *out = (unsigned long long) ({gp.CT[rk]}) {cs} & 0x{gp.MASK[rk]:x}ULL; return 0;")
            dl.append("    }")
            disp += dl
            # ---- rows
            ptypes = [a.type.get_pointee().get_canonical() if kd == "buf" else None for a, kd in zip(args, kinds)]
            dm = DOMAIN.get(n, []) or SCALAR_ROWS.get(n, [])
            if n == "lb_8001204C":
                dm = [None, None, None, list(range(-1, 17))]
            dm = dm + [None] * (len(args) - len(dm))
            pre_rows = lb_8000D148_rows() if n == "lb_8000D148" else None
            for ri in range(NROWS.get(n, 250)):
                row, sc, ab = Row(), [], []
                for kd, d, pt in zip(kinds, dm, ptypes):
                    if kd == "buf":
                        null = n in NULLABLE and len(ab) >= NULLABLE[n] and R.random() < 0.15
                        ab.append(row.buf(pt, BUFN.get(n, 1), null))
                for j, (kd, d) in enumerate(zip(kinds, dm)):
                    if kd != "buf":
                        if pre_rows is not None:
                            v = pre_rows[ri][len(sc)]
                        else:
                            v = R.choice(d if d is not None else gp.dom(kd))
                        sc.append(f"s:{v & gp.MASK[kd]:x}")
                if n in BUILD:
                    BUILD[n](row, ab)
                # scalars first, then buffers (the parser keeps them in separate lists)
                inputs.append(" ".join([n] + sc + row.tokens()))
        if disp:
            body.append("static float bf(unsigned long long b) { union { unsigned u; float f; } x; x.u = (unsigned) b; return x.f; }")
            body.append("static unsigned long long fb(float f) { union { unsigned u; float f; } x; x.f = f; return x.u; }")
            body.append(f"int run_{stem}(const char* name, const unsigned long long* s, void** b, unsigned long long* out)")
            body.append("{")
            body += disp
            body.append("    return -1;")
            body.append("}")
            stems.append(stem)
        (out / f"o_{stem}.c").write_text("\n".join(body) + "\n", encoding="utf-8")
    m = ["#include <stdio.h>", "#include <stdlib.h>", "#include <string.h>"]
    m += [f"int run_{s}(const char*, const unsigned long long*, void**, unsigned long long*);" for s in stems]
    m += [MAIN_C.replace("@@RUN@@", " && ".join(f"run_{s}(name, s, b, &r)" for s in stems))]
    (out / "main.c").write_text("\n".join(m) + "\n")
    (out / "inputs.txt").write_text("\n".join(inputs) + "\n")
    with open(out / "sigs.txt", "w") as f:
        for n, (a, r) in sigs.items():
            f.write(f"{n} {' '.join(a) or '-'} -> {r}\n")
    extra = sorted(seen - {n for ns in FUNCS.values() for n in ns})
    print("callees cut:", " ".join(extra))
    print(f"{len(sigs)} functions, {len(stems)} TUs, {len(inputs)} vectors -> {out}")


MAIN_C = gp.MAIN_C.replace(
    """            } else { fprintf(stderr, "bad token %s\\n", p); return 1; }""",
    """            } else if (sscanf(p, "l:%x:%x:%x%n", &a, &o, &z, &k) == 3) {
                void* t = z == 0xFFFF ? 0 : b[z];
                memcpy((unsigned char*) b[a] + o, &t, sizeof t);
            } else { fprintf(stderr, "bad token %s\\n", p); return 1; }""").replace(
    """        for (k = 0; k < nb; k++) {
            if (!b[k]) continue;
            for (i = 0; i < (int) sz[k]; i++) { h ^= ((unsigned char*) b[k])[i]; h *= 0x100000001B3ULL; }
            free(b[k]);
        }""",
    """        for (k = 0; k < nb; k++) {
            if (!b[k]) continue;
            for (i = 0; i + 8 <= (int) sz[k]; i += 8) {
                unsigned long long w; int q;
                memcpy(&w, (unsigned char*) b[k] + i, 8);
                for (q = 0; q < nb; q++)
                    if (b[q] && w >= (unsigned long long) b[q] && w < (unsigned long long) b[q] + sz[q]) {
                        w = 0x5EED000000000000ULL | ((unsigned long long) q << 24) | (w - (unsigned long long) b[q]);
                        memcpy((unsigned char*) b[k] + i, &w, 8);
                        break;
                    }
            }
            for (i = 0; i < (int) sz[k]; i++) { h ^= ((unsigned char*) b[k])[i]; h *= 0x100000001B3ULL; }
        }
        for (k = 0; k < nb; k++) free(b[k]);""")
MAIN_C = MAIN_C.replace("double __frsqrte(double x) { (void) x; abort(); return 0; }",
    r'''/* __frsqrte comes from pc/gameworld/gekko_fp.c (linked); MSL float.c's NaN/inf words */
int MSL_TrigF_80400770[] = { 0x7FFFFFFF };
int MSL_TrigF_80400774[] = { 0x7F800000 };
void __assert(const char* s, unsigned l, const char* e) { fprintf(stderr, "assert %s:%u %s\n", s, l, e); abort(); }''')
MAIN_C = MAIN_C.replace("void* b[8];", "void* b[32];").replace("unsigned sz[8];", "unsigned sz[32];")
assert "__frsqrte(double x)" not in MAIN_C and "b[32]" in MAIN_C
assert "l:%x" in MAIN_C and "5EED" in MAIN_C

if __name__ == "__main__":
    main(sys.argv[1])
