#!/usr/bin/env python3
"""C oracle for the per-TU ports in rust/melee_modes/src/tu/ (one driver per game TU).

  python rust/melee_modes/gen/cref/tu/run_tu.py <stem> [...]    (from the worktree root)

Each gen/cref/tu/<stem>.c `#include`s the game's own .c file verbatim (first line of the driver:
/* TU: src/... */, whose directory goes on the include path), defines recording stubs for every
callee out of the TU, builds synthetic game state from a seeded generator and prints one line per
case. Built exactly like melee_fighter/gen/cref/run.py (shipping contraction -ffp-contract=on,
gwtool lowering via lower.py). Output: rust/melee_modes/tests/data/tu_<stem>.txt, computed from C
source only (no disc data).
"""
import subprocess, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[4]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
sys.path.insert(0, str(HERE))
import run as R  # noqa: E402

R.DATA = ROOT / "rust/melee_modes/tests/data"
# Same LLP64 layout as the msvc target but Itanium bitfield packing, which puts the fields after a
# bitfield run at the GameCube offsets melee_types uses (as rust/melee_modes/oracle/build_ptr.sh).
# 32-bit as well: pointers are 4 bytes like the PPC's, so a record's field offsets match the GameCube
# layout, and C that indexes records by byte offset (gm_16A2's lbl_8046B488_t) means what it means on the
# game. melee_types' own records are native 64-bit; the oracle compares by value (hsgen.py), not by bytes.
R.FLAGS = [a.replace("x86_64-pc-windows-msvc", "i686-pc-windows-gnu") for a in R.FLAGS] + ["-mno-ms-bitfields"]


def main():
    R.BUILD.mkdir(parents=True, exist_ok=True)
    R.DATA.mkdir(parents=True, exist_ok=True)
    for n in sys.argv[1:]:
        drv = HERE / f"{n}.c"
        if "/* HS:" in drv.read_text(encoding="utf-8"):
            import hsgen
            hsgen.main(n)
        if "/* OPS:" in drv.read_text(encoding="utf-8"):
            import opsgen
            opsgen.main(n)
        if "/* STUBS:" in drv.read_text(encoding="utf-8"):
            import stubgen
            stubgen.main(n)
        first = drv.read_text(encoding="utf-8").splitlines()[0]
        src = first.split("TU:")[1].split("*/")[0].strip()
        exe = R.BUILD / f"tugm_{n}.exe"
        lls = []
        for s in (drv, R.ROOT / "pc/gameworld/gekko_fp.c"):
            ll = R.BUILD / f"tugm_{n}__{s.stem}.ll"
            lo = R.BUILD / f"tugm_{n}__{s.stem}.lo.ll"
            R.run([R.CLANG, *R.FLAGS, "-I", str(HERE), "-I", str((R.ROOT / src).parent),
                   "-DTU_SRC=\"" + str(R.ROOT / src).replace("\\", "/") + "\"", str(s), "-o", str(ll)], n)
            R.run([sys.executable, str(R.LOWER), str(ll), str(lo)], n)
            lls.append(str(lo))
        R.run([R.CLANG, "--target=i686-pc-windows-msvc", "-O0", "-w", *lls, "-llegacy_stdio_definitions", "-o", str(exe)], n)
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        (R.DATA / f"tu_{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"cref tu: {n}: {len(out.splitlines())} lines")


if __name__ == "__main__":
    main()
