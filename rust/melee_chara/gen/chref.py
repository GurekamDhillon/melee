#!/usr/bin/env python3
"""C oracle for the hand-ported character functions listed in gen/ported.txt.

  python rust/melee_chara/gen/chref.py [-v]      (from the worktree root; Windows, LLVM clang)

For every TU named in ported.txt a driver `#include`s the game's own .c file (so static helpers
are callable) and dispatches to the listed functions. Every TU is compiled the way the shipping
build compiles game code: clang `-ffp-contract=on`, `-O2 -Xclang -disable-llvm-passes` to IR,
then gwtool's lowering redone by melee_runtime/oracle/lower.py (f32 fmuladd -> fmadds), then
native x86-64 code. The group-3 ftCommon movement helpers the characters call are cut verbatim
from ftcommon.c (as group 3's gen/cref/run.py does). Everything else the TUs reference stays
unresolved (`/FORCE:UNRESOLVED`): no listed function reaches it.

Each function runs on CASES random fighters (gen/chref_main.c: the fill is repeated exactly by
tests/chars_ref.rs). The output, tests/data/chars.txt, is one FNV-1a hash per function over all
cases (Fighter bytes, dat_attrs block, common data, return value bits, out-params). With -v the
C prints one line per case instead (for debugging a mismatch with CHARS_VERBOSE=1 in Rust).

Also writes tests/chars_dispatch.rs (the Rust side of the dispatch). Computed from C source only.
"""
import os, re, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
CRATE = HERE.parent
BUILD = ROOT / "rust/target/chref"
CLANG = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
GALE = os.environ.get("GALE_INCLUDE", str((ROOT / "../../melee/build/GALE01/include").resolve()))
COMMON = ["--target=x86_64-pc-windows-msvc", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
          "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-fno-strict-aliasing", "-fwrapv",
          "-fcommon", "-fgnu89-inline", "-fno-fast-math", "-ftrivial-auto-var-init=zero",
          "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t", "-mlong-double-64", "-Wno-everything",
          "-I", str(ROOT / "src"), "-I", str(ROOT / "src/melee"), "-I", str(ROOT / "pc"),
          "-I", str(ROOT / "pc/gameworld"), "-I", str(BUILD),
          "-isystem", str(ROOT / "tools/rust-port/include"), "-isystem", str(ROOT / "src/MSL"),
          "-isystem", str(ROOT / "libs/dolphin/include"), "-isystem", str(ROOT / "libs/dolphin/src"),
          "-isystem", GALE, "-include", str(ROOT / "src/MSL/math_ppc.h")]
SHIP_IR = ["-O2", "-Xclang", "-disable-llvm-passes", "-ffp-contract=on", "-S", "-emit-llvm"]
LOWER = ROOT / "rust/melee_runtime/oracle/lower.py"
FTCOMMON = [("src/melee/ft/ftcommon.c", "void ftCommon_CalcGroundAccel_Deaccel(", "void ftCommon_UseAllJumps("),
            ("src/melee/ft/ftcommon.c", "void ftCommon_UpdateFacing(", "void ftCommon_8007DB24("),
            ("src/melee/ft/ftcommon.c", "void ftCommon_8007E2F4(", "void ftCommon_8007E358(")]
# functions whose gobj may be NULL (they test it): Rust takes Option<&Fighter>
NULLABLE = {"ftSs_SpecialN_801291A8", "ftSs_SpecialLw_80129158"}


def signature(src, name):
    m = re.search(r'^((?:static\s+)?[A-Za-z][\w \t\*]*?)\b' + name + r'\s*\(([^;{)]*)\)\s*\{', src, re.M)
    ret = m.group(1).replace("static", "").strip()
    params = [p.strip() for p in " ".join(m.group(2).split()).split(",")]
    kinds = []
    for p in params:
        if re.match(r'(HSD_GObj|Fighter_GObj)\s*\*', p):
            kinds.append("Gn" if name in NULLABLE else "G")
        elif re.match(r'Fighter\s*\*', p):
            kinds.append("F" if not kinds else "V")  # second Fighter* = victim (only read)
        elif p.startswith("Vec3*"):
            kinds.append("V3")
        elif p.startswith("Vec2*"):
            kinds.append("V2")
        elif re.match(r'(f32|float)\b', p):
            kinds.append("A")
        elif re.match(r'(s32|int)\s+\w+$', p):
            kinds.append("I" if kinds else "K")
        elif re.match(r'int\s*\*', p):
            kinds.append("P")
        else:
            raise SystemExit(f"{name}: unhandled param {p!r}")
    if ret == "void":
        rk = "v"
    elif ret in ("float", "f32"):
        rk = "f"
    else:
        rk = "i"  # bool / s32 / enum: MSL bool is int
    return rk, kinds


