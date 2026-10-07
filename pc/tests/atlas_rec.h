/* A recording AtSink: every poly, text and model call is kept for the test to read. */
#ifndef ATLAS_REC_H
#define ATLAS_REC_H
#include <math.h>
#include <stdio.h>
#include <string.h>
#include "../platform/gw_ui_parts.h"

#define REC_POLYS 4096
typedef struct { float x[4], y[4]; unsigned rgba; } RecPoly;
typedef struct { float x, base; char s[160]; int role; unsigned rgba; int align; } RecText;
typedef struct { int model, ring; float x, y, w, h; int focused, dim; } RecModel;
typedef struct { int tex; float x, y, w, h; unsigned rgba; } RecImage;
typedef struct { RecPoly p[REC_POLYS]; int np; RecText t[512]; int nt; RecModel m[64]; int nm; RecImage im[64]; int ni; } Rec;
static Rec REC;

static void rec_poly(void *u, const float x[4], const float y[4], unsigned c)
{
    Rec *r = (Rec *) u;
    if (r->np < REC_POLYS) { memcpy(r->p[r->np].x, x, sizeof r->p[0].x); memcpy(r->p[r->np].y, y, sizeof r->p[0].y); r->p[r->np].rgba = c; r->np++; }
}
static void rec_text(void *u, float x, float base, const char *s, int role, unsigned c, int align, float max_w)
{
    Rec *r = (Rec *) u;
    (void) max_w;
    if (r->nt < 512) { RecText *t = &r->t[r->nt++]; t->x = x; t->base = base; snprintf(t->s, sizeof t->s, "%s", s); t->role = role; t->rgba = c; t->align = align; }
}
static void rec_model(void *u, int model, int ring, float x, float y, float w, float h, int focused, int dim)
{
    Rec *r = (Rec *) u;
    if (r->nm < 64) { RecModel *m = &r->m[r->nm++]; m->model = model; m->ring = ring; m->x = x; m->y = y; m->w = w; m->h = h; m->focused = focused; m->dim = dim; }
}
#ifdef AT_SINK_HAS_IMAGE
static void rec_image(void *u, int tex, float x, float y, float w, float h, unsigned c)
{
    Rec *r = (Rec *) u;
    if (r->ni < 64) { RecImage *m = &r->im[r->ni++]; m->tex = tex; m->x = x; m->y = y; m->w = w; m->h = h; m->rgba = c; }
}
#endif
static AtSink rec_sink(void)
{
    AtSink s;
    memset(&REC, 0, sizeof REC);
    memset(&s, 0, sizeof s);
    s.user = &REC; s.poly = rec_poly; s.text = rec_text; s.model = rec_model;
#ifdef AT_SINK_HAS_IMAGE
    s.image = rec_image;
#endif
    return s;
}
static float poly_area(const RecPoly *p)
{
    float a = 0.0f; int i;
    for (i = 0; i < 4; i++) { int j = (i + 1) % 4; a += p->x[i] * p->y[j] - p->x[j] * p->y[i]; }
    return (float) fabs(a) * 0.5f;
}
static float poly_minx(const RecPoly *p) { float m = p->x[0]; int i; for (i = 1; i < 4; i++) if (p->x[i] < m) m = p->x[i]; return m; }
static float poly_maxx(const RecPoly *p) { float m = p->x[0]; int i; for (i = 1; i < 4; i++) if (p->x[i] > m) m = p->x[i]; return m; }
static float poly_miny(const RecPoly *p) { float m = p->y[0]; int i; for (i = 1; i < 4; i++) if (p->y[i] < m) m = p->y[i]; return m; }
static float poly_maxy(const RecPoly *p) { float m = p->y[0]; int i; for (i = 1; i < 4; i++) if (p->y[i] > m) m = p->y[i]; return m; }
static int count_color(unsigned rgba) { int i, n = 0; for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == rgba) n++; return n; }
static const RecText *find_text(const char *s) { int i; for (i = 0; i < REC.nt; i++) if (strcmp(REC.t[i].s, s) == 0) return &REC.t[i]; return NULL; }
static int texts_legible(void) { int i; for (i = 0; i < REC.nt; i++) if (at_role_size(REC.t[i].role) < 12) return 0; return 1; }
#endif
