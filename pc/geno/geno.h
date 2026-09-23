/* geno.h - Geno, GD's Melee's own fighter-extension layer: constants shared by the native half
 * (pc/platform/geno_registry.c, x86, little-endian) and the game half (pc/geno/geno_game.c,
 * compiled through the gwtool pipeline like the rest of melee, so it sees guest structs in their
 * own byte order). Design: docs/geno.md.
 *
 * Only #defines and enums here: nothing whose layout could differ between the two halves. The
 * halves talk through functions that take and return scalars (ints; floats travel as raw bits).
 *
 * Layering: retail engine -> m-ex compatibility layer (gw_mex_*, unchanged) -> Geno. Geno is
 * OPT-IN per fighter: a fighter no geno.json names runs exactly the code it ran before, except for
 * the escape opcode below, which no shipped script contains (pc/geno/tools/scan_ftcmd_opcodes.py).
 */
#ifndef GENO_H
#define GENO_H

#define GENO_VERSION 1 /* geno.json "geno" field and the script encoding (v1 subs are additive) */
#define GENO_LEVEL 1   /* feature level: 0 = v0 foundation, 1 = v1 (docs/geno.md section 15) */

/* ---- script escape (ftcmd) ---------------------------------------------------------------------
 * A subaction command's opcode is the top 6 bits of its first word. Retail uses 0-58 (lbCommand
 * 0-9, ftAction_803C06E8 10-58); 59-63 index past the end of ftAction's tables. Geno claims 59.
 *
 *   word0  [31:26] 59   [25:20] sub   [19:16] len (words incl. word0, 1-15; 0 reads as 1)
 *          [15:0] sub-specific; for the variable subs: [15:8] var A, [7] B is a var,
 *                 [6:4] compare (IF only), [3:0] 0
 *   word1  B: an immediate (int, or float bits when A is a float var) or a var ref in [7:0]
 *   word2  IF: words to skip (forward, from the end of this command) when the test FAILS
 *
 * A var ref is 8 bits: [7:6] bank, [5:0] index. Banks: LA = long-term (kept across actions, reset
 * at spawn/respawn), RA = per-action (reset whenever the fighter changes action state), each in an
 * int and a float flavour - the split Brawl's PSA uses (LA/RA x basic/float; PSA bits map onto
 * SETBIT/CLRBIT/IF-bit on int vars).
 *
 * Unknown subs are skipped by their len (logged once), so a script written for a newer Geno still
 * walks. Skips only go forward: a script can never loop inside one frame through Geno. */
#define GENO_FTCMD_OP 59

#define GENO_BANK_LA_INT 0
#define GENO_BANK_RA_INT 1
#define GENO_BANK_LA_FLOAT 2
#define GENO_BANK_RA_FLOAT 3
#define GENO_VARS_PER_BANK 64
#define GENO_VAR(bank, idx) ((((bank) & 3) << 6) | ((idx) & 63))

enum {
    GENO_SUB_NOP = 0x00,
    GENO_SUB_SET = 0x01,    /* A = B */
    GENO_SUB_ADD = 0x02,    /* A += B */
    GENO_SUB_SUB = 0x03,    /* A -= B */
    GENO_SUB_MUL = 0x04,    /* A *= B */
    GENO_SUB_SETBIT = 0x05, /* A |= 1 << B   (int vars) */
    GENO_SUB_CLRBIT = 0x06, /* A &= ~(1 << B) */
    GENO_SUB_DIV = 0x07,    /* v1: A /= B (B == 0: unchanged) */
    GENO_SUB_GET = 0x08,    /* v1: A = engine value word1 (GENO_VAL_*) */
    GENO_SUB_PUT = 0x09,    /* v1: engine value word1 = B (word2; [7] B is a var) */
    GENO_SUB_RAND = 0x0A,   /* v1: A = game-RNG random in [0, B) */
    GENO_SUB_IF = 0x10,     /* if !(A cmp B) skip word2 words */
    GENO_SUB_SKIP = 0x11,   /* skip word1 words (the jump over an else branch) */
    GENO_SUB_IFV = 0x12,    /* v1: if !(value word1 cmp B word2) skip word3 words */
    GENO_SUB_ORIG = 0x13,   /* v1: continue with the overlaid subaction's original script */
    GENO_SUB_CALL = 0x20,   /* call native hook word1 with argument word2 */
    GENO_SUB_CHG = 0x30,    /* v1: register a change-action check (word1 target, word2/3 args) */
    GENO_SUB_CHGAND = 0x31, /* v1: AND another condition onto the last CHG (word1/2 args) */
    GENO_SUB_CHGCLR = 0x32, /* v1: drop every change-action check of this action */
    GENO_SUB_REHIT = 0x38,  /* v1: [15:8] hitbox mask; word1 = rehit every N frames (0 = off) */
    GENO_SUB_LINK = 0x39,   /* v1: [15:8] hitbox mask; word1 = autolink mode (GENO_LINK_*) */
};

