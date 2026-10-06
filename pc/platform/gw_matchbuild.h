/* gw_matchbuild.h - the Envoy BUILD RECORD shared by the script layer's staging call, the netplay lobby/handshake and the native tests.
 * Plain static functions so it compiles anywhere (the game side does not need it; the native side and the test runner do).
 *
 * A record is one line of compact ASCII (the Lua writer is mod_codec.lua build_record; this file is its strict native reader):
 *   EB1|<seed>|<game>|<loop>|<port>|<pool digest 16 hex>|<id>=<tier>,...|<key>=<number>,...|<keystone ids>|<digest 16 hex>
 * The last field is a 64-bit digest (two salted FNV-1a words) of everything before the last '|'. The native side does not know the pool, so it
 * checks FORM and DIGEST (and the sorted order of every list); that the ids exist in the pool is the Lua reader's job and the pool digest in the
 * record makes two peers with different pools disagree.
 *
 * The word two peers must agree on before a game (gw_MatchBuild_Word in gw_script_netbuild.inc) is a digest over BOTH staged records' digests and
 * the digests of the compiled ops. It travels in the host's scene string as `envoy=<hex8>`; the handshake carries only the Envoy MODE word below.
 */
#ifndef GW_MATCHBUILD_H
#define GW_MATCHBUILD_H

#include <stdint.h>
#include <string.h>
#include <stdio.h>

/* The mode word of the handshake (protocol 5): 0 = no Envoy rules in this match; otherwise it names the rule set the peers implement, so a client with
 * another native table format is refused at the first packet. The per-game build agreement is separate (the scene's envoy= token). */
#define GW_ENVOY_MODE_OFF 0u
#define GW_ENVOY_MODE_V1  0x45560001u /* "EV" + version 1: adversarial set, passive builds */
static inline int gw_envoy_mode_valid(uint32_t w) { return w == GW_ENVOY_MODE_OFF || w == GW_ENVOY_MODE_V1; }

#define GW_MB_RECORD_MAX 400
#define GW_MB_MODS_MAX 24

static inline uint32_t gw_mb_fnv32(const char *s, size_t n, uint32_t h) {
    size_t i;
    for (i = 0; i < n; ++i) h = (h ^ (uint8_t) s[i]) * 16777619u;
    return h;
}
/* the same two words as mod_codec.digest64(text, salt) */
static inline void gw_mb_digest64(const char *text, size_t n, const char *salt, uint32_t *hi, uint32_t *lo) {
    *hi = gw_mb_fnv32(text, n, 2166136261u);
    *lo = gw_mb_fnv32(text, n, gw_mb_fnv32(salt, strlen(salt), 2166136261u));
}

typedef struct {
    long seed, game, loop, port;
    char pool[17];
    char digest[17];
    uint32_t dig_hi, dig_lo;
    int nmods;       /* modifiers listed */
    int nkeystones;
} GwMbRecord;

