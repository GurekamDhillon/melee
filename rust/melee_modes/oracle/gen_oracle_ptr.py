#!/usr/bin/env python3
"""Group 6 round 3 C oracle: the pointer-taking leaf functions (no calls, no globals).

  python rust/melee_modes/oracle/gen_oracle_ptr.py <outdir>     (from the worktree root)

Like gen_oracle.py, each function is cut verbatim out of its TU (libclang extents) and renamed
o_<name>. The difference is the arguments: a pointer argument the body dereferences gets a
BUFFER, i.e. a block of memory filled from a seed by a generator both sides implement
(`fill` in main.c and in tests/oracle_ptr.rs). The C side reads and writes it through the
game's own struct types. The Rust side views the same bytes as the melee_types records, which
are laid out natively exactly like clang's x86_64 layout (gen_types.py checks every offset).
After the call both sides print the return value and an FNV-1a 64 hash of every buffer.
A pointer argument that is only stored or compared (never dereferenced) is passed as a plain
64-bit value.

Input line tokens: `name` then, in argument order, `s:<hex>` (scalar or plain pointer value)
or `b:<seed>:<size>:<sizeof pointee>` (buffer; seed 0 = NULL), then any `f:<buf>:<off>:<size>:<hex>` fixups
(store a little-endian value into a buffer after filling it, to keep loops and indices
inside the C's defined behaviour). Output: `<input> = <ret> <hash>`.

Functions that touch a C bitfield are refused: melee_types keeps bitfield bytes in GameCube
bit order, so their byte image differs from native C (they need a field-level comparison).
"""
import os, random, re, struct, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g  # noqa: E402
import native_hdr  # noqa: E402  (shadow headers with the widened native slots, see there)

ci, K, T = g.ci, g.K, g.T
# The oracle compiles the cut C for x86_64-pc-windows-gnu with -mno-ms-bitfields (build_ptr.sh):
# the same LLP64 data layout as the msvc target, but Itanium bitfield packing, which puts the
# fields after a bitfield run at the GameCube offsets melee_types uses (MSVC packing moves
# them: StartMeleeRules.x6 is at 8 under msvc, 6 on the GC, in melee_types and here).
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]
NAT = native_hdr.force_include_flags() + NAT

