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
 * Budget (accepted 2026-09-26): 2000 live particles, 64 emitter instances per match. */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "gw_mods.h"
#include "gw_test.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define FX_MAX_PKGS 32
#define FX_MAX_EMITTERS 32      /* per package */
#define FX_MAX_INST 64
#define FX_MAX_PARTICLES 2000
#define FX_RING 16              /* frames of state kept for a rollback */
#define FX_KEYS 8

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

/* ---- definitions -------------------------------------------------------------------------------------- */
enum { FX_SHAPE_POINT, FX_SHAPE_SPHERE, FX_SHAPE_SPHERE_FILL, FX_SHAPE_CIRCLE, FX_SHAPE_CIRCLE_FILL, FX_SHAPE_OTHER };

typedef struct { int n; float k[FX_KEYS][4]; float value[3]; int keyed; } fx_curve;

typedef struct {
    char name[48];
    int mesh, follow;           /* follow: 0 srt, 1 none, 2 translate */
    float trans[3], rot[3];
    int start, duration, one_time, interval;
    float rate, rate_random;
    int by_dist, dist_max_particles;
    float dist_unit, dist_min, dist_max;
    int shape;
    float radius[3], caliber;
    int life, infinite;
    float life_random;          /* 0..1 */
    float vel_all, vel_dir[3], vel_dir_scale, vel_random, inherit;
    float grav_dir[3], grav, air;
    int grav_world;
    float scale[2], scale_random;
    fx_curve scale_keys, color0, alpha0, color1, alpha1;
    float color_scale;
    float rot_init[3], rot_init_random[3], rot_add[3], rot_add_random[3];
} fx_emitter;

typedef struct {
    char name[64];
    char path[MAX_PATH];
    int nem;
    fx_emitter em[FX_MAX_EMITTERS];
    int ntex, nmesh;
} fx_pkg;

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

