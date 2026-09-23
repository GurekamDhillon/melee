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

#define GENO_VERSION 3    /* newest geno.json "geno" field this build reads (v2/v3 keys are additive) */
#define GENO_ID_VERSION 1 /* salt of the stable ids: NOT bumped by v2 (same entry -> same id) */
#define GENO_LEVEL 3      /* feature level: 0 v0, 1 v1 (section 15), 2 v2 (section 16), 3 v3 (section 17) */

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
    /* v2 */
    GENO_VAL_GENO_STATE = 0x1E,   /* i: the Geno state the fighter is in (index), -1 when none */
    GENO_VAL_MOVE_F0 = 0x20,      /* f W: 0x20..0x27 behaviour floats (glide angle/speed, ...) */
    GENO_VAL_MOVE_F7 = 0x27,
    GENO_VAL_MOVE_I0 = 0x28,      /* i W: 0x28..0x2F behaviour ints (timers, counters) */
    GENO_VAL_MOVE_I7 = 0x2F,
    /* v3 */
    GENO_VAL_LEDGE = 0x30,        /* i W: this action's ledge grab (PSA Allow/Disallow Ledgegrab):
                                     0 none, 1 front, 2 front and back; -1 = the state's default */
    GENO_VAL_HIDDEN = 0x31,       /* i W: 1 = the whole fighter is not drawn (model, shadow;
                                     Melee's FighterVis flag), kept across action changes */
    GENO_VAL_TRANSN_FWD = 0x32,   /* f: this frame's root motion (TransN), forward */
    GENO_VAL_TRANSN_UP = 0x33,    /* f: this frame's root motion (TransN), up */
    GENO_VAL_MOTION_GRAVITY = 0x34, /* f W: this action's root-motion gravity multiplier (PSA
                                     Disable / Enable Horizontal Gravity); -1 = the state's */
    GENO_VAL_COUNT = 0x35,
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
#define GENO_TGT_GENO 2u /* a Geno state (v2): id = index in the profile's "states" list */
/* v2, geno.json only ("auto" / "helpless"): Wait on the ground, else Fall / FallSpecial */
#define GENO_TGT_AUTO 0xFFFFFFFEu
#define GENO_TGT_HELPLESS 0xFFFFFFFDu
#define GENO_TGT_STAY 0xFFFFFFFCu /* v3 "stay" ("land": landing only grounds; the state goes on) */
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

/* ---- v2: Geno action states (docs/geno.md section 16) ------------------------------------------
 * A profile's "states" list declares new action states. State n is Melee motion id
 * GENO_MOTION_BASE + n: past every fighter's own special states (the largest special table,
 * Kirby's, ends below 0x220), so no vanilla or m-ex range check ever matches it. Its MotionState
 * row (animation = a subaction of the fighter's own files, flags, move id, and the anim / IASA /
 * phys / coll callbacks) is built by Geno and handed to Fighter_ChangeMotionState, so the state
 * lives in Melee's action-state machine like any other: damage, grabs, death, ledges and landing
 * take the fighter out of it the normal way. */
#define GENO_MOTION_BASE 0x400
#define GENO_MAX_STATES 48 /* v3: was 16 (rows: 48 x 32 profiles x 0x20 bytes, game state) */
#define GENO_MOVE_VARS 8 /* behaviour floats / ints kept across the states of one move */

/* Callback slots of a state row, and the callbacks a geno.json can name for each (stable ids). */
enum { GENO_CB_ANIM = 0, GENO_CB_IASA = 1, GENO_CB_PHYS = 2, GENO_CB_COLL = 3, GENO_CB_SLOTS = 4 };
#define GENO_CB_LIKE 0xFF /* "like": the like-motion's own callback for that slot */

