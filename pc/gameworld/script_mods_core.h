/* script_mods_core.h - the Envoy TRIGGERED-RULE evaluator, pure C (online Envoy, stage 4).
 *
 * The Lua engine (pc/scripts/examples/envoy/scripts/mod_engine.lua: emit / begin_frame / matches / conditions_hold / apply / drain) is the
 * reference. Offline it keeps running in Lua. Online nothing in Lua may decide a match, so the same rules run HERE: a flat numeric PROGRAM per
 * port (compiled from the records by E:trigger_program, every $tier already resolved, durations already scaled) and a small STATE (7 status
 * slots, the recent-event frames, a bounded event queue). No Lua, no allocation, no pointers, no libc, no game types: the file is included by
 * script_mods.inc (the game side, state in snapshotted BSS) and by the standalone parity test pc/tests/script_mods_core_test.c.
 *
 * Everything is a pure function of (program, state, the events emitted, the per-frame player sample). Ports are 0-based here (the Lua side is 1-based).
 * The vocabulary numbers below are mirrored by mod_engine.lua (E.native.*): the parity test checks them against the fixture.
 */
#ifndef SCRIPT_MODS_CORE_H
#define SCRIPT_MODS_CORE_H

#define SM_PORTS 2          /* the two seats of an online set (GW_NB_SLOTS) */
#define SM_MAXR 24          /* rule instances per port */
#define SM_NTRIG 4          /* a record's trigger plus up to 3 alternatives (`also`) */
#define SM_NCOND 8          /* flattened conditions per trigger */
#define SM_NEFF 8
#define SM_QUEUE 128        /* mod_engine: #queue>=128 drops */
#define SM_DEPTH 8          /* mod_engine default chain depth */
#define SM_BUDGET 64        /* mod_engine default effects per frame */
#define SM_FX 32
#define SM_NEV 48           /* event kind ids are 1..SM_NEV-1 */
#define SM_NSTATUS 7
#define SM_MAGIC 0x534D5031

