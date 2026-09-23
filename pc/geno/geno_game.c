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

#include <math.h>

#include <melee/ft/fighter.h>
#include <melee/ft/ft_0892.h>
#include <melee/ft/ftanim.h>
#include <melee/ft/ftcommon.h>
#include <melee/ft/ftparts.h>
#include <melee/ft/inlines.h>
#include <melee/ft/kinds/ftCommon/forward.h>
#include <melee/ft/kinds/ftCommon/ftCo_Fall.h>
#include <melee/ft/kinds/ftCommon/ftCo_FallSpecial.h>
#include <melee/ft/kinds/ftCommon/ftCo_Landing.h>
#include <melee/ft/types.h>
#include <melee/lb/lbcollision.h>
#include <melee/lb/types.h>
#include <sysdolphin/baselib/controller.h>
#include <sysdolphin/baselib/random.h>

#include "geno.h"
#include "geno_state.h"

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
/* v1 */
extern int Geno_SpecialCount(int p);
extern int Geno_SpecialIndex(int p, int i);
extern int Geno_SpecialBits(int p, int i);
extern int Geno_OnLandCount(int p);
extern int Geno_OnLandFrom(int p, int i);
extern int Geno_OnLandTo(int p, int i);
extern int Geno_OverlayCount(int p);
extern int Geno_OverlayAnim(int p, int i);
extern int Geno_OverlaySlot(int p, int i);
extern int Geno_SlotCount(void);
extern int Geno_SlotOffset(int s);
extern int Geno_SlotLen(int s);
extern int Geno_PoolWords(void);
extern int Geno_PoolWord(int i);
extern int Geno_PoolGen(void);
/* v2 */
extern int Geno_StateCount(int p);
extern int Geno_StateBehavior(int p, int s);
extern int Geno_StateCb(int p, int s, int slot);
extern int Geno_StateAnim(int p, int s);
extern int Geno_StateAnimFrom(int p, int s);
extern int Geno_StateLike(int p, int s);
extern int Geno_StateFlagsSet(int p, int s);
extern int Geno_StateFlags(int p, int s);
extern int Geno_StateMoveId(int p, int s);
extern int Geno_StateNext(int p, int s);
extern int Geno_StateLand(int p, int s);
extern int Geno_StateLagBits(int p, int s);
extern int Geno_StateMotion(int p, int s);
extern int Geno_StateGravityBits(int p, int s);
extern int Geno_ParamCount(int p);
extern int Geno_ParamId(int p, int i);
extern int Geno_ParamBits(int p, int i);
extern int Geno_Special(int p, int which);

/* ---- the state block ------------------------------------------------------------------------ */

/* GenoState (the per-fighter block) is in geno_state.h, shared with geno_tests.c. */

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

/* v2: Geno action states and behaviours (geno_game_v2.inc, included at the end of this file) */
static void geno_v2_build(Fighter* fp, int p);
static int geno_state_valid(GenoState* st, int s);
static int geno_enter_state(Fighter_GObj* gobj, Fighter* fp, GenoState* st, int s, u32 target);
static int geno_v2_preanim(Fighter_GObj* gobj, Fighter* fp, GenoState* st);
static int geno_cur_state(Fighter* fp, GenoState* st);

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

/* ---- v1: subaction script overlays -----------------------------------------------------------
 * geno.json "subactions" replace a subaction's script with words from the mod. The registry holds
 * every overlay's words in one pool; the game half keeps a copy in guest memory (so the ftAction
 * loops read it like any Pl file script, in the guest's byte order) and points the fighter's
 * subaction table rows at it. Both arrays are game globals (snapshotted) and, once filled, never
 * change during a match: a refill after a rollback writes the same words again. */
u32 Geno_ScriptPool[GENO_POOL_WORDS];
s32 Geno_ScriptPoolGen;
CmdUnion* Geno_OverlayOrig[256]; /* per overlay slot: the script the overlay replaced (ORIG) */

static void geno_pool_sync(void)
{
    int gen = Geno_PoolGen();
    int n, i;
    if (gen == Geno_ScriptPoolGen) {
        return;
    }
    n = Geno_PoolWords();
    if (n > GENO_POOL_WORDS) {
        n = GENO_POOL_WORDS;
    }
    for (i = 0; i < n; i++) {
        Geno_ScriptPool[i] = (u32) Geno_PoolWord(i);
    }
    Geno_ScriptPoolGen = gen;
}

static void geno_install_overlays(Fighter* fp, int p)
{
    int n = Geno_OverlayCount(p);
    int i;
    Fighter_WaitAnimData* table;
    if (n <= 0 || fp->ft_data == NULL || (table = fp->ft_data->xC) == NULL) {
        return;
    }
    geno_pool_sync();
    for (i = 0; i < n; i++) {
        int anim = Geno_OverlayAnim(p, i);
        int slot = Geno_OverlaySlot(p, i);
        int off = Geno_SlotOffset(slot);
        CmdUnion* mine;
        if (anim < 0 || slot < 0 || slot >= 256 || off < 0 || off >= GENO_POOL_WORDS) {
            continue;
        }
        mine = (CmdUnion*) &Geno_ScriptPool[off];
        /* idempotent: the table is the kind's (shared by every fighter of it, kept while the file
           stays loaded), so a second spawn finds it already pointing at the overlay */
        if (table[anim].xC != mine) {
            Geno_OverlayOrig[slot] = table[anim].xC;
            table[anim].xC = mine;
            Geno_Event(13, fp->kind, fp->player_id, anim, slot);
        }
    }
}

