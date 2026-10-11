/* gw_nucleus_loose.h - installing a Nucleus post that is a plain override of a disc file: a stage skin (a retail versus stage's Gr*.dat), an effect file
 * (Ef*Data.dat), one of four menu files, an audio file. Pure C over stdio / the OS directory calls, header-only, no game types: the native test drives it with
 * synthetic archives, the engine (gw_nucleus_engine.inc) feeds it what it downloaded.
 *
 * What it does, per post: identifies each file by its CONTENT (an HSD archive's public symbols are compared with the game's own copy of the file it would
 * replace, read through a callback; a file with the wrong structure is refused, not installed), groups the files into mods (one per stage, one for effects, one
 * for menus, one for audio), writes each as mods/nucleus-<post>-<group>/ {mod.json, files/<disc path>} through a staging folder that is swapped in whole (a crash
 * leaves the old mod or none, never half of one), records every other mod that replaces the same file in "conflicts" and turns those off (one active per file;
 * the newest install wins and the screen says what moved), and keeps the post link and author in mod.json (the permission's condition).
 *
 * Nothing here weakens the netplay identity: a stage's identity hash reads its Gr file's CONTENT (gw_mexid.c), so a skinned stage is a different stage online
 * until both players have the same one. */
#ifndef GW_NUCLEUS_LOOSE_H
#define GW_NUCLEUS_LOOSE_H

#include "gw_nucleus_install.h"

/* ---- HSD archive symbols ------------------------------------------------------------------------------------ */

typedef struct {
    uint32_t *h;                 /* FNV-1a of every public and external symbol name */
    uint8_t *tex;                /* 1: that symbol is a texture (_image / _tlut / _tlut_desc / _image_desc): a skin may change those */
    int n;
    int map_head, coll_data, ground_param;
} nc_symset;

static void nc_symset_free(nc_symset *s) { free(s->h); free(s->tex); memset(s, 0, sizeof *s); }

static uint32_t nc_fnv32(const char *s) {
    uint32_t h = 2166136261u;
    for (; *s; ++s) { h ^= (unsigned char) *s; h *= 16777619u; }
    return h;
}
static uint64_t nc_fnv64(const uint8_t *p, size_t n) {
    uint64_t h = 1469598103934665603ull;
    size_t i;
    for (i = 0; i < n; ++i) { h ^= p[i]; h *= 1099511628211ull; }
    return h;
}

static int nc_sym_is_tex(const char *s) {
    size_t l = strlen(s);
    return (l > 6 && !strcmp(s + l - 6, "_image")) || (l > 5 && !strcmp(s + l - 5, "_tlut")) || (l > 10 && !strcmp(s + l - 10, "_tlut_desc")) ||
           (l > 11 && !strcmp(s + l - 11, "_image_desc"));
}

/* Reads the public and external symbol names of an HSD archive. 0 ok, -1 with err. */
static int nc_dat_symbols(const uint8_t *raw, size_t n, nc_symset *out, char *err, size_t ecap) {
    uint32_t fsize, dsize, nreloc, npub, next, i, total;
    uint64_t o_pub, o_sym;
    memset(out, 0, sizeof *out);
    if (n < 0x20) { snprintf(err, ecap, "too small to be a DAT"); return -1; }
    fsize = nc_be32(raw); dsize = nc_be32(raw + 4); nreloc = nc_be32(raw + 8); npub = nc_be32(raw + 12); next = nc_be32(raw + 16);
    if (fsize < 0x20 || fsize > n || npub > 8192 || next > 8192 || nreloc > 4000000) { snprintf(err, ecap, "not an HSD archive"); return -1; }
    o_pub = 0x20 + (uint64_t) dsize + (uint64_t) nreloc * 4;
    o_sym = o_pub + (uint64_t) (npub + next) * 8;
    if (o_sym > fsize) { snprintf(err, ecap, "archive tables run past the end"); return -1; }
    total = npub + next;
    out->h = (uint32_t *) calloc(total ? total : 1, sizeof(uint32_t));
    out->tex = (uint8_t *) calloc(total ? total : 1, 1);
    if (!out->h || !out->tex) { nc_symset_free(out); snprintf(err, ecap, "out of memory"); return -1; }
    for (i = 0; i < total; ++i) {
        uint32_t so = nc_be32(raw + o_pub + (uint64_t) i * 8 + 4);
        uint64_t at = o_sym + so, e;
        char sym[200];
        if (at >= fsize) continue;
        for (e = at; e < fsize && e - at < sizeof sym - 1 && raw[e]; ++e) sym[e - at] = (char) raw[e];
        if (e >= fsize || raw[e]) continue;
        sym[e - at] = '\0';
        out->h[out->n] = nc_fnv32(sym);
        out->tex[out->n] = (uint8_t) nc_sym_is_tex(sym);
        ++out->n;
        if (!strcmp(sym, "map_head")) out->map_head = 1;
        else if (!strcmp(sym, "coll_data")) out->coll_data = 1;
        else if (!strcmp(sym, "grGroundParam")) out->ground_param = 1;
    }
    return 0;
}

