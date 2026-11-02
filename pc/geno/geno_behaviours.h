/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 GurekamDhillon
 *
 * geno_behaviours.h - the pure data types and entry points of Geno's v5.1 behaviour procedures
 * (glide, tornado, drill, cape, multi-dash, anim_motion). Implemented from
 * docs/specs/geno-v5.1-behaviours.md alone. The step functions are pure: they read plain structs and
 * write plain structs, so the headless tests drive them without a fighter. The engine adapters
 * (geno_behaviours.inc) copy a fighter's state into these structs, run one step, and copy it back.
 *
 * Numeric rules (spec section 1): every simulation value is f32, no fused multiply-add (every
 * product lands in a named temporary before the add), operations in the order the spec writes them.
 */
#ifndef GENO_BEHAVIOURS_H
#define GENO_BEHAVIOURS_H

#include "geno.h"

/* ---- trig (spec 1.1) ---------------------------------------------------------------------------- */
f32 GenoBhv_SinDeg(f32 a);
f32 GenoBhv_CosDeg(f32 a);
/* The committed quarter-wave table: 1025 f32 bit patterns and their CRC32 (over the little-endian bytes). */
#define GENO_BHV_SINQ_N 1025
u32 GenoBhv_SinqWord(int i);
u32 GenoBhv_SinqCrc32(void);

/* ---- stick (spec 1.3) --------------------------------------------------------------------------- */
#define GENO_BHV_DZ2 0.0826f /* 0.2875 squared */
void GenoBhv_Stick(f32 rx, f32 ry, f32* sx, f32* sy);

/* ---- glide (spec 2) ----------------------------------------------------------------------------- */
enum {
    GB_GL_START = 0,
    GB_GL_FLY = 1,
    GB_GL_ATTACK = 2,
    GB_GL_JUMPOUT = 3,
    GB_GL_LAND = 4,        /* landed with lag */
    GB_GL_TRIP = 5,        /* landed too steeply: knocked down */
    GB_GL_END = 6,         /* stall / attack finished: fall */
    GB_GL_LAND_CANCEL = 7  /* landed with zero lag */
};
enum { GB_GLE_NONE = 0, GB_GLE_STALL = 1, GB_GLE_ATTACK_END = 2, GB_GLE_JUMPOUT = 3 };
enum { GB_GLENTRY_NONE = 0, GB_GLENTRY_HOLD = 1, GB_GLENTRY_FLICK = 2, GB_GLENTRY_CHAIN = 3 };

typedef struct GbGlideP {
    s32 max_glides, entry_hold, flick_window, start_frames, trip_frames, land_lag;
    s32 jumpout_frames, attack_frames, attack_hit0, attack_hit1, attack_endlag, attack_buf_frames;
    f32 entry_vy_max, apex_vy, flick_thr;
    f32 s_start, start_accel, s_start_cap, theta_def, r_down, r_up, turn_rate;
    f32 gk, drag, s_max, s_stall, trip_angle, attack_speed_keep, chain_speed_mul, chain_turn_mul;
    s32 attack_damage;
} GbGlideP;

typedef struct GbGlide {
    s32 phase;        /* GB_GL_* */
    s32 t;            /* frames in the phase */
    s32 glides_used;
    s32 attack_buf;
    s32 slow;         /* chained entry */
    s32 helpless;     /* the glide began after the last midair jump */
    s32 end_reason;   /* GB_GLE_*, set when phase becomes GB_GL_END */
    s32 lag;          /* landing lag (GB_GL_LAND) or knockdown length (GB_GL_TRIP) in frames */
    s32 hit_active;   /* the glide attack's hit frames */
    s32 jump_now;     /* the jump-out finished: the host applies the jump */
    s32 endlag;       /* frames of fall after an attack */
    f32 theta, speed, f;
    f32 vx, vy;       /* the velocity the step produced */
} GbGlide;

typedef struct GbGlideIn {
    f32 sx, sy;       /* stick after the dead zone */
    s32 jump_pressed, attack_pressed, special_pressed;
    s32 jump_left;    /* the fighter has a jump left */
    s32 ground;       /* the host reports floor contact this frame */
    f32 height;       /* height above the floor along the path, < 0 = unknown */
} GbGlideIn;

