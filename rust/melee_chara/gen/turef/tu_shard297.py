#!/usr/bin/env python3
"""C references for the Mr. Game & Watch TUs of game_tus.txt lines 297-304 (tests/tu_ftgamewatch.rs).

  python rust/melee_chara/gen/turef/tu_shard297.py [name ...]     (worktree root)

Same build as tu_shard89.py (group 3's gen/cref/run.py: shipping contraction + gwtool's lowering,
game source cut verbatim through EXTRACTS) pointed at this directory's <name>_ref.c drivers.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

GW = "src/melee/ft/kinds/ftGameWatch/"
MINE = {
    "tu_ftgamewatch": [
        (GW + "ftgamewatchspeciallw.c", "fn:ftGw_SpecialLw_UpdateBucketModel", None),
        (GW + "ftgamewatchspeciallw.c", "fn:ftGw_SpecialLw_IASA", None),
        (GW + "ftgamewatchspeciallw.c", "fn:ftGw_SpecialAirLw_IASA", None),
        (GW + "ftgamewatchspeciallw.c", "fn:ftGw_SpecialLwShoot_ReleaseOil", None),
        (GW + "ftgamewatchspeciallw.c", "fn:ftGw_SpecialAirLwShoot_ReleaseOil", None),
        (GW + "ftgamewatchspecialhi.c", "fn:ftGw_SpecialHi_ItemRescueSetup", None),
        (GW + "ftgamewatchspecialhi.c", "fn:ftGw_SpecialAirHi_IASA", None),
        (GW + "ftgamewatchspecialn.c", "fn:ftGw_SpecialN_CreateSausage", None),
        (GW + "ftgamewatchspecialn.c", "fn:ftGw_SpecialN_Loop", None),
        (GW + "ftgamewatchspecialn.c", "fn:ftGw_SpecialAirN_Loop", None),
        (GW + "ftgamewatchspecialn.c", "fn:ftGw_SpecialN_IASA", None),
        (GW + "ftgamewatchspecialn.c", "fn:ftGw_SpecialAirN_IASA", None),
        (GW + "ftgamewatchspecials.c", "fn:ftGw_SpecialS_GetRandomInt", None),
        (GW + "ftgamewatchspecials.c", "fn:ftGw_SpecialS_ItemJudgementSetup", None),
    ],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