FUNCS = {
    "src/melee/gm/gmcamera.c": ["gmCamera_801A2BF0_get_translate_x"],
    "src/melee/gm/gmclassic.c": ["gmClassic_GetMatchupCount"],
    "src/melee/gm/gmgiant.c": ["fn_801B9060"],
    "src/melee/gm/gminvisible.c": ["initVsPlayer"],
    "src/melee/gm/gmlightning.c": ["fn_801BA7AC"],
    "src/melee/gm/gmmain_lib.c": ["gmMainLib_AdjustNameTag", "gmMainLib_ClearNameTag", "gmMainLib_8015EF30"],
    "src/melee/gm/gmmultiman.c": ["gmMultiman_InitTimedRules"],
    "src/melee/gm/gmregcommon.c": ["gm_8017BE8C"],
    "src/melee/gm/gmregenddisp.c": ["gm_801A9094_get_entry"],
    "src/melee/gm/gmresult.c": ["gmResultFindNth", "matchWasSkipped"],
    "src/melee/gm/gmsinglebutton.c": ["fn_801BA1B4"],
    "src/melee/gm/gmslomo.c": ["fn_801BA5B4"],
    "src/melee/gm/gmstaffroll.c": ["gm_801AB200_SetAppSRT"],
    "src/melee/gm/gmsupersudden.c": ["fn_801B8C5C"],
    "src/melee/gm/gmtiny.c": ["fn_801B8E50"],
    "src/melee/gm/gmtoulib.c": ["gmTournament_SetBracketByes", "fn_8018B090_inline4", "fn_8018F62C"],
    "src/melee/gm/gmtou_0.c": ["fn_80191240_save_root_jobj", "fn_80191FD4_is_selected", "fn_80195CCC_IsUniqueEntry"],
    "src/melee/gm/gmtou_1.c": ["fn_80196564", "fn_80196594"],
    "src/melee/gm/gmvs.c": ["getPort"],
    "src/melee/gm/gmvsmelee.c": ["gmVsMelee_WasAnyPlayerHuman"],
    "src/melee/gm/gm_1601.c": ["fn_80161C90_count_players", "is_character_unlocked", "fn_80165418", "fn_801654A0",
                               "fn_80165548", "fn_801656A8", "fn_80166CBC", "gm_80166CCC", "fn_80167194",
                               "gm_801677C0", "gm_80168940"],
    "src/melee/gm/gm_1798.c": ["get_big_loser"],
    "src/melee/gm/gm_17C0.c": ["gm_8017CE34_CountEnemies", "gm_8017CE34_GetCpuLevel", "gm_8017DB58",
                               "gm_8017DB6C", "gm_8017DB78"],
    "src/melee/gm/gm_181A.c": ["fn_80181E18_ComputeRemaining100", "gm_80182578_GetIndexFromPointer"],
    "src/melee/gm/gm_1884.c": ["resetText"],
    "src/melee/gm/gm_1A3F.c": ["firstState"],
    "src/melee/gm/gm_1B03.c": ["player_standings_inline", "gm_801B0474_inline", "gm_801B05F4", "gm_801B0730",
                               "gm_801B07B4", "gm_801B07E8"],
    "src/melee/if/ifprize.c": ["un_802FE918_update_x3"],
    "src/melee/if/ifstatus.c": ["ifStatus_GetStoredTranslationX"],
    "src/melee/if/ifstock.c": ["ifStock_802F8298_step_anim", "ifStock_ClearGObjSlots"],
    "src/melee/if/soundtest.c": ["un_802FF884"],
    "src/melee/if/textlib.c": ["DevText_SetCursorXY", "DevText_HideCursor", "DevText_80302AC0",
                               "DevText_ShowBackground", "DevText_HideBackground", "DevText_ShowText",
                               "DevText_HideText", "DevText_SetScale", "DevText_SetXY", "DevText_SetBGColor",
                               "un_80302DF8"],
    "src/melee/if/textlib_1.c": ["un_80303FD4_first", "un_80303FD4_count", "un_80304334"],
    "src/melee/lb/lbanim.c": ["lbAnim_8001E8F8"],
    "src/melee/lb/lbcardnew.c": ["readCardFileSize"],
    "src/melee/lb/lbcollision.c": ["lbColl_80005EBC", "lbColl_80005FC0", "end", "lbColl_GetY", "lbColl_DifferenceY",
                                   "sqrDistance", "lbColl_80008428", "lbColl_80008434", "lbColl_80008440",
                                   "lbColl_800089B8", "lbColl_80008A5C", "lbColl_80008D30"],
    "src/melee/lb/lbcommand.c": ["Command_00", "Command_06", "Command_08"],
    "src/melee/lb/lbshadow.c": ["splGetCardinalTangent", "splGetBSplineTangent", "splGetBezierTangent"],
    "src/melee/lb/lbsnap.c": ["lbSnap_ClearText"],
    "src/melee/lb/lbspdisplay.c": ["checkJObjFlags"],
    "src/melee/lb/lbvector.c": ["lbVector_Mirror"],
    "src/melee/lb/lb_00B0.c": ["lb_8000CE30", "lb_8000CE40"],
    "src/melee/lb/lb_00F9.c": ["checkJObjFlags", "lb_800103D8"],
    "src/melee/lb/lb_013B.c": ["lb_80013BB0"],
    "src/melee/lb/lb_0146.c": ["lb_80014638"],
    "src/melee/mn/mncharsel.c": ["loadStickValue", "mnCharSel_MexPackRows"],
    "src/melee/mn/mndiagram.c": ["mnDiagram_TextSetPos"],
    "src/melee/mn/mndiagram3.c": ["mnDiagram3_GetRowStat"],
    "src/melee/mn/mngallery.c": ["float_order_helper"],
    "src/melee/mn/mninfo.c": ["mnInfo_80251AFC_inline_3", "mnInfo_80252720"],
    "src/melee/mn/mninfobonus.c": ["textSize", "mnInfoBonus_80252ADC_inline"],
    "src/melee/mn/mnmain.c": ["GetAnimStartFrame", "GetAnimEndFrame"],
    "src/melee/mn/mnname.c": ["readNameTerminator", "checkStringRest", "mnName_80238C34_inline"],
    "src/melee/mn/mnnamenew.c": ["mnNameNew_8023BAA8", "GlyphVariantCount"],
    "src/melee/mn/mnsound.c": ["getVolumePosition"],
    "src/melee/mn/mnstagesw.c": ["mnStageSw_CountEnabled"],
    "src/melee/mn/mn_22EC.c": ["mn_8022F410", "mn_8022F470"],
    "src/melee/ty/tydisplay.c": ["tyDisplay_SetGridSize"],
    "src/melee/ty/tyfigupon.c": ["tyFigupon_StoreDigits", "tyFigupon_GetBetCount"],
}
# Same name defined in two TUs: the second gets this vector name (and C symbol o_<key>).
DUPKEY = {("src/melee/lb/lb_00F9.c", "checkJObjFlags"): "checkJObjFlags_lb_00F9"}