/* event kinds (E.native.events) */
enum {
    SM_E_NONE = 0, SM_E_HIT_DEALT, SM_E_HIT_TAKEN, SM_E_KO_DEALT, SM_E_STOCK_LOST, SM_E_SHIELD_HIT, SM_E_PERFECT_SHIELD, SM_E_CLANK,
    SM_E_JUMP, SM_E_AIR_JUMP, SM_E_LANDING, SM_E_LEDGE_GRAB, SM_E_GRAB, SM_E_THROW, SM_E_TAUNT, SM_E_INTERVAL,
    SM_E_STATUS_APPLIED, SM_E_STATUS_REMOVED, SM_E_STACKS_CHANGED, SM_E_CRIT, SM_E_ARMOR,
    SM_E_LCANCEL, SM_E_LCANCEL_HIT, SM_E_LCANCEL_MISS, SM_E_WAVEDASH, SM_E_WAVELAND, SM_E_LEDGE_DASH, SM_E_AIR_DODGE, SM_E_TECH,
    SM_E_TECH_MISS, SM_E_DASH_DANCE, SM_E_SHORT_HOP, SM_E_FULL_HOP, SM_E_FAST_FALL, SM_E_SHIELD_DROP, SM_E_JC_GRAB, SM_E_JC_USMASH,
    SM_E_SDI, SM_E_AUTO_CANCEL, SM_E_COMBO, SM_E_COMBO_END
};
/* statuses (D.mod_status.order): ids 1..7 */
enum { SM_S_BURN = 1, SM_S_SHOCK, SM_S_CHILL, SM_S_CURSE, SM_S_HASTE, SM_S_GUARDED, SM_S_MOMENTUM };
/* condition kinds */
enum {
    SM_C_NONE = 0, SM_C_TAG, SM_C_STATUS, SM_C_SELF_STATUS, SM_C_TARGET_STATUS, SM_C_SELF_ABOVE, SM_C_SELF_BELOW, SM_C_TARGET_ABOVE,
    SM_C_GROUNDED, SM_C_AIRBORNE, SM_C_LAST_STOCK, SM_C_NEVER, SM_C_COMBO_AT_LEAST, SM_C_COMBO_DAMAGE_ABOVE, SM_C_HIT, SM_C_AERIAL,
    SM_C_DIRECTION, SM_C_STRENGTH_ABOVE, SM_C_ARMOR_RESULT, SM_C_AIR_FRAMES_ABOVE, SM_C_AERIAL_HIT, SM_C_RECENTLY
};
/* effect ops */
enum {
    SM_X_NONE = 0, SM_X_STATUS, SM_X_STACKS, SM_X_CHAIN, SM_X_REMOVE, SM_X_CLANK_DAMAGE, SM_X_HEAL, SM_X_DAMAGE, SM_X_ARMOR,
    SM_X_INTANGIBLE, SM_X_INTERRUPT, SM_X_CRIT_NEXT, SM_X_EMIT
};
/* fx kinds (outputs the game side turns into setter calls) */
enum { SM_FX_NONE = 0, SM_FX_ARMOR, SM_FX_INTANGIBLE, SM_FX_INTERRUPT, SM_FX_CRIT_NEXT };
/* tag bits (E.native.tags): bit i = the i-th word */
enum {
    SM_T_JAB = 1u << 0, SM_T_TILT = 1u << 1, SM_T_SMASH = 1u << 2, SM_T_AERIAL = 1u << 3, SM_T_SPECIAL = 1u << 4, SM_T_GRAB = 1u << 5,
    SM_T_THROW = 1u << 6, SM_T_PROJECTILE = 1u << 7, SM_T_DASH_ATTACK = 1u << 8, SM_T_GROUNDED = 1u << 9, SM_T_AIRBORNE = 1u << 10,
    SM_T_NORMAL = 1u << 11, SM_T_FIRE = 1u << 12, SM_T_ELECTRIC = 1u << 13, SM_T_ICE = 1u << 14, SM_T_DARKNESS = 1u << 15,
    SM_T_BURNING = 1u << 16, SM_T_SHOCKED = 1u << 17, SM_T_CHILLED = 1u << 18, SM_T_CURSED = 1u << 19, SM_T_HASTED = 1u << 20,
    SM_T_GUARDED = 1u << 21, SM_T_MOMENTUM = 1u << 22, SM_T_DAMAGE = 1u << 23, SM_T_HEALING = 1u << 24, SM_T_UNIQUE = 1u << 25,
    SM_T_KEYSTONE = 1u << 26, SM_T_TECHNIQUE = 1u << 27, SM_T_CRITICAL = 1u << 28
};

/* ---- program layout (flat words; floats are bit patterns) -------------------------------------------------------------------------- */
#define SM_CW 3                                  /* condition: kind, a, b */
#define SM_TW (2 + SM_NCOND * SM_CW)             /* trigger: event, ncond, conds */
#define SM_EW 20                                 /* effect, see SM_EF_* */
#define SM_RW (4 + SM_NTRIG * SM_TW + SM_NEFF * SM_EW) /* rule: interval, ntrig, neff, id-hash, triggers, effects */
#define SM_HDR 136                               /* magic, nrules, nvariants, digest-seed, 4 spare, then 128 mask->variant words */
#define SM_MASKS 128
#define SM_PROG_WORDS (SM_HDR + SM_MAXR * SM_RW)
/* effect word offsets */
enum {
    SM_EF_OP = 0, SM_EF_SUBJECT, SM_EF_WHEN, SM_EF_STATUS, SM_EF_DURATION, SM_EF_MAX, SM_EF_REFRESH, SM_EF_COUNT, SM_EF_FRAMES, SM_EF_ATYPE,
    SM_EF_VALUE /* f */, SM_EF_AMOUNT /* f (heal: negative) */, SM_EF_DIRECTION, SM_EF_EXITS, SM_EF_FLAGS, SM_EF_EVENT, SM_EF_TAG
};
#define SM_FLAG_GUARD 1
#define SM_FLAG_RESTORE 2
#define SM_FLAG_EXITS 4

typedef struct { int w[SM_PROG_WORDS]; int loaded; unsigned digest; } SmProgram;

