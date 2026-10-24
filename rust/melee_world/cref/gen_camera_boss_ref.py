#!/usr/bin/env python3
"""Build and run the C reference for melee_world::cm::camera_boss; writes tests/data/camera_boss_ref.txt.

  python rust/melee_world/cref/gen_camera_boss_ref.py          (from the worktree root; Windows, LLVM clang)

Uses the melee_fighter C-reference toolchain (gen/cref/run.py: native x86_64, the shipping float
contraction, gwtool's lowering redone by melee_runtime/oracle/lower.py). The functions under test are cut
verbatim out of src/melee/cm/camera.c (and atan2f out of lb/lbtrigf.c). Computed from C source only.
"""
import importlib.util
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
spec = importlib.util.spec_from_file_location("fighter_cref_run", ROOT / "rust/melee_fighter/gen/cref/run.py")
R = importlib.util.module_from_spec(spec)
spec.loader.exec_module(R)

NAME = "camera_boss"
R.EXTRACTS[NAME] = [
    ("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf("),
    ("src/melee/lb/lbtrigf.c", "#define SILVER_RATIO_1_CONJUGATE", None),
    *[("src/melee/cm/camera.c", "fn:Camera_" + n, None) for n in
      ("8002FE38", "8002DFE4", "8002E158", "8002E234", "8002E6FC", "8002E948", "8002EB5C", "8002EC7C", "8002ED9C",
       "8002EEC8", "8002EF14", "8002F0E4", "8002F260", "8002F274")],
]


def main():
    R.BUILD.mkdir(parents=True, exist_ok=True)
    R.extract(NAME)
    exe = R.BUILD / f"{NAME}_ref.exe"
    lls = []
    for src in (HERE / f"{NAME}_ref.c", R.ROOT / "pc/gameworld/gekko_fp.c"):
        ll = R.BUILD / f"{NAME}__{src.stem}.ll"
        lo = R.BUILD / f"{NAME}__{src.stem}.lo.ll"
        R.run([R.CLANG, *R.FLAGS, "-I", str(HERE), str(src), "-o", str(ll)], NAME)
        R.run([sys.executable, str(R.LOWER), str(ll), str(lo)], NAME)
        lls.append(str(lo))
    R.run([R.CLANG, "-O0", "-w", *lls, "-Wl,/FORCE:UNRESOLVED", "-o", str(exe)], NAME)
    out = subprocess.run([str(exe)], capture_output=True, text=True, check=True).stdout
    dest = HERE.parent / "tests/data/camera_boss_ref.txt"
    dest.write_text(out, encoding="utf-8", newline="\n")
    print(f"camera_boss: {len(out.splitlines())} lines -> {dest}")


if __name__ == "__main__":
    main()
