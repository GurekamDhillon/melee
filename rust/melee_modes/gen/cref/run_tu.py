#!/usr/bin/env python3
"""Build and run the C oracle driver of one melee_modes::tu port.

  python rust/melee_modes/gen/cref/run_tu.py <stem> [...]        (from the worktree root; Windows, LLVM)

Regenerates gen/cref/tu/<stem>_gen.h and tests/mtu/<stem>_gen.rs (gen_tu.py), compiles
gen/cref/tu/<stem>.c (which #includes the game's .c verbatim) natively with the SHIPPING build's
float contraction (clang default -ffp-contract=on, -O2 -Xclang -disable-llvm-passes,
-ftrivial-auto-var-init=zero), redoes gwtool's IR rewrites (melee_runtime/oracle/lower.py), links
and runs it. Output: rust/melee_modes/tests/data/mtu_<stem>.txt (computed from C source only).
"""
import os, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
CLANG = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
GALE = os.environ.get("GALE_INCLUDE", str((ROOT / "../../melee/build/GALE01/include").resolve()))
BUILD = ROOT / "rust/target/mtu"
DATA = ROOT / "rust/melee_modes/tests/data"
LOWER = ROOT / "rust/melee_runtime/oracle/lower.py"
FLAGS = ["--target=x86_64-pc-windows-gnu", "-mno-ms-bitfields", "-std=c99", "-nostdinc", "-fno-builtin",
         "-DLINT", "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-mlong-double-64",
         "-fno-strict-aliasing", "-fwrapv", "-fcommon", "-fgnu89-inline", "-ffp-contract=on",
         "-fno-fast-math", "-O2", "-Xclang", "-disable-llvm-passes", "-ftrivial-auto-var-init=zero",
         "-S", "-emit-llvm", "-w", "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t",
         "-I.", "-Isrc", "-Ipc", "-Ipc/gameworld", "-Isrc/melee",
         "-isystem", "tools/rust-port/include", "-isystem", "src/MSL", "-isystem", "libs/dolphin/include",
         "-isystem", "libs/dolphin/src", "-isystem", GALE, "-include", "src/MSL/math_ppc.h"]


def run(cmd, n):
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(r.stdout + r.stderr)
        raise SystemExit(f"mtu: {n} failed: {' '.join(map(str, cmd[:3]))}")
    return r


def main():
    BUILD.mkdir(parents=True, exist_ok=True)
    DATA.mkdir(parents=True, exist_ok=True)
    from specs import SPECS
    import gen_tu
    for n in sys.argv[1:]:
        spec = SPECS[n]
        gen_tu.build(n, spec, False)
        drv = HERE / "tu" / f"{n}.c"
        src = spec["src"]
        exe = BUILD / f"mtu_{n}.exe"
        lls = []
        for s in (drv, ROOT / "pc/gameworld/gekko_fp.c"):
            ll = BUILD / f"mtu_{n}__{s.stem}.ll"
            lo = BUILD / f"mtu_{n}__{s.stem}.lo.ll"
            run([CLANG, *FLAGS, "-I", str((ROOT / src).parent), "-I", str(HERE / "tu"),
                 '-DTU_SRC="' + str(ROOT / src).replace("\\", "/") + '"', str(s), "-o", str(ll)], n)
            run([sys.executable, str(LOWER), str(ll), str(lo)], n)
            lls.append(str(lo))
        run([CLANG, "-O0", "-w", *lls, "-o", str(exe), "-Wl,/OPT:NOICF", "-fuse-ld=lld"], n)
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        (DATA / f"mtu_{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"mtu: {n}: {len(out.splitlines())} lines")


if __name__ == "__main__":
    main()
