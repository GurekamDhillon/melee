/* geno_registry.c - Geno's native half: reads every mounting mod's geno.json, gives each fighter
 * entry a STABLE content-derived id, resolves what it attaches to, and answers the game half's
 * questions with scalars. Design and file format: docs/geno.md. Shared constants: pc/geno/geno.h.
 *
 * NATIVE platform code (i686 clang), not gwtool. Nothing here is simulation state: the registry
 * is built once, from files, before the first fighter spawns, and never changes during a match
 * (like the m-ex hook tables). Per-fighter runtime state lives in the game half, in snapshotted
 * memory (pc/geno/geno_game.c).
 *
 * geno.json (mods/<id>/geno.json, beside mod.json; never a disc file):
 *   { "geno": 1,
 *     "fighters": [
 *       { "attach": "kirby" | "PlKb.dat" | "PlSh.dat",     an existing fighter, vanilla or m-ex
 *         "name": "...",                                    display only
 *         "attributes": { "gravity": 0.08, "max_jumps": 9 }, ftCo_DatAttrs fields by name
 *         "jumps": { "max": 9, "air_vy": [1.6, 1.5] },       multi-jump past Melee's 5-row table
 *         "hooks": { "on_init": [..], "on_frame": ["geno.count_frames:3"], "on_action": [..] } } ] }
 *   "define" (a brand-new fighter) is reserved for a later version and is skipped with a log line.
 *   Unknown keys are ignored (but still hashed into the id) so newer files load their known parts.
 *
 * STABLE IDS. A fighter entry's id is a 64-bit hash of the entry's canonical form (keys in file
 * order, numbers normalised, whitespace dropped): the same definition hashes the same on every
 * install, whatever the mod folder is called and whichever dense m-ex slot its target lands in.
 * The id is folded into the TARGET fighter's netplay identity (gw_mexid.c) - only for fighters
 * Geno touches, so every m-ex / vanilla identity is bit-for-bit what it was.
 */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "gw_mods.h"
#include "gw_test.h"
#include "../geno/geno.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* the game half (pc/geno/geno_game.c; gwtool prefixes its symbols with gw_) */
extern int gw_GenoGame_AttrFind(const char *name);
extern int gw_GenoGame_AttrIsInt(int index);
extern int gw_GenoGame_HookFind(const char *name);

extern int gw_Mex_SlotInternal(int slot);
extern const char *gw_Mex_FtPlFile(int k);

/* ---- a tiny JSON reader (DOM over a fixed node pool) ------------------------------------------ */

enum { JN_NULL, JN_BOOL, JN_NUM, JN_STR, JN_ARR, JN_OBJ };

typedef struct {
    int type;
    double num;
    char *str;   /* JN_STR value (owned by the doc's arena) */
    char *key;   /* member name when the parent is an object */
    int first;   /* first child */
    int next;    /* next sibling */
    int count;
} jnode;

#define JDOC_NODES 2048
#define JDOC_ARENA 32768

typedef struct {
    jnode n[JDOC_NODES];
    int nn;
    char arena[JDOC_ARENA];
    int na;
    const char *p;
    const char *err;
} jdoc;

static int jd_new(jdoc *d, int type) {
    jnode *x;
    if (d->nn >= JDOC_NODES) {
        d->err = "too many values";
        return -1;
    }
    x = &d->n[d->nn];
    memset(x, 0, sizeof *x);
    x->type = type;
    x->first = x->next = -1;
    return d->nn++;
}

static void jd_ws(jdoc *d) {
    for (;;) {
        while (*d->p == ' ' || *d->p == '\t' || *d->p == '\r' || *d->p == '\n') d->p++;
        if (d->p[0] == '/' && d->p[1] == '/') { /* tolerate // comments: these files are hand-written */
            while (*d->p && *d->p != '\n') d->p++;
            continue;
        }
        return;
    }
}

static char *jd_string(jdoc *d) {
    char *out = &d->arena[d->na];
    if (*d->p != '"') {
        d->err = "expected a string";
        return NULL;
    }
    d->p++;
    while (*d->p && *d->p != '"') {
        char c = *d->p++;
        if (c == '\\') {
            char e = *d->p++;
            c = e == 'n' ? '\n' : e == 't' ? '\t' : e == 'r' ? '\r' : e == 'u' ? '?' : e;
            if (e == 'u') {
                int k;
                for (k = 0; k < 4 && *d->p; ++k) d->p++;
            }
        }
        if (d->na >= JDOC_ARENA - 1) {
            d->err = "strings too long";
            return NULL;
        }
        d->arena[d->na++] = c;
    }
    if (*d->p != '"') {
        d->err = "unterminated string";
        return NULL;
    }
    d->p++;
    d->arena[d->na++] = '\0';
    return out;
}

static int jd_value(jdoc *d, int depth);

static int jd_container(jdoc *d, int depth, int is_obj) {
    int me = jd_new(d, is_obj ? JN_OBJ : JN_ARR), last = -1;
    char close = is_obj ? '}' : ']';
    if (me < 0) return -1;
    d->p++;
    jd_ws(d);
    if (*d->p == close) {
        d->p++;
        return me;
    }
    for (;;) {
        char *key = NULL;
        int c;
        jd_ws(d);
        if (is_obj) {
            key = jd_string(d);
            if (key == NULL) return -1;
            jd_ws(d);
            if (*d->p != ':') {
                d->err = "expected ':'";
                return -1;
            }
            d->p++;
        }
        c = jd_value(d, depth + 1);
        if (c < 0) return -1;
        d->n[c].key = key;
        if (last < 0) d->n[me].first = c;
        else d->n[last].next = c;
        last = c;
        d->n[me].count++;
        jd_ws(d);
        if (*d->p == ',') {
            d->p++;
            jd_ws(d);
            if (*d->p == close) { /* trailing comma */
                d->p++;
                return me;
            }
            continue;
        }
        if (*d->p == close) {
            d->p++;
            return me;
        }
        d->err = is_obj ? "expected ',' or '}'" : "expected ',' or ']'";
        return -1;
    }
}

