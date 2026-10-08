/* gw_skins_boot.inc - the skin registry's engine half, included by geno_registry.c (it uses Geno's strict JSON reader
 * and its fighter-target tables). The pure logic is gw_skins_core.h; the format and the rules are
 * docs/mods-packaging.md "Skins" and the workspace brief docs/superpowers/plans/2026-10-08-skins-registry.md.
 *
 * A mod with "kind": "skin" and a "skin" block adds costumes to one fighter. gw_Skins_Build runs once, before the
 * arena is carved (gw_CostumeRegionSize needs the counts), reads every MOUNTING skin mod's mod.json, stats its
 * costume files (never reads them), and hands the rows to the core. Game code and the other shims call the
 * gw_Skins_* functions below (game code without the gw_ prefix).
 *
 * Nothing here is simulation state: costumes are cosmetic, and a skin the other side lacks is shown as the default. */

#include "gw_skins_core.h"
#include <time.h>

extern int gw_SceneLaunch_CKindToFKind(int ck);
extern void *gw_DVDReadFileAlloc(const char *path, uint32_t *out_size);

static int sk_built;
static int sk_mods_read, sk_mods_bad;
static double sk_ms;

static void sk_log_sink(const char *m) { gw_log("%s", m); }

static const char *sk_fighter_label(int fk) {
    static char buf[SK_FK_MAX][24];
    if (fk >= 0 && fk < 27) return gn_vanilla[fk].name;
    if (fk < 0 || fk >= SK_FK_MAX) return "?";
    if (!buf[fk][0]) snprintf(buf[fk], sizeof buf[fk], "fighter %d", fk);
    return buf[fk];
}

/* a relative path inside a mod's files/: no drive, no leading slash, no "..", no backslash, not empty */
static int sk_safe_path(const char *p) {
    size_t n = p ? strlen(p) : 0;
    const char *q;
    if (n == 0 || n >= SK_PATH || p[0] == '/' || strchr(p, '\\') || strchr(p, ':')) return 0;
    for (q = p; *q; ++q) if ((unsigned char) *q < 32) return 0;
    if (strstr(p, "..")) return 0;
    return 1;
}

static int sk_printable(const char *s, size_t minlen, size_t maxlen) {
    size_t n = strlen(s), i;
    if (n < minlen || n > maxlen) return 0;
    for (i = 0; i < n; ++i) if ((unsigned char) s[i] < 32 || (unsigned char) s[i] > 126) return 0;
    return 1;
}

static int sk_file_exists(const char *payload, const char *rel) {
    char path[MAX_PATH], *q;
    FILE *f;
    snprintf(path, sizeof path, "%s\\%s", payload, rel);
    for (q = path + strlen(payload) + 1; *q; ++q) if (*q == '/') *q = '\\';
    f = fopen(path, "rb");
    if (!f) return 0;
    fclose(f);
    return 1;
}

static int sk_strkey(const jdoc *d, int obj, const char *key, char *out, size_t cap, const char *id, const char *what, int required) {
    int v = jd_get(d, obj, key);
    if (v < 0) {
        if (required) sk_say("skins: %s: %s needs \"%s\"", id, what, key);
        out[0] = '\0';
        return required ? -1 : 0;
    }
    if (d->n[v].type != JN_STR || strlen(d->n[v].str) >= cap) {
        sk_say("skins: %s: %s \"%s\" must be a string of at most %d characters", id, what, key, (int) cap - 1);
        return -1;
    }
    snprintf(out, cap, "%s", d->n[v].str);
    return 1;
}

static int sk_known_keys(const jdoc *d, int obj, const char *const *keys, int nkeys, const char *id, const char *what) {
    int c, k;
    for (c = d->n[obj].first; c >= 0; c = d->n[c].next) {
        for (k = 0; k < nkeys && strcmp(d->n[c].key, keys[k]); ++k) {}
        if (k == nkeys) {
            sk_say("skins: %s: %s has a key \"%s\" this build does not know - the mod is skipped", id, what, d->n[c].key);
            return -1;
        }
    }
    return 0;
}

static int sk_int(const jdoc *d, int obj, const char *key, int lo, int hi, int def, const char *id, const char *what, int *out) {
    int v = jd_get(d, obj, key);
    *out = def;
    if (v < 0) return 0;
    if (d->n[v].type != JN_NUM || d->n[v].num != (double) (int) d->n[v].num || d->n[v].num < lo || d->n[v].num > hi) {
        sk_say("skins: %s: %s \"%s\" must be a whole number %d..%d", id, what, key, lo, hi);
        return -1;
    }
    *out = (int) d->n[v].num;
    return 0;
}

