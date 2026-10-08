/* gw_skins_core.h - the skin registry's pure logic (no file, no JSON, no game types), header-only so the
 * native test (pc/tests/skins_core_test.c) and the engine (gw_skins_boot.inc, in geno_registry.c) use one copy.
 *
 * A skin costume is one row appended to a fighter's own costume list. This file keeps the rows, orders them,
 * enforces the 255 cap, answers the per-fighter questions the game asks (count, strings, team colour, part
 * visibility, Kirby hat row, name, art) and defines the NETPLAY WIRE COSTUME:
 *
 *   wire < 255                      a base costume's index (the fighter's own costume; equal on both installs)
 *   wire = SK_WIRE_TAG | id30       a skin: 30 bits of its identity (mod id, entry, version or content digest)
 *
 * Costumes are cosmetic and never enter the simulation, so a skin the peer lacks is shown as costume 0.
 * See docs/superpowers/plans/2026-10-08-skins-registry.md in the workspace repo. */
#ifndef GW_SKINS_CORE_H
#define GW_SKINS_CORE_H

#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define SK_FK_MAX 160    /* FighterKinds: retail + the m-ex / Geno slots (GW_MEX_KIND_MAX is 0x21 + 94 = 127) */
#define SK_CAP 255       /* costume ids are 0..254: a u8 with 255 kept as the preload-all sentinel */
#define SK_WIRE_TAG 0x40000000
#define SK_WIRE_MASK 0x3FFFFFFF
#define SK_PATH 160
#define SK_SYM 96
#define SK_NAME 24
#define SK_ID 64

typedef struct sk_costume {
    int fk;                /* target FighterKind */
    char mod[SK_ID];       /* the skin mod's id (its folder name) */
    int entry;             /* costume number inside the mod, 0-based */
    int order;             /* skin.order */
    char name[SK_NAME];
    char file[SK_PATH], joint[SK_SYM], matanim[SK_SYM];
    char csp[SK_PATH], stock[SK_PATH];
    int team;              /* -1 none, 0 red, 1 blue, 2 green */
    int like;              /* the original costume whose part visibility this one copies */
    int kirby_hat;         /* 0..5, or -1: use `like` */
    int partner;           /* 1: pfile/pjoint/pmat are the partner kind's costume */
    char pfile[SK_PATH], pjoint[SK_SYM], pmat[SK_SYM];
    uint64_t id;           /* identity: mix(mod, entry, stamp) */
    int index;             /* the absolute costume index (assigned by sk_finalize) */
    int partner_row;       /* 1: a generated row on a partner kind (Nana, Sheik); strings are the partner block or its default */
} sk_costume;

typedef struct {
    sk_costume *c;
    int n, cap;
    int base[SK_FK_MAX];   /* the fighter's own costume count */
    int first[SK_FK_MAX];  /* index of the fighter's first row in c[], -1 */
    int added[SK_FK_MAX];  /* skin rows accepted for the fighter */
    int refused;           /* rows dropped by the cap */
    int finalized;
} sk_registry;

static sk_registry sk_reg;
static void (*sk_sink)(const char *msg);

static void sk_say(const char *fmt, ...) {
    char m[300];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(m, sizeof m, fmt, ap);
    va_end(ap);
    if (sk_sink) sk_sink(m);
}

static void sk_reset(void) {
    int i;
    free(sk_reg.c);
    memset(&sk_reg, 0, sizeof sk_reg);
    for (i = 0; i < SK_FK_MAX; ++i) sk_reg.first[i] = -1;
}

/* ---- identity ---------------------------------------------------------------------------------- */

static uint64_t sk_mix(uint64_t h, uint64_t v) {
    h ^= v;
    h *= 0x9E3779B97F4A7C15ull;
    h ^= h >> 31;
    h *= 0xBF58476D1CE4E5B9ull;
    h ^= h >> 29;
    return h;
}

static char sk_lower(char c) { return (c >= 'A' && c <= 'Z') ? (char) (c + 32) : c; }

