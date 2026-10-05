/* script_fighter_interrupt.inc - the INTERRUPT WINDOW: a fighter whose attack has connected may
 * cancel the rest of that attack into a wider set of actions for a short time. The match rule
 * TURBO (gw_matchrules.h) opens it on every connecting hit; scripts can open one offline
 * (ScriptGame_IntrWinSet). Included by script_game.c after the fighter-caps files.
 * Prior art, ideas only: Project M / Project+ Turbo (Turbo.asm, "On Hit Interrupts") and
 * UnclePunch's Melee turbo code for 20XX. See _research/turbo-mode-projectm-2026-10-04.md.
 *
 * WHERE IT HOOKS
 *   open   ftcoll.c: a fighter hitbox connects (ftColl_80076808 caller, next to LAB_EV_HIT), a hit
 *          lands on a shield (ftColl_80076CBC), an item/projectile hit lands (item loop) ->
 *          ScriptGame_IntrWinHit(attacker, kind).
 *   run    fighter.c, the per-frame input process, right after `fp->input_cb(gobj)` and inside the
 *          same `!x2219_b5` (not in hitlag) block -> ScriptGame_IntrWinTick(gobj).
 *   hash   RB_GameHash mixes each window, so a divergence is a desync the checksum sees at once.
 *
 * STATE is BSS in this translation unit (like the caps records), so it is inside the savestate and
 * the rollback snapshot; ScriptGame_StageEnd clears it at every scene end. Nothing is read from
 * the host: the rule word comes from gw_MatchTurboRules() (native, constant for a match, agreed by
 * both peers in the handshake), everything else is simulation state, decided in a fixed order.
 *
 * DECISIONS (documented, per the packet)
 *  - HITLAG: the window is NOT usable during hitlag. Retail skips the input callback in hitlag
 *    (fighter.c `!x2219_b5`) and so does this; hitlag frames do not count against the window, so
 *    the player always gets `frames` free logic frames after hitlag ends. (GW_TURBO_HITLAG is
 *    reserved: a word carrying it is refused.)
 *  - WHAT RUNS: the action's own input callback runs first, untouched (its retail IASA, the jab
 *    chain, natural transitions). Only if the action did not change does the window's shared exit
 *    list run, in the order of the neutral states (Wait for the ground, Fall for the air). The
 *    window never sets `allow_interrupt`: each exit is vetted by the rules below instead of
 *    exposing whatever list the action's own IASA happens to carry (an aerial's would otherwise
 *    offer the same aerial again).
 *  - REMAINING HITBOXES: the exits are retail's own state entries, so a cancel clears the move's
 *    hitboxes exactly as any retail IASA exit does. The window itself closes the moment the action
 *    changes for any reason (a cancel, the move ending, being hit, landing).
 *  - LANDING LAG: unchanged. Landing ends the action, which closes the window; Turbo does not
 *    shorten landing lag (P+'s auto-cancel on a connected aerial is not built).
 *  - THROWS: never grant. Throws and grabs do not go through the damage hit this hooks.
 *  - MULTI-HIT: the first connecting hit of a move opens the window; later hits of the same move
 *    instance neither refresh nor reopen it (x2068_attackID + x206C_attack_instance + action).
 *  - SELF-CANCEL: the key of each exit is known before it runs (the move it would enter), so a
 *    move that grants a window never cancels into the same move (jab/tilt/smash/aerial by name).
 *    A special's exit cannot be named beforehand, so with NO_SELF a special's window offers no
 *    special at all (stricter than P+, which exempts multi-action specials).
 */