/* Behaviours: a named set of the four callbacks plus an entry routine (stable ids). */
enum {
    GENO_BHV_NONE = 0,
    GENO_BHV_AIR = 1,           /* "geno.air": aerial state, gravity + drift, anim end -> next */
    GENO_BHV_GROUND = 2,        /* "geno.ground": grounded state, friction, anim end -> next */
    GENO_BHV_ANIM_MOTION = 3,   /* v3 "geno.anim_motion": moved by the clip's root motion (TransN),
                                   on the ground and in the air; lift-off, ledge and landing rules */
    GENO_BHV_GLIDE_START = 10,  /* "geno.glide.start" */
    GENO_BHV_GLIDE = 11,        /* "geno.glide" */
    GENO_BHV_GLIDE_ATTACK = 12, /* "geno.glide.attack" */
    GENO_BHV_GLIDE_LANDING = 13,/* "geno.glide.landing" */
    GENO_BHV_GLIDE_END = 14,    /* "geno.glide.end" */
    GENO_BHV_TORNADO = 20,      /* "geno.tornado": Mach Tornado (tap B to rise, drift, multi-hit) */
    GENO_BHV_DRILL = 30,        /* "geno.drill": Drill Rush (steered dash, bounce on hit / wall) */
    GENO_BHV_DRILL_END = 31,    /* "geno.drill.end": the flip after the rush */
    GENO_BHV_DRILL_START = 32,  /* "geno.drill.start": the wind-up before the rush (optional) */
    GENO_BHV_CAPE = 40,         /* v3 "geno.cape": Dimensional Cape start + vanish (stick-steered) */
    GENO_BHV_CAPE_ATTACK = 41,  /* v3 "geno.cape.attack": a reappear with the slash (root motion);
                                   the profile's 6 in order: N, N air, F, F air, B, B air */
    GENO_BHV_CAPE_END = 42,     /* v3 "geno.cape.end": the reappear without the slash; 2 in order:
                                   ground, air */
    GENO_BHV_MAX = 32
};

/* Behaviour parameters: per profile, by stable id; geno.json names them "<family>.<name>" inside
 * the family's block ("glide": {...}, "tornado": {...}, "drill": {...}). Every family also takes
 * its Brawl block word by word ("w00".."wNN"), which is what the IR / translator has. */
