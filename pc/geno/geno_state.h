/* geno_state.h - Geno's per-fighter state block (game half only: geno_game.c and geno_tests.c,
 * both compiled through gwtool, so the layout is the same in both). Never included by native code.
 *
 * ROLLBACK: one block per fighter object lives in Geno_StateBlock, a plain game global, so gw_snap
 * saves and restores it with the rest of the game. Only ints and floats; no host pointers. */
#ifndef GENO_STATE_H
#define GENO_STATE_H

#include "geno.h"

typedef struct GenoCond {
    u32 head; /* word0 [15:0] of the CHG / CHGAND: [15:8] cond, [7] B var, [6:4] cmp, [3] NOT */
    u32 arg1;
    u32 arg2;
} GenoCond;

typedef struct GenoCheck {
    u32 target; /* GENO_TARGET word */
    u32 once;   /* 1: tested once, at the end of the frame it was registered in */
    u32 ncond;
    GenoCond cond[GENO_CHECK_CONDS];
} GenoCheck;

typedef struct GenoState {
    s32 profile;     /* registry profile of this fighter's kind, -1 = none */
    s32 kind;        /* the kind it was reset for */
    u32 flags;       /* GENO_SF_* */
    u32 resets;      /* times this block was reset (diagnostics) */
    s32 la_i[GENO_VARS_PER_BANK];
    s32 ra_i[GENO_VARS_PER_BANK];
    f32 la_f[GENO_VARS_PER_BANK];
    f32 ra_f[GENO_VARS_PER_BANK];
    u32 hook_calls;  /* hooks run since the reset */
    u32 extra_jumps; /* air jumps taken past Melee's multi-jump table */
    /* ---- v1 (per action unless noted: cleared on every action change) ---- */
    s32 action_time;                     /* GENO_VAL_ACTION_FRAME */
    u32 nchecks;                         /* registered change-action checks */
    s32 last_check;                      /* the CHG a CHGAND extends, -1 none */
    GenoCheck checks[GENO_MAX_CHECKS];
    s32 rehit_period[GENO_MAX_REHIT];    /* per Melee hitbox id; 0 = off */
    s32 rehit_count[GENO_MAX_REHIT];
    s32 link_mode[GENO_MAX_REHIT];       /* GENO_LINK_* per hitbox id */
    /* landing / take-off edges inside the collision callback (not cleared by action changes) */
    u32 in_coll;
    u32 edge_pending;
    u32 edge_target;
    u32 changes; /* Geno-made action changes since the reset (diagnostics) */
    /* ---- v2 (NOT cleared by action changes: a move's Geno states hand these to each other) ---- */
    s32 hold_motion;                 /* glide entry: the air jump the hold count belongs to */
    s32 hold_frames;                 /* frames jump has been held in it, -1 = released */
    s32 move_i[GENO_MOVE_VARS];      /* behaviour ints (GENO_VAL_MOVE_I0..): timers, counters */
    f32 move_f[GENO_MOVE_VARS];      /* behaviour floats (GENO_VAL_MOVE_F0..): angle, speed */
    u32 state_entries;               /* Geno states entered since the reset (diagnostics) */
} GenoState;

#define GENO_SF_SCRIPT 1u /* a script used the escape since the reset */

#endif