static int jd_value(jdoc *d, int depth) {
    int me;
    if (depth > 32) {
        d->err = "nested too deeply";
        return -1;
    }
    jd_ws(d);
    switch (*d->p) {
    case '{': return jd_container(d, depth, 1);
    case '[': return jd_container(d, depth, 0);
    case '"':
        me = jd_new(d, JN_STR);
        if (me < 0) return -1;
        d->n[me].str = jd_string(d);
        return d->n[me].str != NULL ? me : -1;
    case 't':
    case 'f':
    case 'n':
        if (strncmp(d->p, "true", 4) == 0 || strncmp(d->p, "false", 5) == 0) {
            me = jd_new(d, JN_BOOL);
            if (me < 0) return -1;
            d->n[me].num = d->p[0] == 't';
            d->p += d->p[0] == 't' ? 4 : 5;
            return me;
        }
        if (strncmp(d->p, "null", 4) == 0) {
            d->p += 4;
            return jd_new(d, JN_NULL);
        }
        break;
    default:
        if (*d->p == '-' || (*d->p >= '0' && *d->p <= '9')) {
            char *end;
            me = jd_new(d, JN_NUM);
            if (me < 0) return -1;
            d->n[me].num = strtod(d->p, &end);
            if (end == d->p) break;
            d->p = end;
            return me;
        }
    }
    d->err = "unexpected character";
    return -1;
}

/* Parse `text`; returns the root node or -1 (d->err says why). */
static int jd_parse(jdoc *d, const char *text) {
    int root;
    memset(d, 0, sizeof *d);
    d->p = text;
    if ((unsigned char) d->p[0] == 0xEF && (unsigned char) d->p[1] == 0xBB && (unsigned char) d->p[2] == 0xBF)
        d->p += 3;
    root = jd_value(d, 0);
    if (root < 0) return -1;
    jd_ws(d);
    if (*d->p != '\0') {
        d->err = "text after the top-level value";
        return -1;
    }
    return root;
}

static int jd_get(const jdoc *d, int obj, const char *key) {
    int c;
    if (obj < 0 || d->n[obj].type != JN_OBJ) return -1;
    for (c = d->n[obj].first; c >= 0; c = d->n[c].next)
        if (strcmp(d->n[c].key, key) == 0) return c;
    return -1;
}

/* ---- hashing (same mixer family as gw_mexid.c; the constants are Geno's own) ------------------ */

static uint64_t gn_mix(uint64_t h, uint64_t v) {
    h ^= v;
    h *= 0x9E3779B97F4A7C15ull;
    h ^= h >> 31;
    h *= 0xBF58476D1CE4E5B9ull;
    h ^= h >> 29;
    return h;
}

static uint64_t gn_hash_bytes(uint64_t h, const char *s, size_t n) {
    size_t i;
    h = gn_mix(h, n);
    for (i = 0; i < n; ++i) h = gn_mix(h, (unsigned char) s[i]);
    return h;
}

/* The canonical form of a value, fed straight into the hash: type tags, keys in file order,
 * numbers as %.9g (1, 1.0 and 1e0 are the same number), no whitespace. */
static uint64_t gn_hash_node(const jdoc *d, int x, uint64_t h) {
    const jnode *n = &d->n[x];
    char num[40];
    int c;
    h = gn_mix(h, 0x100u + (unsigned) n->type);
    switch (n->type) {
    case JN_BOOL:
        h = gn_mix(h, n->num != 0);
        break;
    case JN_NUM:
        snprintf(num, sizeof num, "%.9g", n->num);
        h = gn_hash_bytes(h, num, strlen(num));
        break;
    case JN_STR:
        h = gn_hash_bytes(h, n->str, strlen(n->str));
        break;
    case JN_ARR:
    case JN_OBJ:
        h = gn_mix(h, (uint64_t) n->count);
        for (c = n->first; c >= 0; c = d->n[c].next) {
            if (n->type == JN_OBJ) h = gn_hash_bytes(h, d->n[c].key, strlen(d->n[c].key));
            h = gn_hash_node(d, c, h);
        }
        break;
    default:
        break;
    }
    return h;
}

/* ---- targets ---------------------------------------------------------------------------------- */

/* Retail FighterKind -> Pl code and the names a geno.json may use for it. */
static const struct {
    const char *code, *name, *alias;
} gn_vanilla[27] = {
    { "Mr", "mario", NULL },      { "Fx", "fox", NULL },         { "Ca", "captain", "falcon" },
    { "Dk", "donkey", "dk" },     { "Kb", "kirby", NULL },       { "Kp", "koopa", "bowser" },
    { "Lk", "link", NULL },       { "Sk", "seak", "sheik" },     { "Ns", "ness", NULL },
    { "Pe", "peach", NULL },      { "Pp", "popo", NULL },        { "Nn", "nana", NULL },
    { "Pk", "pikachu", NULL },    { "Ss", "samus", NULL },       { "Ys", "yoshi", NULL },
    { "Pr", "purin", "jigglypuff" }, { "Mt", "mewtwo", NULL },   { "Lg", "luigi", NULL },
    { "Ms", "mars", "marth" },    { "Zd", "zelda", NULL },       { "Cl", "clink", "younglink" },
    { "Dr", "drmario", NULL },    { "Fc", "falco", NULL },       { "Pc", "pichu", NULL },
    { "Gw", "gamewatch", "gnw" }, { "Gn", "ganon", "ganondorf" }, { "Fe", "emblem", "roy" },
};

#define GN_MEX_KIND0 0x21 /* Ft_Kind_Mex0 */
#define GN_KINDS 64

/* The Pl file a target names ("kirby" -> "PlKb.dat"; "PlSh.dat" stays). "" when unknown. */
static void gn_target_file(const char *target, char *out, size_t cap) {
    size_t n = strlen(target);
    int i;
    out[0] = '\0';
    if (n > 4 && _stricmp(target + n - 4, ".dat") == 0) {
        snprintf(out, cap, "%s", target);
        return;
    }
    for (i = 0; i < 27; ++i) {
        if (_stricmp(target, gn_vanilla[i].name) == 0 ||
            (gn_vanilla[i].alias != NULL && _stricmp(target, gn_vanilla[i].alias) == 0)) {
            snprintf(out, cap, "Pl%s.dat", gn_vanilla[i].code);
            return;
        }
    }
}

/* This install's FighterKind for a Pl file: retail by code, m-ex by scanning the dense slots
 * (so the answer follows the slot wherever the enabled set puts it). -1 = not installed. */
static int gn_kind_for_file(const char *pl) {
    int i, s;
    for (i = 0; i < 27; ++i) {
        char f[16];
        snprintf(f, sizeof f, "Pl%s.dat", gn_vanilla[i].code);
        if (_stricmp(f, pl) == 0) return i;
    }
    for (s = 0; s < 31; ++s) {
        int k = gw_Mex_SlotInternal(s);
        const char *f;
        if (k < 0) break;
        f = gw_Mex_FtPlFile(k);
        if (f != NULL) {
            while (*f == '/') ++f;
            if (_stricmp(f, pl) == 0) return GN_MEX_KIND0 + s;
        }
    }
    return -1;
}