typedef struct { int on, stacks, max, expires, next_tick, cause; float amount; } SmStatus;
typedef struct {
    int has_percent, has_grounded, has_stocks, has_air, has_ahit;
    float percent;
    int grounded, stocks, air_frames, aerial_hit;
} SmCtx;
typedef struct {
    int kind, port, target, depth;
    unsigned tags;
    int status, hit, aerial, direction, count, absorbed, broke;
    int has_damage, has_strength, has_clank;
    float damage, strength, damage_a, damage_b;
    int has_self, has_target;
    SmCtx self, target_ctx;
} SmEvent;
typedef struct { int present, grounded, stocks, has_pos; float percent, x, y; } SmPlayer;
typedef struct { int op, port, atype, frames, direction, count, exits, flags; float value; } SmFx;
typedef struct {
    SmProgram prog[SM_PORTS];
    SmStatus st[SM_PORTS][SM_NSTATUS + 1];
    int recent[SM_PORTS][SM_NEV];            /* frame of the last event of each kind (0 = never) */
    int frame, used, dropped, nq, nfx;
    float damage[SM_PORTS], sustain[SM_PORTS];
    SmPlayer players[SM_PORTS];
    SmEvent queue[SM_QUEUE];
    SmFx fx[SM_FX];
} SmWorld;

static unsigned sm_mix(unsigned h, unsigned v) { h ^= v; h *= 0x01000193u; h ^= h >> 15; return h; }
static unsigned sm_fbits(float f) { union { float f; unsigned u; } v; v.f = f; return v.u; }
static float sm_bitsf(int i) { union { float f; int i; } v; v.i = i; return v.f; }
static unsigned sm_program_digest(const SmProgram* p) {
    unsigned h = 0x811C9DC5u; int i;
    for (i = 0; i < SM_PROG_WORDS; ++i) h = sm_mix(h, (unsigned) p->w[i]);
    return h;
}

/* One word of a program (called by the loader while arming a match). Returns 0 when out of range. */
static int sm_prog_word(SmWorld* w, int port, int index, int value) {
    if (port < 0 || port >= SM_PORTS || index < 0 || index >= SM_PROG_WORDS) return 0;
    w->prog[port].w[index] = value;
    return 1;
}
/* Seal a program: checks the header and every bound the evaluator relies on, computes the digest. 0 = refused (the program stays unloaded). */
static int sm_prog_seal(SmWorld* w, int port) {
    SmProgram* p; int r, t, c, e, i;
    if (port < 0 || port >= SM_PORTS) return 0;
    p = &w->prog[port]; p->loaded = 0;
    if (p->w[0] != SM_MAGIC || p->w[1] < 0 || p->w[1] > SM_MAXR) return 0;
    for (i = 0; i < SM_MASKS; ++i) if (p->w[8 + i] < 0 || p->w[8 + i] > 255) return 0;
    for (r = 0; r < p->w[1]; ++r) {
        const int* rw = &p->w[SM_HDR + r * SM_RW];
        if (rw[0] < 0 || rw[0] > 3600 || rw[1] < 1 || rw[1] > SM_NTRIG || rw[2] < 1 || rw[2] > SM_NEFF) return 0;
        for (t = 0; t < rw[1]; ++t) {
            const int* tw = &rw[4 + t * SM_TW];
            if (tw[0] < 1 || tw[0] >= SM_NEV || tw[1] < 0 || tw[1] > SM_NCOND) return 0;
            for (c = 0; c < tw[1]; ++c) { int k = tw[2 + c * SM_CW]; if (k < 1 || k > SM_C_RECENTLY) return 0; }
        }
        for (e = 0; e < rw[2]; ++e) {
            const int* ew = &rw[4 + SM_NTRIG * SM_TW + e * SM_EW];
            if (ew[SM_EF_OP] < 1 || ew[SM_EF_OP] > SM_X_EMIT || ew[SM_EF_WHEN] < 0 || ew[SM_EF_WHEN] >= SM_NEV) return 0;
            if (ew[SM_EF_STATUS] < 0 || ew[SM_EF_STATUS] > SM_NSTATUS || ew[SM_EF_SUBJECT] < 0 || ew[SM_EF_SUBJECT] > 1) return 0;
            if (ew[SM_EF_OP] == SM_X_EMIT && (ew[SM_EF_EVENT] < 1 || ew[SM_EF_EVENT] >= SM_NEV)) return 0;
        }
    }
    p->digest = sm_program_digest(p); p->loaded = 1;
    return 1;
}
static int sm_loaded(const SmWorld* w) { int p; for (p = 0; p < SM_PORTS; ++p) if (w->prog[p].loaded) return 1; return 0; }
static void sm_reset(SmWorld* w) {
    int i; unsigned char* b = (unsigned char*) w;
    for (i = 0; i < (int) sizeof *w; ++i) b[i] = 0;
}
/* Forget the state of a match (statuses, recents, queue), keep the programs. */
static void sm_clear_state(SmWorld* w) {
    int p, i;
    for (p = 0; p < SM_PORTS; ++p) {
        for (i = 0; i <= SM_NSTATUS; ++i) { SmStatus* s = &w->st[p][i]; s->on = s->stacks = s->max = s->expires = s->next_tick = s->cause = 0; s->amount = 0; }
        for (i = 0; i < SM_NEV; ++i) w->recent[p][i] = 0;
        w->damage[p] = w->sustain[p] = 0;
        w->players[p].present = 0;
    }
    w->frame = w->used = w->dropped = w->nq = w->nfx = 0;
}

