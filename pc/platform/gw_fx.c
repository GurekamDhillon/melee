/* gw_fx.c - Geno effects runtime, part 1: the package loader, the simulation and the numeric census.
 * Design: workspace _research/geno-effects-runtime.md; format: docs/geno.md section 20 (.gfx.json v1).
 *
 * NO DRAWING HERE. The renderer (an Aurora draw hook, a depth copy and a colour copy) is a later part and will
 * read the particle arrays below; until then the runtime simulates and reports numbers (the census), so the
 * behaviour can be checked against the source effect before anything is drawn.
 *
 * NATIVE, NOT GAME STATE. Effects are visual only: gameplay never reads them. They are kept out of gw_snap and
 * re-simulated after a rollback instead (section 5 of the design): the game half keeps a frame counter in game
 * memory (restored by a rollback); when it arrives here smaller than the last one we saw, the state is restored
 * from a ring of per-frame copies and the resimulated frames step it again. Everything is deterministic: one
 * PRNG per emitter instance, seeded from (package, emitter, owner, attach frame); no host time.
 *
 * Owner transform: the game half attaches an effect to a guest JObj (its address and the offset of its world
 * matrix inside HSD_JObj) and the simulation reads that matrix each frame (read-only, big-endian floats).
 *
 * Budget (accepted 2026-09-26): 2000 live particles, 256 emitter instances per match. */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "gw_mods.h"
#include "gw_test.h"
#include "gw_fx_internal.h"
#include "gw_fx_query.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ---- a small JSON DOM (dynamic; the packages are 50-100 KB) ---------------------------------------- */
enum { FJ_NULL, FJ_BOOL, FJ_NUM, FJ_STR, FJ_ARR, FJ_OBJ };
typedef struct { int type; double num; char *str; char *key; int first, next; } fjnode;
typedef struct { fjnode *n; int nn, cap; const char *p; const char *err; } fjdoc;

static int fj_new(fjdoc *d, int type) {
    if (d->nn >= d->cap) {
        int cap = d->cap ? d->cap * 2 : 1024;
        fjnode *n = (fjnode *) realloc(d->n, (size_t) cap * sizeof *n);
        if (n == NULL) { d->err = "out of memory"; return -1; }
        d->n = n;
        d->cap = cap;
    }
    memset(&d->n[d->nn], 0, sizeof d->n[0]);
    d->n[d->nn].type = type;
    d->n[d->nn].first = d->n[d->nn].next = -1;
    return d->nn++;
}
static void fj_ws(fjdoc *d) { while (*d->p == ' ' || *d->p == '\t' || *d->p == '\r' || *d->p == '\n') d->p++; }
static char *fj_str(fjdoc *d) {
    const char *s = ++d->p;
    size_t n = 0;
    char *out, *o;
    while (s[n] && s[n] != '"') n += (s[n] == '\\' && s[n + 1]) ? 2 : 1;
    out = o = (char *) malloc(n + 1);
    if (out == NULL) return NULL;
    while (*d->p && *d->p != '"') {
        if (*d->p == '\\' && d->p[1]) { d->p++; *o++ = *d->p == 'n' ? '\n' : *d->p; d->p++; continue; }
        *o++ = *d->p++;
    }
    *o = 0;
    if (*d->p == '"') d->p++;
    return out;
}
static int fj_value(fjdoc *d, int depth);
static int fj_container(fjdoc *d, int depth, int obj) {
    int me = fj_new(d, obj ? FJ_OBJ : FJ_ARR), last = -1;
    char close = obj ? '}' : ']';
    if (me < 0) return -1;
    d->p++;
    for (;;) {
        char *key = NULL;
        int c;
        fj_ws(d);
        if (*d->p == close) { d->p++; return me; }
        if (obj) {
            if (*d->p != '"') { d->err = "expected a key"; return -1; }
            key = fj_str(d);
            fj_ws(d);
            if (*d->p != ':') { d->err = "expected ':'"; return -1; }
            d->p++;
        }
        c = fj_value(d, depth + 1);
        if (c < 0) return -1;
        d->n[c].key = key;
        if (last < 0) d->n[me].first = c; else d->n[last].next = c;
        last = c;
        fj_ws(d);
        if (*d->p == ',') { d->p++; continue; }
        if (*d->p == close) { d->p++; return me; }
        d->err = "expected ',' or a close";
        return -1;
    }
}
static int fj_value(fjdoc *d, int depth) {
    int me;
    if (depth > 64) { d->err = "too deep"; return -1; }
    fj_ws(d);
    switch (*d->p) {
    case '{': return fj_container(d, depth, 1);
    case '[': return fj_container(d, depth, 0);
    case '"': me = fj_new(d, FJ_STR); if (me >= 0) d->n[me].str = fj_str(d); return me;
    case 't': me = fj_new(d, FJ_BOOL); if (me >= 0) d->n[me].num = 1; d->p += 4; return me;
    case 'f': me = fj_new(d, FJ_BOOL); d->p += 5; return me;
    case 'n': me = fj_new(d, FJ_NULL); d->p += 4; return me;
    default: {
        char *end;
        double v = strtod(d->p, &end);
        if (end == d->p) { d->err = "bad value"; return -1; }
        d->p = end;
        me = fj_new(d, FJ_NUM);
        if (me >= 0) d->n[me].num = v;
        return me;
    }
    }
}
static void fj_free(fjdoc *d) {
    int i;
    for (i = 0; i < d->nn; ++i) { free(d->n[i].str); free(d->n[i].key); }
    free(d->n);
    memset(d, 0, sizeof *d);
}
static int fj_get(const fjdoc *d, int o, const char *key) {
    int c;
    if (o < 0 || d->n[o].type != FJ_OBJ) return -1;
    for (c = d->n[o].first; c >= 0; c = d->n[c].next)
        if (d->n[c].key && strcmp(d->n[c].key, key) == 0) return c;
    return -1;
}
static int fj_path(const fjdoc *d, int o, const char *a, const char *b) { return fj_get(d, fj_get(d, o, a), b); }
static double fj_num(const fjdoc *d, int o, double def) {
    return o >= 0 && (d->n[o].type == FJ_NUM || d->n[o].type == FJ_BOOL) ? d->n[o].num : def;
}
static int fj_at(const fjdoc *d, int arr, int i) {
    int c = arr >= 0 && d->n[arr].type == FJ_ARR ? d->n[arr].first : -1;
    while (c >= 0 && i-- > 0) c = d->n[c].next;
    return c;
}
static void fj_vec(const fjdoc *d, int arr, float *out, int n) {
    int i;
    for (i = 0; i < n; ++i) out[i] = (float) fj_num(d, fj_at(d, arr, i), out[i]);
}
static const char *fj_s(const fjdoc *d, int o) { return o >= 0 && d->n[o].type == FJ_STR ? d->n[o].str : ""; }

static fx_pkg *fx_pkgs[FX_MAX_PKGS];
static int fx_npkg;

static void fx_curve_read(const fjdoc *d, int o, fx_curve *c, int is_color) {
    int keys = fj_get(d, o, "keys"), i, v = fj_get(d, o, "value");
    memset(c, 0, sizeof *c);
    if (is_color) fj_vec(d, v, c->value, 3); else c->value[0] = c->value[1] = c->value[2] = (float) fj_num(d, v, 1.0);
    c->keyed = strcmp(fj_s(d, fj_get(d, o, "kind")), "keys") == 0 && keys >= 0;
    for (i = 0; c->keyed && i < FX_KEYS; ++i) {
        int k = fj_at(d, keys, i);
        if (k < 0) break;
        fj_vec(d, k, c->k[i], 4);
        c->n++;
    }
}

static int fx_tex_index(const fjdoc *d, int root, const char *name) {
    int i, t;
    for (i = 0; (t = fj_at(d, fj_get(d, root, "textures"), i)) >= 0 && i < FX_MAX_TEX; ++i)
        if (strcmp(fj_s(d, fj_get(d, t, "name")), name) == 0) return i;
    return -1;
}
static int fx_mask(const fjdoc *d, int arr) {
    int i, c, m = 0;
    for (i = 0; (c = fj_at(d, arr, i)) >= 0; ++i) m |= 1 << ((int) fj_num(d, c, 0) & 7);
    return m;
}
static int fx_enum(const char *s, const char *const *names, int n, int def) {
    int i;
    for (i = 0; i < n; ++i) if (strcmp(s, names[i]) == 0) return i;
    return def;
}
/* material.shader / .bloom / blend / alpha_test, the samplers and the param curve (what the renderer reads) */
static void fx_material_read(const fjdoc *d, int root, int e, fx_emitter *m) {
    static const char *const sh[] = {"sprite", "warp", "distortion"}, *const col[] = {"flat", "modulate", "lerp"},
                              *const bl[] = {"alpha", "add", "sub", "mul", "screen"},
                              *const pat[] = {"none", "fit_life", "clamp", "loop", "random"},
                              *const wr[] = {"Mirror", "Repeat", "Clamp"};
    int ma = fj_get(d, e, "material"), s = fj_path(d, e, "material", "shader"), b = fj_get(d, ma, "bloom");
    int at = fj_get(d, ma, "alpha_test"), smps = fj_get(d, e, "samplers"), i, x, pa = fj_get(d, e, "particle");
    m->shader = s >= 0 ? fx_enum(fj_s(d, fj_get(d, s, "type")), sh, 3, FX_SH_SPRITE) : FX_SH_SPRITE;
    m->color_mode = s >= 0 ? fx_enum(fj_s(d, fj_get(d, s, "color")), col, 3, FX_COL_MODULATE) : FX_COL_MODULATE;
    m->offset = s >= 0 ? (int) fj_num(d, fj_get(d, s, "offset"), -1) : -1;
    m->offset_mask = fx_mask(d, fj_get(d, s, "offset_targets"));
    m->color_mask = s >= 0 && fj_get(d, s, "color_textures") >= 0 ? fx_mask(d, fj_get(d, s, "color_textures")) : 1;
    m->alpha_mask = s >= 0 && fj_get(d, s, "alpha_textures") >= 0 ? fx_mask(d, fj_get(d, s, "alpha_textures")) : 1;
    m->strength[0] = m->strength[1] = 0.0f;
    fj_vec(d, fj_get(d, s, "strength"), m->strength, 2);
    m->blend = fx_enum(fj_s(d, fj_get(d, ma, "blend")), bl, 5, FX_BLEND_ALPHA);
    m->depth_test = (int) fj_num(d, fj_get(d, ma, "depth_test"), 1);
    m->alpha_test = at >= 0 && d->n[at].type == FJ_OBJ;
    m->alpha_threshold = (float) fj_num(d, fj_get(d, at, "threshold"), 0);
    m->bloom_threshold = (float) fj_num(d, fj_get(d, b, "threshold"), 1);
    m->bloom_intensity = (float) fj_num(d, fj_get(d, b, "intensity"), 0);
    {
        fx_curve *k = &m->param;
        int ks = fj_get(d, pa, "param_keys"), j;
        memset(k, 0, sizeof *k);
        for (j = 0; j < FX_KEYS && fj_at(d, ks, j) >= 0; ++j) fj_vec(d, fj_at(d, ks, j), k->k[k->n++], 4);
        k->keyed = k->n > 0;
        k->value[0] = k->value[1] = k->value[2] = 1.0f;
    }
    for (i = 0; i < FX_SAMPLERS; ++i) m->smp[i].tex = -1;
    for (i = 0; (x = fj_at(d, smps, i)) >= 0; ++i) {
        int slot = (int) fj_num(d, fj_get(d, x, "slot"), i), pt = fj_get(d, x, "pattern"), uv = fj_get(d, x, "uv");
        int en = fj_get(d, uv, "enable"), j, tb = fj_get(d, pt, "table");
        fx_sampler *sp;
        float div[2] = {1, 1};
        if (slot < 0 || slot >= FX_SAMPLERS) continue;
        sp = &m->smp[slot];
        sp->tex = fx_tex_index(d, root, fj_s(d, fj_get(d, x, "texture")));
        sp->wrap[0] = fx_enum(fj_s(d, fj_at(d, fj_get(d, x, "wrap"), 0)), wr, 3, FX_WRAP_REPEAT);
        sp->wrap[1] = fx_enum(fj_s(d, fj_at(d, fj_get(d, x, "wrap"), 1)), wr, 3, FX_WRAP_REPEAT);
        sp->pattern = fx_enum(fj_s(d, fj_get(d, pt, "mode")), pat, 5, FX_PAT_NONE);
        sp->pattern_count = (int) fj_num(d, fj_get(d, pt, "count"), 0);
        sp->pattern_count_random = (int) fj_num(d, fj_get(d, pt, "count_random"), 0);
        sp->pattern_random_start = (int) fj_num(d, fj_get(d, pt, "loop_random_start"), 0);
        sp->pattern_freq = (float) fj_num(d, fj_get(d, pt, "frequency"), 1);
        for (j = 0; j < FX_PATTERN_TABLE && fj_at(d, tb, j) >= 0; ++j) sp->table[j] = (int) fj_num(d, fj_at(d, tb, j), 0);
        sp->table_n = j;
        sp->scale[0] = sp->scale[1] = 1.0f;
        fj_vec(d, fj_get(d, uv, "scroll"), sp->scroll, 2);
        fj_vec(d, fj_get(d, uv, "scroll_add"), sp->scroll_add, 2);
        fj_vec(d, fj_get(d, uv, "scroll_random"), sp->scroll_random, 2);
        fj_vec(d, fj_get(d, uv, "scale"), sp->scale, 2);
        fj_vec(d, fj_get(d, uv, "scale_add"), sp->scale_add, 2);
        fj_vec(d, fj_get(d, uv, "scale_random"), sp->scale_random, 2);
        fj_vec(d, fj_get(d, uv, "divide"), div, 2);
        sp->div[0] = div[0] >= 1 ? (int) div[0] : 1;
        sp->div[1] = div[1] >= 1 ? (int) div[1] : 1;
        sp->rotate = (float) fj_num(d, fj_get(d, uv, "rotate"), 0);
        sp->rotate_add = (float) fj_num(d, fj_get(d, uv, "rotate_add"), 0);
        sp->rotate_random = (float) fj_num(d, fj_get(d, uv, "rotate_random"), 0);
        sp->en_scroll = (int) fj_num(d, fj_get(d, en, "scroll"), 1);
        sp->en_scale = (int) fj_num(d, fj_get(d, en, "scale"), 1);
        sp->en_rotate = (int) fj_num(d, fj_get(d, en, "rotate"), 1);
    }
}

