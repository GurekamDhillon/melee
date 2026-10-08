/* gw_netdir.h - the RUN DIRECTOR of an online stage run (Classic over the 1P tables; stage 7a/7b of _research/envoy-netplay-scoping-2026-10-05.md,
 * brief docs/superpowers/plans/2026-10-08-envoy-online-stage7.md and its "Owner decisions"). Pure functions over integers: no floats, no locale, no
 * game memory, the same on Windows and Linux, compiled into gw_netplay.c and tested headless (netplay_dir_* there).
 *
 * WHAT THE DIRECTOR DECIDES (architecture R2): the stages are ordinary agreed Versus scenes (GS_VS); between them the frontend shows the interstitial.
 * Everything the director decides is a pure function of AGREED history, never of a peer's save, clock or global RNG:
 *   - the PLAN: stage `i` of the run (opponents, flags, difficulty, time) from the run seed, the loop, the agreed unlock mask and the two humans' fighters
 *   - the SHARED STOCKS of the pair: one pool; every stock either human loses in a stage comes out of it; a pool of 0 ends the stage as LOST
 *   - the verdict after a stage: next stage / run cleared / continue offered / run over for both (one continue token per run, spent by agreement)
 *   - the STAGE-END tuple each peer sends before it leaves a scene (epoch, exit frame, final hash, outcome); unequal tuples abort the run
 * The state that rides in the run record (gw_netrun.h RN2) is `flags`, packed here, so the record's digest covers it.
 *
 * flags (31 bits): 0-6 pool (0..99), 7-8 over (0 live, 1 lost, 2 cleared, 3 aborted), 9 continue token spent, 10-11 plan kind (0 plain stage run, 1 Classic),
 *                  12-17 plan length (stages in the run, 0 = endless), 18-19 continue tokens the run started with (0..3).
 */
#ifndef GW_NETDIR_H
#define GW_NETDIR_H

#include "gw_netrun.h"
#include <stdint.h>
#include <stdio.h>
#include <string.h>

enum { GW_ND_LIVE = 0, GW_ND_LOST = 1, GW_ND_CLEARED = 2, GW_ND_ABORTED = 3 };
enum { GW_ND_KIND_STAGES = 0, GW_ND_KIND_CLASSIC = 1 };
#define GW_ND_POOL_MAX 99
#define GW_ND_ENEMY_NONE 33 /* the tables' "no fighter" (ChKind_None in the retail data) */

static inline const char *gw_nd_over_name(int over) {
    return over == GW_ND_LOST ? "stocks" : over == GW_ND_CLEARED ? "cleared" : over == GW_ND_ABORTED ? "aborted" : "";
}

/* The run's director state, the part that is in the record. */
typedef struct {
    int kind;       /* GW_ND_KIND_* */
    int len;        /* stages in the run (0 = endless) */
    int stage;      /* zero-based stage about to be played / being played */
    int pool;       /* shared stocks left */
    int pool_start; /* what a continue refills it to */
    int cont_start; /* continue tokens the run began with */
    int cont_used;  /* tokens spent */
    int over;       /* GW_ND_* */
} GwNdRun;

static inline void gw_nd_init(GwNdRun *r, int kind, int len, int pool, int cont) {
    memset(r, 0, sizeof *r);
    r->kind = kind;
    r->len = len < 0 ? 0 : len > 63 ? 63 : len;
    r->pool = r->pool_start = pool < 1 ? 1 : pool > GW_ND_POOL_MAX ? GW_ND_POOL_MAX : pool;
    r->cont_start = cont < 0 ? 0 : cont > 3 ? 3 : cont;
}
static inline int gw_nd_cont_left(const GwNdRun *r) { return r->cont_start > r->cont_used ? r->cont_start - r->cont_used : 0; }
/* The continue is on offer: the pool ran out, the run is live and a token is left. */
static inline int gw_nd_asking(const GwNdRun *r) { return r->over == GW_ND_LIVE && r->pool == 0 && gw_nd_cont_left(r) > 0; }

