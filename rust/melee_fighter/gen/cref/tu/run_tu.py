#!/usr/bin/env python3
"""C oracle for the per-TU ports in rust/melee_fighter/src/tu/ (one driver per game TU).

  python rust/melee_fighter/gen/cref/tu/run_tu.py <tu_stem> [...]   (from the worktree root)

Each gen/cref/tu/<stem>.c `#include`s the game's own .c file verbatim (TU_SRC below; its dir is
put on the include path), defines recording stubs for every callee out of the TU, and prints
one line per case. Built exactly like gen/cref/run.py (shipping contraction, gwtool lowering).
Output: rust/melee_fighter/tests/data/tu_<stem>.txt. Computed from C source only: no disc data.
"""
import subprocess, sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import run as R  # noqa: E402

HERE = Path(__file__).resolve().parent
FTCO = "src/melee/ft/kinds/ftCommon"


def main():
    R.BUILD.mkdir(parents=True, exist_ok=True)
    R.DATA.mkdir(parents=True, exist_ok=True)
    for n in sys.argv[1:]:
        drv = HERE / f"{n}.c"
        first = drv.read_text(encoding="utf-8").splitlines()[0]
        # first line of the driver: /* TU: src/... */
        src = first.split("TU:")[1].split("*/")[0].strip()
        exe = R.BUILD / f"tu_{n}.exe"
        lls = []
        for s in (drv, R.ROOT / "pc/gameworld/gekko_fp.c"):
            ll = R.BUILD / f"tu_{n}__{s.stem}.ll"
            lo = R.BUILD / f"tu_{n}__{s.stem}.lo.ll"
            R.run([R.CLANG, *R.FLAGS, "-I", str((R.ROOT / src).parent), "-DTU_SRC=\"" + str(R.ROOT / src).replace("\\", "/") + "\"",
                   str(s), "-o", str(ll)], n)
            R.run([sys.executable, str(R.LOWER), str(ll), str(lo)], n)
            lls.append(str(lo))
        R.run([R.CLANG, "-O0", "-w", *lls, "-o", str(exe)], n)
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        (R.DATA / f"tu_{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"cref tu: {n}: {len(out.splitlines())} lines")


if __name__ == "__main__":
    main()