static int nc_symset_has(const nc_symset *s, uint32_t h) {
    int i;
    for (i = 0; i < s->n; ++i) if (s->h[i] == h) return 1;
    return 0;
}

/* how alike two archives are: |common names| * 100 / |names in either|, and how many of the reference's non-texture names the candidate lacks */
static int nc_symset_similarity(const nc_symset *cand, const nc_symset *ref, int *missing_structure) {
    int i, common = 0, miss = 0, uni;
    for (i = 0; i < ref->n; ++i) {
        if (nc_symset_has(cand, ref->h[i])) ++common;
        else if (!ref->tex[i]) ++miss;
    }
    uni = cand->n + ref->n - common;
    if (missing_structure) *missing_structure = miss;
    return uni > 0 ? common * 100 / uni : 0;
}

/* ---- the environment: the game's own files ------------------------------------------------------------------- */

typedef struct {
    void *user;
    /* the game's current copy of a disc file ("GrOp.dat"): 0 and a malloc'd buffer, -1 when the disc has no such file */
    int (*ref)(void *user, const char *disc_path, uint8_t **buf, size_t *len);
    /* does the disc have this file: 1 and its size, else 0 */
    int (*exists)(void *user, const char *disc_path, size_t *len);
} nc_loose_env;

typedef struct { int file_id, cat; const char *filename, *stem; const uint8_t *data; size_t len; } nc_in_file;
typedef struct { int id; const char *title, *author, *page, *updated, *installed_at, *stage; } nc_post;

#define NC_SIM_MIN 55            /* % of symbol names two archives share for one to be a skin of the other */
static const char *const nc_stage_probe[] = { "GrNBa", "GrNLa", "GrSt", "GrOp", "GrPs", "GrIz" };

/* ---- planning ---------------------------------------------------------------------------------------------- */

typedef struct {
    int in;                       /* index into the input files */
    char group[40], kind[8], target[3][64];
    int ntarget;
    char note[200];
    char what[48];                /* "Battlefield", "EfFxData.dat", ... for the message */
} nc_plan;

