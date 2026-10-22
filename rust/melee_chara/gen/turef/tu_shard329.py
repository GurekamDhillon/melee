#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 329-336 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard329.py [name ...]     (worktree root)

Same build as tu_shard89.py (group 3's rust/melee_fighter/gen/cref/run.py: shipping contraction +
gwtool lowering, game source cut verbatim through EXTRACTS); only the extracts and the drivers
(<name>_ref.c next to this file) differ. Output: rust/melee_chara/tests/data/<name>.txt.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

KP = "src/melee/ft/kinds/ftKoopa/"
KB = "src/melee/ft/kinds/ftKirby/"
MINE = {
    # the direction table + ftKp_SpecialN_80134ACC_inline precede the function in the game file
    "tu_ftkoopaspecialn": [(KP + "ftkoopaspecialn.c", "enum_t ftKp_Init_803CF2A0[]", "void ftKp_SpecialLw_80134ACC("),
                           (KP + "ftkoopaspecialn.c", "fn:ftKp_SpecialLw_80134ACC", None)],
}
MINE["tu_ftkirbyspecialseak"] = [
    (KB + "ftkirbyspecialseak.c", "/* 3CB770 */ static float", "void fn_80105FEC("),
    (KB + "ftkirbyspecialseak.c", "bool ftKb_SpecialNSk_80105FF0(", "s32 ftKb_SpecialNSk_80106020("),
    (KB + "ftkirbyspecialseak.c", "fn:ftKb_SpecialNSk_8010603C", None),
    (KB + "ftkirbyspecialseak.c", "// permuter-generated helper", "void fn_80106DB0("),
    (KB + "ftkirbyspecialseak.c", "fn:fn_80106DB0", None),
]
MINE["tu_ftkirbyyoshiegg"] = [(KB + "ftkirbyyoshiegg.c", "fn:fn_8010B16C", None)]
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
