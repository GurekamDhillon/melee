/* gw_netrun.h - the RUN RECORD of an online stage run (Classic / Adventure online, stage 7 of
 * _research/envoy-netplay-scoping-2026-10-05.md; brief docs/superpowers/plans/2026-10-08-envoy-online-stage7.md).
 *
 * A run is a sequence of stages, each a FRESH agreed rollback session (a rollback window never crosses a scene change). What the two peers
 * carry from one stage to the next is this record: a short ASCII line with a 64-bit digest, in the same family as the Envoy build record
 * (gw_matchbuild.h, EB1):
 *
 *   RN1|<seed>|<stage>|<loop>|<score0>|<score1>|<flags>|<ext>|<digest 16 hex>
 *
 * seed   the run seed (1..2147483646), the HOST's choice, sent once per room; every roll of the run is a pure function of it
 * stage  zero-based index of the stage about to be played; loop is the New Game+ pass
 * score* what the two players carry (the spike: games won; later: lives, coins, whatever the run director keeps)
 * flags  run director bits (0 in the spike)
 * ext    the external stage id the plan chose for this stage (so the digest covers WHICH stage, not only its number)
 * digest two salted FNV-1a words over everything before the last '|' (gw_mb_digest64, salt "netrun:")
 *
 * Plain static functions over integers only: the same on Windows and Linux, compiles anywhere, tested headless (netplay_run_* in gw_netplay.c).
 * The stage plan (gw_nr_stage_index) is a pure function of (seed, stage, n): two peers that agree on the seed and on the stage list agree on
 * the stage, and no two consecutive stages are the same.
 */
#ifndef GW_NETRUN_H
#define GW_NETRUN_H

#include "gw_matchbuild.h" /* gw_mb_digest64 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#define GW_NR_RECORD_MAX 112

typedef struct {
    long seed, stage, loop, score0, score1, flags, ext;
    char digest[17];
} GwNrRecord;

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

/* Writes the record with its digest into out (cap >= GW_NR_RECORD_MAX); returns its length or 0. */
static inline int gw_nr_format(char *out, size_t cap, long seed, long stage, long loop, long s0, long s1, long flags, long ext) {
    char body[GW_NR_RECORD_MAX];
    uint32_t hi, lo;
    int n = snprintf(body, sizeof body, "RN1|%ld|%ld|%ld|%ld|%ld|%ld|%ld", seed, stage, loop, s0, s1, flags, ext);
    if (n <= 0 || n >= (int) sizeof body) return 0;
    gw_mb_digest64(body, (size_t) n, "netrun:", &hi, &lo);
    n = snprintf(out, cap, "%s|%08x%08x", body, (unsigned) hi, (unsigned) lo);
    return n > 0 && (size_t) n < cap ? n : 0;
}

static inline int gw_nr_num(const char *s, size_t n, long *v) {
    size_t i;
    long x = 0;
    if (n == 0 || n > 10 || (n > 1 && s[0] == '0')) return 0;
    for (i = 0; i < n; ++i) {
        if (s[i] < '0' || s[i] > '9') return 0;
        x = x * 10 + (s[i] - '0');
    }
    if (x > 2147483647L) return 0;
    *v = x;
    return 1;
}

/* Strict parse: form, canonical integers, digest. Returns 1 and fills out, or 0 with a reason in why. */
static inline int gw_nr_parse(const char *text, GwNrRecord *out, char *why, size_t cap) {
    const char *f[9];
    size_t fl[9], len, i;
    int nf = 0, k;
    uint32_t hi, lo;
    char want[17];
    long v[7];
#define NR_FAIL(msg) do { if (why != NULL && cap > 0) snprintf(why, cap, "%s", msg); return 0; } while (0)
    if (text == NULL) NR_FAIL("no record");
    len = strlen(text);
    if (len == 0 || len >= GW_NR_RECORD_MAX) NR_FAIL("record is not compact ASCII");
    for (i = 0; i < len; ++i) if ((uint8_t) text[i] <= 32 || (uint8_t) text[i] >= 127) NR_FAIL("record is not compact ASCII");
    f[0] = text;
    for (i = 0; i <= len; ++i) {
        if (i == len || text[i] == '|') {
            if (nf >= 9) NR_FAIL("record has too many fields");
            fl[nf] = (size_t) (text + i - f[nf]);
            ++nf;
            if (i < len) f[nf < 9 ? nf : 8] = text + i + 1;
        }
    }
    if (nf != 9) NR_FAIL("record does not have 9 fields");
    if (fl[0] != 3 || memcmp(f[0], "RN1", 3) != 0) NR_FAIL("record version is not supported (this build reads RN1)");
    if (fl[8] != 16) NR_FAIL("bad digest");
    for (k = 0; k < 16; ++k) if (!gw_mb_ishex(f[8][k])) NR_FAIL("bad digest");
    gw_mb_digest64(text, (size_t) (f[8] - 1 - text), "netrun:", &hi, &lo);
    snprintf(want, sizeof want, "%08x%08x", (unsigned) hi, (unsigned) lo);
    if (memcmp(want, f[8], 16) != 0) NR_FAIL("record digest does not match its contents");
    for (k = 1; k <= 7; ++k) if (!gw_nr_num(f[k], fl[k], &v[k - 1])) NR_FAIL("bad number in the record");
    out->seed = v[0]; out->stage = v[1]; out->loop = v[2]; out->score0 = v[3]; out->score1 = v[4]; out->flags = v[5]; out->ext = v[6];
    memcpy(out->digest, f[8], 16);
    out->digest[16] = 0;
    return 1;
#undef NR_FAIL
}

/* The digest field of a record text (16 hex chars after the last '|'), or NULL. */
static inline const char *gw_nr_digest_of(const char *text) {
    const char *p = text != NULL ? strrchr(text, '|') : NULL;
    return p != NULL && strlen(p + 1) == 16 ? p + 1 : NULL;
}

#endif