/* Identify one file by content. 0 planned, -1 refused (err says why in plain words). */
static int nc_loose_plan_file(const nc_in_file *f, const nc_loose_env *env, nc_plan *p, char *err, size_t ecap) {
    nc_symset cs;
    char e2[120];
    uint8_t *rb = NULL;
    size_t rl = 0;
    memset(p, 0, sizeof *p);
    p->note[0] = '\0';
    if (f->cat == NC_FC_AUDIO) {
        const char *bn = nc_basename(f->filename);
        char path[100];
        size_t sz = 0;
        if (nc_has_ext(bn, ".hps")) {
            if (f->len < 0x20 || memcmp(f->data, " HALPST", 7) != 0) { snprintf(err, ecap, "%s is not an HPS music file", bn); return -1; }
        } else if (f->len < 0x20) { snprintf(err, ecap, "%s is too small to be a sound bank", bn); return -1; }
        snprintf(path, sizeof path, "audio/%s", bn);
        if (!env->exists(env->user, path, &sz)) {
            snprintf(path, sizeof path, "audio/us/%s", bn);
            if (!env->exists(env->user, path, &sz)) { snprintf(err, ecap, "%s is not one of the game's audio files", bn); return -1; }
        }
        snprintf(p->group, sizeof p->group, "audio");
        snprintf(p->kind, sizeof p->kind, "misc");
        snprintf(p->target[0], sizeof p->target[0], "%s", path);
        p->ntarget = 1;
        snprintf(p->what, sizeof p->what, "%.40s", bn);
        return 0;
    }
    if (f->cat != NC_FC_STAGE && f->cat != NC_FC_EFFECT && f->cat != NC_FC_MENU) { snprintf(err, ecap, "%s", nc_cat_why_not(f->cat, 0)); return -1; }
    if (nc_dat_symbols(f->data, f->len, &cs, e2, sizeof e2) < 0) { snprintf(err, ecap, "%s: %s", nc_basename(f->filename), e2); return -1; }
    if (f->cat == NC_FC_STAGE) {
        char stem[24], best[24], ext[5];
        int i, sim = 0, bsim = -1, miss = 0, hint_sim = -1;
        nc_symset rs;
        if (!cs.map_head || !cs.coll_data || !cs.ground_param) { nc_symset_free(&cs); snprintf(err, ecap, "%s is not a stage file (no map_head / coll_data / grGroundParam)", nc_basename(f->filename)); return -1; }
        stem[0] = '\0';
        if (f->stem && f->stem[0]) { const char *dot = strchr(f->stem, '.'); snprintf(stem, sizeof stem, "%.*s", (int) (dot ? dot - f->stem : strlen(f->stem)), f->stem); }
        snprintf(ext, sizeof ext, "%s", f->stem && nc_has_ext(f->stem, ".usd") ? ".usd" : ".dat");
        best[0] = '\0';
        for (i = -1; i < (int) (sizeof nc_stage_probe / sizeof nc_stage_probe[0]); ++i) {
            char rp[40], e3[80];
            const char *cand = i < 0 ? stem : nc_stage_probe[i];
            if (!cand[0] || (i >= 0 && stem[0] && !strcmp(cand, stem))) continue;
            snprintf(rp, sizeof rp, "%s.dat", cand);
            if (env->ref(env->user, rp, &rb, &rl) != 0) continue;
            if (nc_dat_symbols(rb, rl, &rs, e3, sizeof e3) < 0) { free(rb); continue; }
            free(rb); rb = NULL;
            sim = nc_symset_similarity(&cs, &rs, &miss);
            nc_symset_free(&rs);
            if (i < 0) hint_sim = (miss == 0) ? sim : 0;
            if (miss == 0 && sim > bsim) { bsim = sim; snprintf(best, sizeof best, "%s", cand); }
            if (i < 0 && miss == 0 && sim >= NC_SIM_MIN) break;      /* the name told the truth: no need to read the others */
        }
        nc_symset_free(&cs);
        if (bsim < NC_SIM_MIN) {
            if (stem[0]) snprintf(err, ecap, "%s does not look like %s (it matches %d%% of the game's own file)", nc_basename(f->filename), nc_stage_name(stem), hint_sim < 0 ? 0 : hint_sim);
            else snprintf(err, ecap, "%s: could not tell which stage this is", nc_basename(f->filename));
            return -1;
        }
        if (stem[0] && strcmp(stem, best) != 0) snprintf(p->note, sizeof p->note, "%s is named %s but is really %s - installed as %s", nc_basename(f->filename), stem, nc_stage_name(best), nc_stage_name(best));
        {
            char fam[24];
            nc_stage_family(best, fam, sizeof fam);
            for (i = 0; fam[i]; ++i) fam[i] = (char) tolower((unsigned char) fam[i]);
            snprintf(p->group, sizeof p->group, "stage-%s", fam);
        }
        snprintf(p->kind, sizeof p->kind, "stage");
        snprintf(p->target[0], sizeof p->target[0], "%s%s", best, ext);
        p->ntarget = 1;
        snprintf(p->what, sizeof p->what, "%.47s", nc_stage_name(best));
        return 0;
    }
    /* an effect or a menu file: the game's own copy of the same name has to exist and be this file's structure */
    {
        char target[40];
        int sim, miss = 0;
        nc_symset rs;
        char e3[80];
        snprintf(target, sizeof target, "%s", f->stem);
        if (!target[0]) { nc_symset_free(&cs); snprintf(err, ecap, "%s: not a game file name", nc_basename(f->filename)); return -1; }
        if (env->ref(env->user, target, &rb, &rl) != 0) { nc_symset_free(&cs); snprintf(err, ecap, "this game has no %s to replace", target); return -1; }
        if (nc_dat_symbols(rb, rl, &rs, e3, sizeof e3) < 0) { free(rb); nc_symset_free(&cs); snprintf(err, ecap, "the game's %s could not be read", target); return -1; }
        free(rb);
        sim = nc_symset_similarity(&cs, &rs, &miss);
        nc_symset_free(&rs);
        nc_symset_free(&cs);
        if (miss > 0) { snprintf(err, ecap, "%s lacks %d part%s the game's own %s has", nc_basename(f->filename), miss, miss == 1 ? "" : "s", target); return -1; }
        if (sim < NC_SIM_MIN) { snprintf(err, ecap, "%s does not look like the game's %s (%d%% alike)", nc_basename(f->filename), target, sim); return -1; }
        snprintf(p->group, sizeof p->group, "%s", f->cat == NC_FC_EFFECT ? "fx" : "menu");
        snprintf(p->kind, sizeof p->kind, "misc");
        snprintf(p->target[0], sizeof p->target[0], "%s", target);
        p->ntarget = 1;
        snprintf(p->what, sizeof p->what, "%.47s", target);
        if (f->cat == NC_FC_MENU) snprintf(p->note, sizeof p->note, "menu files are global: only one mod can replace %s", target);
    }
    return 0;
}