/* The FighterKind a target names, or -1 (logged). */
static int sk_resolve_target(const jdoc *d, int t, const char *id) {
    static const char *const keys[] = { "retail", "mex", "geno" };
    int c, n = 0, fk = -1;
    char name[SK_PATH], pl[SK_PATH];
    if (t < 0 || d->n[t].type != JN_OBJ) { sk_say("skins: %s: skin.target must be an object", id); return -1; }
    if (sk_known_keys(d, t, keys, 3, id, "skin.target") < 0) return -1;
    for (c = d->n[t].first; c >= 0; c = d->n[c].next) ++n;
    if (n != 1) { sk_say("skins: %s: skin.target names exactly one of retail, mex, geno", id); return -1; }
    c = d->n[t].first;
    if (d->n[c].type != JN_STR || strlen(d->n[c].str) >= sizeof name) { sk_say("skins: %s: skin.target.%s must be a string", id, d->n[c].key); return -1; }
    snprintf(name, sizeof name, "%s", d->n[c].str);
    if (!strcmp(d->n[c].key, "geno")) {
        int ck = gw_Geno_DefineResolve(name);
        if (ck < 1) { sk_say("skins: %s: Geno define \"%s\" is not installed - skipped", id, name); return -1; }
        if (gw_Geno_DefineBaseKind(ck - 1) >= 0 && gw_Geno_DefineCostumeCountNative(ck - 1) == 0) {
            sk_say("skins: %s: define \"%s\" wears Mario's costumes - its skins are installed as Mario's", id, name);
            return 0;
        }
        return ck - 1;
    }
    if (!strcmp(d->n[c].key, "retail")) {
        gn_target_file(name, pl, sizeof pl);
        fk = pl[0] ? gn_kind_for_file(pl) : -1;
        if (fk < 0 || fk >= 27) { sk_say("skins: %s: \"%s\" is not a retail fighter (mario, fox, captain, ...)", id, name); return -1; }
        return fk;
    }
    /* mex: the fighter's Pl file */
    if (strlen(name) > 4 && _stricmp(name + strlen(name) - 4, ".dat") == 0) snprintf(pl, sizeof pl, "%s", name);
    else snprintf(pl, sizeof pl, "%s.dat", name);
    fk = gn_kind_for_file(pl);
    if (fk < GN_MEX_KIND0) { sk_say("skins: %s: m-ex fighter %s is not installed - skipped", id, pl); return -1; }
    return fk;
}