R2 = set()
R3 = set()
R4 = set()
R5 = set()
# Fighter's HSD_GObjEvent fields: stored callbacks are mapped back to names before hashing
CB_FIELDS = ["grab_cb", "input_cb", "anim_cb", "phys_cb", "coll_cb", "cam_cb", "accessory1_cb",
             "accessory2_cb", "accessory3_cb", "accessory4_cb", "deal_dmg_cb", "shield_hit_cb",
             "reflect_hit_cb", "hitlag_cb", "pre_hitlag_cb", "post_hitlag_cb", "take_dmg_cb",
             "death1_cb", "death2_cb", "death3_cb", "take_dmg_2_cb", "hurtbox_detect_cb"]


def callbacks():
    out = []
    for name in ("callbacks_r3.txt", "callbacks_r4.txt", "callbacks_r5.txt"):
        p = HERE / name
        if p.exists():
            out += [tuple(l.split()) for l in p.read_text().split("\n") if l.strip() and not l.startswith("#")]
    return out


def load():
    funcs = []
    for line in (HERE / "ported.txt").read_text().split("\n"):
        if not line.strip() or line.startswith("#"):
            continue
        tu, name = line.split()
        src = (ROOT / tu).read_text(encoding="utf-8")
        rk, kinds = signature(src, name)
        funcs.append((tu, name, rk, kinds))
    # round 2 (gen/port_r2.py): void f(HSD_GObj* gobj), Rust takes the raw gobj
    for line in (HERE / "ported_r2.txt").read_text().split("\n"):
        if not line.strip() or line.startswith("#"):
            continue
        tu, name = line.split()
        R2.add(name)
        funcs.append((tu, name, "v", ["R"]))
    # round 3 (gen/port_r3.py): same signature
    p = HERE / "ported_r3.txt"
    for line in (p.read_text().split("\n") if p.exists() else []):
        if not line.strip() or line.startswith("#"):
            continue
        tu, name = line.split()
        R3.add(name)
        funcs.append((tu, name, "v", ["R"]))
    # round 4 (gen/port_r4.py): same signature
    p = HERE / "ported_r4.txt"
    for line in (p.read_text().split("\n") if p.exists() else []):
        if not line.strip() or line.startswith("#"):
            continue
        tu, name = line.split()
        R4.add(name)
        funcs.append((tu, name, "v", ["R"]))
    # round 5 (gen/port_r5.py): same signature
    p = HERE / "ported_r5.txt"
    for line in (p.read_text().split("\n") if p.exists() else []):
        if not line.strip() or line.startswith("#"):
            continue
        tu, name = line.split()
        R5.add(name)
        funcs.append((tu, name, "v", ["R"]))
    return funcs


def c_args(kinds):
    out = []
    for k in kinds:
        out.append({"G": "g", "Gn": "(nul ? NULL : g)", "F": "fp", "V": "fp", "V3": "v3", "V2": "v2",
                    "A": "a", "I": "(ia & 3)", "K": "ia", "R": "g", "P": ["o1", "o2"][sum(1 for x in out if x in ("o1", "o2"))]}[k])
    return ", ".join(out)


def rs_args(kinds):
    out, np = [], 0
    for k in kinds:
        if k == "P":
            out.append(["o1", "o2"][np]); np += 1
        else:
            out.append({"G": "fp", "Gn": "if nul { None } else { Some(&*fp) }", "F": "fp", "V": "&*fp",
                        "V3": "v3", "V2": "v2", "A": "a", "I": "ia & 3", "K": "ia", "R": "g"}[k])
    return ", ".join(out)


def module_path(tu, name=None):
    d = Path(tu).parent.name
    if name in R2:
        return f"melee_chara::chars::r2::{d}::{Path(tu).stem}"
    if name in R3:
        return f"melee_chara::chars::r3::{d}::{Path(tu).stem}"
    if name in R4:
        return f"melee_chara::chars::r4::{d}::{Path(tu).stem}"
    if name in R5:
        return f"melee_chara::chars::r5::{d}::{Path(tu).stem}"
    return f"melee_chara::chars::{d}::{Path(tu).stem}"