/* ---- mods folder helpers ------------------------------------------------------------------------------------ */

/* the mod folders in the mods dir (no dot folders), at most cap, names up to 63 characters */
static int nc_list_subdirs(const char *mods_dir, char names[][64], int cap) {
    int n = 0;
#ifdef _WIN32
    char pat[600];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    snprintf(pat, sizeof pat, "%s\\*", mods_dir);
    h = FindFirstFileA(pat, &fd);
    if (h == INVALID_HANDLE_VALUE) return 0;
    do {
        if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) || fd.cFileName[0] == '.' || n >= cap || strlen(fd.cFileName) > 63) continue;
        snprintf(names[n++], 64, "%s", fd.cFileName);
    } while (FindNextFileA(h, &fd));
    FindClose(h);
#else
    DIR *d = opendir(mods_dir);
    struct dirent *e;
    if (!d) return 0;
    while ((e = readdir(d)) != NULL && n < cap) {
        char path[700];
        if (e->d_name[0] == '.' || strlen(e->d_name) > 63) continue;
        snprintf(path, sizeof path, "%s/%s", mods_dir, e->d_name);
        if (nc_is_dir(path)) snprintf(names[n++], 64, "%s", e->d_name);
    }
    closedir(d);
#endif
    return n;
}

static int nc_file_exists(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (!fp) return 0;
    fclose(fp);
    return 1;
}

/* the disc paths a mod folder replaces: nucleus.targets in its mod.json, else (a mod from elsewhere) none known. n entries of 64. */
static int nc_mod_targets(const char *mods_dir, const char *id, char targets[][64], int cap) {
    char path[700];
    char *text;
    nj_doc d;
    int root, nu, ta, e, n = 0;
    snprintf(path, sizeof path, "%s/%s/mod.json", mods_dir, id);
    nc_fix_slashes(path);
    text = nc_read_all(path, NULL);
    if (!text) return 0;
    root = nj_parse(&d, text);
    if (root >= 0) {
        nu = nj_get(&d, root, "nucleus");
        ta = nu >= 0 ? nj_get(&d, nu, "targets") : -1;
        if (ta >= 0 && d.n[ta].type == NJ_ARR)
            for (e = d.n[ta].first; e >= 0 && n < cap; e = d.n[e].next) if (d.n[e].type == NJ_STR) snprintf(targets[n++], 64, "%s", d.pool + d.n[e].str);
    }
    nj_free(&d);
    free(text);
    return n;
}

/* does mod `id` list `other` in its conflicts? */
static int nc_mod_lists_conflict(const char *mods_dir, const char *id, const char *other) {
    char path[700];
    char *text;
    nj_doc d;
    int root, cf, e, hit = 0;
    snprintf(path, sizeof path, "%s/%s/mod.json", mods_dir, id);
    nc_fix_slashes(path);
    text = nc_read_all(path, NULL);
    if (!text) return 0;
    root = nj_parse(&d, text);
    cf = root >= 0 ? nj_get(&d, root, "conflicts") : -1;
    if (cf >= 0 && d.n[cf].type == NJ_ARR) for (e = d.n[cf].first; e >= 0; e = d.n[e].next) if (d.n[e].type == NJ_STR && nc_ieq(d.pool + d.n[e].str, other)) hit = 1;
    nj_free(&d);
    free(text);
    return hit;
}