#include <melee/ft/kinds/ftCommon/ftCo_AirCatch.h>
#include <melee/ft/kinds/ftCommon/ftCo_Attack1.h>
#include <melee/ft/kinds/ftCommon/ftCo_Attack100.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackAir.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackHi3.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackHi4.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackLw3.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackLw4.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackS3.h>
#include <melee/ft/kinds/ftCommon/ftCo_AttackS4.h>
#include <melee/ft/kinds/ftCommon/ftCo_Dash.h>
#include <melee/ft/kinds/ftCommon/ftCo_Escape.h>
#include <melee/ft/kinds/ftCommon/ftCo_EscapeAir.h>
#include <melee/ft/kinds/ftCommon/ftCo_Guard.h>
#include <melee/ft/kinds/ftCommon/ftCo_Jump.h>
#include <melee/ft/kinds/ftCommon/ftCo_JumpAerial.h>
#include <melee/ft/kinds/ftCommon/ftCo_SpecialAir.h>
#include <melee/ft/kinds/ftCommon/ftCo_SpecialS.h>
#include <melee/ft/kinds/ftCommon/ftCo_Squat.h>
#include <melee/ft/kinds/ftCommon/ftCo_Turn.h>
#include <melee/ft/kinds/ftCommon/ftCo_Walk.h>
#include <melee/ft/ftcolanim.h>
#include <melee/ft/ft_0DF1.h>
#include <melee/ft/ftcommon.h>
#include "../platform/gw_matchrules.h"

extern int MatchTurboRules(void);     /* native: the rule word in force for this match */
extern int MatchTurboColAnim(void);   /* native: the indicator's colour animation id */
extern void MatchTurboNote(int what); /* native counters (not rolled back): 0 window opened, 1 cancel taken, 2 repeat refused by NO_MOVE_LOOP, 3 window opened after resim marker */
extern bool ftCo_Catch_CheckInput(Fighter_GObj* gobj);

/* the moves a window can come from (a key), and the classes of exit */
enum {
    IW_K_NONE, IW_K_JAB, IW_K_DASHATK, IW_K_FTILT, IW_K_UTILT, IW_K_DTILT, IW_K_FSMASH,
    IW_K_USMASH, IW_K_DSMASH, IW_K_NAIR, IW_K_FAIR, IW_K_BAIR, IW_K_UAIR, IW_K_DAIR, IW_K_SPECIAL
};
enum {
    IW_X_JAB = 1 << 0, IW_X_TILT = 1 << 1, IW_X_SMASH = 1 << 2, IW_X_AERIAL = 1 << 3,
    IW_X_SPECIAL = 1 << 4, IW_X_GRAB = 1 << 5, IW_X_JUMP = 1 << 6, IW_X_DASH = 1 << 7,
    IW_X_CROUCH = 1 << 8, IW_X_TURN = 1 << 9, IW_X_WALK = 1 << 10, IW_X_ESCAPE = 1 << 11,
    IW_X_SHIELD = 1 << 12, IW_X_AIRDODGE = 1 << 13, IW_X_AIRJUMP = 1 << 14, IW_X_ALL = 0x7FFF
};
enum { IW_OPEN_HIT, IW_OPEN_SHIELD, IW_OPEN_ITEM };

/* THE INDICATOR, tuned here: the game's own colour overlay (colanim id from settings.cfg
 * `turbo_colanim`, default in gw_runtime.c) started ONCE when the window opens, and cleared after
 * IW_IND_FRAMES free (non-hitlag) frames, or the frame the window closes or is consumed. Never
 * re-applied while the window is open. 0 frames = no indicator. */
#define IW_IND_FRAMES 6


typedef struct IntrWin {
    int owner;        /* 0 = the match rule; else the scripting entry that opened it */
    int frames;       /* free logic frames left (hitlag does not count); 0 = closed */
    int motion;       /* the action the window belongs to */
    int key;          /* the move that granted it (IW_K_*) */
    int exits;        /* IW_X_* classes the window offers */
    int rules;        /* GW_TURBO_* bits in force for this window */
    int last_attack;  /* the last move instance that granted one (multi-hit guard) */
    int last_inst;
    int last_motion;
    int opened;       /* counters, for the log and the tests */
    int cancels;
    int ind;          /* indicator frames left (0 = none showing) */
    int taken;        /* the exit class of the last cancel (IW_X_*), for the movement memory */
    int carry_key;    /* NO_MOVE_LOOP: the move remembered across a movement cancel (IW_K_*), 0 = none */
    int carry_frames; /* ... for this many more free frames */
} IntrWin;
static IntrWin ft_intrwin[12];
static int ft_intrwin_logged;

