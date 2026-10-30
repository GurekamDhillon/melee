#!/usr/bin/env python3
"""run.py for the per-TU references (gen/cref/tu_<stem>_ref.c): same flags and lowering, but the
reference #includes the whole game TU, so its unreferenced-at-runtime callees are left
unresolved (/FORCE:UNRESOLVED; the drivers never reach them).

  python rust/melee_fighter/gen/cref/tu_run.py tu_ftbosslib [...]
"""
import subprocess, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import run as R


def main():
    R.BUILD.mkdir(parents=True, exist_ok=True)
    R.DATA.mkdir(parents=True, exist_ok=True)
    for n in sys.argv[1:]:
        if n in R.EXTRACTS:  # functions cut verbatim from other game files into <n>_extract.inc (run.py EXTRACTS)
            R.extract(n)
        exe = R.BUILD / f"{n}_ref.exe"
        lls = []
        for src in (R.HERE / f"{n}_ref.c", R.ROOT / "pc/gameworld/gekko_fp.c"):
            ll = R.BUILD / f"{n}__{src.stem}.ll"
            lo = R.BUILD / f"{n}__{src.stem}.lo.ll"
            R.run([R.CLANG, *R.FLAGS, str(src), "-o", str(ll)], n)
            R.run([sys.executable, str(R.LOWER), str(ll), str(lo)], n)
            lls.append(str(lo))
        R.run([R.CLANG, "-O0", "-w", *lls, "-Wl,/FORCE:UNRESOLVED", "-o", str(exe)], n)
        out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
        (R.DATA / f"{n}.txt").write_text(out, encoding="utf-8", newline="\n")
        print(f"cref: {n}: {len(out.splitlines())} lines -> {R.DATA / (n + '.txt')}")


if __name__ == "__main__":
    main()
