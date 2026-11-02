/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 GurekamDhillon
 *
 * geno_behaviour_params.h - the script-visible parameter set of Geno's v5.1 behaviour procedures
 * (glide, tornado, drill, cape). Defined ONLY from docs/specs/geno-v5.1-behaviours.md (sections 2.5,
 * 3.2-3.3, 4.1-4.2, 5.1-5.2 and the section 9 revision 1 values); the two host-side glide switches
 * and the derived timing entries are documented in that spec's section 10 addendum.
 *
 * geno.json sets them by family block and spec name: "glide": { "s_start": 1.6, "max_glides": 1 }.
 * Each list entry is X(FAMILY, struct field, C type, default, "script name"); the struct field is the
 * GbXxxP member in geno_behaviours.h, the default is the spec value (source tag in the comment).
 * The ids (GENO_BP_<FAMILY>_<field>) are stable within a build only; scripts use the names.
 */
#ifndef GENO_BEHAVIOUR_PARAMS_H
#define GENO_BEHAVIOUR_PARAMS_H

/* per-profile parameter storage (ids below must stay under this) */
#define GENO_PARAMS 128

/* glide: spec 2.2-2.5, section 9 (S_START 1.50 rev 1) */
#define GENO_BP_GLIDE_LIST(X)                                                   \
    X(GLIDE, max_glides, s32, 2, "max_glides")               /* [W:Glide] */  \
    X(GLIDE, entry_hold, s32, 6, "entry_hold")               /* [E] */        \
    X(GLIDE, flick_window, s32, 12, "flick_window")          /* [E] */        \
    X(GLIDE, start_frames, s32, 12, "start_frames")          /* [E] */        \
    X(GLIDE, trip_frames, s32, 30, "trip_frames")            /* [E] */        \
    X(GLIDE, land_lag, s32, 14, "land_lag")                  /* [E] */        \
    X(GLIDE, jumpout_frames, s32, 8, "jumpout_frames")       /* [E] */        \
    X(GLIDE, attack_frames, s32, 28, "attack_frames")        /* [E] */        \
    X(GLIDE, attack_hit0, s32, 8, "attack_hit0")             /* [E] */        \
    X(GLIDE, attack_hit1, s32, 14, "attack_hit1")            /* [E] */        \
    X(GLIDE, attack_endlag, s32, 14, "attack_endlag")        /* [E] */        \
    X(GLIDE, attack_buf_frames, s32, 6, "attack_buf_frames") /* [E] */        \
    X(GLIDE, entry_vy_max, f32, 0.6f, "entry_vy_max")        /* [E] */        \
    X(GLIDE, apex_vy, f32, 0.25f, "apex_vy")                 /* [E] */        \
    X(GLIDE, flick_thr, f32, 0.55f, "flick_thr")             /* [E] */        \
    X(GLIDE, s_start, f32, 1.50f, "s_start")                 /* [E] rev 1 */  \
    X(GLIDE, start_accel, f32, 0.12f, "start_accel")         /* [E] */        \
    X(GLIDE, s_start_cap, f32, 2.0f, "s_start_cap")          /* [E] */        \
    X(GLIDE, theta_def, f32, -5.0f, "theta_def")             /* [E] */        \
    X(GLIDE, r_down, f32, 60.0f, "r_down")                   /* [E] */        \
    X(GLIDE, r_up, f32, 85.0f, "r_up")                       /* [E] */        \
    X(GLIDE, turn_rate, f32, 3.0f, "turn_rate")              /* [E] */        \
    X(GLIDE, gk, f32, 0.0366f, "gk")                         /* designed */   \
    X(GLIDE, drag, f32, 0.002f, "drag")                      /* designed */   \
    X(GLIDE, s_max, f32, 2.40f, "s_max")                     /* [E] */        \
    X(GLIDE, s_stall, f32, 0.35f, "s_stall")                 /* [E] */        \
    X(GLIDE, trip_angle, f32, 30.0f, "trip_angle")           /* [E] */        \
    X(GLIDE, attack_speed_keep, f32, 0.6f, "attack_speed_keep") /* [E] */     \
    X(GLIDE, chain_speed_mul, f32, 0.75f, "chain_speed_mul") /* [E] */        \
    X(GLIDE, chain_turn_mul, f32, 1.4f, "chain_turn_mul")    /* [E] */        \
    X(GLIDE, attack_damage, s32, 12, "attack_damage")        /* [W:Glide] */

/* tornado: spec 3.1-3.3 */
#define GENO_BP_TORNADO_LIST(X)                                                 \
    X(TORNADO, start_frames, s32, 12, "start_frames")        /* [W] loop at 12 */ \
    X(TORNADO, read_period, s32, 10, "read_period")          /* [W] */        \
    X(TORNADO, read_limit, s32, 70, "read_limit")            /* [W] */        \
    X(TORNADO, end_frames, s32, 29, "end_frames")            /* [W] */        \
    X(TORNADO, ledge_lock, s32, 6, "ledge_lock")             /* [E] */        \
    X(TORNADO, final_hit_ground, s32, 3, "final_hit_ground") /* [W] */        \
    X(TORNADO, final_hit_air, s32, 2, "final_hit_air")       /* [W] */        \
    X(TORNADO, loop_damage, s32, 1, "loop_damage")           /* [W] */        \
    X(TORNADO, final_damage, s32, 3, "final_damage")         /* [W] */        \
    X(TORNADO, spin0, f32, 80.0f, "spin0")                   /* [W]/[D] */    \
    X(TORNADO, decay_a, f32, 1.5f, "decay_a")                /* [D] */        \
    X(TORNADO, decay_b, f32, 2.0f, "decay_b")                /* [D] */        \
    X(TORNADO, mash_add, f32, 16.0f, "mash_add")             /* spec 3.2 */   \
    X(TORNADO, spin_max, f32, 80.0f, "spin_max")             /* [W] */        \
    X(TORNADO, spin_end, f32, 10.0f, "spin_end")             /* [D] */        \
    X(TORNADO, grav, f32, 0.0239f, "grav_tn")                /* 0.25*0.0956 */ \
    X(TORNADO, rise_impulse, f32, 0.9f, "rise_impulse")      /* [E] */        \
    X(TORNADO, rise_max, f32, 1.6f, "rise_max")              /* [E] */        \
    X(TORNADO, fall_max, f32, 0.50f, "fall_tn")              /* spec 3.3 */   \
    X(TORNADO, vx_max, f32, 1.1f, "vx_max")                  /* [E] */        \
    X(TORNADO, vx_max_ground, f32, 1.3f, "vx_max_ground")    /* [E] */        \
    X(TORNADO, vx_accel, f32, 0.06f, "vx_accel")             /* [E] */        \
    X(TORNADO, jump_bonus, f32, 1.0f, "jump_bonus")          /* [E] */        \
    X(TORNADO, start_vy_keep, f32, 0.2f, "start_vy_keep")    /* [E] */        \
    X(TORNADO, start_vx_keep, f32, 0.5f, "start_vx_keep")    /* [E] */