static inline long gw_nd_flags(const GwNdRun *r) {
    return (long) (r->pool & 0x7F) | ((long) (r->over & 3) << 7) | ((long) (r->cont_used > 0) << 9) | ((long) (r->kind & 3) << 10) | ((long) (r->len & 63) << 12) |
           ((long) (r->cont_start & 3) << 18);
}

/* Shared stocks the pair lost in a stage that began with `each` stocks apiece and ended with s0 and s1 left (the two humans' ports). */
static inline int gw_nd_deaths(int each, int s0, int s1) {
    int d = 0;
    if (s0 < 0) s0 = 0;
    if (s1 < 0) s1 = 0;
    if (each > s0) d += each - s0;
    if (each > s1) d += each - s1;
    return d;
}

/* The verdict after a stage that cost `deaths` shared stocks: the pool pays; an empty pool offers the continue (asking) or ends the run (lost); a cleared
 * stage moves to the next, and the last one ends the run as cleared. Returns the new `over`. */
static inline int gw_nd_after_stage(GwNdRun *r, int deaths) {
    if (r->over != GW_ND_LIVE) return r->over;
    r->pool -= deaths < 0 ? 0 : deaths;
    if (r->pool < 0) r->pool = 0;
    if (r->pool == 0) {
        if (gw_nd_cont_left(r) == 0) r->over = GW_ND_LOST; /* else: asking - the same stage again if the pair agrees */
        return r->over;
    }
    r->stage++;
    if (r->len > 0 && r->stage >= r->len) r->over = GW_ND_CLEARED;
    return r->over;
}
/* The continue was agreed: a token is spent, the pool refills, the failed stage is played again. */
static inline int gw_nd_spend_continue(GwNdRun *r) {
    if (!gw_nd_asking(r)) return 0;
    r->cont_used++;
    r->pool = r->pool_start;
    return 1;
}

/* ---- the stage-end barrier ------------------------------------------------------------------------------------------------------------
 * "F <epoch> <frame> <hash 8 hex> <winner> <s0> <s1> <stage> <pool>" on the lobby channel, before a peer lets the scene go. winner -1 none, 0, 1. */
typedef struct {
    int epoch, frame, winner, s0, s1, stage, pool;
    uint32_t hash;
} GwNdEnd;

static inline int gw_nd_end_format(const GwNdEnd *e, char *out, size_t cap) {
    int n = snprintf(out, cap, "F %d %d %08x %d %d %d %d %d", e->epoch, e->frame, (unsigned) e->hash, e->winner, e->s0, e->s1, e->stage, e->pool);
    return n > 0 && (size_t) n < cap ? n : 0;
}
static inline int gw_nd_end_parse(const char *m, GwNdEnd *e) {
    unsigned h = 0;
    if (m == NULL || m[0] != 'F' || m[1] != ' ') return 0;
    if (sscanf(m + 2, "%d %d %x %d %d %d %d %d", &e->epoch, &e->frame, &h, &e->winner, &e->s0, &e->s1, &e->stage, &e->pool) != 8) return 0;
    e->hash = h;
    return e->winner >= -1 && e->winner <= 1 && e->s0 >= -1 && e->s1 >= -1;
}
/* NULL when the two sides saw the same end; else the first field that differs (for the log and the refusal). A zero hash means "not final here": it
 * is not compared (the other fields still are). */
static inline const char *gw_nd_end_diff(const GwNdEnd *a, const GwNdEnd *b) {
    if (a->epoch != b->epoch) return "epoch";
    if (a->stage != b->stage) return "stage";
    if (a->frame != b->frame) return "exit frame";
    if (a->winner != b->winner) return "winner";
    if (a->s0 != b->s0 || a->s1 != b->s1) return "stocks";
    if (a->pool != b->pool) return "shared stocks";
    if (a->hash != 0 && b->hash != 0 && a->hash != b->hash) return "final hash";
    return NULL;
}