/* Read one skin mod's mod.json text (id = its folder name, payload = its files/ folder). Returns the number of costumes taken. */
static int sk_read_text(const char *id, const char *payload, const char *text, jdoc *d) {
    static const char *const top_keys[] = { "format", "target", "order", "costumes" };
    static const char *const cos_keys[] = { "name", "file", "joint", "matanim", "team", "like", "kirby_hat", "csp", "stock", "icon", "partner" };
    static const char *const part_keys[] = { "file", "joint", "matanim" };
    char stamp[96], modname[SK_NAME * 2];
    int root, sk, v, c, entry = 0, taken = 0, fk, order, format, ncost = 0;
    int top_hash;
    root = jd_parse(d, text);
    if (root < 0 || d->n[root].type != JN_OBJ) { sk_say("skins: %s: mod.json is not valid JSON (%s)", id, d->err ? d->err : "?"); return 0; }
    sk = jd_get(d, root, "skin");
    if (sk < 0 || d->n[sk].type != JN_OBJ) { sk_say("skins: %s: kind \"skin\" needs a \"skin\" object", id); return 0; }
    stamp[0] = '\0';
    top_hash = jd_get(d, root, "hash");
    if (top_hash >= 0 && d->n[top_hash].type == JN_STR) snprintf(stamp, sizeof stamp, "%s", d->n[top_hash].str);
    if (!stamp[0]) { v = jd_get(d, root, "version"); if (v >= 0 && d->n[v].type == JN_STR) snprintf(stamp, sizeof stamp, "%s", d->n[v].str); }
    modname[0] = '\0';
    v = jd_get(d, root, "name");
    if (v >= 0 && d->n[v].type == JN_STR) snprintf(modname, sizeof modname, "%s", d->n[v].str);
    if (sk_known_keys(d, sk, top_keys, 4, id, "skin") < 0) { return 0; }
    if (sk_int(d, sk, "format", 1, 1000000, 1, id, "skin", &format) < 0) { return 0; }
    if (format != 1) { sk_say("skins: %s: skin format %d is newer than this build reads (1) - skipped", id, format); return 0; }
    if (sk_int(d, sk, "order", -1000000, 1000000, 0, id, "skin", &order) < 0) { return 0; }
    fk = sk_resolve_target(d, jd_get(d, sk, "target"), id);
    if (fk < 0) { return 0; }
    c = jd_get(d, sk, "costumes");
    if (c < 0 || d->n[c].type != JN_ARR || d->n[c].first < 0) { sk_say("skins: %s: skin.costumes must list at least one costume", id); return 0; }
    for (v = d->n[c].first; v >= 0; v = d->n[v].next) ++ncost;
    for (v = d->n[c].first; v >= 0; v = d->n[v].next, ++entry) {
        sk_costume s;
        char what[40];
        int p, bad = 0, tmp;
        snprintf(what, sizeof what, "costume %d", entry + 1);
        memset(&s, 0, sizeof s);
        s.fk = fk;
        snprintf(s.mod, sizeof s.mod, "%s", id);
        s.entry = entry;
        s.order = order;
        s.team = -1;
        s.kirby_hat = -1;
        if (d->n[v].type != JN_OBJ) { sk_say("skins: %s: %s must be an object", id, what); continue; }
        if (sk_known_keys(d, v, cos_keys, 11, id, what) < 0) { continue; }
        if (sk_strkey(d, v, "name", s.name, sizeof s.name, id, what, 0) < 0) bad = 1;
        if (!bad && s.name[0] && !sk_printable(s.name, 1, SK_NAME - 1)) { sk_say("skins: %s: %s name must be printable ASCII, 1..%d characters", id, what, SK_NAME - 1); bad = 1; }
        if (!bad && !s.name[0]) {
            if (modname[0] && ncost == 1) snprintf(s.name, sizeof s.name, "%.23s", modname);
            else if (modname[0]) snprintf(s.name, sizeof s.name, "%.18s %d", modname, entry + 1);
            else snprintf(s.name, sizeof s.name, "%.18s %d", id, entry + 1);
        }
        if (!bad && sk_strkey(d, v, "file", s.file, sizeof s.file, id, what, 1) < 0) bad = 1;
        if (!bad && !sk_safe_path(s.file)) { sk_say("skins: %s: %s file \"%s\" must be a plain relative path inside files/", id, what, s.file); bad = 1; }
        if (!bad && !sk_file_exists(payload, s.file)) { sk_say("skins: %s: %s file %s is not in the mod's files/ - skipped", id, what, s.file); bad = 1; }
        if (!bad && sk_strkey(d, v, "joint", s.joint, sizeof s.joint, id, what, 1) < 0) bad = 1;
        if (!bad && sk_strkey(d, v, "matanim", s.matanim, sizeof s.matanim, id, what, 0) < 0) bad = 1;
        if (!bad) {
            p = jd_get(d, v, "team");
            if (p >= 0) {
                if (d->n[p].type != JN_STR) bad = 1;
                else if (!strcmp(d->n[p].str, "red")) s.team = 0;
                else if (!strcmp(d->n[p].str, "blue")) s.team = 1;
                else if (!strcmp(d->n[p].str, "green")) s.team = 2;
                else bad = 1;
                if (bad) sk_say("skins: %s: %s team must be \"red\", \"blue\" or \"green\"", id, what);
            }
        }
        if (!bad && sk_int(d, v, "like", 0, 254, 0, id, what, &s.like) < 0) bad = 1;
        if (!bad && sk_int(d, v, "kirby_hat", 0, 5, -1, id, what, &tmp) < 0) bad = 1;
        if (!bad) s.kirby_hat = tmp;
        if (!bad) {
            for (tmp = 0; tmp < 2; ++tmp) {
                const char *key = tmp ? "stock" : "csp";
                char *dst = tmp ? s.stock : s.csp;
                if (sk_strkey(d, v, key, dst, SK_PATH, id, what, 0) < 0) { bad = 1; break; }
                if (dst[0] && !sk_safe_path(dst)) { sk_say("skins: %s: %s %s \"%s\" must be a plain relative path", id, what, key, dst); bad = 1; break; }
                if (dst[0] && !sk_file_exists(payload, dst)) { sk_say("skins: %s: %s %s %s is not in the mod's files/ - default art", id, what, key, dst); dst[0] = '\0'; }
            }
        }
        if (!bad) {
            p = jd_get(d, v, "partner");
            if (p >= 0) {
                if (d->n[p].type != JN_OBJ || sk_known_keys(d, p, part_keys, 3, id, "partner") < 0) bad = 1;
                else if (sk_strkey(d, p, "file", s.pfile, sizeof s.pfile, id, "partner", 1) < 0 || !sk_safe_path(s.pfile) ||
                         !sk_file_exists(payload, s.pfile) || sk_strkey(d, p, "joint", s.pjoint, sizeof s.pjoint, id, "partner", 1) < 0 ||
                         sk_strkey(d, p, "matanim", s.pmat, sizeof s.pmat, id, "partner", 0) < 0) {
                    sk_say("skins: %s: %s partner needs file (inside files/), joint, optional matanim", id, what);
                    bad = 1;
                } else s.partner = 1;
            }
        }
        if (bad) continue;
        s.id = sk_identity(id, entry, stamp);
        if (sk_add(&s) == 0) ++taken;
    }
   
    return taken;
}