static int fx_shape(const char *s) {
    if (!strcmp(s, "sphere")) return FX_SHAPE_SPHERE;
    if (!strcmp(s, "sphere_fill")) return FX_SHAPE_SPHERE_FILL;
    if (!strcmp(s, "circle")) return FX_SHAPE_CIRCLE;
    if (!strcmp(s, "circle_fill")) return FX_SHAPE_CIRCLE_FILL;
    if (!strcmp(s, "point")) return FX_SHAPE_POINT;
    if (!strcmp(s, "circle_same_divide")) return FX_SHAPE_CIRCLE_DIVIDE;
    if (!strcmp(s, "sphere_same_divide")) return FX_SHAPE_SPHERE_DIVIDE;
    if (!strcmp(s, "sphere_same_divide64")) return FX_SHAPE_SPHERE_DIVIDE64;
    if (!strcmp(s, "cylinder")) return FX_SHAPE_CYLINDER;
    if (!strcmp(s, "cylinder_fill")) return FX_SHAPE_CYLINDER_FILL;
    if (!strcmp(s, "box")) return FX_SHAPE_BOX;
    if (!strcmp(s, "box_fill")) return FX_SHAPE_BOX_FILL;
    if (!strcmp(s, "line")) return FX_SHAPE_LINE;
    if (!strcmp(s, "line_same_divide")) return FX_SHAPE_LINE_DIVIDE;
    if (!strcmp(s, "rectangle")) return FX_SHAPE_RECTANGLE;
    return FX_SHAPE_PRIMITIVE;
}

static char *fx_read(const char *path);

/* A package mesh (mesh/<name>.json: position, uv0, color0, indices), expanded to triangle vertices of 12 floats
 * (position, u, v, 0, 0, colour rgba) so the renderer draws it without an index buffer. Loaded once with the package. */
static void fx_load_mesh(fx_pkg *p, const char *pkgpath, const char *name, const char *file) {
    char path[MAX_PATH], *text, *slash;
    fjdoc d;
    int root, pos, uv, col, idx, n, i, k;
    float *v;
    const int m = p->nmesh_loaded;
    if (m >= FX_MAX_MESH) return;
    snprintf(path, sizeof path, "%s", pkgpath);
    slash = strrchr(path, '\\');
    if (strrchr(path, '/') > slash) slash = strrchr(path, '/');
    if (slash == NULL) return;
    snprintf(slash + 1, sizeof path - (size_t) (slash + 1 - path), "%s", file);
    text = fx_read(path);
    if (text == NULL) { gw_log("fx: %s: mesh %s not found", p->name, path); return; }
    memset(&d, 0, sizeof d);
    d.p = text;
    root = fj_value(&d, 0);
    pos = fj_get(&d, root, "position");
    uv = fj_get(&d, root, "uv0");
    col = fj_get(&d, root, "color0");
    idx = fj_get(&d, root, "indices");
    for (n = 0; fj_at(&d, idx, n) >= 0; ++n) {}
    n -= n % 3;
    v = n > 0 ? (float *) calloc((size_t) n * 12, sizeof(float)) : NULL;
    if (v != NULL) {
        /* node index of each vertex's position / uv / colour (fj_at walks a list, so collect them once) */
        int nv = 0, *pn, *un, *cn, c;
        for (c = pos >= 0 ? d.n[pos].first : -1; c >= 0; c = d.n[c].next) nv++;
        pn = (int *) malloc((size_t) (nv + 1) * sizeof(int));
        un = (int *) malloc((size_t) (nv + 1) * sizeof(int));
        cn = (int *) malloc((size_t) (nv + 1) * sizeof(int));
        for (k = 0; k < nv; ++k) pn[k] = un[k] = cn[k] = -1;
        for (k = 0, c = pos >= 0 ? d.n[pos].first : -1; c >= 0 && k < nv; c = d.n[c].next) pn[k++] = c;
        for (k = 0, c = uv >= 0 ? d.n[uv].first : -1; c >= 0 && k < nv; c = d.n[c].next) un[k++] = c;
        for (k = 0, c = col >= 0 ? d.n[col].first : -1; c >= 0 && k < nv; c = d.n[c].next) cn[k++] = c;
        c = idx >= 0 ? d.n[idx].first : -1;
        for (i = 0; i < n && c >= 0; ++i, c = d.n[c].next) {
            const int j = (int) d.n[c].num;
            float *o = v + (size_t) i * 12;
            const int pj = j >= 0 && j < nv ? pn[j] : -1, uj = j >= 0 && j < nv ? un[j] : -1,
                      cj = j >= 0 && j < nv ? cn[j] : -1;
            o[8] = o[9] = o[10] = o[11] = 1.0f;
            for (k = 0; k < 3; ++k) o[k] = (float) fj_num(&d, fj_at(&d, pj, k), 0);
            o[3] = (float) fj_num(&d, fj_at(&d, uj, 0), 0);
            o[4] = (float) fj_num(&d, fj_at(&d, uj, 1), 0);
            for (k = 0; k < 4; ++k) o[8 + k] = (float) fj_num(&d, fj_at(&d, cj, k), 1);
        }
        free(pn);
        free(un);
        free(cn);
    }
    fj_free(&d);
    free(text);
    snprintf(p->mesh_name[m], sizeof p->mesh_name[m], "%s", name);
    p->mesh_v[m] = v;
    p->mesh_nv[m] = v != NULL ? n : 0;
    p->nmesh_loaded++;
}

static fx_pkg *fx_parse(const char *text, const char *path) {
    fjdoc d;
    int root, ems, e, i;
    fx_pkg *p;
    memset(&d, 0, sizeof d);
    d.p = text;
    root = fj_value(&d, 0);
    if (root < 0 || fj_num(&d, fj_get(&d, root, "geno_fx"), 0) < 1) {
        gw_log("fx: %s: not a Geno effect package (%s)", path, d.err ? d.err : "no geno_fx");
        fj_free(&d);
        return NULL;
    }
    p = (fx_pkg *) calloc(1, sizeof *p);
    snprintf(p->name, sizeof p->name, "%s", fj_s(&d, fj_get(&d, root, "name")));
    snprintf(p->path, sizeof p->path, "%s", path);
    snprintf(p->dir, sizeof p->dir, "%s", path);
    {
        char *sl = strrchr(p->dir, '\\'), *sl2 = strrchr(p->dir, '/');
        if (sl2 > sl) sl = sl2;
        if (sl) *sl = 0;
    }
    p->ntex = 0;
    for (i = 0; (e = fj_at(&d, fj_get(&d, root, "textures"), i)) >= 0 && p->ntex < FX_MAX_TEX; ++i) {
        const char *sw = fj_s(&d, fj_get(&d, e, "swizzle"));
        snprintf(p->tex_file[p->ntex], sizeof p->tex_file[0], "%s", fj_s(&d, fj_get(&d, e, "file")));
        snprintf(p->tex_swizzle[p->ntex], sizeof p->tex_swizzle[0], "%s", strlen(sw) == 4 ? sw : "rgba");
        p->ntex++;
    }
    for (i = 0; (e = fj_at(&d, fj_get(&d, root, "meshes"), i)) >= 0; ++i) {
        p->nmesh++;
        fx_load_mesh(p, path, fj_s(&d, fj_get(&d, e, "name")), fj_s(&d, fj_get(&d, e, "file")));
    }
    /* "space": the effect's forward / up axes; absent = the owner's own (+X forward, +Y up) */
    p->forward[0] = 1.0f; p->forward[1] = p->forward[2] = 0.0f;
    p->up[1] = 1.0f; p->up[0] = p->up[2] = 0.0f;
    fj_vec(&d, fj_path(&d, root, "space", "forward"), p->forward, 3);
    fj_vec(&d, fj_path(&d, root, "space", "up"), p->up, 3);
    ems = fj_get(&d, root, "emitters");
    for (i = 0; (e = fj_at(&d, ems, i)) >= 0 && p->nem < FX_MAX_EMITTERS; ++i) {
        fx_emitter *m = &p->em[p->nem++];
        int t = fj_get(&d, e, "transform"), em = fj_get(&d, e, "emission"), sh = fj_get(&d, e, "shape");
        int pa = fj_get(&d, e, "particle"), co = fj_get(&d, e, "color");
        int ve = fj_get(&d, pa, "velocity"), fo = fj_get(&d, pa, "forces"), ro = fj_get(&d, pa, "rotation");
        int sc = fj_get(&d, pa, "scale"), bd = fj_get(&d, em, "by_distance");
        int cl = fj_get(&d, co, "loop"), wa = fj_get(&d, e, "wave"), fa = fj_get(&d, em, "fade"), j;
        const char *fl = fj_s(&d, fj_get(&d, e, "follow"));
        memset(m, 0, sizeof *m);
        snprintf(m->name, sizeof m->name, "%s", fj_s(&d, fj_get(&d, e, "name")));
        m->order = (int) fj_num(&d, fj_get(&d, e, "order"), i);
        m->priority = (int) fj_num(&d, fj_get(&d, e, "priority"), m->order);
        m->mesh = strcmp(fj_s(&d, fj_get(&d, e, "kind")), "mesh") == 0;
        m->mesh_idx = -1;
        if (m->mesh) {
            const char *mn = fj_s(&d, fj_get(&d, e, "mesh"));
            int q;
            for (q = 0; q < p->nmesh_loaded; ++q)
                if (strcmp(p->mesh_name[q], mn) == 0) m->mesh_idx = q;
        }
        {
            const char *ps = fj_s(&d, fj_path(&d, e, "particle", "shape"));
            m->pshape = !strcmp(ps, "y_billboard") ? FX_PS_Y_BILLBOARD
                        : (!strcmp(ps, "directional_y") || !strcmp(ps, "directional_polygon") ||
                           !strcmp(ps, "stripe") || !strcmp(ps, "complex_stripe")) ? FX_PS_DIRECTIONAL
                        : !strcmp(ps, "plate_xy") ? FX_PS_PLATE_XY
                        : !strcmp(ps, "plate_xz") ? FX_PS_PLATE_XZ : FX_PS_BILLBOARD;
        }
        m->escale[0] = m->escale[1] = m->escale[2] = 1.0f;
        fj_vec(&d, fj_get(&d, t, "scale"), m->escale, 3);
        m->follow = !strcmp(fl, "none") ? 1 : !strcmp(fl, "translate") ? 2 : 0;
        fj_vec(&d, fj_get(&d, t, "translate"), m->trans, 3);
        fj_vec(&d, fj_get(&d, t, "rotate"), m->rot, 3);
        m->start = (int) fj_num(&d, fj_get(&d, em, "start"), 0);
        m->duration = (int) fj_num(&d, fj_get(&d, em, "duration"), 0);
        m->one_time = (int) fj_num(&d, fj_get(&d, em, "one_time"), 0);
        m->fade_on_stop = (int) fj_num(&d, fj_get(&d, fa, "on_stop"), 0);
        m->fade_alpha_frames = (int) fj_num(&d, fj_get(&d, fa, "alpha_frames"), 0);
        m->fade_in_frames = (int) fj_num(&d, fj_get(&d, fa, "fade_in_frames"), 0);
        m->alpha_fade_in = (int) fj_num(&d, fj_get(&d, fa, "alpha_fade_in"), 0);
        m->scale_fade_in = (int) fj_num(&d, fj_get(&d, fa, "scale_fade_in"), 0);
        m->rate = (float) fj_num(&d, fj_get(&d, em, "rate"), 1);
        m->rate_random = (float) fj_num(&d, fj_get(&d, em, "rate_random"), 0);
        m->interval = (int) fj_num(&d, fj_get(&d, em, "interval"), 0);
        if (bd >= 0 && d.n[bd].type == FJ_OBJ) {
            m->by_dist = 1;
            m->dist_unit = (float) fj_num(&d, fj_get(&d, bd, "unit"), 1);
            m->dist_min = (float) fj_num(&d, fj_get(&d, bd, "min"), 0);
            m->dist_max = (float) fj_num(&d, fj_get(&d, bd, "max"), 0);
            m->dist_max_particles = (int) fj_num(&d, fj_get(&d, bd, "max_particles"), 0);
        }
        m->shape = fx_shape(fj_s(&d, fj_get(&d, sh, "type")));
        fj_vec(&d, fj_get(&d, sh, "radius"), m->radius, 3);
        m->form_scale[0] = m->form_scale[1] = m->form_scale[2] = 1.0f;
        fj_vec(&d, fj_get(&d, sh, "form_scale"), m->form_scale, 3);
        m->caliber = (float) fj_num(&d, fj_get(&d, sh, "caliber"), 1);
        m->sweep[1] = 6.2831853f; m->sweep[2] = 3.1415927f;
        fj_vec(&d, fj_get(&d, sh, "sweep"), m->sweep, 3);
        m->sweep_start_random = (int) fj_num(&d, fj_get(&d, sh, "sweep_start_random"), 0);
        m->surface_random = (float) fj_num(&d, fj_get(&d, sh, "surface_random"), 0);
        m->line[1] = 1.0f;
        fj_vec(&d, fj_get(&d, sh, "line"), m->line, 2);
        for (j = 0; j < 4; ++j) m->divide[j] = (int) fj_num(&d, fj_at(&d, fj_get(&d, sh, "divide"), j), 1);
        m->life = (int) fj_num(&d, fj_get(&d, pa, "life"), 30);
        m->life_random = (float) fj_num(&d, fj_get(&d, pa, "life_random_pct"), 0) / 100.0f;
        m->infinite = (int) fj_num(&d, fj_get(&d, pa, "infinite"), 0);
        m->vel_all = (float) fj_num(&d, fj_get(&d, ve, "all_direction"), 0);
        fj_vec(&d, fj_get(&d, ve, "direction"), m->vel_dir, 3);
        m->vel_dir_scale = (float) fj_num(&d, fj_get(&d, ve, "direction_scale"), 0);
        m->vel_random = (float) fj_num(&d, fj_get(&d, ve, "random_pct"), 0) / 100.0f;
        m->inherit = (float) fj_num(&d, fj_get(&d, ve, "inherit"), 0);
        fj_vec(&d, fj_get(&d, fo, "gravity_dir"), m->grav_dir, 3);
        m->grav = (float) fj_num(&d, fj_get(&d, fo, "gravity"), 0);
        m->grav_world = (int) fj_num(&d, fj_get(&d, fo, "gravity_world"), 1);
        m->air = (float) fj_num(&d, fj_get(&d, fo, "air_resistance"), 1);
        fj_vec(&d, fj_get(&d, sc, "base"), m->scale, 2);
        m->scale_z = (float) fj_num(&d, fj_at(&d, fj_get(&d, sc, "base"), 2), m->scale[0]);
        m->scale_random = (float) fj_num(&d, fj_at(&d, fj_get(&d, sc, "random_pct"), 0), 0) / 100.0f;
        {
            fx_curve *k = &m->scale_keys;
            int ks = fj_get(&d, sc, "keys"), j;
            memset(k, 0, sizeof *k);
            for (j = 0; j < FX_KEYS && fj_at(&d, ks, j) >= 0; ++j) fj_vec(&d, fj_at(&d, ks, j), k->k[k->n++], 4);
            k->keyed = k->n > 0;
            k->value[0] = k->value[1] = k->value[2] = 1.0f;
        }
        fx_curve_read(&d, fj_get(&d, co, "color0"), &m->color0, 1);
        fx_curve_read(&d, fj_get(&d, co, "alpha0"), &m->alpha0, 0);
        fx_curve_read(&d, fj_get(&d, co, "color1"), &m->color1, 1);
        fx_curve_read(&d, fj_get(&d, co, "alpha1"), &m->alpha1, 0);
        m->color_scale = (float) fj_num(&d, fj_get(&d, co, "scale"), 1);
        {
            fx_curve *curves[4] = {&m->color0, &m->alpha0, &m->color1, &m->alpha1};
            const char *names[4] = {"color0", "alpha0", "color1", "alpha1"};
            for (j = 0; j < 4; ++j) {
                int loop = fj_get(&d, cl, names[j]);
                curves[j]->loop = (int) fj_num(&d, fj_at(&d, loop, 0), 0);
                curves[j]->loop_rate = (int) fj_num(&d, fj_at(&d, loop, 1), 0);
            }
            m->scale_keys.loop = (int) fj_num(&d, fj_get(&d, sc, "loop"), 0);
            m->scale_keys.loop_rate = (int) fj_num(&d, fj_get(&d, sc, "loop_rate"), 0);
        }
        m->wave_type = (int) fj_num(&d, fj_get(&d, wa, "type"), 0);
        fj_vec(&d, fj_get(&d, wa, "amplitude"), m->wave_amplitude, 2);
        fj_vec(&d, fj_get(&d, wa, "cycle"), m->wave_cycle, 2);
        fj_vec(&d, fj_get(&d, wa, "phase_random"), m->wave_phase_random, 2);
        fj_vec(&d, fj_get(&d, wa, "phase_init"), m->wave_phase_init, 2);
        for (j = 0; j < 3; ++j) m->wave_apply[j] = (int) fj_num(&d, fj_at(&d, fj_get(&d, wa, "apply"), j), 0);
        fj_vec(&d, fj_get(&d, ro, "init"), m->rot_init, 3);
        fj_vec(&d, fj_get(&d, ro, "init_random"), m->rot_init_random, 3);
        fj_vec(&d, fj_get(&d, ro, "add"), m->rot_add, 3);
        fj_vec(&d, fj_get(&d, ro, "add_random"), m->rot_add_random, 3);
        fx_material_read(&d, root, e, m);
    }
    fj_free(&d);
    return p;
}

