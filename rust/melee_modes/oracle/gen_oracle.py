#!/usr/bin/env python3
"""Group 6 C oracle: cut the ported leaf functions VERBATIM out of their game TUs (libclang
extents), wrap each TU's cut in its own C file with that TU's #include and one-line #define
lines, add a stdin-driven dispatcher, and write the input vectors.

  python rust/melee_modes/oracle/gen_oracle.py <outdir>     (from the worktree root)

Writes <outdir>/o_<tu>.c, <outdir>/main.c and <outdir>/inputs.txt. build.sh compiles each
o_*.c the way the shipping build does (contraction on, then rust/melee_runtime/oracle/lower.py
for gwtool's fmadds lowering and float->int saturation) and runs it.

Vector line format (inputs and outputs): `name a0 a1 ... [= r]`, every value the raw bits of
its C type as lower-case hex (floats as their u32 bits, GXColor as r<<24|g<<16|b<<8|a, a Vec3
as three f32 args).
"""
import os, random, sys, struct
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g  # noqa: E402  (libclang setup + the game's NAT clang flags)

ci, K, T = g.ci, g.K, g.T

# (tu, function) -> optional per-argument domain overrides (lists of python values).
FUNCS = {
    "src/melee/gm/gmevent.c": ["gm_GetNextColor"],
    "src/melee/gm/gmmain_lib.c": ["bitset_mask", "bitset64_mask", "selkind_mask"],
    "src/melee/gm/gmfrontend.c": ["fe_np_zero", "fe_rgba", "fe_ease", "fe_row_y"],
    "src/melee/gm/gmregcommon.c": ["gm_8017BE84"],
    "src/melee/gm/gmresult.c": ["gm_WasMatchCanceled"],
    "src/melee/gm/gmresultplayer.c": ["gmRst_IsMex"],
    "src/melee/gm/gmtoulib.c": ["fn_8018F3D0", "fn_8018F6DC", "fn_8018F6FC", "fn_8018F71C",
                                "fn_8018FA24_inline0", "gm_801905F0_inline0"],
    "src/melee/gm/gmtou_0.c": ["fn_801953C8_GetPreviousPosition"],
    "src/melee/gm/gmvs.c": ["direction"],
    "src/melee/gm/gm_1601.c": ["gm_801601C4", "gm_80160244", "fn_80160840", "gm_80160854",
                               "fn_801653E8", "fn_8016588C_clamp"],
    "src/melee/gm/gm_16F1.c": ["fn_801724C8"],
    "src/melee/gm/gm_17AD.c": ["fn_8017AD04", "fn_8017AD28", "fn_8017AD48", "fn_8017AD78"],
    "src/melee/gm/gm_17C0.c": ["BossHook_Kind"],
    # gm_1884.c gm_801891F4_GetTickRate: OS_TIMER_CLOCK reads *(u32*)0x800000F8 (bus clock); not pure.
    "src/melee/gm/gm_1A3F.c": ["gm_Is1PMode"],
    "src/melee/if/ifnametag.c": ["inlineA1"],
    "src/melee/if/ifstock.c": ["ifStock_802F89F8_count", "ifStock_802F89F8_divisor",
                               "ifStock_802F89F8_digit"],
    "src/melee/if/soundtest.c": ["fn_802FFCC8"],
    "src/melee/if/textlib.c": ["DevText_Clamp"],
    "src/melee/if/textlib_1.c": ["adjust"],
    "src/melee/lb/lbaudio_ax.c": ["calcPan"],
    "src/melee/lb/lbcardnew.c": ["convertSdkResult"],
    "src/melee/lb/lbcollision.c": ["isSmall"],
    "src/melee/lb/lbdvd.c": ["same"],
    "src/melee/lb/lbsnap.c": ["RGB565_TO_RGB5A3", "lbSnap_8001E204", "lbSnap_8001E210"],
    "src/melee/lb/lbtime.c": ["lbTime_8000AEC8", "lbTime_8000AEE4", "lbTime_8000AF24",
                              "lbTime_8000AF74"],
    "src/melee/lb/lb_00CE.c": ["expf", "powi"],
    "src/melee/lb/lb_00F9.c": ["approximatelyZeroVec3", "absf"],
    "src/melee/mn/mncharsel.c": ["toMeters", "toFeet", "equalU8", "getIconOffset"],
    "src/melee/mn/mncount.c": ["inline_is_row_time", "inline_is_row_char"],
    "src/melee/mn/mndiagram2.c": ["mnDiagram2_IsTimeStat", "mnDiagram2_IsDistanceStat",
                                  "mnDiagram2_IsPercentageStat", "mnDiagram2_IsIconOnlyStat"],
    "src/melee/mn/mnmain.c": ["GetSelectionFrameOffset", "mn_8022C010"],
    "src/melee/mn/mnname.c": ["signedCharactersEqual", "unsignedCharacter"],
    "src/melee/mn/mnvibration.c": ["mnVibration_GetPreviousCursorRow",
                                   "mnVibration_GetNextCursorRow", "mnVibration_GetPadIndex"],
    "src/melee/ty/tydisplay.c": ["_tyDisplay_80318CB4_calc_dist_sq"],
}

