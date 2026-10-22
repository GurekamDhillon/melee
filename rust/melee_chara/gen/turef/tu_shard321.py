#!/usr/bin/env python3
"""C references for the ftKirby special TUs (game_tus.txt lines 321-328; tests/tu_ftkirbyspecial*.rs).

  python rust/melee_chara/gen/turef/tu_shard321.py [name ...]     (worktree root)

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
KB = "src/melee/ft/kinds/ftKirby/"


def fns(src, *names):
    return [(KB + src, "fn:" + n, None) for n in names]


MINE = {
    "tu_ftkirbyspecialn": [*TRIG, (KB + "ftkirbyspecialn.c", "float ftKb_SpecialN_800F58D8(", "void ftKb_SpecialN_800F5954("), *fns(
        "ftkirbyspecialn.c", "ftKb_SpecialN_800F5B5C", "ftKb_SpecialN_800F5DE8",
        "ftKb_SpecialN_800F5EA8", "ftKb_SpecialNCapture_EnterState", "ftKb_SpecialNCapture0_Anim",
        "ftKb_SpecialNCapture_Anim")],
    "tu_ftkirbyspecialpikachu": [*fns("ftkirbyspecialpikachu.c", "ftKb_PkSpecialN_Anim", "ftKb_PkSpecialAirN_Anim")],
    "tu_ftkirbyspecialness": [*fns("ftkirbyspecialness.c", "ftKb_NsSpecialNStart_Anim_inline")],
    "tu_ftkirbyspecialmewtwo": [*fns(
        "ftkirbyspecialmewtwo.c", "ftKb_SpecialNMt_80107040", "ftKb_SpecialNMt_SetRecoil", "ftKb_SpecialNMt_801071FC",
        "ftKb_SpecialNMt_80107410", "ftKb_MtSpecialNStart_Anim"),
        (KB + "ftkirbyspecialmewtwo.c", "static inline void ftKb_SpecialNMt_CreateHeldShadow(", "void ftKb_MtSpecialNLoop_Anim("),
        *fns("ftkirbyspecialmewtwo.c", "ftKb_MtSpecialNLoop_Anim")],
    "tu_ftkirbyspecialsamus": [*fns(
        "ftkirbyspecialsamus.c", "ftKb_SpecialNSs_it_802B5974", "ftKb_SpecialNSs_DestroyChargeShot", "ftKb_ChargeShot_inline",
        "ftKb_SpecialNSs_800FCDE0_inline", "facing_to_angle", "ftKb_SpecialNSs_UnsetChargeShot", "ftKb_SpecialNSs_800FCDE0",
        "ftKb_SsSpecialNStart_Anim", "ftKb_SsSpecialNHold_Anim", "ftKb_SsSpecialAirNStart_Anim")],
    "tu_ftkirbyspecialpurin": [
        *TRIG, ("src/melee/lb/lbtrigf.c", "#define SILVER_RATIO_1_CONJUGATE", None),
        (KB + "ftkirbyspecialpurin.c", "static inline void ftKb_PrScaleAnimStep(", None)],
}
run.EXTRACTS.update(MINE)
run.HERE = HERE
run.DATA = ROOT / "rust/melee_chara/tests/data"

if __name__ == "__main__":
    sys.argv = [sys.argv[0], *(sys.argv[1:] or sorted(MINE))]
    run.main()
