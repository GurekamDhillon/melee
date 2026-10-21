#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 89-92 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard89.py [name ...]     (worktree root)

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

TRIG = [("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf(")]
CA = "src/melee/ft/kinds/ftCaptain/"
MINE = {
    "tu_ftcaptainspecialn": [*TRIG,
                             (CA + "ftcaptainspecialn.c", "fn:ftCaptain_SpecialN_GetAngleVel", None),
                             (CA + "ftcaptainspecialn.c", "fn:ftCa_SpecialAirN_IASA", None)],
    "tu_ftcaptainspecials": [(CA + "ftcaptainspecials.c", "fn:ftCa_SpecialAirSStart_Phys", None),
                             (CA + "ftcaptainspecials.c", "fn:ftCa_SpecialAirS_Phys", None)],
    "tu_ftclink": [("src/melee/ft/kinds/ftCLink/ftclink.c", "fn:ftCl_Init_80149114", None)],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