/* drill: spec 4.1-4.2 */
#define GENO_BP_DRILL_LIST(X)                                                   \
    X(DRILL, move0, s32, 16, "move0")                        /* [E] */        \
    X(DRILL, ramp, s32, 4, "ramp")                           /* spec 4.1 */   \
    X(DRILL, hit0, s32, 26, "hit0")                          /* [W] */        \
    X(DRILL, hit_period, s32, 4, "hit_period")               /* [D] 40/10 */  \
    X(DRILL, nhits, s32, 10, "nhits")                        /* [W] */        \
    X(DRILL, hit_end, s32, 65, "hit_end")                    /* [W] */        \
    X(DRILL, final_frame, s32, 66, "final_frame")            /* [W] */        \
    X(DRILL, final_end_ground, s32, 67, "final_end_ground")  /* [W] */        \
    X(DRILL, final_end_air, s32, 70, "final_end_air")        /* [W] */        \
    X(DRILL, total, s32, 96, "total")                        /* [W] */        \
    X(DRILL, rush_damage, s32, 1, "rush_damage")             /* [W] */        \
    X(DRILL, final_damage, s32, 3, "final_damage")           /* [W] */        \
    X(DRILL, dive_default, f32, -20.0f, "dive_default")      /* [E] */        \
    X(DRILL, psi_stick, f32, 45.0f, "psi_stick")             /* spec 4.2 */   \
    X(DRILL, psi_min, f32, -60.0f, "psi_min")                /* [E] */        \
    X(DRILL, psi_max, f32, 30.0f, "psi_max")                 /* [E] */        \
    X(DRILL, steer_rate, f32, 2.0f, "steer_rate")            /* [E] */        \
    X(DRILL, speed, f32, 2.0f, "dr_speed")                   /* [E] */        \
    X(DRILL, recoil_vy, f32, 1.0f, "recoil_vy")              /* [E] */        \
    X(DRILL, end_vx_keep, f32, 0.2f, "end_vx_keep")          /* spec 4.2 */

/* cape: spec 5.1-5.2 (all [E]) */
#define GENO_BP_CAPE_LIST(X)                                                    \
    X(CAPE, start, s32, 6, "cape_start")                                      \
    X(CAPE, end, s32, 25, "cape_end")                                         \
    X(CAPE, total, s32, 40, "cape_total")                                     \
    X(CAPE, turn_thr, f32, 0.5f, "turn_thr")                                  \
    X(CAPE, stall_x, f32, 0.80f, "stall_x")                                   \
    X(CAPE, stall_y_up, f32, 0.85f, "stall_y_up")                             \
    X(CAPE, stall_vy_hi, f32, 0.0f, "stall_vy_hi")                            \
    X(CAPE, stall_grav, f32, 0.015f, "stall_grav")                            \
    X(CAPE, stall_fall, f32, 0.45f, "stall_fall")                             \
    X(CAPE, ground_x, f32, 0.5f, "ground_x")                                  \
    X(CAPE, reflect_dmg_mul, f32, 1.5f, "reflect_dmg_mul")                    \
    X(CAPE, reflect_speed_mul, f32, 1.2f, "reflect_speed_mul")

/* glide host switches (spec section 10 addendum): not GbGlideP members, read by the adapters.
 * auto_entry: entry paths A and B (spec 2.2) run from the preanim hook when non-zero (default on).
 * pose_center: the glide clip's level-pose frame (spec 1.2: the host owns animation); 0 = start at
 * frame 0, a character with a pitch-posed clip sets it. */
#define GENO_BP_GLIDE_HOST_LIST(X)                                              \
    X(GLIDE, auto_entry, f32, 1.0f, "auto_entry")                             \
    X(GLIDE, pose_center, f32, 0.0f, "pose_center")

#define GENO_BP_ALL_LIST(X)                                                     \
    GENO_BP_GLIDE_LIST(X) GENO_BP_GLIDE_HOST_LIST(X) GENO_BP_TORNADO_LIST(X)  \
    GENO_BP_DRILL_LIST(X) GENO_BP_CAPE_LIST(X)

#define GENO_BP_ENUM_(F, n, t, d, s) GENO_BP_##F##_##n,
enum { GENO_BP_ALL_LIST(GENO_BP_ENUM_) GENO_BP_COUNT };
#undef GENO_BP_ENUM_

typedef char geno_bp_fits_[(GENO_BP_COUNT <= GENO_PARAMS) ? 1 : -1];

#endif