# Macros the depth-0 one-line #define scan cannot pick up.
EXTRA = {"src/melee/lb/lb_00B0.c": ["#include <sysdolphin/baselib/dobj.h>"],
         "src/melee/mn/mn_22EC.c": ["#define MY_MAX(x, y) ((x) < (y) ? (y) : (x))",
                                    "#define MY_MIN(x, y) ((x) > (y) ? (y) : (x))"]}

# Functions whose pointer arguments may be NULL (checked in the body): seed 0 in some rows
# for every buffer argument from the given index on.
NULLABLE = {"lbAnim_8001E8F8": 0, "gm_801B0730": 1, "gm_801B07E8": 1, "lb_8000CE30": 0, "lb_8000CE40": 0}  # first nullable buffer

# Scalar domains (index arguments stay inside their arrays; see group-modes.md round 3).
DOMAIN = {
    "gm_801A9094_get_entry": [None, list(range(0, 64))],
    "gmResultFindNth": [None, list(range(-2, 300))],
    "gmTournament_SetBracketByes": [None, list(range(-1, 10))],
    "fn_80191FD4_is_selected": [list(range(0, 40)), list(range(0, 4)), None],
    "fn_80195CCC_IsUniqueEntry": [None, list(range(0, 5)), list(range(-1, 5)), list(range(-2, 40))],
    "getPort": [None, list(range(-2, 8))],
    "is_character_unlocked": [list(range(0, 64)), None],
    "fn_80165548": [None, None, list(range(-1, 6))],
    "fn_801656A8": [None, list(range(0, 6))],
    "fn_80166CBC": [None, list(range(0, 6))],
    "get_big_loser": [list(range(0, 6)), None],
    "gm_8017DB6C": [None, list(range(0, 32))],
    "gm_8017DB78": [None, list(range(0, 32))],
    "player_standings_inline": [None, None, list(range(0, 4)), [0, 1, 2, 0xFFFFFFFF], [-1, 0, 1, 5]],
    "gm_801B0474_inline": [None, list(range(0, 6))],
    "gm_801B05F4": [None, list(range(-3, 8))],
    "ifStatus_GetStoredTranslationX": [None, list(range(0, 4))],
    "ifStock_ClearGObjSlots": [None, list(range(-2, 40))],
    "DevText_SetCursorXY": [None, list(range(-300, 300, 7)) + [-0x80000000, 0x7FFFFFFF], list(range(-300, 300, 7))],
    "DevText_SetXY": [None, list(range(-40, 40)) + [0x12345, -0x80000000], list(range(-40, 40)) + [0x7FFF8, 0x7FFFFFFF]],
    "mnInfo_80251AFC_inline_3": [None, list(range(0, 64))],
    "mnCharSel_MexPackRows": [None, list(range(-2, 12)), list(range(-3, 80)), None],
    "tyFigupon_StoreDigits": [None, None],
}

# Buffers are max(512, sizeof) bytes; these hold arrays of N elements of the pointee.
BUFN = {"gmTournament_SetBracketByes": 48, "firstState": 30, "un_80303FD4_first": 40, "un_80303FD4_count": 40,
        "gmClassic_GetMatchupCount": 40, "gm_8017DB6C": 32, "gm_8017DB78": 32}

R = random.Random(0x726F756E6433)
MASK = {"s8": 0xFF, "u8": 0xFF, "s16": 0xFFFF, "u16": 0xFFFF, "s32": 0xFFFFFFFF, "u32": 0xFFFFFFFF,
        "s64": (1 << 64) - 1, "u64": (1 << 64) - 1, "f32": 0xFFFFFFFF, "color": 0xFFFFFFFF, "ptr": (1 << 64) - 1}