/* the other mod folders that replace any of `targets` (by their mod.json targets, or a files/<target> of their own) or that list `id` as a conflict */
static int nc_find_conflicts(const char *mods_dir, const char *id, char targets[][64], int nt, char out[][64], int cap) {
    static char names[256][64];
    int n = nc_list_subdirs(mods_dir, names, 256), i, k, j, c = 0;
    for (i = 0; i < n && c < cap; ++i) {
        char their[16][64];
        int nth, hit = 0;
        if (nc_ieq(names[i], id) || !strncmp(names[i], ".staging", 8)) continue;
        nth = nc_mod_targets(mods_dir, names[i], their, 16);
        for (k = 0; k < nt && !hit; ++k) {
            char p[800];
            for (j = 0; j < nth; ++j) if (nc_ieq(their[j], targets[k])) hit = 1;
            snprintf(p, sizeof p, "%s/%s/files/%s", mods_dir, names[i], targets[k]);
            nc_fix_slashes(p);
            if (!hit && nc_file_exists(p)) hit = 1;
        }
        if (!hit && nc_mod_lists_conflict(mods_dir, names[i], id)) hit = 1;
        if (hit) snprintf(out[c++], 64, "%s", names[i]);
    }
    return c;
}

/* 1 when the mod is in the next-boot set (no enabled.txt: every mod is) */
static int nc_enabled_query(const char *mods_dir, const char *id) {
    char path[600], *text;
    const char *p;
    size_t idl = strlen(id), n = 0;
    int on = 0;
    snprintf(path, sizeof path, "%s/enabled.txt", mods_dir);
    nc_fix_slashes(path);
    text = nc_read_all(path, &n);
    if (!text) return 1;
    for (p = text; *p; ) {
        const char *e = strchr(p, '\n');
        size_t ll = e ? (size_t) (e - p) : strlen(p), tl = ll;
        while (tl && (p[tl - 1] == '\r' || p[tl - 1] == ' ')) --tl;
        if (tl == idl && nc_ieqn(p, id, idl)) on = 1;
        p = e ? e + 1 : p + ll;
    }
    free(text);
    return on;
}

/* enabled.txt does not exist: every mod is on. Write it with every mod folder except `except` (so one can be turned off). */
static int nc_enabled_materialize(const char *mods_dir, char except[][64], int nex) {
    static char names[256][64];
    char path[600], *buf;
    int n = nc_list_subdirs(mods_dir, names, 256), i, k, rc;
    size_t o = 0;
    buf = (char *) malloc((size_t) (n + 1) * 66);
    if (!buf) return -1;
    for (i = 0; i < n; ++i) {
        int skip = !nc_ieq(names[i], "targettest") ? 0 : 1;
        for (k = 0; k < nex; ++k) if (nc_ieq(names[i], except[k])) skip = 1;
        if (skip) continue;
        o += (size_t) snprintf(buf + o, 66, "%s\n", names[i]);
    }
    snprintf(path, sizeof path, "%s/enabled.txt", mods_dir);
    nc_fix_slashes(path);
    rc = nc_write_file_atomic(path, buf, o);
    free(buf);
    return rc;
}

/* Turn `id` on or off for the next boot. Turning on also turns off every mod that conflicts with it (same disc file); returns how many OTHER mods were turned
 * off, names in off[] (cap), or -1 when enabled.txt could not be written. */
