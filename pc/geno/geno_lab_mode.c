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
    case GENO_LAB_TO_MATCH:
        gm_SetNextGameModeStateId(LAB_STATE_LOADING); /* a restart: same fighters, same stage */
        break;
    default:
        gm_SetNextGameModeStateId(LAB_STATE_CSS);
        break;
    }
    OSReport("geno lab: LAB match over -> %s\n", lab_next == GENO_LAB_TO_SSS     ? "stage select"
                                                 : lab_next == GENO_LAB_TO_MENU  ? "menus"
                                                 : lab_next == GENO_LAB_TO_MATCH ? "the same match"
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
    lab_next = where == GENO_LAB_TO_SSS || where == GENO_LAB_TO_MENU || where == GENO_LAB_TO_MATCH
                   ? where
                   : GENO_LAB_TO_CSS;
    gmVs_EndMatch(OUTCOME_NO_CONTEST);
    return 1;
}

/* ---- debug movement: fly / noclip and teleport (geno_lab_mode.h) --------------------------------
 * Flying is "the fighter's state callbacks are these": ftCo_Fall_Enter first (air, no hitboxes,
 * Fall's animation), then the anim / input / phys / coll callbacks are replaced. So:
 *  - input: none (no jump, attack or airdodge interrupts);
 *  - phys: velocity = stick x speed (A held: x0.25, B held: x4), no gravity, knockback cleared;
 *  - coll: no stage collision or ledge grab; the collision data follows the fighter (its previous
 *    position is reset every frame), so leaving the flight never sweeps across the stage;
 *  - ft_0D31.c skips the blast-zone KO for it, and the off-screen damage counter is held at 0;
 *  - cm/camera.c: while any fighter flies, the camera frames only the flying fighters and ignores
 *    the stage's camera bounds (GenoFly_Any, GenoFly_CameraSubject).
 * A change of action by anything else (KO, respawn, match end) replaces the callbacks, which ends
 * the flight with nothing to restore. The speed and the solid switch are this TU's statics (in a
 * snapshot: pc_geno_*). */
#include <melee/ft/fighter.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/types.h>
#include <melee/ft/kinds/ftCommon/ftCo_Fall.h>
#include <melee/ft/kinds/ftCommon/forward.h>
#include <melee/mp/mpcoll.h>
#include <melee/mp/mplib.h>
#include <melee/pl/player.h>
#include <sysdolphin/baselib/controller.h>
#include <sysdolphin/baselib/gobj.h>

static float fly_speed = 2.0f; /* units per frame at full stick */
static int fly_solid;          /* 1: keep hurtboxes */

static void fly_anim(Fighter_GObj* gobj)
{
    (void) gobj;
}

static void fly_input(Fighter_GObj* gobj)
{
    (void) gobj;
}

static void fly_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    float sx = fp->input.lstick[0].x, sy = fp->input.lstick[0].y, v = fly_speed;
    if (fp->input.held_buttons[0] & HSD_PAD_A) {
        v *= 0.25f;
    }
    if (fp->input.held_buttons[0] & HSD_PAD_B) {
        v *= 4.0f;
    }
    if (sx < 0.1f && sx > -0.1f) { /* a resting stick's drift stays put */
        sx = 0.0f;
    }
    if (sy < 0.1f && sy > -0.1f) {
        sy = 0.0f;
    }
    fp->self_vel.x = sx * v;
    fp->self_vel.y = sy * v;
    fp->self_vel.z = 0.0f;
    fp->x8c_kb_vel.x = fp->x8c_kb_vel.y = fp->x8c_kb_vel.z = 0.0f;
    if (sx > 0.0f) {
        fp->facing_dir = 1.0f;
    } else if (sx < 0.0f) {
        fp->facing_dir = -1.0f;
    }
    fp->dmg.x1910 = 0; /* off-screen damage (fighter.c) never builds up */
    /* intangible (ftColl_8007B62C's state 2, without its colour flash), or normal when solid: set every
     * frame, so switching solid mid-flight takes effect at once */
    fp->x1988 = fly_solid ? 0 : 2;
}

static void fly_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    mpColl_80043680(&fp->coll_data, &fp->cur_pos); /* follow: no floor, wall, ceiling or ledge */
    fp->coll_data.env_flags = 0;
}

int GenoFly_Fighter(Fighter* fp)
{
    return fp != NULL && fp->phys_cb == fly_phys;
}

static Fighter_GObj* fly_gobj(int slot)
{
    HSD_GObj* g;
    HSD_GObj* want;
    if (slot < 0 || slot >= 6 || HSD_GObjPLinkHead == NULL) {
        return NULL;
    }
    want = Player_GetEntity(slot);
    for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g != NULL; g = g->next) {
        if (g == want) {
            return g;
        }
    }
    return NULL;
}

/* states a fighter cannot be lifted out of cleanly: dead / asleep / respawning, held or thrown,
 * the entry, holding someone, the hands */