/* ---- profiles ----------------------------------------------------------------------------------- */

typedef struct {
    uint64_t id;
    char hex[17];
    char mod[64];
    char target[64];
    char plfile[32];
    char name[64];
    int kind; /* resolved at install; -1 = the target fighter is not on this install */
    int max_jumps; /* -1 = keep the fighter's own */
    int njvy;
    uint32_t jvy[GENO_MAX_JUMP_VY]; /* float bits */
    int nattr;
    int attr_index[GENO_MAX_ATTRS];
    uint32_t attr_bits[GENO_MAX_ATTRS];
    int nhook[GENO_EV_COUNT];
    int hook[GENO_EV_COUNT][GENO_EV_MAX_HOOKS];
    int hook_arg[GENO_EV_COUNT][GENO_EV_MAX_HOOKS];
    /* v1 */
    int nspec;
    int spec_index[GENO_MAX_SPECIAL]; /* word index into the special-attribute block */
    uint32_t spec_bits[GENO_MAX_SPECIAL];
    int nland;
    uint32_t land_from[GENO_MAX_ONLAND]; /* target words (GENO_TARGET) */
    uint32_t land_to[GENO_MAX_ONLAND];
    int nov;
    int ov_anim[GENO_MAX_OVERLAYS]; /* subaction index */
    int ov_slot[GENO_MAX_OVERLAYS]; /* registry-wide overlay slot */
} gn_profile;

#define GN_MAX_SLOTS 256 /* overlay slots over every profile */

typedef struct {
    gn_profile p[GENO_MAX_PROFILES];
    int n;
    int files;
    int kind_profile[GN_KINDS]; /* FighterKind -> profile index, -1 none */
    int kinds_built;
    /* v1: every overlay's words, one pool (the game half copies it into guest memory) */
    uint32_t pool[GENO_POOL_WORDS];
    int npool;
    int nslot;
    int slot_off[GN_MAX_SLOTS];
    int slot_len[GN_MAX_SLOTS];
} gn_registry;

static int gn_pool_gen = 1; /* bumped whenever a registry is (re)installed: the game half refills */

static gn_registry gn_boot;
static int gn_boot_loaded;

static uint32_t gn_fbits(double v) {
    float f = (float) v;
    uint32_t b;
    memcpy(&b, &f, 4);
    return b;
}

/* "name" or "name:arg" -> hook id (+arg). -1 when unknown. */
static int gn_hook_ref(const char *s, int *arg) {
    char name[64];
    const char *colon = strchr(s, ':');
    size_t n = colon != NULL ? (size_t) (colon - s) : strlen(s);
    if (n >= sizeof name) return -1;
    memcpy(name, s, n);
    name[n] = '\0';
    *arg = colon != NULL ? atoi(colon + 1) : 0;
    return gw_GenoGame_HookFind(name);
}

/* A geno.json target: a number (motion id), "motion:N", "special:N" or "geno:N". 1 when valid. */
static int gn_target(const jdoc *d, int x, uint32_t *out) {
    const char *s;
    unsigned kind;
    long id;
    char *end;
    if (x < 0) return 0;
    if (d->n[x].type == JN_NUM) {
        if (d->n[x].num < 0 || d->n[x].num > 0xFFFF) return 0;
        *out = GENO_TARGET(GENO_TGT_MOTION, (unsigned) d->n[x].num);
        return 1;
    }
    if (d->n[x].type != JN_STR) return 0;
    s = d->n[x].str;
    if (_strnicmp(s, "motion:", 7) == 0) kind = GENO_TGT_MOTION, s += 7;
    else if (_strnicmp(s, "special:", 8) == 0) kind = GENO_TGT_SPECIAL, s += 8;
    else if (_strnicmp(s, "geno:", 5) == 0) kind = GENO_TGT_GENO, s += 5;
    else return 0;
    id = strtol(s, &end, 0);
    if (end == s || *end != '\0' || id < 0 || id > 0xFFFF) return 0;
    *out = GENO_TARGET(kind, (unsigned) id);
    return 1;
}

/* One script word: a number (0..2^32-1, or a negative int) or a "0x..." / decimal string. */
static int gn_word(const jdoc *d, int x, uint32_t *out) {
    if (d->n[x].type == JN_NUM) {
        double v = d->n[x].num;
        if (v < -2147483648.0 || v > 4294967295.0 || v != (double) (long long) v) return 0;
        *out = (uint32_t) (long long) v;
        return 1;
    }
    if (d->n[x].type == JN_STR) {
        char *end;
        unsigned long long v = strtoull(d->n[x].str, &end, 0);
        if (end == d->n[x].str || *end != '\0' || v > 0xFFFFFFFFull) return 0;
        *out = (uint32_t) v;
        return 1;
    }
    return 0;
}

/* A words file (mods/<id>/<file>): whitespace-separated numbers (0x.. or decimal), '#' comments.
 * Appends to the pool; returns the word count or -1. */
static int gn_words_file(gn_registry *r, const char *mod, const char *file, const char *where) {
    char path[MAX_PATH];
    const char *dir = gw_Mods_Dir();
    FILE *f;
    char tok[64];
    int n = 0, c, k;
    if (dir == NULL || dir[0] == '\0' || strstr(file, "..") != NULL) {
        gw_log("geno: %s: script file \"%s\" refused", where, file);
        return -1;
    }
    snprintf(path, sizeof path, "%s\\%s\\%s", dir, mod, file);
    f = fopen(path, "r");
    if (f == NULL) {
        gw_log("geno: %s: cannot open script file %s", where, path);
        return -1;
    }
    for (;;) {
        c = fgetc(f);
        if (c == EOF) break;
        if (c == '#') {
            while (c != EOF && c != '\n') c = fgetc(f);
            continue;
        }
        if (c == ' ' || c == '\t' || c == '\r' || c == '\n' || c == ',') continue;
        k = 0;
        while (c != EOF && c != ' ' && c != '\t' && c != '\r' && c != '\n' && c != ',' && c != '#') {
            if (k < (int) sizeof tok - 1) tok[k++] = (char) c;
            c = fgetc(f);
        }
        tok[k] = '\0';
        {
            char *end;
            unsigned long long v = strtoull(tok, &end, 0);
            if (end == tok || *end != '\0' || v > 0xFFFFFFFFull || r->npool >= GENO_POOL_WORDS) {
                gw_log("geno: %s: %s: bad word \"%s\" (or the script pool is full)", where, file, tok);
                fclose(f);
                return -1;
            }
            r->pool[r->npool++] = (uint32_t) v;
            n++;
        }
        if (c == '#') {
            while (c != EOF && c != '\n') c = fgetc(f);
        }
        if (c == EOF) break;
    }
    fclose(f);
    return n;
}