/* ---- the Classic plan over the 1P tables -------------------------------------------------------------------------------------------------
 * The retail Classic mode (gmclassic.c) has 11 rows (gmClassic_803DDEC8.x00: a flags byte, a CPU level, a time limit) and five matchup pools of "stage
 * kind, up to three enemy fighters". Retail picks each row's matchup with HSD_Randi and with this machine's unlock bits, which two humans cannot share.
 * The plan is the same SELECTION over the same DATA with the seed and the agreed unlock mask instead: rows in the retail pass order (trio rows, metal,
 * team, then the plain and giant rows), no fighter twice in a run, never one of the two humans' fighters, only fighters in the mask.
 * Bonus rows (target test, platforms, race: flags 0x80) are special stages and are SKIPPED online (owner decision), so the plan has 8 stages:
 * fight, team of two, fight, metal, fight, trio, giant, boss. The pools' stage kinds are kept in the plan (table_stage) for a later stage that can load
 * them; the stage actually played is chosen from the agreed stage list by the seed. The boss row is data only until CPUs are online (stage 6). */
typedef struct { int stage_kind; int ck[3]; } GwNdMatchup;
/* the 1P matchup pools of src/melee/gm/gmclassic.c (gmClassic_803DDEC8: x0CC plain, x1B8 team, x26C metal, x2B0 trio, x0C0 boss), terminators dropped */
static const GwNdMatchup gw_nd_boss[] = {
    { 176, { 30, 26, 33 } },
};
static const GwNdMatchup gw_nd_normal[] = {
    { 86, { 8, 33, 33 } }, { 87, { 8, 33, 33 } }, { 88, { 1, 33, 33 } }, { 89, { 1, 33, 33 } }, { 90, { 6, 33, 33 } }, { 91, { 6, 33, 33 } },
    { 92, { 16, 33, 33 } }, { 93, { 16, 33, 33 } }, { 94, { 17, 33, 33 } }, { 95, { 17, 33, 33 } }, { 96, { 4, 33, 33 } }, { 97, { 4, 33, 33 } },
    { 98, { 2, 33, 33 } }, { 99, { 2, 33, 33 } }, { 100, { 13, 33, 33 } }, { 101, { 7, 33, 33 } }, { 102, { 7, 33, 33 } }, { 103, { 0, 33, 33 } },
    { 104, { 0, 33, 33 } }, { 105, { 11, 33, 33 } }, { 106, { 11, 33, 33 } }, { 107, { 15, 33, 33 } }, { 108, { 5, 33, 33 } }, { 109, { 5, 33, 33 } },
    { 110, { 12, 33, 33 } }, { 111, { 12, 33, 33 } }, { 112, { 18, 33, 33 } }, { 113, { 9, 33, 33 } }, { 114, { 10, 33, 33 } },
    { 115, { 10, 33, 33 } }, { 116, { 14, 33, 33 } }, { 117, { 14, 33, 33 } }, { 118, { 22, 33, 33 } }, { 119, { 21, 33, 33 } },
    { 120, { 21, 33, 33 } }, { 121, { 20, 33, 33 } }, { 122, { 20, 33, 33 } }, { 124, { 24, 33, 33 } },
};
static const GwNdMatchup gw_nd_team[] = {
    { 125, { 8, 5, 33 } }, { 126, { 8, 12, 33 } }, { 127, { 1, 2, 33 } }, { 128, { 6, 18, 33 } }, { 129, { 6, 21, 33 } }, { 130, { 6, 7, 33 } },
    { 131, { 9, 6, 33 } }, { 132, { 16, 0, 33 } }, { 133, { 16, 2, 33 } }, { 134, { 17, 7, 33 } }, { 135, { 17, 11, 33 } }, { 136, { 4, 13, 33 } },
    { 137, { 4, 24, 33 } }, { 138, { 4, 15, 33 } }, { 139, { 4, 14, 33 } }, { 140, { 2, 20, 33 } }, { 141, { 2, 0, 33 } }, { 142, { 13, 24, 33 } },
    { 143, { 13, 15, 33 } }, { 144, { 7, 22, 33 } }, { 145, { 11, 12, 33 } }, { 146, { 11, 10, 33 } }, { 147, { 0, 20, 33 } }, { 148, { 5, 10, 33 } },
    { 149, { 5, 12, 33 } }, { 150, { 5, 18, 33 } }, { 151, { 12, 18, 33 } }, { 152, { 18, 21, 33 } }, { 153, { 18, 9, 33 } },
};
static const GwNdMatchup gw_nd_metal[] = {
    { 155, { 8, 33, 33 } }, { 156, { 1, 33, 33 } }, { 157, { 6, 33, 33 } }, { 158, { 17, 33, 33 } }, { 159, { 7, 33, 33 } }, { 160, { 0, 33, 33 } },
    { 161, { 15, 33, 33 } }, { 162, { 5, 33, 33 } }, { 163, { 22, 33, 33 } }, { 164, { 21, 33, 33 } },
};
static const GwNdMatchup gw_nd_ally[] = {
    { 165, { 8, 8, 8 } }, { 166, { 1, 1, 1 } }, { 167, { 4, 4, 4 } }, { 168, { 7, 7, 7 } }, { 169, { 11, 11, 11 } }, { 170, { 15, 15, 15 } },
    { 174, { 0, 0, 0 } }, { 172, { 24, 24, 24 } }, { 173, { 3, 3, 3 } },
};

