#!/usr/bin/env python3
"""Where does the SHIPPING build fuse a*b+c? Per-function llvm.fmuladd counts, from the exact
front-end command `_build/masstest/pipe_win.sh` runs (ppc32-none-eabi, clang default
-ffp-contract=on, -O2 -Xclang -disable-llvm-passes). gwtool then lowers every f32 fmuladd to
fmadds (f64 multiply + f64 add, one f32 rounding) and every f64 one to a true fma; with
GW_GWTOOL_FLAGS unset (the default) `--no-exact-fmuladd` is off, so every site is fused.

  python tools/rust-port/fma_sites.py [--func REGEX] src/melee/mp/mplib.c ...   (worktree root)

Prints `file function f32_sites f64_sites`, then each site's IR line with -v. A Rust port must
fuse exactly these (docs/rust-port/group-1 ...md, "fuse exactly where clang fused").
"""
import argparse, os, re, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WS = ROOT.parents[1]  # <workspace>/worktrees/<wt>
CLANG = os.environ.get("GW_CLANG") or next((str(p) for p in [WS / "_toolchains/llvm/bin/clang.exe",
                                                               Path(r"C:\Program Files\LLVM\bin\clang.exe")] if p.exists()))
GALE = os.environ.get("GALE_INCLUDE") or str((ROOT / "../../melee/build/GALE01/include").resolve())
SHIP = ["--target=ppc32-none-eabi", "-std=c99", "-nostdinc", "-fno-builtin", "-DLINT", "-DTARGET_PC",
        "-fno-short-enums", "-fsigned-char", "-mlong-double-64", "-fno-strict-aliasing", "-fwrapv", "-fcommon",
        "-fgnu89-inline", "-ftrivial-auto-var-init=zero", "-O2", "-Xclang", "-disable-llvm-passes",
        "-emit-llvm", "-S", "-w", "-Isrc", "-Ipc", "-Ipc/gameworld", "-isystem", "src/MSL", "-isystem",
        "libs/dolphin/include", "-isystem", "libs/dolphin/src", "-isystem", GALE,
        "-include", "src/MSL/math_ppc.h"]


def sites(ll):
    out, fn = {}, None
    for line in ll.splitlines():
        m = re.match(r'define .*@"?([\w.$]+)"?\(', line)
        if m:
            fn = m.group(1); out.setdefault(fn, [])
        elif line.startswith("}"):
            fn = None
        elif fn and "llvm.fmuladd" in line:
            out[fn].append(line.strip())
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+")
    ap.add_argument("--func", default=".")
    ap.add_argument("-v", action="store_true")
    a = ap.parse_args()
    with tempfile.TemporaryDirectory() as td:
        for f in a.files:
            o = Path(td) / "x.ll"
            subprocess.run([CLANG, *SHIP, f, "-o", str(o)], cwd=ROOT, check=True)
            for fn, s in sites(o.read_text()).items():
                if not re.search(a.func, fn):
                    continue
                f32 = sum("fmuladd.f32" in x for x in s); f64 = sum("fmuladd.f64" in x for x in s)
                print(f"{f} {fn} {f32} {f64}")
                if a.v:
                    for x in s:
                        print("    " + x)


if __name__ == "__main__":
    main()
