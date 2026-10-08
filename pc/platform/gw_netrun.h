/* gw_netrun.h - the RUN RECORD of an online run: ONE format for the stage run (Classic / Adventure online, stage 7 of
 * _research/envoy-netplay-scoping-2026-10-05.md) and for the Envoy set, resume and abandon (stage 5). Plain static functions over integers only
 * (no floats, no locale, fixed-width values where it matters), so the text is byte-identical on Windows and Linux; compiles anywhere, tested headless.
 *
 *   RN2|<seed>|<stage>|<loop>|<score0>|<score1>|<flags>|<ext>|<mode 8 hex>|<round>|<winner 0|1|n>|<picks>|<stages>|<bw 8 hex>|<x 16 hex|->|<stocks>|<continues>|<lost>|<digest 16 hex>
 *
 * The first seven fields are the stage-7 spike's RN1 record, unchanged and in the same order (RN2 only appends):
 *   seed    the run seed (1..2147483646), the HOST's choice; every roll of the run is a pure function of it
 *   stage   zero-based index of the stage / game about to be played or being played (the DEPTH of the run); loop is the New Game+ pass
 *   score*  what the two players carry (games won)
 *   flags   run director bits (0 for now)
 *   ext     the external stage id the plan chose for this stage (a stage run); 0 when the lobby's pick/ban chooses (an Envoy set)
 * and the stage-5 extension (what resuming an Envoy set needs; every field is its cause, not a copy):
 *   mode    the room's Envoy mode word (gw_matchbuild.h: V1 Versus set, COOP co-op run); 0 for a run that has none
 *   round   the last game whose reward resolved (0 = none)
 *   winner  of the last game (n = none)
 *   picks   2 chars per game from game 2 to `round` ('0'..'3' per player: host, guest; '-' unknown); '-' when round < 2. With the seed they regenerate
 *           both players' BAGS and KEYSTONES (mod_progression.set_build)
 *   stages  6 hex per game started: the low 24 bits of the FNV-1a of that stage's content identity ("ffffff" unknown); '-' when none started
 *   bw      the build word both peers verified at the start of the latest game (0 before the first)
 *   stocks  shared stock pool (0..198); Versus sets use 0
 *   continues one run-level token, 0 or 1; lost is 0 or 1 and a lost stage has zero stocks
 *   x       an optional digest a script supplies (the co-op director's record digest); '-' none
 *   digest  two salted FNV-1a words (gw_mb_digest64, salt "netrun:") over everything before the last '|'
 * The record's STATE (active, interrupted, abandoned, continued) and the local seat are not in it: they belong to one side.
 *
 * The stage plan (gw_nr_stage_index) is a pure function of (seed, stage, n): two peers that agree on the seed and on the stage list agree on the stage.
 */
#ifndef GW_NETRUN_H
#define GW_NETRUN_H

#include "gw_matchbuild.h" /* gw_mb_digest64 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define GW_NR_RECORD_MAX 144   /* a record with no extension content (what gw_nr_format writes) always fits */
#define GW_NR_TEXT_MAX 480     /* the longest record (31 games); a settings value holds 511 */
#define GW_NR_MAXGAMES 31      /* games the extension names (the lobby remembers 32 slots, index 0 unused) */
#define GW_NR_CHUNK 120        /* record bytes per lobby chunk message */
#define GW_NR_NOSTAGE 0xFFFFFFu

enum { GW_NR_NONE = 0, GW_NR_ACTIVE = 1, GW_NR_INTERRUPTED = 2, GW_NR_ABANDONED = 3, GW_NR_CONTINUED = 4 };
static inline int gw_nr_resumable(int state) { return state == GW_NR_ACTIVE || state == GW_NR_INTERRUPTED; }
static inline const char *gw_nr_state_name(int s) {
    return s == GW_NR_ACTIVE ? "active" : s == GW_NR_INTERRUPTED ? "interrupted" : s == GW_NR_ABANDONED ? "abandoned" : s == GW_NR_CONTINUED ? "continued" : "none";
}