static int intrwin_key(int motion)
{
    if (motion >= ftCo_MS_Count) return IW_K_SPECIAL;
    switch (motion) {
    case ftCo_MS_Attack11: case ftCo_MS_Attack12: case ftCo_MS_Attack13:
    case ftCo_MS_Attack100Start: case ftCo_MS_Attack100Loop: case ftCo_MS_Attack100End:
        return IW_K_JAB;
    case ftCo_MS_AttackDash: return IW_K_DASHATK;
    case ftCo_MS_AttackS3Hi: case ftCo_MS_AttackS3HiS: case ftCo_MS_AttackS3S:
    case ftCo_MS_AttackS3LwS: case ftCo_MS_AttackS3Lw: return IW_K_FTILT;
    case ftCo_MS_AttackHi3: return IW_K_UTILT;
    case ftCo_MS_AttackLw3: return IW_K_DTILT;
    case ftCo_MS_AttackS4Hi: case ftCo_MS_AttackS4HiS: case ftCo_MS_AttackS4S:
    case ftCo_MS_AttackS4LwS: case ftCo_MS_AttackS4Lw: return IW_K_FSMASH;
    case ftCo_MS_AttackHi4: return IW_K_USMASH;
    case ftCo_MS_AttackLw4: return IW_K_DSMASH;
    case ftCo_MS_AttackAirN: return IW_K_NAIR;
    case ftCo_MS_AttackAirF: return IW_K_FAIR;
    case ftCo_MS_AttackAirB: return IW_K_BAIR;
    case ftCo_MS_AttackAirHi: return IW_K_UAIR;
    case ftCo_MS_AttackAirLw: return IW_K_DAIR;
    }
    return IW_K_NONE;
}
static int intrwin_key_smash(int k) { return k == IW_K_FSMASH || k == IW_K_USMASH || k == IW_K_DSMASH; }
static int intrwin_key_aerial(int k) { return k >= IW_K_NAIR && k <= IW_K_DAIR; }

/* The rules, as a pure function (the native tests drive it): may a window granted by a move with
 * `key` offer exit class `cls`, with the rule bits `rules`? `cand` is the key of the move the exit
 * would enter (IW_K_NONE when it is not a named move). */
static int intrwin_allows(unsigned rules, int key, int ground, int cls, int cand)
{
    if ((rules & GW_TURBO_NO_SELF) && cand != IW_K_NONE && cand == key) return 0;
    if ((rules & GW_TURBO_NO_SELF) && key == IW_K_SPECIAL && cls == IW_X_SPECIAL) return 0;
    if ((rules & GW_TURBO_DASH_JAB) && cls == IW_X_DASH && key != IW_K_JAB && key != IW_K_DASHATK) return 0;
    if ((rules & GW_TURBO_SMASH_STILL) && intrwin_key_smash(key) &&
        (cls == IW_X_CROUCH || cls == IW_X_JUMP || cls == IW_X_DASH)) return 0;
    if ((rules & GW_TURBO_NO_SHIELD) && ground && key != IW_K_NONE && cls == IW_X_SHIELD) return 0;
    if ((rules & GW_TURBO_NO_STEER) && (cls == IW_X_WALK || cls == IW_X_TURN)) return 0;
    if ((rules & GW_TURBO_NO_AIRDODGE) && (intrwin_key_aerial(key) || key == IW_K_SPECIAL) &&
        cls == IW_X_AIRDODGE) return 0;
    return 1;
}
int ScriptGame_IntrWinAllows(int rules, int key, int ground, int cls, int cand)
{
    return intrwin_allows((unsigned) rules, key, ground, cls, cand);
}

static void intrwin_close(IntrWin* w, int cancelled)
{
    if (cancelled) w->cancels++;
    w->ind = 0;
    w->frames = 0;
    w->owner = 0;
}

/* NO_MOVE_LOOP: a cancel that took a movement exit (crouch, dash, jump, air jump) remembers the granting move for the
 * rest of the window's length; the memory runs down on free (non-hitlag) frames, one per tick. Pure state: tested. */
