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

MINE = {n: [] for n in ("tu_gmboot",)}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_modes/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