def f32(x):
    return struct.unpack("<I", struct.pack("<f", x))[0]


def dom(kind):
    if kind == "u8":
        return list(range(256))
    if kind == "s8":
        return list(range(-128, 128))
    if kind == "u16":
        return [0, 1, 2, 0xFF, 0x100, 0x1FF, 0x7FFF, 0x8000, 0xFFFF] + [R.randrange(65536) for _ in range(40)]
    if kind in ("s32", "u32"):
        v = [0, 1, -1, 2, -2, 5, 9, 10, 99, 100, 101, 0xFFFF, 0x7FFFFFFF, -0x80000000] + list(range(-20, 60)) \
            + [R.randrange(-0x80000000, 0x80000000) for _ in range(60)]
        return [x & 0xFFFFFFFF for x in v] if kind == "u32" else v
    if kind == "f32":
        return [f32(x) for x in [0.0, -0.0, 1.0, -1.0, 0.5, 2.0, 1e-3, 100.0, -100.0]] + \
            [f32(R.uniform(-3, 3)) for _ in range(80)]
    if kind == "color":
        return [R.getrandbits(32) for _ in range(40)]
    if kind == "ptr":
        return [0, 1, 2, 0x100000001, 0x300000002, 0x1000, R.getrandbits(64), R.getrandbits(64)]
    raise SystemExit(kind)


def kind_of(t):
    t = t.get_canonical()
    m = {T.CHAR_S: "s8", T.SCHAR: "s8", T.UCHAR: "u8", T.CHAR_U: "u8", T.SHORT: "s16", T.BOOL: "s32",
         T.USHORT: "u16", T.INT: "s32", T.LONG: "s32", T.UINT: "u32", T.ULONG: "u32",
         T.LONGLONG: "s64", T.ULONGLONG: "u64", T.FLOAT: "f32", T.ENUM: "s32"}
    if t.kind in m:
        return m[t.kind]
    if t.kind == T.POINTER:
        return "ptr"
    if t.kind == T.RECORD and t.spelling.replace("struct ", "").endswith("GXColor"):
        return "color"
    raise SystemExit(f"type {t.spelling}")


CT = {"s8": "signed char", "u8": "unsigned char", "s16": "short", "u16": "unsigned short",
      "s32": "int", "u32": "unsigned int", "s64": "long long", "u64": "unsigned long long"}


def strip(e):
    while e.kind in (K.UNEXPOSED_EXPR, K.PAREN_EXPR, K.CSTYLE_CAST_EXPR):
        ch = list(e.get_children())
        if len(ch) != 1:
            break
        e = ch[-1]
    return e


def derefd(fn, params):
    """Parameters the body dereferences (->, *, [] or pointer arithmetic on them)."""
    out = set()
    for d in fn.walk_preorder():
        if d.kind in (K.MEMBER_REF_EXPR, K.ARRAY_SUBSCRIPT_EXPR, K.UNARY_OPERATOR):
            ch = list(d.get_children())
            if not ch:
                continue
            b = strip(ch[0])
            if b.kind == K.DECL_REF_EXPR and b.spelling in params and b.type.get_canonical().kind == T.POINTER:
                toks = [t.spelling for t in d.get_tokens()]
                if d.kind != K.UNARY_OPERATOR or (toks and toks[0] in ("*", "++", "--")) or (toks and toks[-1] in ("++", "--")):
                    out.add(b.spelling)
    return out


def bitfields(fn):
    return sorted({d.referenced.spelling for d in fn.walk_preorder()
                   if d.kind == K.MEMBER_REF_EXPR and d.referenced is not None and d.referenced.kind == K.FIELD_DECL
                   and d.referenced.is_bitfield()})


def field_off(t, path):
    """Byte offset of a dotted field path inside record type t (native layout)."""
    t = t.get_canonical()
    off = 0
    for p in path.split("."):
        o = t.get_offset(p)
        assert o >= 0, (t.spelling, p, o)
        off += o // 8
        f = next(c for c in t.get_fields() if c.spelling == p)
        t = f.type.get_canonical()
    return off, t.get_size()