static char *fx_read(const char *path) {
    FILE *f = fopen(path, "rb");
    long n;
    char *b;
    if (f == NULL) return NULL;
    fseek(f, 0, SEEK_END);
    n = ftell(f);
    fseek(f, 0, SEEK_SET);
    b = (char *) malloc((size_t) n + 1);
    if (b) b[fread(b, 1, (size_t) n, f)] = 0;
    fclose(f);
    return b;
}

/* A package by name: mods/<id>/fx/<name>/<name>.gfx.json over every mounting mod (the later mod wins). Loaded
 * once and kept; returns its index, -1 when no mod has it. */
int gw_Fx_Find(const char *name) {
    int i, k, n;
    const char *dir = gw_Mods_Dir();
    for (i = 0; i < fx_npkg; ++i)
        if (strcmp(fx_pkgs[i]->name, name) == 0) return i;
    if (fx_npkg >= FX_MAX_PKGS || dir == NULL || !dir[0]) return -1;
    n = gw_Mods_ActiveCount();
    for (k = n - 1; k >= 0; --k) {
        char path[MAX_PATH];
        char *text;
        fx_pkg *p;
        snprintf(path, sizeof path, "%s\\%s\\fx\\%s\\%s.gfx.json", dir, gw_Mods_Id(gw_Mods_ActiveAt(k)), name, name);
        text = fx_read(path);
        if (text == NULL) continue;
        p = fx_parse(text, path);
        free(text);
        if (p == NULL) continue;
        fx_pkgs[fx_npkg] = p;
        gw_log("fx: package %s loaded from %s: %d emitters, %d textures, %d meshes", p->name, path, p->nem, p->ntex,
               p->nmesh);
        return fx_npkg++;
    }
    gw_log("fx: package %s: no mounted mod has fx/%s/%s.gfx.json", name, name, name);
    return -1;
}

/* ---- simulation state (one block, so the rollback ring is a plain copy; types in gw_fx_internal.h) */
int gw_Fx_Stat(int what);
void gw_Fx_Census(int handle, int frame);

static fx_state fx_cur;
static fx_state fx_ring[FX_RING];
/* An idle state (no instance in use, nothing live) is kept as its header only: every slot is fully
 * re-initialised when it is taken, so a cleared state is equivalent to it. Copying the whole block (600 KB)
 * each frame of a match with no effects cost ~0.5 ms of game thread. */
static unsigned char fx_ring_idle[FX_RING];
static int fx_ring_hi[FX_RING]; /* particle slots [0, hi) were copied; the rest were free */

/* copy a state leaving out the free particle slots past the last used one */
static void fx_copy(fx_state *dst, const fx_state *src, int hi) {
    dst->frame = src->frame;
    memcpy(dst->inst, src->inst, sizeof src->inst);
    memcpy(dst->part, src->part, (size_t) hi * sizeof src->part[0]);
    dst->nlive = src->nlive;
    dst->spawned = src->spawned; dst->killed = src->killed;
    dst->refused = src->refused; dst->refused_emitters = src->refused_emitters;
    memcpy(dst->drv, src->drv, sizeof src->drv);
}

static int fx_idle(void) {
    int i;
    if (fx_cur.nlive > 0) return 0;
    for (i = 0; i < FX_MAX_INST; ++i) if (fx_cur.inst[i].used) return 0;
    return 1;
}
static int fx_ready;

static void fx_reset(void) {
    int i;
    memset(&fx_cur, 0, sizeof fx_cur);
    for (i = 0; i < FX_MAX_PARTICLES; ++i) fx_cur.part[i].inst = -1;
    fx_cur.frame = -1;
    for (i = 0; i < FX_RING; ++i) fx_ring[i].frame = -2;
    fx_ready = 1;
}

static uint32_t fx_rnd(uint32_t *s) {
    uint32_t x = *s ? *s : 0x9E3779B9u;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    return *s = x;
}
static float fx_rndf(uint32_t *s) { return (fx_rnd(s) >> 8) * (1.0f / 16777216.0f); }

float gw_fx_curve_at(const fx_curve *c, float t, int ch) {
    int i;
    if (!c->keyed || c->n == 0) return c->value[ch];
    if (t <= c->k[0][3]) return c->k[0][ch];
    for (i = 1; i < c->n; ++i)
        if (t <= c->k[i][3]) {
            float span = c->k[i][3] - c->k[i - 1][3], f = span > 1e-6f ? (t - c->k[i - 1][3]) / span : 1.0f;
            return c->k[i - 1][ch] + (c->k[i][ch] - c->k[i - 1][ch]) * f;
        }
    return c->k[c->n - 1][ch];
}

/* Loop rates in the source format are frame periods. Keep the ordinary normalized-life evaluator
 * for callers that do not have a particle, and use this one for per-particle simulation. */
static float fx_curve_part_at(const fx_curve *c, int age, int life, int ch) {
    float t = life > 0 ? (float) age / (float) life : 0.0f;
    if (c->loop && c->loop_rate > 0)
        t = (float) (age % c->loop_rate) / (float) c->loop_rate;
    return gw_fx_curve_at(c, t, ch);
}

static float fx_clamp(float x, float lo, float hi) { return x < lo ? lo : x > hi ? hi : x; }

static int fx_pattern_cell(const fx_sampler *s, const fx_part *p, int slot) {
    int n = p->pattern_count[slot], idx = 0, step;
    if (n < 1) return 0;
    step = (int) ((float) p->age / (s->pattern_freq > 0.0f ? s->pattern_freq : 1.0f));
    switch (s->pattern) {
    case FX_PAT_FIT_LIFE: idx = p->life > 0 ? p->age * n / p->life : 0; break;
    case FX_PAT_CLAMP: idx = step; break;
    case FX_PAT_LOOP: idx = (step + p->pattern_start[slot]) % n; break;
    case FX_PAT_RANDOM: {
        uint32_t seed = p->seed ^ (uint32_t) slot * 0x9E3779B9u ^ (uint32_t) step * 0x85EBCA6Bu;
        idx = (int) (fx_rnd(&seed) % (uint32_t) n);
        break;
    }
    default: idx = 0; break;
    }
    if (idx < 0) idx = 0;
    if (idx >= n) idx = n - 1;
    return idx < s->table_n ? s->table[idx] : idx;
}

static void fx_visual_update(fx_part *p, const fx_emitter *e, const fx_inst *in) {
    int k, c;
    float phase[2], w[2], fade = 1.0f;
    for (k = 0; k < 2; ++k) {
        phase[k] = e->wave_cycle[k] > 0.0f ? 6.2831853f * (float) p->age / e->wave_cycle[k] + p->wave_phase[k] : 0.0f;
        w[k] = e->wave_type ? e->wave_amplitude[k] * sinf(phase[k]) : 0.0f;
        p->wave_offset[k] = w[k];
        p->wave_scale[k] = 1.0f + w[k];
    }
    for (k = 0; k < 3; ++k)
        p->visual_pos[k] = p->pos[k] + in->m[k][0] * w[0] + in->m[k][1] * w[1];
    if (in->detached && e->fade_on_stop && e->fade_alpha_frames > 0)
        fade = fx_clamp(1.0f - (float) (fx_cur.frame - in->detach_frame + 1) / (float) e->fade_alpha_frames, 0.0f, 1.0f);
    if (e->alpha_fade_in && e->fade_in_frames > 0)
        fade *= fx_clamp((float) p->age / (float) e->fade_in_frames, 0.0f, 1.0f);
    p->fade_alpha = fade;
    for (k = 0; k < 2; ++k) {
        float value = e->scale[k] * fx_curve_part_at(&e->scale_keys, p->age, p->life, k) * p->scale;
        if (e->wave_apply[1]) value *= p->wave_scale[0];
        if (k == 1 && e->wave_apply[2]) value *= p->wave_scale[1];
        if (e->scale_fade_in && e->fade_in_frames > 0)
            value *= fx_clamp((float) p->age / (float) e->fade_in_frames, 0.0f, 1.0f);
        p->visual_scale[k] = value;
    }
    for (c = 0; c < 3; ++c) {
        p->visual_color0[c] = fx_curve_part_at(&e->color0, p->age, p->life, c) * e->color_scale;
        p->visual_color1[c] = fx_curve_part_at(&e->color1, p->age, p->life, c) * e->color_scale;
    }
    p->visual_color0[3] = fx_curve_part_at(&e->alpha0, p->age, p->life, 0) * fade;
    p->visual_color1[3] = fx_curve_part_at(&e->alpha1, p->age, p->life, 0) * fade;
    if (e->wave_apply[0]) { p->visual_color0[3] *= p->wave_scale[0]; p->visual_color1[3] *= p->wave_scale[0]; }
    p->visual_param = fx_curve_part_at(&e->param, p->age, p->life, 0);
    for (k = 0; k < FX_SAMPLERS; ++k) {
        const fx_sampler *s = &e->smp[k];
        const int cols = s->div[0] > 0 ? s->div[0] : 1, rows = s->div[1] > 0 ? s->div[1] : 1;
        p->pattern_cell[k] = fx_pattern_cell(s, p, k);
        p->pattern_uv[k][0] = (float) (p->pattern_cell[k] % cols) / (float) cols;
        p->pattern_uv[k][1] = (float) ((p->pattern_cell[k] / cols) % rows) / (float) rows;
    }
}

/* the owner's world matrix (3x4, big-endian floats in guest memory) */
/* The owner's world matrix x the effect's basis: the effect's own forward / up axes (package "space") turned
 * onto the owner's travel direction (+X x facing: a Melee item's root joint does not turn with the facing) and
 * +Y. Without it an Ultimate article effect (+Z forward) trails into the screen and does not mirror. */
static void fx_owner_mtx(fx_inst *in) {
    int r, c, k;
    float M[3][4];
    if (in->owner == 0 || in->detached || in->fixed) return;
    for (r = 0; r < 3; ++r)
        for (c = 0; c < 4; ++c) M[r][c] = gw_rf32((const void *) (uintptr_t) (in->owner + in->mtx_off + (r * 4 + c) * 4));
    if (in->has_local) { /* a binding's joint-local offset / rotation / scale */
        float L[3][4];
        for (r = 0; r < 3; ++r)
            for (c = 0; c < 4; ++c) {
                float v = c == 3 ? M[r][3] : 0.0f;
                for (k = 0; k < 3; ++k) v += M[r][k] * in->local[k][c];
                L[r][c] = v;
            }
        memcpy(M, L, sizeof M);
    }
    for (r = 0; r < 3; ++r) {
        for (c = 0; c < 3; ++c) {
            float v = 0.0f;
            for (k = 0; k < 3; ++k) v += M[r][k] * in->basis[k][c];
            in->m[r][c] = v;
        }
        in->m[r][3] = M[r][3];
    }
}

