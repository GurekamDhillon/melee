#!/usr/bin/env python3
"""Build and run the C oracle driver of one melee_items::tu port (run_tu.py retargeted).

  python rust/melee_items/gen/mtu_it2/run_it.py <stem> [...]      (from the worktree root; Windows, LLVM)

Regenerates gen/mtu_it2/tu/<stem>_gen.h and tests/mtu/<stem>_gen.rs (gen_it.py), compiles
gen/mtu_it2/tu/<stem>.c (the driver written by mk_it_driver.py, which #includes the game's .c verbatim)
natively with the SHIPPING build's float contraction (clang default -ffp-contract=on, -O2
-Xclang -disable-llvm-passes, -ftrivial-auto-var-init=zero), redoes gwtool's IR rewrites
(melee_runtime/oracle/lower.py), links and runs it. Output: rust/melee_items/tests/data/mtu_<stem>.txt
(computed from the C source only).
"""
import os, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(HERE))
import gen_it  # noqa: E402
gen_tu = gen_it.gen_tu
W = ROOT / "rust/melee_world/gen/mtu"
CLANG = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
GALE = os.environ.get("GALE_INCLUDE", str((ROOT / "../../melee/build/GALE01/include").resolve()))
BUILD = ROOT / "rust/target/mtu_it2"
DATA = ROOT / "rust/melee_items/tests/data"
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
    from specs_it import SPECS
    for n in sys.argv[1:]:
        spec = SPECS[n]
        gen_it.build(n)
        drv = HERE / "tu" / f"{n}.c"
        src = gen_tu.patched_source(n, spec)
        exe = BUILD / f"mtu_{n}.exe"
        dbg = []
        if os.environ.get("MTU_DBG"):
            dop, dk = os.environ["MTU_DBG"].split(":")
            dbg = [f"-DDBG_OP={dop}", f"-DDBG_K={dk}"]
        lls = []
        for s in (drv, ROOT / "pc/gameworld/gekko_fp.c"):
            ll = BUILD / f"mtu_{n}__{s.stem}.ll"
            lo = BUILD / f"mtu_{n}__{s.stem}.lo.ll"
            run([CLANG, *FLAGS, "-I", str((ROOT / spec["src"]).parent), "-I", str(HERE / "tu"), "-I", str(W / "tu"),
                 '-DTU_SRC="' + str(ROOT / src).replace("\\", "/") + '"', *dbg, str(s), "-o", str(ll)], n)
            run([sys.executable, str(LOWER), str(ll), str(lo)], n)
            lls.append(str(lo))
        run([CLANG, "-O0", "-w", *lls, "-o", str(exe), "-Wl,/OPT:NOICF", "-fuse-ld=lld"] + (["-g", "-Wl,/DEBUG:SYMTAB"] if os.environ.get("MTU_G") else []), n)
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        if dbg:
            tr = [l for l in out.splitlines() if l.startswith(("T ", "H ", "B "))]
            (BUILD / f"dbg_{n}.txt").write_text(chr(10).join(tr) + chr(10), encoding="utf-8")
            print(f"mtu: {n}: {len(tr)} trace words")
            continue
        DATA.mkdir(parents=True, exist_ok=True)
        (DATA / f"mtu_{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"mtu: {n}: {len(out.splitlines())} lines")


if __name__ == "__main__":
    main()