def main(out):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    idx = ci.Index.create()
    keys = {}
    for tu_path, names in FUNCS.items():
        for n in names:
            keys[(tu_path, n)] = DUPKEY.get((tu_path, n), n)
    inputs, sigs, tus, refused = [], {}, [], []
    for tu_path, names in FUNCS.items():
        src = open(tu_path, "rb").read()
        tu = idx.parse(tu_path, args=NAT)
        found = {c.spelling: c for c in tu.cursor.get_children()
                 if c.kind == K.FUNCTION_DECL and c.is_definition() and c.spelling in names and c.location.file
                 and Path(str(c.location.file)).resolve() == Path(tu_path).resolve()}
        assert set(names) <= set(found), (tu_path, set(names) - set(found))
        stem = Path(tu_path).stem
        lines = src.decode("utf-8", "replace").splitlines()
        pre = []
        for ln in lines:
            s = ln.strip()
            if s.startswith('#include') and not s.rstrip('">').endswith('.h'):
                continue
            if s.startswith('#include "'):
                inc = s.split('"')[1]
                if (Path(tu_path).parent / inc).exists():
                    s = '#include "' + (Path(tu_path).parent / inc).as_posix() + '"'
                pre.append(s)
            elif s.startswith("#include") or (s.startswith("#define") and not s.endswith("\\")
                                            and "(" not in s.split()[1]):
                pre.append(s)
        body = ["/* generated by gen_oracle_ptr.py from " + tu_path + "; do not edit */"] + pre + \
            EXTRA.get(tu_path, []) + ["#include <string.h>", ""]
        disp = []
        # types declared in the .c itself that the cut functions name (e.g. gmClassicMatchup)
        here = Path(tu_path).resolve()
        tdecls = {}
        for n in names:
            for d in found[n].walk_preorder():
                if d.kind == K.TYPE_REF and d.referenced is not None and d.referenced.location.file \
                        and Path(str(d.referenced.location.file)).resolve() == here:
                    r = d.referenced
                    tdecls[r.extent.start.offset] = r
        for o in sorted(tdecls):
            r = tdecls[o]
            body.append(src[r.extent.start.offset:r.extent.end.offset].decode("utf-8") + ";")
        for n in names:
            c = found[n]
            key = keys[(tu_path, n)]
            bf = bitfields(c)
            if bf:
                refused.append(f"{key}: bitfield {bf}")
                continue
            text = src[c.extent.start.offset:c.extent.end.offset].decode("utf-8")
            head, rest = text.split("{", 1)
            i = [m.start() for m in re.finditer(r"(?<![A-Za-z0-9_])" + n + r"\s*\(", head)][-1]
            head = head[:i] + "o_" + key + head[i + len(n):]
            head = head.replace("static ", "").replace("inline ", "")
            body.append(head + "{" + rest + "\n")
            args = list(c.get_arguments())
            pnames = {a.spelling for a in args}
            dr = derefd(c, pnames)
            kinds = []
            for a in args:
                k = kind_of(a.type)
                if k == "ptr" and a.spelling in dr:
                    k = "buf"
                kinds.append(k)
            ret = c.result_type.get_canonical()
            rk = "void" if ret.kind == T.VOID else kind_of(c.result_type)
            sigs[key] = (kinds, rk)
            dl = [f'    if (!strcmp(name, "{key}")) {{']
            call, js, jb = [], 0, 0
            for k, (a, kd) in enumerate(zip(args, kinds)):
                ts = a.type.spelling
                if kd == "buf":
                    call.append(f"({ts}) b[{jb}]"); jb += 1
                elif kd == "ptr":
                    call.append(f"({ts}) (unsigned long long) s[{js}]"); js += 1
                elif kd == "f32":
                    dl.append(f"        float x{k} = bf(s[{js}]);"); call.append(f"x{k}"); js += 1
                elif kd == "color":
                    dl.append(f"        GXColor x{k}; x{k}.r = s[{js}] >> 24; x{k}.g = s[{js}] >> 16; "
                              f"x{k}.b = s[{js}] >> 8; x{k}.a = s[{js}];"); call.append(f"x{k}"); js += 1
                else:
                    call.append(f"({ts}) ({CT[kd]}) s[{js}]"); js += 1
            cs = f"o_{key}({', '.join(call)})"
            if rk == "void":
                dl.append(f"        {cs}; *out = 0; return 0;")
            elif rk == "f32":
                dl.append(f"        *out = fb({cs}); return 0;")
            elif rk == "color":
                dl.append(f"        GXColor r = {cs}; *out = ((unsigned) r.r << 24) | ((unsigned) r.g << 16) | ((unsigned) r.b << 8) | r.a; return 0;")
            else:
                dl.append(f"        *out = (unsigned long long) ({CT[rk]}) {cs} & 0x{MASK[rk]:x}ULL; return 0;")
            dl.append("    }")
            disp += dl
            # ---- input rows
            ptypes = [a.type.get_pointee().get_canonical() if kd == "buf" else None for a, kd in zip(args, kinds)]
            dm = DOMAIN.get(key, [])
            dm = dm + [None] * (len(args) - len(dm))
            nrows = 250
            for _ in range(nrows):
                toks, bufs = [key], []
                for kd, d, pt in zip(kinds, dm, ptypes):
                    if kd == "buf":
                        sz = max(512, max(pt.get_size(), 0) * BUFN.get(key, 1))
                        sz = (sz + 7) // 8 * 8
                        seed = R.getrandbits(64) | 1
                        if key in NULLABLE and len(bufs) >= NULLABLE[key] and R.random() < 0.15:
                            seed = 0
                        bufs.append((seed, sz, pt))
                        toks.append(f"b:{seed:x}:{sz:x}:{max(pt.get_size(), 0):x}")
                    else:
                        v = R.choice(d if d is not None else dom(kd))
                        toks.append(f"s:{v & MASK[kd]:x}")
                for fx in FIX.get(key, lambda b: [])(bufs):
                    toks.append("f:%x:%x:%x:%x" % fx)
                inputs.append(" ".join(toks))
        body.append("static float bf(unsigned long long b) { union { unsigned u; float f; } x; x.u = (unsigned) b; return x.f; }")
        body.append("static unsigned long long fb(float f) { union { unsigned u; float f; } x; x.f = f; return x.u; }")
        body.append(f"int run_{stem}(const char* name, const unsigned long long* s, void** b, unsigned long long* out)")
        body.append("{")
        body += disp
        body.append("    return -1;")
        body.append("}")
        (out / f"o_{stem}.c").write_text("\n".join(body) + "\n", encoding="utf-8")
        tus.append(stem)
    m = ["#include <stdio.h>", "#include <stdlib.h>", "#include <string.h>"]
    m += [f"int run_{s}(const char*, const unsigned long long*, void**, unsigned long long*);" for s in tus]
    m += [MAIN_C.replace("@@RUN@@", " && ".join(f"run_{s}(name, s, b, &r)" for s in tus))]
    (out / "main.c").write_text("\n".join(m) + "\n")
    (out / "inputs.txt").write_text("\n".join(inputs) + "\n")
    with open(out / "sigs.txt", "w") as f:
        for n, (a, r) in sigs.items():
            f.write(f"{n} {' '.join(a) or '-'} -> {r}\n")
    for r in refused:
        print("refused", r)
    print(f"{len(sigs)} functions, {len(tus)} TUs, {len(inputs)} vectors -> {out}")


