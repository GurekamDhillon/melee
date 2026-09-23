/*
 * geno_game.c - Geno's game half: per-fighter state, the ftcmd escape interpreter, native hooks,
 * attribute overrides and the multi-jump mechanic. Design: docs/geno.md; constants: geno.h.
 *
 * Game-world code: compiled for ppc32 and run through gwtool like the rest of melee, so Fighter
 * and script words are read in the guest's byte order and nothing here swaps by hand. It asks the
 * native registry (pc/platform/geno_registry.c) about profiles through scalar-only calls.
 *
 * COMPATIBILITY. Every entry point returns at once for a fighter no geno.json names (profile -1)
 * and whose scripts never used the escape: the engine sites that call in here (fighter.c,
 * ftchangeparam.c, ftCo_JumpAerialF1.c, ftaction.c) then behave exactly as before.
 *
 * ROLLBACK. The only mutable state is Geno_StateBlock, a plain game global: gwtool names it
 * _gw_Geno_StateBlock and gw_snap.c saves/restores every game global, so a savestate or a rollback
 * covers it with no extra code. It is reset when a fighter (re)spawns and never holds a pointer.
 * The hook table is const. Everything else Geno knows comes from the registry, which is fixed
 * before the first fighter spawns.
 */

#include <Runtime/platform.h>

#include <melee/ft/fighter.h>
#include <melee/ft/inlines.h>
#include <melee/ft/types.h>
#include <melee/lb/types.h>

#include "geno.h"

/* the native registry (gwtool adds the gw_ prefix to these references) */
extern int Geno_ProfileForKind(int kind);
extern int Geno_MaxJumps(int p);
extern int Geno_JumpVyCount(int p);
extern int Geno_JumpVyBits(int p, int i);
extern int Geno_AttrCount(int p);
extern int Geno_AttrIndex(int p, int i);
extern int Geno_AttrBits(int p, int i);
extern int Geno_HookCount(int p, int ev);
extern int Geno_Hook(int p, int ev, int i);
extern int Geno_HookArg(int p, int ev, int i);
extern void Geno_Event(int what, int a, int b, int c, int d);

/* ---- the state block ------------------------------------------------------------------------ */

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
} GenoState;

#define GENO_SF_SCRIPT 1u /* a script used the escape since the reset */

#define GENO_PLAYERS 6
/* [player slot][sub-fighter]: one block per fighter object a player can own (Ice Climbers' Nana,
 * the inactive Zelda/Sheik form). Deliberately NOT static: an uninitialised global is a common
 * symbol, which gw_snap.c's map walk takes as game state by its _gw_ name. */
GenoState Geno_StateBlock[GENO_PLAYERS][2];

static GenoState* geno_state(Fighter* fp)
{
    int p = fp->player_id < GENO_PLAYERS ? fp->player_id : GENO_PLAYERS - 1;
    return &Geno_StateBlock[p][fp->is_sub_fighter ? 1 : 0];
}

static void geno_zero(void* p, int n)
{
    u8* b = p;
    while (n-- > 0) {
        *b++ = 0;
    }
}

/* ---- native hooks ---------------------------------------------------------------------------- */

typedef int (*GenoHookFn)(Fighter_GObj* gobj, Fighter* fp, GenoState* st, s32 arg);

static int hook_log(Fighter_GObj* gobj, Fighter* fp, GenoState* st, s32 arg)
{
    Geno_Event(2, fp->kind, fp->player_id, arg, st->la_i[0]);
    return 0;
}

static int hook_jumps_refill(Fighter_GObj* gobj, Fighter* fp, GenoState* st, s32 arg)
{
    /* jumpsUsed counts the ground jump too, and is 1 in the air after leaving the ground */
    int used = fp->x1968_jumpsUsed;
    used = arg <= 0 ? 1 : used - arg;
    fp->x1968_jumpsUsed = used < 1 ? 1 : used;
    return 0;
}

static int hook_jumps_to_var(Fighter_GObj* gobj, Fighter* fp, GenoState* st, s32 arg)
{
    int left = fp->co_attrs.max_jumps - fp->x1968_jumpsUsed;
    st->la_i[arg & (GENO_VARS_PER_BANK - 1)] = left < 0 ? 0 : left;
    return 0;
}