static void gn_add_v1(gn_registry *r, gn_profile *p, const jdoc *d, int e, const char *mod,
                      const char *where) {
    int x, c;
    /* special_attributes: [ { "index": i | "offset": bytes, "float": x | "int": n } ] */
    x = jd_get(d, e, "special_attributes");
    if (x >= 0 && d->n[x].type == JN_ARR) {
        for (c = d->n[x].first; c >= 0; c = d->n[c].next) {
            int ix = jd_get(d, c, "index"), off = jd_get(d, c, "offset");
            int vf = jd_get(d, c, "float"), vi = jd_get(d, c, "int");
            long idx = -1;
            uint32_t w;
            if (ix >= 0 && d->n[ix].type == JN_NUM) idx = (long) d->n[ix].num;
            else if (off >= 0 && gn_word(d, off, &w) && (w & 3) == 0) idx = (long) (w / 4);
            if (idx < 0 || idx >= GENO_SPECIAL_WORDS ||
                ((vf < 0 || d->n[vf].type != JN_NUM) && (vi < 0 || d->n[vi].type != JN_NUM))) {
                gw_log("geno: %s: a special_attributes entry needs \"index\" (0-%d) or a 4-aligned "
                       "\"offset\", and a \"float\" or \"int\" value - ignored", where,
                       GENO_SPECIAL_WORDS - 1);
                continue;
            }
            if (p->nspec >= GENO_MAX_SPECIAL) break;
            p->spec_index[p->nspec] = (int) idx;
            p->spec_bits[p->nspec] = vf >= 0 && d->n[vf].type == JN_NUM
                                         ? gn_fbits(d->n[vf].num)
                                         : (uint32_t) (int32_t) (long long) d->n[vi].num;
            p->nspec++;
        }
    }
    /* on_land: [ { "from": target, "to": target, "keep_frame": bool } ] */
    x = jd_get(d, e, "on_land");
    if (x >= 0 && d->n[x].type == JN_ARR) {
        for (c = d->n[x].first; c >= 0; c = d->n[c].next) {
            uint32_t from, to;
            int kf = jd_get(d, c, "keep_frame");
            if (!gn_target(d, jd_get(d, c, "from"), &from) || !gn_target(d, jd_get(d, c, "to"), &to)) {
                gw_log("geno: %s: an on_land entry needs \"from\" and \"to\" (a motion id, "
                       "\"special:N\", \"motion:N\" or \"geno:N\") - ignored", where);
                continue;
            }
            if (kf >= 0 && d->n[kf].type == JN_BOOL && d->n[kf].num != 0) to |= GENO_TGT_KEEP_FRAME;
            if (p->nland >= GENO_MAX_ONLAND) break;
            p->land_from[p->nland] = from;
            p->land_to[p->nland] = to;
            p->nland++;
        }
    }
    /* subactions: [ { "index": n, "words": [..] | "file": "geno/x.txt" } ] - script overlays */
    x = jd_get(d, e, "subactions");
    if (x >= 0 && d->n[x].type == JN_ARR) {
        for (c = d->n[x].first; c >= 0; c = d->n[c].next) {
            int ix = jd_get(d, c, "index"), wl = jd_get(d, c, "words"), fl = jd_get(d, c, "file");
            int start = r->npool, n = 0, ok = 1, w;
            if (ix < 0 || d->n[ix].type != JN_NUM || d->n[ix].num < 0 || d->n[ix].num > 0x3FF) {
                gw_log("geno: %s: a subactions entry needs \"index\" 0-1023 - ignored", where);
                continue;
            }
            if (p->nov >= GENO_MAX_OVERLAYS || r->nslot >= GN_MAX_SLOTS) {
                gw_log("geno: %s: too many subaction overlays - the rest are ignored", where);
                break;
            }
            if (wl >= 0 && d->n[wl].type == JN_ARR) {
                for (w = d->n[wl].first; w >= 0; w = d->n[w].next) {
                    uint32_t v;
                    if (!gn_word(d, w, &v) || r->npool >= GENO_POOL_WORDS) {
                        ok = 0;
                        break;
                    }
                    r->pool[r->npool++] = v;
                    n++;
                }
            } else if (fl >= 0 && d->n[fl].type == JN_STR) {
                n = gn_words_file(r, mod, d->n[fl].str, where);
                ok = n >= 0;
            } else {
                ok = 0;
            }
            if (!ok || n == 0) {
                gw_log("geno: %s: subaction %d: no usable \"words\" / \"file\" - ignored", where,
                       (int) d->n[ix].num);
                r->npool = start;
                continue;
            }
            /* an overlay always ends (an End word after it): a script that ran off its last word
               would read the next overlay */
            if (r->npool >= GENO_POOL_WORDS) {
                r->npool = start;
                continue;
            }
            r->pool[r->npool++] = 0;
            n++;
            p->ov_anim[p->nov] = (int) d->n[ix].num;
            p->ov_slot[p->nov] = r->nslot;
            r->slot_off[r->nslot] = start;
            r->slot_len[r->nslot] = n;
            r->nslot++;
            p->nov++;
        }
    }
}

