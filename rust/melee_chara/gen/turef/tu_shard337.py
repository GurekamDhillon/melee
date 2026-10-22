#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 337-344 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard337.py [name ...]     (worktree root)

Reuses group 3's rust/melee_fighter/gen/cref/run.py build (shipping contraction + gwtool's
lowering, game source cut verbatim through EXTRACTS); only points it at this directory's
<name>_ref.c drivers and at rust/melee_chara/tests/data. run.py itself is untouched.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

# atan2f is in the first region, atanf (which it calls) in the second.
TRIG = [("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf("),
        ("src/melee/lb/lbtrigf.c", "#define SILVER_RATIO_1_CONJUGATE", None)]
LK = "src/melee/ft/kinds/ftLink/"
MINE = {
    "tu_ftlink": [("src/melee/ft/kinds/ftCommon/ftCo_Guard.c", "fn:ftCo_80092ED8", None),
                  (LK + "ftlink.c", "fn:ftLk_800EB334", None)],
    "tu_ftlinkspecials": [*TRIG, (LK + "ftlinkspecials.c", "fn:calcAnglePos", None)],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