static int fly_refused(Fighter* fp)
{
    if (fp->x221F_b3 || fp->kind == Ft_Kind_MasterH || fp->kind == Ft_Kind_CrezyH) {
        return 1;
    }
    if (fp->motion_id >= ftCo_MS_DeadDown && fp->motion_id <= ftCo_MS_RebirthWait) {
        return 1;
    }
    if (fp->motion_id >= ftCo_MS_CapturePulledHi && fp->motion_id <= ftCo_MS_ThrownCrazyHand) {
        return 1;
    }
    return fp->victim_gobj != NULL || fp->x1A5C != NULL;
}

/* into Fall at the current spot: velocities cleared, the collision data moved here, hurtboxes on */
static void fly_drop(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    fp->self_vel.x = fp->self_vel.y = fp->self_vel.z = 0.0f;
    fp->x8c_kb_vel.x = fp->x8c_kb_vel.y = fp->x8c_kb_vel.z = 0.0f;
    fp->prev_pos = fp->cur_pos;
    mpColl_80043680(&fp->coll_data, &fp->cur_pos);
    ftCo_Fall_Enter(gobj);
    fp->x1988 = 0;
}

int GenoFly_Set(int slot, int mode)
{
    Fighter_GObj* gobj = fly_gobj(slot);
    Fighter* fp;
    if (gobj == NULL) {
        return -1;
    }
    fp = GET_FIGHTER(gobj);
    if (mode == GENO_FLY_ON) {
        if (GenoFly_Fighter(fp)) {
            return 0;
        }
        if (fly_refused(fp)) {
            return -2;
        }
        fly_drop(gobj);
        fp->anim_cb = fly_anim;
        fp->input_cb = fly_input;
        fp->phys_cb = fly_phys;
        fp->coll_cb = fly_coll;
        if (!fly_solid) {
            fp->x1988 = 2;
        }
        return 0;
    }
    if (!GenoFly_Fighter(fp)) {
        return 0;
    }
    if (mode == GENO_FLY_PLACE) {
        Vec3 floor;
        if (mpCheckFloor(fp->cur_pos.x, fp->cur_pos.y, fp->cur_pos.x, fp->cur_pos.y - 100000.0f, 0.0f,
                         &floor, NULL, NULL, NULL, -1, -1, -1, NULL, NULL))
        {
            fp->cur_pos.x = floor.x;
            fp->cur_pos.y = floor.y + 0.5f; /* Fall lands on it the next frame */
            fly_drop(gobj);
            return 0;
        }
        fly_drop(gobj);
        return 1;
    }
    fly_drop(gobj);
    return 0;
}

int GenoFly_Get(int slot)
{
    Fighter_GObj* gobj = fly_gobj(slot);
    return gobj == NULL ? -1 : GenoFly_Fighter(GET_FIGHTER(gobj));
}

int GenoFly_Any(void)
{
    HSD_GObj* g;
    if (HSD_GObjPLinkHead == NULL) {
        return 0;
    }
    for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g != NULL; g = g->next) {
        if (GenoFly_Fighter(GET_FIGHTER(g))) {
            return 1;
        }
    }
    return 0;
}

/* cm/camera.c: 1 when `subject` is a flying fighter's camera subject */
int GenoFly_CameraSubject(void* subject)
{
    HSD_GObj* g;
    if (HSD_GObjPLinkHead == NULL) {
        return 0;
    }
    for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g != NULL; g = g->next) {
        Fighter* fp = GET_FIGHTER(g);
        if (GenoFly_Fighter(fp) && (void*) fp->x890_cameraBox == subject) {
            return 1;
        }
    }
    return 0;
}

int GenoFly_Teleport(int slot, int x_bits, int y_bits)
{
    Fighter_GObj* gobj = fly_gobj(slot);
    Fighter* fp;
    union {
        int i;
        float f;
    } x, y;
    if (gobj == NULL) {
        return -1;
    }
    fp = GET_FIGHTER(gobj);
    if (!GenoFly_Fighter(fp) && fly_refused(fp)) {
        return -2;
    }
    x.i = x_bits;
    y.i = y_bits;
    fp->cur_pos.x = x.f;
    fp->cur_pos.y = y.f;
    fp->prev_pos = fp->cur_pos;
    if (GenoFly_Fighter(fp)) {
        mpColl_80043680(&fp->coll_data, &fp->cur_pos);
    } else {
        fly_drop(gobj); /* a grounded fighter would snap back along its floor line */
    }
    return 0;
}

void GenoFly_SetSpeed(int speed_bits)
{
    union {
        int i;
        float f;
    } s;
    s.i = speed_bits;
    fly_speed = s.f < 0.05f ? 0.05f : s.f > 200.0f ? 200.0f : s.f;
}

float GenoFly_Speed(void)
{
    return fly_speed;
}

void GenoFly_SetSolid(int solid)
{
    fly_solid = solid != 0;
}

int GenoFly_Solid(void)
{
    return fly_solid;
}

/* geno_tests.c: the fly callbacks on a test fighter (no player, no stage, no Fall) */
void GenoFly_TestArm(Fighter* fp)
{
    fp->anim_cb = fly_anim;
    fp->input_cb = fly_input;
    fp->phys_cb = fly_phys;
    fp->coll_cb = fly_coll;
}