/* `game` (= stage + 1) and `score[]` are the working fields of the Envoy lobby code; `stage`, `score0`, `score1` are the RN1 names the stage-run code reads.
 * gw_nr_encode writes the text from game/score; gw_nr_parse and gw_nr_format fill both spellings. */
typedef struct {
    long seed, stage, loop, score0, score1, flags, ext;
    char digest[17];
    uint32_t mode;
    int game;
    int round;
    int score[2];
    int winner;                                /* -1 none, 0 host, 1 guest */
    signed char pick[GW_NR_MAXGAMES + 1][2];   /* index = game; -1 none */
    uint32_t gstage[GW_NR_MAXGAMES + 1];       /* index = game (1..started); GW_NR_NOSTAGE unknown */
    int started;                               /* games started */
    uint32_t bw;
    int stocks, continues, lost;                /* shared pair stock pool; 0/1 token; lost stage ends both seats */
    char x[17];                                /* "" none */
} GwNrRecord;

static inline void gw_nr_clear(GwNrRecord *r) {
    int g;
    memset(r, 0, sizeof *r);
    r->continues = 1;
    r->winner = -1;
    r->game = 1;
    for (g = 0; g <= GW_NR_MAXGAMES; ++g) { r->pick[g][0] = r->pick[g][1] = -1; r->gstage[g] = GW_NR_NOSTAGE; }
}

static inline uint32_t gw_nr_mix(uint32_t seed, uint32_t stage) {
    uint32_t h = 2166136261u;
    int i;
    for (i = 0; i < 4; ++i) h = (h ^ ((seed >> (8 * i)) & 0xFFu)) * 16777619u;
    for (i = 0; i < 4; ++i) h = (h ^ ((stage >> (8 * i)) & 0xFFu)) * 16777619u;
    h ^= h >> 15;
    h *= 2246822519u;
    h ^= h >> 13;
    return h;
}

/* The index (0..n-1) of the stage the plan chooses for run stage `stage` out of a list of n. n <= 1 is always 0; otherwise a stage never repeats its predecessor. */
static inline int gw_nr_stage_index(uint32_t seed, int stage, int n) {
    int s, idx = 0;
    if (n <= 1 || stage < 0) return 0;
    idx = (int) (gw_nr_mix(seed, 0) % (uint32_t) n);
    for (s = 1; s <= stage; ++s) {
        int r = (int) (gw_nr_mix(seed, (uint32_t) s) % (uint32_t) (n - 1));
        idx = r >= idx ? r + 1 : r;
    }
    return idx;
}

/* A stage's identity word from its content-identity token. */
static inline uint32_t gw_nr_stage_word(const char *token) {
    if (token == NULL || token[0] == '\0') return GW_NR_NOSTAGE;
    return gw_mb_fnv32(token, strlen(token), 2166136261u) & 0xFFFFFFu;
}

static inline void gw_nr_digest_text(const char *body, size_t n, char out[17]) {
    uint32_t hi, lo;
    gw_mb_digest64(body, n, "netrun:", &hi, &lo);
    snprintf(out, 17, "%08x%08x", (unsigned) hi, (unsigned) lo);
}