static int hook_count_frames(Fighter_GObj* gobj, Fighter* fp, GenoState* st, s32 arg)
{
    st->la_i[arg & (GENO_VARS_PER_BANK - 1)] += 1;
    return 0;
}

/* A Geno feature registers a hook by adding a row here with a new stable number from geno.h. The
 * table is const on purpose: registration is part of the build, so it can never differ between
 * two peers or between a snapshot and the live game. */
static const struct {
    int id;
    const char* name;
    GenoHookFn fn;
} geno_hooks[] = {
    { GENO_HOOK_LOG, "geno.log", hook_log },
    { GENO_HOOK_JUMPS_REFILL, "geno.jumps.refill", hook_jumps_refill },
    { GENO_HOOK_JUMPS_TO_VAR, "geno.jumps.to_var", hook_jumps_to_var },
    { GENO_HOOK_COUNT_FRAMES, "geno.count_frames", hook_count_frames },
};
#define GENO_NHOOKS ((int) (sizeof(geno_hooks) / sizeof(geno_hooks[0])))

static int geno_streq(const char* a, const char* b)
{
    while (*a != '\0' && *a == *b) {
        a++;
        b++;
    }
    return *a == *b;
}

/* For the registry: a hook name -> its number, -1 when unknown. */
int GenoGame_HookFind(const char* name)
{
    int i;
    for (i = 0; i < GENO_NHOOKS; i++) {
        if (geno_streq(name, geno_hooks[i].name)) {
            return geno_hooks[i].id;
        }
    }
    return -1;
}

/* Run hook `id`. Returns 0 when it ran, -1 when there is no such hook (logged). */
int GenoGame_CallHook(Fighter_GObj* gobj, int id, s32 arg)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    int i;
    for (i = 0; i < GENO_NHOOKS; i++) {
        if (geno_hooks[i].id == id) {
            st->hook_calls++;
            geno_hooks[i].fn(gobj, fp, st, arg);
            return 0;
        }
    }
    Geno_Event(4, fp->kind, fp->player_id, id, 0);
    return -1;
}

static void geno_run_event(Fighter_GObj* gobj, GenoState* st, int ev)
{
    int n = Geno_HookCount(st->profile, ev);
    int i;
    for (i = 0; i < n; i++) {
        GenoGame_CallHook(gobj, Geno_Hook(st->profile, ev, i),
                          Geno_HookArg(st->profile, ev, i));
    }
}

/* ---- dispatch points (called from the engine under TARGET_PC) ------------------------------- */

/* Fighter_UnkInitReset_80067C98: spawn, respawn, and the Zelda/Sheik swap. */
void Geno_FighterReset(Fighter* fp)
{
    GenoState* st = geno_state(fp);
    u32 resets = st->resets;
    geno_zero(st, sizeof(*st));
    st->resets = resets + 1;
    st->kind = fp->kind;
    st->profile = Geno_ProfileForKind(fp->kind);
    if (st->profile >= 0) {
        Geno_Event(0, fp->kind, fp->player_id, st->profile, 0);
        if (fp->gobj != NULL) {
            geno_run_event(fp->gobj, st, GENO_EV_INIT);
        }
    }
}

/* Fighter_ChangeMotionState, right after motion_id is set: the RA banks belong to one action. */
void Geno_OnActionChange(Fighter_GObj* gobj)
{
    GenoState* st = geno_state(GET_FIGHTER(gobj));
    if (st->profile < 0 && !(st->flags & GENO_SF_SCRIPT)) {
        return;
    }
    geno_zero(st->ra_i, sizeof(st->ra_i));
    geno_zero(st->ra_f, sizeof(st->ra_f));
    if (st->profile >= 0) {
        geno_run_event(gobj, st, GENO_EV_ACTION);
    }
}

/* Fighter_8006A360, after the m-ex onFrame dispatch. */
void Geno_OnFrame(Fighter_GObj* gobj)
{
    GenoState* st = geno_state(GET_FIGHTER(gobj));
    if (st->profile < 0) {
        return;
    }
    geno_run_event(gobj, st, GENO_EV_FRAME);
}