def write_sources(funcs):
    tus = sorted({f[0] for f in funcs})
    drivers = []
    for ti, tu in enumerate(tus):
        body = [f'/* generated by gen/chref.py */\n#include "{ROOT / tu}"\n'.replace("\\", "/"),
                "void drv_%d(int which, HSD_GObj* g, Fighter* fp, float a, int ia, int nul, Vec3* v3, "
                "Vec2* v2, int* o1, int* o2, unsigned* ret)\n{\n    switch (which) {" % ti]
        for fi, (t, name, rk, kinds) in enumerate(funcs):
            if t != tu:
                continue
            call = f"{name}({c_args(kinds)})"
            if rk == "f":
                stmt = f"{{ float r = {call}; *ret = *(unsigned*) &r; }}"
            elif rk == "i":
                stmt = f"*ret = (unsigned) {call};"
            else:
                stmt = f"{call};"
            body.append(f"    case {fi}: {stmt} break;")
        body.append("    }\n}\n")
        cbs = [(k, n) for t, k, n in callbacks() if t == tu]
        body.append("/* round-3 callbacks this TU stores (address -> key) */")
        body.append("const void* const cbp_%d[] = { %s0 };" % (ti, "".join(f"(const void*) {n}, " for _, n in cbs)))
        body.append("const char* const cbn_%d[] = { %s0 };" % (ti, "".join(f'"{t}:{n}", ' for t, n in cbs)))
        p = BUILD / f"drv_{ti}.c"
        p.write_text("\n".join(body), encoding="utf-8")
        drivers.append((ti, tu, p))
    tbl = ["/* generated by gen/chref.py */",
           *[f"void drv_{ti}(int, HSD_GObj*, Fighter*, float, int, int, Vec3*, Vec2*, int*, int*, unsigned*);"
             for ti, _, _ in drivers],
           "typedef void (*drv_fn)(int, HSD_GObj*, Fighter*, float, int, int, Vec3*, Vec2*, int*, int*, unsigned*);",
           "static const struct { drv_fn drv; const char* name; } FUNCS[] = {"]
    tu_index = {tu: ti for ti, tu, _ in drivers}
    for t, name, rk, kinds in funcs:
        tbl.append(f'    {{ drv_{tu_index[t]}, "{name}" }},')
    tbl.append("};")
    tbl.append(f"#define N_R1 {sum(1 for f in funcs if f[1] not in R2 and f[1] not in R3 and f[1] not in R4 and f[1] not in R5)}")
    for ti, _, _ in drivers:
        tbl.append(f"extern const void* const cbp_{ti}[]; extern const char* const cbn_{ti}[];")
    tbl.append("static const void* const* const CBP[] = { %s };" % ", ".join(f"cbp_{ti}" for ti, _, _ in drivers))
    tbl.append("static const char* const* const CBN[] = { %s };" % ", ".join(f"cbn_{ti}" for ti, _, _ in drivers))
    tbl.append("#define N_CBTAB %d" % len(drivers))
    tbl.append("static const unsigned CB_OFF[] = { %s };" % ", ".join(f"(unsigned) __builtin_offsetof(Fighter, {f})" for f in CB_FIELDS))
    tbl.append("#define N_CB_OFF %d" % len(CB_FIELDS))
    # callbacks no built TU defines: a weak definition gives each its own address
    srcs = [(ROOT / tu).read_text(encoding="utf-8", errors="replace") for _, tu, _ in drivers]
    ext = sorted({n for t, k, n in callbacks()
                  if not any(re.search(r"^(?:static\s+)?(?:inline\s+)?void\s+" + n + r"\s*\([^;]*\)\s*\{", src, re.M)
                             for src in srcs)})
    weak = ["/* generated by gen/chref.py: callbacks stored by ports whose defining TU is not built here */",
            "typedef struct HSD_GObj HSD_GObj;", "volatile unsigned chref_weak_sink;"]
    weak += [f"__attribute__((weak)) void {n}(HSD_GObj* g) {{ (void) g; chref_weak_sink = {k + 1}u; }}"
             for k, n in enumerate(ext)]
    (BUILD / "chref_weak.c").write_text("\n".join(weak) + "\n", encoding="utf-8")
    (BUILD / "chref_table.inc").write_text("\n".join(tbl) + "\n", encoding="utf-8")
    # ftCommon extract
    parts = []
    for src, start, stop in FTCOMMON:
        lines = (ROOT / src).read_text(encoding="utf-8").splitlines()
        i = next(k for k, l in enumerate(lines) if l.startswith(start))
        j = next(k for k in range(i, len(lines)) if lines[k].startswith(stop))
        parts.append(f"/* extracted from {src}:{i + 1}-{j} by gen/chref.py */\n" + "\n".join(lines[i:j]) + "\n")
    (BUILD / "ftcommon_extract.inc").write_text("".join(parts), encoding="utf-8")
    # Rust dispatch
    rs = ["// @generated by rust/melee_chara/gen/chref.py from gen/ported*.txt. Do not edit.",
          "#![allow(unused_variables, unused_imports, clippy::all)]",
          "use melee_chara::types_gen::{Fighter, HSD_GObj, Vec2, Vec3};", "",
          f"pub const N_R1: usize = {sum(1 for f in funcs if f[1] not in R2 and f[1] not in R3 and f[1] not in R4 and f[1] not in R5)};",
          "pub const CB_OFF: &[usize] = &[%s];" % ", ".join(f"core::mem::offset_of!(Fighter, {f})" for f in CB_FIELDS),
          "pub const NAMES: &[&str] = &[", *[f'    "{f[1]}",' for f in funcs], "];", "",
          "#[allow(clippy::too_many_arguments)]",
          "pub fn call(which: usize, g: *mut HSD_GObj, fp: &mut Fighter, a: f32, ia: i32, nul: bool, v3: &mut Vec3, v2: &mut Vec2,",
          "            o1: &mut i32, o2: &mut i32) -> u32 {",
          "    match which {"]
    for fi, (tu, name, rk, kinds) in enumerate(funcs):
        call = f"{module_path(tu, name)}::{name}({rs_args(kinds)})"
        if rk == "f":
            e = f"{call}.to_bits()"
        elif rk == "i":
            e = f"{call} as u32"
        else:
            e = f"{{ {call}; 0 }}"
        rs.append(f"        {fi} => {e},")
    rs += ["        _ => unreachable!(),", "    }", "}", ""]
    (CRATE / "tests/chars_dispatch.rs").write_text("\n".join(rs), encoding="utf-8", newline="\n")
    return drivers