/* Encode `r` from its working fields (fills r->stage, score0/1 and r->digest). Returns the length, 0 when it does not fit or is not valid. */
static inline int gw_nr_encode(GwNrRecord *r, char *out, size_t cap) {
    char picks[2 * GW_NR_MAXGAMES + 2], stages[6 * GW_NR_MAXGAMES + 2], x[18];
    int g, n, o = 0;
    if (r->game < 1 || r->round < 0 || r->round > r->game || r->started < 0 || r->started > r->game) return 0;
    if (r->round > GW_NR_MAXGAMES || r->started > GW_NR_MAXGAMES) return 0;
    if (r->seed <= 0 || r->seed > 2147483646L || r->loop < 0 || r->loop > 99999 || r->winner < -1 || r->winner > 1 || r->score[0] < 0 || r->score[1] < 0 || r->score[0] > 99 || r->score[1] > 99) return 0;
    if (r->stocks < 0 || r->stocks > 198 || r->continues < 0 || r->continues > 1 || r->lost < 0 || r->lost > 1 || (r->lost && r->stocks != 0)) return 0;
    if (r->flags < 0 || r->ext < 0 || r->game > 99999) return 0;
    if (r->round >= 2) {
        for (g = 2; g <= r->round; ++g) {
            int a = r->pick[g][0], b = r->pick[g][1];
            picks[o++] = a < 0 || a > 3 ? '-' : (char) ('0' + a);
            picks[o++] = b < 0 || b > 3 ? '-' : (char) ('0' + b);
        }
        picks[o] = '\0';
    } else { picks[0] = '-'; picks[1] = '\0'; }
    o = 0;
    if (r->started >= 1) {
        for (g = 1; g <= r->started; ++g) o += snprintf(stages + o, sizeof stages - (size_t) o, "%06x", (unsigned) (r->gstage[g] & 0xFFFFFFu));
        stages[o] = '\0';
    } else { stages[0] = '-'; stages[1] = '\0'; }
    snprintf(x, sizeof x, "%s", r->x[0] != '\0' ? r->x : "-");
    r->stage = r->game - 1;
    r->score0 = r->score[0];
    r->score1 = r->score[1];
    n = snprintf(out, cap, "RN2|%ld|%ld|%ld|%d|%d|%ld|%ld|%08x|%d|%c|%s|%s|%08x|%s|%d|%d|%d", r->seed, r->stage, r->loop, r->score[0], r->score[1], r->flags, r->ext, (unsigned) r->mode, r->round,
                 r->winner < 0 ? 'n' : (char) ('0' + r->winner), picks, stages, (unsigned) r->bw, x, r->stocks, r->continues, r->lost);
    if (n <= 0 || (size_t) n + 18 > cap || n + 17 > GW_NR_TEXT_MAX) return 0;
    gw_nr_digest_text(out, (size_t) n, r->digest);
    snprintf(out + n, cap - (size_t) n, "|%s", r->digest);
    return n + 17;
}

/* Writes the record with its digest into out (cap >= GW_NR_RECORD_MAX); returns its length or 0. The stage-run spike's call: no extension content. */
static inline int gw_nr_format(char *out, size_t cap, long seed, long stage, long loop, long s0, long s1, long flags, long ext) {
    GwNrRecord r;
    gw_nr_clear(&r);
    r.seed = seed; r.game = (int) stage + 1; r.loop = loop; r.score[0] = (int) s0; r.score[1] = (int) s1; r.flags = flags; r.ext = ext;
    if (stage < 0 || stage > 99998) return 0;
    return gw_nr_encode(&r, out, cap);
}

static inline int gw_nr_hexn(const char *s, size_t n, uint32_t *v) {
    size_t i;
    uint32_t a = 0;
    for (i = 0; i < n; ++i) {
        int c = (uint8_t) s[i];
        a <<= 4;
        if (c >= '0' && c <= '9') a |= (uint32_t) (c - '0');
        else if (c >= 'a' && c <= 'f') a |= (uint32_t) (c - 'a' + 10);
        else return 0;
    }
    *v = a;
    return 1;
}

static inline int gw_nr_num(const char *s, size_t n, long *v) {
    size_t i;
    uint32_t x = 0;
    if (n == 0 || n > 10 || (n > 1 && s[0] == '0')) return 0;
    for (i = 0; i < n; ++i) {
        if (s[i] < '0' || s[i] > '9') return 0;
        if (x > (2147483647u - (unsigned) (s[i] - '0')) / 10u) return 0;
        x = x * 10u + (unsigned) (s[i] - '0');
    }
    if (x > 2147483647L) return 0;
    *v = x;
    return 1;
}