static void gn_add_fighter(gn_registry *r, const jdoc *d, int e, const char *mod, const char *where) {
    static const char *const ev_names[GENO_EV_COUNT] = { "on_init", "on_frame", "on_action",
                                                         "on_land" };
    gn_profile *p;
    int x, c, ev;
    if (d->n[e].type != JN_OBJ) {
        gw_log("geno: %s: a fighters[] entry is not an object - skipped", where);
        return;
    }
    if (jd_get(d, e, "define") >= 0) {
        gw_log("geno: %s: \"define\" (new fighters) is not supported by Geno v%d yet - entry skipped",
               where, GENO_VERSION);
        return;
    }
    x = jd_get(d, e, "attach");
    if (x < 0 || d->n[x].type != JN_STR) {
        gw_log("geno: %s: entry has no \"attach\" - skipped", where);
        return;
    }
    if (r->n >= GENO_MAX_PROFILES) {
        gw_log("geno: %s: more than %d fighter entries - the rest are skipped", where, GENO_MAX_PROFILES);
        return;
    }
    p = &r->p[r->n];
    memset(p, 0, sizeof *p);
    p->kind = -1;
    p->max_jumps = -1;
    snprintf(p->mod, sizeof p->mod, "%s", mod);
    snprintf(p->target, sizeof p->target, "%s", d->n[x].str);
    gn_target_file(p->target, p->plfile, sizeof p->plfile);
    if (p->plfile[0] == '\0') {
        gw_log("geno: %s: attach \"%s\" names no fighter I know (use a vanilla name or a Pl*.dat)",
               where, p->target);
        return;
    }
    x = jd_get(d, e, "name");
    snprintf(p->name, sizeof p->name, "%s", x >= 0 && d->n[x].type == JN_STR ? d->n[x].str : p->plfile);

    x = jd_get(d, e, "attributes");
    if (x >= 0 && d->n[x].type == JN_OBJ) {
        for (c = d->n[x].first; c >= 0; c = d->n[c].next) {
            int idx = gw_GenoGame_AttrFind(d->n[c].key);
            if (idx < 0 || d->n[c].type != JN_NUM) {
                gw_log("geno: %s: attribute \"%s\" %s - ignored", where, d->n[c].key,
                       idx < 0 ? "is not a common attribute Geno knows" : "is not a number");
                continue;
            }
            if (p->nattr >= GENO_MAX_ATTRS) break;
            p->attr_index[p->nattr] = idx;
            p->attr_bits[p->nattr] = gw_GenoGame_AttrIsInt(idx) ? (uint32_t) (int32_t) d->n[c].num
                                                               : gn_fbits(d->n[c].num);
            p->nattr++;
        }
    }
    x = jd_get(d, e, "jumps");
    if (x >= 0 && d->n[x].type == JN_OBJ) {
        int m = jd_get(d, x, "max"), v = jd_get(d, x, "air_vy");
        if (m >= 0 && d->n[m].type == JN_NUM) {
            int mj = (int) d->n[m].num;
            p->max_jumps = mj < 1 ? 1 : mj > 250 ? 250 : mj; /* x1968_jumpsUsed is a u8 */
        }
        if (v >= 0 && d->n[v].type == JN_ARR) {
            for (c = d->n[v].first; c >= 0 && p->njvy < GENO_MAX_JUMP_VY; c = d->n[c].next)
                if (d->n[c].type == JN_NUM) p->jvy[p->njvy++] = gn_fbits(d->n[c].num);
        }
    }
    x = jd_get(d, e, "hooks");
    for (ev = 0; ev < GENO_EV_COUNT && x >= 0; ++ev) {
        int l = jd_get(d, x, ev_names[ev]);
        if (l < 0 || d->n[l].type != JN_ARR) continue;
        for (c = d->n[l].first; c >= 0; c = d->n[c].next) {
            int arg = 0, id = d->n[c].type == JN_STR ? gn_hook_ref(d->n[c].str, &arg) : -1;
            if (id <= 0) {
                gw_log("geno: %s: %s hook \"%s\" is unknown - ignored", where, ev_names[ev],
                       d->n[c].type == JN_STR ? d->n[c].str : "?");
                continue;
            }
            if (p->nhook[ev] >= GENO_EV_MAX_HOOKS) break;
            p->hook[ev][p->nhook[ev]] = id;
            p->hook_arg[ev][p->nhook[ev]] = arg;
            p->nhook[ev]++;
        }
    }
    gn_add_v1(r, p, d, e, mod, where);
    /* the id covers the whole entry as written - including keys this version ignores, which a
       newer Geno might act on, so two installs never agree on an id while behaving differently */
    p->id = gn_hash_node(d, e, gn_mix(0x47454E4F00000000ull, GENO_VERSION)); /* "GENO" */
    {
        /* script overlays loaded from files are content too: fold their words in (an entry with
           no overlays hashes exactly as before) */
        int o, w;
        for (o = 0; o < p->nov; ++o) {
            int s = p->ov_slot[o];
            p->id = gn_mix(p->id, 0x4F56000000000000ull | (uint64_t) p->ov_anim[o]);
            for (w = 0; w < r->slot_len[s]; ++w) p->id = gn_mix(p->id, r->pool[r->slot_off[s] + w]);
        }
    }
    if (p->id == 0) p->id = 1;
    snprintf(p->hex, sizeof p->hex, "%016llx", (unsigned long long) p->id);
    r->n++;
}

/* Parse one geno.json text into r. Returns the number of fighter entries taken, -1 on bad JSON. */
static int gn_parse_text(gn_registry *r, const char *text, const char *mod) {
    static jdoc d; /* 70 KB: not on the stack */
    int root, v, f, e, before = r->n;
    char where[96];
    snprintf(where, sizeof where, "%s/geno.json", mod);
    root = jd_parse(&d, text);
    if (root < 0) {
        gw_log("geno: %s is not valid JSON (%s near \"%.20s\") - ignored", where, d.err, d.p);
        return -1;
    }
    v = jd_get(&d, root, "geno");
    if (v < 0 || d.n[v].type != JN_NUM || (int) d.n[v].num < 1) {
        gw_log("geno: %s has no \"geno\": <version> - ignored", where);
        return -1;
    }
    if ((int) d.n[v].num > GENO_VERSION)
        gw_log("geno: %s is Geno v%d, this build is v%d - reading what it knows", where,
               (int) d.n[v].num, GENO_VERSION);
    f = jd_get(&d, root, "fighters");
    if (f >= 0 && d.n[f].type == JN_ARR)
        for (e = d.n[f].first; e >= 0; e = d.n[e].next) gn_add_fighter(r, &d, e, mod, where);
    r->files++;
    return r->n - before;
}

/* FighterKind -> profile, once the m-ex slots exist. Two entries for one fighter: the later one
 * (mount order; later mods win a disc path too) wins, and the loser is logged. */
static void gn_build_kinds(gn_registry *r) {
    int i;
    for (i = 0; i < GN_KINDS; ++i) r->kind_profile[i] = -1;
    for (i = 0; i < r->n; ++i) {
        gn_profile *p = &r->p[i];
        p->kind = gn_kind_for_file(p->plfile);
        if (p->kind < 0 || p->kind >= GN_KINDS) {
            gw_log("geno: %s/%s attaches to %s, which this install does not have - inactive", p->mod,
                   p->name, p->plfile);
            p->kind = -1;
            continue;
        }
        if (r->kind_profile[p->kind] >= 0)
            gw_log("geno: %s's entry for %s replaces %s's", p->mod, p->plfile,
                   r->p[r->kind_profile[p->kind]].mod);
        r->kind_profile[p->kind] = i;
        gw_log("geno: fighter %s (%s) -> kind %d: id %s, %d attribute(s), max jumps %d, %d air vy, "
               "hooks %d/%d/%d/%d, %d special attribute(s), %d on_land, %d subaction overlay(s)",
               p->name, p->plfile, p->kind, p->hex, p->nattr, p->max_jumps, p->njvy, p->nhook[0],
               p->nhook[1], p->nhook[2], p->nhook[3], p->nspec, p->nland, p->nov);
    }
    r->kinds_built = 1;
}

