/*
 * script_lab.h - field numbers shared by the two halves of the Geno Lab inspection API:
 * pc/gameworld/script_game.c (game side, gwtool) and pc/platform/gw_script.c (native side, which
 * owns the names Lua sees). Plain enums only, so both compilers can include it. docs/geno.md
 * section "Geno Lab" is the reference.
 *
 * Everything here is READ-ONLY except the two cosmetic debug-draw switches (LabDebugDraw /
 * LabStageDraw), which gw_script.c refuses during a netplay/rollback session.
 */
#ifndef SCRIPT_LAB_H
#define SCRIPT_LAB_H

/* ScriptGame_LabF(slot, field) */
enum {
    LAB_F_ANIM_RATE = 0, /* fp->frame_speed_mul */
    LAB_F_HITSTUN,       /* frames of hitstun left (0 when not in hitstun) */
    LAB_F_KB_VX,         /* knockback velocity (fp->x8c_kb_vel) */
    LAB_F_KB_VY,
    LAB_F_SHIELD,        /* shield health */
    LAB_F_ECB_TOP_X,     /* ECB points, world coordinates (coll_data.cur_pos + ecb.*) */
    LAB_F_ECB_TOP_Y,
    LAB_F_ECB_BOTTOM_X,
    LAB_F_ECB_BOTTOM_Y,
    LAB_F_ECB_LEFT_X,
    LAB_F_ECB_LEFT_Y,
    LAB_F_ECB_RIGHT_X,
    LAB_F_ECB_RIGHT_Y,
    LAB_F_GR_VEL,        /* ground velocity */
    LAB_F_KB_APPLIED,    /* knockback of the last hit taken (dmg.kb_applied) */
    LAB_F_Z,             /* cur_pos.z */
    LAB_F_SCALE,         /* model scale (x34_scale.y) */
    LAB_F_CMD_TIMER,     /* subaction script timer */
    LAB_F_COUNT
};

/* ScriptGame_LabI(slot, field) */
enum {
    LAB_I_ANIM_ID = 0,    /* the subaction/animation index, -1 none */
    LAB_I_NAME_KIND,      /* the kind whose motion-name table applies (Kirby clones -> Kirby) */
    LAB_I_INTANG_TIMER,   /* fp->x1990: frames of timed intangibility left */
    LAB_I_INVINC_TIMER,   /* fp->x1994: frames of timed invincibility left */
    LAB_I_BODY_STATE,     /* fp->x1988: script body state 0 normal, 1 invincible, 2 intangible */
    LAB_I_TIMED_STATE,    /* fp->x198C: the timers' state, same values */
    LAB_I_JUMPS_USED,
    LAB_I_MAX_JUMPS,
    LAB_I_WALLJUMPS_USED,
    LAB_I_IN_HITLAG,      /* fp->x2219_b5 */
    LAB_I_IN_HITSTUN,     /* fp->x221C_b6 */
    LAB_I_IASA,           /* fp->allow_interrupt (the script's "interruptible" flag) */
    LAB_I_LEDGE_COOLDOWN, /* fp->x2064_ledgeCooldown */
    LAB_I_ECB_LOCK,
    LAB_I_DRAW_FLAGS,     /* fp->x21FC_flag.byte */
    LAB_I_SUB,            /* is_sub_fighter */
    LAB_I_JOINTS,         /* number of joints (fp->parts) */
    LAB_I_HURTBOXES,      /* hurt_capsules_len */
    LAB_I_HITSTUN_TOTAL,  /* unused, reserved */
    LAB_I_COUNT
};

