/*
 * geno_lab_mode.c - LAB, the Geno Lab's own game mode (private, Geno build). See geno_lab_mode.h.
 *
 * Game-world code (a gwtool game TU, like geno_game.c). It is VS mode's machinery with LAB's own
 * VsModeData row and rules: gmVsMelee_EnterCss / ExitCss / EnterSss / ExitSss / EnterVs run as for
 * any VS variant (Giant, Single-button...), so the kit's character select takes any fighters on
 * any ports, humans and CPUs. What LAB changes:
 *  - its select screens are always the kit's (gmFrontend_ModeSelect, breadcrumbs SOLO / LAB), even
 *    with MELEE_NATIVE_CSS=1; Training's own select is untouched;
 *  - the rules (GenoLab_ApplyRules): time mode with the clock off, so a KO respawns forever and the
 *    match never ends on its own; no items; Melee's pause off (the Lab script draws its own);
 *  - the match ends only through GenoLab_Leave (the Lab's pause menu, or a no contest), and goes
 *    back to LAB's character select, the stage select, or the menus - never to a results screen,
 *    and nothing is recorded to the save's statistics.
 * LAB's VsModeData is its own static row, not one of the save file's, so nothing LAB picks changes
 * VS mode's remembered fighters or rules.
 */

#include "geno_lab_mode.h"

#include <dolphin/os.h>

#include <melee/gm/gm_1601.h>
#include <melee/gm/gm_1A3F.h>
#include <melee/gm/gm_unsplit.h>
#include <melee/gm/gmfrontend.h>
#include <melee/gm/gmmain_lib.h>
#include <melee/gm/gmscenelaunch.h>
#include <melee/gm/gmvsmelee.h>
#include <melee/gm/types.h>
#include <melee/lb/lbdvd.h>
#include <melee/lb/types.h>
#include <melee/mn/types.h>

enum {
    LAB_STATE_CSS = 0,
    LAB_STATE_SSS = 1,
    LAB_STATE_MATCH = 2,
    LAB_STATE_LOADING = 3,
};

static VsModeData lab_vs;
static int lab_in_match;
static int lab_next = GENO_LAB_TO_CSS;

/* native (gw_script.c): the Lab script's LAB request, which turns the Lab on in the match */
extern void Script_LabRequest(void);
/* gmvs.c: end the running VS match with this outcome, as a no contest does */
extern void gmVs_EndMatch(int outcome);

static void lab_enter_css(GameModeState* state)
{
    gmVsMelee_EnterCss(state, &lab_vs, VS_MELEE);
    gmFrontend_ModeSelect(state, 0, "LAB");
}

static void lab_exit_css(GameModeState* state)
{
    gmVsMelee_ExitCss(state, &lab_vs); /* B: to the menus (GM_MENU) */
}

static void lab_enter_sss(GameModeState* state)
{
    gmVsMelee_EnterSss(state, &lab_vs);
    gmFrontend_ModeSelect(state, 1, "LAB");
}

static void lab_exit_sss(GameModeState* state)
{
    gmVsMelee_ExitSss(state, &lab_vs, LAB_STATE_CSS);
    if (((SSSData*) gm_GetGameModeStateExitData(state))->start_game) {
        gm_SetNextGameModeStateId(LAB_STATE_LOADING);
    }
}

static void lab_enter_loading(GameModeState* state)
{
    (void) state;
    gmFrontend_BeginLoading();
}

static void lab_exit_loading(GameModeState* state)
{
    (void) state;
    gm_SetNextGameModeStateId(LAB_STATE_MATCH);
}

void GenoLab_ApplyRules(StartMeleeData* start)
{
    int i;
    start->rules.match_kind = MatchKind_Time;
    start->rules.is_stock = false;
    start->rules.timer_enabled = false; /* no clock: gm_GetMatchOutcome never times out */
    start->rules.timer_counts_up = false;
    start->rules.time_limit = 0;
    start->rules.item_freq = -1;        /* no items */
    start->rules.disable_pausing = true; /* START opens the Lab's pause menu instead */
    start->rules.x5_0 = false;
    start->rules.x4_2 = false; /* no "P1 out of stocks" game over */
    for (i = 0; i < Gm_Player_NumMax; i++) {
        start->players[i].stocks = 0;
    }
}