static void fx_cross(const float a[3], const float b[3], float o[3]) {
    o[0] = a[1] * b[2] - a[2] * b[1];
    o[1] = a[2] * b[0] - a[0] * b[2];
    o[2] = a[0] * b[1] - a[1] * b[0];
}
static void fx_norm(float v[3]) {
    float l = sqrtf(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    if (l > 1e-6f) { v[0] /= l; v[1] /= l; v[2] /= l; }
}
/* basis = [S U F] [s u f]^T: the effect's side / up / forward (s = u x f) onto the owner's S / U / F */
static void fx_basis(const fx_pkg *p, int facing, float B[3][3]) {
    float f[3] = {p->forward[0], p->forward[1], p->forward[2]}, u[3] = {p->up[0], p->up[1], p->up[2]}, s[3];
    float F[3] = {facing < 0 ? -1.0f : 1.0f, 0, 0}, U[3] = {0, 1, 0}, S[3];
    int r, c;
    fx_norm(f);
    fx_norm(u);
    fx_cross(u, f, s);
    fx_cross(U, F, S);
    for (r = 0; r < 3; ++r)
        for (c = 0; c < 3; ++c) B[r][c] = S[r] * s[c] + U[r] * u[c] + F[r] * f[c];
}
static void fx_xform(const float m[3][4], const float v[3], float out[3], int point) {
    int r;
    for (r = 0; r < 3; ++r) out[r] = m[r][0] * v[0] + m[r][1] * v[1] + m[r][2] * v[2] + (point ? m[r][3] : 0.0f);
}
/* the emitter's own rotation (EmitterInfo.RotateX/Y/Z, XYZ order) applied to a local vector */
static void fx_erot(const fx_emitter *e, float v[3]) {
    float x = v[0], y = v[1], z = v[2], c, s;
    c = cosf(e->rot[0]); s = sinf(e->rot[0]); { float y2 = y * c - z * s, z2 = y * s + z * c; y = y2; z = z2; }
    c = cosf(e->rot[1]); s = sinf(e->rot[1]); { float x2 = x * c + z * s, z2 = -x * s + z * c; x = x2; z = z2; }
    c = cosf(e->rot[2]); s = sinf(e->rot[2]); { float x2 = x * c - y * s, y2 = x * s + y * c; x = x2; y = y2; }
    v[0] = x; v[1] = y; v[2] = z;
}

/* Emitter-local position and outward direction. Sweep angles are radians. For the even-division
 * variants the sampled angle/line coordinate is snapped to a segment centre. */
static void fx_sample_shape(const fx_emitter *e, uint32_t *rng, float local[3], float dir[3]) {
    float th, u, rad = 1.0f, inner = fx_clamp(1.0f - e->caliber, 0.0f, 1.0f);
    int k;
    th = e->sweep[0] + (e->sweep_start_random ? fx_rndf(rng) * 6.2831853f : 0.0f)
         + fx_rndf(rng) * e->sweep[1];
    if (e->shape == FX_SHAPE_CIRCLE_DIVIDE || e->shape == FX_SHAPE_SPHERE_DIVIDE || e->shape == FX_SHAPE_SPHERE_DIVIDE64) {
        int n = e->shape == FX_SHAPE_SPHERE_DIVIDE64 ? 64 : e->divide[0];
        if (n > 1) th = floorf(th * (float) n / 6.2831853f) * 6.2831853f / (float) n;
    }
    u = fx_rndf(rng);
    if (e->shape == FX_SHAPE_SPHERE || e->shape == FX_SHAPE_SPHERE_FILL ||
        e->shape == FX_SHAPE_SPHERE_DIVIDE || e->shape == FX_SHAPE_SPHERE_DIVIDE64) {
        float polar = acosf(1.0f - u * (1.0f - cosf(e->sweep[2])));
        dir[0] = sinf(polar) * cosf(th); dir[1] = cosf(polar); dir[2] = sinf(polar) * sinf(th);
        if (e->shape == FX_SHAPE_SPHERE_FILL)
            rad = cbrtf(inner * inner * inner + fx_rndf(rng) * (1.0f - inner * inner * inner));
        for (k = 0; k < 3; ++k) local[k] = dir[k] * e->radius[k] * rad;
    } else if (e->shape == FX_SHAPE_CIRCLE || e->shape == FX_SHAPE_CIRCLE_FILL || e->shape == FX_SHAPE_CIRCLE_DIVIDE ||
               e->shape == FX_SHAPE_CYLINDER || e->shape == FX_SHAPE_CYLINDER_FILL) {
        dir[0] = cosf(th); dir[1] = 0.0f; dir[2] = sinf(th);
        if (e->shape == FX_SHAPE_CIRCLE_FILL || e->shape == FX_SHAPE_CYLINDER_FILL)
            rad = sqrtf(inner * inner + u * (1.0f - inner * inner));
        local[0] = dir[0] * e->radius[0] * rad;
        local[1] = (e->shape == FX_SHAPE_CYLINDER || e->shape == FX_SHAPE_CYLINDER_FILL) ?
                   (fx_rndf(rng) * 2.0f - 1.0f) * e->radius[1] : 0.0f;
        local[2] = dir[2] * e->radius[2] * rad;
    } else if (e->shape == FX_SHAPE_BOX || e->shape == FX_SHAPE_BOX_FILL || e->shape == FX_SHAPE_RECTANGLE) {
        for (k = 0; k < 3; ++k) local[k] = (fx_rndf(rng) * 2.0f - 1.0f) * e->radius[k];
        if (e->shape == FX_SHAPE_RECTANGLE) local[1] = 0.0f;
        if (e->shape == FX_SHAPE_BOX) {
            int face = (int) (fx_rndf(rng) * 6.0f);
            local[face / 2] = (face & 1 ? 1.0f : -1.0f) * e->radius[face / 2];
        }
        dir[0] = local[0]; dir[1] = local[1]; dir[2] = local[2]; fx_norm(dir);
    } else if (e->shape == FX_SHAPE_LINE || e->shape == FX_SHAPE_LINE_DIVIDE) {
        float t = u;
        if (e->shape == FX_SHAPE_LINE_DIVIDE && e->divide[2] > 1)
            t = floorf(u * (float) e->divide[2]) / (float) (e->divide[2] - 1);
        local[1] = e->line[0] + (t - 0.5f) * e->line[1];
        dir[0] = 0.0f; dir[1] = local[1] >= e->line[0] ? 1.0f : -1.0f; dir[2] = 0.0f;
    } else {
        float z = u * 2.0f - 1.0f, xy = sqrtf(1.0f - z * z);
        dir[0] = xy * cosf(th); dir[1] = xy * sinf(th); dir[2] = z;
    }
    for (k = 0; k < 3; ++k) local[k] *= e->form_scale[k];
}

static void fx_spawn(int ii, fx_inst *in, const fx_emitter *e, const float emvel[3]) {
    int k;
    fx_part *p = NULL;
    float local[3] = {0, 0, 0}, dir[3], v[3], world[3], len;
    for (k = 0; k < FX_MAX_PARTICLES; ++k)
        if (fx_cur.part[k].inst < 0) { p = &fx_cur.part[k]; break; }
    if (p == NULL) { fx_cur.refused++; return; }
    memset(p, 0, sizeof *p);
    p->inst = ii;
    p->seed = fx_rnd(&in->rng);
    fx_sample_shape(e, &p->seed, local, dir);
    /* velocity: radial x all_direction + direction x scale, random, in the emitter frame */
    for (k = 0; k < 3; ++k) v[k] = dir[k] * e->vel_all + e->vel_dir[k] * e->vel_dir_scale;
    if (e->vel_random > 0) {
        float f = 1.0f - e->vel_random * fx_rndf(&p->seed);
        for (k = 0; k < 3; ++k) v[k] *= f;
    }
    for (k = 0; k < 3; ++k) local[k] += e->trans[k];
    fx_erot(e, local);
    fx_erot(e, v);
    fx_xform((const float (*)[4]) in->m, local, world, 1);
    p->pos[0] = world[0]; p->pos[1] = world[1]; p->pos[2] = world[2];
    fx_xform((const float (*)[4]) in->m, v, world, 0);
    for (k = 0; k < 3; ++k) p->vel[k] = world[k] + e->inherit * emvel[k];
    /* follow (EmitterInfo.FollowType): srt = the particle lives in the emitter's frame and moves with it,
     * translate = with its position only, none = in the world where it was born */
    for (k = 0; k < 3; ++k) {
        p->lpos[k] = e->follow == 0 ? local[k] : p->pos[k] - in->m[k][3];
        p->lvel[k] = e->follow == 0 ? v[k] : p->vel[k];
    }
    len = e->life_random > 0 ? 1.0f - e->life_random * fx_rndf(&p->seed) : 1.0f;
    p->life = (int) (e->life * len + 0.5f);
    if (p->life < 1) p->life = 1;
    p->scale = 1.0f - e->scale_random * fx_rndf(&p->seed);
    p->rot = e->rot_init[2] + e->rot_init_random[2] * fx_rndf(&p->seed);
    p->rot_add = e->rot_add[2] + e->rot_add_random[2] * (fx_rndf(&p->seed) * 2 - 1);
    for (k = 0; k < 2; ++k)
        p->wave_phase[k] = e->wave_phase_init[k] +
                           (e->wave_phase_random[k] != 0.0f ? e->wave_phase_random[k] * fx_rndf(&p->seed) * 6.2831853f : 0.0f);
    for (k = 0; k < FX_SAMPLERS; ++k) {
        const fx_sampler *s = &e->smp[k];
        int axis;
        p->pattern_count[k] = s->pattern_count + (s->pattern_count_random > 0 ?
                              (int) (fx_rndf(&p->seed) * (float) (s->pattern_count_random + 1)) : 0);
        if (p->pattern_count[k] < 1) p->pattern_count[k] = 1;
        p->pattern_start[k] = s->pattern_random_start ? (int) (fx_rndf(&p->seed) * (float) p->pattern_count[k]) : 0;
        for (axis = 0; axis < 2; ++axis) {
            p->uv_scroll[k][axis] = s->scroll[axis] + (s->scroll_random[axis] != 0.0f ?
                                      s->scroll_random[axis] * (fx_rndf(&p->seed) * 2.0f - 1.0f) : 0.0f);
            p->uv_scale[k][axis] = s->scale[axis] + (s->scale_random[axis] != 0.0f ?
                                     s->scale_random[axis] * (fx_rndf(&p->seed) * 2.0f - 1.0f) : 0.0f);
        }
        p->uv_rotate[k] = s->rotate + (s->rotate_random != 0.0f ?
                           s->rotate_random * (fx_rndf(&p->seed) * 2.0f - 1.0f) : 0.0f);
    }
    fx_visual_update(p, e, in);
    in->emitted++;
    fx_cur.spawned++;
    fx_cur.nlive++;
}

static void fx_step(void) {
    int i, k;
    for (i = 0; i < FX_MAX_INST; ++i) {
        fx_inst *in = &fx_cur.inst[i];
        const fx_emitter *e;
        float emvel[3] = {0, 0, 0}, travel;
        if (!in->used) continue;
        e = &fx_pkgs[in->pkg]->em[in->em];
        fx_owner_mtx(in);
        in->pos[0] = in->m[0][3]; in->pos[1] = in->m[1][3]; in->pos[2] = in->m[2][3];
        if (in->have_prev) for (k = 0; k < 3; ++k) emvel[k] = in->pos[k] - in->prev[k];
        travel = sqrtf(emvel[0] * emvel[0] + emvel[1] * emvel[1] + emvel[2] * emvel[2]);
        in->age++;
        if (!in->detached && (!e->mesh || e->mesh_idx >= 0) && in->age > e->start) {
            float n = 0.0f;
            if (e->one_time) {
                n = in->emitted == 0 ? e->rate : 0.0f;
            } else if (e->by_dist) {
                in->dist_accum += travel;
                if (e->dist_unit > 0) { n = e->rate * floorf(in->dist_accum / e->dist_unit); in->dist_accum = fmodf(in->dist_accum, e->dist_unit); }
            } else if (e->interval <= 1 || (in->age - e->start - 1) % e->interval == 0) {
                n = e->rate;
            }
            in->accum += n;
            while (in->accum >= 1.0f) { fx_spawn(i, in, e, emvel); in->accum -= 1.0f; }
        }
        /* a binding's emitter_life instance: once its emission is over nothing more comes; let it finish and go */
        if (in->keep && !in->detached &&
            ((e->one_time && in->age > e->start) || (e->duration > 0 && in->age > e->start + e->duration))) {
            in->detached = 1;
            in->detach_frame = fx_cur.frame;
        }
        in->prev[0] = in->pos[0]; in->prev[1] = in->pos[1]; in->prev[2] = in->pos[2];
        in->have_prev = 1;
    }
    for (k = 0; k < FX_MAX_PARTICLES; ++k) {
        fx_part *p = &fx_cur.part[k];
        const fx_emitter *e;
        int c;
        if (p->inst < 0) continue;
        e = &fx_pkgs[fx_cur.inst[p->inst].pkg]->em[fx_cur.inst[p->inst].em];
        {
            const fx_inst *in = &fx_cur.inst[p->inst];
            float g[3], gw[3];
            for (c = 0; c < 3; ++c) g[c] = e->grav_dir[c] * e->grav;
            /* gravity in the world, or in the emitter's frame */
            if (e->grav_world) { gw[0] = g[0]; gw[1] = g[1]; gw[2] = g[2]; }
            else fx_xform((const float (*)[4]) in->m, g, gw, 0);
            if (e->follow == 0) { /* srt: integrate in the emitter frame; world gravity turned into it */
                float gl[3];
                if (e->grav_world)
                    for (c = 0; c < 3; ++c) gl[c] = in->m[0][c] * g[0] + in->m[1][c] * g[1] + in->m[2][c] * g[2];
                else { gl[0] = g[0]; gl[1] = g[1]; gl[2] = g[2]; }
                for (c = 0; c < 3; ++c) {
                    p->lvel[c] = p->lvel[c] * e->air + gl[c];
                    p->lpos[c] += p->lvel[c];
                }
                fx_xform((const float (*)[4]) in->m, p->lpos, p->pos, 1);
            } else if (e->follow == 2) { /* translate: world axes, carried by the owner's position */
                for (c = 0; c < 3; ++c) {
                    p->lvel[c] = p->lvel[c] * e->air + gw[c];
                    p->lpos[c] += p->lvel[c];
                    p->pos[c] = in->m[c][3] + p->lpos[c];
                }
            } else {
                for (c = 0; c < 3; ++c) {
                    p->vel[c] = p->vel[c] * e->air + gw[c];
                    p->pos[c] += p->vel[c];
                }
            }
        }
        p->rot += p->rot_add;
        p->age++;
        fx_visual_update(p, e, &fx_cur.inst[p->inst]);
        if ((p->age >= p->life && (!e->infinite ||
             (fx_cur.inst[p->inst].detached && !(e->fade_on_stop && e->fade_alpha_frames > 0)))) ||
            (fx_cur.inst[p->inst].detached && e->fade_on_stop && e->fade_alpha_frames > 0 && p->fade_alpha <= 0.0f)) {
            p->inst = -1; fx_cur.nlive--; fx_cur.killed++;
        }
    }
    /* an emitter instance whose owner is gone and whose particles are gone is freed */
    for (i = 0; i < FX_MAX_INST; ++i) {
        fx_inst *in = &fx_cur.inst[i];
        int live = 0;
        if (!in->used || !in->detached) continue;
        for (k = 0; k < FX_MAX_PARTICLES && !live; ++k) live = fx_cur.part[k].inst == i;
        if (!live) in->used = 0;
    }
}

/* MELEE_FX_TRACE=1: per frame and per attached package, the owner joint's world position and axes, the
 * emitter origin the runtime computes (owner x the first emitter's local translate) and the centroid of all its
 * live particles - where the effect sits relative to its owner, numerically. */
static void fx_trace(int frame) {
    static int on = -1;
    int i, j, k;
    if (on < 0) {
        const char *v = getenv("MELEE_FX_TRACE");
        on = v ? atoi(v) : 0;
    }
    if (!on) return;
    if (on == 2) /* per emitter instance */
        for (i = 0; i < FX_MAX_INST; ++i) {
            const fx_inst *in = &fx_cur.inst[i];
            float c[3] = {0, 0, 0}, sgn = in->facing < 0 ? -1.0f : 1.0f;
            int n = 0;
            if (!in->used || in->detached) continue;
            for (k = 0; k < FX_MAX_PARTICLES; ++k)
                if (fx_cur.part[k].inst == i) {
                    for (j = 0; j < 3; ++j) c[j] += fx_cur.part[k].pos[j] - in->pos[j];
                    n++;
                }
            if (!n) continue;
            gw_log("fx: trace2 f=%d %s/%s follow=%d n=%d along %.2f up %.2f depth %.2f", frame, fx_pkgs[in->pkg]->name,
                   fx_pkgs[in->pkg]->em[in->em].name, fx_pkgs[in->pkg]->em[in->em].follow, n, sgn * c[0] / n, c[1] / n,
                   c[2] / n);
        }
    for (i = 0; i < FX_MAX_INST; ++i) {
        const fx_inst *in = &fx_cur.inst[i];
        float c[2][3] = {{0, 0, 0}, {0, 0, 0}}, sgn = in->facing < 0 ? -1.0f : 1.0f;
        int n[2] = {0, 0}, first = 1, g;
        if (!in->used) continue;
        for (j = 0; j < i; ++j) /* one line per attach: the first instance of it */
            if (fx_cur.inst[j].used && fx_cur.inst[j].owner == in->owner && fx_cur.inst[j].attach_frame == in->attach_frame)
                first = 0;
        if (!first) continue;
        /* group 0: particles that follow the owner (srt / translate), group 1: world particles (follow none) */
        for (j = i; j < FX_MAX_INST; ++j) {
            if (!fx_cur.inst[j].used || fx_cur.inst[j].owner != in->owner || fx_cur.inst[j].attach_frame != in->attach_frame)
                continue;
            g = fx_pkgs[fx_cur.inst[j].pkg]->em[fx_cur.inst[j].em].follow == 1;
            for (k = 0; k < FX_MAX_PARTICLES; ++k)
                if (fx_cur.part[k].inst == j) {
                    c[g][0] += fx_cur.part[k].pos[0] - in->pos[0];
                    c[g][1] += fx_cur.part[k].pos[1] - in->pos[1];
                    c[g][2] += fx_cur.part[k].pos[2] - in->pos[2];
                    n[g]++;
                }
        }
        for (g = 0; g < 2; ++g)
            if (n[g]) { c[g][0] *= sgn / n[g]; c[g][1] /= n[g]; c[g][2] /= n[g]; }
        gw_log("fx: trace f=%d %s owner=0x%08X facing=%s%s root=(%.2f %.2f %.2f) | following n=%d along %.2f up %.2f "
               "depth %.2f | world n=%d along %.2f up %.2f depth %.2f",
               frame, fx_pkgs[in->pkg]->name, in->owner, in->facing < 0 ? "left" : "right", in->detached ? " (gone)" : "",
               in->pos[0], in->pos[1], in->pos[2], n[0], c[0][0], c[0][1], c[0][2], n[1], c[1][0], c[1][1], c[1][2]);
    }
}

/* set only around a binding's gw_Fx_Attach call (transient, never state) */
typedef struct { float local[3][4]; uint32_t tag; int keep, follow, port, joint; } fx_attach_extra;
static const fx_attach_extra *fx_attach_ex;

/* the renderer's view (gw_fx_render.cpp; game thread) */
const fx_state *gw_fx_state(void) { return fx_ready ? &fx_cur : NULL; }
const fx_pkg *gw_fx_pkg(int i) { return i >= 0 && i < fx_npkg ? fx_pkgs[i] : NULL; }
int gw_fx_npkg(void) { return fx_npkg; }

/* ---- the API the game half calls (scalars; guest addresses as ints) ---------------------------------- */

/* Once per logic frame from the game half, with ITS frame counter (game memory: a rollback restores it). */
/* MELEE_FX_SANITY=1: every 30 frames, over live particles: non-finite position / velocity / size, and the largest
 * |position| from the emitter origin, size (as drawn) and speed, with the emitter that holds each maximum. */
static void fx_sanity(int frame) {
    static int on = -1;
    int k, nan = 0, n = 0, wp = -1, ws = -1, wv = -1;
    float mp = 0, ms = 0, mv = 0;
    if (on < 0) { const char *v = getenv("MELEE_FX_SANITY"); on = v != NULL && v[0] == '1'; }
    if (!on || (frame % 30) != 0) return;
    for (k = 0; k < FX_MAX_PARTICLES; ++k) {
        const fx_part *p = &fx_cur.part[k];
        const fx_inst *in;
        const fx_emitter *e;
        float t, sx, sy, d, v, s;
        int c;
        if (p->inst < 0) continue;
        in = &fx_cur.inst[p->inst];
        e = &fx_pkgs[in->pkg]->em[in->em];
        t = p->life > 0 ? (float) p->age / (float) p->life : 0.0f;
        sx = e->scale[0] * gw_fx_curve_at(&e->scale_keys, t, 0) * p->scale;
        sy = e->scale[1] * gw_fx_curve_at(&e->scale_keys, t, 1) * p->scale;
        d = 0; v = 0;
        for (c = 0; c < 3; ++c) {
            float dp = p->pos[c] - in->pos[c];
            d += dp * dp; v += p->vel[c] * p->vel[c];
            if (!isfinite(p->pos[c]) || !isfinite(p->vel[c])) nan++;
        }
        if (!isfinite(sx) || !isfinite(sy)) nan++;
        d = sqrtf(d); v = sqrtf(v); s = fabsf(sx) > fabsf(sy) ? fabsf(sx) : fabsf(sy);
        if (d > mp) { mp = d; wp = k; }
        if (s > ms) { ms = s; ws = k; }
        if (v > mv) { mv = v; wv = k; }
        n++;
    }
#define FX_WHO(k) ((k) < 0 ? "-" : fx_pkgs[fx_cur.inst[fx_cur.part[k].inst].pkg]->em[fx_cur.inst[fx_cur.part[k].inst].em].name)
    gw_log("fx: sanity frame %d: %d live, %d non-finite; max offset %.1f (%s), max size %.1f (%s), max speed %.1f (%s)",
           frame, n, nan, mp, FX_WHO(wp), ms, FX_WHO(ws), mv, FX_WHO(wv));
#undef FX_WHO
}

/* A rollback / LAB rewind: when fx_cur is at or past `frame`, make it the state at the end of frame - 1. */
static void fx_rewind(int frame) {
    if (!fx_ready) fx_reset();
    if (frame <= fx_cur.frame) {
        int was = fx_cur.frame;
        /* a rollback / LAB rewind: restore the state at the end of frame - 1 and let the resim step again */
        const fx_state *s = &fx_ring[(frame - 1) & (FX_RING - 1)];
        if (frame >= 1 && s->frame == frame - 1) {
            if (fx_ring_idle[(frame - 1) & (FX_RING - 1)]) {
                int i;
                uint32_t sp = s->spawned, ki = s->killed, re = s->refused, rem = s->refused_emitters;
                memset(&fx_cur, 0, sizeof fx_cur);
                for (i = 0; i < FX_MAX_PARTICLES; ++i) fx_cur.part[i].inst = -1;
                fx_cur.spawned = sp; fx_cur.killed = ki; fx_cur.refused = re; fx_cur.refused_emitters = rem;
                fx_cur.frame = s->frame;
                memcpy(fx_cur.drv, s->drv, sizeof s->drv);
            } else {
                int i, hi = fx_ring_hi[(frame - 1) & (FX_RING - 1)];
                fx_copy(&fx_cur, s, hi);
                for (i = hi; i < FX_MAX_PARTICLES; ++i) fx_cur.part[i].inst = -1;
            }
        } else fx_reset();
        gw_log("fx: frame %d after %d - state restored to %d (re-simulating)", frame, was, fx_cur.frame);
    }
}

void gw_Fx_Frame(int frame) {
    fx_rewind(frame);
    if (fx_idle()) {
        fx_state *r = &fx_ring[frame & (FX_RING - 1)];
        fx_cur.frame = frame;
        r->frame = frame; r->nlive = 0;
        r->spawned = fx_cur.spawned; r->killed = fx_cur.killed;
        r->refused = fx_cur.refused; r->refused_emitters = fx_cur.refused_emitters;
        memcpy(r->drv, fx_cur.drv, sizeof r->drv);
        fx_ring_idle[frame & (FX_RING - 1)] = 1;
        return;
    }
    fx_step();
    fx_cur.frame = frame;
    fx_trace(frame);
    fx_sanity(frame);
    if ((frame % 30) == 0 && (fx_cur.nlive > 0 || gw_Fx_Stat(1) > 0)) gw_Fx_Census(0, frame);
    {
        int hi = FX_MAX_PARTICLES;
        while (hi > 0 && fx_cur.part[hi - 1].inst < 0) --hi;
        fx_copy(&fx_ring[frame & (FX_RING - 1)], &fx_cur, hi);
        fx_ring_hi[frame & (FX_RING - 1)] = hi;
        fx_ring_idle[frame & (FX_RING - 1)] = 0;
    }
}

/* Attach every emitter of package `pkg` to the guest JObj at `owner` (world matrix at `mtx_off`); returns a
 * handle (the first instance index + 1) or 0. */
int gw_Fx_Attach(int pkg, int owner, int mtx_off, int frame, int facing) {
    int e, i, first = -1, free_slots = 0, chosen, attached = 0;
    unsigned char keep[FX_MAX_EMITTERS] = {0};
    if (!fx_ready) fx_reset();
    if (pkg < 0 || pkg >= fx_npkg) return 0;
    for (i = 0; i < FX_MAX_INST; ++i) free_slots += !fx_cur.inst[i].used;
    chosen = fx_pkgs[pkg]->nem;
    for (e = 0; e < chosen; ++e) keep[e] = 1;
    while (chosen > free_slots) {
        int worst = -1;
        for (e = 0; e < fx_pkgs[pkg]->nem; ++e)
            if (keep[e] && (worst < 0 || fx_pkgs[pkg]->em[e].priority > fx_pkgs[pkg]->em[worst].priority ||
                (fx_pkgs[pkg]->em[e].priority == fx_pkgs[pkg]->em[worst].priority && e > worst))) worst = e;
        if (worst < 0) break;
        keep[worst] = 0;
        chosen--;
        fx_cur.refused++;
        fx_cur.refused_emitters++;
    }
    for (e = 0; e < fx_pkgs[pkg]->nem; ++e) {
        if (!keep[e]) continue;
        for (i = 0; i < FX_MAX_INST && fx_cur.inst[i].used; ++i) {}
        if (i == FX_MAX_INST) break;
        memset(&fx_cur.inst[i], 0, sizeof fx_cur.inst[i]);
        fx_cur.inst[i].used = 1;
        fx_cur.inst[i].pkg = pkg;
        fx_cur.inst[i].em = e;
        fx_cur.inst[i].owner = (uint32_t) owner;
        fx_cur.inst[i].mtx_off = (uint32_t) mtx_off;
        fx_cur.inst[i].attach_frame = frame;
        fx_cur.inst[i].facing = facing;
        fx_cur.inst[i].joint = -1;
        fx_basis(fx_pkgs[pkg], facing, fx_cur.inst[i].basis);
        if (fx_attach_ex != NULL) { /* a fighter binding: joint-local frame, the package's axes as the joint's own */
            float f[3] = {fx_pkgs[pkg]->forward[0], fx_pkgs[pkg]->forward[1], fx_pkgs[pkg]->forward[2]};
            float u[3] = {fx_pkgs[pkg]->up[0], fx_pkgs[pkg]->up[1], fx_pkgs[pkg]->up[2]}, sd[3];
            int r, c;
            fx_norm(f); fx_norm(u); fx_cross(u, f, sd);
            /* joint space follows the source rig (+X side, +Y up, +Z forward): basis = [x y z] [s u f]^T */
            for (r = 0; r < 3; ++r)
                for (c = 0; c < 3; ++c)
                    fx_cur.inst[i].basis[r][c] = (r == 0 ? sd[c] : r == 1 ? u[c] : f[c]);
            fx_cur.inst[i].has_local = 1;
            memcpy(fx_cur.inst[i].local, fx_attach_ex->local, sizeof fx_cur.inst[i].local);
            fx_cur.inst[i].tag = fx_attach_ex->tag;
            fx_cur.inst[i].keep = fx_attach_ex->keep;
            fx_cur.inst[i].owner_kind = 1;
            fx_cur.inst[i].port = fx_attach_ex->port;
            fx_cur.inst[i].joint = fx_attach_ex->joint;
        }
        fx_cur.inst[i].rng = 0x811C9DC5u ^ ((uint32_t) pkg * 16777619u) ^ ((uint32_t) e << 8) ^ (uint32_t) owner ^ ((uint32_t) frame << 16);
        fx_owner_mtx(&fx_cur.inst[i]);
        if (fx_attach_ex != NULL && !fx_attach_ex->follow) fx_cur.inst[i].fixed = 1; /* world-fixed from here */
        if (first < 0) first = i;
        attached++;
    }
    gw_log("fx: %s attached to 0x%08X at frame %d, facing %s: %d/%d emitter(s), refused %d", fx_pkgs[pkg]->name,
           (uint32_t) owner, frame, facing < 0 ? "left" : "right", attached, fx_pkgs[pkg]->nem, gw_Fx_Stat(4));
    return first + 1;
}

/* Article attachment has only a JObj address. The game half supplies its owner
 * port immediately after Fx_Attach; this only annotates the new emitter group. */
void gw_Fx_SetOwner(int handle, int kind, int port) {
    int i, first = handle - 1;
    if (!fx_ready || first < 0 || first >= FX_MAX_INST || !fx_cur.inst[first].used) return;
    for (i = first; i < FX_MAX_INST; ++i) {
        fx_inst *in = &fx_cur.inst[i], *start = &fx_cur.inst[first];
        if (!in->used || in->pkg != start->pkg || in->owner != start->owner ||
            in->attach_frame != start->attach_frame || in->tag != start->tag) continue;
        in->owner_kind = kind;
        in->port = port;
    }
}

void gw_Fx_SetOwnerPort(int owner, int port) {
    int i;
    if (!fx_ready) return;
    for (i = 0; i < FX_MAX_INST; ++i)
        if (fx_cur.inst[i].used && fx_cur.inst[i].owner == (uint32_t) owner &&
            fx_cur.inst[i].owner_kind == 2) fx_cur.inst[i].port = port;
}

/* The owner is going away: its emitters stop emitting; their live particles finish their lives. */
void gw_Fx_Detach(int owner) {
    int i;
    for (i = 0; i < FX_MAX_INST; ++i)
        if (fx_cur.inst[i].used && fx_cur.inst[i].owner == (uint32_t) owner && !fx_cur.inst[i].detached) {
            fx_cur.inst[i].detached = 1;
            fx_cur.inst[i].detach_frame = fx_cur.frame;
        }
}

/* ---- fighter bindings (fx_bindings.json, format 1: the workspace's ports/ir/schema/fx_bindings.schema.json) ----
 * A fighter's states name effect calls: a package, a frame on the state's clock (animation frame, or frames since
 * the state began), a joint of the fighter's own skeleton with a joint-local offset / rotation (degrees, X then Y
 * then Z) / scale, follow or world-fixed, and an end (state exit, an explicit off frame, or the emitters' own life).
 * The tables are read once at load; what happened this state (which calls fired / ended) lives in fx_cur.drv, so a
 * rollback or LAB rewind restores it with the instances. Branch conditions ("when") cannot be evaluated here: a call
 * is taken when every condition it lists holds; a call under a condition that does not hold is the other branch and
 * is skipped (Sora: the unrotated AirLwImpact, no FireImpact). "owner_destroy" is treated as state exit. */
typedef struct {
    int pkg, joint, follow, keep, situation; /* keep: emitter_life (never detached); situation 0 any, 1 ground, 2 air */
    float frame, end_frame;                   /* end_frame < 0: none */
    float off[3], rot[3], scale;
} fx_bcall;
typedef struct { int subaction, game_clock, first, n; char name[32]; } fx_bstate;
typedef struct {
    char path[MAX_PATH];
    int nstate, ncall;
    fx_bstate st[FX_BIND_STATES];
    fx_bcall call[FX_BIND_CALLS];
} fx_bset;
static fx_bset *fx_bsets[FX_BIND_SETS];
static int fx_nbset;

int gw_Fx_BindLoad(const char *mod_id, const char *rel) {
    char path[MAX_PATH], *text;
    const char *dir = gw_Mods_Dir();
    fjdoc d;
    fx_bset *b;
    int i, root, states, s, taken = 0, skipped = 0, missing = 0;
    if (dir == NULL || !dir[0] || mod_id == NULL || rel == NULL) return -1;
    snprintf(path, sizeof path, "%s\\%s\\%s", dir, mod_id, rel);
    for (i = 0; i < fx_nbset; ++i)
        if (strcmp(fx_bsets[i]->path, path) == 0) return i;
    if (fx_nbset >= FX_BIND_SETS) return -1;
    text = fx_read(path);
    if (text == NULL) { gw_log("fx: bindings %s: not found", path); return -1; }
    memset(&d, 0, sizeof d);
    d.p = text;
    root = fj_value(&d, 0);
    if (root < 0 || (int) fj_num(&d, fj_get(&d, root, "geno_fx_bindings"), 0) != 1) {
        gw_log("fx: bindings %s: not a geno_fx_bindings 1 document", path);
        fj_free(&d); free(text);
        return -1;
    }
    b = (fx_bset *) calloc(1, sizeof *b);
    snprintf(b->path, sizeof b->path, "%s", path);
    states = fj_get(&d, root, "states");
    for (s = 0; fj_at(&d, states, s) >= 0 && b->nstate < FX_BIND_STATES; ++s) {
        int so = fj_at(&d, states, s), calls = fj_get(&d, so, "calls"), c;
        fx_bstate *st = &b->st[b->nstate++];
        snprintf(st->name, sizeof st->name, "%s", fj_s(&d, fj_get(&d, so, "state")));
        st->subaction = (int) fj_num(&d, fj_get(&d, so, "subaction"), -1);
        st->game_clock = strcmp(fj_s(&d, fj_get(&d, so, "clock")), "game") == 0;
        st->first = b->ncall;
        for (c = 0; fj_at(&d, calls, c) >= 0; ++c) {
            int co = fj_at(&d, calls, c), when = fj_get(&d, co, "when"), w, holds = 1, pk;
            const char *ev = fj_s(&d, fj_get(&d, co, "end_event")), *sit = fj_s(&d, fj_get(&d, co, "situation"));
            fx_bcall *k;
            for (w = 0; fj_at(&d, when, w) >= 0; ++w) {
                if (fj_num(&d, fj_get(&d, fj_at(&d, when, w), "holds"), 1) == 0) holds = 0;
            }
            if (!holds) { skipped++; continue; }
            pk = gw_Fx_Find(fj_s(&d, fj_get(&d, co, "package")));
            if (pk < 0) { missing++; continue; }
            if (b->ncall >= FX_BIND_CALLS || st->n >= FX_BIND_PER_STATE) break;
            k = &b->call[b->ncall++];
            st->n++;
            k->pkg = pk;
            k->joint = (int) fj_num(&d, fj_get(&d, co, "joint"), 0);
            k->follow = fj_num(&d, fj_get(&d, co, "follow"), 1) != 0;
            k->keep = strcmp(ev, "emitter_life") == 0;
            k->situation = strcmp(sit, "ground") == 0 ? 1 : strcmp(sit, "air") == 0 ? 2 : 0;
            k->frame = (float) fj_num(&d, fj_get(&d, co, "frame"), 0);
            k->end_frame = (strcmp(ev, "off") == 0 || strcmp(ev, "detach") == 0)
                               ? (float) fj_num(&d, fj_get(&d, co, "end_frame"), -1) : -1.0f;
            fj_vec(&d, fj_get(&d, co, "offset"), k->off, 3);
            fj_vec(&d, fj_get(&d, co, "rotation"), k->rot, 3);
            k->scale = (float) fj_num(&d, fj_get(&d, co, "scale"), 1);
            taken++;
        }
    }
    fj_free(&d);
    free(text);
    fx_bsets[fx_nbset] = b;
    gw_log("fx: bindings %s: %d states, %d calls (%d under a branch not taken, %d with no package)", path, b->nstate,
           taken, skipped, missing);
    return fx_nbset++;
}

static void fx_detach_tag(uint32_t mask, uint32_t val, int keep_too) {
    int i;
    for (i = 0; i < FX_MAX_INST; ++i) {
        fx_inst *in = &fx_cur.inst[i];
        const fx_emitter *e = in->used ? &fx_pkgs[in->pkg]->em[in->em] : NULL;
        /* keep (emitter_life) stays, unless its emitter never ends by itself */
        if (in->used && !in->detached && in->tag != 0 && (in->tag & mask) == val &&
            (keep_too || !in->keep || (!e->one_time && e->duration <= 0))) {
            in->detached = 1;
            in->detach_frame = fx_cur.frame;
        }
    }
}

/* T(offset) x R (X, then Y, then Z; degrees) x S(scale): a call's joint-local frame */
static void fx_call_local(const fx_bcall *k, float L[3][4]) {
    float cx = cosf(k->rot[0] * 0.017453293f), sx = sinf(k->rot[0] * 0.017453293f);
    float cy = cosf(k->rot[1] * 0.017453293f), sy = sinf(k->rot[1] * 0.017453293f);
    float cz = cosf(k->rot[2] * 0.017453293f), sz = sinf(k->rot[2] * 0.017453293f);
    float R[3][3] = {{cy * cz, sx * sy * cz - cx * sz, cx * sy * cz + sx * sz},
                     {cy * sz, sx * sy * sz + cx * cz, cx * sy * sz - sx * cz},
                     {-sy, sx * cy, cx * cy}}; /* Rz Ry Rx */
    int r, c;
    for (r = 0; r < 3; ++r) {
        for (c = 0; c < 3; ++c) L[r][c] = R[r][c] * k->scale;
        L[r][3] = k->off[r];
    }
}

/* Once per scene frame per fighter with a binding set, BEFORE gw_Fx_Frame(frame), after the fighter's joints are
 * final. owner: the fighter's key (guest address); motion: its motion state; anim: its subaction; anim_frame: its animation frame; time:
 * frames since its state began (Geno's action_time); parts: its FighterBone array (guest), stride / count;
 * joint_to_part: the kind's u8 table (guest; a binding names a joint of the model's tree, parts[] is by part). */
void gw_Fx_Drive(int set, int owner, int motion, int anim, float anim_frame, int time, int airborne, int facing, int port, int parts,
                 int stride, int nparts, int joint_to_part, int frame) {
    const fx_bset *b;
    fx_drv *dv;
    int i, slot = -1, st = -1;
    if (set < 0 || set >= fx_nbset || owner == 0) return;
    fx_rewind(frame);
    b = fx_bsets[set];
    for (i = 0; i < FX_MAX_DRV; ++i)
        if (fx_cur.drv[i].owner == (uint32_t) owner) { slot = i; break; }
    if (slot < 0) {
        for (i = 0; i < FX_MAX_DRV && slot < 0; ++i)
            if (fx_cur.drv[i].owner == 0) slot = i;
        if (slot < 0) /* all taken: the one not driven for longest */
            for (slot = 0, i = 1; i < FX_MAX_DRV; ++i)
                if (fx_cur.drv[i].last_frame < fx_cur.drv[slot].last_frame) slot = i;
        if (fx_cur.drv[slot].owner != 0) fx_detach_tag(0xFF000000u, 0x80000000u | ((uint32_t) slot << 24), 0);
        memset(&fx_cur.drv[slot], 0, sizeof fx_cur.drv[slot]);
        fx_cur.drv[slot].owner = (uint32_t) owner;
        fx_cur.drv[slot].anim = -1;
    }
    dv = &fx_cur.drv[slot];
    dv->set = set;
    if (dv->anim != anim || dv->motion != motion || time < dv->time) { /* a new state (or the same one again) */
        fx_detach_tag(0xFFFFFF00u, 0x80000000u | ((uint32_t) slot << 24) | ((uint32_t) (dv->serial & 0xFFFF) << 8), 0);
        dv->serial = (dv->serial + 1) & 0xFFFF;
        dv->fired = dv->ended = 0;
        dv->anim = anim;
        dv->motion = motion;
    }
    dv->time = time;
    dv->last_frame = frame;
    for (i = 0; i < b->nstate; ++i)
        if (b->st[i].subaction == anim) { st = i; break; }
    if (st < 0) return;
    for (i = 0; i < b->st[st].n && i < FX_BIND_PER_STATE; ++i) {
        const fx_bcall *k = &b->call[b->st[st].first + i];
        float clock = b->st[st].game_clock ? (float) time : anim_frame;
        uint32_t bit = 1u << i, tag = 0x80000000u | ((uint32_t) slot << 24) | ((uint32_t) (dv->serial & 0xFFFF) << 8) | (uint32_t) i;
        if (!(dv->fired & bit)) {
            fx_attach_extra ex;
            int jobj, h;
            if ((k->situation == 1 && airborne) || (k->situation == 2 && !airborne) || clock < k->frame) continue;
            dv->fired |= bit;
            {
                int part = k->joint;
                if (joint_to_part != 0 && part >= 0 && part < 256)
                    part = *(const unsigned char *) (uintptr_t) (joint_to_part + part);
                if (part < 0 || part >= nparts) {
                    gw_log("fx: bind %s call %d: joint %d (part %d) outside the fighter's %d parts", b->st[st].name, i,
                           k->joint, part, nparts);
                    continue;
                }
                jobj = (int) gw_r32((const void *) (uintptr_t) (parts + part * stride));
            }
            if (jobj == 0) continue;
            fx_call_local(k, ex.local);
            ex.tag = tag;
            ex.keep = k->keep;
            ex.follow = k->follow;
            ex.port = port;
            ex.joint = k->joint;
            fx_attach_ex = &ex;
            h = gw_Fx_Attach(k->pkg, jobj, 0x44, frame, facing);
            fx_attach_ex = NULL;
            if (h > 0) {
                const fx_inst *in = &fx_cur.inst[h - 1];
                const void *jm = (const void *) (uintptr_t) (jobj + 0x44);
                float cn[3];
                int cc, rr;
                for (cc = 0; cc < 3; ++cc) { /* the joint matrix's axis lengths: its scale */
                    float s2 = 0;
                    for (rr = 0; rr < 3; ++rr) { float x = gw_rf32((const char *) jm + (rr * 4 + cc) * 4); s2 += x * x; }
                    cn[cc] = sqrtf(s2);
                }
                gw_log("fx: bind %s call %d %s at %s frame %.1f (call frame %.1f) joint %d at (%.2f %.2f %.2f) scale "
                       "(%.3f %.3f %.3f) %s: offset (%.2f %.2f %.2f) x%.2f -> emitter origin (%.2f %.2f %.2f)",
                       b->st[st].name, i, fx_pkgs[k->pkg]->name, b->st[st].game_clock ? "state" : "anim", clock,
                       k->frame, k->joint, gw_rf32((const char *) jm + 12), gw_rf32((const char *) jm + 28),
                       gw_rf32((const char *) jm + 44), cn[0], cn[1], cn[2], k->follow ? "follow" : "world-fixed",
                       k->off[0], k->off[1], k->off[2], k->scale, in->m[0][3], in->m[1][3], in->m[2][3]);
            }
        } else if (!(dv->ended & bit) && k->end_frame >= 0 && clock >= k->end_frame) {
            dv->ended |= bit;
            fx_detach_tag(0xFFFFFFFFu, tag, 1);
            gw_log("fx: bind %s call %d %s off at %s frame %.1f", b->st[st].name, i, fx_pkgs[k->pkg]->name,
                   b->st[st].game_clock ? "state" : "anim", clock);
        }
    }
}

/* One row per attachment (not per emitter). Particle bounds use the visual
 * world position that the renderer consumes; empty attachments have no box. */
int gw_Fx_Query(int index, GwFxQuery *out) {
    int i, j, p, row = 0;
    if (!fx_ready || index < 0 || out == NULL) return 0;
    for (i = 0; i < FX_MAX_INST; ++i) {
        const fx_inst *in = &fx_cur.inst[i];
        int first = 1;
        if (!in->used) continue;
        for (j = 0; j < i; ++j)
            if (fx_cur.inst[j].used && fx_cur.inst[j].pkg == in->pkg &&
                fx_cur.inst[j].owner == in->owner && fx_cur.inst[j].attach_frame == in->attach_frame &&
                fx_cur.inst[j].tag == in->tag) { first = 0; break; }
        if (!first) continue;
        if (row++ != index) continue;
        memset(out, 0, sizeof *out);
        out->package = fx_pkgs[in->pkg]->name;
        out->owner_kind = in->owner_kind;
        out->port = in->port;
        out->joint = in->joint;
        out->x = in->pos[0]; out->y = in->pos[1]; out->z = in->pos[2];
        out->facing = in->facing;
        for (j = i; j < FX_MAX_INST; ++j) {
            const fx_inst *e = &fx_cur.inst[j];
            if (!e->used || e->pkg != in->pkg || e->owner != in->owner ||
                e->attach_frame != in->attach_frame || e->tag != in->tag) continue;
            ++out->emitters;
            for (p = 0; p < FX_MAX_PARTICLES; ++p) {
                const float *v;
                if (fx_cur.part[p].inst != j) continue;
                v = fx_cur.part[p].visual_pos;
                if (!out->has_bbox) {
                    out->min_x = out->max_x = v[0];
                    out->min_y = out->max_y = v[1];
                    out->min_z = out->max_z = v[2];
                    out->has_bbox = 1;
                } else {
                    if (v[0] < out->min_x) out->min_x = v[0];
                    if (v[1] < out->min_y) out->min_y = v[1];
                    if (v[2] < out->min_z) out->min_z = v[2];
                    if (v[0] > out->max_x) out->max_x = v[0];
                    if (v[1] > out->max_y) out->max_y = v[1];
                    if (v[2] > out->max_z) out->max_z = v[2];
                }
                ++out->particles;
            }
        }
        return 1;
    }
    return 0;
}

/* The census (numbers only): 0 live particles, 1 instances, 2 spawned, 3 killed, 4 refused,
 * 5 refused emitters (a subset of 4; other refusals are particles).
 * The old 16/100/200/300 selectors remain for indices 0..63. With 256 instances their ranges
 * overlap, so new callers use 1000/2000/3000/4000 + instance for count/forward/world X/world Z. */
int gw_Fx_Stat(int what) {
    int i, n = 0;
    if (!fx_ready) return 0;
    switch (what) {
    case 0: return fx_cur.nlive;
    case 1: for (i = 0; i < FX_MAX_INST; ++i) n += fx_cur.inst[i].used; return n;
    case 2: return (int) fx_cur.spawned;
    case 3: return (int) fx_cur.killed;
    case 4: return (int) fx_cur.refused;
    case 5: return (int) fx_cur.refused_emitters;
    default: break;
    }
    if ((what >= 16 && what < 80) || (what >= 1000 && what < 1000 + FX_MAX_INST)) {
        const int ii = what >= 1000 ? what - 1000 : what - 16;
        for (i = 0; i < FX_MAX_PARTICLES; ++i) n += fx_cur.part[i].inst == ii;
        return n;
    }
    if ((what >= 200 && what < 264) || (what >= 300 && what < 364) ||
        (what >= 3000 && what < 3000 + FX_MAX_INST) || (what >= 4000 && what < 4000 + FX_MAX_INST)) {
        const int ii = what >= 4000 ? what - 4000 : what >= 3000 ? what - 3000 : what % 100;
        const int axis = (what >= 4000 || (what >= 300 && what < 364)) ? 2 : 0;
        const fx_inst *in = &fx_cur.inst[ii];
        float sum = 0.0f;
        for (i = 0; i < FX_MAX_PARTICLES; ++i)
            if (fx_cur.part[i].inst == ii) { sum += fx_cur.part[i].pos[axis] - in->pos[axis]; n++; }
        return n ? (int) (100.0f * sum / (float) n) : 0;
    }
    if ((what >= 100 && what < 164) || (what >= 2000 && what < 2000 + FX_MAX_INST)) {
        const int ii = what >= 2000 ? what - 2000 : what - 100;
        const fx_inst *in = &fx_cur.inst[ii];
        float s = 0.0f;
        for (i = 0; i < FX_MAX_PARTICLES; ++i)
            if (fx_cur.part[i].inst == ii) {
                float d[3] = {fx_cur.part[i].pos[0] - in->pos[0], fx_cur.part[i].pos[1] - in->pos[1], fx_cur.part[i].pos[2] - in->pos[2]};
                s += d[0] * in->m[0][2] + d[1] * in->m[1][2] + d[2] * in->m[2][2];
                n++;
            }
        return n ? (int) (100.0f * s / (float) n) : 0;
    }
    return 0;
}

/* The census as a log line (a run's check): live particles per instance of `handle`'s package (0 = every
 * instance; gw_Fx_Frame logs that every 30 frames while anything is live). */
void gw_Fx_Census(int handle, int frame) {
    int i;
    char line[512];
    int o = 0;
    if (!fx_ready || handle < 0) return;
    line[0] = 0;
    for (i = handle > 0 ? handle - 1 : 0; i < FX_MAX_INST; ++i) {
        const fx_emitter *e;
        if (!fx_cur.inst[i].used) { if (handle > 0) break; continue; }
        if (handle > 0 && i > handle - 1 && (fx_cur.inst[i].pkg != fx_cur.inst[handle - 1].pkg || fx_cur.inst[i].em <= fx_cur.inst[i - 1].em)) break;
        e = &fx_pkgs[fx_cur.inst[i].pkg]->em[fx_cur.inst[i].em];
        o += snprintf(line + o, sizeof line - (size_t) o, " %s=%d", e->name, gw_Fx_Stat(1000 + i));
        if (o > (int) sizeof line - 40) break;
    }
    gw_log("fx: census frame %d: %d live, %d instances, spawned %d killed %d refused %d (emitters %d) |%s", frame,
           gw_Fx_Stat(0), gw_Fx_Stat(1), gw_Fx_Stat(2), gw_Fx_Stat(3), gw_Fx_Stat(4), gw_Fx_Stat(5), line);
}

/* ---- tests ---------------------------------------------------------------------------------------------- */
static int test_fx_sim_features(void) {
    static fx_pkg q;
    static float mtx[12];
    fx_emitter *e = &q.em[0];
    fx_part part;
    fx_inst inst;
    uint32_t rng = 0x12345678u;
    int i, h, rc = 0;
    float v[3], dir[3];
    memset(&q, 0, sizeof q);
    q.nem = 1;
    q.forward[0] = q.up[1] = 1.0f;
    e->scale[0] = e->scale[1] = 1.0f;
    e->scale_keys.value[0] = e->scale_keys.value[1] = 1.0f;
    e->alpha0.value[0] = e->alpha1.value[0] = 1.0f;
    e->param.value[0] = 1.0f;
    e->color_scale = 1.0f;
    e->wave_type = 8;
    e->wave_amplitude[0] = 0.5f;
    e->wave_cycle[0] = 4.0f;
    e->wave_apply[0] = e->wave_apply[1] = 1;
    memset(&part, 0, sizeof part);
    memset(&inst, 0, sizeof inst);
    inst.m[0][0] = inst.m[1][1] = inst.m[2][2] = 1.0f;
    part.age = 1; part.life = 8; part.scale = 1.0f;
    fx_visual_update(&part, e, &inst);
    if (fabsf(part.wave_offset[0] - 0.5f) > 0.001f || fabsf(part.visual_pos[0] - 0.5f) > 0.001f ||
        fabsf(part.visual_scale[0] - 1.5f) > 0.001f ||
        fabsf(part.visual_color0[3] - 1.5f) > 0.001f) {
        gw_test_fail("fx: wave frame 1 offset/scale/alpha expected 0.5/1.5/1.5"); rc = 1;
    }

    {   /* a four-frame colour ramp repeats after its period, independently of particle life */
        fx_curve c = {0};
        c.keyed = c.loop = 1; c.loop_rate = 4; c.n = 2;
        c.k[0][0] = 0.0f; c.k[0][3] = 0.0f;
        c.k[1][0] = 1.0f; c.k[1][3] = 1.0f;
        if (fabsf(fx_curve_part_at(&c, 5, 20, 0) - 0.25f) > 0.001f ||
            fabsf(fx_curve_part_at(&c, 7, 20, 0) - 0.75f) > 0.001f) {
            gw_test_fail("fx: curve loop at ages 5/7 expected 0.25/0.75"); rc = 1;
        }
    }

    {   /* frame cells at 25, 50, 75 percent of an eight-frame life; clamp and loop are age based */
        fx_sampler s = {0};
        s.pattern_count = 4; s.pattern_freq = 2.0f; s.div[0] = s.div[1] = 2;
        part.pattern_count[0] = 4; part.pattern_start[0] = 1; part.life = 8;
        s.pattern = FX_PAT_FIT_LIFE;
        for (i = 1; i <= 3; ++i) {
            part.age = i * 2;
            if (fx_pattern_cell(&s, &part, 0) != i) { gw_test_fail("fx: fit_life at %d/8 expected cell %d", part.age, i); rc = 1; }
        }
        s.pattern = FX_PAT_CLAMP; part.age = 10;
        if (fx_pattern_cell(&s, &part, 0) != 3) { gw_test_fail("fx: clamp cell at age 10 expected 3"); rc = 1; }
        s.pattern = FX_PAT_LOOP;
        if (fx_pattern_cell(&s, &part, 0) != 2) { gw_test_fail("fx: loop cell at age 10 plus phase 1 expected 2"); rc = 1; }
        s.pattern = FX_PAT_RANDOM; part.age = 4; part.seed = 0x12345678u;
        if (fx_pattern_cell(&s, &part, 0) != 1) {
            gw_test_fail("fx: seeded random pattern at age 4 expected atlas cell 1"); rc = 1;
        }
        s.pattern = FX_PAT_FIT_LIFE; s.table_n = 4;
        s.table[0] = 3; s.table[1] = 2; s.table[2] = 1; s.table[3] = 0; part.age = 4;
        if (fx_pattern_cell(&s, &part, 0) != 1) { gw_test_fail("fx: table remap at half life expected cell 1"); rc = 1; }
        e->smp[0] = s;
        fx_visual_update(&part, e, &inst);
        if (fabsf(part.pattern_uv[0][0] - 0.5f) > 0.001f || fabsf(part.pattern_uv[0][1]) > 0.001f) {
            gw_test_fail("fx: atlas cell 1 of 2x2 expected UV origin (0.5, 0)"); rc = 1;
        }
    }

    {   /* bounds for the source census's cylinder, plus the remaining shape families */
        const int shapes[] = {FX_SHAPE_CYLINDER, FX_SHAPE_CYLINDER_FILL, FX_SHAPE_BOX, FX_SHAPE_BOX_FILL,
                              FX_SHAPE_LINE, FX_SHAPE_LINE_DIVIDE, FX_SHAPE_RECTANGLE, FX_SHAPE_CIRCLE_DIVIDE,
                              FX_SHAPE_SPHERE_DIVIDE, FX_SHAPE_SPHERE_DIVIDE64};
        e->radius[0] = 2.0f; e->radius[1] = 3.0f; e->radius[2] = 4.0f;
        e->form_scale[0] = e->form_scale[1] = e->form_scale[2] = 1.0f;
        e->sweep[1] = 6.2831853f; e->sweep[2] = 3.1415927f;
        e->line[1] = 6.0f; e->divide[0] = e->divide[2] = 8;
        e->caliber = 0.5f;
        for (i = 0; i < (int) (sizeof shapes / sizeof shapes[0]); ++i) {
            int j;
            e->shape = shapes[i];
            for (j = 0; j < 64; ++j) {
                fx_sample_shape(e, &rng, v, dir);
                if (fabsf(v[0]) > 2.001f || fabsf(v[1]) > 3.001f || fabsf(v[2]) > 4.001f) {
                    gw_test_fail("fx: shape %d spawned outside radius 2/3/4", e->shape); rc = 1; break;
                }
                if (e->shape == FX_SHAPE_CYLINDER &&
                    fabsf(v[0] * v[0] / 4.0f + v[2] * v[2] / 16.0f - 1.0f) > 0.001f) {
                    gw_test_fail("fx: cylinder surface must lie on elliptical rim"); rc = 1; break;
                }
                if (e->shape == FX_SHAPE_CYLINDER_FILL &&
                    v[0] * v[0] / 4.0f + v[2] * v[2] / 16.0f < 0.25f - 0.001f) {
                    gw_test_fail("fx: cylinder caliber 0.5 must leave inner radius 0.5 empty"); rc = 1; break;
                }
            }
        }
        e->shape = FX_SHAPE_CIRCLE;
        e->sweep[0] = 0.0f; e->sweep[1] = 1.5707963f;
        for (i = 0; i < 64; ++i) {
            fx_sample_shape(e, &rng, v, dir);
            if (v[0] < -0.001f || v[2] < -0.001f) {
                gw_test_fail("fx: quarter-circle sweep must stay in positive X/Z quadrant"); rc = 1; break;
            }
        }
        e->shape = FX_SHAPE_SPHERE; e->sweep[1] = 6.2831853f; e->sweep[2] = 1.5707963f;
        for (i = 0; i < 64; ++i) {
            fx_sample_shape(e, &rng, v, dir);
            if (v[1] < -0.001f) { gw_test_fail("fx: hemisphere latitude sweep must stay above Y=0"); rc = 1; break; }
        }
    }

    {   /* detach keeps the particles alive while their alpha reaches zero in four frames */
        e->wave_type = 0; e->fade_on_stop = 1; e->fade_alpha_frames = 4;
        memset(&inst, 0, sizeof inst); inst.detached = 1; inst.detach_frame = 10;
        part.age = 2; part.life = 20; part.scale = 1.0f;
        fx_cur.frame = 10;
        fx_visual_update(&part, e, &inst);
        if (fabsf(part.fade_alpha - 0.75f) > 0.001f || fabsf(part.visual_color0[3] - 0.75f) > 0.001f) {
            gw_test_fail("fx: detach frame 1 alpha expected 0.75"); rc = 1;
        }
        fx_cur.frame = 11; fx_visual_update(&part, e, &inst);
        if (fabsf(part.fade_alpha - 0.5f) > 0.001f) { gw_test_fail("fx: detach frame 2 alpha expected 0.5"); rc = 1; }
        fx_reset(); fx_pkgs[fx_npkg] = &q; h = fx_npkg++;
        e->mesh = 1; /* no new emission in this synthetic case */
        e->mesh_idx = -1;
        fx_cur.frame = 10;
        fx_cur.inst[0] = inst;
        fx_cur.inst[0].used = 1; fx_cur.inst[0].pkg = h; fx_cur.inst[0].em = 0;
        fx_cur.part[0] = part;
        fx_cur.part[0].inst = 0;
        fx_cur.nlive = 1;
        for (i = 0; i < 3; ++i) {
            fx_cur.frame = 10 + i;
            fx_step();
            if (gw_Fx_Stat(0) != 1 || fabsf(fx_cur.part[0].fade_alpha - (0.75f - 0.25f * (float) i)) > 0.001f) {
                gw_test_fail("fx: fade frame %d expected one live particle and alpha %.2f", i + 1, 0.75f - 0.25f * (float) i);
                rc = 1;
            }
        }
        fx_cur.frame = 13; fx_step();
        if (gw_Fx_Stat(0) != 0 || gw_Fx_Stat(1) != 0) {
            gw_test_fail("fx: fade frame 4 must release particle and emitter"); rc = 1;
        }
        fx_npkg--; fx_reset();
    }

    {   /* actual spawn initializes UV offsets/angles from the seeded instance PRNG */
        fx_part first;
        fx_inst before;
        memset(&q.em[0], 0, sizeof q.em[0]);
        e->life = 20; e->scale[0] = e->scale[1] = 1.0f;
        e->scale_keys.value[0] = e->scale_keys.value[1] = 1.0f;
        e->color_scale = e->alpha0.value[0] = e->alpha1.value[0] = 1.0f;
        e->param.value[0] = 1.0f;
        e->form_scale[0] = e->form_scale[1] = e->form_scale[2] = 1.0f;
        e->smp[0].scroll_random[0] = 0.25f;
        e->smp[0].scale[0] = 1.0f; e->smp[0].scale_random[0] = 0.5f;
        e->smp[0].rotate_random = 0.75f;
        e->smp[0].pattern_count = 4; e->smp[0].pattern_random_start = 1;
        e->smp[0].div[0] = e->smp[0].div[1] = 2;
        memset(mtx, 0, sizeof mtx);
        for (i = 0; i < 12; i += 5) gw_wf32(&mtx[i], 1.0f);
        fx_reset(); fx_pkgs[fx_npkg] = &q; h = fx_npkg++;
        i = gw_Fx_Attach(h, (int) (uintptr_t) mtx, 0, 0, 1) - 1;
        before = fx_cur.inst[i];
        { float vel[3] = {0, 0, 0}; fx_spawn(i, &fx_cur.inst[i], e, vel); }
        first = fx_cur.part[0];
        if (fabsf(first.uv_scroll[0][0]) > 0.25f || first.uv_scale[0][0] < 0.5f ||
            first.uv_scale[0][0] > 1.5f || fabsf(first.uv_rotate[0]) > 0.75f ||
            first.pattern_start[0] < 0 || first.pattern_start[0] >= 4) {
            gw_test_fail("fx: seeded initial UV scroll/scale/rotation or flipbook phase outside expected range"); rc = 1;
        }
        fx_cur.inst[i] = before; fx_cur.part[0].inst = -1; fx_cur.nlive = 0; fx_cur.spawned = 0;
        { float vel[3] = {0, 0, 0}; fx_spawn(i, &fx_cur.inst[i], e, vel); }
        if (memcmp(&first, &fx_cur.part[0], sizeof first) != 0) {
            gw_test_fail("fx: repeated spawn after state restore must reproduce every UV and pattern random"); rc = 1;
        }
        fx_npkg--; fx_reset();
    }

    {   /* a full pool accepts the newest package's two highest-priority emitters only */
        fx_reset();
        q.nem = 3;
        q.em[0].priority = 0; q.em[1].priority = 1; q.em[2].priority = 2;
        fx_pkgs[fx_npkg] = &q; h = fx_npkg++;
        for (i = 0; i < FX_MAX_INST - 2; ++i) fx_cur.inst[i].used = 1;
        (void) gw_Fx_Attach(h, (int) (uintptr_t) mtx, 0, 0, 1);
        fx_cur.part[0].inst = 255;
        if (FX_MAX_INST != 256 || FX_MAX_PARTICLES != 2000 || gw_Fx_Stat(1) != 256 ||
            gw_Fx_Stat(4) != 1 || gw_Fx_Stat(5) != 1 ||
            fx_cur.inst[254].em != 0 || fx_cur.inst[255].em != 1 || gw_Fx_Stat(1255) != 1) {
            gw_test_fail("fx: 256-instance cap must refuse only priority-2 emitter (instances %d refused %d)",
                         gw_Fx_Stat(1), gw_Fx_Stat(4)); rc = 1;
        }
        fx_npkg--; fx_reset();
    }
    return rc;
}

static const char fx_test_pkg[] =
    "{\"geno_fx\":1,\"name\":\"t\",\"space\":{\"forward\":[0,0,1],\"up\":[0,1,0]},\"textures\":[],\"meshes\":[],\"emitters\":["
    "{\"name\":\"steady\",\"kind\":\"particle\",\"follow\":\"none\",\"transform\":{\"translate\":[0,0,0],\"rotate\":[0,0,0]},"
    " \"emission\":{\"start\":2,\"duration\":1,\"one_time\":false,\"rate\":1,\"interval\":2,\"by_distance\":false},"
    " \"shape\":{\"type\":\"point\",\"radius\":[0,0,0],\"caliber\":1},"
    " \"particle\":{\"life\":10,\"life_random_pct\":0,\"velocity\":{\"all_direction\":0,\"direction\":[0,0,-1],\"direction_scale\":0.5,"
    "  \"random_pct\":0,\"inherit\":0},\"forces\":{\"gravity_dir\":[0,1,0],\"gravity\":0,\"air_resistance\":1},"
    "  \"rotation\":{},\"scale\":{\"base\":[1,1],\"random_pct\":[0,0,0],\"keys\":[]}},\"color\":{\"scale\":1}},"
    "{\"name\":\"trail\",\"kind\":\"particle\",\"follow\":\"none\",\"transform\":{},"
    " \"emission\":{\"start\":0,\"rate\":1,\"interval\":0,\"by_distance\":{\"unit\":2,\"min\":0,\"max\":10,\"max_particles\":100}},"
    " \"shape\":{\"type\":\"sphere_fill\",\"radius\":[1,1,1],\"caliber\":1},"
    " \"particle\":{\"life\":4,\"velocity\":{\"direction\":[0,0,0]},\"forces\":{\"air_resistance\":1},\"scale\":{\"base\":[1,1]}},"
    " \"color\":{\"scale\":1}}]}";

static int test_fx_sim(void) {
    static float mtx[12];                  /* a fake guest JObj world matrix, big-endian (gw_rf32 reads it) */
    int pkg, h, f, rc = 0, live;
    GwFxQuery q;
    fx_pkg *p = fx_parse(fx_test_pkg, "test");
    if (p == NULL || p->nem != 2) { gw_test_fail("fx: the test package must parse with 2 emitters"); return 1; }
    fx_reset();
    fx_pkgs[fx_npkg] = p;
    pkg = fx_npkg++;
    {   /* a Melee item's root joint: identity rotation (it does not turn with the facing), moving +1 X a frame; the
         * package's space says +Z is its forward, so the effect's -Z (the steady emitter's velocity) is behind it */
        float m[12] = {1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0};
        int i;
        for (i = 0; i < 12; ++i) gw_wf32(&mtx[i], m[i]);
    }
    h = gw_Fx_Attach(pkg, (int) (uintptr_t) mtx, 0, 0, 1);
    gw_Fx_SetOwner(h, 2, 1);
    for (f = 1; f <= 20; ++f) {
        gw_wf32(&mtx[3], (float) f);        /* the owner travels +1 a frame */
        gw_Fx_Frame(f);
    }
    /* steady: from frame 3 one every 2 frames (3,5,..19), life 10 incl. its spawn frame -> 13..19 alive = 4 */
    if (gw_Fx_Stat(16 + h - 1) != 4) { gw_test_fail("fx: steady emitter: %d live (expect 4)", gw_Fx_Stat(16 + h - 1)); rc = 1; }
    /* its particles go backwards (local -Z = world -X here) at 0.5 a frame: mean offset behind the emitter */
    if (gw_Fx_Stat(100 + h - 1) >= 0) { gw_test_fail("fx: steady particles must trail behind (offset %d)", gw_Fx_Stat(100 + h - 1)); rc = 1; }
    if (gw_Fx_Stat(200 + h - 1) >= 0 || gw_Fx_Stat(300 + h - 1) != 0) {
        gw_test_fail("fx: facing right, the trail must be at world -X and depth 0 (x100: %d, z %d)", gw_Fx_Stat(200 + h - 1),
                     gw_Fx_Stat(300 + h - 1));
        rc = 1;
    }
    /* trail: by distance, 1 per 2 units at 1 unit a frame (spawns on frames 3,5,..19), life 4 -> only 19's alive */
    live = gw_Fx_Stat(16 + h);
    if (live != 1) { gw_test_fail("fx: by-distance emitter: %d live (expect 1)", live); rc = 1; }
    if (!gw_Fx_Query(0, &q) || q.emitters != 2 || q.particles != 5 || !q.has_bbox ||
        q.owner_kind != 2 || q.port != 1 || q.min_x > q.max_x || q.min_y > q.max_y ||
        q.min_z > q.max_z || gw_Fx_Query(1, &q)) {
        gw_test_fail("fx: read-only query did not aggregate the two emitter instances");
        rc = 1;
    }
    /* rollback: back to frame 15 and forward again gives the same state */
    {
        int before = gw_Fx_Stat(2), l0 = gw_Fx_Stat(0);
        gw_wf32(&mtx[3], 15.0f);
        for (f = 15; f <= 20; ++f) { gw_wf32(&mtx[3], (float) f); gw_Fx_Frame(f); }
        if (gw_Fx_Stat(0) != l0 || gw_Fx_Stat(2) != before) {
            gw_test_fail("fx: re-simulating frames 15-20 must give the same state (live %d/%d spawned %d/%d)", gw_Fx_Stat(0), l0,
                         gw_Fx_Stat(2), before);
            rc = 1;
        }
    }
    {   /* the same effect facing left mirrors: the trail at world +X */
        static float mtx2[12];
        float m[12] = {1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0};
        int i, h2;
        for (i = 0; i < 12; ++i) gw_wf32(&mtx2[i], m[i]);
        h2 = gw_Fx_Attach(pkg, (int) (uintptr_t) mtx2, 0, 21, -1);
        for (f = 21; f <= 34; ++f) { gw_wf32(&mtx2[3], (float) -(f - 21)); gw_Fx_Frame(f); }
        if (gw_Fx_Stat(200 + h2 - 1) <= 0) { gw_test_fail("fx: facing left, the trail must be at world +X (x100: %d)", gw_Fx_Stat(200 + h2 - 1)); rc = 1; }
        gw_Fx_Detach((int) (uintptr_t) mtx2);
    }
    gw_Fx_Detach((int) (uintptr_t) mtx);
    for (f = 35; f <= 60; ++f) gw_Fx_Frame(f);
    if (gw_Fx_Stat(0) != 0 || gw_Fx_Stat(1) != 0) { gw_test_fail("fx: after detach everything must die out (%d live, %d inst)", gw_Fx_Stat(0), gw_Fx_Stat(1)); rc = 1; }
    free(p);
    fx_npkg--;
    fx_reset();
    rc |= test_fx_sim_features();
    return rc;
}

void gw_fx_tests_register(void) { gw_test_register("fx_sim", test_fx_sim); }