# Round 2: callers of round-1 functions and readers of file-scope const tables (scalar in/out).
ROUND2 = {
    "src/melee/gm/gm_1601.c": ["gm_801601C4_inner", "gm_801601C4_noinline", "gm_80160244_inner",
                               "gm_80160244_noinline", "gm_SelKindToUnlockIndex", "gm_CKindToUnlockIndex",
                               "get_unlockable_selkind_by_bit_index", "gm_GetCKindByUnlockIndex",
                               "fn_80160710", "gm_GetChallengerStKind", "fn_801607A8", "fn_801607F4",
                               "gm_80160968", "gm_SelKindToCKind", "gm_CKindToSelKind", "fn_8016419C",
                               "fn_801641B4", "gm_801641CC", "getStageUnlockIndex", "getStageUnlockNotifyId"],
    "src/melee/gm/gmmain_lib.c": ["selkind_bit"],
    "src/melee/lb/lb_00CE.c": ["powf", "lb_8000D008"],
    "src/melee/lb/lbcardnew.c": ["convertHsdResult"],
    "src/melee/mn/mnmain.c": ["mn_GetDigitAt", "mn_GetDigitCount"],
}
for _t, _n in ROUND2.items():
    FUNCS.setdefault(_t, []).extend(_n)

# Callees from other groups, cut (renamed o_<name>) but not dispatched or tested here.
HELPERS = {"src/melee/lb/lbtrigf.c": ["lbTrigf_Fmadds", "lbTrigf_Fnmsubs", "atanf"]}

# Macros defined under #if blocks the depth-0 #define scan cannot pick (TARGET_PC branch).
EXTRA = {
    "src/melee/lb/lbtrigf.c": ["#define BITWISE(f) (*(u32*) &f)", "#define SIGNED_BITWISE(f) ((s32) BITWISE(f))"],
    "src/melee/gm/gm_1601.c": [
        "#define GM_IS_MEX_CK(ck) ((u32) (ck) >= ChKind_Mex0 && (u32) (ck) < ChKind_Cap)",
        "#define GM_CK_TO_SELKIND(ck) (GM_IS_MEX_CK(ck) ? ckind_to_selkind_map[ChKind_Popo] : ckind_to_selkind_map[ck])",
    ],
}

# Domains where the C has undefined behaviour that gwtool redefines (oversize shift, div by
# zero) or falls off the end; lower.py does not model those, so the oracle stays inside.
DOMAIN = {
    "bitset64_mask": [list(range(64))],
    "selkind_mask": [list(range(32))],
    "ifStock_802F89F8_divisor": [list(range(-3, 13))],
    "ifStock_802F89F8_digit": [None, list(range(-3, 13))],
    "mn_8022C010": [list(range(34)), None],  # MENU_KIND_34 and up fall off the end (no return)
    "powi": [list(range(-12, 13)), list(range(-3, 40))],
    # expf's series only terminates when the f32 sum stops changing; it does not for 68 of
    # the multiples of 1/16 in 12.625 <= |x| <= 14.6875 (the C spins forever too), so the oracle
    # keeps |x| <= 12 (plus 20, -40 and +-inf, which terminate).
    "expf": ["expf"],
    # round 2: table indices kept inside the tables (the C reads past them otherwise)
    "gm_80160968": [list(range(9)) + [0x100, 0x203, 0xFFFFFF08]],
    "gm_SelKindToCKind": [list(range(0x1C))],
    "gm_CKindToSelKind": [list(range(0x21)) + list(range(0x22, 0x80))],
    "gm_CKindToUnlockIndex": [list(range(0x21)) + list(range(0x22, 0x80)) + [0x105, -251]],
    "gm_GetCKindByUnlockIndex": [list(range(-3, 16)) + [0x100, -0x100]],
    "fn_8016419C": [list(range(1, 10))],
    "fn_801641B4": [list(range(1, 10))],
    "gm_801641CC": [list(range(30))],
    "selkind_bit": [None, list(range(32))],
    "mn_GetDigitAt": [None, list(range(32))],
    "powf": ["powf"],
}