static int sk_read_mod(int mi, jdoc *d) {
    const char *id = gw_Mods_Id(mi), *dir = gw_Mods_Dir();
    char path[MAX_PATH];
    char *text;
    int n;
    snprintf(path, sizeof path, "%s\\%s\\mod.json", dir, id);
    text = gn_read_file(path);
    if (!text) { sk_say("skins: %s: mod.json cannot be read", id); return 0; }
    n = sk_read_text(id, gw_Mods_PayloadDir(mi), text, d);
    free(text);
    return n;
}

static const char *sk_name_fn(int fk) { return sk_fighter_label(fk); }

/* Build the registry once. base_fn(fk) is the fighter's own costume count (the runtime knows retail, m-ex and Geno counts). */
void gw_Skins_Build(int (*base_fn)(int)) {
    jdoc *d;
    int n, k, costumes = 0, fighters = 0, fk;
    clock_t t0;
    if (sk_built) return;
    sk_built = 1;
    sk_reset();
    sk_sink = sk_log_sink;
    t0 = clock();
    d = (jdoc *) malloc(sizeof *d);
    n = gw_Mods_ActiveCount();
    for (k = 0; d && k < n; ++k) {
        int i = gw_Mods_ActiveAt(k);
        if (strcmp(gw_Mods_Kind(i), "skin") != 0) continue;
        sk_mods_read++;
        if (sk_read_mod(i, d) == 0) sk_mods_bad++;
    }
    free(d);
    sk_finalize(base_fn, sk_name_fn);
    for (fk = 0; fk < SK_FK_MAX; ++fk) if (sk_reg.added[fk] > 0) { fighters++; costumes += sk_reg.added[fk]; }
    sk_ms = (double) (clock() - t0) * 1000.0 / (double) CLOCKS_PER_SEC;
    if (sk_mods_read > 0 || costumes > 0)
        gw_log("skins: %d skin mod(s) read (%d with nothing usable), %d costume(s) on %d fighter(s), %d refused, %.1f ms",
               sk_mods_read, sk_mods_bad, costumes, fighters, sk_reg.refused, sk_ms);
}

/* ---- queries by FighterKind (the ftData side) --------------------------------------------------- */

int gw_Skins_Added(int fk) { return sk_added(fk); }
int gw_Skins_Base(int fk) { return sk_base(fk); }
int gw_Skins_Total(int fk) { return sk_total(fk); }
int gw_Skins_Count(void) { int fk, t = 0; for (fk = 0; fk < SK_FK_MAX; ++fk) t += sk_reg.added[fk]; return t; }

/* Costume c of fighter fk: which 0 = file, 1 = joint symbol, 2 = matanim symbol (NULL = none). A generated partner row that
 * has no files of its own answers NULL for file: the installer then copies the partner's costume 0. */
const char *gw_Skins_String(int fk, int c, int which) {
    const sk_costume *r = sk_row(fk, c);
    const char *s;
    if (!r) return NULL;
    s = which == 0 ? r->file : which == 1 ? r->joint : r->matanim;
    return s[0] ? s : NULL;
}
int gw_Skins_IsRow(int fk, int c) { return sk_row(fk, c) != NULL; }

