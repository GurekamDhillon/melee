#!/usr/bin/env python3
"""Build and run a pure-function C oracle driver with the SHIPPING build's float contraction.

  python rust/melee_runtime/gen/pure/run_pure.py <stem>      (from the worktree root; Windows, LLVM)

Compiles rust/melee_runtime/gen/pure/drv_<stem>.c (it #includes the game's .c verbatim) natively
(clang -ffp-contract=on -O2 -Xclang -disable-llvm-passes, as gen/cref/run_tu.py), applies gwtool's IR
rewrites (oracle/lower.py), links, runs; writes rust/melee_runtime/tests/data/pure_<stem>.txt.
"""
import os, re, subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
CLANG = os.environ.get("CLANG", r"C:\Program Files\LLVM\bin\clang.exe")
GALE = os.environ.get("GALE_INCLUDE", str((ROOT / "../../melee/build/GALE01/include").resolve()))
BUILD = ROOT / "rust/target/pure"
DATA = ROOT / "rust/melee_runtime/tests/data"
LOWER = ROOT / "rust/melee_runtime/oracle/lower.py"
FLAGS = ["--target=x86_64-pc-windows-gnu", "-mno-ms-bitfields", "-std=c99", "-nostdinc", "-fno-builtin",
         "-DLINT", "-DTARGET_PC", "-fno-short-enums", "-fsigned-char", "-mlong-double-64",
         "-U__UINTPTR_TYPE__", "-D__UINTPTR_TYPE__=unsigned int", "-U__INTPTR_TYPE__", "-D__INTPTR_TYPE__=int",
         "-fno-strict-aliasing", "-fwrapv", "-fcommon", "-fgnu89-inline", "-ffp-contract=on",
         "-fno-fast-math", "-O2", "-Xclang", "-disable-llvm-passes", "-ftrivial-auto-var-init=zero",
         "-S", "-emit-llvm", "-w", "-D_Static_assert(a,b)=", "-Dsize_t=gc_size_t",
         "-I.", "-Isrc", "-Ipc", "-Ipc/gameworld", "-Isrc/melee",
         "-isystem", "tools/rust-port/include", "-isystem", "src/MSL", "-isystem", "libs/dolphin/include",
         "-isystem", "libs/dolphin/src", "-isystem", GALE, "-include", "src/MSL/math_ppc.h"]


if os.environ.get("PURE_DEBUG"):
    FLAGS += ["-g", "-gcodeview"]


def run(cmd):
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True)
    if r.returncode:
        sys.stderr.write(r.stdout + r.stderr)
        raise SystemExit("pure: failed: " + " ".join(map(str, cmd[:3])))
    return r


def main():
    BUILD.mkdir(parents=True, exist_ok=True)
    DATA.mkdir(parents=True, exist_ok=True)
    for n in sys.argv[1:]:
        drv = HERE / f"drv_{n}.c"
        link, plain = [], []
        for ln in drv.read_text().splitlines()[:12]:
            if "LINK:" in ln:
                link += ln.split("LINK:")[1].replace("*/", "").split()
            if "PLAIN:" in ln:
                plain += ln.split("PLAIN:")[1].replace("*/", "").split()
        # `/* EXTRACT: <c file> <name> ... */` lines: the verbatim definitions of those functions are
        # written to extract_<stem>.h (which the driver includes), for files that cannot be compiled whole.
        ext_txt = ""
        for ln in drv.read_text().splitlines()[:16]:
            if "EXTRACT:" in ln:
                toks = ln.split("EXTRACT:")[1].replace("*/", "").split()
                src_t = (ROOT / toks[0]).read_text(encoding="latin1")
                for nm in toks[1:]:
                    m = re.search(r"^[A-Za-z_][\w \*]*?" + re.escape(nm) + r"\s*\([^;{}]*\)\s*\n?\{", src_t, re.M)
                    if not m:
                        raise SystemExit(f"pure: {nm} not found in {toks[0]}")
                    end = src_t.find("\n}\n", m.start())
                    ext_txt += src_t[m.start(): end + 3] + "\n"
        (HERE / f"extract_{n}.h").write_text(ext_txt, encoding="utf-8")
        lls = []
        for s in (drv, ROOT / "pc/gameworld/gekko_fp.c", *[ROOT / x for x in link]):
            ll = BUILD / f"pure_{n}__{s.parent.name}_{s.stem}.ll"
            lo = BUILD / f"pure_{n}__{s.parent.name}_{s.stem}.lo.ll"
            run([CLANG, *FLAGS, "-I", str(HERE), str(s), "-o", str(ll)])
            run([sys.executable, str(LOWER), str(ll), str(lo)])
            lls.append(str(lo))
        exe = BUILD / f"pure_{n}.exe"
        run([CLANG, "-O0", "-w", *lls, *[str(ROOT / x) for x in plain], "-o", str(exe), "-Wl,/OPT:NOICF", "-fuse-ld=lld"] + (["-g", "-Wl,/DEBUG"] if os.environ.get("PURE_DEBUG") else []))
        pr = subprocess.run([str(exe)], capture_output=True, text=True)
        if pr.returncode:
            raise SystemExit(f"pure: {n}: driver exited {pr.returncode}")
        (DATA / f"pure_{n}.txt").write_text(pr.stdout, encoding="utf-8", newline="\n")
        print(f"pure: {n}: {len(pr.stdout.splitlines())} lines")


if __name__ == "__main__":
    main()