#define GENO_PARAMS 128
enum {
    GENO_P_GLIDE_W0 = 0x00,       /* glide.w00..w21: Brawl's Misc Glide block, 0x00..0x15 */
    GENO_P_GLIDE_HOLD = 0x18,     /* glide.hold_frames: jump held this long in an air jump */
    GENO_P_GLIDE_FROM_JUMP = 0x19,/* glide.from_ground_jump: 1 = the ground jump counts too */
    GENO_P_GLIDE_END_HELPLESS = 0x1A, /* glide.end_helpless: 1 = GlideEnd -> FallSpecial */
    GENO_P_GLIDE_LANDING_LAG = 0x1B,  /* glide.landing_lag: GlideAttack's landing lag (frames) */
    GENO_P_GLIDE_ENTRY = 0x1C,    /* glide.entry: 0 = no jump-hold entry (scripts only) */
    GENO_P_GLIDE_MAX_FRAMES = 0x1D, /* glide.max_frames: a time limit (0 = none, as Brawl) */
    GENO_P_GLIDE_POSE = 0x1E,     /* glide.pose_center: pose frame = this - angle (0 = off) */
    GENO_P_GLIDE_END_BUTTONS = 0x1F, /* glide.end_buttons: GENO_BTN_* mask that ends the glide */
    GENO_P_TORNADO_W0 = 0x20,     /* tornado.w00..w19: Brawl paramSpecialN, 0x20..0x33 */
    GENO_P_TORNADO_MAX_SPEED = 0x38, /* tornado.max_speed: hard horizontal cap (Brawl 2.5) */
    GENO_P_TORNADO_END_HELPLESS = 0x39, /* tornado.end_helpless: air end -> FallSpecial */
    GENO_P_TORNADO_SPIN_ANIM = 0x3A, /* v4 tornado.spin_anim: the spin clip plays at spin_anim x the
                                        spin rate (Brawl: Frame Speed Modifier = the rate, 1 clip
                                        frame = 1 degree); 0 = rate 1 (v3) */
    GENO_P_TORNADO_SPIN_PERIOD = 0x3B, /* v4 tornado.spin_period: the clip loops at this frame (MK 360:
                                          frame 360 = frame 0); 0 = at the clip's end */
    GENO_P_DRILL_W0 = 0x40,       /* drill.w00..w05: Brawl paramSpecialS, 0x40..0x45 */
    GENO_P_DRILL_SPEED = 0x48,    /* drill.speed: travel speed when the clip has no root motion */
    GENO_P_DRILL_ANGLE_MAX = 0x49,/* drill.angle_max: steering limit, degrees (0 = none, Brawl) */
    GENO_P_DRILL_BOUNCE = 0x4A,   /* drill.bounce: 1 wall | 2 hit | 4 shield -> DrillEnd at once */
    GENO_P_DRILL_POP_VX = 0x4B,   /* drill.pop_vx: DrillEnd's backward pop (Brawl 1.0) */
    GENO_P_DRILL_POP_VY = 0x4C,   /* drill.pop_vy: DrillEnd's upward pop (Brawl 2.1) */
    GENO_P_DRILL_END_HELPLESS = 0x4D, /* drill.end_helpless: 1 = FallSpecial unless it hit */
    GENO_P_DRILL_PITCH_MODEL = 0x4E, /* v4 drill.pitch_model: 1 = the model (TopN) turns with the
                                        rush pitch, hitboxes and hurtboxes with it (Brawl's posture
                                        rot.x); 0 = the model stays level (v3) */
    /* v3 */
    GENO_P_GLIDE_SCRIPT_HELPLESS = 0x50, /* glide.script_entry_helpless: a Glide entered straight
                                          from another action (not GlideStart; Brawl's up-B sets
                                          LA-Bit61) ends helpless: GlideEnd / GlideAttack -> FallSpecial */
    GENO_P_CAPE_W0 = 0x58,        /* cape.w00..w05: Brawl paramSpecialLw (ids 4021-4026), 0x58..0x5D */
    GENO_P_CAPE_STEER_FRAME = 0x60, /* cape.steer_frame: the vanish (stick steering) starts (12) */
    GENO_P_CAPE_DECIDE_FRAME = 0x61, /* cape.decide_frame: the reappear is chosen (26) */
    GENO_P_CAPE_NEUTRAL_X = 0x62, /* cape.neutral_x: |stick x| below it = the neutral reappear */
    GENO_P_CAPE_BUTTONS = 0x63,   /* cape.attack_buttons: GENO_BTN mask held = the slash (B | A) */
};

/* v3: per-state root-motion options ("geno.anim_motion"; geno.json state keys "ledge", "liftoff",
 * "origin", "gravity"). Packed as Geno_StateMotion(p, s). */
#define GENO_MOTION_LEDGE_MASK 3u   /* default ledge grab: 0 none, 1 front, 2 front and back */
#define GENO_MOTION_LIFTOFF 4u      /* on the ground, upward root motion takes off */
#define GENO_MOTION_ORIGIN 8u       /* the first frame also moves by the clip's frame-0 offset */
#define GENO_MOTION_ENTRY_FACING 16u /* "facing": "entry": the root motion keeps the facing the state
                                       was entered with (a mid-clip Reverse Direction turns the model
                                       and the hitboxes, not the travel) */

/* v2: specials bound to Geno states ("specials": {"n": "geno:5", "air_s": "geno:7", ...}) */
enum {
    GENO_SP_N = 0, GENO_SP_S = 1, GENO_SP_HI = 2, GENO_SP_LW = 3,
    GENO_SP_AIR_N = 4, GENO_SP_AIR_S = 5, GENO_SP_AIR_HI = 6, GENO_SP_AIR_LW = 7,
    GENO_SP_COUNT = 8
};

#endif