/* The part-visibility row a skin costume copies (an original costume's index), or -1: not a skin. */
int gw_Skins_Like(int fk, int c) { return sk_like(fk, c); }
int gw_Skins_KirbyRow(int fk, int c) { return sk_kirby_row(fk, c); }

/* ---- queries by CharacterKind (the menus, the netplay layer) -------------------------------------- */

static int sk_fk(int ck) { return ck >= 0 ? gw_SceneLaunch_CKindToFKind(ck) : -1; }

/* The fighter's costume count with its skins, or 0 when it has none (the caller keeps its own answer). */
int gw_Skins_CostumeCountCK(int ck) {
    int fk = sk_fk(ck);
    return (fk >= 0 && sk_added(fk) > 0) ? sk_total(fk) : 0;
}
int gw_Skins_TeamCostumeCK(int ck, int team) {
    int fk = sk_fk(ck);
    return fk >= 0 ? sk_team(fk, team) : -1;
}
int gw_Skins_NameCK(int ck, int costume, char *out, int cap) {
    int fk = sk_fk(ck);
    const sk_costume *r = fk >= 0 ? sk_row(fk, costume) : NULL;
    if (!r || !out || cap <= 0) return 0;
    snprintf(out, (size_t) cap, "%s", r->name);
    return 1;
}
int gw_Skins_ToWireCK(int ck, int color) {
    int fk = sk_fk(ck);
    return fk >= 0 ? sk_to_wire(fk, color) : color;
}
int gw_Skins_FromWireCK(int ck, int wire) {
    int fk = sk_fk(ck);
    if (fk < 0) return (wire >= 0 && wire < SK_CAP) ? wire : 0;
    return sk_from_wire(fk, wire);
}

/* ---- art ---------------------------------------------------------------------------------------- */

static const char *sk_art_file(int ck, int what, int costume) {
    int fk = sk_fk(ck);
    const sk_costume *r = fk >= 0 ? sk_row(fk, costume) : NULL;
    if (!r || r->partner_row) return NULL;
    if (what == 1) return r->csp[0] ? r->csp : NULL;
    if (what == 2) return r->stock[0] ? r->stock : NULL;
    return NULL;
}
int gw_Skins_HasArtCK(int ck, int what, int costume) { return sk_art_file(ck, what, costume) != NULL; }

/* The portrait (what 1) as a kit texture, or -1. */
int gw_Skins_ArtTexCK(int ck, int what, int costume) {
    extern int gw_Kit_TexAddGxtex(const char *key, const uint8_t *blob, size_t len);
    extern int gw_Kit_TexHsdFind(const char *key);
    const char *file = sk_art_file(ck, what, costume);
    char key[SK_PATH + 16];
    uint32_t len = 0;
    void *raw;
    int tex;
    if (!file) return -1;
    snprintf(key, sizeof key, "sk:%d:%s", what, file);
    tex = gw_Kit_TexHsdFind(key);
    if (tex >= 0) return tex;
    raw = gw_DVDReadFileAlloc(file, &len);
    if (!raw) { gw_log("skins: art file %s could not be read - default art", file); return -1; }
    tex = gw_Kit_TexAddGxtex(key, (const uint8_t *) raw, len);
    if (tex < 0) gw_log("skins: art file %s is not a usable .gxtex - default art", file);
    else gw_log("skins: %s from the skin: %s as kit texture %d", what == 1 ? "portrait" : "stock icon", file, tex);
    free(raw);
    return tex;
}
/* A gxtex handle (the HUD stock icon builds its own image from it); the caller closes it. -1 as above. */
int gw_Skins_ArtOpenCK(int ck, int what, int costume) {
    extern int gw_GxTex_OpenBlob(const unsigned char *blob, uint32_t len, const char *label);
    const char *file = sk_art_file(ck, what, costume);
    uint32_t len = 0;
    void *raw;
    int h;
    if (!file) return -1;
    raw = gw_DVDReadFileAlloc(file, &len);
    if (!raw) return -1;
    h = gw_GxTex_OpenBlob((const unsigned char *) raw, len, file);
    free(raw);
    return h;
}

/* ---- test hooks ---------------------------------------------------------------------------------- */

/* Rebuild from the current mounted mods (tests only; the engine builds once). */
void gw_Skins_RebuildForTest(int (*base_fn)(int)) {
    sk_built = 0;
    sk_mods_read = sk_mods_bad = 0;
    gw_Skins_Build(base_fn);
}
int gw_Skins_BuiltForTest(void) { return sk_built; }