/* the presence mask of a port: bit (id-1) per status, as the Lua side keys its variants */
static int sm_mask(const SmWorld* w, int port) {
    int i, m = 0;
    for (i = 1; i <= SM_NSTATUS; ++i) if (w->st[port][i].on) m |= 1 << (i - 1);
    return m;
}
static int sm_variant(const SmWorld* w, int port) { return w->prog[port].w[8 + (sm_mask(w, port) & ~2)]; }

/* ---- events -------------------------------------------------------------------------------------------------------------------------- */
static void sm_event_init(SmEvent* e, int kind, int port, int target) {
    unsigned char* b = (unsigned char*) e; int i;
    for (i = 0; i < (int) sizeof *e; ++i) b[i] = 0;
    e->kind = kind; e->port = port; e->target = target; e->depth = 1;
    e->aerial = e->direction = e->count = -1;
}
/* mod_engine E:emit / emit_trusted: false (and `dropped`) when too deep or the queue is full */
static int sm_emit(SmWorld* w, const SmEvent* e) {
    if (e->port < 0 || e->port >= SM_PORTS || e->kind < 1 || e->kind >= SM_NEV) return 0;
    if (e->depth > SM_DEPTH || w->nq >= SM_QUEUE) { w->dropped++; return 0; }
    w->queue[w->nq++] = *e;
    return 1;
}
static const unsigned sm_status_tag[SM_NSTATUS + 1] = { 0, SM_T_BURNING, SM_T_SHOCKED, SM_T_CHILLED, SM_T_CURSED, SM_T_HASTED, SM_T_GUARDED, SM_T_MOMENTUM };
/* afterimage cause (mod_skill.cause index) of the technique events; 0 = not a skill event */
static int sm_cause_of(int kind) {
    switch (kind) {
    case SM_E_LCANCEL: case SM_E_LCANCEL_HIT: case SM_E_AUTO_CANCEL: return 1;
    case SM_E_PERFECT_SHIELD: case SM_E_TECH: case SM_E_SDI: case SM_E_SHIELD_DROP: return 2;
    case SM_E_WAVEDASH: case SM_E_WAVELAND: case SM_E_LEDGE_DASH: case SM_E_AIR_DODGE: return 3;
    case SM_E_COMBO: case SM_E_COMBO_END: case SM_E_JC_GRAB: case SM_E_JC_USMASH: return 4;
    case SM_E_SHORT_HOP: case SM_E_FULL_HOP: case SM_E_FAST_FALL: case SM_E_DASH_DANCE: return 5;
    case SM_E_LCANCEL_MISS: case SM_E_TECH_MISS: return 6;
    }
    return 0;
}