/* ORIG: the overlay slot whose words contain `w`, -1 when `w` is not in the pool. */
static int geno_slot_of(u32* w)
{
    int off, s, n;
    if (w < &Geno_ScriptPool[0] || w >= &Geno_ScriptPool[GENO_POOL_WORDS]) {
        return -1;
    }
    off = (int) (w - &Geno_ScriptPool[0]);
    n = Geno_SlotCount();
    for (s = 0; s < n && s < 256; s++) {
        int o = Geno_SlotOffset(s);
        if (off >= o && off < o + Geno_SlotLen(s)) {
            return s;
        }
    }
    return -1;
}

/* ---- dispatch points (called from the engine under TARGET_PC) ------------------------------- */

static int geno_inert(GenoState* st)
{
    return st->profile < 0 && !(st->flags & GENO_SF_SCRIPT);
}

/* The per-action part of the block (v1): checks, rehit timers, autolink flags, the frame count. */
static void geno_clear_action(GenoState* st)
{
    geno_zero(st->ra_i, sizeof(st->ra_i));
    geno_zero(st->ra_f, sizeof(st->ra_f));
    st->action_time = 0;
    st->nchecks = 0;
    st->last_check = -1;
    geno_zero(st->rehit_period, sizeof(st->rehit_period));
    geno_zero(st->rehit_count, sizeof(st->rehit_count));
    geno_zero(st->link_mode, sizeof(st->link_mode));
    st->ledge = -1;
    st->motion_started = 0;
    st->motion_vy = 0.0f;
    st->motion_land = 0;
}

/* Fighter_UnkInitReset_80067C98: spawn, respawn, and the Zelda/Sheik swap. */
void Geno_FighterReset(Fighter* fp)
{
    GenoState* st = geno_state(fp);
    u32 resets = st->resets;
    geno_zero(st, sizeof(*st));
    st->resets = resets + 1;
    st->kind = fp->kind;
    st->last_check = -1;
    st->profile = Geno_ProfileForKind(fp->kind);
    st->hold_motion = -1;
    st->hold_frames = -1;
    st->ledge = -1;
    st->enter_from = -1;
    if (st->profile >= 0) {
        Geno_Event(0, fp->kind, fp->player_id, st->profile, 0);
        geno_install_overlays(fp, st->profile);
        geno_v2_build(fp, st->profile);
        if (fp->gobj != NULL) {
            geno_run_event(fp->gobj, st, GENO_EV_INIT);
        }
    }
}

/* Fighter_ChangeMotionState, right after motion_id is set: the RA banks, change-action checks,
 * rehit timers and autolink flags belong to one action. */
