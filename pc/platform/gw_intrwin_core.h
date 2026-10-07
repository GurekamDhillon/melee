/* Shared interrupt-window state core. Included by the PPC simulation and native fixtures.
 * Storage remains in script_game.c BSS; this header introduces no native simulation state.
 * Extracted from script_fighter_interrupt.inc without changing the rule decisions. */
#ifndef GW_INTRWIN_CORE_H
#define GW_INTRWIN_CORE_H
#include "gw_matchrules.h"

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

/* Scalar view of simulation state; no pointers into native memory cross the shim boundary. */
typedef struct IntrWinAttack {
    int key;
    int motion_id;
    int x2068_attackID;
    int x206C_attack_instance;
    int air;
} IntrWinAttack;

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
static void intrwin_close(IntrWin* w, int cancelled)
{
    if (cancelled) w->cancels++;
    w->ind = 0;
    w->frames = 0;
    w->owner = 0;
}

/* Called only after exits failed on a free logic frame (never during hitlag).
 * The game caller clears the colour overlay before closing an expired window. */
static int intrwin_age(IntrWin* w) { return --w->frames <= 0; }

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

/* The core of opening a rule window: pure state, no engine calls.
 * Returns 1 if opened, 0 if ignored, -1 for a movement-loop refusal to log.
 * Caller validates fighter/entity and supplies a window and scalar attack view. */
static int intrwin_open_state(IntrWin* w, const IntrWinAttack* fp, int kind, unsigned rules, int* jumps)
{
    int key;
    if (!(rules & GW_TURBO_ON)) return 0;
    if (kind == IW_OPEN_SHIELD && !(rules & GW_TURBO_SHIELD_HIT)) return 0;
    if (kind == IW_OPEN_ITEM && !(rules & GW_TURBO_ITEM_HIT)) return 0;
    key = fp->key;
    if (key == IW_K_NONE) return 0; /* throws, grabs, item swings: not a move this rule knows */
    if ((rules & GW_TURBO_NO_MOVE_LOOP) && w->carry_frames > 0) {
        if (w->carry_key == key) {
            /* the same move again through a movement cancel: no window, and the memory re-arms for a full window length, so
             * the rest of a chain (jab 2, the rapid jab) that keeps landing without a different attack cannot reopen it */
            w->carry_frames = gw_turbo_frames(rules);
            return -1; /* report a movement-loop refusal outside the state core */
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
    if ((rules & GW_TURBO_AIR_JUMPS) && fp->air && *jumps > 1)
        *jumps = 1;
    return 1;
}

#endif
