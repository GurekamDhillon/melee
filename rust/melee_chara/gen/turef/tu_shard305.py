#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 305-312 (ftKirby; tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard305.py [name ...]     (worktree root)

Same build as tu_shard89.py (group 3's rust/melee_fighter/gen/cref/run.py: shipping contraction + gwtool's
lowering, game source cut verbatim through EXTRACTS), pointed at this directory's <name>_ref.c drivers
and at rust/melee_chara/tests/data. run.py itself is untouched.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

TRIG = [("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf(")]
KB = "src/melee/ft/kinds/ftKirby/"
MINE = {
    "tu_ftkirbyspecialcaptain": [*TRIG, (KB + "ftkirbyspecialcaptain.c", "fn:ftKb_CaSpecialAirN_IASA", None)],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