R = random.Random(0x6D6F646573)


def f32(x):
    return struct.unpack("<I", struct.pack("<f", x))[0]


def dom(kind, small):
    if kind == "u8":
        return list(range(256)) if not small else [0, 1, 2, 3, 4, 5, 9, 0x13, 0x1D, 0x7F, 0x80, 0xFE, 0xFF]
    if kind == "s8":
        return list(range(-128, 128)) if not small else [-128, -101, -100, -1, 0, 1, 50, 100, 127]
    if kind == "u16":
        return (list(range(0, 65536, 7)) + [0xFFFF]) if not small else [0, 1, 0x7FFF, 0x8000, 0xFFFF]
    if kind in ("s32", "u32", "s64"):
        e = [0, 1, -1, 2, -2, 9, 10, 0x13, 0x1D, 0xFFFF, 0x10000, 9999, 10000, -9999, -10000,
             99999, 100000, -100000, 9999999, 10000000, -10000000, (1 << 24) - 1, 1 << 24,
             -(1 << 24), 0x7FFFFFFF, -0x80000000, 0x7FFFFFFE, -0x7FFFFFFF]
        if small:
            v = e + [R.randrange(-0x80000000, 0x80000000) for _ in range(8)] + list(range(-3, 40))
        else:
            v = e + list(range(-300, 301)) + [R.randrange(-0x80000000, 0x80000000) for _ in range(300)]
        if kind == "u32":
            v = [x & 0xFFFFFFFF for x in v]
        return v
    if kind == "f32":
        sp = [0.0, -0.0, float("inf"), float("-inf"), float("nan"), 1e-5, -1e-5, 0.00001, 0.0000099,
              -0.0000099, 1e-38, 1e-45, 0.5, 1.0, -1.0, 16.0, 15.999, 3e9, -3e9, 127.5, 50.0]
        bits = [f32(x) for x in sp]
        if small:
            return bits + [f32(R.uniform(-200, 200)) for _ in range(30)]
        return (bits + [f32(i * 0.37) for i in range(-800, 801)] + [f32(R.uniform(-2, 2)) for _ in range(500)]
                + [R.getrandbits(32) for _ in range(500)])
    if kind == "color":
        return [R.getrandbits(32) for _ in range(300 if not small else 8)] + [0, 0xFFFFFFFF, 0x80808080]
    raise SystemExit(kind)


def kind_of(t):
    t = t.get_canonical()
    m = {T.CHAR_S: "s8", T.SCHAR: "s8", T.UCHAR: "u8", T.CHAR_U: "u8", T.SHORT: "s16",
         T.USHORT: "u16", T.INT: "s32", T.LONG: "s32", T.UINT: "u32", T.ULONG: "u32",
         T.LONGLONG: "s64", T.ULONGLONG: "u64", T.FLOAT: "f32", T.ENUM: "s32"}
    if t.kind in m:
        return m[t.kind]
    if t.kind == T.RECORD:
        n = t.spelling.replace("struct ", "")
        if n.endswith("GXColor"):
            return "color"
        if n.endswith("Vec3") or n == "Vec":
            return "vec3"
    raise SystemExit(f"type {t.spelling}")


CT = {"s8": "signed char", "u8": "unsigned char", "s16": "short", "u16": "unsigned short",
      "s32": "int", "u32": "unsigned int", "s64": "long long", "u64": "unsigned long long"}
MASK = {"s8": 0xFF, "u8": 0xFF, "s16": 0xFFFF, "u16": 0xFFFF, "s32": 0xFFFFFFFF, "u32": 0xFFFFFFFF,
        "s64": (1 << 64) - 1, "u64": (1 << 64) - 1, "f32": 0xFFFFFFFF, "color": 0xFFFFFFFF}


def proto(c):
    def ct(t):
        t = t.get_canonical()
        return "int" if t.kind == T.ENUM else t.spelling
    a = ", ".join(ct(x.type) for x in c.get_arguments()) or "void"
    return f"{ct(c.result_type)} o_{c.spelling}({a});"