static void intrwin_carry_set(IntrWin* w)
{
    if ((w->rules & GW_TURBO_NO_MOVE_LOOP) &&
        (w->taken & (IW_X_CROUCH | IW_X_DASH | IW_X_JUMP | IW_X_AIRJUMP)) && w->frames > 1) {
        w->carry_key = w->key;
        w->carry_frames = w->frames - 1;
    }
}
static void intrwin_carry_tick(IntrWin* w)
{
    if (w->carry_frames > 0 && --w->carry_frames == 0) w->carry_key = 0;
}

/* The core of opening a rule window: pure state, no engine calls (the tests drive it with a fake
 * fighter). Returns 1 if a window opened. */
static int intrwin_open(int entity, Fighter* fp, int kind, unsigned rules)
{
    int key;
    IntrWin* w;
    if (!(rules & GW_TURBO_ON) || fp == NULL || entity < 1 || entity > 12) return 0;
    if (kind == IW_OPEN_SHIELD && !(rules & GW_TURBO_SHIELD_HIT)) return 0;
    if (kind == IW_OPEN_ITEM && !(rules & GW_TURBO_ITEM_HIT)) return 0;
    key = intrwin_key(fp->motion_id);
    if (key == IW_K_NONE) return 0; /* throws, grabs, item swings: not a move this rule knows */
    w = &ft_intrwin[entity - 1];
    if ((rules & GW_TURBO_NO_MOVE_LOOP) && w->carry_frames > 0) {
        if (w->carry_key == key) {
            /* the same move again through a movement cancel: no window, and the memory re-arms for a full window length, so
             * the rest of a chain (jab 2, the rapid jab) that keeps landing without a different attack cannot reopen it */
            w->carry_frames = gw_turbo_frames(rules);
            MatchTurboNote(2);
            return 0;
        }
        w->carry_key = 0; w->carry_frames = 0;                    /* a different attack landed: the memory ends */
    }
    if (w->last_attack == fp->x2068_attackID && w->last_inst == (int) fp->x206C_attack_instance &&
        w->last_motion == fp->motion_id && w->opened > 0)
        return 0; /* the same move instance already granted one: a multi-hit counts once */
    if (w->frames > 0) return 0; /* already open (or a script's window is running): never stomped */
    w->owner = 0;
    w->frames = gw_turbo_frames(rules);
    w->motion = fp->motion_id;
    w->key = key;
    w->exits = IW_X_ALL;
    w->rules = (int) rules;
    w->last_attack = fp->x2068_attackID;
    w->last_inst = (int) fp->x206C_attack_instance;
    w->last_motion = fp->motion_id;
    w->opened++;
    MatchTurboNote(0);
    if ((rules & GW_TURBO_AIR_JUMPS) && fp->ground_or_air == GA_Air && fp->x1968_jumpsUsed > 1)
        fp->x1968_jumpsUsed = 1;
    return 1;
}

static void intrwin_clear_ind(Fighter* fp, IntrWin* w)
{
    if (w->ind > 0) ftCo_800C0074(fp);
    w->ind = 0;
}

/* Open a rule window for `fp`, whose attack just connected. kind: IW_OPEN_*. */
void ScriptGame_IntrWinHit(Fighter* fp, int kind)
{
    unsigned rules = (unsigned) MatchTurboRules();
    int entity;
    if (!(rules & GW_TURBO_ON) || fp == NULL) return;
    entity = ScriptGame_CapsEntity(fp);
    if (!intrwin_open(entity, fp, kind, rules)) return;
    /* the indicator: the game's own colour overlay; deterministic, applied once per window */
    if (IW_IND_FRAMES > 0) {
        ftCo_800BFFD0(fp, (enum_t) MatchTurboColAnim(), 0);
        ft_intrwin[entity - 1].ind = IW_IND_FRAMES;
    }
    if (ft_intrwin_logged < 4000 || (ft_intrwin[entity - 1].opened & 63) == 0) {
        ft_intrwin_logged++;
        OSReport("turbo: window open entity=%d key=%d motion=%d kind=%d frames=%d air=%d\n", entity,
                 ft_intrwin[entity - 1].key, fp->motion_id, kind, ft_intrwin[entity - 1].frames,
                 fp->ground_or_air == GA_Air);
    }
}

