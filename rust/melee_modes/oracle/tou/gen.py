#!/usr/bin/env python3
"""State oracle for the gm tournament / title TUs (gmtitle, gmtitlemode, gmtoulib, gmtoumode,
gmtou_0/1/2, gmtoycollection).

  python rust/melee_modes/oracle/tou/gen.py <outdir>          (from the worktree root; see build.sh)

Builds libtou_oracle.lib from the game's own C, compiled the way the shipping build compiles it
(clang default contraction, -O2 -disable-llvm-passes, then gwtool's IR rewrites via
melee_runtime/oracle/lower.py), and emits tests/tou/gen_tables.rs for the Rust side.

* Each TU is compiled as a unity file: the (lightly patched) .c is #included and followed by a
  generated driver, so static functions and static state are reachable. Patches only touch the
  places where the C reads GameCube memory directly (raw byte offsets into 32-bit-pointer structs,
  big-endian u16 / word tables); each one rewrites the access to the named field / native
  byte order and is listed in PATCHES.
* gm_1601.c and mn_22EC.c are linked as ordinary support TUs (the ported pure callees:
  mn_8022F410/470, fn_8016419C, fn_801641B4, fn_801653E8, fn_80166CBC, gm_80166CCC, ...).
* Every other undefined function / data symbol gets a generated stub. A function stub forwards
  `(name, argument words)` to `orc_cb`, which the Rust test implements (the same hook the Rust
  port's `mod ext` calls), so both implementations see identical callee behaviour and the call
  log is compared too.
"""
import os, re, struct, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_types/gen"))
os.chdir(ROOT)
import gen_types as g  # noqa: E402  (sets up libclang)

ci, K, T = g.ci, g.K, g.T
HERE = Path(__file__).resolve().parent
OUT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "rust/target/tou-oracle"
GM = "src/melee/gm"
# the oracle compiles for x86_64-pc-windows-gnu with -mno-ms-bitfields (same LLP64 layout as msvc,
# Itanium bitfield packing: matches melee_types, see gen_oracle_ptr.py)
NAT = [a.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for a in g.NAT] + ["-mno-ms-bitfields"]
NAT += ["-I.", "-Isrc/melee/gm", "-ftrivial-auto-var-init=zero"]

TUS = ["gmtitle", "gmtitlemode", "gmtoulib", "gmtoumode", "gmtou_0", "gmtou_1", "gmtou_2", "gmtoycollection"]
SUPPORT = ["src/melee/gm/gm_1601.c", "src/melee/mn/mn_22EC.c"]
LIBC = {"memcpy", "memset", "memmove", "strcpy", "strlen", "strcmp", "abort", "abs", "fabsf", "fabs", "sqrtf", "sqrt"}


# ------------------------------------------------------------------------------ patches
def sub1(text, old, new, count=1):
    n = text.count(old)
    assert n == count, f"patch anchor {old!r}: expected {count}, found {n}"
    return text.replace(old, new)


def patch_gmtoulib(t):
    # the three name templates are written at [7..9] (a 9-byte array in the source; 12-byte slots on the GC)
    for a in ("131, 81, 131, 88, 131, 103, 32, 48, 48,", "130, 103, 130, 108, 130, 109, 32, 48, 48,", "130, 98, 130, 111, 130, 116, 32, 48, 48,"):
        t = sub1(t, a, a + " 0, 0, 0,")
    # the header declares it HSD_Archive* (8 bytes native); the 4-byte placeholder would be overrun
    t = sub1(t, "u8 lbl_804D6638[0x4];", "u8 lbl_804D6638[0x8] __attribute__((aligned(8)));")
    # fn_8018E85C: raw slot bytes (stride 0x2C) -> the named BracketEntrySlot fields
    t = sub1(t, "sub = &lbl_80473AB8[outer_idx].x0 + inner_idx * 0x2C;",
             "sub = (u8*) &lbl_80473AB8[outer_idx].slots[inner_idx];")
    t = re.sub(r"\bsub\[0x([0-9A-F]{2})\]", lambda m: f"((BracketEntrySlot*) sub)->x{m.group(1)}", t)
    for ty, off in (("u16", "54"), ("HSD_GObj\*\*", "2C"), ("s32", "44"), ("s32", "48")):
        pat = r"\*\s*\(%s\*?\)\s*\(sub \+ 0x%s\)" % (ty if "HSD" in ty else ty, off)
        t, n = re.subn(pat if "HSD" not in ty else r"\*\s*\(HSD_GObj\*\*\)\s*\(sub \+ 0x2C\)", f"((BracketEntrySlot*) sub)->x{off}", t)
        assert n >= 1, (ty, off)
    # fn_8018F508
    t = sub1(t, "if (slot_ptr[0x4E] != 3) {", "if (((BracketEntry*) base_ptr)->slots[i].x4E != 3) {")
    # fn_8018FA24
    t = sub1(t, "ptr = (u8*) lbl_80473AB8 + i * (s32) 0xDC;", "ptr = (u8*) &lbl_80473AB8[i].slots[0];")
    for off in ("30", "50", "4D", "4E", "4F", "51", "52"):
        t = re.sub(r"\bptr\[0x%s\]" % off, f"((BracketEntrySlot*) ptr)->x{off}", t)
    t, n = re.subn(r"(?<![A-Za-z_])ptr \+= 0x2C;", "ptr = (u8*) ((BracketEntrySlot*) ptr + 1);", t)
    assert n == 1
    return t


def patch_gmtou_0(t):
    # TmData_80194F30 is packed only under MUST_MATCH, so the shipping PC build lays it out with
    # aligned u16s (x37 stride 0x14, x9 at +0xA) while the GC layout (and TmData) is packed.
    # Pack it so the oracle follows the GC layout that the port implements.
    t = sub1(t, "#ifdef MUST_MATCH\n#pragma pack(push, 1)\n#endif\ntypedef struct TmData_80194F30",
             "#pragma pack(push, 1)\ntypedef struct TmData_80194F30")
    t = sub1(t, "#ifdef MUST_MATCH\n#pragma pack(pop)\n#endif\n\n/// Handles tournament settings menu",
             "#pragma pack(pop)\n\n/// Handles tournament settings menu")
    # `(u16*) &lbl_803D9F80` reads big-endian game memory: give it a byte-pair-swapped copy
    m = re.search(r"lbl_803D9F80 = \{(.*?)\};", t, re.S)
    vals = [int(x) for x in re.findall(r"\d+", m.group(1))]
    assert len(vals) == 0x58
    sw = []
    for i in range(0, 0x58, 2):
        sw += [vals[i + 1], vals[i]]
    decl = "static unsigned char lbl_803D9F80_swapped[0x58] = {%s};\n" % ", ".join(map(str, sw))
    t = sub1(t, "typedef void (*lbl_803D9FD8_fn)(s32*, u32, u32);", decl + "typedef void (*lbl_803D9FD8_fn)(s32*, u32, u32);")
    t = sub1(t, "u16* table = (u16*) &lbl_803D9F80;", "u16* table = (u16*) lbl_803D9F80_swapped;")
    return t