/* The three entry paths (spec 2.2). */
typedef struct GbGlideEntry {
    s32 hold_frames;  /* consecutive frames the jump button has been held since the midair jump, -1 released / none */
    s32 last_dir;     /* -1, 0, +1: the last strong horizontal direction (stick x times facing) */
    s32 age;          /* frames since last_dir */
} GbGlideEntry;

void GenoBhv_GlideParams(GbGlideP* p);
void GenoBhv_GlideEntryInit(GbGlideEntry* e);
/* One frame of entry detection. Returns a GB_GLENTRY_* path (NONE = keep flying). `chain` is the host
 * flag of path C. */
int GenoBhv_GlideEntryStep(GbGlideEntry* e, const GbGlideP* p, s32 midair_jumped, s32 jump_held, f32 vy,
                           f32 sx, f32 f, s32 glides_used, s32 airborne, s32 helpless, s32 chain);
/* Starts GB_GL_START from the fighter's velocity. */
void GenoBhv_GlideBegin(GbGlide* g, const GbGlideP* p, f32 vx, f32 vy, f32 f, f32 sx, s32 slow, s32 helpless,
                        s32 glides_used);
void GenoBhv_GlideStep(GbGlide* g, const GbGlideIn* in, const GbGlideP* p);
/* The ground-contact exit alone (spec 2.4, exit 1): sets phase to LAND / TRIP / LAND_CANCEL and lag. */
void GenoBhv_GlideGround(GbGlide* g, const GbGlideP* p);

/* ---- tornado (spec 3) --------------------------------------------------------------------------- */
enum { GB_TN_START = 0, GB_TN_LOOP = 1, GB_TN_END = 2, GB_TN_DONE = 3 };
typedef struct GbTornadoP {
    s32 start_frames, read_period, read_limit, end_frames, ledge_lock;
    s32 final_hit_ground, final_hit_air; /* last end-animation frame (1-based) the final hit is active */
    s32 loop_damage, final_damage;
    f32 spin0, decay_a, decay_b, mash_add, spin_max, spin_end;
    f32 grav, rise_impulse, rise_max, fall_max, vx_max, vx_max_ground, vx_accel, jump_bonus;
    f32 start_vy_keep, start_vx_keep;
} GbTornadoP;
typedef struct GbTornado {
    s32 phase;       /* GB_TN_* */
    s32 n;           /* frames since the move began */
    s32 k;           /* loop frame */
    s32 end_t;       /* end animation frame, 1-based */
    s32 press_latch, rise_impulse, jump_start, hits;
    s32 hit_spawn;       /* a loop hit spawns this frame */
    s32 final_spawn;     /* the final hit spawns this frame */
    s32 final_active;    /* the final hit is active this frame */
    s32 ledge_locked;
    s32 helpless;        /* set the frame the move ends in the air */
    f32 spin, rot, vx, vy;
} GbTornado;
typedef struct GbTornadoIn {
    f32 sx;
    s32 special_pressed, jump_held, airborne;
} GbTornadoIn;
void GenoBhv_TornadoParams(GbTornadoP* p);
void GenoBhv_TornadoBegin(GbTornado* t, f32 vx, f32 vy);
void GenoBhv_TornadoStep(GbTornado* t, const GbTornadoIn* in, const GbTornadoP* p);

/* ---- drill (spec 4) ----------------------------------------------------------------------------- */
enum { GB_DR_WIND = 0, GB_DR_RUSH = 1, GB_DR_FINAL = 2, GB_DR_DONE = 3 };
typedef struct GbDrillP {
    s32 move0, ramp, hit0, hit_period, nhits, hit_end, final_frame, final_end_ground, final_end_air, total;
    s32 rush_damage, final_damage;
    f32 dive_default, psi_stick, psi_min, psi_max, steer_rate, speed, recoil_vy, end_vx_keep;
} GbDrillP;
typedef struct GbDrill {
    s32 phase;       /* GB_DR_* */
    s32 n;
    s32 hit_spawn;   /* a rushing hit spawns this frame */
    s32 hits;
    s32 final_active;
    s32 helpless;    /* set the frame the move ends in the air */
    s32 done;
    f32 psi, speed, f, vx, vy;
} GbDrill;
typedef struct GbDrillIn {
    f32 sy;
    s32 airborne;
    f32 slope;       /* floor slope in degrees (grounded) */
} GbDrillIn;
void GenoBhv_DrillParams(GbDrillP* p);
void GenoBhv_DrillBegin(GbDrill* d, const GbDrillP* p, f32 f, f32 sy);
void GenoBhv_DrillStep(GbDrill* d, const GbDrillIn* in, const GbDrillP* p);