# ---- fixups: (buf, off, size, value) stores made after the fill
def _rec(pt, path):
    return field_off(pt, path)


def fix_terminator(field, val, cap):
    def f(bufs):
        seed, sz, pt = bufs[0]
        if not seed:
            return []
        es = pt.get_size()
        off, s = _rec(pt, field) if field else (0, es)
        j = R.randrange(0, cap)
        return [(0, j * es + off, s, val)]
    return f


def fix_fields(spec):
    """spec: list of (buf, dotted field, list of values)."""
    def f(bufs):
        r = []
        for k, path, vals in spec:
            seed, sz, pt = bufs[k]
            if not seed:
                continue
            off, s = _rec(pt, path)
            r.append((k, off, s, R.choice(vals) & ((1 << (8 * s)) - 1)))
        return r
    return f


def fix_sjis(bufs):
    # checkStringRest: k full-width spaces (U+3000 as the UTF-8 source compiles: E3 80) then a
    # candidate terminator byte (which the scalar arg may or may not match), sometimes a
    # broken pair.
    k = R.randrange(0, 12)
    r = []
    for i in range(k):
        r += [(0, 2 * i, 1, 0xE3), (0, 2 * i + 1, 1, 0x80)]
    if R.random() < 0.3 and k:
        r.append((0, 2 * R.randrange(k) + 1, 1, R.randrange(256)))
    r.append((0, 2 * k, 1, R.choice([0, 0x0A, 0x7F, 0xFF])))
    # guarantee termination: a run of every candidate terminator byte later on
    r += [(0, 2 * k + 2 + i, 1, v) for i, v in enumerate([0, 0x0A, 0x7F, 0xFF] * 2)]
    return r