static uint64_t sk_identity(const char *mod, int entry, const char *stamp) {
    uint64_t h = 0x534B494E00000001ull;
    const char *p;
    for (p = mod; *p; ++p) h = sk_mix(h, (unsigned char) sk_lower(*p));
    h = sk_mix(h, 0xFFu);
    h = sk_mix(h, (uint64_t) (unsigned) entry);
    h = sk_mix(h, 0xFEu);
    for (p = stamp ? stamp : ""; *p; ++p) h = sk_mix(h, (unsigned char) *p);
    return h;
}

/* ---- building ---------------------------------------------------------------------------------- */

/* Add a row (copied). The caller has validated it; fk must be 0..SK_FK_MAX-1. */
static int sk_add(const sk_costume *s) {
    if (s->fk < 0 || s->fk >= SK_FK_MAX) return -1;
    if (sk_reg.n == sk_reg.cap) {
        int nc = sk_reg.cap ? sk_reg.cap * 2 : 64;
        sk_costume *p = (sk_costume *) realloc(sk_reg.c, (size_t) nc * sizeof *p);
        if (!p) return -1;
        sk_reg.c = p;
        sk_reg.cap = nc;
    }
    sk_reg.c[sk_reg.n] = *s;
    sk_reg.c[sk_reg.n].partner_row = 0;
    sk_reg.c[sk_reg.n].index = -1;
    sk_reg.n++;
    return 0;
}

static int sk_strcasecmp(const char *a, const char *b) {
    for (;; ++a, ++b) {
        char x = sk_lower(*a), y = sk_lower(*b);
        if (x != y) return (unsigned char) x < (unsigned char) y ? -1 : 1;
        if (!x) return 0;
    }
}

/* (fk, order, mod id case-folded then exact, entry): the whole of the ordering rule. */
static int sk_cmp(const void *pa, const void *pb) {
    const sk_costume *a = (const sk_costume *) pa, *b = (const sk_costume *) pb;
    int r;
    if (a->fk != b->fk) return a->fk < b->fk ? -1 : 1;
    if (a->partner_row != b->partner_row) return a->partner_row < b->partner_row ? -1 : 1;
    if (a->order != b->order) return a->order < b->order ? -1 : 1;
    r = sk_strcasecmp(a->mod, b->mod);
    if (r) return r;
    r = strcmp(a->mod, b->mod);
    if (r) return r < 0 ? -1 : 1;
    if (a->entry != b->entry) return a->entry < b->entry ? -1 : 1;
    return 0;
}

static int sk_cmp_final(const void *pa, const void *pb) {
    const sk_costume *a = (const sk_costume *) pa, *b = (const sk_costume *) pb;
    if (a->fk != b->fk) return a->fk < b->fk ? -1 : 1;
    if (a->index != b->index) return a->index < b->index ? -1 : 1;
    return 0;
}

/* The kind a skin on `fk` also adds a row to (Popo -> Nana, Zelda -> Sheik), or -1. Retail FighterKinds. */
static int sk_partner_of(int fk) {
    return fk == 10 ? 11 : fk == 19 ? 7 : -1;
}
/* A partner kind takes no skins of its own: pick the main fighter. */
static int sk_is_partner_kind(int fk) { return fk == 11 || fk == 7; }

static const char *sk_fighter_name(int fk, char *buf, size_t cap, const char *(*name_fn)(int)) {
    const char *n = name_fn ? name_fn(fk) : NULL;
    if (n && *n) return n;
    snprintf(buf, cap, "fighter %d", fk);
    return buf;
}

/* Order the rows, assign indices after each fighter's own costumes, refuse past SK_CAP, resolve team claims,
 * and generate the partner rows. base_fn(fk) is the fighter's own costume count (0 = no list: its rows are refused).
 * name_fn(fk) is only for messages. Idempotent after the first call. */