def patch_gmtou_1(t):
    # lbl_803B7CE0: big-endian word initializer read as u16 triplets
    m = re.search(r"lbl_803B7CE0 = \{\s*\{(.*?)\}\s*\};", t, re.S)
    words = [int(x, 16) for x in re.findall(r"0x[0-9A-Fa-f]+", m.group(1))]
    assert len(words) == 9
    sw = ["0x%08X" % (((w & 0xFFFF) << 16) | (w >> 16)) for w in words]
    t = t[: m.start(1)] + ", ".join(sw) + t[m.end(1):]
    # fn_8019A158 / fn_8019AF50: lbl_804799D8 + 0x4E (GC offset of x4E) and raw BracketEntry slot bytes
    t = sub1(t, "u8 pad[0x4E];\n    char x4E[20];", "u8 pad[0x52];\n    char x4E[20];")
    t = sub1(t, "cursor[0x4C] = 3;", "bracket->slots[i].x4C = 3;", 2)
    t = sub1(t, "if (cursor[0x4E] == 3) {", "if (bracket->slots[i].x4E == 3) {")
    t = sub1(t, "cursor[0x4C] = v;", "bracket->slots[i].x4C = v;")
    t = sub1(t, "u8 model_idx = cursor[0x50];", "u8 model_idx = lbl_80473AB8[bracket_idx].slots[sel].x50;")
    t = sub1(t, "if (bp[0x30] != 0 && bp[0x4C] == 0) {",
             "if (lbl_80473AB8[bracketIdx].slots[j].x30 != 0 && lbl_80473AB8[bracketIdx].slots[j].x4C == 0) {")
    t = sub1(t, "(&lbl_80473AB8[bracketIdx].slots[0].x4D)[j * 0x2C]", "lbl_80473AB8[bracketIdx].slots[j].x4D")
    t = sub1(t, "dest[0] = base[0x4E];", "dest[0] = base[0x52];")
    t = sub1(t, "dest[1] = base[0x4F];", "dest[1] = base[0x53];")
    t = sub1(t, "(char*) &lbl_804799D8 + 0x4E", "(char*) &lbl_804799D8 + 0x52")
    return t


PATCHES = {"gmtoulib": patch_gmtoulib, "gmtou_0": patch_gmtou_0, "gmtou_1": patch_gmtou_1}