static char *gn_read_file(const char *path) {
    FILE *f = fopen(path, "rb");
    long n;
    char *buf;
    if (f == NULL) return NULL;
    fseek(f, 0, SEEK_END);
    n = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (n < 0 || n > 1 << 20) {
        fclose(f);
        return NULL;
    }
    buf = (char *) malloc((size_t) n + 1);
    if (buf != NULL) {
        buf[fread(buf, 1, (size_t) n, f)] = '\0';
    }
    fclose(f);
    return buf;
}

static gn_registry *gn_reg(void) {
    if (!gn_boot_loaded) {
        int n, k;
        const char *dir = gw_Mods_Dir();
        gn_boot_loaded = 1;
        memset(&gn_boot, 0, sizeof gn_boot);
        for (k = 0; k < GN_KINDS; ++k) gn_boot.kind_profile[k] = -1;
        n = gw_Mods_ActiveCount();
        for (k = 0; k < n && dir != NULL && dir[0]; ++k) {
            int i = gw_Mods_ActiveAt(k);
            char path[MAX_PATH];
            char *text;
            snprintf(path, sizeof path, "%s\\%s\\geno.json", dir, gw_Mods_Id(i));
            text = gn_read_file(path);
            if (text == NULL) continue;
            gn_parse_text(&gn_boot, text, gw_Mods_Id(i));
            free(text);
        }
        if (gn_boot.files > 0)
            gw_log("geno: %d geno.json file(s), %d fighter entr%s", gn_boot.files, gn_boot.n,
                   gn_boot.n == 1 ? "y" : "ies");
    }
    return &gn_boot;
}

static const gn_profile *gn_at(int p) {
    gn_registry *r = gn_reg();
    return p >= 0 && p < r->n ? &r->p[p] : NULL;
}

/* ---- the game half's API (scalars only; declared without gw_ in pc/geno/geno_game.c) ---------- */

int gw_Geno_ProfileForKind(int kind) {
    gn_registry *r = gn_reg();
    if (r->n == 0 || kind < 0 || kind >= GN_KINDS) return -1;
    if (!r->kinds_built) gn_build_kinds(r);
    return r->kind_profile[kind];
}
int gw_Geno_MaxJumps(int p) { return gn_at(p) ? gn_at(p)->max_jumps : -1; }
int gw_Geno_JumpVyCount(int p) { return gn_at(p) ? gn_at(p)->njvy : 0; }
int gw_Geno_JumpVyBits(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->njvy ? (int) x->jvy[i] : 0;
}
int gw_Geno_AttrCount(int p) { return gn_at(p) ? gn_at(p)->nattr : 0; }
int gw_Geno_AttrIndex(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nattr ? x->attr_index[i] : -1;
}
int gw_Geno_AttrBits(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nattr ? (int) x->attr_bits[i] : 0;
}
int gw_Geno_HookCount(int p, int ev) {
    const gn_profile *x = gn_at(p);
    return x != NULL && ev >= 0 && ev < GENO_EV_COUNT ? x->nhook[ev] : 0;
}
int gw_Geno_Hook(int p, int ev, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && ev >= 0 && ev < GENO_EV_COUNT && i >= 0 && i < x->nhook[ev] ? x->hook[ev][i] : 0;
}
int gw_Geno_HookArg(int p, int ev, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && ev >= 0 && ev < GENO_EV_COUNT && i >= 0 && i < x->nhook[ev] ? x->hook_arg[ev][i]
                                                                                   : 0;
}

/* v1 */
int gw_Geno_SpecialCount(int p) { return gn_at(p) ? gn_at(p)->nspec : 0; }
int gw_Geno_SpecialIndex(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nspec ? x->spec_index[i] : -1;
}
int gw_Geno_SpecialBits(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nspec ? (int) x->spec_bits[i] : 0;
}
int gw_Geno_OnLandCount(int p) { return gn_at(p) ? gn_at(p)->nland : 0; }
int gw_Geno_OnLandFrom(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nland ? (int) x->land_from[i] : -1;
}
int gw_Geno_OnLandTo(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nland ? (int) x->land_to[i] : -1;
}
int gw_Geno_OverlayCount(int p) { return gn_at(p) ? gn_at(p)->nov : 0; }
int gw_Geno_OverlayAnim(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nov ? x->ov_anim[i] : -1;
}
int gw_Geno_OverlaySlot(int p, int i) {
    const gn_profile *x = gn_at(p);
    return x != NULL && i >= 0 && i < x->nov ? x->ov_slot[i] : -1;
}
int gw_Geno_SlotCount(void) { return gn_reg()->nslot; }
int gw_Geno_SlotOffset(int s) { return s >= 0 && s < gn_reg()->nslot ? gn_reg()->slot_off[s] : -1; }
int gw_Geno_SlotLen(int s) { return s >= 0 && s < gn_reg()->nslot ? gn_reg()->slot_len[s] : 0; }
int gw_Geno_PoolWords(void) { return gn_reg()->npool; }
int gw_Geno_PoolWord(int i) { return i >= 0 && i < gn_reg()->npool ? (int) gn_reg()->pool[i] : 0; }
/* Changes whenever the installed registry changes (tests swap registries): the game half keeps a
 * copy of the pool in guest memory and refills it when this differs from what it copied. */
int gw_Geno_PoolGen(void) {
    gn_reg();
    return gn_pool_gen;
}

/* Log lines for the game half, which cannot format strings portably. Rate-limited per `what`:
 * rollback resimulates frames, and a per-frame event would flood the log. */