static inline int gw_mb_isid(char c) { return (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '_'; }
static inline int gw_mb_ishex(char c) { return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'); }

/* Compare two id tokens [a,a+an) and [b,b+bn) byte-wise: <0, 0, >0 (the order Lua's `<` on strings gives for these characters). */
static inline int gw_mb_idcmp(const char *a, size_t an, const char *b, size_t bn) {
    size_t n = an < bn ? an : bn;
    int c = memcmp(a, b, n);
    if (c != 0) return c;
    return an < bn ? -1 : an > bn ? 1 : 0;
}

/* Validates a list "<id><sep><rest>,<id>..." for fields 6 (mods: id=tier), 7 (implicits: key=number) and 8 (keystones: id). `kind`: 0 mods, 1 implicits,
 * 2 ids only. Strictly sorted, no empty items, bounded. Returns the item count or -1. */
static inline int gw_mb_list(const char *s, size_t n, int kind) {
    size_t i = 0;
    int count = 0;
    const char *last = NULL;
    size_t lastn = 0;
    if (n == 0) return 0;
    while (i < n) {
        size_t start = i, idn;
        while (i < n && gw_mb_isid(s[i])) ++i;
        idn = i - start;
        if (idn == 0 || idn > 40) return -1;
        if (kind != 2) {
            size_t vs;
            if (i >= n || s[i] != '=') return -1;
            ++i;
            vs = i;
            while (i < n && s[i] != ',') ++i;
            if (i == vs || i - vs > 30) return -1;
            if (kind == 0) { /* tier text: "2", "2x3" or "t1.2" */
                const char *t = s + vs;
                size_t tn = i - vs, k;
                if (t[0] == 't') {
                    if (tn < 2) return -1;
                    for (k = 1; k < tn; ++k) if (!((t[k] >= '1' && t[k] <= '9') || t[k] == '.')) return -1;
                } else if (tn == 1) {
                    if (t[0] < '1' || t[0] > '9') return -1;
                } else if (tn == 3) {
                    if (t[0] < '1' || t[0] > '9' || t[1] != 'x' || t[2] < '1' || t[2] > '6') return -1;
                } else return -1;
            } else { /* number: digits . - e + */
                size_t k;
                for (k = vs; k < i; ++k) {
                    char c = s[k];
                    if (!((c >= '0' && c <= '9') || c == '.' || c == '-' || c == 'e' || c == '+')) return -1;
                }
            }
        }
        if (last != NULL && gw_mb_idcmp(last, lastn, s + start, idn) >= 0) return -1; /* strictly sorted */
        last = s + start;
        lastn = idn;
        ++count;
        if (count > GW_MB_MODS_MAX) return -1;
        if (i < n) {
            if (s[i] != ',') return -1;
            ++i;
            if (i >= n) return -1; /* trailing comma */
        }
    }
    return count;
}

/* Strict parse of one record. Returns 1 and fills `out`, or 0 with a reason in `why`. */
static inline int gw_mb_record_parse(const char *text, GwMbRecord *out, char *why, size_t cap) {
    const char *f[10], *p;
    size_t fl[10], len, i;
    int nf = 0, k;
    uint32_t hi, lo;
    char want[17];
#define MB_FAIL(msg) do { if (why != NULL && cap > 0) snprintf(why, cap, "%s", msg); return 0; } while (0)
    if (text == NULL) MB_FAIL("no record");
    len = strlen(text);
    if (len == 0 || len > GW_MB_RECORD_MAX) MB_FAIL("record is not compact ASCII");
    for (i = 0; i < len; ++i) if ((uint8_t) text[i] <= 32 || (uint8_t) text[i] >= 127) MB_FAIL("record is not compact ASCII");
    p = text;
    f[0] = p;
    for (i = 0; i <= len; ++i) {
        if (i == len || text[i] == '|') {
            if (nf >= 10) MB_FAIL("record has too many fields");
            fl[nf] = (size_t) (text + i - f[nf]);
            ++nf;
            if (i < len) f[nf < 10 ? nf : 9] = text + i + 1;
        }
    }
    if (nf != 10) MB_FAIL("record does not have 10 fields");
    if (fl[0] != 3 || memcmp(f[0], "EB1", 3) != 0) MB_FAIL("record version is not supported (this build reads EB1)");
    if (fl[9] != 16) MB_FAIL("bad digest");
    for (k = 0; k < 16; ++k) if (!gw_mb_ishex(f[9][k])) MB_FAIL("bad digest");
    gw_mb_digest64(text, (size_t) (f[9] - 1 - text), "build:", &hi, &lo);
    snprintf(want, sizeof want, "%08x%08x", (unsigned) hi, (unsigned) lo);
    if (memcmp(want, f[9], 16) != 0) MB_FAIL("record digest does not match its contents");
    for (k = 1; k <= 4; ++k) { /* seed, game, loop, port: canonical non-negative decimal integers */
        long v = 0;
        if (fl[k] == 0 || fl[k] > 10 || (fl[k] > 1 && f[k][0] == '0')) MB_FAIL("bad seed/game/loop/port");
        for (i = 0; i < fl[k]; ++i) {
            if (f[k][i] < '0' || f[k][i] > '9') MB_FAIL("bad seed/game/loop/port");
            v = v * 10 + (f[k][i] - '0');
        }
        if (v > 2147483647L) MB_FAIL("bad seed/game/loop/port");
        (&out->seed)[k - 1] = v;
    }
    if (fl[5] != 16) MB_FAIL("bad pool digest");
    for (k = 0; k < 16; ++k) if (!gw_mb_ishex(f[5][k])) MB_FAIL("bad pool digest");
    memcpy(out->pool, f[5], 16); out->pool[16] = 0;
    memcpy(out->digest, f[9], 16); out->digest[16] = 0;
    out->dig_hi = hi; out->dig_lo = lo;
    out->nmods = gw_mb_list(f[6], fl[6], 0);
    if (out->nmods < 0) MB_FAIL("modifier list is not strictly sorted id=tier items");
    if (gw_mb_list(f[7], fl[7], 1) < 0) MB_FAIL("implicit list is not strictly sorted key=number items");
    out->nkeystones = gw_mb_list(f[8], fl[8], 2);
    if (out->nkeystones < 0) MB_FAIL("keystone list is not strictly sorted ids");
    if (out->nkeystones > out->nmods) MB_FAIL("more keystones than modifiers");
    return 1;
#undef MB_FAIL
}

#endif