# ------------------------------------------------------------------------------ globals
# (C symbol, owning C TU or None for a stub-defined one, [Rust paths], flags)
#   ro   : input only (the oracle does not hash it after the call)
#   size : compare this many bytes (the C object is smaller than the Rust one)
M = "melee_modes::tu::"
GLOBALS = [
    # gmtoulib
    ("lbl_804D663C", "gmtoulib", [M + "gmtoulib::lbl_804D663C"], {}),
    ("lbl_804D6638", "gmtoulib", [M + "gmtoulib::lbl_804D6638"], {"ptrs": [0]}),
    ("lbl_804D6630", "gmtoulib", [M + "gmtoulib::lbl_804D6630"], {}),
    ("lbl_804D6634", "gmtoulib", [M + "gmtoulib::lbl_804D6634"], {}),
    ("lbl_80473AB8", "gmtoulib", [M + "gmtoulib::lbl_80473AB8"], {}),
    ("lbl_804771B8", "gmtoulib", [M + "gmtoulib::lbl_804771B8"], {}),
    ("gm_80477738", "gmtoulib", [M + "gmtoulib::gm_80477738"], {}),
    ("gm_804771C4", "gmtoulib", [M + "gmtoulib::gm_804771C4"], {}),
    ("lbl_803D9D20", "gmtoulib", [M + "gmtoulib::lbl_803D9D20"], {}),
    ("lbl_803D9DAC", "gmtoulib", [M + "gmtoulib::lbl_803D9DAC"], {}),
    ("lbl_803D9DD0", "gmtoulib", [M + "gmtoulib::lbl_803D9DD0"], {}),
    ("lbl_803D9F0C", "gmtoulib", [M + "gmtoulib::lbl_803D9F0C"], {}),
    ("lbl_803D9EE8", "gmtoulib", [M + "gmtoulib::lbl_803D9EE8"], {}),
    ("lbl_803D9EF4", "gmtoulib", [M + "gmtoulib::lbl_803D9EF4"], {}),
    ("lbl_803D9F00", "gmtoulib", [M + "gmtoulib::lbl_803D9F00"], {}),
    # gmtou_0
    ("lbl_804799B8", "gmtou_0", [M + "gmtou_0::lbl_804799B8"], {}),
    ("lbl_804D6640", "gmtou_0", [M + "gmtou_0::lbl_804D6640"], {}),
    ("lbl_804D6644", "gmtou_0", [M + "gmtou_0::lbl_804D6644"], {}),
    ("lbl_804D6648", "gmtou_0", [M + "gmtou_0::lbl_804D6648"], {}),
    ("lbl_804D664C", "gmtou_0", [M + "gmtou_0::lbl_804D664C"], {}),
    ("lbl_804D6650", "gmtou_0", [M + "gmtou_0::lbl_804D6650"], {}),
    ("lbl_804D6654", "gmtou_0", [M + "gmtou_0::lbl_804D6654"], {}),
    ("lbl_804D6658", "gmtou_0", [M + "gmtou_0::lbl_804D6658"], {}),
    ("lbl_804D665C", "gmtou_0", [M + "gmtou_0::lbl_804D665C"], {}),
    # gmtou_1
    ("lbl_804D6660", "gmtou_1", [M + "gmtou_1::lbl_804D6660"], {}),
    ("lbl_804D6664", "gmtou_1", [M + "gmtou_1::lbl_804D6664"], {}),
    ("lbl_804D6668", "gmtou_1", [M + "gmtou_1::lbl_804D6668"], {}),
    ("lbl_804D666C", "gmtou_1", [M + "gmtou_1::lbl_804D666C"], {}),
    ("lbl_804D6670", "gmtou_1", [M + "gmtou_1::lbl_804D6670"], {}),
    ("lbl_804D6674", "gmtou_1", [M + "gmtou_1::lbl_804D6674"], {}),
    ("lbl_804D6678", "gmtou_1", [M + "gmtou_1::lbl_804D6678"], {}),
    ("lbl_804D667C", "gmtou_1", [M + "gmtou_1::lbl_804D667C"], {}),
    ("lbl_804D6680", "gmtou_1", [M + "gmtou_1::lbl_804D6680"], {}),
    ("lbl_804799D8", "gmtou_1", [M + "gmtou_1::lbl_804799D8"], {}),
    ("lbl_803DA0D0", "gmtou_1", [M + "gmtou_1::lbl_803DA0D0"], {}),
    ("lbl_804D4190", "gmtou_1", [M + "gmtou_1::lbl_804D4190"], {}),
    ("lbl_804D4194", "gmtou_1", [M + "gmtou_1::lbl_804D4194"], {}),
    # gmtou_2
    ("lbl_804D6688", "gmtou_2", [M + "gmtou_2::lbl_804D6688"], {}),
    ("lbl_80479A58", "gmtou_2", [M + "gmtou_2::lbl_80479A58"], {}),
    ("lbl_804D668C", "gmtou_2", [M + "gmtou_2::lbl_804D668C"], {}),
    ("lbl_804D6690", "gmtou_2", [M + "gmtou_2::lbl_804D6690"], {}),
    ("lbl_804D6694", "gmtou_2", [M + "gmtou_2::lbl_804D6694"], {}),
    ("lbl_803DA2E0", "gmtou_2", [M + "gmtou_2::lbl_803DA2E0"], {}),
    # gmtitle
    ("model_desc_0", "gmtitle", [M + "gmtitle::model_desc_0"], {}),
    ("model_desc_1", "gmtitle", [M + "gmtitle::model_desc_1"], {}),
    ("debug_text_buffer", "gmtitle", [M + "gmtitle::debug_text_buffer"], {}),
    ("cobj_desc", "gmtitle", [M + "gmtitle::cobj_desc"], {}),
    ("list_list", "gmtitle", [M + "gmtitle::list_list"], {}),
    ("fog_desc", "gmtitle", [M + "gmtitle::fog_desc"], {}),
    ("countdown_timer", "gmtitle", [M + "gmtitle::countdown_timer"], {}),
    ("frame_count", "gmtitle", [M + "gmtitle::frame_count"], {}),
    ("bg_initialized", "gmtitle", [M + "gmtitle::bg_initialized"], {}),
    # gmtitlemode / gmtoycollection / gmtoumode
    ("exit_data", "gmtitlemode", [M + "gmtitlemode::exit_data"], {}),
    ("gm_804D6918", "gmtoycollection", [M + "gmtoycollection::gm_804D6918"], {}),
    ("gm_804876D8", "gmtoumode", [M + "gmtoumode::gm_804876D8"], {}),
    ("gm_80487810", "gmtoumode", [M + "gmtoumode::gm_80487810"], {}),
    ("gm_80489A98", "gmtoumode", [M + "gmtoumode::gm_80489A98"], {"size": 0x2288}),
    ("gm_8048BD20", "gmtoumode", [M + "gmtoumode::gm_8048BD20"], {"size": 0x148}),
    ("gm_8048BE68", "gmtoumode", [M + "gmtoumode::gm_8048BE68"], {"size": 0x23C8}),
    # stub-defined externs the TUs read (inputs)
    ("HSD_GObjPLinkHead", None, [M + "gmtoulib::ext::HSD_GObjPLinkHead"], {"ro": 1}),
    ("HSD_GObj_JObjKind", None, [M + "gmtoulib::ext::HSD_GObj_JObjKind", M + "gmtitle::ext::HSD_GObj_JObjKind"], {"ro": 1}),
    ("HSD_GObj_FogKind", None, [M + "gmtoulib::ext::HSD_GObj_FogKind", M + "gmtitle::ext::HSD_GObj_FogKind"], {"ro": 1}),
    ("HSD_GObj_LightKind", None, [M + "gmtoulib::ext::HSD_GObj_LightKind", M + "gmtitle::ext::HSD_GObj_LightKind"], {"ro": 1}),
    ("HSD_GObj_CameraKind", None, [M + "gmtoulib::ext::HSD_GObj_CameraKind", M + "gmtitle::ext::HSD_GObj_CameraKind"], {"ro": 1}),
    ("HSD_PadMasterStatus", None, [M + "gmtoulib::ext::HSD_PadMasterStatus"], {"ro": 1}),
    ("gm_804D67EC", None, [M + "gmtitle::ext::gm_804D67EC"], {"ro": 1}),
    ("gm_804D67F0", None, [M + "gmtitle::ext::gm_804D67F0"], {"ro": 1}),
    ("DbLevel", None, [M + "gmtitle::ext::DbLevel", M + "gmtitlemode::ext::DbLevel"], {"ro": 1}),
    ("db_build_timestamp", None, [M + "gmtitle::ext::db_build_timestamp"], {"ro": 1}),
]
GMAP = {n: (tu, rp, fl) for n, tu, rp, fl in GLOBALS}

# types allocated by stubs (return type pointee) and walked for pointer slots
EXTRA_PTR_TYPES = ["HSD_GObj", "HSD_JObj", "HSD_DObj", "HSD_MObj", "HSD_Text", "HSD_GObjProc", "SceneDesc",
                   "SceneCameraDesc", "LightList", "SceneFogDesc", "DynamicModelDesc", "HSD_CObj", "HSD_LObj",
                   "HSD_Fog", "HSD_Archive", "PreloadedGameModeState", "GameRules", "GamePrefs", "HSD_CObjDesc",
                   "HSD_Joint", "HSD_FogDesc", "BracketEntry", "TmData", "MatchEnd", "StartMeleeData", "MatchExitInfo",
                   "TmVsData", "SSSData", "ResultsMatchInfo", "ResultsData", "datetime", "GameModeState", "HSD_SObjDesc"]


# ------------------------------------------------------------------------------ helpers
def run(cmd, **kw):
    r = subprocess.run(cmd, capture_output=True, text=True, **kw)
    if r.returncode:
        sys.stderr.write(" ".join(map(str, cmd)) + "\n" + r.stdout + r.stderr)
        raise SystemExit(1)
    return r.stdout