#define GW_ND_ROW_BOSS 0x20
/* the retail row list (gmClassic_803DDEC8.x00), battle rows only: row index in the retail list, flags, CPU level, time limit in seconds */
typedef struct { int row, flags, level, seconds; } GwNdRow;
static const GwNdRow gw_nd_rows[] = {
    { 0, 0x00, 2, 300 }, { 1, 0x10, 4, 300 }, { 3, 0x00, 2, 300 }, { 4, 0x02, 4, 300 },
    { 6, 0x00, 2, 300 }, { 7, 0x08, 4, 300 }, { 9, 0x04, 2, 300 }, { 10, 0x20, 2, 300 },
};
#define GW_ND_CLASSIC_STAGES 8

typedef struct {
    int row, flags, level, seconds;
    int table_stage;   /* the matchup's stage kind in the 1P tables (data only) */
    int n_enemy;
    int ck[3];
    int relaxed;       /* a rule had to be relaxed to fill this row (the mask was too small) */
} GwNdStage;
typedef struct {
    int n;
    GwNdStage st[GW_ND_CLASSIC_STAGES];
    char digest[17];
} GwNdPlan;

static inline const char *gw_nd_flags_name(int flags) {
    return (flags & GW_ND_ROW_BOSS) ? "boss" : (flags & 0x10) ? "team of two" : (flags & 0x08) ? "trio" : (flags & 0x04) ? "giant" : (flags & 0x02) ? "metal" : "fight";
}

/* a seeded permutation of 0..n-1 (Fisher-Yates over gw_nr_mix), `salt` names the pool */
static inline void gw_nd_perm(uint32_t seed, uint32_t salt, int n, int *p) {
    int i;
    for (i = 0; i < n; ++i) p[i] = i;
    for (i = n - 1; i > 0; --i) {
        int j = (int) (gw_nr_mix(seed + salt * 40503u, (uint32_t) i) % (uint32_t) (i + 1)), t = p[i];
        p[i] = p[j];
        p[j] = t;
    }
}

static inline int gw_nd_mask_has(uint32_t mask, int ck) { return ck >= 0 && ck < 32 && ((mask >> ck) & 1u) != 0; }

/* Pick the matchup of one row from `pool` (n entries). Rules, strongest first: every enemy in the mask; none of the two humans' fighters; none already
 * used by an earlier row. If nothing fits, the rules are relaxed in the reverse order (used, then humans, then the mask) so a row is always filled. */
static inline const GwNdMatchup *gw_nd_pick(const GwNdMatchup *pool, int n, const int *perm, uint32_t mask, int h0, int h1, uint32_t used, int *relaxed) {
    int level, i, k;
    for (level = 0; level < 4; ++level) {
        for (i = 0; i < n; ++i) {
            const GwNdMatchup *m = &pool[perm[i]];
            int ok = 1;
            for (k = 0; k < 3 && ok; ++k) {
                int c = m->ck[k];
                if (c == GW_ND_ENEMY_NONE) continue;
                if (level < 3 && !gw_nd_mask_has(mask, c)) ok = 0;
                if (level < 2 && (c == h0 || c == h1)) ok = 0;
                if (level < 1 && gw_nd_mask_has(used, c)) ok = 0;
            }
            if (ok) { *relaxed = level; return m; }
        }
    }
    *relaxed = 4;
    return &pool[perm[0]];
}

