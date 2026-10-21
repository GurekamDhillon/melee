#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 281-288 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard281.py [name ...]     (worktree root)

Same mechanism as tu_shard89.py: reuses group 3's rust/melee_fighter/gen/cref/run.py build
(shipping contraction + gwtool's lowering, game source cut verbatim through EXTRACTS), pointed at
this directory's <name>_ref.c drivers and at rust/melee_chara/tests/data.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

DK = "src/melee/ft/kinds/ftDonkey/"
FX = "src/melee/ft/kinds/ftFox/"
MINE = {
    "tu_ftdonkeyspecialn": [
        (DK + "ftdonkeyspecialn.c", "fn:" + n, None) for n in (
            "setCallbacks", "updateVelocity", "clearCallbacks", "ftDk_SpecialN_Enter",
            "ftDk_SpecialAirN_Enter", "ftDk_SpecialNLoop_Anim", "ftDk_SpecialN_Anim",
            "ftDk_SpecialNFull_Anim", "ftDk_SpecialAirNLoop_Anim", "ftDk_SpecialAirN_Anim",
            "ftDk_SpecialAirNFull_Anim", "ftDk_SpecialNLoop_IASA")],
    "tu_ftdonkeyspecials": [
        (DK + "ftdonkeyspecials.c", "fn:ftDk_SpecialAirS_Enter", None),
        (DK + "ftdonkeyspecials.c", "fn:ftDk_SpecialAirS_Phys", None),
        ("src/melee/ft/ftcommon.c", "fn:ftCommon_CalcSelfAccel_Deaccel", None),
        ("src/melee/ft/ftcommon.c", "fn:ftCommon_Fall", None)],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
