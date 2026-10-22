/* C reference for rust/melee_chara/src/tu/ftzeldaspecialn.rs: the game's ftzeldaspecialn.c (verbatim,
 * with its own shipping contraction) on the state oracle of tu_zelda_common.h. The group-3 callees
 * of the TU that the Rust port calls directly (ftCommon_Fall, CalcSelfAccel_*, 8007D5D4/D7FC) are cut
 * from ftcommon.c by gen/turef/tu_shard457.py. Run: python rust/melee_chara/gen/turef/tu_shard457.py */
#include "../../../../src/melee/ft/kinds/ftZelda/ftzeldaspecialn.c"
#include "tu_ftzeldaspecialn_extract.inc"

ftCommonData* p_ftCommonData;

/* TARGET_PC script hook (pc/gameworld/script_fighter_mod.inc): no script mods in this oracle. */
float ScriptGame_ModValue(int slot, int field, float base)
{
    (void) slot;
    (void) field;
    return base;
}

void ftZd_Init_801393AC(HSD_GObj* gobj);

typedef struct {
    void* p;
    const char* n;
} CbEntry;

static const CbEntry CB_TABLE[] = {
    {NULL, "0"},
    {(void*) ftZd_SpecialN_8013A830, "nA830"},
    {(void*) ftZd_SpecialN_8013A8AC, "nA8AC"},
    {(void*) ftZd_SpecialN_8013ADB0, "nADB0"},
    {(void*) ftZd_Init_801393AC, "init"},
    {(void*) efLib_PauseAll, "pause"},
    {(void*) efLib_ResumeAll, "resume"},
};
#define CB_COUNT ((int) (sizeof CB_TABLE / sizeof CB_TABLE[0]))

static const char* FUNC_NAMES[] = {
    "ftZd_SpecialN_8013A830", "ftZd_SpecialN_8013A8AC", "ftZd_SpecialN_Enter", "ftZd_SpecialAirN_Enter",
    "ftZd_SpecialN_Anim", "ftZd_SpecialAirN_Anim", "ftZd_SpecialN_Coll", "ftZd_SpecialAirN_Coll",
    "ftZd_SpecialN_8013AC88", "ftZd_SpecialN_8013AD1C", "ftZd_SpecialAirN_Phys",
};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))

static int g_ret; /* return value of the function under test (0 for void) */

static void call_fn(int f, HSD_GObj* g)
{
    switch (f) {
    case 0: ftZd_SpecialN_8013A830(g); break;
    case 1: ftZd_SpecialN_8013A8AC(g); break;
    case 2: ftZd_SpecialN_Enter(g); break;
    case 3: ftZd_SpecialAirN_Enter(g); break;
    case 4: ftZd_SpecialN_Anim(g); break;
    case 5: ftZd_SpecialAirN_Anim(g); break;
    case 6: ftZd_SpecialN_Coll(g); break;
    case 7: ftZd_SpecialAirN_Coll(g); break;
    case 8: ftZd_SpecialN_8013AC88(g); break;
    case 9: ftZd_SpecialN_8013AD1C(g); break;
    case 10: ftZd_SpecialAirN_Phys(g); break;
    }
}

#include "tu_zelda_common.h"

int main(void)
{
    run_all();
    return 0;
}