/* ---- attributes ----------------------------------------------------------------------------- */

#define GENO_ATTR(field, is_int) \
    { #field, (int) __builtin_offsetof(ftCo_DatAttrs, field), is_int }

static const struct {
    const char* name;
    int offset;
    int is_int;
} geno_attrs[] = {
    GENO_ATTR(walk_accel_mul, 0),
    GENO_ATTR(walk_accel_base, 0),
    GENO_ATTR(walk_max_vel, 0),
    GENO_ATTR(slow_walk_max, 0),
    GENO_ATTR(mid_walk_point, 0),
    GENO_ATTR(fast_walk_min, 0),
    GENO_ATTR(ground_friction, 0),
    GENO_ATTR(dash_initial_velocity, 0),
    GENO_ATTR(dash_accel_mul, 0),
    GENO_ATTR(dash_accel_base, 0),
    GENO_ATTR(dash_max_velocity, 0),
    GENO_ATTR(run_animation_scaling, 0),
    GENO_ATTR(max_run_brake_frames, 0),
    GENO_ATTR(ground_max_horizontal_velocity, 0),
    GENO_ATTR(jump_startup_time, 0),
    GENO_ATTR(jump_h_initial_velocity, 0),
    GENO_ATTR(jump_v_initial_velocity, 0),
    GENO_ATTR(ground_to_air_jump_momentum_multiplier, 0),
    GENO_ATTR(jump_h_max_velocity, 0),
    GENO_ATTR(hop_v_initial_velocity, 0),
    GENO_ATTR(air_jump_v_multiplier, 0),
    GENO_ATTR(air_jump_h_multiplier, 0),
    GENO_ATTR(max_jumps, 1),
    GENO_ATTR(gravity, 0),
    GENO_ATTR(terminal_velocity, 0),
    GENO_ATTR(air_drift_stick_mul, 0),
    GENO_ATTR(aerial_drift_base, 0),
    GENO_ATTR(air_drift_max, 0),
    GENO_ATTR(aerial_friction, 0),
    GENO_ATTR(fast_fall_velocity, 0),
    GENO_ATTR(air_max_horizontal_velocity, 0),
    GENO_ATTR(jab_2_input_window, 0),
    GENO_ATTR(jab_3_input_window, 0),
    GENO_ATTR(standing_turn_frames, 0),
    GENO_ATTR(weight, 0),
    GENO_ATTR(model_scaling, 0),
    GENO_ATTR(initial_shield_size, 0),
    GENO_ATTR(shield_break_initial_velocity, 0),
    GENO_ATTR(rapid_jab_window, 1),
    GENO_ATTR(clank_animation_length, 0),
};
#define GENO_NATTRS ((int) (sizeof(geno_attrs) / sizeof(geno_attrs[0])))

int GenoGame_AttrFind(const char* name)
{
    int i;
    for (i = 0; i < GENO_NATTRS; i++) {
        if (geno_streq(name, geno_attrs[i].name)) {
            return i;
        }
    }
    return -1;
}

int GenoGame_AttrIsInt(int index)
{
    return index >= 0 && index < GENO_NATTRS ? geno_attrs[index].is_int : 0;
}

int GenoGame_AttrOffset(int index)
{
    return index >= 0 && index < GENO_NATTRS ? geno_attrs[index].offset : -1;
}

/* ftchangeparam.c, right after co_attrs is copied from the fighter's data and BEFORE any scaling:
 * a Geno value replaces the file's value, then the engine's own modifiers apply as usual. */
void Geno_ApplyAttrs(Fighter* fp)
{
    int p = Geno_ProfileForKind(fp->kind);
    int n, i, mj;
    if (p < 0) {
        return;
    }
    n = Geno_AttrCount(p);
    for (i = 0; i < n; i++) {
        int idx = Geno_AttrIndex(p, i);
        if (idx >= 0 && idx < GENO_NATTRS) {
            *(u32*) ((u8*) &fp->co_attrs + geno_attrs[idx].offset) =
                (u32) Geno_AttrBits(p, i);
        }
    }
    mj = Geno_MaxJumps(p);
    if (mj > 0) {
        fp->co_attrs.max_jumps = mj;
    }
    Geno_Event(5, fp->kind, fp->player_id, n, fp->co_attrs.max_jumps);
}

/* ---- multi-jump past Melee's table -----------------------------------------------------------
 * Kirby and Jigglypuff jump through a per-kind table (fp->x2D0): state count x28 (5), first state
 * x2C, and x14[5] vertical impulses, indexed by jumpsUsed - 1. The engine never bounds that index,
 * so max_jumps above 6 would read past x14 and enter motion ids past the multi-jump states.
 * ftCo_800D74A4 calls this with the state and impulse it computed; for a Geno fighter, air jumps
 * past the table repeat the LAST multi-jump state (so ftCo_800D72A0 still recognises it), and
 * take their impulse from the profile's air_vy list (the last entry repeats), else the table's
 * last row. air_vy also overrides the table's own rows, which is how Brawl numbers ship. */
void Geno_MultiJump(Fighter* fp, int first_state, int* msid, float* vy)
{
    int p = Geno_ProfileForKind(fp->kind);
    int n = fp->x1968_jumpsUsed - 1; /* 0-based air jump */
    int rows, nvy;
    union {
        u32 u;
        f32 f;
    } bits;
    if (p < 0 || fp->x2D0 == NULL || n < 0) {
        return;
    }
    rows = fp->x2D0->x28;
    if (rows < 1 || rows > 5) {
        rows = 5;
    }
    nvy = Geno_JumpVyCount(p);
    if (n >= rows) {
        GenoState* st = geno_state(fp);
        *msid = first_state + rows - 1;
        *vy = fp->x2D0->x14[rows - 1];
        st->extra_jumps++;
        Geno_Event(1, fp->kind, fp->player_id, n + 1, fp->co_attrs.max_jumps - 1);
    } else {
        Geno_Event(6, fp->kind, fp->player_id, n + 1, fp->co_attrs.max_jumps - 1);
    }
    if (nvy > 0) {
        bits.u = (u32) Geno_JumpVyBits(p, n < nvy ? n : nvy - 1);
        *vy = bits.f;
    }
}

/* ---- the ftcmd escape ------------------------------------------------------------------------ */

typedef union {
    u32 u;
    s32 i;
    f32 f;
} GenoWord;

static int geno_var_is_float(int ref)
{
    return ((ref >> 6) & 3) >= GENO_BANK_LA_FLOAT;
}

static void* geno_var(GenoState* st, int ref)
{
    int i = ref & (GENO_VARS_PER_BANK - 1);
    switch ((ref >> 6) & 3) {
    case GENO_BANK_LA_INT:
        return &st->la_i[i];
    case GENO_BANK_RA_INT:
        return &st->ra_i[i];
    case GENO_BANK_LA_FLOAT:
        return &st->la_f[i];
    default:
        return &st->ra_f[i];
    }
}

/* Operand B in A's type. */
static GenoWord geno_operand_b(GenoState* st, int a, u32 w0, u32 w1)
{
    GenoWord r;
    int want_f = geno_var_is_float(a);
    if (w0 & 0x80) {
        int b = w1 & 0xFF;
        void* pb = geno_var(st, b);
        if (geno_var_is_float(b)) {
            if (want_f) {
                r.f = *(f32*) pb;
            } else {
                r.i = (s32) * (f32*) pb;
            }
        } else if (want_f) {
            r.f = (f32) * (s32*) pb;
        } else {
            r.i = *(s32*) pb;
        }
    } else {
        r.u = w1;
    }
    return r;
}

static int geno_compare(GenoState* st, int a, int cmp, GenoWord b)
{
    void* pa = geno_var(st, a);
    if (cmp == GENO_CMP_BIT || cmp == GENO_CMP_NOBIT) {
        s32 v = geno_var_is_float(a) ? (s32) * (f32*) pa : *(s32*) pa;
        int set = (v >> (b.i & 31)) & 1;
        return cmp == GENO_CMP_BIT ? set : !set;
    }
    if (geno_var_is_float(a)) {
        f32 x = *(f32*) pa, y = b.f;
        switch (cmp) {
        case GENO_CMP_EQ:
            return x == y;
        case GENO_CMP_NE:
            return x != y;
        case GENO_CMP_LT:
            return x < y;
        case GENO_CMP_LE:
            return x <= y;
        case GENO_CMP_GT:
            return x > y;
        default:
            return x >= y;
        }
    } else {
        s32 x = *(s32*) pa, y = b.i;
        switch (cmp) {
        case GENO_CMP_EQ:
            return x == y;
        case GENO_CMP_NE:
            return x != y;
        case GENO_CMP_LT:
            return x < y;
        case GENO_CMP_LE:
            return x <= y;
        case GENO_CMP_GT:
            return x > y;
        default:
            return x >= y;
        }
    }
}

static void geno_arith(GenoState* st, int sub, int a, GenoWord b)
{
    void* pa = geno_var(st, a);
    if (geno_var_is_float(a)) {
        f32* x = pa;
        switch (sub) {
        case GENO_SUB_SET:
            *x = b.f;
            break;
        case GENO_SUB_ADD:
            *x += b.f;
            break;
        case GENO_SUB_SUB:
            *x -= b.f;
            break;
        case GENO_SUB_MUL:
            *x *= b.f;
            break;
        }
    } else {
        s32* x = pa;
        switch (sub) {
        case GENO_SUB_SET:
            *x = b.i;
            break;
        case GENO_SUB_ADD:
            *x += b.i;
            break;
        case GENO_SUB_SUB:
            *x -= b.i;
            break;
        case GENO_SUB_MUL:
            *x *= b.i;
            break;
        case GENO_SUB_SETBIT:
            *x |= (s32) (1u << (b.i & 31));
            break;
        case GENO_SUB_CLRBIT:
            *x &= ~(s32) (1u << (b.i & 31));
            break;
        }
    }
}

/* One escape command at cmd->u; advances cmd->u past it (and past any skipped words). Called
 * from the three ftAction loops for opcode GENO_FTCMD_OP only. */
void Geno_FtCmd(Fighter_GObj* gobj, CommandInfo* cmd, int mode)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    u32* w = (u32*) cmd->u;
    u32 w0 = w[0];
    int sub = (w0 >> 20) & 63;
    int len = (w0 >> 16) & 15;
    int a = (w0 >> 8) & 0xFF;
    u32 skip = 0;
    if (len == 0) {
        len = 1;
    }
    st->flags |= GENO_SF_SCRIPT;
    switch (sub) {
    case GENO_SUB_NOP:
        break;
    case GENO_SUB_SET:
    case GENO_SUB_ADD:
    case GENO_SUB_SUB:
    case GENO_SUB_MUL:
    case GENO_SUB_SETBIT:
    case GENO_SUB_CLRBIT:
        geno_arith(st, sub, a, geno_operand_b(st, a, w0, len > 1 ? w[1] : 0));
        break;
    case GENO_SUB_IF:
        if (!geno_compare(st, a, (w0 >> 4) & 7,
                          geno_operand_b(st, a, w0, len > 1 ? w[1] : 0)))
        {
            skip = len > 2 ? w[2] : 0;
        }
        break;
    case GENO_SUB_SKIP:
        skip = len > 1 ? w[1] : 0;
        break;
    case GENO_SUB_CALL:
        if (mode != GENO_MODE_SKIP) {
            GenoGame_CallHook(gobj, len > 1 ? (int) w[1] : 0,
                              len > 2 ? (s32) w[2] : 0);
        }
        break;
    default:
        Geno_Event(3, fp->kind, fp->player_id, sub, 0);
        break;
    }
    if (skip > 0x10000) {
        skip = 0x10000; /* a corrupt count must not send the pointer across the heap */
    }
    cmd->u = (CmdUnion*) (w + len + skip);
}

/* For tests: the block of a fighter. */
void* GenoGame_StateOf(Fighter* fp)
{
    return geno_state(fp);
}