void gw_Geno_Event(int what, int a, int b, int c, int d) {
    static int count[32];
    static const char *const fmt[] = {
        "geno: kind %d player %d reset (profile %d)%.0d",                                  /* 0 */
        "geno: kind %d player %d air jump %d of %d (beyond Melee's multi-jump table)",       /* 1 */
        "geno: kind %d player %d hook geno.log arg %d LA0 %d",                              /* 2 */
        "geno: kind %d player %d script sub-command %d is unknown - skipped%.0d",           /* 3 */
        "geno: kind %d player %d script calls unknown hook %d%.0d",                         /* 4 */
        "geno: kind %d player %d attributes overridden: %d field(s), max_jumps %d",         /* 5 */
        "geno: kind %d player %d air jump %d of %d",                                        /* 6 */
        "geno: kind %d player %d change action: motion %d -> target 0x%08x",                /* 7 */
        "geno: kind %d player %d change action to Geno state %d: Geno states are v2 - ignored%.0d", /* 8 */
        "geno: kind %d player %d landed in motion %d -> target 0x%08x",                     /* 9 */
        "geno: kind %d player %d rehit: cleared the hit lists of hitbox mask 0x%x (every %d frames)", /* 10 */
        "geno: kind %d player %d autolink: victim launched at angle %d, kb %d",             /* 11 */
        "geno: kind %d player %d special attributes: %d word(s) overridden (first word %d)", /* 12 */
        "geno: kind %d player %d subaction %d script replaced by overlay slot %d",           /* 13 */
        "geno: kind %d player %d engine value 0x%x is read-only or unknown - write ignored%.0d", /* 14 */
        "geno: kind %d player %d too many change-action checks (max %d) - dropped%.0d",      /* 15 */
    };
    if (what < 0 || what >= (int) (sizeof fmt / sizeof fmt[0])) return;
    if (++count[what] > 40) {
        if (count[what] == 41) gw_log("geno: (further event %d lines suppressed)", what);
        return;
    }
    gw_log(fmt[what], a, b, c, d);
}

/* ---- for gw_mexid.c: fold a Geno overlay into its target's netplay identity ------------------ */

/* The id of the profile that attaches to the fighter whose main Pl file is `pl`, 0 when none. Only
 * profiles that are ACTIVE on this install count (their target exists). */
uint64_t gw_Geno_SaltForPlFile(const char *pl) {
    gn_registry *r = gn_reg();
    int i, best = -1;
    if (pl == NULL || r->n == 0) return 0;
    while (*pl == '/') ++pl;
    for (i = 0; i < r->n; ++i)
        if (_stricmp(r->p[i].plfile, pl) == 0) best = i; /* later wins, as in gn_build_kinds */
    return best >= 0 ? r->p[best].id : 0;
}

int gw_Geno_ProfileCount(void) { return gn_reg()->n; }
const char *gw_Geno_ProfileHex(int p) { return gn_at(p) ? gn_at(p)->hex : ""; }

/* ---- test support: swap in a registry parsed from text (game-side tests), then put the boot one
 * back. Never called outside --test. */
static gn_registry gn_saved;
static int gn_saved_loaded, gn_swapped;

int gw_Geno_TestInstall(const char *text) {
    int n;
    if (!gn_swapped) {
        gn_reg();
        gn_saved = gn_boot;
        gn_saved_loaded = gn_boot_loaded;
        gn_swapped = 1;
    }
    memset(&gn_boot, 0, sizeof gn_boot);
    gn_boot_loaded = 1;
    n = gn_parse_text(&gn_boot, text, "test");
    gn_build_kinds(&gn_boot);
    gn_pool_gen++;
    return n;
}

void gw_Geno_TestRestore(void) {
    if (!gn_swapped) return;
    gn_boot = gn_saved;
    gn_boot_loaded = gn_saved_loaded;
    gn_swapped = 0;
    gn_pool_gen++;
}

static const char *gn_test_pinned_hex(void) { return "39ebc1bd1de2fc58"; }

/* ---- tests ------------------------------------------------------------------------------------- */

static const char gn_test_json[] =
    "{ \"geno\": 1,\n"
    "  \"fighters\": [\n"
    "    { \"attach\": \"kirby\", \"name\": \"Test Kirby\",\n"
    "      \"attributes\": { \"gravity\": 0.08, \"max_jumps\": 9, \"no_such_attr\": 1 },\n"
    "      \"jumps\": { \"max\": 9, \"air_vy\": [1.5, 1.25, 1.0] },\n"
    "      \"hooks\": { \"on_frame\": [\"geno.count_frames:3\", \"nope\"], \"on_init\": [\"geno.log\"] },\n"
    "      \"future_key\": { \"x\": [1, 2] } },\n"
    "    { \"define\": { \"name\": \"Meta Knight\" } },\n"
    "    { \"attach\": \"PlZz.dat\" }\n"
    "  ] }\n";

static int test_geno_registry_parse(void) {
    static gn_registry r;
    const gn_profile *p;
    float f;
    memset(&r, 0, sizeof r);
    if (gn_parse_text(&r, gn_test_json, "test-mod") != 2) {
        gw_test_fail("expected 2 entries (kirby, PlZz.dat; the define is skipped), got %d", r.n);
        return 1;
    }
    p = &r.p[0];
    if (strcmp(p->plfile, "PlKb.dat") != 0 || strcmp(p->name, "Test Kirby") != 0) {
        gw_test_fail("target/name: %s / %s", p->plfile, p->name);
        return 1;
    }
    if (p->nattr != 2 || p->max_jumps != 9 || p->njvy != 3) {
        gw_test_fail("nattr %d max_jumps %d njvy %d", p->nattr, p->max_jumps, p->njvy);
        return 1;
    }
    memcpy(&f, &p->attr_bits[0], 4);
    if (f != 0.08f || !gw_GenoGame_AttrIsInt(p->attr_index[1]) || p->attr_bits[1] != 9) {
        gw_test_fail("attribute values wrong (gravity %g, max_jumps bits %u)", f, p->attr_bits[1]);
        return 1;
    }
    memcpy(&f, &p->jvy[1], 4);
    if (f != 1.25f) {
        gw_test_fail("air_vy[1] = %g", f);
        return 1;
    }
    if (p->nhook[GENO_EV_FRAME] != 1 || p->hook[GENO_EV_FRAME][0] != GENO_HOOK_COUNT_FRAMES ||
        p->hook_arg[GENO_EV_FRAME][0] != 3 || p->nhook[GENO_EV_INIT] != 1 ||
        p->hook[GENO_EV_INIT][0] != GENO_HOOK_LOG) {
        gw_test_fail("hooks not parsed as expected");
        return 1;
    }
    /* vanilla targets resolve without m-ex; an unknown file stays inactive */
    gn_build_kinds(&r);
    if (r.kind_profile[4] != 0 || r.p[1].kind != -1) {
        gw_test_fail("kind table: kirby -> %d, PlZz.dat kind %d", r.kind_profile[4], r.p[1].kind);
        return 1;
    }
    return 0;
}