/* ---- v1: engine values (GET / PUT / IFV / the VALUE condition). Stable numbers. ----------- */
enum {
    GENO_VAL_AIR = 0x00,          /* i W: 1 airborne; writing 1 on the ground = become airborne */
    GENO_VAL_FACING = 0x01,       /* f W: +1 / -1; writing 0 turns around */
    GENO_VAL_VEL_X = 0x02,        /* f W: self_vel.x */
    GENO_VAL_VEL_Y = 0x03,        /* f W: self_vel.y */
    GENO_VAL_GROUND_VEL = 0x04,   /* f W: gr_vel */
    GENO_VAL_FWD_VEL = 0x05,      /* f W: self_vel.x * facing */
    GENO_VAL_KB_VEL_X = 0x06,     /* f */
    GENO_VAL_KB_VEL_Y = 0x07,     /* f */
    GENO_VAL_STICK_X = 0x08,      /* f */
    GENO_VAL_STICK_Y = 0x09,      /* f */
    GENO_VAL_STICK_FWD = 0x0A,    /* f: stick x * facing */
    GENO_VAL_CSTICK_X = 0x0B,     /* f */
    GENO_VAL_CSTICK_Y = 0x0C,     /* f */
    GENO_VAL_ANIM_FRAME = 0x0D,   /* f */
    GENO_VAL_ACTION_FRAME = 0x0E, /* i: frames in this action (1 on its first frame) */
    GENO_VAL_MOTION = 0x0F,       /* i: motion (action state) id */
    GENO_VAL_PERCENT = 0x10,      /* f */
    GENO_VAL_JUMPS_USED = 0x11,   /* i W */
    GENO_VAL_JUMPS_MAX = 0x12,    /* i */
    GENO_VAL_BUTTONS_HELD = 0x13, /* i: GENO_BTN_* mask */
    GENO_VAL_BUTTONS_PRESSED = 0x14, /* i: GENO_BTN_* mask, this frame */
    GENO_VAL_POS_X = 0x15,        /* f */
    GENO_VAL_POS_Y = 0x16,        /* f */
    GENO_VAL_CMD_VAR0 = 0x17,     /* i W: fp->cmd_vars[0..3] = 0x17..0x1A */
    GENO_VAL_CMD_VAR3 = 0x1A,
    GENO_VAL_ANIM_RATE = 0x1B,    /* f */
    GENO_VAL_FAST_FALL = 0x1C,    /* i */
    GENO_VAL_TRIGGER = 0x1D,      /* f: analog shield trigger */
    GENO_VAL_COUNT = 0x1E,
    GENO_VAL_SPECIAL_F = 0x1000,  /* + word index: fp->dat_attrs word as float */
    GENO_VAL_SPECIAL_I = 0x2000,  /* + word index: fp->dat_attrs word as int */
};
#define GENO_SPECIAL_WORDS 265 /* 0x424 bytes: the per-fighter special-attribute buffer */

/* Geno button mask (GENO_VAL_BUTTONS_*, the PRESSED / HELD conditions); bit n = Brawl button n */
#define GENO_BTN_ATTACK 0x01
#define GENO_BTN_SPECIAL 0x02
#define GENO_BTN_JUMP 0x04
#define GENO_BTN_SHIELD 0x08
#define GENO_BTN_GRAB 0x10
#define GENO_BTN_TAUNT 0x20

/* ---- v1: change action. CHG word0 [15:8] condition, [7] B is a var, [6:4] cmp, [3] NOT,
 * [2] ONCE; word1 target; word2 arg1; word3 arg2. CHGAND: same flags, word1 arg1, word2 arg2. */
enum {
    GENO_COND_ALWAYS = 0,
    GENO_COND_ANIM_END = 1,
    GENO_COND_GROUND = 2,
    GENO_COND_AIR = 3,
    GENO_COND_PRESSED = 4, /* arg1 GENO_BTN_* mask */
    GENO_COND_HELD = 5,    /* arg1 GENO_BTN_* mask */
    GENO_COND_BIT = 6,     /* arg1 var ref, arg2 bit */
    GENO_COND_VAR = 7,     /* arg1 var ref A, cmp, arg2 B */
    GENO_COND_FRAME = 8,   /* animation frame >= arg1 */
    GENO_COND_VALUE = 9,   /* arg1 GENO_VAL_*, cmp, arg2 B */
    GENO_COND_COUNT = 10
};
#define GENO_CHG_NOT 0x08
#define GENO_CHG_ONCE 0x04
#define GENO_W0_CHG(sub, len, cond, b_is_var, cmp, flags) \
    GENO_W0(sub, len, (((cond) & 0xFF) << 8) | ((b_is_var) ? 0x80 : 0) | (((cmp) & 7) << 4) | \
                          ((flags) & 0x0C))