/* Which grounded attack the pressed buttons ask for this frame, by retail's own predicates
 * (ftCo_AttackS4/Hi4/Lw4/S3/Hi3/Lw3/Attack1_CheckInput) and its priority: a smash flick (A with a
 * fresh stick flick, or the C-stick), then a tilt (A with the stick past the tilt threshold), then
 * a jab (A alone). IW_K_NONE if no attack button was pressed. */
static int intrwin_attack_want(Fighter* fp)
{
    int a = (fp->input.pressed_buttons & HSD_PAD_A) != 0;
    if (a && ABS(fp->input.lstick[0].x) >= p_ftCommonData->dash_smash_stick_threshold &&
        fp->active_timer.lstick.x < p_ftCommonData->dash_smash_window) return IW_K_FSMASH;
    if (a && fp->input.lstick[0].y >= p_ftCommonData->xCC &&
        fp->active_timer.lstick.y < p_ftCommonData->xD0) return IW_K_USMASH;
    if (a && fp->input.lstick[0].y <= p_ftCommonData->xD4 &&
        fp->active_timer.lstick.y < p_ftCommonData->xD8) return IW_K_DSMASH;
    if (ftCo_800DF1C8(fp)) return IW_K_FSMASH;
    if (ftCo_800DF2D8(fp)) return IW_K_USMASH;
    if (ftCo_800DF3A8(fp)) return IW_K_DSMASH;
    if (!a) return IW_K_NONE;
    if (fp->input.lstick[0].x * fp->facing_dir >= p_ftCommonData->x98 &&
        ABS(ftCo_GetLStickAngle(fp)) < p_ftCommonData->x20_radians) return IW_K_FTILT;
    if (fp->input.lstick[0].y >= p_ftCommonData->attackhi3_stick_threshold_y &&
        ftCo_GetLStickAngle(fp) > p_ftCommonData->x20_radians) return IW_K_UTILT;
    if (fp->input.lstick[0].y <= p_ftCommonData->xB0 &&
        ftCo_GetLStickAngle(fp) < -p_ftCommonData->x20_radians) return IW_K_DTILT;
    return IW_K_JAB;
}

#define IW_TRY(w, fp, cls, cand, call)                                                              \
    do {                                                                                            \
        if ((w)->exits & (cls) &&                                                                   \
            intrwin_allows((unsigned) (w)->rules, (w)->key, ground, (cls), (cand)) && (call)) {    \
            (w)->taken = (cls);                                                                     \
            return 1;                                                                               \
        }                                                                                           \
    } while (0)

/* The shared exit list. Ground: Wait's order (ftCo_Wait_IASA); air: Fall's (ftCo_Fall_IASA_Inner).
 * Returns 1 if an exit was taken. Taunts, item pickup/throw and Peach's float are left out. */