static const SmStatus* sm_status(const SmWorld* w, int port, int id) {
    if (port < 0 || port >= SM_PORTS || id < 1 || id > SM_NSTATUS) return 0;
    return w->st[port][id].on ? &w->st[port][id] : 0;
}
static int sm_any_status(const SmWorld* w, int port) {
    int i;
    if (port < 0 || port >= SM_PORTS) return 0;
    for (i = 1; i <= SM_NSTATUS; ++i) if (w->st[port][i].on) return 1;
    return 0;
}
static void sm_sustain_damage(SmWorld* w, int port, float amount) {
    float next = w->sustain[port] + amount, delta;
    if (next > 100.0f) next = 100.0f;
    if (next < -100.0f) next = -100.0f;
    delta = next - w->sustain[port];
    w->sustain[port] = next;
    w->damage[port] += delta;
}

/* mod_engine E:begin_frame: the frame counter, Burn ticks, expiry (in status order), then one interval event per present port */
static void sm_begin_frame(SmWorld* w, const SmPlayer* players) {
    int p, i;
    w->frame++;
    for (p = 0; p < SM_PORTS; ++p) { w->players[p] = players[p]; w->damage[p] = 0; w->sustain[p] = 0; }
    w->used = 0; w->nfx = 0;
    for (p = 0; p < SM_PORTS; ++p) {
        for (i = 1; i <= SM_NSTATUS; ++i) {
            SmStatus* v = &w->st[p][i];
            if (!v->on) continue;
            if (i == SM_S_BURN && v->next_tick <= w->frame) {
                sm_sustain_damage(w, p, v->amount * (float) v->stacks);
                v->next_tick += 60;
            }
            if (v->expires <= w->frame) {
                SmEvent e;
                v->on = 0; v->stacks = v->max = v->expires = v->next_tick = v->cause = 0; v->amount = 0;
                sm_event_init(&e, SM_E_STATUS_REMOVED, p, -1); e.status = i;
                sm_emit(w, &e);
            }
        }
    }
    for (p = 0; p < SM_PORTS; ++p) if (w->players[p].present) {
        SmEvent e;
        sm_event_init(&e, SM_E_INTERVAL, p, -1);
        sm_emit(w, &e);
    }
}

