#!/usr/bin/env python3
"""C references for the ftCommon TUs of game_tus.txt lines 97-104 (tests/tu_ftco_*.rs).

  python rust/melee_fighter/gen/cref/tu_shard97.py [name ...]     (worktree root)

Reuses run.py's build (shipping contraction + gwtool lowering); only adds EXTRACTS entries, so
run.py itself (shared) is untouched. Output: rust/melee_fighter/tests/data/<name>.txt.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import run  # noqa: E402

CO = "src/melee/ft/kinds/ftCommon/"
MINE = {
    "tu_ftco_0d67": [(CO + "ftCo_0D67.c", "fn:ftCo_800D67C4", None)],
    "tu_ftco_0d72": [(CO + "ftCo_0D72.c", "fn:ftCo_800D72A0", None)],
    "tu_ftco_0c35": [("src/melee/ft/ftcommon.c", "fn:ftCommon_800804FC", None),
                     (CO + "ftCo_0C35.c", "static void inlineA0(", None)],
    "tu_ftco_0d8e": [(CO + "ftCo_0D8E.c", "typedef struct {", None)],
    "tu_ftco_aircatch": [(CO + "ftCo_AirCatch.c", "fn:ftCo_800C3CC0", None),
                         (CO + "ftCo_AirCatch.c", "fn:ftCo_AirCatch_Phys", None),
                         (CO + "ftCo_AirCatch.c", "fn:ftCo_AirCatchHit_Phys", None),
                         ("src/melee/ft/ftcommon.c", "fn:ftCommon_ClampSelfVelX", None),
                         ("src/melee/ft/ftcommon.c", "fn:ftCommon_Fall", None),
                         ("src/melee/ft/ftcommon.c", "fn:ftCommon_FallFast", None),
                         ("src/melee/ft/ftcommon.c", "fn:ftCommon_CheckFallFast", None),
                         ("src/melee/ft/ftcommon.c", "fn:ftCommon_CalcSelfAccel_Drift", None)],
}
run.EXTRACTS.update(MINE)

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