def main(out):
    import re
    out = Path(out)
    allnames = set(n for v in FUNCS.values() for n in v) | set(n for v in HELPERS.values() for n in v)
    ident = re.compile(r"\b(" + "|".join(sorted(allnames, key=len, reverse=True)) + r")\b")
    protos = {}
    files = []
    out.mkdir(parents=True, exist_ok=True)
    idx = ci.Index.create()
    sigs, inputs, tus = {}, [], []
    for tu_path, names0 in list(HELPERS.items()) + list(FUNCS.items()):
        helper = tu_path in HELPERS and names0 is HELPERS[tu_path]
        names = list(names0)
        src = open(tu_path, "rb").read()
        tu = idx.parse(tu_path, args=g.NAT)
        found = {}
        for c in tu.cursor.get_children():
            if (c.kind == K.FUNCTION_DECL and c.is_definition() and c.spelling in names and c.location.file
                    and Path(str(c.location.file)).resolve() == Path(tu_path).resolve()):
                found[c.spelling] = c
        missing = set(names) - set(found)
        assert not missing, (tu_path, missing)
        for n in names:
            protos[n] = proto(found[n])
        stem = Path(tu_path).stem + ("_h" if helper else "")
        # file-scope variables defined in this .c that the cut functions read
        tuvars = {}
        for c in tu.cursor.get_children():
            if (c.kind == K.VAR_DECL and c.location.file
                    and Path(str(c.location.file)).resolve() == Path(tu_path).resolve()):
                tuvars[c.spelling] = c
        used = []
        for n in names:
            for d in found[n].walk_preorder():
                if d.kind == K.DECL_REF_EXPR and d.referenced is not None and d.referenced.spelling in tuvars and d.referenced.spelling not in used:
                    used.append(d.referenced.spelling)
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
        body = ["/* generated by gen_oracle.py from " + tu_path + "; do not edit */"] + pre + EXTRA.get(tu_path, []) + [""]
        for v in used:
            c = tuvars[v]
            body.append(src[c.extent.start.offset:c.extent.end.offset].decode() + ";")
        body.append("")
        body.append("@@PROTOS@@")
        disp = []
        for n in names:
            c = found[n]
            text = src[c.extent.start.offset:c.extent.end.offset].decode()
            head, rest = text.split("{", 1)
            # rename every cut function o_<name> (`expf`/`powi` clash with MSL), calls included
            i = head.rfind(n)
            head = head[:i] + "o_" + n + head[i + len(n):]
            head = head.replace("static ", "").replace("inline ", "")
            rest = ident.sub(lambda m: "o_" + m.group(1), rest)
            body.append(head + "{" + rest + "\n")
            if helper:
                continue
            args = [kind_of(a.type) for a in c.get_arguments()]
            ret = c.result_type.get_canonical()
            rk = "void" if ret.kind == T.VOID else ("s32" if ret.kind == T.BOOL else kind_of(c.result_type))
            sigs[n] = (args, rk)
            # dispatcher: parse hex args, call, print
            dl = [f'    if (!strcmp(name, "{n}")) {{']
            call = []
            j = 0
            for k, a in enumerate(args):
                if a == "f32":
                    dl.append(f"        float x{k} = bf(v[{j}]);"); call.append(f"x{k}"); j += 1
                elif a == "color":
                    dl.append(f"        GXColor x{k}; x{k}.r = v[{j}] >> 24; x{k}.g = v[{j}] >> 16; "
                              f"x{k}.b = v[{j}] >> 8; x{k}.a = v[{j}];"); call.append(f"x{k}"); j += 1
                elif a == "vec3":
                    dl.append(f"        Vec3 x{k}; x{k}.x = bf(v[{j}]); x{k}.y = bf(v[{j+1}]); x{k}.z = bf(v[{j+2}]);")
                    call.append(f"x{k}"); j += 3
                else:
                    dl.append(f"        {CT[a]} x{k} = ({CT[a]}) v[{j}];"); call.append(f"x{k}"); j += 1
            cs = f"o_{n}({', '.join(call)})"
            if rk == "f32":
                dl.append(f"        return fb({cs});")
            elif rk == "color":
                dl.append(f"        GXColor r = {cs}; *out = ((unsigned) r.r << 24) | ((unsigned) r.g << 16) | ((unsigned) r.b << 8) | r.a; return 0;")
            elif rk == "void":
                raise SystemExit(n)
            else:
                dl.append(f"        return (unsigned long long) ({CT[rk]}) {cs} & 0x{MASK[rk]:x}ULL;")
            dl.append("    }")
            disp += dl
            # inputs
            flat = []
            for a in args:
                flat += ["f32"] * 3 if a == "vec3" else [a]
            dm = DOMAIN.get(n, [None] * len(args))
            if dm == ["expf"]:
                ex = [f32(x) for x in [0.0, -0.0, 1e-30, 1e-7, 0.5, 1.0, -1.0, 2.5, -2.5, 10.0, -10.0, 12.0, -12.0, 20.0, -40.0, float('inf'), float('-inf')]]
                ex += [f32(i * 0.0625) for i in range(-192, 193)] + [f32(R.uniform(-12, 12)) for _ in range(800)]
                sets = [ex]
            elif dm == ["powf"]:
                import math
                pr = [(0.0, 3.0), (-0.0, 1.0), (0.0, float("nan")), (1.0, 5.0), (2.0, 10.0), (10.0, 2.0), (0.5, -3.0),
                      (100.0, 2.5), (0.01, -2.5), (4.0, 0.5)]
                for _ in range(1400):
                    a = math.exp(R.uniform(-4.5, 4.5))
                    lim = 12.0 / max(abs(math.log(a)), 1e-3)
                    pr.append((a, R.uniform(-lim, lim)))
                rows = [[f32(a), f32(b)] for a, b in pr]
                for r in rows:
                    inputs.append(n + "".join(f" {x:x}" for x in r))
                continue
            else:
                sets = [d if d is not None else dom(a, len(flat) > 1) for a, d in zip(flat, dm + [None] * (len(flat) - len(dm)))]
            rows = [[]]
            for s in sets:
                rows = [r + [x] for r in rows for x in s]
                if len(rows) > 40000:
                    rows = R.sample(rows, 40000)
            if not flat:
                rows = [[]]
            if len(rows) > 1500:  # keep the leading edge values, sample the rest
                rows = rows[:300] + R.sample(rows[300:], 1200)
            for r in rows:
                inputs.append(n + "".join(f" {x & MASK.get(a, 0xFFFFFFFF):x}" for x, a in zip(r, flat)))
        body.append("#include <string.h>")
        body.append("static float bf(unsigned long long b) { union { unsigned u; float f; } x; x.u = (unsigned) b; return x.f; }")
        body.append("static unsigned long long fb(float f) { union { unsigned u; float f; } x; x.f = f; return x.u; }")
        body.append(f"int run_{stem}(const char* name, const unsigned long long* v, unsigned long long* out)")
        body.append("{")
        for d in disp:
            body.append(d.replace("return ", "*out = ").replace(";\n", ";") if False else d)
        body.append("    return -1;")
        body.append("}")
        # turn `return X;` inside dispatcher branches into `*out = X; return 0;`
        txt = "\n".join(body)
        head_, disp_ = txt.split(f"int run_{stem}(", 1)
        import re
        disp_ = re.sub(r"\n        return (.*);\n    \}", r"\n        *out = \1; return 0;\n    }", disp_)
        disp_ = re.sub(r"(GXColor r = [^;]*;) \*out = ", r"\1 *out = ", disp_)
        files.append((stem, set(names), head_ + f"int run_{stem}(" + disp_ + "\n"))
        tus.append(stem)
    for stem, mine, text in files:
        ext = sorted(n for n in allnames if re.search(r"\bo_" + n + r"\(", text))
        (out / f"o_{stem}.c").write_text(text.replace("@@PROTOS@@", "\n".join(protos[n] for n in ext)))
    m = ["#include <stdio.h>", "#include <stdlib.h>", "#include <string.h>"]
    m += [f"int run_{s}(const char*, const unsigned long long*, unsigned long long*);" for s in tus]
    m += ["double __frsqrte(double x) { (void) x; abort(); return 0; } /* math_ppc.h sqrtf, never called */", "int main(void) {", "    char line[512];",
          "    while (fgets(line, sizeof line, stdin)) {",
          "        char name[128]; unsigned long long v[8], r; int n = 0, off = 0, k;",
          "        line[strcspn(line, \"\\r\\n\")] = 0;",
          "        if (sscanf(line, \"%127s%n\", name, &off) != 1) continue;",
          "        { char* p = line + off; while (sscanf(p, \"%llx%n\", &v[n], &k) == 1) { n++; p += k; } }",
          "        if (" + " && ".join(f"run_{s}(name, v, &r)" for s in tus) + ") { fprintf(stderr, \"no %s\\n\", name); return 1; }",
          "        printf(\"%s = %llx\\n\", line, r);",
          "    }", "    return 0;", "}"]
    (out / "main.c").write_text("\n".join(m) + "\n")
    (out / "inputs.txt").write_text("\n".join(inputs) + "\n")
    with open(out / "sigs.txt", "w") as f:
        for n, (a, r) in sigs.items():
            f.write(f"{n} {' '.join(a) or '-'} -> {r}\n")
    print(f"{len(sigs)} functions, {len(tus)} TUs, {len(inputs)} vectors -> {out}")


if __name__ == "__main__":
    main(sys.argv[1])