def run(cmd):
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(" ".join(map(str, cmd)) + "\n" + r.stdout + r.stderr)
        raise SystemExit("chref: build failed")
    return r


def ship_obj(c, stem):
    ll, lo = BUILD / f"{stem}.ll", BUILD / f"{stem}.lo.ll"
    run([CLANG, *COMMON, *SHIP_IR, str(c), "-o", str(ll)])
    run([sys.executable, str(LOWER), str(ll), str(lo)])
    obj = BUILD / f"{stem}.obj"
    run([CLANG, "--target=x86_64-pc-windows-msvc", "-O2", "-c", "-w", str(lo), "-o", str(obj)])
    return obj


def main():
    verbose = "-v" in sys.argv
    BUILD.mkdir(parents=True, exist_ok=True)
    (CRATE / "tests/data").mkdir(parents=True, exist_ok=True)
    funcs = load()
    drivers = write_sources(funcs)
    objs = [ship_obj(p, f"drv_{ti}") for ti, _, p in drivers]
    objs.append(ship_obj(HERE / "chref_main.c", "chref_main"))
    objs.append(ship_obj(BUILD / "chref_weak.c", "chref_weak"))
    gk = BUILD / "gekko_fp.obj"
    run([CLANG, *COMMON, "-O2", "-c", str(ROOT / "pc/gameworld/gekko_fp.c"), "-o", str(gk)])
    exe = BUILD / "chref.exe"
    run([CLANG, "--target=x86_64-pc-windows-msvc", *map(str, objs), str(gk), "-o", str(exe),
         "-fuse-ld=lld", "-Wl,/FORCE:UNRESOLVED", "-Wl,/IGNORE:4006", "-Wl,/IGNORE:4088"])
    out = subprocess.run([str(exe)] + (["-v"] if verbose else []), capture_output=True, text=True, check=True).stdout
    dst = BUILD / "chars_verbose.txt" if verbose else CRATE / "tests/data/chars.txt"
    dst.write_text(out, encoding="utf-8", newline="\n")
    print(f"chref: {len(funcs)} functions in {len(drivers)} TUs -> {dst}")


if __name__ == "__main__":
    main()