/* Strict parse: form, canonical integers, digest. Returns 1 and fills out, or 0 with a reason in why. */
static inline int gw_nr_parse(const char *text, GwNrRecord *out, char *why, size_t cap) {
    const char *f[19];
    size_t fl[19], len, i;
    int nf = 0, k, g, w;
    long v[8];
    uint32_t hv;
    char want[17];
#define NR_FAIL(msg) do { if (why != NULL && cap > 0) snprintf(why, cap, "%s", msg); return 0; } while (0)
    if (text == NULL) NR_FAIL("no record");
    len = strlen(text);
    if (len == 0 || len > GW_NR_TEXT_MAX) NR_FAIL("record is not compact ASCII");
    for (i = 0; i < len; ++i) if ((uint8_t) text[i] <= 32 || (uint8_t) text[i] >= 127) NR_FAIL("record is not compact ASCII");
    f[0] = text;
    for (i = 0; i <= len; ++i) {
        if (i == len || text[i] == '|') {
            if (nf >= 19) NR_FAIL("record has too many fields");
            fl[nf] = (size_t) (text + i - f[nf]);
            ++nf;
            if (i < len && nf < 19) f[nf] = text + i + 1;
        }
    }
    if (nf != 19) NR_FAIL("record does not have 19 fields");
    if (fl[0] != 3 || memcmp(f[0], "RN2", 3) != 0) NR_FAIL("record version is not supported (this build reads RN2)");
    if (fl[18] != 16) NR_FAIL("bad digest");
    for (k = 0; k < 16; ++k) if (!gw_mb_ishex(f[18][k])) NR_FAIL("bad digest");
    gw_nr_digest_text(text, (size_t) (f[18] - 1 - text), want);
    if (memcmp(want, f[18], 16) != 0) NR_FAIL("record digest does not match its contents");
    for (k = 1; k <= 7; ++k) if (!gw_nr_num(f[k], fl[k], &v[k - 1])) NR_FAIL("bad number in the record");
    gw_nr_clear(out);
    out->seed = v[0]; out->stage = v[1]; out->loop = v[2]; out->score0 = v[3]; out->score1 = v[4]; out->flags = v[5]; out->ext = v[6];
    if (out->seed < 1 || out->seed > 2147483646L || out->loop > 99999 || out->score0 > 99 || out->score1 > 99 || out->stage > 99998) NR_FAIL("a field is out of range");
    out->game = (int) out->stage + 1;
    out->score[0] = (int) out->score0;
    out->score[1] = (int) out->score1;
    memcpy(out->digest, f[18], 16); out->digest[16] = '\0';
    if (fl[8] != 8 || !gw_nr_hexn(f[8], 8, &hv)) NR_FAIL("bad mode word");
    out->mode = hv;
    if (!gw_nr_num(f[9], fl[9], &v[7]) || v[7] > out->game || v[7] > GW_NR_MAXGAMES) NR_FAIL("bad round");
    out->round = (int) v[7];
    if (fl[10] != 1 || !(f[10][0] == 'n' || f[10][0] == '0' || f[10][0] == '1')) NR_FAIL("bad winner");
    out->winner = f[10][0] == 'n' ? -1 : f[10][0] - '0';
    if (out->round >= 2) {
        if (fl[11] != (size_t) (2 * (out->round - 1))) NR_FAIL("picks do not match the round");
        for (g = 2, k = 0; g <= out->round; ++g) {
            for (w = 0; w < 2; ++w, ++k) {
                char c = f[11][k];
                if (c == '-') out->pick[g][w] = -1;
                else if (c >= '0' && c <= '3') out->pick[g][w] = (signed char) (c - '0');
                else NR_FAIL("bad pick");
            }
        }
    } else if (fl[11] != 1 || f[11][0] != '-') NR_FAIL("picks must be '-' before round 2");
    if (fl[12] == 1 && f[12][0] == '-') out->started = 0;
    else {
        if (fl[12] == 0 || fl[12] % 6 != 0) NR_FAIL("bad stages");
        out->started = (int) (fl[12] / 6);
        if (out->started < 1 || out->started > out->game || out->started > GW_NR_MAXGAMES) NR_FAIL("stages do not match the game");
        for (g = 1; g <= out->started; ++g) {
            if (!gw_nr_hexn(f[12] + (g - 1) * 6, 6, &hv)) NR_FAIL("bad stage");
            out->gstage[g] = hv;
        }
    }
    if (fl[13] != 8 || !gw_nr_hexn(f[13], 8, &hv)) NR_FAIL("bad build word");
    out->bw = hv;
    if (fl[14] == 1 && f[14][0] == '-') out->x[0] = '\0';
    else {
        if (fl[14] != 16) NR_FAIL("bad x digest");
        for (i = 0; i < 16; ++i) if (!gw_mb_ishex(f[14][i])) NR_FAIL("bad x digest");
        memcpy(out->x, f[14], 16); out->x[16] = '\0';
    }
    if (!gw_nr_num(f[15], fl[15], &v[7]) || v[7] > 198) NR_FAIL("bad shared stocks");
    out->stocks = (int) v[7];
    if (!gw_nr_num(f[16], fl[16], &v[7]) || v[7] > 1) NR_FAIL("bad continue token");
    out->continues = (int) v[7];
    if (!gw_nr_num(f[17], fl[17], &v[7]) || v[7] > 1 || (v[7] && out->stocks != 0)) NR_FAIL("bad lost stage");
    out->lost = (int) v[7];
    return 1;
#undef NR_FAIL
}

