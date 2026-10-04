#include <gameworld/script_hit_rules.h>
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
#include <math.h>
#include <string.h>

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
    if ((lab_vs.start.players[4].slot_type != Gm_PKind_NA ||
         lab_vs.start.players[5].slot_type != Gm_PKind_NA) &&
        lab_next != GENO_LAB_TO_MATCH && lab_next != GENO_LAB_TO_MENU) {
        OSReport("geno lab: six-slot match cannot return through CSS/SSS; going to menus\n");
        lab_next = GENO_LAB_TO_MENU;
    }
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
    if ((lab_vs.start.players[4].slot_type != Gm_PKind_NA ||
         lab_vs.start.players[5].slot_type != Gm_PKind_NA) &&
        where != GENO_LAB_TO_MATCH && where != GENO_LAB_TO_MENU) {
        OSReport("geno lab: six-slot selection refused; restart the match or leave to menus\n");
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
#include <melee/ft/ftanim.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ftcoll.h>
#include <melee/lb/lbcollision.h>
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

typedef struct {
    Fighter* fighter;
    int target, attack, damage, pulses;
    float x, y, radius;
} FlyCursor;
static FlyCursor fly_cursor[6];
extern u16 plStale_IncrementAttackInstance(void);
extern int Netplay_Enabled(void);
extern int RB_Enabled(void);
extern int Replay_Active(void);

static int fly_boundary_blocked(int netplay, int rollback, int replay)
{
    return netplay || rollback || replay;
}

int GenoFly_OfflineAllowed(void)
{
    return !fly_boundary_blocked(Netplay_Enabled(), RB_Enabled(), Replay_Active());
}

static FlyCursor* fly_cursor_for(Fighter* fp)
{
    int slot = fp->player_id;
    return slot >= 0 && slot < 6 && fly_cursor[slot].fighter == fp ? &fly_cursor[slot] : NULL;
}

/* Rearm ordinary collision, not damage injection. Both fighter and item victim
 * histories belong to this attack instance and must be fresh for every pulse. */
static void fly_cursor_pulse(Fighter* fp, FlyCursor* c)
{
    HitCapsule* h = &fp->x914[0];
    int i;
    for (i = 0; i < 4; ++i) fp->x914[i].state = HitCapsule_Disabled;
    memset(h, 0, sizeof *h);
    h->state = HitCapsule_Enabled;
    h->damage = c->damage; h->unk_count = c->damage; h->scale = c->radius;
    h->kb_angle = 45; h->x24 = 70; h->x2C = 20;
    h->x40_b0 = 1; h->x40_b2 = 1; h->x40_b3 = 1;
    h->x42_b5 = 1; h->x42_b7 = 1; /* native fighter AND item target eligibility */
    h->b_offset = fp->cur_pos;
    h->x4C = h->x58 = fp->cur_pos;
    ScriptGame_HitRuleCreate(fp,h,0);
    fp->x206C_attack_instance = plStale_IncrementAttackInstance();
    if (c->pulses < 0x7FFFFFFF) ++c->pulses;
}

static void fly_input(Fighter_GObj* gobj);
static void hold_step(Fighter_GObj* gobj);
static Fighter_GObj* fly_gobj(int slot);
int GenoFly_Fighter(Fighter* fp);
/* gd.hold_hitbox: per-port (pc_geno_* statics, so savestates carry them) */
static int hold_on[6], hold_action[6], hold_frame[6], hold_frozen[6], hold_tick[6], hold_mask[6], hold_hits[6];
#define HOLD_REHIT_FRAMES 8
static void fly_phys(Fighter_GObj* gobj);
static void fly_coll(Fighter_GObj* gobj);

/* Loop the idle: Wait's own anim callback (ftCo_Wait_Anim) is replaced, so restart the animation
 * when it ends and put the fly callbacks back over the ones Fighter_ChangeMotionState installs. */
static void fly_anim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    if (fp->motion_id != ftCo_MS_Wait || ftAnim_IsFramesRemaining(gobj)) {
        return;
    }
    Fighter_ChangeMotionState(gobj, ftCo_MS_Wait, 0, 0.0f, 1.0f, 0.0f, NULL);
    fp->anim_cb = fly_anim;
    fp->input_cb = fly_input;
    fp->phys_cb = fly_phys;
    fp->coll_cb = fly_coll;
}