static int fx_shape(const char *s) {
    if (!strcmp(s, "sphere")) return FX_SHAPE_SPHERE;
    if (!strcmp(s, "sphere_fill")) return FX_SHAPE_SPHERE_FILL;
    if (!strcmp(s, "circle")) return FX_SHAPE_CIRCLE;
    if (!strcmp(s, "circle_fill")) return FX_SHAPE_CIRCLE_FILL;
    if (!strcmp(s, "point")) return FX_SHAPE_POINT;
    return FX_SHAPE_OTHER;
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
    p->ntex = 0;
    for (i = 0; fj_at(&d, fj_get(&d, root, "textures"), i) >= 0; ++i) p->ntex++;
    for (i = 0; fj_at(&d, fj_get(&d, root, "meshes"), i) >= 0; ++i) p->nmesh++;
    ems = fj_get(&d, root, "emitters");
    for (i = 0; (e = fj_at(&d, ems, i)) >= 0 && p->nem < FX_MAX_EMITTERS; ++i) {
        fx_emitter *m = &p->em[p->nem++];
        int t = fj_get(&d, e, "transform"), em = fj_get(&d, e, "emission"), sh = fj_get(&d, e, "shape");
        int pa = fj_get(&d, e, "particle"), co = fj_get(&d, e, "color");
        int ve = fj_get(&d, pa, "velocity"), fo = fj_get(&d, pa, "forces"), ro = fj_get(&d, pa, "rotation");
        int sc = fj_get(&d, pa, "scale"), bd = fj_get(&d, em, "by_distance");
        const char *fl = fj_s(&d, fj_get(&d, e, "follow"));
        memset(m, 0, sizeof *m);
        snprintf(m->name, sizeof m->name, "%s", fj_s(&d, fj_get(&d, e, "name")));
        m->mesh = strcmp(fj_s(&d, fj_get(&d, e, "kind")), "mesh") == 0;
        m->follow = !strcmp(fl, "none") ? 1 : !strcmp(fl, "translate") ? 2 : 0;
        fj_vec(&d, fj_get(&d, t, "translate"), m->trans, 3);
        fj_vec(&d, fj_get(&d, t, "rotate"), m->rot, 3);
        m->start = (int) fj_num(&d, fj_get(&d, em, "start"), 0);
        m->duration = (int) fj_num(&d, fj_get(&d, em, "duration"), 0);
        m->one_time = (int) fj_num(&d, fj_get(&d, em, "one_time"), 0);
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
        m->caliber = (float) fj_num(&d, fj_get(&d, sh, "caliber"), 1);
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
        fj_vec(&d, fj_get(&d, ro, "init"), m->rot_init, 3);
        fj_vec(&d, fj_get(&d, ro, "init_random"), m->rot_init_random, 3);
        fj_vec(&d, fj_get(&d, ro, "add"), m->rot_add, 3);
        fj_vec(&d, fj_get(&d, ro, "add_random"), m->rot_add_random, 3);
        (void) fj_path;
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

/* ---- simulation state (one block, so the rollback ring is a plain copy) ------------------------------ */
typedef struct {
    int used, pkg, em, detached, attach_frame, age, emitted;
    uint32_t owner, mtx_off, rng;
    float accum, dist_accum;
    float pos[3], prev[3], m[3][4];
    int have_prev;
} fx_inst;

typedef struct {
    int inst;                   /* -1 = free */
    int age, life;
    float pos[3], vel[3], scale, rot, rot_add;
    uint32_t seed;
} fx_part;

typedef struct {
    int frame;                  /* the game frame this state is at the end of */
    fx_inst inst[FX_MAX_INST];
    fx_part part[FX_MAX_PARTICLES];
    int nlive;
    uint32_t spawned, killed, refused;
} fx_state;

int gw_Fx_Stat(int what);
void gw_Fx_Census(int handle, int frame);

static fx_state fx_cur;
static fx_state fx_ring[FX_RING];
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

static float fx_curve_at(const fx_curve *c, float t, int ch) {
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

/* the owner's world matrix (3x4, big-endian floats in guest memory) */
static void fx_owner_mtx(fx_inst *in) {
    int r, c;
    if (in->owner == 0 || in->detached) return;
    for (r = 0; r < 3; ++r)
        for (c = 0; c < 4; ++c) in->m[r][c] = gw_rf32((const void *) (uintptr_t) (in->owner + in->mtx_off + (r * 4 + c) * 4));
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
    /* position in the emitter volume */
    if (e->shape == FX_SHAPE_SPHERE || e->shape == FX_SHAPE_SPHERE_FILL) {
        float u = fx_rndf(&p->seed) * 2 - 1, th = fx_rndf(&p->seed) * 6.2831853f, r = sqrtf(1 - u * u), rad = 1.0f;
        if (e->shape == FX_SHAPE_SPHERE_FILL) {
            float in_ = 1.0f - e->caliber;          /* hollow ratio: caliber 1 = full */
            rad = cbrtf(in_ * in_ * in_ + fx_rndf(&p->seed) * (1 - in_ * in_ * in_));
        }
        dir[0] = r * cosf(th); dir[1] = r * sinf(th); dir[2] = u;
        for (k = 0; k < 3; ++k) local[k] = dir[k] * e->radius[k] * rad;
    } else if (e->shape == FX_SHAPE_CIRCLE || e->shape == FX_SHAPE_CIRCLE_FILL) {
        float th = fx_rndf(&p->seed) * 6.2831853f, rad = e->shape == FX_SHAPE_CIRCLE_FILL ? sqrtf(fx_rndf(&p->seed)) : 1.0f;
        dir[0] = cosf(th); dir[1] = 0; dir[2] = sinf(th);
        local[0] = dir[0] * e->radius[0] * rad; local[2] = dir[2] * e->radius[2] * rad;
    } else {
        float u = fx_rndf(&p->seed) * 2 - 1, th = fx_rndf(&p->seed) * 6.2831853f, r = sqrtf(1 - u * u);
        dir[0] = r * cosf(th); dir[1] = r * sinf(th); dir[2] = u;
    }
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
    len = e->life_random > 0 ? 1.0f - e->life_random * fx_rndf(&p->seed) : 1.0f;
    p->life = (int) (e->life * len + 0.5f);
    if (p->life < 1) p->life = 1;
    p->scale = 1.0f - e->scale_random * fx_rndf(&p->seed);
    p->rot = e->rot_init[2] + e->rot_init_random[2] * fx_rndf(&p->seed);
    p->rot_add = e->rot_add[2] + e->rot_add_random[2] * (fx_rndf(&p->seed) * 2 - 1);
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
        if (!in->detached && !e->mesh && in->age > e->start) {
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
        in->prev[0] = in->pos[0]; in->prev[1] = in->pos[1]; in->prev[2] = in->pos[2];
        in->have_prev = 1;
    }
    for (k = 0; k < FX_MAX_PARTICLES; ++k) {
        fx_part *p = &fx_cur.part[k];
        const fx_emitter *e;
        int c;
        if (p->inst < 0) continue;
        e = &fx_pkgs[fx_cur.inst[p->inst].pkg]->em[fx_cur.inst[p->inst].em];
        for (c = 0; c < 3; ++c) {
            p->vel[c] *= e->air;
            p->vel[c] += e->grav_dir[c] * e->grav;
            p->pos[c] += p->vel[c];
        }
        p->rot += p->rot_add;
        if (++p->age >= p->life && !e->infinite) { p->inst = -1; fx_cur.nlive--; fx_cur.killed++; }
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

/* ---- the API the game half calls (scalars; guest addresses as ints) ---------------------------------- */

/* Once per logic frame from the game half, with ITS frame counter (game memory: a rollback restores it). */
void gw_Fx_Frame(int frame) {
    if (!fx_ready) fx_reset();
    if (frame <= fx_cur.frame) {
        int was = fx_cur.frame;
        /* a rollback / LAB rewind: restore the state at the end of frame - 1 and let the resim step again */
        const fx_state *s = &fx_ring[(frame - 1) & (FX_RING - 1)];
        if (frame >= 1 && s->frame == frame - 1) fx_cur = *s;
        else fx_reset();
        gw_log("fx: frame %d after %d - state restored to %d (re-simulating)", frame, was, fx_cur.frame);
    }
    fx_step();
    fx_cur.frame = frame;
    if ((frame % 30) == 0 && (fx_cur.nlive > 0 || gw_Fx_Stat(1) > 0)) gw_Fx_Census(0, frame);
    fx_ring[frame & (FX_RING - 1)] = fx_cur;
}

/* Attach every emitter of package `pkg` to the guest JObj at `owner` (world matrix at `mtx_off`); returns a
 * handle (the first instance index + 1) or 0. */
int gw_Fx_Attach(int pkg, int owner, int mtx_off, int frame) {
    int e, i, first = -1;
    if (!fx_ready) fx_reset();
    if (pkg < 0 || pkg >= fx_npkg) return 0;
    for (e = 0; e < fx_pkgs[pkg]->nem; ++e) {
        for (i = 0; i < FX_MAX_INST && fx_cur.inst[i].used; ++i) {}
        if (i == FX_MAX_INST) { gw_log("fx: %s: more than %d emitter instances - the rest are not attached", fx_pkgs[pkg]->name, FX_MAX_INST); break; }
        memset(&fx_cur.inst[i], 0, sizeof fx_cur.inst[i]);
        fx_cur.inst[i].used = 1;
        fx_cur.inst[i].pkg = pkg;
        fx_cur.inst[i].em = e;
        fx_cur.inst[i].owner = (uint32_t) owner;
        fx_cur.inst[i].mtx_off = (uint32_t) mtx_off;
        fx_cur.inst[i].attach_frame = frame;
        fx_cur.inst[i].rng = 0x811C9DC5u ^ ((uint32_t) pkg * 16777619u) ^ ((uint32_t) e << 8) ^ (uint32_t) owner ^ ((uint32_t) frame << 16);
        fx_owner_mtx(&fx_cur.inst[i]);
        if (first < 0) first = i;
    }
    gw_log("fx: %s attached to 0x%08X at frame %d: %d emitter(s)", fx_pkgs[pkg]->name, (uint32_t) owner, frame, fx_pkgs[pkg]->nem);
    return first + 1;
}

/* The owner is going away: its emitters stop emitting; their live particles finish their lives. */
void gw_Fx_Detach(int owner) {
    int i;
    for (i = 0; i < FX_MAX_INST; ++i)
        if (fx_cur.inst[i].used && fx_cur.inst[i].owner == (uint32_t) owner) fx_cur.inst[i].detached = 1;
}

/* The census (numbers only): what = 0 live particles, 1 emitter instances, 2 spawned, 3 killed, 4 refused;
 * 16 + i = live particles of instance i; 100 + i = instance i's mean particle offset from its emitter along
 * the owner's local Z (the effect's forward) axis x100. */
int gw_Fx_Stat(int what) {
    int i, n = 0;
    if (!fx_ready) return 0;
    switch (what) {
    case 0: return fx_cur.nlive;
    case 1: for (i = 0; i < FX_MAX_INST; ++i) n += fx_cur.inst[i].used; return n;
    case 2: return (int) fx_cur.spawned;
    case 3: return (int) fx_cur.killed;
    case 4: return (int) fx_cur.refused;
    default: break;
    }
    if (what >= 16 && what < 16 + FX_MAX_INST) {
        for (i = 0; i < FX_MAX_PARTICLES; ++i) n += fx_cur.part[i].inst == what - 16;
        return n;
    }
    if (what >= 100 && what < 100 + FX_MAX_INST) {
        const fx_inst *in = &fx_cur.inst[what - 100];
        float s = 0.0f;
        for (i = 0; i < FX_MAX_PARTICLES; ++i)
            if (fx_cur.part[i].inst == what - 100) {
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
        o += snprintf(line + o, sizeof line - (size_t) o, " %s=%d", e->name, gw_Fx_Stat(16 + i));
        if (o > (int) sizeof line - 40) break;
    }
    gw_log("fx: census frame %d: %d live, %d instances, spawned %d killed %d refused %d |%s", frame, gw_Fx_Stat(0),
           gw_Fx_Stat(1), gw_Fx_Stat(2), gw_Fx_Stat(3), gw_Fx_Stat(4), line);
}

/* ---- tests ---------------------------------------------------------------------------------------------- */
static const char fx_test_pkg[] =
    "{\"geno_fx\":1,\"name\":\"t\",\"textures\":[],\"meshes\":[],\"emitters\":["
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
    fx_pkg *p = fx_parse(fx_test_pkg, "test");
    if (p == NULL || p->nem != 2) { gw_test_fail("fx: the test package must parse with 2 emitters"); return 1; }
    fx_reset();
    fx_pkgs[fx_npkg] = p;
    pkg = fx_npkg++;
    {   /* identity rotation, moving 1 unit per frame along +X (Z forward in the effect's frame -> world X) */
        float m[12] = {0, 0, 1, 0, 0, 1, 0, 0, -1, 0, 0, 0};
        int i;
        for (i = 0; i < 12; ++i) gw_wf32(&mtx[i], m[i]);
    }
    h = gw_Fx_Attach(pkg, (int) (uintptr_t) mtx, 0, 0);
    for (f = 1; f <= 20; ++f) {
        gw_wf32(&mtx[3], (float) f);        /* the owner travels +1 a frame */
        gw_Fx_Frame(f);
    }
    /* steady: from frame 3 one every 2 frames (3,5,..19), life 10 incl. its spawn frame -> 13..19 alive = 4 */
    if (gw_Fx_Stat(16 + h - 1) != 4) { gw_test_fail("fx: steady emitter: %d live (expect 4)", gw_Fx_Stat(16 + h - 1)); rc = 1; }
    /* its particles go backwards (local -Z = world -X here) at 0.5 a frame: mean offset behind the emitter */
    if (gw_Fx_Stat(100 + h - 1) >= 0) { gw_test_fail("fx: steady particles must trail behind (offset %d)", gw_Fx_Stat(100 + h - 1)); rc = 1; }
    /* trail: by distance, 1 per 2 units at 1 unit a frame (spawns on frames 3,5,..19), life 4 -> only 19's alive */
    live = gw_Fx_Stat(16 + h);
    if (live != 1) { gw_test_fail("fx: by-distance emitter: %d live (expect 1)", live); rc = 1; }
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
    gw_Fx_Detach((int) (uintptr_t) mtx);
    for (f = 21; f <= 40; ++f) gw_Fx_Frame(f);
    if (gw_Fx_Stat(0) != 0 || gw_Fx_Stat(1) != 0) { gw_test_fail("fx: after detach everything must die out (%d live, %d inst)", gw_Fx_Stat(0), gw_Fx_Stat(1)); rc = 1; }
    free(p);
    fx_npkg--;
    fx_reset();
    return rc;
}

void gw_fx_tests_register(void) { gw_test_register("fx_sim", test_fx_sim); }