/* ---- cape (spec 5) ------------------------------------------------------------------------------ */
typedef struct GbCapeP {
    s32 start, end, total;
    f32 turn_thr, stall_x, stall_y_up, stall_vy_hi, stall_grav, stall_fall, ground_x;
    f32 reflect_dmg_mul, reflect_speed_mul;
} GbCapeP;
typedef struct GbCape {
    s32 n;
    s32 flipped;     /* the facing flip was decided (and taken or refused) */
    s32 flip_now;    /* the host flips the facing this frame */
    s32 reflect;     /* the reflect window is open this frame */
    s32 done;
    f32 f, vx, vy;
} GbCape;
typedef struct GbCapeIn {
    f32 sx;
    s32 airborne;
} GbCapeIn;
void GenoBhv_CapeParams(GbCapeP* p);
void GenoBhv_CapeBegin(GbCape* c, f32 f, f32 vx, f32 vy);
void GenoBhv_CapeStep(GbCape* c, const GbCapeIn* in, const GbCapeP* p);

/* ---- multi-dash (spec 6) ------------------------------------------------------------------------ */
enum { GB_MD_AIM = 0, GB_MD_DASH = 1, GB_MD_PAUSE = 2, GB_MD_END = 3, GB_MD_DONE = 4 };
typedef struct GbMultiDashP {
    s32 max_seg, aim_first, aim_later, dash_frames, pause_frames, repress_frames, end_lag;
    f32 dash_speed, end_keep, aim_slow, ay_max, pause_keep;
} GbMultiDashP;
typedef struct GbMultiDash {
    s32 seg, phase, t;
    s32 repress;      /* special seen inside the re-press window */
    s32 hit_spawn;    /* a dash hit spawns this frame (host: DASH_HIT(seg)) */
    s32 helpless;     /* set when the move ends: the last segment was used */
    s32 done;
    f32 ax, ay, f, speed, vx, vy;
    f32 dist;         /* path length travelled in the current segment (diagnostic) */
} GbMultiDash;
typedef struct GbMultiDashIn {
    f32 sx, sy;       /* stick after the dead zone */
    s32 special_pressed;
} GbMultiDashIn;
void GenoBhv_MultiDashParams(GbMultiDashP* p);
void GenoBhv_MultiDashBegin(GbMultiDash* m, f32 f);
void GenoBhv_MultiDashStep(GbMultiDash* m, const GbMultiDashIn* in, const GbMultiDashP* p);

/* ---- anim_motion (spec 7) ----------------------------------------------------------------------- */
enum { GB_AM_DELTA = 0, GB_AM_ABSOLUTE = 1 };
typedef struct GbAnimMotionP {
    f32 rate, scale, grav;
    s32 mode;        /* GB_AM_* */
    s32 gravity_on, ignore_y, loop;
} GbAnimMotionP;
typedef struct GbAnimMotion {
    f32 anim_t;
    f32 vy_carry;
    f32 origin_x, origin_y;
    f32 vx, vy;
    f32 pos_x, pos_y;   /* the position the step requests */
    s32 ended;
    s32 started;
} GbAnimMotion;
/* The track is an array of (x, y) pairs, n samples. */
void GenoBhv_AnimMotionSample(const f32* track, int n, f32 t, f32* x, f32* y);
void GenoBhv_AnimMotionParams(GbAnimMotionP* p);
void GenoBhv_AnimMotionBegin(GbAnimMotion* a, f32 pos_x, f32 pos_y);
/* One frame. `blocked` = the host reports a wall hit for the previous request. */
void GenoBhv_AnimMotionStep(GbAnimMotion* a, const f32* track, int n, const GbAnimMotionP* p, f32 facing,
                            s32 blocked);
/* One engine frame of geno.anim_motion: the clip's sampled root delta (fwd, up) through one DELTA step. */
void GenoBhv_AnimMotionFrameDelta(f32 fwd, f32 up, f32 lr, f32* dx, f32* dy);

#endif