static void fly_input(Fighter_GObj* gobj)
{
    (void) gobj;
}

static void fly_phys(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    float sx = fp->input.lstick[0].x, sy = fp->input.lstick[0].y, v = fly_speed;
    FlyCursor* cursor = fly_cursor_for(fp);
    if (!GenoFly_OfflineAllowed()) { GenoFly_Watch(); return; }
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
    if (cursor && cursor->target) {
        float dx = cursor->x - fp->cur_pos.x, dy = cursor->y - fp->cur_pos.y;
        float distance = sqrtf(dx * dx + dy * dy);
        float fraction = distance > fly_speed ? fly_speed / distance : 1.0f;
        fp->self_vel.x = dx * fraction;
        fp->self_vel.y = dy * fraction;
    }
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
    /* The pulse reads the frame's real hitlag, so it precedes hold_step, which zeroes
     * x195c_hitlag_frames; hold_step stays last so a held hitbox re-arms last. */
    if (cursor && cursor->attack && fp->dmg.x195c_hitlag_frames <= 0.0f) fly_cursor_pulse(fp, cursor);
    hold_step(gobj);
}

static void fly_coll(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    FlyCursor* cursor = fly_cursor_for(fp);
    mpColl_80043680(&fp->coll_data, &fp->cur_pos); /* follow: no floor, wall, ceiling or ledge */
    fp->coll_data.env_flags = 0;
    /* Physics integrates velocity after fly_phys. Centre the world capsule only
     * now, before native hitbox updates/collision. This is not another pulse:
     * victim histories and the attack instance survive hitlag unchanged. */
    if (cursor && cursor->attack && fp->x914[0].state != HitCapsule_Disabled) {
        fp->x914[0].b_offset = fp->cur_pos;
        fp->x914[0].x4C = fp->x914[0].x58 = fp->cur_pos;
    }
}

/* ---- gd.hold_hitbox: a fighter flying in an attack state, frozen where its hitbox is live -------
 * hold_step runs from fly_phys. It (re)enters the attack state, lets the animation run until a hitbox is
 * enabled (and past opts.frame), then freezes the animation (rate 0) so that hitbox stays on, and every
 * HOLD_REHIT_FRAMES clears the hit victims so the same enemy is hit again. */
static void hold_callbacks(Fighter* fp)
{
    fp->anim_cb = fly_anim;
    fp->input_cb = fly_input;
    fp->phys_cb = fly_phys;
    fp->coll_cb = fly_coll;
}

static int hold_live(Fighter* fp)
{
    int i;
    for (i = 0; i < 4; i++) {
        if (fp->x914[i].state == HitCapsule_Enabled) {
            return 1;
        }
    }
    return 0;
}

static void hold_step(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    int slot = fp->player_id, i;
    if (slot < 0 || slot >= 6 || !hold_on[slot]) {
        return;
    }
    if (fp->motion_id != hold_action[slot] || hold_frozen[slot] < 0) {
        Fighter_ChangeMotionState(gobj, hold_action[slot], 0, 0.0f, 1.0f, 0.0f, NULL);
        hold_callbacks(fp);
        hold_frozen[slot] = 0;
        hold_tick[slot] = 0;
        return;
    }
    if (!hold_frozen[slot]) {
        int mask = 0;
        for (i = 0; i < 4; i++) {
            if (fp->x914[i].state == HitCapsule_Enabled) {
                mask |= 1 << i;
            }
        }
        if (mask != 0 && fp->cur_anim_frame >= (float) hold_frame[slot]) {
            hold_frozen[slot] = 1;
            hold_mask[slot] = mask;
            OSReport("geno hold: P%d frozen in motion %d at frame %.1f, hitboxes %x\n", slot + 1,
                     (int) fp->motion_id, fp->cur_anim_frame, mask);
        } else if (!ftAnim_IsFramesRemaining(gobj)) {
            hold_frozen[slot] = -1; /* restart the animation next frame */
            return;
        }
    }
    if (hold_frozen[slot] == 1) {
        ftAnim_SetAnimRate(gobj, 0.0f);
        fp->dmg.x195c_hitlag_frames = 0.0f; /* the attacker's hitlag never freezes the hold */
        for (i = 0; i < 4; i++) {
            if ((hold_mask[slot] & (1 << i)) && fp->x914[i].state != HitCapsule_Enabled) {
                fp->x914[i].state = HitCapsule_Enabled; /* re-armed: a connect or the move's end cleared it */
            }
        }
        if (++hold_tick[slot] % HOLD_REHIT_FRAMES == 0) {
            hold_hits[slot]++; /* rehit intervals offered while the hitbox was live (item hits leave no record) */
            for (i = 0; i < 4; i++) {
                lbColl_80008440(&fp->x914[i]); /* victims cleared: the same target can be hit again */
            }
        }
    }
}