static void lab_start_cb(StartMeleeData* start, StartMeleeData* vs_start)
{
    (void) vs_start;
    GenoLab_ApplyRules(start);
}

static void lab_player_cb(PlayerInitData* player, PlayerInitData* vs_player)
{
    (void) vs_player;
    player->stocks = 0;
}

static void lab_enter_match(GameModeState* state)
{
    gmVsMelee_EnterVs(state, &lab_vs, lab_start_cb, lab_player_cb);
    /* the callbacks run before the players are copied in: stocks are set again here */
    GenoLab_ApplyRules(gm_GetGameModeStateEnterData(state));
    lab_in_match = 1;
    lab_next = GENO_LAB_TO_CSS;
    Script_LabRequest(); /* the Lab script turns itself on for this match */
    OSReport("geno lab: LAB match - no timer, no stocks, infinite respawn\n");
}

int GenoLab_NextAfterMatch(void)
{
    return lab_next;
}

static void lab_exit_match(GameModeState* state)
{
    (void) state;
    lab_in_match = 0;
    switch (lab_next) {
    case GENO_LAB_TO_SSS:
        gm_SetNextGameModeStateId(LAB_STATE_SSS);
        break;
    case GENO_LAB_TO_MENU:
        gm_ChangeGameModeAfterCurrentScene(GM_MENU);
        break;
    default:
        gm_SetNextGameModeStateId(LAB_STATE_CSS);
        break;
    }
    OSReport("geno lab: LAB match over -> %s\n", lab_next == GENO_LAB_TO_SSS    ? "stage select"
                                                 : lab_next == GENO_LAB_TO_MENU ? "menus"
                                                                                : "character select");
    lab_next = GENO_LAB_TO_CSS;
}

GameModeState gm_Mode_Lab_States[] = {
    {
        LAB_STATE_CSS,
        lbDvdPreload_3,
        0,
        lab_enter_css,
        lab_exit_css,
        {
            GS_CSS,
            &gmVsMelee_CssData,
            &gmVsMelee_CssData,
        },
    },
    {
        LAB_STATE_SSS,
        lbDvdPreload_3,
        0,
        lab_enter_sss,
        lab_exit_sss,
        {
            GS_SSS,
            &gmVsMelee_SssData,
            &gmVsMelee_SssData,
        },
    },
    {
        LAB_STATE_MATCH,
        lbDvdPreload_3,
        0,
        lab_enter_match,
        lab_exit_match,
        {
            GS_VS,
            &gmVsMelee_StartData,
            &gmVsMelee_VsExitInfo,
        },
    },
    {
        LAB_STATE_LOADING,
        lbDvdPreload_3,
        0,
        lab_enter_loading,
        lab_exit_loading,
        {
            GS_FRONTEND,
            NULL,
            NULL,
        },
    },
    { GM_GAMEMODESTATE_TERMINATE },
};

void gm_Mode_Lab_OnInit(void)
{
    gm_InitVsMode(&lab_vs);
}

void gm_Mode_Lab_OnLoad(void)
{
    lab_in_match = 0;
    lab_next = GENO_LAB_TO_CSS;
    Script_LabRequest();
    /* MELEE_SCENE=mode=lab;p1=...;p2=.../cpu;stage=... seeds LAB's own row and jumps to the
       requested screen (the match by default). Only a LAB scene seeds LAB. */
    if (SceneLaunch_BootGameMode() == GM_LAB) {
        SceneLaunch_SeedVs(&lab_vs, false);
    }
}

int GenoLab_ModeActive(void)
{
    return gm_GetCurrentGameMode() == GM_LAB;
}

int GenoLab_InMatch(void)
{
    return gm_GetCurrentGameMode() == GM_LAB && lab_in_match;
}

int GenoLab_Leave(int where)
{
    if (gm_GetCurrentGameMode() != GM_LAB || !lab_in_match) {
        return 0;
    }
    lab_next = where == GENO_LAB_TO_SSS || where == GENO_LAB_TO_MENU ? where : GENO_LAB_TO_CSS;
    gmVs_EndMatch(OUTCOME_NO_CONTEST);
    return 1;
}
