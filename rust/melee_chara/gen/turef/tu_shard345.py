#!/usr/bin/env python3
"""C references for the character TUs of game_tus.txt lines 345-352 (tests/tu_<stem>.rs).

  python rust/melee_chara/gen/turef/tu_shard345.py [name ...]     (worktree root)

Same build as tu_shard89.py (group 3's rust/melee_fighter/gen/cref/run.py: shipping contraction +
gwtool's lowering, game source cut verbatim through EXTRACTS); the drivers #include the game TU
itself and stand in for everything outside it.
"""
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
sys.path.insert(0, str(ROOT / "rust/melee_fighter/gen/cref"))
import run  # noqa: E402

FT = "src/melee/ft/ftcommon.c"
TRIG = [("src/melee/lb/lbtrigf.c", "#define SIGN_BIT", "float acosf("),
        ("src/melee/lb/lbtrigf.c", "#define SILVER_RATIO_1_CONJUGATE", None)]


def cc(*names):
    return [(FT, "fn:" + n, None) for n in names]


MINE = {
    "tu_ftmariospecialhi": cc("ftCommon_Fall", "ftCommon_CalcSelfAccel_DeaccelQuickAir", "ftCommon_UpdateFacing"),
    "tu_ftluigispecialhi": cc("ftCommon_Fall", "ftCommon_CalcSelfAccel_DeaccelQuickAir", "ftCommon_UpdateFacing"),
    "tu_ftmariospecialn": [],
    "tu_ftmariospeciallw": [*TRIG,
                            *cc("ftCommon_Fall", "ftCommon_FallBasic", "ftCommon_ClampSelfVelX", "ftCommon_ClampFallSpeed",
                                "ftCommon_ClampGroundVel", "ftCommon_Ascend", "ftCommon_SetSelfMovementFromGroundedMovement",
                                "ftCommon_8007D5D4"),
                            (FT, "void ftCommon_CalcGroundAccel_AccelToVel(", "static inline void"),
                            (FT, "void ftCommon_CalcSelfAccel_AccelToVel(", "void ftCommon_CalcSelfAccel_DriftSimple("),
                            (FT, "void ftCommon_CalcSelfAccel_DriftSimple_NoFriction(", "void ftCommon_ClampSelfVelX(")],
    "tu_ftluigispeciallw": [*TRIG,
                            *cc("ftCommon_Fall", "ftCommon_FallBasic", "ftCommon_ClampSelfVelX", "ftCommon_ClampFallSpeed",
                                "ftCommon_ClampGroundVel", "ftCommon_Ascend", "ftCommon_SetSelfMovementFromGroundedMovement",
                                "ftCommon_8007D5D4"),
                            (FT, "void ftCommon_CalcGroundAccel_AccelToVel(", "static inline void"),
                            (FT, "void ftCommon_CalcSelfAccel_AccelToVel(", "void ftCommon_CalcSelfAccel_DriftSimple("),
                            (FT, "void ftCommon_CalcSelfAccel_DriftSimple_NoFriction(", "void ftCommon_ClampSelfVelX(")],
    "tu_ftmariospecials": cc("ftCommon_Fall", "ftCommon_FallBasic", "ftCommon_CalcSelfAccel_Deaccel"),
    "tu_ftluigispecials": cc("ftCommon_Fall", "ftCommon_CalcSelfAccel_Deaccel", "ftCommon_CalcGroundAccel_Deaccel",
                           "ftCommon_SetSelfMovementFromGroundedMovement"),
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