/* action: 0 nair (default), 1 fair, 2 bair, 3 uair, 4 dair, 5 jab, or a raw motion state id (>5).
 * frame: freeze no earlier than this animation frame (< 0: the first frame a hitbox is live). */
int GenoFly_HoldHitbox(int slot, int on, int action, int frame)
{
    Fighter_GObj* gobj = fly_gobj(slot);
    Fighter* fp;
    static const int codes[6] = {ftCo_MS_AttackAirN, ftCo_MS_AttackAirF, ftCo_MS_AttackAirB,
                                 ftCo_MS_AttackAirHi, ftCo_MS_AttackAirLw, ftCo_MS_Attack11};
    if (gobj == NULL) {
        return -1;
    }
    fp = GET_FIGHTER(gobj);
    if (!GenoFly_Fighter(fp)) {
        return -3;
    }
    if (!on) {
        if (hold_on[slot]) {
            hold_on[slot] = 0;
            Fighter_ChangeMotionState(gobj, ftCo_MS_Wait, 0, 0.0f, 1.0f, 0.0f, NULL);
            hold_callbacks(fp);
            fp->self_vel.x = fp->self_vel.y = fp->self_vel.z = 0.0f;
        }
        return 0;
    }
    hold_on[slot] = 1;
    hold_hits[slot] = 0;
    hold_action[slot] = action >= 0 && action < 6 ? codes[action] : action;
    hold_frame[slot] = frame;
    hold_tick[slot] = 0;
    hold_frozen[slot] = -1; /* enter the attack on the next flight frame */
    return 0;
}

int GenoFly_HoldHits(int slot)
{
    return slot >= 0 && slot < 6 ? hold_hits[slot] : 0;
}