/* mod_engine E:conditions_hold for one flattened condition */
static int sm_cond(const SmWorld* w, const SmEvent* e, const int* c) {
    int kind = c[0], a = c[1];
    SmCtx own, tgt;
    {   /* own = e.self_context or players[e.port] or {} (same for the target) */
        unsigned char* b; int i;
        if (e->has_self) own = e->self;
        else {
            b = (unsigned char*) &own; for (i = 0; i < (int) sizeof own; ++i) b[i] = 0;
            if (e->port >= 0 && e->port < SM_PORTS && w->players[e->port].present) {
                own.has_percent = own.has_grounded = own.has_stocks = 1;
                own.percent = w->players[e->port].percent; own.grounded = w->players[e->port].grounded; own.stocks = w->players[e->port].stocks;
            }
        }
        if (e->has_target) tgt = e->target_ctx;
        else {
            b = (unsigned char*) &tgt; for (i = 0; i < (int) sizeof tgt; ++i) b[i] = 0;
            if (e->target >= 0 && e->target < SM_PORTS && w->players[e->target].present) {
                tgt.has_percent = tgt.has_grounded = tgt.has_stocks = 1;
                tgt.percent = w->players[e->target].percent; tgt.grounded = w->players[e->target].grounded; tgt.stocks = w->players[e->target].stocks;
            }
        }
    }
    switch (kind) {
    case SM_C_TAG: return (e->tags & (unsigned) a) != 0;
    case SM_C_STATUS: return a == 0 ? e->status != 0 : e->status == a;
    case SM_C_SELF_STATUS: return a == 0 ? sm_any_status(w, e->port) : sm_status(w, e->port, a) != 0;
    case SM_C_TARGET_STATUS: return a == 0 ? sm_any_status(w, e->target) : sm_status(w, e->target, a) != 0;
    case SM_C_SELF_ABOVE: return own.has_percent && own.percent > sm_bitsf(c[2]);
    case SM_C_SELF_BELOW: return own.has_percent && own.percent < sm_bitsf(c[2]);
    case SM_C_TARGET_ABOVE: return tgt.has_percent && tgt.percent > sm_bitsf(c[2]);
    case SM_C_GROUNDED: return own.has_grounded && ((own.grounded != 0) == (a != 0));
    case SM_C_AIRBORNE: return own.has_grounded && ((own.grounded == 0) == (a != 0));
    case SM_C_LAST_STOCK: return (((own.has_stocks ? own.stocks : 0) == 1) == (a != 0));
    case SM_C_NEVER: return 0;
    case SM_C_COMBO_AT_LEAST: return (e->count < 0 ? 0 : e->count) >= a;
    case SM_C_COMBO_DAMAGE_ABOVE: return (e->has_damage ? e->damage : 0.0f) > sm_bitsf(c[2]);
    case SM_C_HIT: return (e->hit != 0) == (a != 0);
    case SM_C_AERIAL: return e->aerial == a;
    case SM_C_DIRECTION: return e->direction == a;
    case SM_C_STRENGTH_ABOVE: return (e->has_strength ? e->strength : 0.0f) > sm_bitsf(c[2]);
    case SM_C_ARMOR_RESULT: return (a == 1 && e->absorbed) || (a == 2 && e->broke);
    case SM_C_AIR_FRAMES_ABOVE: return (own.has_air ? own.air_frames : 0) > a;
    case SM_C_AERIAL_HIT: return (own.has_ahit && own.aerial_hit != 0) == (a != 0);
    case SM_C_RECENTLY: {
        int f = (e->port >= 0 && e->port < SM_PORTS && a >= 1 && a < SM_NEV) ? w->recent[e->port][a] : 0;
        if (!f || w->frame - f > c[2]) return 0;
        return 1;
    }
    }
    return 0;
}
static int sm_conds(const SmWorld* w, const SmEvent* e, const int* trig) {
    int i, n = trig[1];
    for (i = 0; i < n; ++i) if (!sm_cond(w, e, &trig[2 + i * SM_CW])) return 0;
    return 1;
}
/* mod_engine E:matches */
static int sm_matches(const SmWorld* w, const SmEvent* e, const int* rule) {
    int t, ntrig = rule[1];
    const int* tw = &rule[4];
    int main_ok = tw[0] == e->kind;
    if (!main_ok) {
        int any = 0;
        for (t = 1; t < ntrig; ++t) if (rule[4 + t * SM_TW] == e->kind) { any = 1; break; }
        if (!any) return 0;
    }
    if (main_ok && e->kind == SM_E_INTERVAL && (rule[0] <= 0 || (w->frame % rule[0]) != 0)) return 0;
    if (main_ok && sm_conds(w, e, tw)) return 1;
    for (t = 1; t < ntrig; ++t) {
        const int* aw = &rule[4 + t * SM_TW];
        if (aw[0] == e->kind && sm_conds(w, e, aw)) return 1;
    }
    return 0;
}
static int sm_nearest_other(const SmWorld* w, const SmEvent* e) {
    int p, best = -1; float bd = 0;
    if (e->target < 0 || e->target >= SM_PORTS || !w->players[e->target].present || !w->players[e->target].has_pos) return -1;
    for (p = 0; p < SM_PORTS; ++p) {
        float dx, dy, d;
        if (p == e->port || p == e->target || !w->players[p].present || !w->players[p].has_pos) continue;
        dx = w->players[p].x - w->players[e->target].x; dy = w->players[p].y - w->players[e->target].y; d = dx * dx + dy * dy;
        if (best < 0 || d < bd) { best = p; bd = d; }
    }
    return best;
}
static void sm_fx_add(SmWorld* w, int op, int port, int atype, int frames, int direction, int count, int exits, int flags, float value) {
    SmFx* f;
    if (w->nfx >= SM_FX) { w->dropped++; return; }
    f = &w->fx[w->nfx++];
    f->op = op; f->port = port; f->atype = atype; f->frames = frames; f->direction = direction; f->count = count; f->exits = exits; f->flags = flags; f->value = value;
}
/* mod_engine E:apply: every effect of one matched rule instance for event e */
static void sm_apply(SmWorld* w, const SmEvent* e, const int* rule) {
    int k, neff = rule[2];
    for (k = 0; k < neff; ++k) {
        const int* ef = &rule[4 + SM_NTRIG * SM_TW + k * SM_EW];
        int port, op;
        if (w->used >= SM_BUDGET) { w->dropped++; break; }
        if (ef[SM_EF_WHEN] && ef[SM_EF_WHEN] != e->kind) continue;
        w->used++;
        op = ef[SM_EF_OP];
        port = ef[SM_EF_SUBJECT] ? e->target : e->port;
        if (op == SM_X_CHAIN) port = sm_nearest_other(w, e);
        if (port < 0 || port >= SM_PORTS || !w->players[port].present) continue;
        if (op == SM_X_STATUS || op == SM_X_STACKS || op == SM_X_CHAIN) {
            int id = ef[SM_EF_STATUS], dur = ef[SM_EF_DURATION], cause = sm_cause_of(e->kind);
            float amount = sm_bitsf(ef[SM_EF_AMOUNT]);
            SmStatus* v = &w->st[port][id];
            int expires = w->frame + dur;
            SmEvent ne;
            if (id < 1 || id > SM_NSTATUS) continue;
            if (v->on) {
                int mx = ef[SM_EF_MAX];
                if (mx > v->max) v->max = mx;
                v->stacks = v->stacks + 1 < v->max ? v->stacks + 1 : v->max;
                if (ef[SM_EF_REFRESH] == 0) v->expires = expires;
                else if (ef[SM_EF_REFRESH] == 1) { int cap = w->frame + 3600, ex = v->expires + dur; v->expires = ex < cap ? ex : cap; }
                if (amount > v->amount) v->amount = amount;
            } else {
                v->on = 1; v->expires = expires; v->stacks = 1; v->max = ef[SM_EF_MAX]; v->amount = amount; v->next_tick = w->frame + 60; v->cause = 0;
            }
            if (cause) v->cause = cause;
            sm_event_init(&ne, SM_E_STATUS_APPLIED, port, e->target); ne.depth = e->depth + 1; ne.status = id; ne.tags = sm_status_tag[id];
            sm_emit(w, &ne);
            if (op == SM_X_STACKS || v->stacks > 1) {
                sm_event_init(&ne, SM_E_STACKS_CHANGED, port, -1); ne.depth = e->depth + 1; ne.status = id; ne.tags = sm_status_tag[id];
                sm_emit(w, &ne);
            }
        } else if (op == SM_X_REMOVE) {
            int id = ef[SM_EF_STATUS], count = ef[SM_EF_COUNT];
            SmStatus* v = (id >= 1 && id <= SM_NSTATUS) ? &w->st[port][id] : 0;
            SmEvent ne;
            if (!v || !v->on) continue;
            if (count > 0 && v->stacks > count) {
                v->stacks -= count;
                sm_event_init(&ne, SM_E_STACKS_CHANGED, port, -1); ne.depth = e->depth + 1; ne.status = id; ne.tags = sm_status_tag[id];
                sm_emit(w, &ne);
            } else {
                v->on = 0; v->stacks = v->max = v->expires = v->next_tick = v->cause = 0; v->amount = 0;
                sm_event_init(&ne, SM_E_STATUS_REMOVED, port, -1); ne.depth = e->depth + 1; ne.status = id;
                sm_emit(w, &ne);
            }
        } else if (op == SM_X_CLANK_DAMAGE) {
            if (e->has_clank) w->damage[port] += e->damage_a + e->damage_b;
        } else if (op == SM_X_HEAL || op == SM_X_DAMAGE) {
            sm_sustain_damage(w, port, sm_bitsf(ef[SM_EF_AMOUNT]));   /* heal carries a negative amount */
        } else if (op == SM_X_ARMOR) {
            sm_fx_add(w, SM_FX_ARMOR, port, ef[SM_EF_ATYPE], ef[SM_EF_FRAMES], ef[SM_EF_DIRECTION], 0, 0, 0, sm_bitsf(ef[SM_EF_VALUE]));
        } else if (op == SM_X_INTANGIBLE) {
            sm_fx_add(w, SM_FX_INTANGIBLE, port, 0, ef[SM_EF_FRAMES], 0, 0, 0, 0, 0);
        } else if (op == SM_X_INTERRUPT) {
            sm_fx_add(w, SM_FX_INTERRUPT, port, 0, ef[SM_EF_FRAMES], 0, 0, ef[SM_EF_EXITS], ef[SM_EF_FLAGS], 0);
        } else if (op == SM_X_CRIT_NEXT) {
            sm_fx_add(w, SM_FX_CRIT_NEXT, port, 0, 0, 0, ef[SM_EF_COUNT], 0, 0, 0);
        } else if (op == SM_X_EMIT) {
            SmEvent ne;
            sm_event_init(&ne, ef[SM_EF_EVENT], port, e->target); ne.depth = e->depth + 1; ne.tags = (unsigned) ef[SM_EF_TAG];
            sm_emit(w, &ne);
        }
    }
}
/* mod_engine E:drain: the queue in order (it grows while it is read), then emptied */
static void sm_drain(SmWorld* w) {
    int at = 0, lost[SM_PORTS], p, r, i;
    for (p = 0; p < SM_PORTS; ++p) lost[p] = 0;
    while (at < w->nq) {
        SmEvent e = w->queue[at++];
        if (lost[e.port] && e.depth > 1) continue;
        if (w->used >= SM_BUDGET) w->dropped++;
        if (e.kind >= 1 && e.kind < SM_NEV) w->recent[e.port][e.kind] = w->frame;
        if (w->prog[e.port].loaded) {
            const SmProgram* pr = &w->prog[e.port];
            for (r = 0; r < pr->w[1]; ++r) {
                const int* rule = &pr->w[SM_HDR + r * SM_RW];
                if (w->used >= SM_BUDGET) break;
                if (sm_matches(w, &e, rule)) sm_apply(w, &e, rule);
            }
        }
        if (e.kind == SM_E_STOCK_LOST) {
            lost[e.port] = 1;
            for (i = 0; i <= SM_NSTATUS; ++i) { SmStatus* s = &w->st[e.port][i]; s->on = s->stacks = s->max = s->expires = s->next_tick = s->cause = 0; s->amount = 0; }
            for (i = 0; i < SM_NEV; ++i) w->recent[e.port][i] = 0;
            w->damage[e.port] = 0;
        }
    }
    w->nq = 0;
}
/* one logic frame */
static void sm_tick(SmWorld* w, const SmPlayer* players) { sm_begin_frame(w, players); sm_drain(w); }