static int intrwin_exits(Fighter_GObj* gobj, Fighter* fp, IntrWin* w)
{
    int ground = fp->ground_or_air == GA_Ground;
    if (ground) {
        IW_TRY(w, fp, IW_X_SPECIAL, IW_K_NONE, ftCo_SpecialS_CheckInput(gobj));
        IW_TRY(w, fp, IW_X_SPECIAL, IW_K_NONE, ftCo_Attack100_CheckInput(gobj)); /* special hi */
        IW_TRY(w, fp, IW_X_SPECIAL, IW_K_NONE, ftCo_800D6824(gobj));             /* special n */
        IW_TRY(w, fp, IW_X_SPECIAL, IW_K_NONE, ftCo_800D68C0(gobj));             /* special lw */
        IW_TRY(w, fp, IW_X_GRAB, IW_K_NONE, ftCo_Catch_CheckInput(gobj));
        {
            /* The attack buttons: decide WHICH move the press asks for first (retail's own tests,
             * in retail's priority: smash, tilt, jab), then run that exit only if the rules allow
             * it. A press for a vetoed move does nothing: it never falls through to a lower
             * priority exit (an ftilt press that the guard refuses must not become a jab). */
            int want = intrwin_attack_want(fp);
            if (want != IW_K_NONE) {
                int cls = intrwin_key_smash(want) ? IW_X_SMASH : want == IW_K_JAB ? IW_X_JAB : IW_X_TILT;
                switch (want) {
                case IW_K_FSMASH: IW_TRY(w, fp, cls, want, ftCo_AttackS4_CheckInput(gobj)); break;
                case IW_K_USMASH: IW_TRY(w, fp, cls, want, ftCo_AttackHi4_CheckInput(gobj)); break;
                case IW_K_DSMASH: IW_TRY(w, fp, cls, want, ftCo_AttackLw4_CheckInput(gobj)); break;
                case IW_K_FTILT: IW_TRY(w, fp, cls, want, ftCo_AttackS3_CheckInput(gobj)); break;
                case IW_K_UTILT: IW_TRY(w, fp, cls, want, ftCo_AttackHi3_CheckInput(gobj)); break;
                case IW_K_DTILT: IW_TRY(w, fp, cls, want, ftCo_AttackLw3_CheckInput(gobj)); break;
                default: IW_TRY(w, fp, cls, want, ftCo_Attack1_CheckInput(gobj)); break;
                }
            }
        }
        IW_TRY(w, fp, IW_X_ESCAPE, IW_K_NONE, ftCo_80099794(gobj));              /* roll / spot dodge */
        IW_TRY(w, fp, IW_X_SHIELD, IW_K_NONE, ftCo_80091A4C(gobj));
        IW_TRY(w, fp, IW_X_JUMP, IW_K_NONE, ftCo_Jump_CheckInput(gobj));
        IW_TRY(w, fp, IW_X_DASH, IW_K_NONE, ftCo_Dash_CheckInput(gobj));
        IW_TRY(w, fp, IW_X_CROUCH, IW_K_NONE, ftCo_800D5FB0(gobj));
        IW_TRY(w, fp, IW_X_TURN, IW_K_NONE, ftCo_Turn_CheckInput(gobj));
        IW_TRY(w, fp, IW_X_WALK, IW_K_NONE, ftCo_Walk_CheckInput(gobj));
    } else {
        int amsid = ftCo_AttackAir_GetMsidFromCStick(fp);
        int akey = amsid >= ftCo_MS_AttackAirN && amsid <= ftCo_MS_AttackAirLw
                       ? intrwin_key(amsid) : IW_K_NONE;
        IW_TRY(w, fp, IW_X_SPECIAL, IW_K_NONE, ftCo_SpecialAir_CheckInput(gobj));
        IW_TRY(w, fp, IW_X_GRAB, IW_K_NONE, ftCo_800C3B10(gobj));                /* tether / zair */
        IW_TRY(w, fp, IW_X_AIRDODGE, IW_K_NONE, ftCo_80099A58(gobj));
        IW_TRY(w, fp, IW_X_AERIAL, akey, ftCo_AttackAir_CheckItemThrowInput(gobj));
        IW_TRY(w, fp, IW_X_AIRJUMP, IW_K_NONE, ftCo_800CB870(gobj));
    }
    return 0;
}

/* fighter.c, right after fp->input_cb, outside hitlag. */
void ScriptGame_IntrWinTick(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    int entity;
    IntrWin* w;
    if (fp == NULL || fp->player_id < 0 || fp->player_id > 5) return;
    /* the entity number without the 12-way search: port + 1, +6 for the follower (Nana) */
    entity = fp->player_id + 1 + (fp->is_sub_fighter ? 6 : 0);
    w = &ft_intrwin[entity - 1];
    if (w->ind > 0 && --w->ind == 0) ftCo_800C0074(fp); /* the overlay's own list: stop it */
    intrwin_carry_tick(w);
    if (w->frames <= 0) return;
    if (fp->motion_id != w->motion) { intrwin_clear_ind(fp, w); intrwin_close(w, 0); return; }
    if (intrwin_exits(gobj, fp, w)) {
        if (ft_intrwin_logged < 4000) {
            ft_intrwin_logged++;
            OSReport("turbo: cancel entity=%d from key=%d (motion %d) into motion %d\n", entity, w->key,
                     w->motion, fp->motion_id);
        }
        MatchTurboNote(1);
        intrwin_carry_set(w);
        intrwin_clear_ind(fp, w);
        intrwin_close(w, 1);
        return;
    }
    if (--w->frames <= 0) { intrwin_clear_ind(fp, w); intrwin_close(w, 0); }
}

