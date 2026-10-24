#!/usr/bin/env python3
"""Build and run the C reference for melee_runtime::tu::jobj_ptr; writes tests/data/jobjptr_ref.txt.

  python rust/melee_runtime/cref/gen_jobjptr_ref.py          (from the worktree root; Windows, LLVM clang)

Uses the melee_fighter C-reference toolchain (gen/cref/run.py: native x86_64, the shipping float
contraction, gwtool's lowering redone by melee_runtime/oracle/lower.py). The functions under test are cut
verbatim out of src/sysdolphin/baselib/jobj.c. Computed from C source only.
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

NAME = "jobjptr"
J = "src/sysdolphin/baselib/jobj.c"
R.EXTRACTS[NAME] = [
    (J, "typedef void (*ufc_callback)", "void JObjUpdateFunc("),
    *[(J, "fn:" + n, None) for n in
      ("HSD_JObjCheckDepend", "JObjUpdateFunc", "JObjSortAnim", "HSD_JObjSetFlags", "HSD_JObjSetFlagsAll",
       "HSD_JObjClearFlags", "HSD_JObjClearFlagsAll", "HSD_JObjGetDObj", "HSD_JObjAddDObj", "HSD_JObjSetMtxDirtySub")],
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
    dest = HERE.parent / "tests/data/jobjptr_ref.txt"
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(out, encoding="utf-8", newline="\n")
    print(f"jobjptr: {len(out.splitlines())} lines -> {dest}")


if __name__ == "__main__":
    main()
