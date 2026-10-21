#!/usr/bin/env python3
"""Build and run the C reference programs; write their expected values for the Rust tests.

  python rust/melee_fighter/gen/cref/run.py          (from the worktree root; Windows, LLVM clang)

Each <name>_ref.c is compiled natively (x86_64-pc-windows-msvc, the same flags and masks as
tools/rust-port/gen_cc.py, -O0 -ffp-contract=off) against the real headers; source that must
be the game's own is cut out of the game's .c file (EXTRACTS) rather than retyped. Output goes
to rust/melee_fighter/tests/data/<name>.txt, which the tests read. The data is computed from C
source only: no disc data.
"""
import os, subprocess, sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
DATA = ROOT / "rust/melee_fighter/tests/data"
BUILD = ROOT / "rust/target/cref"
CLANG = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
GALE = os.environ.get("GALE_INCLUDE", str(ROOT / "../../melee/build/GALE01/include"))
FLAGS = ["--target=x86_64-pc-windows-msvc", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT",
         "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-fno-strict-aliasing", "-fwrapv",
         "-fcommon", "-fgnu89-inline", "-ffp-contract=off", "-fno-fast-math", "-O0",
         "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t", "-mlong-double-64", "-Wno-everything",
         "-I", str(ROOT / "src"), "-I", str(ROOT / "src/melee"), "-I", str(ROOT / "pc"),
         "-I", str(ROOT / "pc/gameworld"), "-I", str(BUILD),
         "-isystem", str(ROOT / "tools/rust-port/include"), "-isystem", str(ROOT / "src/MSL"),
         "-isystem", str(ROOT / "libs/dolphin/include"), "-isystem", str(ROOT / "libs/dolphin/src"),
         "-isystem", GALE, "-include", str(ROOT / "src/MSL/math_ppc.h")]

# name -> list of (game source, first line starting with, stop before line starting with)
EXTRACTS = {
    "nh": [("src/melee/ft/fighter.c", "static u32 nh_Mix(", "/* Kept in game memory")],
    "stale": [],
}


def extract(name):
    parts = []
    for src, start, stop in EXTRACTS.get(name, []):
        lines = (ROOT / src).read_text(encoding="utf-8").splitlines()
        i = next(k for k, l in enumerate(lines) if l.startswith(start))
        j = next(k for k in range(i, len(lines)) if lines[k].startswith(stop))
        parts.append(f"/* extracted from {src}:{i + 1}-{j} by gen/cref/run.py */\n" + "\n".join(lines[i:j]) + "\n")
    (BUILD / f"{name}_extract.inc").write_text("".join(parts), encoding="utf-8")


def main():
    BUILD.mkdir(parents=True, exist_ok=True)
    DATA.mkdir(parents=True, exist_ok=True)
    names = sys.argv[1:] or sorted(p.name[:-6] for p in HERE.glob("*_ref.c"))
    for n in names:
        extract(n)
        exe = BUILD / f"{n}_ref.exe"
        # gekko_fp.c: __frsqrte and friends that math_ppc.h (force-included) calls
        cmd = [CLANG, *FLAGS, str(HERE / f"{n}_ref.c"), str(ROOT / "pc/gameworld/gekko_fp.c"), "-o", str(exe)]
        r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
        if r.returncode:
            sys.stderr.write(r.stdout + r.stderr)
            raise SystemExit(f"cref: {n} failed to build")
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        (DATA / f"{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"cref: {n}: {len(out.splitlines())} lines -> {DATA / (n + '.txt')}")


if __name__ == "__main__":
    main()