static void sk_finalize(int (*base_fn)(int), const char *(*name_fn)(int)) {
    int i, fk, k, n0;
    sk_costume *out;
    int *seen;
    if (sk_reg.finalized) return;
    sk_reg.finalized = 1;
    for (fk = 0; fk < SK_FK_MAX; ++fk) sk_reg.base[fk] = base_fn ? base_fn(fk) : 0;
    if (sk_reg.n > 1) qsort(sk_reg.c, (size_t) sk_reg.n, sizeof *sk_reg.c, sk_cmp);
    n0 = sk_reg.n;
    out = (sk_costume *) calloc((size_t) (n0 * 2 + 1), sizeof *out);
    seen = (int *) calloc(SK_FK_MAX, sizeof *seen);
    k = 0;
    for (i = 0; i < n0; ++i) {
        sk_costume *s = &sk_reg.c[i];
        char nb[24];
        int idx, partner;
        fk = s->fk;
        if (sk_is_partner_kind(fk)) {
            sk_say("skins: %s/%d: %s takes its main fighter's skins - target %s instead", s->mod, s->entry,
                   sk_fighter_name(fk, nb, sizeof nb, name_fn), fk == 11 ? "popo" : "zelda");
            sk_reg.refused++;
            continue;
        }
        if (sk_reg.base[fk] <= 0) {
            sk_say("skins: %s/%d: %s has no costume list - skipped", s->mod, s->entry, sk_fighter_name(fk, nb, sizeof nb, name_fn));
            sk_reg.refused++;
            continue;
        }
        idx = sk_reg.base[fk] + seen[fk];
        if (idx >= SK_CAP) {
            sk_reg.refused++;
            seen[fk]++;
            if (idx == SK_CAP)
                sk_say("skins: %s is at %d costumes - refused %s (and any later ones)", sk_fighter_name(fk, nb, sizeof nb, name_fn),
                       SK_CAP, s->mod);
            continue;
        }
        seen[fk]++;
        out[k] = *s;
        out[k].index = idx;
        out[k].id = s->id;
        k++;
        sk_reg.added[fk]++;
        partner = sk_partner_of(fk);
        if (partner >= 0 && sk_reg.base[partner] == sk_reg.base[fk]) {
            sk_costume *p = &out[k];
            *p = out[k - 1];
            p->fk = partner;
            p->partner_row = 1;
            p->team = -1;
            if (p->partner) {
                snprintf(p->file, sizeof p->file, "%s", p->pfile);
                snprintf(p->joint, sizeof p->joint, "%s", p->pjoint);
                snprintf(p->matanim, sizeof p->matanim, "%s", p->pmat);
            } else {
                p->file[0] = p->joint[0] = p->matanim[0] = '\0'; /* the installer copies the partner's costume 0 */
            }
            p->csp[0] = p->stock[0] = '\0';
            sk_reg.added[partner]++;
            k++;
        } else if (partner >= 0) {
            sk_say("skins: %s/%d: the partner kind has %d costumes against %d - it keeps its default", s->mod, s->entry,
                   sk_reg.base[partner], sk_reg.base[fk]);
        }
    }
    free(sk_reg.c);
    free(seen);
    sk_reg.c = out;
    sk_reg.cap = n0 * 2 + 1;
    sk_reg.n = k;
    if (k > 1) qsort(sk_reg.c, (size_t) k, sizeof *sk_reg.c, sk_cmp_final);
    for (fk = 0; fk < SK_FK_MAX; ++fk) sk_reg.first[fk] = -1;
    for (i = 0; i < k; ++i) {
        int j;
        fk = sk_reg.c[i].fk;
        if (sk_reg.first[fk] < 0) sk_reg.first[fk] = i;
        /* a team colour is claimed once; the first row in order keeps it */
        if (sk_reg.c[i].team >= 0) {
            for (j = sk_reg.first[fk]; j < i; ++j)
                if (sk_reg.c[j].team == sk_reg.c[i].team) {
                    sk_say("skins: %s/%d: team %s is already costume %d - ignored", sk_reg.c[i].mod, sk_reg.c[i].entry,
                           sk_reg.c[i].team == 0 ? "red" : sk_reg.c[i].team == 1 ? "blue" : "green", sk_reg.c[j].index);
                    sk_reg.c[i].team = -1;
                    break;
                }
        }
    }
    /* two skins of one fighter with the same 30-bit wire identity would show each other's costume on a peer */
    for (i = 0; i < k; ++i) {
        int j;
        for (j = i + 1; j < k && sk_reg.c[j].fk == sk_reg.c[i].fk; ++j)
            if ((sk_reg.c[j].id & SK_WIRE_MASK) == (sk_reg.c[i].id & SK_WIRE_MASK))
                sk_say("skins: %s/%d and %s/%d share a netplay identity (cosmetic only)", sk_reg.c[i].mod, sk_reg.c[i].entry,
                       sk_reg.c[j].mod, sk_reg.c[j].entry);
    }
}