/* The whole plan (a pure function of the seed, the loop, the agreed mask and the two humans' fighters). Returns the number of stages. */
static inline int gw_nd_classic_plan(uint32_t seed, int loop, uint32_t mask, int h0, int h1, GwNdPlan *out) {
    static const int pass_of[4] = { 0x08, 0x02, 0x10, 0 }; /* the retail pass order: trio rows, metal rows, team rows, then plain and giant rows */
    int perm[48], pass, r, i, k, o = 0;
    uint32_t used = 0, s = seed + (uint32_t) loop * 40503u;
    char text[400];
    memset(out, 0, sizeof *out);
    out->n = GW_ND_CLASSIC_STAGES;
    for (r = 0; r < GW_ND_CLASSIC_STAGES; ++r) {
        out->st[r].row = gw_nd_rows[r].row;
        out->st[r].flags = gw_nd_rows[r].flags;
        out->st[r].level = gw_nd_rows[r].level;
        out->st[r].seconds = gw_nd_rows[r].seconds;
    }
    for (pass = 0; pass < 4; ++pass) {
        const GwNdMatchup *pool = pass == 0 ? gw_nd_ally : pass == 1 ? gw_nd_metal : pass == 2 ? gw_nd_team : gw_nd_normal;
        int n = pass == 0 ? (int) (sizeof gw_nd_ally / sizeof gw_nd_ally[0]) : pass == 1 ? (int) (sizeof gw_nd_metal / sizeof gw_nd_metal[0])
              : pass == 2 ? (int) (sizeof gw_nd_team / sizeof gw_nd_team[0]) : (int) (sizeof gw_nd_normal / sizeof gw_nd_normal[0]);
        gw_nd_perm(s, (uint32_t) pass + 1u, n, perm);
        for (r = 0; r < GW_ND_CLASSIC_STAGES; ++r) {
            GwNdStage *g = &out->st[r];
            const GwNdMatchup *m;
            int relaxed = 0, f = g->flags;
            if (f & GW_ND_ROW_BOSS) continue;
            if (pass < 3 ? !(f & pass_of[pass]) : !(f == 0 || f == 4)) continue;
            m = gw_nd_pick(pool, n, perm, mask, h0, h1, used, &relaxed);
            g->table_stage = m->stage_kind;
            for (k = 0; k < 3; ++k) {
                g->ck[k] = m->ck[k];
                if (m->ck[k] != GW_ND_ENEMY_NONE) {
                    g->n_enemy++;
                    if (m->ck[k] >= 0 && m->ck[k] < 32) used |= 1u << m->ck[k];
                }
            }
            g->relaxed = relaxed;
        }
    }
    for (r = 0; r < GW_ND_CLASSIC_STAGES; ++r) { /* the boss row: the fixed matchup (Master Hand and Crazy Hand) */
        GwNdStage *g = &out->st[r];
        if (!(g->flags & GW_ND_ROW_BOSS)) continue;
        g->table_stage = gw_nd_boss[0].stage_kind;
        for (k = 0; k < 3; ++k) {
            g->ck[k] = gw_nd_boss[0].ck[k];
            if (g->ck[k] != GW_ND_ENEMY_NONE) g->n_enemy++;
        }
    }
    o = snprintf(text, sizeof text, "CL1|%u|%d|%08x|%d|%d", (unsigned) seed, loop, (unsigned) mask, h0, h1);
    for (i = 0; i < out->n && o < (int) sizeof text - 40; ++i) {
        const GwNdStage *g = &out->st[i];
        o += snprintf(text + o, sizeof text - (size_t) o, "|%d.%x.%d.%d.%d.%d.%d.%d", g->row, g->flags, g->level, g->seconds, g->table_stage, g->ck[0], g->ck[1], g->ck[2]);
    }
    {
        uint32_t hi, lo;
        gw_mb_digest64(text, (size_t) o, "ndplan:", &hi, &lo);
        snprintf(out->digest, sizeof out->digest, "%08x%08x", (unsigned) hi, (unsigned) lo);
    }
    return out->n;
}

#endif
