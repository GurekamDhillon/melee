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
FTCOMMON = [("src/melee/ft/ftcommon.c", "void ftCommon_CalcGroundAccel_Deaccel(", "void ftCommon_UnlockECB("),
            ("src/melee/ft/ftcommon.c", "void ftCommon_UpdateFacing(", "void ftCommon_8007DB24(")]
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


def load():
    funcs = []
    for line in (HERE / "ported.txt").read_text().split("\n"):
        if not line.strip() or line.startswith("#"):
            continue
        tu, name = line.split()
        src = (ROOT / tu).read_text(encoding="utf-8")
        rk, kinds = signature(src, name)
        funcs.append((tu, name, rk, kinds))
    return funcs


def c_args(kinds):
    out = []
    for k in kinds:
        out.append({"G": "g", "Gn": "(nul ? NULL : g)", "F": "fp", "V": "fp", "V3": "v3", "V2": "v2",
                    "A": "a", "I": "(ia & 3)", "K": "ia", "P": ["o1", "o2"][sum(1 for x in out if x in ("o1", "o2"))]}[k])
    return ", ".join(out)


def rs_args(kinds):
    out, np = [], 0
    for k in kinds:
        if k == "P":
            out.append(["o1", "o2"][np]); np += 1
        else:
            out.append({"G": "fp", "Gn": "if nul { None } else { Some(&*fp) }", "F": "fp", "V": "&*fp",
                        "V3": "v3", "V2": "v2", "A": "a", "I": "ia & 3", "K": "ia"}[k])
    return ", ".join(out)


def module_path(tu):
    d = Path(tu).parent.name
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
    rs = ["// @generated by rust/melee_chara/gen/chref.py from gen/ported.txt. Do not edit.",
          "#![allow(unused_variables, unused_imports, clippy::all)]",
          "use melee_chara::types_gen::{Fighter, Vec2, Vec3};", "",
          "pub const NAMES: &[&str] = &[", *[f'    "{f[1]}",' for f in funcs], "];", "",
          "#[allow(clippy::too_many_arguments)]",
          "pub fn call(which: usize, fp: &mut Fighter, a: f32, ia: i32, nul: bool, v3: &mut Vec3, v2: &mut Vec2,",
          "            o1: &mut i32, o2: &mut i32) -> u32 {",
          "    match which {"]
    for fi, (tu, name, rk, kinds) in enumerate(funcs):
        call = f"{module_path(tu)}::{name}({rs_args(kinds)})"
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