/* ---- queries ----------------------------------------------------------------------------------- */

static int sk_valid_fk(int fk) { return fk >= 0 && fk < SK_FK_MAX; }
static int sk_base(int fk) { return sk_valid_fk(fk) ? sk_reg.base[fk] : 0; }
static int sk_added(int fk) { return sk_valid_fk(fk) ? sk_reg.added[fk] : 0; }
/* The fighter's costume count with its skins (the number the select screen steps through); the base alone with none. */
static int sk_total(int fk) { return sk_valid_fk(fk) ? sk_reg.base[fk] + sk_reg.added[fk] : 0; }

/* The row for absolute costume `idx` of `fk`, or NULL for an original costume or no such costume. */
static const sk_costume *sk_row(int fk, int idx) {
    int j;
    if (!sk_valid_fk(fk) || sk_reg.first[fk] < 0 || idx < sk_reg.base[fk]) return NULL;
    j = sk_reg.first[fk] + (idx - sk_reg.base[fk]);
    if (j < 0 || j >= sk_reg.n || sk_reg.c[j].fk != fk || sk_reg.c[j].index != idx) return NULL;
    return &sk_reg.c[j];
}

/* The costume that is team colour t (0 red, 1 blue, 2 green) because a skin said so, or -1. */
static int sk_team(int fk, int t) {
    int j;
    if (!sk_valid_fk(fk) || sk_reg.first[fk] < 0) return -1;
    for (j = sk_reg.first[fk]; j < sk_reg.n && sk_reg.c[j].fk == fk; ++j)
        if (sk_reg.c[j].team == t) return sk_reg.c[j].index;
    return -1;
}

/* The original costume a skin's part visibility copies, or -1 for an original costume. */
static int sk_like(int fk, int idx) {
    const sk_costume *r = sk_row(fk, idx);
    if (!r) return -1;
    return r->like >= 0 && r->like < sk_reg.base[fk] ? r->like : 0;
}

/* The Kirby copy-hat row (0..5) for a Kirby costume that is a skin, or -1. */
static int sk_kirby_row(int fk, int idx) {
    const sk_costume *r = sk_row(fk, idx);
    if (!r) return -1;
    if (r->kirby_hat >= 0 && r->kirby_hat <= 5) return r->kirby_hat;
    return sk_like(fk, idx) % 6;
}

/* ---- the netplay wire costume ---------------------------------------------------------------- */

static int sk_to_wire(int fk, int idx) {
    const sk_costume *r;
    if (idx < 0) return 0;
    if (idx < sk_base(fk)) return idx;
    r = sk_row(fk, idx);
    if (r) return SK_WIRE_TAG | (int) (r->id & SK_WIRE_MASK);
    return idx < SK_CAP ? idx : 0; /* no such skin: leave the value as it was (a fighter with no list) */
}

/* A wire value from a peer -> this install's index for fighter fk. Unknown costume or skin: 0, the default. */
static int sk_from_wire(int fk, int wire) {
    int j;
    if (wire < 0) return 0;
    if (!(wire & SK_WIRE_TAG)) {
        int b = sk_base(fk);
        if (b <= 0) return wire < SK_CAP ? wire : 0; /* a fighter the registry does not know: pass it through */
        return wire < b ? wire : 0;
    }
    if (!sk_valid_fk(fk) || sk_reg.first[fk] < 0) return 0;
    for (j = sk_reg.first[fk]; j < sk_reg.n && sk_reg.c[j].fk == fk; ++j)
        if ((int) (sk_reg.c[j].id & SK_WIRE_MASK) == (wire & SK_WIRE_MASK) && !sk_reg.c[j].partner_row) return sk_reg.c[j].index;
    return 0;
}

#endif /* GW_SKINS_CORE_H */