int GenoFly_Holding(int slot)
{
    return slot >= 0 && slot < 6 && hold_on[slot];
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

void GenoFly_Clear(int slot)
{
    Fighter_GObj* gobj;
    FlyCursor* c;
    if (slot < 0 || slot >= 6) return;
    c = &fly_cursor[slot]; gobj = fly_gobj(slot);
    /* A replaced action owns its own hitboxes: never clear those using stale
     * cursor data. Enabling a new flight always clears its old configuration. */
    if (c->attack && gobj && GET_FIGHTER(gobj) == c->fighter && GenoFly_Fighter(c->fighter))
        c->fighter->x914[0].state = HitCapsule_Disabled;
    memset(c, 0, sizeof *c);
}

void GenoFly_Reset(void) { int i; for (i = 0; i < 6; ++i) GenoFly_Clear(i); }

static Fighter* fly_cursor_actor(int slot)
{
    Fighter_GObj* g = fly_gobj(slot);
    Fighter* fp = g ? GET_FIGHTER(g) : NULL;
    return fp && !fp->is_sub_fighter && GenoFly_Fighter(fp) ? fp : NULL;
}

int GenoFly_Target(int slot, int x_bits, int y_bits)
{
    union { int i; float f; } x, y;
    Fighter* fp = fly_cursor_actor(slot);
    x.i = x_bits; y.i = y_bits;
    if (!fp) return -1;
    if (!(x.f >= -49000 && x.f <= 49000 && y.f >= -49000 && y.f <= 49000)) return -2;
    fly_cursor[slot].fighter = fp; fly_cursor[slot].target = 1;
    fly_cursor[slot].x = x.f; fly_cursor[slot].y = y.f;
    return 0;
}

int GenoFly_AttackSet(int slot, int on, int damage, int radius_bits)
{
    union { int i; float f; } radius;
    Fighter* fp = fly_cursor_actor(slot);
    FlyCursor* c;
    if (slot < 0 || slot >= 6) return -1;
    c = &fly_cursor[slot];
    if (!on) {
        if (fp && c->fighter == fp && c->attack) fp->x914[0].state = HitCapsule_Disabled;
        c->attack = 0; return 0;
    }
    if (!fp) return -1;
    radius.i = radius_bits;
    if (damage < 1 || damage > 30 || !(radius.f >= 1 && radius.f <= 30)) return -2;
    c->fighter = fp; c->attack = 1; c->damage = damage; c->radius = radius.f;
    return 0;
}

int GenoFly_State(int slot, int field)
{
    FlyCursor* c;
    union { int i; float f; } value;
    if (slot < 0 || slot >= 6) return -1;
    if (!fly_cursor_actor(slot)) return 0;
    c = &fly_cursor[slot];
    if (c->fighter != fly_cursor_actor(slot)) return 0;
    switch (field) {
    case 0: return c->attack; case 1: return c->target; case 2: return c->damage;
    case 3: value.f = c->radius; return value.i;
    case 4: value.f = c->x; return value.i;
    case 5: value.f = c->y; return value.i;
    case 6: return c->pulses;
    }
    return -1;
}
int GenoFly_Pulses(int slot) { return GenoFly_State(slot, 6); }

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
    /* Held, thrown or carried. The enum runs CapturePulledHi..ThrownCrazyHand with free states in the
     * middle (rolls, Rebound, Pass, Ottotto / OttottoWait, wall stops, the ledge, taunts): those fly
     * and teleport like any other. A fighter teetering at an edge (OttottoWait never ends on its own)
     * used to be refused, which left the map editor's return-to-cursor stuck. */
    if ((fp->motion_id >= ftCo_MS_CapturePulledHi && fp->motion_id <= ftCo_MS_CaptureFoot) ||
        (fp->motion_id >= ftCo_MS_ThrownF && fp->motion_id <= ftCo_MS_ThrownlwWomen) ||
        (fp->motion_id >= ftCo_MS_ShoulderedWait && fp->motion_id <= ftCo_MS_ThrownCrazyHand))
    {
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

/* Separate the boundary predicate from the normal motion transition so a small
 * headless fixture can verify cleanup without constructing Fall's disc data. */
static int fly_boundary_fighter(Fighter_GObj* gobj, int blocked,
                                void (*drop)(Fighter_GObj*))
{
    Fighter* fp = GET_FIGHTER(gobj);
    FlyCursor* cursor;
    if (!blocked || !GenoFly_Fighter(fp)) return 0;
    cursor = fly_cursor_for(fp);
    if (cursor) {
        if (cursor->attack) fp->x914[0].state = HitCapsule_Disabled;
        memset(cursor, 0, sizeof *cursor);
    }
    drop(gobj);
    return 1;
}

int GenoFly_Watch(void)
{
    HSD_GObj* g;
    if (GenoFly_OfflineAllowed()) return 0; /* ordinary offline flight stays on */
    if (HSD_GObjPLinkHead) {
        for (g = HSD_GObjPLinkHead[HSD_GOBJ_PLINK_FIGHTER]; g; g = g->next)
            fly_boundary_fighter(g, 1, fly_drop);
    }
    GenoFly_Reset(); /* retire stale configs too, even if the fighter went away */
    return 1;
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
        GenoFly_Clear(slot);
        if (fly_refused(fp)) {
            return -2;
        }
        fly_drop(gobj);
        /* Hover in the idle pose, not Fall's looping fall: Wait's animation only (no
         * translation), the fighter stays airborne (ground_or_air is Fall's), and the fly
         * callbacks below replace Wait's, so nothing lands, falls or tumbles. Leaving the
         * flight re-enters Fall (fly_drop), where gravity resumes from here. */
        if (fp->ground_or_air == GA_Air) {
            Fighter_ChangeMotionState(gobj, ftCo_MS_Wait, 0, 0.0f, 1.0f, 0.0f, NULL);
            fp->self_vel.x = fp->self_vel.y = fp->self_vel.z = 0.0f;
        }
        fp->anim_cb = fly_anim;
        fp->input_cb = fly_input;
        fp->phys_cb = fly_phys;
        fp->coll_cb = fly_coll;
        if (!fly_solid) {
            fp->x1988 = 2;
        }
        return 0;
    }
    GenoFly_Clear(slot);
    if (!GenoFly_Fighter(fp)) {
        return 0;
    }
    hold_on[slot] = 0;
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

/* Isolated fixtures verify native attack rearming and target convergence. */
static void fly_boundary_test_drop(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    fp->phys_cb = NULL;
    fp->x1988 = 0;
}

int GenoFly_CursorTest(void)
{
    static Fighter fp;
    static Fighter_GObj gobj;
    FlyCursor saved = fly_cursor[0];
    float speed = fly_speed;
    int solid = fly_solid, rc = 0, instance;
    memset(&fp, 0, sizeof fp); memset(&gobj, 0, sizeof gobj);
    gobj.user_data = &fp; fp.player_id = 0;
    GenoFly_TestArm(&fp); fly_speed = 2; fly_solid = 1;
    memset(&fly_cursor[0], 0, sizeof fly_cursor[0]);
    fly_cursor[0].fighter = &fp; fly_cursor[0].target = 1;
    fly_cursor[0].x = 3; fly_cursor[0].y = 4;
    fly_cursor[0].attack = 1; fly_cursor[0].damage = 3; fly_cursor[0].radius = 6;
    fp.x914[0].x44 = fp.x914[0].x45 = 12;
    fly_phys(&gobj);
    if (fp.self_vel.x < 1.199f || fp.self_vel.x > 1.201f ||
        fp.self_vel.y < 1.599f || fp.self_vel.y > 1.601f || fp.x1988 != 0 ||
        fp.x914[0].x44 || fp.x914[0].x45 || fp.x914[0].damage != 3 || fp.x914[0].scale != 6 ||
        !fp.x914[0].x42_b5 || !fp.x914[0].x42_b7 || fp.x914[0].x2C != 20) rc = 1;
    instance = fp.x206C_attack_instance;
    /* Match real callback order: integrate velocity, then run map collision.
     * The original implementation left the capsule at the pre-integration point. */
    fp.cur_pos.x += fp.self_vel.x; fp.cur_pos.y += fp.self_vel.y;
    fp.x914[0].x44 = fp.x914[0].x45 = 1;
    fp.x914[0].victims_1[0].victim = &gobj;
    fp.x914[0].victims_2[0].victim = &gobj;
    fly_coll(&gobj);
    if (fp.x914[0].b_offset.x != fp.cur_pos.x || fp.x914[0].b_offset.y != fp.cur_pos.y ||
        fp.x914[0].x4C.x != fp.cur_pos.x || fp.x914[0].x58.y != fp.cur_pos.y ||
        fp.x206C_attack_instance != instance || fly_cursor[0].pulses != 1 ||
        fp.x914[0].x44 != 1 || fp.x914[0].x45 != 1 ||
        fp.x914[0].victims_1[0].victim != &gobj || fp.x914[0].victims_2[0].victim != &gobj) rc = 1;
    fp.x914[0].x44 = fp.x914[0].x45 = 1;
    fp.x914[0].victims_1[0].victim = &gobj;
    fp.x914[0].victims_2[0].victim = &gobj;
    fp.cur_pos.x = 2.9f; fp.cur_pos.y = 3.9f;
    fly_phys(&gobj);
    if (fp.x206C_attack_instance == instance || fp.x914[0].x44 || fp.x914[0].x45 ||
        fp.x914[0].victims_1[0].victim || fp.x914[0].victims_2[0].victim ||
        fp.self_vel.x < .099f || fp.self_vel.x > .101f ||
        fp.self_vel.y < .099f || fp.self_vel.y > .101f || fly_cursor[0].pulses != 2) rc = 1;
    fp.dmg.x195c_hitlag_frames = 2; fly_phys(&gobj);
    if (fly_cursor[0].pulses != 2) rc = 1;
    instance = fp.x206C_attack_instance;
    fp.x914[0].x44 = fp.x914[0].x45 = 1;
    fp.x914[0].victims_1[0].victim = &gobj;
    fp.x914[0].victims_2[0].victim = &gobj;
    fp.cur_pos.x = 10; fp.cur_pos.y = 20; fly_coll(&gobj);
    if (fp.x914[0].b_offset.x != 10 || fp.x914[0].x4C.y != 20 || fp.x914[0].x58.y != 20 ||
        fp.x206C_attack_instance != instance || fly_cursor[0].pulses != 2 ||
        fp.x914[0].x44 != 1 || fp.x914[0].x45 != 1 ||
        fp.x914[0].victims_1[0].victim != &gobj || fp.x914[0].victims_2[0].victim != &gobj) rc = 1;
    /* Each boundary ends a configured cursor; ordinary offline flight survives. */
    if (fly_boundary_blocked(0, 0, 0) || !fly_boundary_blocked(1, 0, 0) ||
        !fly_boundary_blocked(0, 1, 0) || !fly_boundary_blocked(0, 0, 1)) rc = 1;
    if (fly_boundary_fighter(&gobj, 0, fly_boundary_test_drop) || !GenoFly_Fighter(&fp) ||
        !fly_cursor[0].attack || fp.x914[0].state == HitCapsule_Disabled) rc = 1;
    if (!fly_boundary_fighter(&gobj, 1, fly_boundary_test_drop) || GenoFly_Fighter(&fp) ||
        fp.x1988 != 0 || fly_cursor[0].attack || fly_cursor[0].target ||
        fp.x914[0].state != HitCapsule_Disabled) rc = 1;
    GenoFly_TestArm(&fp); /* existing stale-actor check still exercises fly_phys */
    fly_cursor[0].fighter = NULL; fp.x914[0].state = HitCapsule_Disabled;
    fp.dmg.x195c_hitlag_frames = 0; fly_phys(&gobj);
    if (fp.x914[0].state != HitCapsule_Disabled) rc = 1;
    fly_cursor[0] = saved; fly_speed = speed; fly_solid = solid;
    return rc;
}

/* Native scene suite supplies a parsed VS config; no heaps, preload or scene transitions. */
int GenoLab_SixSeedTest(int six)
{
    static VsModeData fixture;
    int i;
    memset(&fixture,0,sizeof fixture);
    gm_SetupAllPlayerDefaults(fixture.start.players);
    if (!six) {
        fixture.start.players[4].ckind=fixture.start.players[5].ckind=2;
        fixture.start.players[4].slot_type=fixture.start.players[5].slot_type=Gm_PKind_Cpu;
        SceneLaunch_SeedPlayers(&fixture,false);
        return fixture.start.players[4].ckind!=ChKind_None || fixture.start.players[5].ckind!=ChKind_None ||
               fixture.start.players[4].slot_type!=Gm_PKind_NA || fixture.start.players[5].slot_type!=Gm_PKind_NA;
    }
    if (SceneLaunch_SeedPlayers(&fixture,false)!=6) return 1;
    for (i=0;i<GM_MAX_PLAYERS;++i) {
        PlayerInitData* p=&fixture.start.players[i];
        if (p->slot!=i+1 || p->ckind!=2 || p->slot_type!=(i ? Gm_PKind_Cpu : Gm_PKind_Human) ||
            p->team!=(i ? 1 : 0) || p->stocks!=i+1 || p->color>=gm_GetNumCostumesForCKind(2) ||
            (i>=4 && (p->spawn_pos!=-1 || p->attack_ratio!=1.0f || p->defense_ratio!=1.0f))) return 1;
    }
    return 0;
}