/* The digest field of a record text (16 hex chars after the last '|'), or NULL. */
static inline const char *gw_nr_digest_of(const char *text) {
    const char *p = text != NULL ? strrchr(text, '|') : NULL;
    return p != NULL && strlen(p + 1) == 16 ? p + 1 : NULL;
}

/* progress key: (game, round, started) is monotone through a run's lifetime */
static inline int gw_nr_progress_cmp(const GwNrRecord *a, const GwNrRecord *b) {
    if (a->game != b->game) return a->game < b->game ? -1 : 1;
    if (a->round != b->round) return a->round < b->round ? -1 : 1;
    if (a->started != b->started) return a->started < b->started ? -1 : 1;
    return 0;
}

/* Compare two records of (supposedly) one run. 0 equal, 1 `a` is the same run further on, -1 `b` is, 2 not the same run / contradict. */
static inline int gw_nr_compare(const GwNrRecord *a, const GwNrRecord *b) {
    int g, n, c;
    if (strcmp(a->digest, b->digest) == 0) return 0;
    if (a->mode != b->mode || a->seed != b->seed || a->loop != b->loop) return 2;
    n = a->round < b->round ? a->round : b->round;
    for (g = 2; g <= n; ++g) if (a->pick[g][0] != b->pick[g][0] || a->pick[g][1] != b->pick[g][1]) return 2;
    n = a->started < b->started ? a->started : b->started;
    for (g = 1; g <= n; ++g) if (a->gstage[g] != b->gstage[g]) return 2;
    c = gw_nr_progress_cmp(a, b);
    if (c == 0) return 2; /* the same boundary with different content (build word, x, score) */
    { const GwNrRecord *lo = c > 0 ? b : a, *hi = c > 0 ? a : b;
      if (hi->continues > lo->continues || lo->lost) return 2; /* a loss requires an explicit agreed continue, never automatic prefix adoption */
    }
    if (a->game == b->game && (a->score[0] != b->score[0] || a->score[1] != b->score[1] || a->winner != b->winner)) return 2;
    if (a->game != b->game) { /* scores only grow */
        const GwNrRecord *lo = c > 0 ? b : a, *hi = c > 0 ? a : b;
        if (hi->score[0] < lo->score[0] || hi->score[1] < lo->score[1]) return 2;
        if (hi->score[0] + hi->score[1] - lo->score[0] - lo->score[1] > hi->game - lo->game) return 2;
    }
    return c;
}

/* Stage-7 director helpers: a loss ends the pair; spending the single token needs both seats' agreement (bit mask 3).
 * A disconnect is NOT a stage loss: resume replays the last started stage without spending stocks or the token. */
static inline void gw_nr_lose_stage(GwNrRecord *r) { r->stocks = 0; r->lost = 1; }
static inline int gw_nr_continue(GwNrRecord *r, unsigned agreed, int stocks) {
    if (agreed != 3 || !r->lost || r->continues != 1 || stocks < 1 || stocks > 198) return 0;
    r->stocks = stocks; r->continues = 0; r->lost = 0; return 1;
}

#endif
