#!/usr/bin/env python3
"""C references for the gm TUs of game_tus.txt lines 457-464 (rust/melee_modes/tests/tu_<stem>.rs).

  python rust/melee_modes/gen/turef/tu_shard457_gm.py [name ...]     (worktree root)

Same build as tu_shard457.py (group 3's rust/melee_fighter/gen/cref/run.py: shipping contraction, gwtool's
lowering). Each driver includes its game TU verbatim with the callees it reaches stubbed (logged), and
writes its state lines to rust/melee_modes/tests/data. run.py itself is untouched.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

MINE = {n: [] for n in ("tu_gmboot", "tu_gm_1BFA")}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_modes/tests/data"

# Drivers whose structs hold u32 bitfield runs (StartMeleeRules ...) must be laid out like the Rust records (and the
# PPC): bitfields packed, not MSVC-style whole allocation units. They are compiled for the MinGW (Itanium) front end
# with -mno-ms-bitfields (which also evaluates call arguments left to right, as the PPC build does) and linked for
# the MSVC target as before.
ITANIUM = {"tu_gm_1BFA"}
_orig_run = run.run


def _run(cmd, n):
    if n in ITANIUM:
        cmd = list(cmd)
        if "-S" in cmd:
            cmd = [c.replace("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu") for c in cmd] + ["-mno-ms-bitfields"]
        elif cmd[0] == run.CLANG and "-O0" in cmd:
            cmd.insert(1, "--target=x86_64-pc-windows-msvc")
    return _orig_run(cmd, n)


run.run = _run

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
