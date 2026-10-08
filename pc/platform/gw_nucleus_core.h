/* gw_nucleus_core.h - the SSBM Nucleus mod browser's pure logic: no network, no game types, no threads. Header-only (static functions) so the native
 * test (pc/tests/nucleus_core_test.c) and the engine half (gw_nucleus_engine.inc, included by gw_script.c) use one copy.
 *
 * What is here
 *   - the catalog: mods and their files, parsed from the public API's JSON (https://ssbmnucleus.net/developers), upserted by id, removed by id,
 *     saved to and loaded from a JSON-lines cache, filtered / searched / sorted;
 *   - the sync algorithm (nc_sync_run): first a full keyset page-through noted at T0, then updated_since = W - 15 min and the removed feed, at most
 *     one poll per 10 minutes, 429 / 5xx back-off, over an injected fetch function (so a test replays recorded JSON);
 *   - reading a costume DAT's public symbols (the slot / fighter is typed by CONTENT), the filename slot code, the cross-check;
 *   - the install: PNG -> .gxtex (RGB5A3, as pc/tools/png2gx.py writes it), the skin mod's mod.json, the installed-mod scan.
 *
 * Etiquette the API's owner asked for, and where it is kept: a User-Agent naming this tool (NC_USER_AGENT_FMT), polling no more than every 10
 * minutes (NC_POLL_SECONDS), downloads only through download_url and cached (the engine), the author and page link shown (the UI). */
#ifndef GW_NUCLEUS_CORE_H
#define GW_NUCLEUS_CORE_H

#include "gw_nucleus_json.h"
#include <ctype.h>
#include <time.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <dirent.h>
#include <sys/stat.h>
#endif

#define NC_API_BASE "https://ssbmnucleus.net/api/public/v1"
static char nc_api_base[200] = NC_API_BASE;      /* MELEE_NUCLEUS_API replaces it (a loopback fixture server only; see gw_nucleus_http.inc) */
#define NC_MEDIA_BASE "https://media.ssbmnucleus.net/"
#define NC_SITE "ssbmnucleus.net"
#define NC_CREDIT "Mods from SSBM Nucleus - ssbmnucleus.net"
#define NC_USER_AGENT_FMT "GDMelee/%s (+https://github.com/GurekamDhillon/melee)"
#define NC_POLL_SECONDS 600          /* the API owner's rule: no more than one poll per 10 minutes */
#define NC_OVERLAP_SECONDS 900       /* updated_since = W - 15 minutes */
#define NC_PAGE_LIMIT 100
#define NC_MAX_FILES_PER_MOD 40
#define NC_MAX_BODY (4u * 1024u * 1024u)

enum { NC_T_COSTUME, NC_T_STAGE_SKIN, NC_T_EFFECTS, NC_T_CUSTOM_STAGE, NC_T_GAMEPLAY, NC_T_PATCH, NC_T_OTHER, NC_T_N };
static const char *const nc_type_api[NC_T_N] = { "costume", "stage_skin", "effects", "custom_stage", "gameplay_mod", "patch", "other" };
static const char *const nc_type_label[NC_T_N] = { "Costume", "Stage skin", "Effects", "Custom stage", "Gameplay", "Patch", "Other" };

static int nc_type_of(const char *s) {
    int i;
    for (i = 0; i < NC_T_N; ++i) if (!strcmp(s, nc_type_api[i])) return i;
    return NC_T_OTHER;
}

/* ---- text helpers ------------------------------------------------------------------------------------ */

static void nc_copy(char *dst, size_t cap, const char *src) {
    size_t n = strlen(src);
    if (n >= cap) n = cap - 1;
    memcpy(dst, src, n);
    dst[n] = '\0';
}

/* The UI font is Latin; keep printable ASCII, one '?' per other code point, collapse newlines to spaces. */
static void nc_ascii(char *dst, size_t cap, const char *src) {
    size_t o = 0;
    while (*src && o + 1 < cap) {
        unsigned char c = (unsigned char) *src;
        if (c >= 0x80) {
            dst[o++] = '?';
            ++src;
            while (((unsigned char) *src & 0xC0) == 0x80) ++src;
        } else if (c < 0x20) { if (o && dst[o - 1] != ' ') dst[o++] = ' '; ++src; }
        else { dst[o++] = (char) c; ++src; }
    }
    dst[o] = '\0';
}

