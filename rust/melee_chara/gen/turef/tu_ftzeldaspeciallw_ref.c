/* C reference for rust/melee_chara/src/tu/ftzeldaspeciallw.rs: the game's ftzeldaspeciallw.c (verbatim)
 * on the state oracle of tu_zelda_common.h. Run: python rust/melee_chara/gen/turef/tu_shard457.py */
#include "../../../../src/melee/ft/kinds/ftZelda/ftzeldaspeciallw.c"
#include "tu_ftzeldaspeciallw_extract.inc"

ftCommonData* p_ftCommonData;

/* TARGET_PC script hook (pc/gameworld/script_fighter_mod.inc): no script mods in this oracle. */
float ScriptGame_ModValue(int slot, int field, float base)
{
    (void) slot;
    (void) field;
    return base;
}

/* Referenced only as a callback address (group 4 / ftSeak, not ported). */
void ftSk_SpecialLw_80114758(Fighter_GObj* g)
{
    (void) g;
}

void ftZd_Init_801393AC(HSD_GObj* gobj);

typedef struct {
    void* p;
    const char* n;
} CbEntry;

static const CbEntry CB_TABLE[] = {
    {NULL, "0"},
    {(void*) ftZd_SpecialLw_8013ADB4, "lADB4"},
    {(void*) ftZd_SpecialLw_8013AE30, "lAE30"},
    {(void*) ftZd_SpecialLw_8013AEAC, "lAEAC"},
    {(void*) ftZd_Init_801393AC, "init"},
    {(void*) efLib_PauseAll, "pause"},
    {(void*) efLib_ResumeAll, "resume"},
    {(void*) ftSk_SpecialLw_80114758, "skLw"},
};
#define CB_COUNT ((int) (sizeof CB_TABLE / sizeof CB_TABLE[0]))

static const char* FUNC_NAMES[] = {
    "ftZd_SpecialLw_8013ADB4", "ftZd_SpecialLw_8013AE30", "ftZd_SpecialLw_8013AEAC",
    "ftZd_SpecialLw_Enter",    "ftZd_SpecialAirLw_Enter", "ftZd_SpecialLw_8013B4D8",
    "ftZd_SpecialLw_8013B540", "ftZd_SpecialLw_8013B574", "ftZd_SpecialLw_8013B5EC",
};
#define NFUNCS ((int) (sizeof FUNC_NAMES / sizeof FUNC_NAMES[0]))

static int g_ret; /* return value of the function under test (0 for void) */

static void call_fn(int f, HSD_GObj* g)
{
    switch (f) {
    case 0: ftZd_SpecialLw_8013ADB4(g); break;
    case 1: ftZd_SpecialLw_8013AE30(g); break;
    case 2: ftZd_SpecialLw_8013AEAC(g); break;
    case 3: ftZd_SpecialLw_Enter(g); break;
    case 4: ftZd_SpecialAirLw_Enter(g); break;
    case 5: ftZd_SpecialLw_8013B4D8(g); break;
    case 6: g_ret = ftZd_SpecialLw_8013B540(g); break;
    case 7: g_ret = ftZd_SpecialLw_8013B574(g); break;
    case 8: ftZd_SpecialLw_8013B5EC(g); break;
    }
}

#include "tu_zelda_common.h"

int main(void)
{
    run_all();
    return 0;
}