static int test_geno_registry_stable_ids(void) {
    static gn_registry a, b, c;
    /* same content, different whitespace, number spelling and mod folder: same id */
    const char *t1 = "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":9}}]}";
    const char *t2 = "{ \"geno\": 1, \"fighters\": [ {\n \"attach\" : \"kirby\" , \"jumps\" : { \"max\" : 9.0 } } ] }";
    const char *t3 = "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\",\"jumps\":{\"max\":8}}]}";
    memset(&a, 0, sizeof a);
    memset(&b, 0, sizeof b);
    memset(&c, 0, sizeof c);
    if (gn_parse_text(&a, t1, "one") != 1 || gn_parse_text(&b, t2, "two") != 1 ||
        gn_parse_text(&c, t3, "one") != 1) {
        gw_test_fail("parse failed");
        return 1;
    }
    if (a.p[0].id != b.p[0].id) {
        gw_test_fail("same definition, different ids: %s vs %s", a.p[0].hex, b.p[0].hex);
        return 1;
    }
    if (a.p[0].id == c.p[0].id) {
        gw_test_fail("different definitions share id %s", a.p[0].hex);
        return 1;
    }
    /* the known value pins the canonical form: if this changes, every installed overlay's
       netplay identity changes with it - that must be a deliberate version bump */
    if (strcmp(a.p[0].hex, gn_test_pinned_hex()) != 0) {
        gw_test_fail("canonical hash changed: %s (pinned %s)", a.p[0].hex, gn_test_pinned_hex());
        return 1;
    }
    return 0;
}

static int test_geno_registry_bad_json(void) {
    static gn_registry r;
    memset(&r, 0, sizeof r);
    if (gn_parse_text(&r, "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\"", "bad") != -1 || r.n != 0) {
        gw_test_fail("truncated JSON was accepted");
        return 1;
    }
    if (gn_parse_text(&r, "{\"fighters\":[]}", "nover") != -1) {
        gw_test_fail("a file without \"geno\" was accepted");
        return 1;
    }
    /* comments and trailing commas are tolerated (hand-written files) */
    if (gn_parse_text(&r, "{\"geno\":1, // v1\n\"fighters\":[{\"attach\":\"fox\",},],}", "lax") != 1) {
        gw_test_fail("comment / trailing comma rejected");
        return 1;
    }
    return 0;
}

/* With no geno.json anywhere, Geno must be invisible: no profile for any kind, and no salt for any
 * fighter (so every netplay identity is unchanged). The suite runs with an empty mods folder. */
static int test_geno_registry_empty_is_inert(void) {
    int k;
    if (gn_reg()->n != 0) {
        gw_log("geno: test note: the boot registry has %d entries (a geno mod is mounted)", gn_reg()->n);
        return 0;
    }
    for (k = 0; k < GN_KINDS; ++k) {
        if (gw_Geno_ProfileForKind(k) != -1) {
            gw_test_fail("kind %d has a profile with no geno.json", k);
            return 1;
        }
    }
    if (gw_Geno_SaltForPlFile("PlKb.dat") != 0 || gw_Geno_SaltForPlFile("PlSh.dat") != 0) {
        gw_test_fail("a salt with no geno.json");
        return 1;
    }
    return 0;
}

/* v1 keys: special_attributes, on_land, subactions (script overlays), hooks.on_land. */
static int test_geno_registry_v1(void) {
    static gn_registry r;
    const gn_profile *p;
    float f;
    const char *t =
        "{\"geno\":1,\"fighters\":[{\"attach\":\"kirby\","
        "\"special_attributes\":[{\"index\":13,\"float\":16.0},{\"offset\":\"0x2C\",\"int\":10},"
        "{\"index\":400,\"int\":1},{\"index\":2}],"
        "\"on_land\":[{\"from\":\"special:4\",\"to\":\"special:6\"},{\"from\":66,\"to\":43,\"keep_frame\":true},"
        "{\"from\":\"nope:1\",\"to\":1}],"
        "\"subactions\":[{\"index\":87,\"words\":[\"0xEC220001\",5,4294967295]},{\"index\":2000,\"words\":[1]},"
        "{\"index\":88,\"words\":[]}],"
        "\"hooks\":{\"on_land\":[\"geno.count_frames:5\"]}}]}";
    memset(&r, 0, sizeof r);
    if (gn_parse_text(&r, t, "v1") != 1) {
        gw_test_fail("v1 entry not parsed");
        return 1;
    }
    p = &r.p[0];
    memcpy(&f, &p->spec_bits[0], 4);
    if (p->nspec != 2 || p->spec_index[0] != 13 || f != 16.0f || p->spec_index[1] != 11 ||
        p->spec_bits[1] != 10) {
        gw_test_fail("special_attributes: n %d, [0] %d=%g, [1] %d=%u", p->nspec, p->spec_index[0], f,
                     p->spec_index[1], p->spec_bits[1]);
        return 1;
    }
    if (p->nland != 2 || p->land_from[0] != GENO_TARGET(GENO_TGT_SPECIAL, 4) ||
        p->land_to[0] != GENO_TARGET(GENO_TGT_SPECIAL, 6) || p->land_from[1] != 66 ||
        p->land_to[1] != (43u | GENO_TGT_KEEP_FRAME)) {
        gw_test_fail("on_land parsed wrong (n %d)", p->nland);
        return 1;
    }
    if (p->nov != 1 || p->ov_anim[0] != 87 || r.nslot != 1 || r.slot_len[0] != 4 ||
        r.pool[r.slot_off[0]] != 0xEC220001u || r.pool[r.slot_off[0] + 1] != 5 ||
        r.pool[r.slot_off[0] + 2] != 0xFFFFFFFFu || r.pool[r.slot_off[0] + 3] != 0) {
        gw_test_fail("subactions: n %d, slots %d, len %d (expected 3 words + End)", p->nov, r.nslot,
                     r.nslot > 0 ? r.slot_len[0] : -1);
        return 1;
    }
    if (p->nhook[GENO_EV_LAND] != 1 || p->hook[GENO_EV_LAND][0] != GENO_HOOK_COUNT_FRAMES ||
        p->hook_arg[GENO_EV_LAND][0] != 5) {
        gw_test_fail("hooks.on_land not parsed");
        return 1;
    }
    return 0;
}

void geno_registry_tests_register(void) {
    gw_test_register("geno_registry_v1", test_geno_registry_v1);
    gw_test_register("geno_registry_parse", test_geno_registry_parse);
    gw_test_register("geno_registry_stable_ids", test_geno_registry_stable_ids);
    gw_test_register("geno_registry_bad_json", test_geno_registry_bad_json);
    gw_test_register("geno_registry_empty_is_inert", test_geno_registry_empty_is_inert);
}