def parse(path, extra=()):
    idx = ci.Index.create()
    tu = idx.parse(str(path), args=NAT + list(extra), options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
    errs = [d for d in tu.diagnostics if d.severity >= ci.Diagnostic.Error]
    if errs:
        raise SystemExit("clang errors in %s:\n%s" % (path, "\n".join(str(e) for e in errs[:10])))
    return tu


def canon(t):
    return t.get_canonical()


BT = {"p": "void*", "f": "float", "d": "double", "v": "void", "i8": "signed char", "i16": "short", "i32": "int",
      "i64": "long long", "u8": "unsigned char", "u16": "unsigned short", "u32": "unsigned int", "u64": "unsigned long long"}


def bt(t):
    return BT[tclass(t)]


def tclass(t):
    """'p' pointer, 'f' float, 'd' double, 'i<bits>' / 'u<bits>' integers, 'v' void, None unsupported."""
    c = canon(t)
    k = c.kind
    if k == T.POINTER:
        return "p"
    if k == T.FLOAT:
        return "f"
    if k in (T.DOUBLE, T.LONGDOUBLE):
        return "d"
    if k == T.VOID:
        return "v"
    if k == T.ENUM:
        u = canon(c.get_declaration().enum_type)
        return tclass(u)
    if k == T.BOOL:
        return "u8"
    size = c.get_size() * 8
    if k in (T.CHAR_S, T.SCHAR, T.SHORT, T.INT, T.LONG, T.LONGLONG):
        return f"i{size}"
    if k in (T.CHAR_U, T.UCHAR, T.USHORT, T.UINT, T.ULONG, T.ULONGLONG):
        return f"u{size}"
    return None


def ctype_str(t):
    s = t.spelling
    return s


def ptr_offsets(t, base=0, out=None):
    """Byte offsets of every pointer (data or function) inside native type t."""
    if out is None:
        out = []
    c = canon(t)
    if c.kind == T.POINTER:
        out.append(base)
    elif c.kind in (T.CONSTANTARRAY, T.INCOMPLETEARRAY):
        n = c.element_count if c.kind == T.CONSTANTARRAY else 0
        el = c.element_type
        esz = canon(el).get_size()
        if canon(el).kind == T.POINTER or canon(el).kind in (T.RECORD, T.CONSTANTARRAY):
            for i in range(n):
                ptr_offsets(el, base + i * esz, out)
    elif c.kind == T.RECORD:
        for fc in c.get_fields():
            if fc.is_bitfield():
                continue
            ptr_offsets(fc.type, base + fc.get_field_offsetof() // 8, out)
    return out


# ------------------------------------------------------------------------------ bitfields
# The Rust records keep bitfield runs as GC-order bytes (`bf_<off>: [u8; n]`, MSB first); the native
# C build packs them LSB first. To compare states, the harness transcodes between the two with a
# per-field table: (native bit offset, width, Rust group byte offset, Rust group length, shift in
# the big-endian group integer), all relative to the start of the object.
def bf_walk(nt, pt, bn, bp, out):
    nt, pt = canon(nt), canon(pt)
    fn_, fp_ = list(nt.get_fields()), list(pt.get_fields())
    if len(fn_) != len(fp_):
        assert nt.get_size() <= 0 or pt.get_size() <= 0, (nt.spelling, len(fn_), len(fp_))
        return
    union = nt.get_declaration().kind == K.UNION_DECL
    run = []

    def close():
        if not run:
            return
        lo = min(p_off for _, p_off, _, _ in run) // 8
        hi = max((p_off + w + 7) // 8 for _, p_off, w, _ in run)
        n0, _, _, us = run[0]
        unit = (n0 // (us * 8)) * us
        phys = bn // 8 + unit
        for n_off, p_off, w, us in run:
            out.append((bn + n_off, w, phys, hi - lo, (hi - lo) * 8 - (p_off - lo * 8) - w))
        run.clear()

    for a, b in zip(fn_, fp_):
        if a.is_bitfield():
            w = a.get_bitfield_width()
            if w == 0:
                close()
                continue
            run.append((a.get_field_offsetof(), b.get_field_offsetof(), w, canon(a.type).get_size()))
            if union:
                close()
            continue
        close()
        ta, tb = canon(a.type), canon(b.type)
        on, op = a.get_field_offsetof(), b.get_field_offsetof()
        dims = []
        while ta.kind == T.CONSTANTARRAY:
            dims.append(ta.element_count)
            ta, tb = canon(ta.element_type), canon(tb.element_type)
        if ta.kind != T.RECORD:
            continue
        ns, ps = ta.get_size() * 8, tb.get_size() * 8
        total = 1
        for d in dims:
            total *= d
        for i in range(total):
            bf_walk(ta, tb, bn + on + i * ns, bp + op + i * ps, out)
    close()


def bf_table(nt, pt):
    out = []
    t_, p_ = canon(nt), canon(pt)
    dims = []
    while t_.kind == T.CONSTANTARRAY:
        dims.append(t_.element_count)
        t_, p_ = canon(t_.element_type), canon(p_.element_type)
    if t_.kind != T.RECORD:
        return out
    total = 1
    for d in dims:
        total *= d
    for i in range(total):
        bf_walk(t_, p_, i * t_.get_size() * 8, i * p_.get_size() * 8, out)
    return out


def collect_decls(tu, main_file=None):
    funcs, vars_, protos, vdecls = {}, {}, {}, {}
    for c in tu.cursor.walk_preorder():
        if c.kind == K.FUNCTION_DECL:
            if c.is_definition():
                if main_file is None or (c.location.file and Path(c.location.file.name).resolve() == main_file):
                    funcs[c.spelling] = c
            else:
                protos.setdefault(c.spelling, c)
        elif c.kind == K.VAR_DECL and c.semantic_parent.kind == K.TRANSLATION_UNIT:
            in_main = main_file is None or (c.location.file and Path(c.location.file.name).resolve() == main_file)
            if c.storage_class == ci.StorageClass.EXTERN and not c.is_definition():
                vdecls.setdefault(c.spelling, c)
            else:
                vars_[c.spelling] = c
    return funcs, vars_, protos, vdecls


def compile_ll(src, ll, defs=()):
    F = ["--target=x86_64-pc-windows-gnu", "-mno-ms-bitfields", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
         "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-mlong-double-64", "-fno-strict-aliasing", "-fwrapv",
         "-fcommon", "-fgnu89-inline", "-ftrivial-auto-var-init=zero", "-ffp-contract=on", "-O2", "-Xclang",
         "-disable-llvm-passes", "-S", "-emit-llvm", "-w", "-I.", "-Isrc", "-Ipc", "-Ipc/gameworld",
         "-Isrc/melee/gm", "-isystem", "src/MSL", "-isystem", "libs/dolphin/include", "-isystem",
         "libs/dolphin/src", "-isystem", g.GALE, "-include", "src/MSL/math_ppc.h", "-D_Static_assert(a,b)=",
         "-Dsize_t=gc_size_t"] + [f"-D{d}=orcreal_{d}" for d in defs]
    run([str(g.LLVM / "bin" / "clang.exe")] + F + [str(src), "-o", str(ll)])
    lo = str(ll).replace(".ll", ".lo.ll")
    run([sys.executable, str(ROOT / "rust/melee_runtime/oracle/lower.py"), str(ll), lo])
    obj = lo.replace(".lo.ll", ".obj")
    run([str(g.LLVM / "bin" / "clang.exe"), "--target=x86_64-pc-windows-gnu", "-O1", "-w", "-c", lo, "-o", obj])
    return lo, obj


def ll_symbols(lo):
    text = open(lo, encoding="utf8").read()
    defs = set(re.findall(r"^define [^@\n]*@\"?([\w.$]+)\"?\(", text, re.M))
    decls = set(re.findall(r"^declare [^@\n]*@\"?([\w.$]+)\"?\(", text, re.M))
    gdefs = set(re.findall(r"^@\"?([\w.$]+)\"? = (?:dso_local |internal |common |weak )*(?:global|constant|unnamed_addr constant)", text, re.M))
    gdecl = set(re.findall(r"^@\"?([\w.$]+)\"? = external (?:dso_local )?(?:global|constant)", text, re.M))
    return defs, decls, gdefs, gdecl


# ------------------------------------------------------------------------------ main
def conv_arg_c(t, i):
    k = tclass(t)
    sp = t.spelling
    if k == "p":
        return f"({sp})(uintptr_t) w[{i}]"
    if k == "f":
        return f"orc_f32(w[{i}])"
    if k == "d":
        return f"orc_f64(w[{i}])"
    return f"({sp}) w[{i}]"


def conv_ret_c(t, expr):
    k = tclass(t)
    if k == "v":
        return f"{expr}; return 0;"
    if k == "p":
        return f"return (uint64_t)(uintptr_t) ({expr});"
    if k == "f":
        return f"return orc_bits_f32({expr});"
    if k == "d":
        return f"return orc_bits_f64({expr});"
    if k and k[0] == "i":
        return f"return (uint64_t)(int64_t) ({expr});"
    return f"return (uint64_t) ({expr});"


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "patched").mkdir(exist_ok=True)
    lo_files, objs = [], []
    info = {}  # tu -> dict(funcs, vars, tu cursor)
    all_defs, all_decl_funcs, all_decl_vars = set(), {}, {}
    proto_ast, vdecl_ast = {}, {}

    # 1. patched sources + unity drivers
    for tu in TUS:
        text = open(ROOT / GM / f"{tu}.c", encoding="utf8").read()
        if tu in PATCHES:
            text = PATCHES[tu](text)
        pfile = OUT / "patched" / f"{tu}.c"
        pfile.write_text(text, encoding="utf8", newline="\n")
        unity = OUT / f"{tu}_unity.c"
        unity.write_text(f'#include "{pfile.as_posix()}"\n', encoding="utf8", newline="\n")
        astu = parse(unity)
        funcs, vars_, protos, vdecls = collect_decls(astu, pfile.resolve())
        info[tu] = dict(unity=unity, pfile=pfile, funcs=funcs, vars=vars_, ast=astu)
        for n, c in protos.items():
            proto_ast.setdefault(n, c)
        for n, c in vdecls.items():
            vdecl_ast.setdefault(n, c)
        # statics: libclang reports static functions too
    for s in SUPPORT:
        astu = parse(ROOT / s)
        _, _, protos, vdecls = collect_decls(astu, None)
        for n, c in protos.items():
            proto_ast.setdefault(n, c)
        for n, c in vdecls.items():
            vdecl_ast.setdefault(n, c)

    # 2. drivers
    fn_tables = {}
    for tu in TUS:
        d = info[tu]
        funcs = d["funcs"]
        lines = [f'#include "{d["pfile"].as_posix()}"', '#include "orc_rt.h"', ""]
        order = [n for n in funcs]
        # registries
        lines.append(f"void orc_{tu}_globals(void (*reg)(const char*, void*, orc_size)) {{")
        for n, (otu, rp, fl) in GMAP.items():
            if otu == tu:
                lines.append(f'    reg("{n}", (void*) &{n}, (orc_size) sizeof({n}));')
        lines.append("}")
        lines.append(f"void orc_{tu}_funcs(void (*reg)(const char*, void*)) {{")
        for n in order:
            lines.append(f'    reg("{n}", (void*) &{n});')
        lines.append("}")
        lines.append(f"int orc_{tu}_nfuncs(void) {{ return {len(order)}; }}")
        lines.append(f"const char* orc_{tu}_name(int i) {{ static const char* const n[] = {{" + ", ".join(f'"{n}"' for n in order) + "}; return n[i]; }")
        lines.append(f"uint64_t orc_{tu}_call(int i, const uint64_t* w) {{")
        lines.append("    switch (i) {")
        sigs = []
        for i, n in enumerate(order):
            c = funcs[n]
            args = list(c.get_arguments())
            rt = c.result_type
            ok = tclass(rt) is not None and all(tclass(a.type) not in (None, "v") for a in args)
            if not ok or c.type.kind == T.FUNCTIONNOPROTO:
                lines.append(f"    case {i}: return 0; /* {n}: unsupported signature */")
                sigs.append((n, None))
                continue
            call = f"{n}(" + ", ".join(conv_arg_c(a.type, j) for j, a in enumerate(args)) + ")"
            lines.append(f"    case {i}: {{ " + conv_ret_c(rt, call) + " }")
            sigs.append((n, ([tclass(a.type) for a in args], tclass(rt), [a.type.spelling for a in args])))
        lines.append("    }")
        lines.append("    return 0;")
        lines.append("}")
        drv = OUT / f"{tu}_orc.c"
        drv.write_text("\n".join(lines) + "\n", encoding="utf8", newline="\n")
        fn_tables[tu] = sigs
        info[tu]["drv"] = drv

    # rt header
    RT = [
        "typedef unsigned long long uint64_t;", "typedef long long int64_t;", "typedef unsigned int uint32_t;",
        "typedef unsigned long long uintptr_t;", "typedef unsigned long long orc_size;",
        "extern uint64_t orc_cb(const char* name, const uint64_t* args, int n);",
        "static inline float orc_f32(uint64_t w) { union { uint32_t b; float f; } u; u.b = (uint32_t) w; return u.f; }",
        "static inline double orc_f64(uint64_t w) { union { uint64_t b; double f; } u; u.b = w; return u.f; }",
        "static inline uint64_t orc_bits_f32(float f) { union { uint32_t b; float f; } u; u.f = f; return u.b; }",
        "static inline uint64_t orc_bits_f64(double f) { union { uint64_t b; double f; } u; u.f = f; return u.b; }",
    ]
    (OUT / "orc_rt.h").write_text("\n".join(RT) + "\n", encoding="utf8", newline="\n")

    # 3. compile everything
    for tu in TUS:
        lo, obj = compile_ll(info[tu]["drv"], OUT / f"{tu}_orc.ll")
        lo_files.append(lo)
        objs.append(obj)
    # functions the Rust port calls through its `mod ext` hook must be stubbed on the C side too, even
    # when they live in a support TU (the Rust side has no port of them)
    ext_names = set()
    for tu in TUS:
        rt = (ROOT / "rust/melee_modes/src/tu" / f"{tu}.rs").read_text(encoding="utf8")
        ext_names |= set(re.findall(r"^\s+fn (\w+)\s*\([^;{}]*\)[^;{}]*;", rt, re.M))
    for s in SUPPORT:
        name = Path(s).stem
        src_text = (ROOT / s).read_text(encoding="utf8")
        rn = sorted(n for n in ext_names if re.search(r"^[A-Za-z_][\w \*]*[ \*]%s\(" % re.escape(n), src_text, re.M))
        print("support", name, "stubbed:", rn)
        lo, obj = compile_ll(ROOT / s, OUT / f"{name}.ll", rn)
        lo_files.append(lo)
        objs.append(obj)

    defs, gdefs, undef_f, undef_v = set(), set(), set(), set()
    for lo in lo_files:
        d, dc, gd, gdc = ll_symbols(lo)
        defs |= d
        gdefs |= gd
        undef_f |= dc
        undef_v |= gdc
    undef_f = {n for n in undef_f - defs if not n.startswith("llvm.") and n not in LIBC}
    undef_v = {n for n in undef_v - gdefs if n not in LIBC}

    # 4. stubs
    stubs = ['#include "orc_rt.h"', "extern void abort(void);"]
    ret_kinds = {}
    stub_names = []
    skipped = []
    for n in sorted(undef_f):
        c = proto_ast.get(n)
        if c is None:
            skipped.append(n)
            continue
        args = list(c.get_arguments())
        rt = c.result_type
        variadic = c.type.kind == T.FUNCTIONPROTO and c.type.is_function_variadic()
        k = tclass(rt)
        if k is None or any(tclass(a.type) in (None, "v") for a in args):
            skipped.append(n)
            continue
        params = ", ".join(f"{bt(a.type)} a{j}" for j, a in enumerate(args))
        if variadic:
            params = (params + ", ..." if params else "...")
        if not params:
            params = "void"
        conv = []
        for j, a in enumerate(args):
            ak = tclass(a.type)
            if ak == "p":
                conv.append(f"(uint64_t)(uintptr_t) a{j}")
            elif ak == "f":
                conv.append(f"orc_bits_f32(a{j})")
            elif ak == "d":
                conv.append(f"orc_bits_f64(a{j})")
            elif ak[0] == "i":
                conv.append(f"(uint64_t)(int64_t) a{j}")
            else:
                conv.append(f"(uint64_t) a{j}")
        na = len(args)
        wl = ", ".join(conv) if conv else "0"
        rts = bt(rt)
        noret = "NORETURN" if "noreturn" in str(c.raw_comment or "") else ""
        body = [f"{rts if k != 'v' else 'void'} {n}({params}) {{", f"    uint64_t w[{max(na,1)}] = {{{wl}}};",
                f'    uint64_t r = orc_cb("{n}", w, {na});']
        if n == "__assert":
            body.append("    abort();")
        elif k == "v":
            pass
        elif k == "p":
            body.append(f"    return ({rts})(uintptr_t) r;")
        elif k == "f":
            body.append("    return orc_f32(r);")
        elif k == "d":
            body.append("    return orc_f64(r);")
        else:
            body.append(f"    return ({rts}) r;")
        body.append("}")
        stubs.append("\n".join(body))
        stub_names.append(n)
        pointee = ""
        if k == "p":
            pt = canon(rt).get_pointee()
            pointee = canon(pt).get_declaration().spelling or ""
            ptn = pt.spelling.replace("struct ", "")
            pointee = ptn
        ret_kinds[n] = (na, k, pointee, variadic, [tclass(a.type) for a in args])
    # data externs
    stub_vars = []
    for n in sorted(undef_v):
        c = vdecl_ast.get(n)
        if c is None:
            skipped.append(n)
            continue
        sz = canon(c.type).get_size()
        stubs.append(f"unsigned char {n}[{sz if sz > 0 else 4096}] __attribute__((aligned(16)));")
        stub_vars.append(n)
    # globals registry for stub-defined data
    stubs.append("void orc_stub_globals(void (*reg)(const char*, void*, orc_size)) {")
    for n, (otu, rp, fl) in GMAP.items():
        if otu is None and n in stub_vars:
            stubs.append(f'    reg("{n}", (void*) &{n}, (orc_size) sizeof({n}));')
    stubs.append("}")
    stubs.append("void orc_stub_funcs(void (*reg)(const char*, void*)) {")
    for n in stub_names:
        stubs.append(f'    reg("{n}", (void*) &{n});')
    stubs.append("}")
    sfile = OUT / "stubs.c"
    sfile.write_text("\n".join(stubs) + "\n", encoding="utf8", newline="\n")
    lo, obj = compile_ll(sfile, OUT / "stubs.ll")
    objs.append(obj)
    print("stubs:", len(stub_names), "funcs,", len(stub_vars), "vars; skipped:", sorted(skipped))

    # 5. library
    lib = OUT / "tou_oracle.lib"
    if lib.exists():
        lib.unlink()
    run([str(g.LLVM / "bin" / "lld-link.exe"), "/lib", f"/out:{lib}"] + objs)

    # 6. ptr map for globals + extra types
    ptrmap = {}
    for tu in TUS:
        for n, c in info[tu]["vars"].items():
            if n in GMAP:
                ptrmap[n] = (canon(c.type).get_size(), ptr_offsets(c.type))
    for n in stub_vars:
        if n in GMAP:
            ptrmap[n] = (canon(vdecl_ast[n].type).get_size(), ptr_offsets(vdecl_ast[n].type))
    tymap = {}
    anyast = info["gmtoulib"]["ast"]
    typedefs = {}
    def keep(n, ty):
        old_ = typedefs.get(n)
        if old_ is None or (canon(old_).get_size() <= 0 and canon(ty).get_size() > 0):
            typedefs[n] = ty

    for tu_ in TUS:
        for c in info[tu_]["ast"].cursor.walk_preorder():
            if c.kind == K.TYPEDEF_DECL and c.spelling in EXTRA_PTR_TYPES:
                keep(c.spelling, c.underlying_typedef_type)
            elif c.kind == K.STRUCT_DECL and c.spelling in EXTRA_PTR_TYPES and c.is_definition():
                keep(c.spelling, c.type)
    # types referenced by stub return pointees
    for n, (na, k, pt, v, _acl) in ret_kinds.items():
        if pt and pt not in typedefs and pt != "void":
            for c in anyast.cursor.walk_preorder():
                if c.kind == K.TYPEDEF_DECL and c.spelling == pt:
                    typedefs[pt] = c.underlying_typedef_type
                    break
    for n, t in typedefs.items():
        ct = canon(t)
        if ct.kind == T.RECORD:
            tymap[n] = (ct.get_size(), ptr_offsets(t))

    # bitfield tables (native layout vs the ppc32 layout the Rust records follow)
    pidx = ci.Index.create()
    past = {}
    for tu in TUS:
        past[tu] = pidx.parse(str(info[tu]["unity"]), args=list(g.PPC) + ["-I.", "-Isrc/melee/gm", "-ftrivial-auto-var-init=zero"],
                              options=ci.TranslationUnit.PARSE_DETAILED_PROCESSING_RECORD)
    ptd, pvar = {}, {}
    for tu in TUS:
        for c in past[tu].cursor.walk_preorder():
            if c.kind == K.TYPEDEF_DECL:
                o_ = ptd.get(c.spelling)
                if o_ is None or (canon(o_).get_size() <= 0 and canon(c.underlying_typedef_type).get_size() > 0):
                    ptd[c.spelling] = c.underlying_typedef_type
            elif c.kind == K.VAR_DECL and c.semantic_parent.kind == K.TRANSLATION_UNIT:
                pvar.setdefault(c.spelling, c.type)
    bf_types, bf_globals = {}, {}
    for n in tymap:
        if n in typedefs and n in ptd:
            bf_types[n] = bf_table(typedefs[n], ptd[n])
    for n, (otu, rp, fl) in GMAP.items():
        if otu and n in info[otu]["vars"] and n in pvar:
            tb = bf_table(info[otu]["vars"][n].type, pvar[n])
            if tb:
                bf_globals[n] = tb
    print("bitfield types:", {k: len(v) for k, v in bf_types.items() if v}, "globals:", {k: len(v) for k, v in bf_globals.items()})
    emit_rust(info, fn_tables, ptrmap, tymap, ret_kinds, bf_types, bf_globals)
    print("ok:", lib)


def type_decl(t, name):
    c = canon(t)
    dims = []
    el = t
    while canon(el).kind == T.CONSTANTARRAY:
        dims.append(canon(el).element_count)
        el = canon(el).element_type
    base = el.spelling
    return f"{base} {name}" + "".join(f"[{d}]" for d in dims)


# ------------------------------------------------------------------------------ rust tables
RUST_TU_FILES = {tu: ROOT / "rust/melee_modes/src/tu" / f"{tu}.rs" for tu in TUS}
FN_RE = re.compile(r'pub unsafe extern "C" fn (\w+)\(')


def rust_sig(text, name):
    m = re.search(r'pub unsafe extern "C" fn %s\(' % re.escape(name), text)
    if not m:
        return None
    i = m.end()
    depth = 1
    j = i
    while depth:
        if text[j] == "(":
            depth += 1
        elif text[j] == ")":
            depth -= 1
        j += 1
    params = text[i: j - 1]
    rest = text[j:j + 200]
    mr = re.match(r"\s*(?:->\s*([^{]+?))?\s*\{", rest)
    ret = mr.group(1).strip() if mr and mr.group(1) else None
    return params, ret


def split_params(p):
    out, depth, cur = [], 0, ""
    for ch in p:
        if ch in "<([":
            depth += 1
        elif ch in ">)]":
            depth -= 1
        if ch == "," and depth == 0:
            if cur.strip():
                out.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur.strip())
    res = []
    for s in out:
        name, _, ty = s.partition(":")
        res.append((name.strip().lstrip("_") or "a", ty.strip()))
    return res


def rust_dec(ty, expr):
    ty = ty.replace(" ", "")
    if ty.startswith("*mut") or ty.startswith("*const"):
        return f"({expr}) as usize as {ty.replace('*mut', '*mut ').replace('*const', '*const ')}"
    if ty in ("f32",):
        return f"f32::from_bits(({expr}) as u32)"
    if ty in ("f64",):
        return f"f64::from_bits({expr})"
    if ty in ("HSD_GObjEvent",):
        return f"cb_event({expr})"
    if ty in ("i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize"):
        return f"({expr}) as {ty}"
    if ty in ("GObj_RenderFunc",):
        return "None"
    return f"({expr}) as {ty}"


def rust_enc(ty, expr):
    if ty is None:
        return f"{{ {expr}; 0 }}"
    ty = ty.replace(" ", "")
    if ty.startswith("*mut") or ty.startswith("*const"):
        return f"({expr}) as usize as u64"
    if ty == "f32":
        return f"({expr}).to_bits() as u64"
    if ty in ("i8", "i16", "i32", "i64", "isize"):
        return f"({expr}) as i64 as u64"
    if ty in ("u8", "u16", "u32", "u64", "usize"):
        return f"({expr}) as u64"
    return f"({expr}) as i64 as u64"


def emit_rust(info, fn_tables, ptrmap, tymap, ret_kinds, bf_types, bf_globals):
    out = ["// @generated by rust/melee_modes/oracle/tou/gen.py. Do not edit.",
           "#![allow(unused, non_snake_case, clippy::all)]",
           "use melee_modes::tu::*;", "use melee_types::*;", "use std::ffi::{c_char, c_void};", "use std::vec::Vec;", "use super::ext::cb_event;", ""]
    # function table
    rows, arms = [], []
    seen = set()
    for tu in TUS:
        text = RUST_TU_FILES[tu].read_text(encoding="utf8")
        for i, (n, sig) in enumerate(fn_tables[tu]):
            rs = rust_sig(text, n)
            if rs is None or sig is None:
                continue
            params, ret = rs
            ps = split_params(params)
            cargs, crt, spell = sig
            if len(ps) != len(cargs):
                print("arity mismatch", tu, n, len(ps), len(cargs))
                continue
            if (tu, n) in seen:
                continue
            seen.add((tu, n))
            dec = [rust_dec(t, f"w[{j}]") for j, (nm, t) in enumerate(ps)]
            call = f"{tu}::{n}(" + ", ".join(dec) + ")"
            enc = rust_enc(ret, call)
            arms.append(f'        ("{tu}", "{n}") => {enc},')
            rows.append((tu, n, i, cargs, crt, spell))
    out.append("pub const FUNCS: &[(&str, &str, i32, &[&str], &str)] = &[")
    for tu, n, i, cargs, crt, spell in rows:
        sp = ", ".join('"%s"' % x for x in spell)
        out.append(f'    ("{tu}", "{n}", {i}, &[{sp}], "{crt}"),')
    out.append("];")
    out.append("")
    out.append("pub unsafe fn call_rust(tu: &str, name: &str, w: &[u64]) -> u64 {")
    out.append("    match (tu, name) {")
    out += arms
    out.append('        _ => panic!("no rust fn {tu}::{name}"),')
    out.append("    }")
    out.append("}")
    out.append("")
    out.append("pub fn rust_fn_addrs() -> Vec<(&'static str, usize)> { vec![")
    for tu, n, i, cargs, crt, spell in rows:
        out.append(f'    ("{n}", {tu}::{n} as usize),')
    for mod_, cbn in (("gmtoulib", "HSD_GObj_803910D8"), ("gmtoulib", "HSD_GObj_JObjCallback"), ("gmtoulib", "HSD_GObj_LObjCallback"),
                      ("gmtou_0", "HSD_GObj_FogCallback"), ("gmtitle", "HSD_GObj_JObjCallback"), ("gmtitle", "HSD_GObj_LObjCallback"),
                      ("gmtitle", "HSD_GObj_FogCallback")):
        out.append(f'    ("{cbn}", {mod_}::ext::{cbn} as usize),')
    out.append("] }")
    out.append("")
    out.append("pub const TUS: &[&str] = &[" + ", ".join(f'"{t}"' for t in TUS) + "];")
    out.append("")
    # callee table
    out.append("/// (name, fixed arity, return class, pointee type) of every stubbed C callee")
    out.append("pub const CALLEES: &[(&str, usize, &str, &str, &[&str])] = &[")
    for n, (na, k, pt, v, acl) in sorted(ret_kinds.items()):
        ac = ", ".join('"%s"' % x for x in acl)
        out.append(f'    ("{n}", {na}, "{k}", "{pt}", &[{ac}]),')
    out.append("];")
    out.append("")
    out.append("pub struct BfEnt { pub nat: u32, pub w: u8, pub phys: u32, pub len: u8, pub shift: u8 }")
    out.append("/// per TYPES entry: the bitfields of one element")
    out.append("pub const TYPE_BF: &[&[BfEnt]] = &[")
    for n, (sz, offs) in sorted(tymap.items()):
        ents = ", ".join(f"BfEnt {{ nat: {a}, w: {w}, phys: {ph}, len: {ln}, shift: {sh} }}" for a, w, ph, ln, sh in bf_types.get(n, []))
        out.append(f"    &[{ents}],")
    out.append("];")
    out.append("/// (type, size, pointer offsets)")
    out.append("pub const TYPES: &[(&str, usize, &[usize])] = &[")
    for n, (sz, offs) in sorted(tymap.items()):
        out.append(f'    ("{n}", {max(sz, 0)}, &[{", ".join(map(str, offs))}]),')
    out.append("];")
    out.append("")
    # globals
    out.append("pub struct GSpec { pub bf: &'static [BfEnt], pub name: &'static str, pub tu: &'static str, pub c_size: usize, pub ptrs: &'static [usize], pub ro: bool, pub cmp: usize, pub rust: fn() -> Vec<(*mut u8, usize)> }")
    out.append("pub fn globals() -> Vec<GSpec> { vec![")
    for n, (otu, rp, fl) in GMAP.items():
        if n not in ptrmap:
            print("no ptrmap for global", n)
            continue
        sz, offs = ptrmap[n]
        sz = sz if sz > 0 else 4096
        offs = fl.get("ptrs", offs)
        cmp_ = fl.get("size", sz)
        rustfn = "|| vec![" + ", ".join(
            f"(std::ptr::addr_of_mut!({p}) as *mut u8, std::mem::size_of_val(unsafe {{ &*std::ptr::addr_of!({p}) }}))" for p in rp) + "]"
        ents = ", ".join(f"BfEnt {{ nat: {a}, w: {w}, phys: {ph}, len: {ln}, shift: {sh} }}" for a, w, ph, ln, sh in bf_globals.get(n, []))
        out.append(f'    GSpec {{ bf: &[{ents}], name: "{n}", tu: "{otu or ""}", c_size: {sz}, ptrs: &[{", ".join(map(str, offs))}], ro: {str(bool(fl.get("ro"))).lower()}, cmp: {cmp_}, rust: {rustfn} }},')
    out.append("] }")
    out.append("")
    # C FFI glue (only linked with --features c_oracle)
    out.append('#[cfg(feature = "c_oracle")]')
    out.append("pub mod cffi {")
    out.append("    use std::ffi::{c_char, c_void};")
    out.append('    #[link(name = "tou_oracle", kind = "static")]')
    out.append('    extern "C" {')
    for tu in TUS + ["stub"]:
        out.append(f"        fn orc_{tu}_globals(reg: extern \"C\" fn(*const c_char, *mut c_void, u64));")
        out.append(f"        fn orc_{tu}_funcs(reg: extern \"C\" fn(*const c_char, *mut c_void));")
    for tu in TUS:
        out.append(f"        fn orc_{tu}_nfuncs() -> i32;")
        out.append(f"        fn orc_{tu}_name(i: i32) -> *const c_char;")
        out.append(f"        fn orc_{tu}_call(i: i32, w: *const u64) -> u64;")
    out.append("    }")
    out.append("    pub unsafe fn globals(tu: &str, reg: extern \"C\" fn(*const c_char, *mut c_void, u64)) {")
    out.append("        match tu {")
    for tu in TUS + ["stub"]:
        out.append(f'            "{tu}" => orc_{tu}_globals(reg),')
    out.append("            _ => panic!(),")
    out.append("        }")
    out.append("    }")
    out.append("    pub unsafe fn funcs(tu: &str, reg: extern \"C\" fn(*const c_char, *mut c_void)) {")
    out.append("        match tu {")
    for tu in TUS + ["stub"]:
        out.append(f'            "{tu}" => orc_{tu}_funcs(reg),')
    out.append("            _ => panic!(),")
    out.append("        }")
    out.append("    }")
    out.append("    pub unsafe fn nfuncs(tu: &str) -> i32 {")
    out.append("        match tu {")
    for tu in TUS:
        out.append(f'            "{tu}" => orc_{tu}_nfuncs(),')
    out.append("            _ => panic!(),")
    out.append("        }")
    out.append("    }")
    out.append("    pub unsafe fn name(tu: &str, i: i32) -> *const c_char {")
    out.append("        match tu {")
    for tu in TUS:
        out.append(f'            "{tu}" => orc_{tu}_name(i),')
    out.append("            _ => panic!(),")
    out.append("        }")
    out.append("    }")
    out.append("    pub unsafe fn call(tu: &str, i: i32, w: *const u64) -> u64 {")
    out.append("        match tu {")
    for tu in TUS:
        out.append(f'            "{tu}" => orc_{tu}_call(i, w),')
    out.append("            _ => panic!(),")
    out.append("        }")
    out.append("    }")
    out.append("}")
    out.append("")
    (ROOT / "rust/melee_modes/tests/tou/gen_tables.rs").parent.mkdir(parents=True, exist_ok=True)
    (ROOT / "rust/melee_modes/tests/tou/gen_tables.rs").write_text("\n".join(out) + "\n", encoding="utf8", newline="\n")


if __name__ == "__main__":
    main()