static int nc_ieq(const char *a, const char *b) {
    for (; *a && *b; ++a, ++b) if (tolower((unsigned char) *a) != tolower((unsigned char) *b)) return 0;
    return *a == *b;
}
static int nc_ieqn(const char *a, const char *b, size_t n) {
    size_t i;
    for (i = 0; i < n; ++i) if (tolower((unsigned char) a[i]) != tolower((unsigned char) b[i])) return 0;
    return 1;
}
static int nc_icontains(const char *hay, const char *needle) {
    size_t n = strlen(needle), i;
    if (!n) return 1;
    for (; *hay; ++hay) {
        for (i = 0; i < n && hay[i] && tolower((unsigned char) hay[i]) == tolower((unsigned char) needle[i]); ++i) {}
        if (i == n) return 1;
    }
    return 0;
}

/* ---- time -------------------------------------------------------------------------------------------- */

static int64_t nc_days_from_civil(int y, int m, int d) {
    int64_t era;
    unsigned yoe, doy, doe;
    y -= m <= 2;
    era = (y >= 0 ? y : y - 399) / 400;
    yoe = (unsigned) (y - era * 400);
    doy = (unsigned) ((153 * (m + (m > 2 ? -3 : 9)) + 2) / 5 + d - 1);
    doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    return era * 146097 + (int64_t) doe - 719468;
}

/* "2026-10-08T22:37:39Z" or "...39.000000Z": epoch seconds, or -1 */
static int64_t nc_parse_iso(const char *s) {
    int y, mo, d, h, mi, se;
    if (sscanf(s, "%4d-%2d-%2dT%2d:%2d:%2d", &y, &mo, &d, &h, &mi, &se) != 6) return -1;
    if (mo < 1 || mo > 12 || d < 1 || d > 31 || h > 23 || mi > 59 || se > 60) return -1;
    return nc_days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + se;
}

static void nc_iso(int64_t t, char *out, size_t cap) {
    int64_t z = t / 86400, rem = t % 86400, era;
    int y, m, d, h, mi, se;
    unsigned doe, yoe, doy, mp;
    if (rem < 0) { rem += 86400; --z; }
    z += 719468;
    era = (z >= 0 ? z : z - 146096) / 146097;
    doe = (unsigned) (z - era * 146097);
    yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    y = (int) yoe + (int) (era * 400);
    doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    mp = (5 * doy + 2) / 153;
    d = (int) (doy - (153 * mp + 2) / 5 + 1);
    m = (int) (mp < 10 ? mp + 3 : mp - 9);
    y += m <= 2;
    h = (int) (rem / 3600); mi = (int) (rem % 3600 / 60); se = (int) (rem % 60);
    snprintf(out, cap, "%04d-%02d-%02dT%02d:%02d:%02dZ", y, m, d, h, mi, se);
}

/* ---- fighters ---------------------------------------------------------------------------------------- */

/* One row per retail fighter, in FighterKind order. token: the symbol name Ply<token>5K<colour>_Share_joint; target: skin.target.retail; pl: the
 * PlXx file code; names: the API's "character" spellings, lower case, '|' separated; colours: the costume order from the game's own tables
 * (src/melee/ft/kinds/ft*), Nr = the default; they give a skin its `like` (the original costume whose part visibility it copies). */