void Geno_OnActionChange(Fighter_GObj* gobj)
{
    GenoState* st = geno_state(GET_FIGHTER(gobj));
    if (geno_inert(st)) {
        return;
    }
    geno_clear_action(st);
    if ((s32) GET_FIGHTER(gobj)->motion_id < GENO_MOTION_BASE) {
        st->hidden = 0; /* v3: HIDDEN lives only across Geno states (damage etc. show the fighter) */
    }
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

/* v1: special attributes (fp->dat_attrs, the fighter's own parameter block) by word index.
 * Written into the file's block (ft_data->ext_attr), which every fighter copies its dat_attrs
 * from - so the fighter's own attribute code (ftKindCalcIndiviParamTable / m-ex's
 * OnReapplyAttributes, which runs right after this) copies and scales the Geno values like the
 * file's - and into the fighter's own buffer too, for fighters that copy only once, at load.
 * Idempotent (the same words every time), so spawns, respawns and rollbacks agree. */
static void geno_apply_special(Fighter* fp, int p)
{
    int n = Geno_SpecialCount(p);
    int i;
    u32* src;
    u32* dst;
    if (n <= 0) {
        return;
    }
    src = fp->ft_data != NULL ? (u32*) fp->ft_data->ext_attr : NULL;
    /* the fighter's own buffer only when dat_attrs already points at it: fighters made for menus
       and the results screen (ftDemo_CreateFighter) re-apply attributes before dat_attrs is set,
       and it holds garbage then (a netplay results screen faulted on exactly that) */
    dst = (u32*) fp->dat_attrs;
    if (dst != (u32*) fp->dat_attrs_backup) {
        dst = NULL;
    }
    for (i = 0; i < n; i++) {
        int idx = Geno_SpecialIndex(p, i);
        u32 bits = (u32) Geno_SpecialBits(p, i);
        if (idx < 0 || idx >= GENO_SPECIAL_WORDS) {
            continue;
        }
        if (src != NULL) {
            src[idx] = bits;
        }
        if (dst != NULL && dst != src) {
            dst[idx] = bits;
        }
    }
    Geno_Event(12, fp->kind, fp->player_id, n, Geno_SpecialIndex(p, 0));
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
    geno_apply_special(fp, p);
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

/* A var read in the wanted type (float or int). */
static GenoWord geno_var_as(GenoState* st, int ref, int want_f)
{
    GenoWord r;
    void* pb = geno_var(st, ref);
    if (geno_var_is_float(ref)) {
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
    return r;
}

/* An operand in the wanted type: `word` is an immediate already in that type, or (b_is_var) a var
 * ref in [7:0], converted. */
static GenoWord geno_operand(GenoState* st, int want_f, int b_is_var, u32 word)
{
    GenoWord r;
    if (b_is_var) {
        return geno_var_as(st, word & 0xFF, want_f);
    }
    r.u = word;
    return r;
}

/* Operand B in A's type (the v0 variable subs: [7] of word0 says B is a var). */
static GenoWord geno_operand_b(GenoState* st, int a, u32 w0, u32 w1)
{
    return geno_operand(st, geno_var_is_float(a), (w0 & 0x80) != 0, w1);
}

/* x cmp y, both in the same type. BIT / NOBIT test bit y of x (as an int). */
static int geno_cmp(int is_f, GenoWord x, int cmp, GenoWord y)
{
    if (cmp == GENO_CMP_BIT || cmp == GENO_CMP_NOBIT) {
        s32 v = is_f ? (s32) x.f : x.i;
        s32 bit = is_f ? (s32) y.f : y.i;
        int set = (v >> (bit & 31)) & 1;
        return cmp == GENO_CMP_BIT ? set : !set;
    }
    if (is_f) {
        switch (cmp) {
        case GENO_CMP_EQ:
            return x.f == y.f;
        case GENO_CMP_NE:
            return x.f != y.f;
        case GENO_CMP_LT:
            return x.f < y.f;
        case GENO_CMP_LE:
            return x.f <= y.f;
        case GENO_CMP_GT:
            return x.f > y.f;
        default:
            return x.f >= y.f;
        }
    }
    switch (cmp) {
    case GENO_CMP_EQ:
        return x.i == y.i;
    case GENO_CMP_NE:
        return x.i != y.i;
    case GENO_CMP_LT:
        return x.i < y.i;
    case GENO_CMP_LE:
        return x.i <= y.i;
    case GENO_CMP_GT:
        return x.i > y.i;
    default:
        return x.i >= y.i;
    }
}

static int geno_compare(GenoState* st, int a, int cmp, GenoWord b)
{
    int is_f = geno_var_is_float(a);
    return geno_cmp(is_f, geno_var_as(st, a, is_f), cmp, b);
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
        case GENO_SUB_DIV:
            if (b.f != 0.0f) {
                *x /= b.f;
            }
            break;
        case GENO_SUB_RAND:
            *x = HSD_Randf() * b.f;
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
        case GENO_SUB_DIV:
            if (b.i != 0 && !(b.i == -1 && *x == (s32) 0x80000000)) {
                *x /= b.i;
            }
            break;
        case GENO_SUB_RAND:
            *x = b.i > 0 ? HSD_Randi(b.i) : 0;
            break;
        }
    }
}

/* ---- v1: engine values ------------------------------------------------------------------------ */

static u32 geno_buttons(Fighter* fp, int pressed)
{
    HSD_Pad b = pressed ? fp->input.pressed_buttons : fp->input.held_buttons[0];
    u32 m = 0;
    if (b & HSD_PAD_A) {
        m |= GENO_BTN_ATTACK;
    }
    if (b & HSD_PAD_B) {
        m |= GENO_BTN_SPECIAL;
    }
    if (b & HSD_PAD_XY) {
        m |= GENO_BTN_JUMP;
    }
    if (b & (HSD_PAD_L | HSD_PAD_R | HSD_PAD_LR)) {
        m |= GENO_BTN_SHIELD;
    }
    if (!pressed && p_ftCommonData != NULL &&
        fp->input.triggers[0] >= p_ftCommonData->shield_press_threshold)
    {
        m |= GENO_BTN_SHIELD;
    }
    if (b & HSD_PAD_Z) {
        m |= GENO_BTN_GRAB;
    }
    if (b & HSD_PAD_DPADUP) {
        m |= GENO_BTN_TAUNT;
    }
    return m;
}

static int geno_val_is_float(u32 id)
{
    if (id >= GENO_VAL_SPECIAL_I) {
        return 0;
    }
    if (id >= GENO_VAL_SPECIAL_F) {
        return 1;
    }
    switch (id) {
    case GENO_VAL_AIR:
    case GENO_VAL_ACTION_FRAME:
    case GENO_VAL_MOTION:
    case GENO_VAL_JUMPS_USED:
    case GENO_VAL_JUMPS_MAX:
    case GENO_VAL_BUTTONS_HELD:
    case GENO_VAL_BUTTONS_PRESSED:
    case GENO_VAL_CMD_VAR0:
    case GENO_VAL_CMD_VAR0 + 1:
    case GENO_VAL_CMD_VAR0 + 2:
    case GENO_VAL_CMD_VAR3:
    case GENO_VAL_FAST_FALL:
    case GENO_VAL_GENO_STATE:
    case GENO_VAL_LEDGE:
    case GENO_VAL_HIDDEN:
        return 0;
    default:
        if (id >= GENO_VAL_MOVE_I0 && id <= GENO_VAL_MOVE_I7) {
            return 0;
        }
        return 1;
    }
}

static GenoWord geno_val_get(Fighter* fp, GenoState* st, u32 id)
{
    GenoWord r;
    r.u = 0;
    if (id >= GENO_VAL_SPECIAL_F && id < GENO_VAL_SPECIAL_I + GENO_SPECIAL_WORDS) {
        u32 idx = id >= GENO_VAL_SPECIAL_I ? id - GENO_VAL_SPECIAL_I : id - GENO_VAL_SPECIAL_F;
        if (idx < GENO_SPECIAL_WORDS && fp->dat_attrs != NULL) {
            r.u = ((u32*) fp->dat_attrs)[idx];
        }
        return r;
    }
    switch (id) {
    case GENO_VAL_AIR:
        r.i = fp->ground_or_air == GA_Air;
        break;
    case GENO_VAL_FACING:
        r.f = fp->facing_dir;
        break;
    case GENO_VAL_VEL_X:
        r.f = fp->self_vel.x;
        break;
    case GENO_VAL_VEL_Y:
        r.f = fp->self_vel.y;
        break;
    case GENO_VAL_GROUND_VEL:
        r.f = fp->gr_vel;
        break;
    case GENO_VAL_FWD_VEL:
        r.f = fp->self_vel.x * fp->facing_dir;
        break;
    case GENO_VAL_KB_VEL_X:
        r.f = fp->x8c_kb_vel.x;
        break;
    case GENO_VAL_KB_VEL_Y:
        r.f = fp->x8c_kb_vel.y;
        break;
    case GENO_VAL_STICK_X:
        r.f = fp->input.lstick[0].x;
        break;
    case GENO_VAL_STICK_Y:
        r.f = fp->input.lstick[0].y;
        break;
    case GENO_VAL_STICK_FWD:
        r.f = fp->input.lstick[0].x * fp->facing_dir;
        break;
    case GENO_VAL_CSTICK_X:
        r.f = fp->input.cstick[0].x;
        break;
    case GENO_VAL_CSTICK_Y:
        r.f = fp->input.cstick[0].y;
        break;
    case GENO_VAL_ANIM_FRAME:
        r.f = fp->cur_anim_frame;
        break;
    case GENO_VAL_ACTION_FRAME:
        r.i = st->action_time;
        break;
    case GENO_VAL_MOTION:
        r.i = (s32) fp->motion_id;
        break;
    case GENO_VAL_PERCENT:
        r.f = fp->dmg.x1830_percent;
        break;
    case GENO_VAL_JUMPS_USED:
        r.i = fp->x1968_jumpsUsed;
        break;
    case GENO_VAL_JUMPS_MAX:
        r.i = fp->co_attrs.max_jumps;
        break;
    case GENO_VAL_BUTTONS_HELD:
        r.i = (s32) geno_buttons(fp, 0);
        break;
    case GENO_VAL_BUTTONS_PRESSED:
        r.i = (s32) geno_buttons(fp, 1);
        break;
    case GENO_VAL_POS_X:
        r.f = fp->cur_pos.x;
        break;
    case GENO_VAL_POS_Y:
        r.f = fp->cur_pos.y;
        break;
    case GENO_VAL_CMD_VAR0:
    case GENO_VAL_CMD_VAR0 + 1:
    case GENO_VAL_CMD_VAR0 + 2:
    case GENO_VAL_CMD_VAR3:
        r.i = (s32) fp->cmd_vars[id - GENO_VAL_CMD_VAR0];
        break;
    case GENO_VAL_ANIM_RATE:
        r.f = fp->frame_speed_mul;
        break;
    case GENO_VAL_FAST_FALL:
        r.i = fp->fall_fast ? 1 : 0;
        break;
    case GENO_VAL_TRIGGER:
        r.f = fp->input.triggers[0];
        break;
    case GENO_VAL_GENO_STATE:
        r.i = geno_cur_state(fp, st);
        break;
    case GENO_VAL_LEDGE:
        r.i = st->ledge;
        break;
    case GENO_VAL_HIDDEN:
        r.i = st->hidden;
        break;
    case GENO_VAL_TRANSN_FWD:
        r.f = fp->x594_b0 ? fp->x6A4_transNOffset.z : 0.0f;
        break;
    case GENO_VAL_TRANSN_UP:
        r.f = fp->x594_b0 ? fp->x6A4_transNOffset.y : 0.0f;
        break;
    default:
        if (id >= GENO_VAL_MOVE_F0 && id <= GENO_VAL_MOVE_F7) {
            r.f = st->move_f[id - GENO_VAL_MOVE_F0];
        } else if (id >= GENO_VAL_MOVE_I0 && id <= GENO_VAL_MOVE_I7) {
            r.i = st->move_i[id - GENO_VAL_MOVE_I0];
        }
        break;
    }
    return r;
}

/* `v` is in the value's own type. Returns 1 when written. */
static int geno_val_put(Fighter* fp, u32 id, GenoWord v)
{
    if (id >= GENO_VAL_MOVE_F0 && id <= GENO_VAL_MOVE_F7) {
        geno_state(fp)->move_f[id - GENO_VAL_MOVE_F0] = v.f;
        return 1;
    }
    if (id >= GENO_VAL_MOVE_I0 && id <= GENO_VAL_MOVE_I7) {
        geno_state(fp)->move_i[id - GENO_VAL_MOVE_I0] = v.i;
        return 1;
    }
    switch (id) {
    case GENO_VAL_LEDGE:
        geno_state(fp)->ledge = v.i < -1 ? -1 : v.i > 2 ? 2 : v.i;
        return 1;
    case GENO_VAL_HIDDEN:
        geno_state(fp)->hidden = v.i != 0;
        fp->x221E_b5 = v.i != 0;
        return 1;
    case GENO_VAL_AIR:
        if (v.i != 0 && fp->ground_or_air == GA_Ground) {
            ftCommon_8007D5D4(fp); /* Melee's own "become airborne" */
        }
        return 1;
    case GENO_VAL_FACING:
        if (v.f == 0.0f) {
            fp->facing_dir = -fp->facing_dir;
        } else {
            fp->facing_dir = v.f < 0.0f ? -1.0f : 1.0f;
        }
        if (fp->parts != NULL) {
            ftPartSetRotY(fp, 0, 1.5707964f * fp->facing_dir); /* as ChangeMotionState does */
        }
        return 1;
    case GENO_VAL_VEL_X:
        fp->self_vel.x = v.f;
        return 1;
    case GENO_VAL_VEL_Y:
        fp->self_vel.y = v.f;
        return 1;
    case GENO_VAL_GROUND_VEL:
        fp->gr_vel = v.f;
        return 1;
    case GENO_VAL_FWD_VEL:
        fp->self_vel.x = v.f * fp->facing_dir;
        return 1;
    case GENO_VAL_JUMPS_USED:
        fp->x1968_jumpsUsed = (u8) (v.i < 0 ? 0 : v.i > 255 ? 255 : v.i);
        return 1;
    case GENO_VAL_CMD_VAR0:
    case GENO_VAL_CMD_VAR0 + 1:
    case GENO_VAL_CMD_VAR0 + 2:
    case GENO_VAL_CMD_VAR3:
        fp->cmd_vars[id - GENO_VAL_CMD_VAR0] = (u32) v.i;
        return 1;
    default:
        Geno_Event(14, fp->kind, fp->player_id, (int) id, 0);
        return 0;
    }
}

/* ---- v1: change action ------------------------------------------------------------------------ */

/* Test support (geno_tests.c): with capture on, a change is recorded instead of performed (the test
 * fighter has no model to change), and ANIM_END reads geno_test_anim_end. Always 0 in the game. */
static s32 geno_test_capture;
static s32 geno_test_anim_end;
static u32 geno_test_target;
static s32 geno_test_changes;

void GenoGame_TestCapture(int on, int anim_end)
{
    geno_test_capture = on;
    geno_test_anim_end = anim_end;
    geno_test_changes = 0;
    geno_test_target = 0;
}

int GenoGame_TestChanges(void)
{
    return geno_test_changes;
}

u32 GenoGame_TestLastTarget(void)
{
    return geno_test_target;
}

/* The flags Melee's own moves use to swap to the other-situation version of a move mid-way (the
 * hitboxes, effects, sounds and sword trail carry over; the new script is fast-forwarded). */
#define GENO_KEEP_FRAME_FLAGS                                                                  \
    (Ft_MF_KeepGfx | Ft_MF_SkipHit | Ft_MF_SkipMatAnim | Ft_MF_KeepSfx | Ft_MF_SkipColAnim |  \
     Ft_MF_UpdateCmd | Ft_MF_SkipNametagVis | Ft_MF_KeepSwordTrail | Ft_MF_SkipItemVis |      \
     Ft_MF_SkipModelPartVis | Ft_MF_SkipAttackCount | Ft_MF_KeepFastFall)

/* The motion a target names, -1 for none (an undeclared Geno state, or a bad kind). */
static int geno_target_motion(Fighter* fp, u32 target)
{
    u32 kind = target >> 28;
    int id = (int) (target & 0xFFFF);
    if (kind == GENO_TGT_MOTION) {
        return id;
    }
    if (kind == GENO_TGT_SPECIAL) {
        return (int) fp->x18 + id;
    }
    if (kind == GENO_TGT_GENO && geno_state_valid(geno_state(fp), id)) {
        return GENO_MOTION_BASE + id;
    }
    return -1;
}

/* A target a check may fire: anything but an undeclared Geno state. */
static int geno_target_ok(Fighter* fp, u32 target)
{
    return (target >> 28) != GENO_TGT_GENO ||
           geno_state_valid(geno_state(fp), (int) (target & 0xFFFF));
}

/* Perform a change-action target. Returns 1 when the fighter changed action. */
static int geno_do_change(Fighter_GObj* gobj, Fighter* fp, GenoState* st, u32 target)
{
    int msid;
    if (target == GENO_TGT_AUTO || target == GENO_TGT_HELPLESS) {
        /* v2 geno.json shorthands: Wait on the ground, else Fall / FallSpecial */
        target = GENO_TARGET(GENO_TGT_MOTION, fp->ground_or_air == GA_Ground ? ftCo_MS_Wait
                                              : target == GENO_TGT_AUTO     ? ftCo_MS_Fall
                                                                            : ftCo_MS_FallSpecial);
    }
    if ((target >> 28) == GENO_TGT_GENO) {
        int s = (int) (target & 0xFFFF);
        if (!geno_state_valid(st, s)) {
            Geno_Event(8, fp->kind, fp->player_id, s, 0);
            return 0;
        }
        return geno_enter_state(gobj, fp, st, s, target);
    }
    msid = geno_target_motion(fp, target);
    if (msid < 0 || (msid < (int) fp->x18 && fp->x1C_actionStateList == NULL) ||
        (msid >= (int) fp->x18 && (fp->x20_actionStateList == NULL || msid - (int) fp->x18 > 0xFF)))
    {
        return 0;
    }
    st->changes++;
    Geno_Event(7, fp->kind, fp->player_id, (int) fp->motion_id, (int) target);
    if (geno_test_capture) {
        geno_test_target = target;
        geno_test_changes++;
        fp->motion_id = msid;
        Geno_OnActionChange(gobj);
        return 1;
    }
    if (target & GENO_TGT_KEEP_FRAME) {
        Fighter_ChangeMotionState(gobj, msid, GENO_KEEP_FRAME_FLAGS, fp->cur_anim_frame, 1.0f,
                                  0.0f, NULL);
        return 1;
    }
    if (!(target & GENO_TGT_RAW)) {
        /* the common states Geno enters the way the game does */
        switch (msid) {
        case ftCo_MS_Wait:
            if (fp->ground_or_air == GA_Ground) {
                ft_8008A2BC(gobj);
            } else {
                ftCo_Fall_Enter(gobj);
            }
            return 1;
        case ftCo_MS_Fall:
            ftCo_Fall_Enter(gobj);
            return 1;
        case ftCo_MS_FallSpecial:
            ftCo_800968C8(gobj);
            return 1;
        case ftCo_MS_Landing:
            if (fp->ground_or_air == GA_Ground) {
                ftCo_Landing_Enter_Basic(gobj);
                return 1;
            }
            break;
        case ftCo_MS_LandingFallSpecial:
            if (fp->ground_or_air == GA_Ground) {
                ftCo_LandingFallSpecial_Enter_Basic(gobj);
                return 1;
            }
            break;
        default:
            break;
        }
    }
    Fighter_ChangeMotionState(gobj, msid, Ft_MF_None, 0.0f, 1.0f, 0.0f, NULL);
    return 1;
}

static int geno_cond(Fighter_GObj* gobj, Fighter* fp, GenoState* st, const GenoCond* c)
{
    int kind = (c->head >> 8) & 0xFF;
    int b_is_var = (c->head & 0x80) != 0;
    int cmp = (c->head >> 4) & 7;
    int r = 0;
    switch (kind) {
    case GENO_COND_ALWAYS:
        r = 1;
        break;
    case GENO_COND_ANIM_END:
        r = geno_test_capture ? geno_test_anim_end : !ftAnim_IsFramesRemaining(gobj);
        break;
    case GENO_COND_GROUND:
        r = fp->ground_or_air == GA_Ground;
        break;
    case GENO_COND_AIR:
        r = fp->ground_or_air == GA_Air;
        break;
    case GENO_COND_PRESSED:
        r = (geno_buttons(fp, 1) & c->arg1) != 0;
        break;
    case GENO_COND_HELD:
        r = (geno_buttons(fp, 0) & c->arg1) != 0;
        break;
    case GENO_COND_BIT: {
        GenoWord v = geno_var_as(st, c->arg1 & 0xFF, 0);
        r = (v.i >> (c->arg2 & 31)) & 1;
        break;
    }
    case GENO_COND_VAR: {
        int a = c->arg1 & 0xFF;
        int is_f = geno_var_is_float(a);
        r = geno_cmp(is_f, geno_var_as(st, a, is_f), cmp,
                     geno_operand(st, is_f, b_is_var, c->arg2));
        break;
    }
    case GENO_COND_FRAME:
        r = fp->cur_anim_frame >= (f32) (s32) c->arg1;
        break;
    case GENO_COND_VALUE: {
        int is_f = geno_val_is_float(c->arg1);
        r = geno_cmp(is_f, geno_val_get(fp, st, c->arg1), cmp,
                     geno_operand(st, is_f, b_is_var, c->arg2));
        break;
    }
    default:
        r = 0;
        break;
    }
    return (c->head & GENO_CHG_NOT) ? !r : r;
}

static int geno_check_true(Fighter_GObj* gobj, Fighter* fp, GenoState* st, const GenoCheck* k,
                           int from)
{
    u32 j;
    for (j = (u32) from; j < k->ncond && j < GENO_CHECK_CONDS; j++) {
        if (!geno_cond(gobj, fp, st, &k->cond[j])) {
            return 0;
        }
    }
    return 1;
}

/* CHG: register (or re-find: a script loop running the same CHG twice registers it once). */
static void geno_register_check(Fighter* fp, GenoState* st, u32 head, u32 target, u32 a1, u32 a2)
{
    u32 i, once = (head & GENO_CHG_ONCE) ? 1u : 0u;
    GenoCheck* k;
    head &= 0xFFFFu & ~(u32) GENO_CHG_ONCE;
    for (i = 0; i < st->nchecks && i < GENO_MAX_CHECKS; i++) {
        k = &st->checks[i];
        if (k->target == target && k->once == once && k->cond[0].head == head &&
            k->cond[0].arg1 == a1 && k->cond[0].arg2 == a2)
        {
            st->last_check = (s32) i;
            return;
        }
    }
    if (st->nchecks >= GENO_MAX_CHECKS) {
        Geno_Event(15, fp->kind, fp->player_id, GENO_MAX_CHECKS, 0);
        st->last_check = -1;
        return;
    }
    k = &st->checks[st->nchecks];
    geno_zero(k, sizeof(*k));
    k->target = target;
    k->once = once;
    k->ncond = 1;
    k->cond[0].head = head;
    k->cond[0].arg1 = a1;
    k->cond[0].arg2 = a2;
    st->last_check = (s32) st->nchecks;
    st->nchecks++;
}

static void geno_and_check(GenoState* st, u32 head, u32 a1, u32 a2)
{
    GenoCheck* k;
    u32 j;
    if (st->last_check < 0 || (u32) st->last_check >= st->nchecks) {
        return;
    }
    k = &st->checks[st->last_check];
    head &= 0xFFFFu & ~(u32) GENO_CHG_ONCE;
    for (j = 0; j < k->ncond; j++) {
        if (k->cond[j].head == head && k->cond[j].arg1 == a1 && k->cond[j].arg2 == a2) {
            return; /* already there (a loop re-ran it) */
        }
    }
    if (k->ncond >= GENO_CHECK_CONDS) {
        return;
    }
    k->cond[k->ncond].head = head;
    k->cond[k->ncond].arg1 = a1;
    k->cond[k->ncond].arg2 = a2;
    k->ncond++;
}

/* Drop the ONCE checks (they have had their one test). */
static void geno_drop_once(GenoState* st)
{
    u32 i, n = 0;
    for (i = 0; i < st->nchecks && i < GENO_MAX_CHECKS; i++) {
        if (!st->checks[i].once) {
            if (n != i) {
                st->checks[n] = st->checks[i];
            }
            n++;
        }
    }
    if (n != st->nchecks) {
        st->nchecks = n;
        st->last_check = -1;
    }
}

/* ---- v1: rehit -------------------------------------------------------------------------------- */

static void geno_rehit_tick(Fighter* fp, GenoState* st)
{
    int i;
    for (i = 0; i < GENO_MAX_REHIT; i++) {
        if (st->rehit_period[i] > 0 && ++st->rehit_count[i] >= st->rehit_period[i]) {
            st->rehit_count[i] = 0;
            lbColl_80008440(&fp->x914[i]); /* Melee's "clear the hit list" */
            Geno_Event(10, fp->kind, fp->player_id, 1 << i, st->rehit_period[i]);
        }
    }
}

/* ---- v1 dispatch points ----------------------------------------------------------------------- */

/* fighter.c (Fighter_8006A360), after the animation and the subaction script advanced and BEFORE
 * the state's anim callback: the frame tick, rehit timers, then the registered change-action
 * checks. Returns 1 when a check changed the action (the caller then skips the old state's anim
 * callback, exactly as when that callback itself changes the state). */
int Geno_PreAnim(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    u32 i, n;
    s32 hit = -1;
    u32 target = 0;
    if (geno_inert(st)) {
        return 0;
    }
    st->action_time++;
    geno_rehit_tick(fp, st);
    n = st->nchecks;
    for (i = 0; i < n && i < GENO_MAX_CHECKS; i++) {
        if (geno_check_true(gobj, fp, st, &st->checks[i], 0)) {
            if (!geno_target_ok(fp, st->checks[i].target)) {
                /* a Geno state this profile does not declare: logged, and a later check (the
                   translator's fallback) may still win */
                Geno_Event(8, fp->kind, fp->player_id, (int) (st->checks[i].target & 0xFFFF), 0);
                continue;
            }
            hit = (s32) i;
            target = st->checks[i].target;
            break;
        }
    }
    if (n != 0) {
        geno_drop_once(st);
    }
    if (hit >= 0) {
        return geno_do_change(gobj, fp, st, target);
    }
    /* v2: the glide's jump-hold entry (only profiles with Geno states) */
    return st->profile >= 0 ? geno_v2_preanim(gobj, fp, st) : 0;
}

/* Fighter_procMap, around the state's collision callback. A landing (or take-off) edge inside it
 * picks a target; it is performed right after the callback, so it wins over whatever the state did
 * on landing. Edges outside the callback (a hit launching a grounded fighter, a state's entry)
 * never trigger one. */
void Geno_CollBegin(Fighter_GObj* gobj)
{
    GenoState* st = geno_state(GET_FIGHTER(gobj));
    if (geno_inert(st)) {
        return;
    }
    st->in_coll = 1;
    st->edge_pending = 0;
}

void Geno_CollEnd(Fighter_GObj* gobj)
{
    Fighter* fp = GET_FIGHTER(gobj);
    GenoState* st = geno_state(fp);
    if (geno_inert(st)) {
        return;
    }
    st->in_coll = 0;
    if (st->edge_pending) {
        st->edge_pending = 0;
        geno_do_change(gobj, fp, st, st->edge_target);
    }
}

/* ftcommon.c: landed (ftCommon_8007D6A4, landing = 1) or left the ground (ftCommon_8007D5D4 /
 * 8007D60C, landing = 0). Called after ground_or_air changed. */
void Geno_GroundEdge(Fighter* fp, int landing)
{
    GenoState* st = geno_state(fp);
    Fighter_GObj* gobj = fp->gobj;
    u32 i;
    int want = landing ? GENO_COND_GROUND : GENO_COND_AIR;
    if (geno_inert(st) || gobj == NULL) {
        return;
    }
    if (landing && st->profile >= 0) {
        geno_run_event(gobj, st, GENO_EV_LAND); /* on_land hooks: the moment of landing */
    }
    if (!st->in_coll || st->edge_pending) {
        return;
    }
    /* 1. the script's own checks whose first condition is this edge */
    for (i = 0; i < st->nchecks && i < GENO_MAX_CHECKS; i++) {
        GenoCheck* k = &st->checks[i];
        if (((k->cond[0].head >> 8) & 0xFF) == (u32) want && !(k->cond[0].head & GENO_CHG_NOT) &&
            geno_target_ok(fp, k->target) && geno_check_true(gobj, fp, st, k, 1))
        {
            st->edge_pending = 1;
            st->edge_target = k->target;
            Geno_Event(9, fp->kind, fp->player_id, (int) fp->motion_id, (int) k->target);
            return;
        }
    }
    /* 2. geno.json on_land: from this motion to a target */
    if (landing && st->profile >= 0) {
        int n = Geno_OnLandCount(st->profile);
        int j;
        for (j = 0; j < n; j++) {
            if (geno_target_motion(fp, (u32) Geno_OnLandFrom(st->profile, j)) ==
                (int) fp->motion_id)
            {
                st->edge_pending = 1;
                st->edge_target = (u32) Geno_OnLandTo(st->profile, j);
                Geno_Event(9, fp->kind, fp->player_id, (int) fp->motion_id, (int) st->edge_target);
                return;
            }
        }
    }
}

/* ftcoll.c (ftColl_8007A06C, a fighter's hitbox won the hit): LINK - Brawl's autolink angle 365.
 * The victim is launched along the attacker's momentum: `dir` / `angle` are rewritten so the
 * launch vector (-dir * cos(angle), sin(angle)) points where the attacker is going. Mode SPEED
 * also raises `kb` so the launch speed is at least the attacker's. An attacker slower than 0.05
 * keeps the hitbox's own Melee angle. Returns 1 when it changed anything. */
int Geno_Autolink(Fighter* attacker, HitCapsule* hit, float* dir, float* angle, float* kb)
{
    GenoState* st;
    int idx, mode;
    float vx, vy, speed2, deg;
    if (attacker == NULL || hit < &attacker->x914[0] || hit >= &attacker->x914[4]) {
        return 0;
    }
    st = geno_state(attacker);
    if (geno_inert(st)) {
        return 0;
    }
    idx = (int) (hit - &attacker->x914[0]);
    mode = st->link_mode[idx];
    if (mode == GENO_LINK_OFF) {
        return 0;
    }
    if (attacker->ground_or_air == GA_Ground) {
        vx = attacker->gr_vel;
        vy = 0.0f;
    } else {
        vx = attacker->self_vel.x;
        vy = attacker->self_vel.y;
    }
    speed2 = vx * vx + vy * vy;
    if (speed2 < 0.05f * 0.05f) {
        return 0;
    }
    *dir = vx > 0.0f ? -1.0f : 1.0f;
    deg = atan2f(vy, vx < 0.0f ? -vx : vx) * 57.29578f;
    if (deg < 0.0f) {
        deg += 360.0f;
    }
    *angle = (float) (s32) (deg + 0.5f);
    if (*angle >= 360.0f) {
        *angle -= 360.0f;
    }
    if (mode == GENO_LINK_SPEED && p_ftCommonData != NULL && p_ftCommonData->x100 > 0.0f) {
        float need = sqrtf(speed2) / p_ftCommonData->x100; /* launch speed = kb * x100 */
        if (*kb < need) {
            *kb = need;
        }
    }
    Geno_Event(11, attacker->kind, attacker->player_id, (int) *angle, (int) *kb);
    return 1;
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
#define GENO_W(i) (len > (i) ? w[i] : 0u)
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
    case GENO_SUB_DIV:
        geno_arith(st, sub, a, geno_operand_b(st, a, w0, GENO_W(1)));
        break;
    case GENO_SUB_RAND:
        /* the fast-forward pass replays commands of frames already gone: no RNG draw there */
        if (mode != GENO_MODE_SKIP) {
            geno_arith(st, sub, a, geno_operand_b(st, a, w0, GENO_W(1)));
        }
        break;
    case GENO_SUB_GET: {
        u32 id = GENO_W(1);
        int is_f = geno_val_is_float(id);
        GenoWord v = geno_val_get(fp, st, id);
        void* pa = geno_var(st, a);
        if (geno_var_is_float(a)) {
            *(f32*) pa = is_f ? v.f : (f32) v.i;
        } else {
            *(s32*) pa = is_f ? (s32) v.f : v.i;
        }
        break;
    }
    case GENO_SUB_PUT: {
        u32 id = GENO_W(1);
        geno_val_put(fp, id, geno_operand(st, geno_val_is_float(id), (w0 & 0x80) != 0, GENO_W(2)));
        break;
    }
    case GENO_SUB_IF:
        if (!geno_compare(st, a, (w0 >> 4) & 7, geno_operand_b(st, a, w0, GENO_W(1)))) {
            skip = GENO_W(2);
        }
        break;
    case GENO_SUB_IFV: {
        u32 id = GENO_W(1);
        int is_f = geno_val_is_float(id);
        if (!geno_cmp(is_f, geno_val_get(fp, st, id), (w0 >> 4) & 7,
                      geno_operand(st, is_f, (w0 & 0x80) != 0, GENO_W(2))))
        {
            skip = GENO_W(3);
        }
        break;
    }
    case GENO_SUB_SKIP:
        skip = GENO_W(1);
        break;
    case GENO_SUB_ORIG: {
        int slot = geno_slot_of(w);
        cmd->u = slot >= 0 ? Geno_OverlayOrig[slot] : NULL;
        return;
    }
    case GENO_SUB_CALL:
        if (mode != GENO_MODE_SKIP) {
            GenoGame_CallHook(gobj, (int) GENO_W(1), (s32) GENO_W(2));
        }
        break;
    case GENO_SUB_CHG:
        if (!(mode == GENO_MODE_SKIP && (w0 & GENO_CHG_ONCE))) {
            geno_register_check(fp, st, w0 & 0xFFFF, GENO_W(1), GENO_W(2), GENO_W(3));
        }
        break;
    case GENO_SUB_CHGAND:
        geno_and_check(st, w0 & 0xFFFF, GENO_W(1), GENO_W(2));
        break;
    case GENO_SUB_CHGCLR:
        st->nchecks = 0;
        st->last_check = -1;
        break;
    case GENO_SUB_REHIT: {
        int i;
        s32 n = (s32) GENO_W(1);
        for (i = 0; i < GENO_MAX_REHIT; i++) {
            if (a & (1 << i)) {
                st->rehit_period[i] = n > 0 ? n : 0;
                st->rehit_count[i] = 0;
            }
        }
        break;
    }
    case GENO_SUB_LINK: {
        int i;
        for (i = 0; i < GENO_MAX_REHIT; i++) {
            if (a & (1 << i)) {
                st->link_mode[i] = (s32) GENO_W(1);
            }
        }
        break;
    }
    default:
        Geno_Event(3, fp->kind, fp->player_id, sub, 0);
        break;
    }
#undef GENO_W
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

/* ---- v2 ---------------------------------------------------------------------------------------- */
#include "geno_game_v2.inc"
