#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 441-448 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard441.py [name ...]     (worktree root)

Same build as tu_shard289.py (group 3's rust/melee_fighter/gen/cref/run.py: shipping
contraction + gwtool's lowering, game source cut verbatim through EXTRACTS) pointed at this
directory's <name>_ref.c drivers and at rust/melee_chara/tests/data.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

SK = "src/melee/ft/kinds/ftSeak/"
MINE = {
    "tu_ftseakspecialhi": [
        (SK + "ftseakspecialhi.c", "fn:ftSk_SpecialHi_80113EAC_inline", None),
        (SK + "ftseakspecialhi.c", "fn:ftSk_SpecialHi_80113EAC", None),
        (SK + "ftseakspecialhi.c", "fn:ftSk_SpecialHi_80113F68_inline", None),
        (SK + "ftseakspecialhi.c", "fn:ftSk_SpecialHi_80113F68", None),
        (SK + "ftseakspecialhi.c", "fn:ftSk_SpecialHi_801137C8", None),
        (SK + "ftseakspecialhi.c", "fn:ftSk_SpecialHiStart_1_Anim", None),
        (SK + "ftseakspecialhi.c", "fn:ftSk_SpecialAirHiStart_1_Anim", None),
    ],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