typedef struct { const char *token, *target, *pl, *label, *names, *colours; int installable; } nc_fighter;
static const nc_fighter nc_fighters[] = {
    { "Mario", "mario", "Mr", "Mario", "mario", "Nr,Ye,Bk,Bu,Gr", 1 },
    { "Fox", "fox", "Fx", "Fox", "fox", "Nr,Or,La,Gr", 1 },
    { "Captain", "captain", "Ca", "Captain Falcon", "captain falcon", "Nr,Gy,Re,Wh,Gr,Bu", 1 },
    { "Donkey", "donkey", "Dk", "Donkey Kong", "donkey kong", "Nr,Bk,Re,Bu,Gr", 1 },
    { "Kirby", "kirby", "Kb", "Kirby", "kirby", "Nr,Ye,Bu,Re,Gr,Wh", 1 },
    { "Koopa", "koopa", "Kp", "Bowser", "bowser", "Nr,Re,Bu,Bk", 1 },
    { "Link", "link", "Lk", "Link", "link", "Nr,Re,Bu,Bk,Wh", 1 },
    { "Seak", "seak", "Sk", "Sheik", "sheik", "Nr,Re,Bu,Gr,Wh", 0 },
    { "Ness", "ness", "Ns", "Ness", "ness", "Nr,Ye,Bu,Gr", 1 },
    { "Peach", "peach", "Pe", "Peach", "peach", "Nr,Ye,Wh,Bu,Gr", 1 },
    { "Popo", "popo", "Pp", "Ice Climbers", "ice climbers|popo", "Nr,Gr,Or,Re", 1 },
    { "Nana", "nana", "Nn", "Nana", "nana", "Nr,Ye,Aq,Wh", 0 },
    { "Pikachu", "pikachu", "Pk", "Pikachu", "pikachu", "Nr,Re,Bu,Gr", 1 },
    { "Samus", "samus", "Ss", "Samus", "samus", "Nr,Pi,Bk,Gr,La", 1 },
    { "Yoshi", "yoshi", "Ys", "Yoshi", "yoshi", "Nr,Re,Bu,Ye,Pi,Aq", 1 },
    { "Purin", "purin", "Pr", "Jigglypuff", "jigglypuff", "Nr,Re,Bu,Gr,Ye", 1 },
    { "Mewtwo", "mewtwo", "Mt", "Mewtwo", "mewtwo", "Nr,Re,Bu,Gr", 1 },
    { "Luigi", "luigi", "Lg", "Luigi", "luigi", "Nr,Wh,Aq,Pi", 1 },
    { "Mars", "mars", "Ms", "Marth", "marth", "Nr,Re,Gr,Bk,Wh", 1 },
    { "Zelda", "zelda", "Zd", "Zelda", "zelda", "Nr,Re,Bu,Gr,Wh", 1 },
    { "Clink", "clink", "Cl", "Young Link", "young link", "Nr,Re,Bu,Wh,Bk", 1 },
    { "Drmario", "drmario", "Dr", "Dr. Mario", "dr. mario|dr mario|doctor mario", "Nr,Re,Bu,Gr,Bk", 1 },
    { "Falco", "falco", "Fc", "Falco", "falco", "Nr,Re,Bu,Gr", 1 },
    { "Pichu", "pichu", "Pc", "Pichu", "pichu", "Nr,Re,Bu,Gr", 1 },
    { "Gamewatch", "gamewatch", "Gw", "Mr. Game & Watch", "mr. game & watch|game & watch|mr game & watch", "Nr", 1 },
    { "Ganon", "ganon", "Gn", "Ganondorf", "ganondorf|ganon", "Nr,Re,Bu,Gr,La", 1 },
    { "Emblem", "emblem", "Fe", "Roy", "roy", "Nr,Re,Bu,Gr,Ye", 1 },
};
#define NC_NFIGHTERS ((int) (sizeof nc_fighters / sizeof nc_fighters[0]))

static int nc_fighter_by_token(const char *tok) {
    int i;
    for (i = 0; i < NC_NFIGHTERS; ++i) if (nc_ieq(nc_fighters[i].token, tok)) return i;
    return -1;
}
static int nc_fighter_by_pl(const char *pl) {
    int i;
    for (i = 0; i < NC_NFIGHTERS; ++i) if (!strcmp(nc_fighters[i].pl, pl)) return i;
    return -1;
}
static int nc_fighter_by_name(const char *name) {
    int i;
    if (!name || !*name) return -1;
    for (i = 0; i < NC_NFIGHTERS; ++i) {
        const char *p = nc_fighters[i].names;
        while (*p) {
            const char *e = strchr(p, '|');
            size_t n = e ? (size_t) (e - p) : strlen(p);
            if (n == strlen(name) && nc_ieqn(p, name, n)) return i;
            if (!e) break;
            p = e + 1;
        }
    }
    return -1;
}

#endif
