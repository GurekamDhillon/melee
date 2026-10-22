#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 369-376 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard369.py [name ...]     (worktree root)

Same mechanism as tu_shard89.py: reuses group 3's rust/melee_fighter/gen/cref/run.py build
(shipping contraction + gwtool lowering, game source cut verbatim through EXTRACTS), pointed at
this directory's <name>_ref.c drivers and at rust/melee_chara/tests/data.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

LBV = [("src/melee/lb/lbvector.c", "fn:lbVector_Diff", None),
       ("src/melee/lb/lbvector.c", "fn:lbVector_Normalize", None)]
MINE = {
    "tu_ftmasterhandpoke": LBV,
    "tu_ftmasterhandfingergun": LBV,
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
