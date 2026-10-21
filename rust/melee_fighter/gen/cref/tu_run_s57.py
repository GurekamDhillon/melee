#!/usr/bin/env python3
"""C references for the per-TU ports of game_tus.txt lines 57-64 (src/tu/, tests/tu_<stem>.rs):
the same build as run.py (shipping contraction + lower.py) with their own extract ranges, so
run.py stays untouched (tu_run.py is another shard's runner).

  python rust/melee_fighter/gen/cref/tu_run_s57.py [tu_ftwalkcommon ...]
"""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import run  # noqa: E402

FC = "src/melee/ft/ftcommon.c"
TU_EXTRACTS = {
    "tu_ftwalkcommon": [(FC, "void ftCommon_CalcGroundAccel_Deaccel(", "void ftCommon_UnlockECB("),
                        ("src/melee/ft/ftwalkcommon.c", "FtWalkType ftWalkCommon_GetWalkType(", None)],
    "tu_ftwalljump": [("src/melee/ft/ftwalljump.c", "static int const max_input_frames", None)],
}

if __name__ == "__main__":
    run.EXTRACTS.update(TU_EXTRACTS)
    names = sys.argv[1:] or sorted(TU_EXTRACTS)
    sys.argv = [sys.argv[0], *names]
    run.main()