static int nc_loose_set_enabled(const char *mods_dir, const char *id, int on, char off[][64], int cap) {
    char targets[16][64], conf[16][64], path[600];
    int nt = nc_mod_targets(mods_dir, id, targets, 16), nc = 0, i, k = 0;
    snprintf(path, sizeof path, "%s/enabled.txt", mods_dir);
    nc_fix_slashes(path);
    if (on) nc = nc_find_conflicts(mods_dir, id, targets, nt, conf, 16);
    if (!nc_file_exists(path)) {
        if (on && nc == 0) return 0;                    /* everything is on already */
        if (on) { if (nc_enabled_materialize(mods_dir, conf, nc) != 0) return -1; for (i = 0; i < nc && i < cap; ++i) snprintf(off[i], 64, "%s", conf[i]); return nc; }
        {   char ex[1][64];
            snprintf(ex[0], 64, "%s", id);
            if (nc_enabled_materialize(mods_dir, ex, 1) != 0) return -1;
            return 0;
        }                                      /* the conflicting ones were left out of the new file */
    }
    if (on) {
        if (!nc_enabled_query(mods_dir, id) && nc_enabled_edit(mods_dir, id, 1) != 0) return -1;
        for (i = 0; i < nc; ++i) if (nc_enabled_query(mods_dir, conf[i])) { if (nc_enabled_edit(mods_dir, conf[i], 0) != 0) return -1; if (k < cap) snprintf(off[k], 64, "%s", conf[i]); ++k; }
        return k;
    }
    return nc_enabled_edit(mods_dir, id, 0) == 0 ? 0 : -1;
}

/* ---- mod.json for a loose mod ---------------------------------------------------------------------------------- */

static void nc_loose_json(nj_buf *b, const char *mod_id, const char *kind, const char *cat, const nc_post *post, const nc_plan *pl, const nc_in_file *in, int np,
                          char conflicts[][64], int ncf, const char *name, const char *stage_name) {
    int i, first = 1;
    char desc[360];
    snprintf(desc, sizeof desc, "By %s. From SSBM Nucleus: %s/post/%d.%s", post->author[0] ? post->author : "unknown", NC_SITE, post->id,
             !strcmp(cat, "stage") ? " Online, both players need the same skin for the stage to be offered." : !strcmp(cat, "menu") ? " Menu files are global: one active per file." : "");
    nj_puts(b, "{\n  \"id\": "); nj_qstr(b, mod_id);
    nj_puts(b, ",\n  \"name\": "); nj_qstr(b, name);
    nj_puts(b, ",\n  \"version\": "); nj_qstr(b, post->updated);
    nj_puts(b, ",\n  \"kind\": "); nj_qstr(b, kind);
    nj_puts(b, ",\n  \"authors\": "); nj_qstr(b, post->author);
    nj_puts(b, ",\n  \"description\": "); nj_qstr(b, desc);
    nj_puts(b, ",\n  \"conflicts\": [");
    for (i = 0; i < ncf; ++i) { if (i) nj_puts(b, ", "); nj_qstr(b, conflicts[i]); }
    nj_puts(b, "],\n  \"source\": \"nucleus\",\n  \"nucleus\": {\n    \"id\": ");
    nj_printf(b, "%d,\n    \"author\": ", post->id); nj_qstr(b, post->author);
    nj_puts(b, ",\n    \"page_url\": "); nj_qstr(b, post->page);
    nj_puts(b, ",\n    \"updated_at\": "); nj_qstr(b, post->updated);
    nj_puts(b, ",\n    \"installed_at\": "); nj_qstr(b, post->installed_at);
    nj_puts(b, ",\n    \"category\": "); nj_qstr(b, cat);
    nj_puts(b, ",\n    \"stage\": "); nj_qstr(b, stage_name);
    nj_puts(b, ",\n    \"targets\": [");
    for (i = 0; i < np; ++i) { int t; for (t = 0; t < pl[i].ntarget; ++t) { nj_puts(b, first ? "" : ", "); nj_qstr(b, pl[i].target[t]); first = 0; } }
    nj_puts(b, "],\n    \"files\": [");
    for (i = 0; i < np; ++i) nj_printf(b, "%s{\"id\": %d, \"size\": %u, \"fnv64\": \"%016llx\"}", i ? ", " : "", in[pl[i].in].file_id, (unsigned) in[pl[i].in].len,
                                       (unsigned long long) nc_fnv64(in[pl[i].in].data, in[pl[i].in].len));
    nj_puts(b, "]\n  }\n}\n");
}

/* ---- the install ---------------------------------------------------------------------------------------------- */

typedef struct {
    int installed;                 /* mods written */
    int files;                     /* files written */
    int refused;                   /* files refused */
    char first_err[200];
    char turned_off[4][64]; int nturned_off;
    char mods[8][64]; int nmods;
} nc_loose_result;

