#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 289-296 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard289.py [name ...]     (worktree root)

Same mechanism as tu_shard89.py: group 3's rust/melee_fighter/gen/cref/run.py build (shipping
contraction + gwtool's lowering, game source cut verbatim through EXTRACTS), pointed at this
directory's <name>_ref.c drivers and at rust/melee_chara/tests/data.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

TRIG = [("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf("),
        ("src/melee/lb/lbtrigf.c", "#define SILVER_RATIO_1_CONJUGATE", None)]
FX = "src/melee/ft/kinds/ftFox/"
MINE = {
    "tu_ftfoxspecialhi": [*TRIG,
                          ("src/melee/ft/ftcommon.c", "fn:ftCommon_UpdateFacing", None),
                          (FX + "ftfoxspecialhi.c", "fn:ftFox_SpecialHi_RotateModel", None),
                          (FX + "ftfoxspecialhi.c", "fn:ftFx_SpecialAirHi_Phys", None),
                          (FX + "ftfoxspecialhi.c", "fn:ftFx_SpecialAirHi_Enter", None)],
    "tu_ftfoxspeciallw": [(FX + "ftfoxspeciallw.c", "fn:ftFx_SpecialLw_Turn", None),
                          (FX + "ftfoxspeciallw.c", "fn:ftFox_SpecialLw_Turn_Inline", None)],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
