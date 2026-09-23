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

#define GENO_VERSION 1 /* geno.json "geno" field and the script encoding */

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
    GENO_SUB_IF = 0x10,     /* if !(A cmp B) skip word2 words */
    GENO_SUB_SKIP = 0x11,   /* skip word1 words (the jump over an else branch) */
    GENO_SUB_CALL = 0x20,   /* call native hook word1 with argument word2 */
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
    GENO_EV_COUNT = 3
};
#define GENO_EV_MAX_HOOKS 8

/* Attribute overrides travel as (index, value bits); the index names a field of ftCo_DatAttrs
 * through the game half's table (GenoGame_AttrFind). */
#define GENO_MAX_ATTRS 48
#define GENO_MAX_JUMP_VY 16
#define GENO_MAX_PROFILES 32

#endif