/* Installs the post's files. 0 when at least one mod was written. msg: one plain sentence for the screen. log: optional one-line-per-event sink. */
static int nc_loose_install(const char *mods_dir, const nc_post *post, const nc_in_file *in, int nin, const nc_loose_env *env, nc_loose_result *res,
                            char *msg, size_t mcap, void (*logf)(const char *line)) {
    nc_plan *pl = (nc_plan *) calloc((size_t) (nin > 0 ? nin : 1), sizeof *pl);
    int *done = (int *) calloc((size_t) (nin > 0 ? nin : 1), sizeof(int));
    int i, g, nplanned = 0;
    char line[300];
    memset(res, 0, sizeof *res);
    if (!pl || !done) { free(pl); free(done); snprintf(msg, mcap, "Out of memory."); return -1; }
    for (i = 0; i < nin; ++i) {
        char err[200];
        nc_plan one;
        if (nc_loose_plan_file(&in[i], env, &one, err, sizeof err) == 0) {
            one.in = i;
            pl[nplanned++] = one;
            if (one.note[0] && logf) { snprintf(line, sizeof line, "install %d: %s", post->id, one.note); logf(line); }
        } else {
            ++res->refused;
            if (!res->first_err[0]) snprintf(res->first_err, sizeof res->first_err, "%s", err);
            if (logf) { snprintf(line, sizeof line, "install %d: file %d (%s) refused: %s", post->id, in[i].file_id, in[i].filename, err); logf(line); }
        }
    }
    for (g = 0; g < nplanned; ++g) {
        char mod_id[96], stage_dir[620], final_dir[620], tmp[760], conf[16][64], title[100], grp_name[48], cat[8];
        int members[64], nm = 0, k, nf, t;
        nc_plan sub[64];
        nj_buf b;
        if (done[g]) continue;
        for (k = g; k < nplanned && nm < 64; ++k) if (!done[k] && !strcmp(pl[k].group, pl[g].group)) { members[nm] = k; sub[nm] = pl[k]; ++nm; done[k] = 1; }
        /* stage skins for the prefix-named stages (GrPs ...): the game reads the .usd on an American disc, so the .dat is mirrored there unless the post has one */
        for (k = 0; k < nm; ++k) {
            const char *tg = sub[k].target[0];
            size_t l = strlen(tg);
            if (!strcmp(sub[k].kind, "stage") && l > 4 && !strcmp(tg + l - 4, ".dat")) {
                char stem[24];
                int has = 0, j;
                snprintf(stem, sizeof stem, "%.*s", (int) (l - 4), tg);
                if (nc_stage_has_usd(stem)) {
                    for (j = 0; j < nm; ++j) { char want[70]; snprintf(want, sizeof want, "%s.usd", stem); if (!strcmp(sub[j].target[0], want)) has = 1; }
                    if (!has) snprintf(sub[k].target[sub[k].ntarget++], 64, "%s.usd", stem);
                }
            }
        }
        snprintf(cat, sizeof cat, "%s", !strncmp(pl[g].group, "stage", 5) ? "stage" : pl[g].group);
        snprintf(mod_id, sizeof mod_id, "nucleus-%d-%s", post->id, pl[g].group);
        snprintf(grp_name, sizeof grp_name, "%s", sub[0].what);
        if (!strcmp(cat, "stage")) snprintf(title, sizeof title, "%.60s", post->title[0] ? post->title : "Stage skin");
        else snprintf(title, sizeof title, "%.60s", post->title[0] ? post->title : "Nucleus mod");
        if (!strcmp(cat, "stage") && !strstr(title, grp_name)) { size_t tl = strlen(title); snprintf(title + tl, sizeof title - tl, " (%.28s)", grp_name); }
        snprintf(stage_dir, sizeof stage_dir, "%s/.staging-%s", mods_dir, mod_id);
        snprintf(final_dir, sizeof final_dir, "%s/%s", mods_dir, mod_id);
        nc_fix_slashes(stage_dir); nc_fix_slashes(final_dir);
        nc_rmtree(stage_dir);
        nf = 0;
        {   /* write the payload */
            int bad = 0;
            for (k = 0; k < nm && !bad; ++k) {
                const nc_in_file *f = &in[sub[k].in];
                for (t = 0; t < sub[k].ntarget; ++t) {
                    char *sl;
                    snprintf(tmp, sizeof tmp, "%s/files/%s", stage_dir, sub[k].target[t]);
                    nc_fix_slashes(tmp);
                    sl = strrchr(tmp, '\\'); if (!sl) sl = strrchr(tmp, '/');
                    if (sl) { char d[760]; snprintf(d, sizeof d, "%.*s", (int) (sl - tmp), tmp); nc_mkdirs(d); }
                    if (nc_write_file(tmp, f->data, f->len) != 0) { bad = 1; break; }
                    ++nf;
                }
            }
            if (bad) { nc_rmtree(stage_dir); if (!res->first_err[0]) snprintf(res->first_err, sizeof res->first_err, "Cannot write to the mods folder."); continue; }
        }
        {
            char targets[16][64];
            int nt = 0, ncf;
            for (k = 0; k < nm; ++k) for (t = 0; t < sub[k].ntarget && nt < 16; ++t) snprintf(targets[nt++], 64, "%s", sub[k].target[t]);
            ncf = nc_find_conflicts(mods_dir, mod_id, targets, nt, conf, 16);
            memset(&b, 0, sizeof b);
            nc_loose_json(&b, mod_id, sub[0].kind, cat, post, sub, in, nm, conf, ncf, title, !strcmp(cat, "stage") ? grp_name : "");
            snprintf(tmp, sizeof tmp, "%s/mod.json", stage_dir); nc_fix_slashes(tmp);
            if (b.bad || nc_write_file(tmp, b.s, b.n) != 0) { free(b.s); nc_rmtree(stage_dir); if (!res->first_err[0]) snprintf(res->first_err, sizeof res->first_err, "Cannot write mod.json."); continue; }
            free(b.s);
            if (nc_swap_dir(stage_dir, final_dir) != 0) {
                nc_rmtree(stage_dir);
                snprintf(res->first_err, sizeof res->first_err, "Could not replace %s (the game may be using it). Restart and try again.", mod_id);
                continue;
            }
            {   /* on for the next boot; whatever else replaces the same files goes off (the newest install wins) */
                char off[8][64];
                int noff = nc_loose_set_enabled(mods_dir, mod_id, 1, off, 8), x;
                if (noff < 0 && logf) { snprintf(line, sizeof line, "install %d: could not update enabled.txt", post->id); logf(line); }
                if (noff == 0 && ncf > 0) { /* enabled.txt was just written without them */ }
                for (x = 0; x < ncf && res->nturned_off < 4; ++x) snprintf(res->turned_off[res->nturned_off++], 64, "%s", conf[x]);
                (void) off;
            }
            if (res->nmods < 8) snprintf(res->mods[res->nmods++], 64, "%s", mod_id);
            ++res->installed;
            res->files += nf;
            if (logf) { snprintf(line, sizeof line, "install %d: %s: %d file(s) for %s", post->id, mod_id, nf, grp_name); logf(line); }
        }
    }
    free(pl); free(done);
    if (res->installed > 0) {
        char extra[220];
        extra[0] = '\0';
        if (res->nturned_off > 0) snprintf(extra, sizeof extra, " Turned off %s (it replaces the same file).", res->turned_off[0]);
        snprintf(msg, mcap, "Installed %d mod%s. Restart the game to use %s.%s", res->installed, res->installed == 1 ? "" : "s", res->installed == 1 ? "it" : "them", extra);
        return 0;
    }
    snprintf(msg, mcap, "%s", res->first_err[0] ? res->first_err : "Nothing in this post could be installed.");
    return -1;
}

/* remove every folder of post `id` (nucleus-<id>-*): returns how many were removed. The enabled.txt lines go too. */
static int nc_loose_remove(const char *mods_dir, int post_id) {
    static char names[256][64];
    int n = nc_list_subdirs(mods_dir, names, 256), i, removed = 0;
    char prefix[40];
    snprintf(prefix, sizeof prefix, "nucleus-%d-", post_id);
    for (i = 0; i < n; ++i) {
        char dir[700];
        if (strncmp(names[i], prefix, strlen(prefix)) != 0) continue;
        snprintf(dir, sizeof dir, "%s/%s", mods_dir, names[i]);
        nc_fix_slashes(dir);
        nc_rmtree(dir);
        nc_enabled_edit(mods_dir, names[i], 0);
        if (!nc_is_dir(dir)) ++removed;
    }
    return removed;
}

#endif