/* ScriptGame_HitI / HitF (slot, index 0-3 = the fighter's hitboxes, 4 = the thrown hitbox) */
enum {
    LAB_HI_STATE = 0, /* 0 disabled, else active (engine HitCapsuleState) */
    LAB_HI_GROUP,     /* the hitbox "id" group the script gave it */
    LAB_HI_BONE,      /* fighter joint index, -1 unknown */
    LAB_HI_ANGLE,
    LAB_HI_KBG,
    LAB_HI_WBK,       /* weight-based (set) knockback */
    LAB_HI_BKB,
    LAB_HI_ELEMENT,
    LAB_HI_SHIELD_DMG,
    LAB_HI_SFX_SEVERITY,
    LAB_HI_SFX_KIND,
    LAB_HI_HIT_AIR,
    LAB_HI_HIT_GROUND,
    LAB_HI_CLANK,
    LAB_HI_REBOUND,
    LAB_HI_COUNT
};
enum {
    LAB_HF_DAMAGE = 0,
    LAB_HF_SIZE, /* radius */
    LAB_HF_X,    /* world position now */
    LAB_HF_Y,
    LAB_HF_Z,
    LAB_HF_PX,   /* world position last frame (the swept capsule's other end) */
    LAB_HF_PY,
    LAB_HF_PZ,
    LAB_HF_OX,   /* offset from the bone */
    LAB_HF_OY,
    LAB_HF_OZ,
    LAB_HF_COUNT
};

/* ScriptGame_HurtI / HurtF (slot, index < LAB_I_HURTBOXES) */
enum {
    LAB_UI_STATE = 0, /* HurtCapsuleState: 0 normal, 1 invincible, 2 intangible */
    LAB_UI_BONE,
    LAB_UI_HEIGHT,    /* 0 low, 1 mid, 2 high */
    LAB_UI_GRABBABLE,
    LAB_UI_COUNT
};
enum {
    LAB_UF_AX = 0, /* capsule ends, world */
    LAB_UF_AY,
    LAB_UF_AZ,
    LAB_UF_BX,
    LAB_UF_BY,
    LAB_UF_BZ,
    LAB_UF_SIZE,
    LAB_UF_COUNT
};

/* ScriptGame_CameraF(field): the match camera, read-only */
enum {
    LAB_CAM_OK = 0,   /* 1 when there is a game camera */
    LAB_CAM_VIEW,     /* 12 entries: the viewing matrix, row-major 3x4 */
    LAB_CAM_PROJ = LAB_CAM_VIEW + 12, /* projection type: 0 perspective, 1 frustum, 2 ortho */
    LAB_CAM_P0,       /* perspective: fov, aspect; ortho/frustum: top, bottom, left, right */
    LAB_CAM_P1,
    LAB_CAM_P2,
    LAB_CAM_P3,
    LAB_CAM_NEAR,
    LAB_CAM_FAR,
    LAB_CAM_VP_XMIN,
    LAB_CAM_VP_XMAX,
    LAB_CAM_VP_YMIN,
    LAB_CAM_VP_YMAX,
    LAB_CAM_COUNT
};

/* LabStageDraw bits (the match camera's own collision display, cm/camera.c) */
enum {
    LAB_STAGE_COLL = 1,     /* stage lines + every fighter's ECB + ledge-snap boxes */
    LAB_STAGE_TERRAIN = 2,  /* colour the lines by terrain kind */
    LAB_STAGE_LEDGES = 4,   /* colour the lines by ledge / platform */
    LAB_STAGE_POINTS = 8,   /* spawn, respawn, item and other special points */
    LAB_STAGE_ZONES = 16    /* camera / blast zones */
};

/* Script_GameEvent(what, a, b, c, d): engine sites -> the native event queue */
enum {
    LAB_EV_ACTION = 1, /* a player, b old motion, c new motion, d sub */
    LAB_EV_HIT = 2,    /* a attacker player (-1 none), b victim player, c flags, d damage (f32 bits) */
    LAB_EV_HITLAG = 3, /* a player, b 1 entering / 0 leaving, c sub */
    LAB_EV_LAND = 4    /* a player, b motion, c sub */
};
/* LAB_EV_HIT flags in c */
#define LAB_HIT_INDEX_MASK 0xFF   /* hitbox index 0-3, 0xFF unknown */
#define LAB_HIT_ATTACKER_SUB 0x100
#define LAB_HIT_VICTIM_SUB 0x200
#define LAB_HIT_BY_ITEM 0x400

#endif /* SCRIPT_LAB_H */