def fix_row_caps(bufs):
    # mnCharSel_MexPackRows: small row capacities (a cap of -10^9 makes the C fill loop run
    # 10^9 times, natively as in Rust); a few negative and zero caps stay in.
    return [(0, 4 * r, 4, R.choice([-2, -1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 20, 30]) & 0xFFFFFFFF) for r in range(12)]


FIX = {
    "mnCharSel_MexPackRows": fix_row_caps,
    "gmClassic_GetMatchupCount": fix_terminator("x00", 0x148, 40),
    "un_80303FD4_first": fix_terminator("x0", 9, 40),
    "un_80303FD4_count": fix_terminator("x0", 9, 40),
    "firstState": fix_terminator("id", 0xFF, 30),
    "Command_06": fix_fields([(0, "loop_count", [1, 2, 3])]),
    "gm_801B0730": fix_fields([(0, "unk_0x0", [1, 2, 3, 4])]),
    "tyDisplay_SetGridSize": fix_fields([(0, "x08", list(range(-3, 400)) + [2000, 2540]),
                                         (1, "x00", [0, 1, 2, 3, 4, 7])]),
    "fn_80191FD4_is_selected": None,
    "checkStringRest": fix_sjis,
}
FIX = {k: v for k, v in FIX.items() if v is not None}

MAIN_C = r"""
static unsigned long long sm(unsigned long long* x)
{
    unsigned long long z = (*x += 0x9E3779B97F4A7C15ULL);
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}
static unsigned word(unsigned long long* st)
{
    unsigned long long r = sm(st);
    unsigned v = (unsigned) (r >> 32);
    union { float f; unsigned u; } c;
    switch (r & 7) {
    case 0: case 1: return 0;
    case 2: return v % 3 + 1;
    case 3: return v & 0xFF;
    case 4: c.f = (float) ((int) (v % 1000001u) - 500000) / 1000.0f; return c.u;
    case 5: c.f = (float) ((int) (v % 4001u) - 2000) / 1000.0f; return c.u;
    default: return v & 0xBFFFFFFFu;
    }
}
double __frsqrte(double x) { (void) x; abort(); return 0; }
int main(void)
{
    static char line[4096];
    while (fgets(line, sizeof line, stdin)) {
        char name[128], *p;
        unsigned long long s[16], r = 0, h = 0xCBF29CE484222325ULL;
        void* b[8];
        unsigned sz[8];
        int ns = 0, nb = 0, off = 0, k, i;
        line[strcspn(line, "\r\n")] = 0;
        if (sscanf(line, "%127s%n", name, &off) != 1) continue;
        p = line + off;
        for (;;) {
            unsigned long long v, seed; unsigned a, o, z;
            while (*p == ' ') p++;
            if (!*p) break;
            if (sscanf(p, "s:%llx%n", &v, &k) == 1) { s[ns++] = v; }
            else if (sscanf(p, "b:%llx:%x:%x%n", &seed, &z, &a, &k) == 3) {
                sz[nb] = z;
                if (seed == 0) { b[nb++] = 0; }
                else {
                    unsigned* w = (unsigned*) calloc(z / 4 + 4, 4);
                    unsigned long long st = seed;
                    for (i = 0; i < (int) (z / 4); i++) w[i] = word(&st);
                    b[nb++] = w;
                }
            } else if (sscanf(p, "f:%x:%x:%x:%llx%n", &a, &o, &z, &v, &k) == 4) {
                for (i = 0; i < (int) z; i++) ((unsigned char*) b[a])[o + i] = (unsigned char) (v >> (8 * i));
            } else { fprintf(stderr, "bad token %s\n", p); return 1; }
            p += k;
        }
        if (@@RUN@@) { fprintf(stderr, "no %s\n", name); return 1; }
        for (k = 0; k < nb; k++) {
            if (!b[k]) continue;
            for (i = 0; i < (int) sz[k]; i++) { h ^= ((unsigned char*) b[k])[i]; h *= 0x100000001B3ULL; }
            free(b[k]);
        }
        printf("%s = %llx %llx\n", line, r, h);
    }
    return 0;
}
"""

if __name__ == "__main__":
    main(sys.argv[1])