/* The target word: [31:28] kind, [27] RAW, [26] KEEP_FRAME, [15:0] id. */
#define GENO_TGT_MOTION 0u
#define GENO_TGT_SPECIAL 1u
#define GENO_TGT_GENO 2u /* a Geno state (v2): v1 ignores it */
#define GENO_TGT_RAW 0x08000000u
#define GENO_TGT_KEEP_FRAME 0x04000000u
#define GENO_TARGET(kind, id) ((((unsigned) (kind) & 15u) << 28) | ((unsigned) (id) & 0xFFFFu))

#define GENO_MAX_CHECKS 8     /* registered change-action checks per fighter */
#define GENO_CHECK_CONDS 3    /* conditions per check (CHG + 2 CHGAND) */
#define GENO_MAX_REHIT 4      /* one per Melee hitbox id */

enum {
    GENO_LINK_OFF = 0,
    GENO_LINK_DIRECTION = 1, /* launch along the attacker's momentum, Melee knockback */
    GENO_LINK_SPEED = 2,     /* ... and at least the attacker's speed */
};

enum {
    GENO_CMP_EQ = 0,
    GENO_CMP_NE = 1,
    GENO_CMP_LT = 2,
    GENO_CMP_LE = 3,
    GENO_CMP_GT = 4,
    GENO_CMP_GE = 5,
    GENO_CMP_BIT = 6,   /* A & (1 << B) */
    GENO_CMP_NOBIT = 7, /* !(A & (1 << B)) */
};

#define GENO_W0(sub, len, low16) \
    ((unsigned) (GENO_FTCMD_OP << 26) | (((unsigned) (sub) & 63u) << 20) | \
     (((unsigned) (len) & 15u) << 16) | ((unsigned) (low16) & 0xFFFFu))
#define GENO_W0_VAR(sub, len, a, b_is_var, cmp) \
    GENO_W0(sub, len, (((a) & 0xFF) << 8) | ((b_is_var) ? 0x80 : 0) | (((cmp) & 7) << 4))

/* How the escape is being run: the three ftAction loops. EXEC is the frame's normal pass, ANIM is
 * the first pass of a new animation (ftAction_80073354), SKIP is the fast-forward pass that runs
 * control flow only (ftAction_8007349C). Variables and control flow run in all three - they are
 * script logic, like lbCommand 0-9; hook calls run only in EXEC and ANIM. */
#define GENO_MODE_EXEC 0
#define GENO_MODE_ANIM 1
#define GENO_MODE_SKIP 2

/* ---- native hooks: stable numbers. Scripts and geno.json refer to these; never renumber. ---- */
enum {
    GENO_HOOK_NONE = 0,
    GENO_HOOK_LOG = 1,           /* "geno.log": log arg and the first LA vars (debugging) */
    GENO_HOOK_JUMPS_REFILL = 2,  /* "geno.jumps.refill": give back arg air jumps (0 = all) */
    GENO_HOOK_JUMPS_TO_VAR = 3,  /* "geno.jumps.to_var": LA int var[arg] = air jumps left */
    GENO_HOOK_COUNT_FRAMES = 4,  /* "geno.count_frames": LA int var[arg] += 1 */
    GENO_HOOK_BUILTIN_COUNT = 5,
    GENO_HOOK_MAX = 64
};

/* Dispatch points a geno.json can bind hooks to ("hooks": {"on_frame": [...]}). */
enum {
    GENO_EV_INIT = 0,   /* spawn and respawn, after the state block is reset */
    GENO_EV_FRAME = 1,  /* every frame, after the m-ex onFrame dispatch */
    GENO_EV_ACTION = 2, /* action state change, after the RA banks are cleared */
    GENO_EV_LAND = 3,   /* v1: the moment of landing (inside the collision callback) */
    GENO_EV_COUNT = 4
};
#define GENO_EV_MAX_HOOKS 8

/* Attribute overrides travel as (index, value bits); the index names a field of ftCo_DatAttrs
 * through the game half's table (GenoGame_AttrFind). */
#define GENO_MAX_ATTRS 48
#define GENO_MAX_JUMP_VY 16
#define GENO_MAX_PROFILES 32
#define GENO_MAX_SPECIAL 64      /* v1: special_attributes entries per profile */
#define GENO_MAX_ONLAND 16       /* v1: on_land map entries per profile */
#define GENO_MAX_OVERLAYS 64     /* v1: subaction script overlays per profile */
#define GENO_POOL_WORDS 16384    /* v1: all overlay words of every profile (64 KB) */

#endif