/* A window opened by a script (offline only; the Lua wrapper enforces that). frames 0 closes it.
 * exits: IW_X_* mask. flags bit 0: guard (no self-cancel), bit 1: restore air jumps. A window
 * belongs to one script; another cannot replace it. Returns 1 on success. */
int ScriptGame_IntrWinSet(int entity, int owner, int frames, int exits, int flags)
{
    Fighter* fp = ScriptGame_CapsFighter(entity);
    IntrWin* w;
    if (entity < 1 || entity > 12 || owner <= 0 || frames < 0 || frames > GW_TURBO_MAX_FRAMES * 30 ||
        exits < 0 || exits > IW_X_ALL || flags < 0 || flags > 3 || fp == NULL) return 0;
    w = &ft_intrwin[entity - 1];
    if (w->frames > 0 && w->owner && w->owner != owner) return 0;
    if (frames == 0) { if (w->owner == owner) intrwin_close(w, 0); return 1; }
    w->owner = owner;
    w->frames = frames;
    w->motion = fp->motion_id;
    w->key = intrwin_key(fp->motion_id);
    w->exits = exits;
    w->rules = (flags & 1) ? GW_TURBO_NO_SELF : 0;
    w->opened++;
    if ((flags & 2) && fp->ground_or_air == GA_Air && fp->x1968_jumpsUsed > 1) fp->x1968_jumpsUsed = 1;
    OSReport("script interrupt: entity=%d owner=%d frames=%d exits=0x%x flags=%d key=%d\n", entity,
             owner, frames, exits, flags, w->key);
    return 1;
}

/* field: 0 frames left, 1 owner, 2 exits, 3 granting key, 4 opened (count), 5 cancels (count) */
int ScriptGame_IntrWinRead(int entity, int field)
{
    IntrWin* w;
    if (entity < 1 || entity > 12) return 0;
    w = &ft_intrwin[entity - 1];
    switch (field) {
    case 0: return w->frames;
    case 1: return w->owner;
    case 2: return w->frames > 0 ? w->exits : 0;
    case 3: return w->frames > 0 ? w->key : 0;
    case 4: return w->opened;
    case 5: return w->cancels;
    }
    return 0;
}

void ScriptGame_IntrWinRelease(int owner)
{
    int i;
    if (!owner) { /* a scene ended: what the rule did this match (the final timeline's counters) */
        for (i = 0; i < 12; ++i)
            if (ft_intrwin[i].opened)
                OSReport("turbo: summary entity=%d windows=%d cancels=%d\n", i + 1, ft_intrwin[i].opened,
                         ft_intrwin[i].cancels);
    }
    for (i = 0; i < 12; ++i)
        if (!owner || ft_intrwin[i].owner == owner) memset(&ft_intrwin[i], 0, sizeof ft_intrwin[i]);
}

/* RB_GameHash: one word per entity; 0 when nothing ever opened (so a match without the rule
 * hashes exactly as before). */
unsigned ScriptGame_IntrWinHashWord(int entity)
{
    IntrWin* w;
    if (entity < 1 || entity > 12) return 0;
    w = &ft_intrwin[entity - 1];
    if (w->opened == 0) return 0;
    /* everything that steers a later decision is in the word: the open window, the counters, the repeat guard's memory of
     * the last move instance, and the NO_MOVE_LOOP memory (0 when unused) */
    return ((unsigned) w->frames << 24) ^ ((unsigned) w->opened << 12) ^ ((unsigned) w->cancels << 4) ^
           (unsigned) w->motion ^ (unsigned) w->key << 16 ^
           ((unsigned) w->last_inst * 0x85EBCA6Bu) ^ ((unsigned) w->last_attack * 0xC2B2AE35u) ^
           ((unsigned) w->last_motion * 0x27D4EB2Fu) ^
           ((unsigned) (w->carry_key * 131 + w->carry_frames) * 0x9E3779B1u);
}

#include "script_fighter_interrupt_tests.inc"
