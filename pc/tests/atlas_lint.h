/* Style-rule checks over a recorded sink (atlas_rec.h), used by every Atlas screen test. Each returns the number of
 * violations. The rules are the spec's: 4.1 chamfers (top-left and bottom-right only), 4.6 focus (three cues at once),
 * 10 text (12 px floor, nothing outside its plate, nothing on top of other text). Include once per test. */
#ifndef ATLAS_LINT_H
#define ATLAS_LINT_H
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
#include "../platform/gw_ui_tokens.h"

/* is the point inside a recorded convex quad (either winding; a triangle has a repeated vertex) */
static int lint_in_poly(const RecPoly *p, float x, float y)
{
    int i, pos = 0, neg = 0;
    for (i = 0; i < 4; i++) {
        int j = (i + 1) % 4;
        float cr = (p->x[j] - p->x[i]) * (y - p->y[i]) - (p->y[j] - p->y[i]) * (x - p->x[i]);
        if (cr > 0.001f) pos++; else if (cr < -0.001f) neg++;
    }
    return pos == 0 || neg == 0;
}

static int lint_covered(float x, float y)
{
    int i;
    for (i = 0; i < REC.np; i++) if (lint_in_poly(&REC.p[i], x, y)) return 1;
    return 0;
}

/* The chamfer rule. A plate of rectangle r with chamfer c > 2 px leaves the pixel just inside its top-left and its
 * bottom-right corner empty and the other two covered. (Probing the silhouette, not the polygons, because a plate is
 * several convex quads and their inner diagonals say nothing about the corners.) Probe ONE plate on a fresh sink. */
static int lint_plate_corners(AtRect r, float c)
{
    int bad = 0;
    if (c <= 2.0f) return 0;
    if (lint_covered(r.x + 1.0f, r.y + 1.0f)) bad++;                    /* top-left must be cut */
    if (lint_covered(r.x + r.w - 1.0f, r.y + r.h - 1.0f)) bad++;        /* bottom-right must be cut */
    if (!lint_covered(r.x + r.w - 1.0f, r.y + 1.0f)) bad++;             /* top-right must be square */
    if (!lint_covered(r.x + 1.0f, r.y + r.h - 1.0f)) bad++;             /* bottom-left must be square */
    return bad;
}

static void lint_text_box(const RecText *t, float *x0, float *x1, float *y0, float *y1)
{
    float w = FAKE.width(FAKE.user, t->role, t->s), sz = (float) at_role_size(t->role);
    *x0 = t->align == AT_ALIGN_LEFT ? t->x : t->align == AT_ALIGN_CENTER ? t->x - w * 0.5f : t->x - w;
    *x1 = *x0 + w;
    *y0 = t->base - sz * 0.8f;
    *y1 = t->base + sz * 0.2f;
}

/* texts [from, REC.nt) must lie inside r, and be at least 12 px */
static int lint_text_inside(AtRect r, int from)
{
    int i, bad = 0;
    for (i = from; i < REC.nt; i++) {
        float x0, x1, y0, y1;
        lint_text_box(&REC.t[i], &x0, &x1, &y0, &y1);
        if (at_role_size(REC.t[i].role) < 12) { bad++; continue; }
        if (x0 < r.x - 0.5f || x1 > r.x + r.w + 0.5f || y0 < r.y - 0.5f || y1 > r.y + r.h + 0.5f) bad++;
    }
    return bad;
}

/* no two texts overlap by more than 1 px in both axes (overflow into a neighbour) */
static int lint_text_overlaps(void)
{
    int i, j, bad = 0;
    for (i = 0; i < REC.nt; i++) for (j = i + 1; j < REC.nt; j++) {
        float a0, a1, b0, b1, c0, c1, d0, d1;
        lint_text_box(&REC.t[i], &a0, &a1, &b0, &b1);
        lint_text_box(&REC.t[j], &c0, &c1, &d0, &d1);
        if (a0 < c1 - 1.0f && c0 < a1 - 1.0f && b0 < d1 - 1.0f && d0 < b1 - 1.0f) bad++;
    }
    return bad;
}

static float lint_top(void)
{
    float top = 1e9f; int i;
    for (i = 0; i < REC.np; i++) if (poly_miny(&REC.p[i]) < top) top = poly_miny(&REC.p[i]);
    return top;
}

/* A focused row: lifted 2 px, an ember front edge, an ember tick (two ember polys), the lift face. Returns the number
 * of the three cues missing. `rest` is the rectangle the row has at rest. */
static int lint_focus_row(AtRect rest)
{
    int bad = 0;
    if (lint_top() > rest.y - 1.5f) bad++;                       /* the lift */
    if (count_color(AT_C_EMBER) < 1) bad++;                      /* the edge */
    if (count_color(AT_C_EMBER) < 2) bad++;                      /* the tick (edge plus tick = two) */
    return bad;
}

/* A focused cell: lifted 2 px, ember front edge, and four registration brackets (at least four small polys of the
 * focus colour: a bracket is two thin strips). */
static int lint_focus_cell(AtRect rest, unsigned focus_rgba)
{
    int i, bad = 0, small = 0;
    if (lint_top() > rest.y - 1.5f) bad++;
    if (count_color(AT_C_EMBER) < 1) bad++;
    for (i = 0; i < REC.np; i++)
        if (REC.p[i].rgba == focus_rgba && poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]) <= 14.0f &&
            poly_maxy(&REC.p[i]) - poly_miny(&REC.p[i]) <= 14.0f) small++;
    if (small < 4) bad++;
    return bad;
}
#endif
