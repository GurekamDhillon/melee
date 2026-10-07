/* A recording AtSink: every poly, text and model call is kept for the test to read. */
#ifndef ATLAS_REC_H
#define ATLAS_REC_H
#include <math.h>
#include <stdio.h>
#include <string.h>
#include "atlas_fake.h"
#include "../platform/gw_ui_parts.h"
#include "../platform/gw_ui_tokens.h"

#define REC_POLYS 4096
typedef struct { float x[4], y[4]; unsigned rgba; } RecPoly;
typedef struct { float x, base; char s[160]; int role; unsigned rgba; int align; } RecText;
typedef struct { int model, ring; float x, y, w, h; int focused, dim; } RecModel;
typedef struct { RecPoly p[REC_POLYS]; int np; RecText t[512]; int nt; RecModel m[64]; int nm; } Rec;
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
static AtSink rec_sink(void)
{
    AtSink s;
    memset(&REC, 0, sizeof REC);
    s.user = &REC; s.poly = rec_poly; s.text = rec_text; s.model = rec_model;
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
/* ---- the three style checks every part and screen test shares (step 3, Task 2) ---- */
/* the right and left edge of a recorded text, whatever its alignment (measured with the fake width the tests draw with) */
static float text_right(const RecText *t)
{
    float w = fake_width(0, t->role, t->s);
    return t->align == AT_ALIGN_RIGHT ? t->x : t->align == AT_ALIGN_CENTER ? t->x + w * 0.5f : t->x + w;
}
static float text_left(const RecText *t)
{
    float w = fake_width(0, t->role, t->s);
    return t->align == AT_ALIGN_RIGHT ? t->x - w : t->align == AT_ALIGN_CENTER ? t->x - w * 0.5f : t->x;
}
/* 1 when no polygon vertex lies inside the cut-away triangle of the top-left or bottom-right chamfer of r */
static int corners_clear(AtRect r, float c)
{
    int i, k;
    for (i = 0; i < REC.np; i++) {
        for (k = 0; k < 4; k++) {
            float dx = REC.p[i].x[k] - r.x, dy = REC.p[i].y[k] - r.y, ex = r.x + r.w - REC.p[i].x[k], ey = r.y + r.h - REC.p[i].y[k];
            if (dx >= -0.01f && dy >= -0.01f && dx + dy < c - 0.01f) return 0;
            if (ex >= -0.01f && ey >= -0.01f && ex + ey < c - 0.01f) return 0;
        }
    }
    return 1;
}
/* 1 when every recorded text lies inside r, measured with the width that drew it */
static int texts_inside(AtRect r)
{
    int i;
    for (i = 0; i < REC.nt; i++)
        if (text_left(&REC.t[i]) < r.x - 0.01f || text_right(&REC.t[i]) > r.x + r.w + 0.01f) {
            printf("  text \"%s\" leaves its box [%g, %g]\n", REC.t[i].s, r.x, r.x + r.w);
            return 0;
        }
    return 1;
}
/* The focus cues of a part drawn at r: 1 for the lift (a lifted face whose top is r.y - 2), 1 for the ember front edge (a wide ember
 * strip along the bottom), 1 for the tick (rows: a 4 px ember strip at the left edge) or for the brackets (cells and cards: eight
 * strips reaching outside r). A part at rest has none (no_focus_cues). */
static int focus_cues_at(AtRect r, int is_cell)
{
    int i, lift = 0, edge = 0, tick = 0, br = 0;
    for (i = 0; i < REC.np; i++) {
        const RecPoly *p = &REC.p[i];
        if (p->rgba == AT_C_LIFT && fabsf(poly_miny(p) - (r.y - 2.0f)) < 0.6f) lift = 1;
        if (p->rgba == AT_C_EMBER && poly_maxy(p) >= r.y + r.h - 2.01f && poly_maxy(p) <= r.y + r.h + 0.01f && poly_maxx(p) - poly_minx(p) > r.w * 0.5f) edge = 1;
        if (!is_cell && p->rgba == AT_C_EMBER && poly_maxx(p) - poly_minx(p) <= 4.01f && poly_minx(p) <= r.x + 0.01f) tick = 1;
        if (is_cell && (poly_minx(p) < r.x - 0.01f || poly_maxx(p) > r.x + r.w + 0.01f) && p->rgba != AT_C_LIFT) br++;
    }
    return lift + edge + (is_cell ? (br >= 8) : tick);
}
/* no ember and no lift anywhere: nothing on screen shows focus */
static int no_focus_cues(void) { return count_color(AT_C_EMBER) == 0 && count_color(AT_C_LIFT) == 0; }
#endif
