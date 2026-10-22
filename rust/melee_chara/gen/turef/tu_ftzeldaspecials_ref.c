/* C reference for rust/melee_chara/src/tu/ftzeldaspecials.rs: the game's ftzeldaspecials.c (verbatim)
 * on the state oracle of tu_zelda_common.h. Run: python rust/melee_chara/gen/turef/tu_shard457.py */
#include "../../../../src/melee/ft/kinds/ftZelda/ftzeldaspecials.c"
#include "tu_ftzeldaspecials_extract.inc"

ftCommonData* p_ftCommonData;

/* TARGET_PC script hooks (pc/gameworld/script_fighter_mod.inc, script_fighter_caps_movement.inc): the
 * oracle runs with no script mods and the vanilla jump count, as the Rust port does by default. */
float ScriptGame_ModValue(int slot, int field, float base)
{
    (void) slot;
    (void) field;
    return base;
}
int ScriptGame_CapsMaxJumps(Fighter* fp)
{
    return fp->co_attrs.max_jumps;
}

typedef struct {
    void* p;
    const char* n;
} CbEntry;

static const CbEntry CB_TABLE[] = {
    {NULL, "0"},
    {(void*) ftZd_Init_801393AC, "init"},
    {(void*) efLib_PauseAll, "pause"},
    {(void*) efLib_ResumeAll, "resume"},
};
#define CB_COUNT ((int) (sizeof CB_TABLE / sizeof CB_TABLE[0]))

static const char* FUNC_NAMES[] = {
    "ftZd_SpecialS_Enter",        "ftZd_SpecialAirS_Enter",   "ftZd_SpecialSStart_Anim",
    "ftZd_SpecialSLoop_Anim",     "ftZd_SpecialSEnd_Anim",    "ftZd_SpecialAirSStart_Anim",
    "ftZd_SpecialAirSLoop_Anim",  "ftZd_SpecialAirSEnd_Anim", "ftZd_SpecialSStart_IASA",
    "ftZd_SpecialSLoop_IASA",     "ftZd_SpecialSEnd_IASA",    "ftZd_SpecialAirSStart_IASA",
    "ftZd_SpecialAirSLoop_IASA",  "ftZd_SpecialSStart_Phys",
};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))

static int g_ret; /* return value of the function under test (0 for void) */

static void call_fn(int f, HSD_GObj* g)
{
    switch (f) {
    case 0: ftZd_SpecialS_Enter(g); break;
    case 1: ftZd_SpecialAirS_Enter(g); break;
    case 2: ftZd_SpecialSStart_Anim(g); break;
    case 3: ftZd_SpecialSLoop_Anim(g); break;
    case 4: ftZd_SpecialSEnd_Anim(g); break;
    case 5: ftZd_SpecialAirSStart_Anim(g); break;
    case 6: ftZd_SpecialAirSLoop_Anim(g); break;
    case 7: ftZd_SpecialAirSEnd_Anim(g); break;
    case 8: ftZd_SpecialSStart_IASA(g); break;
    case 9: ftZd_SpecialSLoop_IASA(g); break;
    case 10: ftZd_SpecialSEnd_IASA(g); break;
    case 11: ftZd_SpecialAirSStart_IASA(g); break;
    case 12: ftZd_SpecialAirSLoop_IASA(g); break;
    case 13: ftZd_SpecialSStart_Phys(g); break;
    }
}

#include "tu_zelda_common.h"

int main(void)
{
    run_all();
    return 0;
}
