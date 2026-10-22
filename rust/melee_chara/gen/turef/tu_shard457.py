#!/usr/bin/env python3
"""C references for the Zelda TUs of game_tus.txt lines 457-464 (rust/melee_chara/tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard457.py [name ...]     (worktree root)

Same build as tu_shard305.py (group 3's rust/melee_fighter/gen/cref/run.py: shipping contraction, gwtool's
lowering). Each driver includes its game TU verbatim and tu_zelda_common.h (state oracle); the group-3
ftCommon.CalcSelfAccel_* callees are cut from ftcommon.c below; ftCommon_Fall / 8007D5D4 / 8007D7FC are
logging stubs (hookable in Rust, see tu/ftzeldaspecialn.rs ext). run.py itself is untouched.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

FT = "src/melee/ft/ftcommon.c"
G3 = [(FT, "fn:" + n, None) for n in (
    "ftCommon_CalcSelfAccel_Deaccel",
    "ftCommon_CalcSelfAccel_DeaccelAir",
    "ftCommon_CalcSelfAccel_DeaccelQuickAir",
)]
MINE = {n: G3 for n in ("tu_ftzeldaspecialn", "tu_ftzeldaspeciallw", "tu_ftzeldaspecials")}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