/* The word the rollback hash sees: programs, statuses, recent frames, the counters. 0 when nothing is loaded. */
static unsigned sm_hash(const SmWorld* w) {
    unsigned h = 0x4D4F4453u; int p, i, q;
    if (!sm_loaded(w)) return 0;
    h = sm_mix(h, (unsigned) w->frame); h = sm_mix(h, (unsigned) w->dropped); h = sm_mix(h, (unsigned) w->nq);
    for (p = 0; p < SM_PORTS; ++p) {
        h = sm_mix(h, w->prog[p].loaded ? w->prog[p].digest : 0u);
        for (i = 1; i <= SM_NSTATUS; ++i) {
            const SmStatus* s = &w->st[p][i];
            h = sm_mix(h, s->on ? 1u : 0u);
            if (!s->on) continue;
            h = sm_mix(h, (unsigned) s->stacks); h = sm_mix(h, (unsigned) s->max); h = sm_mix(h, (unsigned) s->expires);
            h = sm_mix(h, (unsigned) s->next_tick); h = sm_mix(h, (unsigned) s->cause); h = sm_mix(h, sm_fbits(s->amount));
        }
        for (i = 1; i < SM_NEV; ++i) h = sm_mix(h, (unsigned) w->recent[p][i]);
    }
    for (q = 0; q < w->nq; ++q) { h = sm_mix(h, (unsigned) w->queue[q].kind); h = sm_mix(h, (unsigned) (w->queue[q].port * 16 + w->queue[q].target + 1)); h = sm_mix(h, w->queue[q].tags); }
    return h | 1u;
}
#endif
