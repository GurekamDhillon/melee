#!/usr/bin/env python3
"""C references for the gm TUs of game_tus.txt lines 537-544 (rust/melee_modes/tests/tu_<stem>.rs).

  python rust/melee_modes/gen/turef/tu_shard537_gm.py [name ...]     (worktree root)

Same build as tu_shard457_gm.py (group 3's rust/melee_fighter/gen/cref/run.py: shipping contraction, gwtool's
lowering). Each driver includes its game TU verbatim with the callees it reaches stubbed (logged), and
writes its state lines to rust/melee_modes/tests/data. run.py itself is untouched.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

MINE = {n: [] for n in ("tu_gm_1A33", "tu_gm_1A36", "tu_gm_1A3F", "tu_gm_1A7A", "tu_gm_1A9B", "tu_gm_1ADD",
                        "tu_gmscmemcard", "tu_gm_1B03")}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_modes/tests/data"

if __name__ == "__main__":
    names = sys.argv[1:] or sorted(n for n in MINE if (HERE / f"{n}_ref.c").exists())
    sys.argv = [sys.argv[0], *names]
    run.main()
