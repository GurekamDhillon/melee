/* Checks of the Atlas style rules (spec 4.1 depth and chamfer, 4.6 focus, 4.3 text floor and fit, 4.7 ports) over a recorded draw.
 * Include after atlas_rec.h. A check returns 0 or -1 as documented per function; none of them is a substitute for the owner's look. */
#ifndef ATLAS_STYLE_H
#define ATLAS_STYLE_H
#include "atlas_rec.h"
#include "atlas_fake.h"
#include "../platform/gw_ui_tokens.h"

static int sty_poly_has(const RecPoly *p, float x, float y)
{
    int i, pos = 0, neg = 0;
    for (i = 0; i < 4; i++) {
        int j = (i + 1) % 4;
        float cr = (p->x[j] - p->x[i]) * (y - p->y[i]) - (p->y[j] - p->y[i]) * (x - p->x[i]);
        if (cr > 0.001f) pos = 1; else if (cr < -0.001f) neg = 1;
    }
    return !(pos && neg);
}

static unsigned sty_top_at(float x, float y)
{
    int i;
    for (i = REC.np - 1; i >= 0; i--)
        if ((REC.p[i].rgba & 0xFFu) >= 0x80u && sty_poly_has(&REC.p[i], x, y)) return REC.p[i].rgba;
    return 0u;
}

/* 4.1: a chamfered element is cut on the top-left and bottom-right corners only. The point a quarter of the chamfer in from each
 * cut corner must still show what is UNDER the element (`under`), and the point the same distance in from the other two corners
 * must show the element. 1 = top-left was drawn over, 2 = bottom-right, 3 = top-right is cut, 4 = bottom-left is cut. */
static int sty_chamfer(AtRect r, float c, unsigned under)
{
    float q = c * 0.25f;
    if (sty_top_at(r.x + q, r.y + q) != under) return 1;
    if (sty_top_at(r.x + r.w - q, r.y + r.h - q) != under) return 2;
    if (sty_top_at(r.x + r.w - q, r.y + q) == under) return 3;
    if (sty_top_at(r.x + q, r.y + r.h - q) == under) return 4;
    return 0;
}

typedef struct { float miny; int ember_polys; int polys; } StySig;
static StySig sty_sig(int from_poly)
{
    StySig s;
    int i, k;
    s.miny = 1.0e9f; s.ember_polys = 0; s.polys = REC.np - from_poly;
    for (i = from_poly; i < REC.np; i++) {
        if (REC.p[i].rgba == AT_C_EMBER) s.ember_polys++;
        for (k = 0; k < 4; k++) if (REC.p[i].y[k] < s.miny) s.miny = REC.p[i].y[k];
    }
    return s;
}

/* 4.6: three cues at once. The same part drawn at rest and focused: it is lifted by 2 px, the ember edge appears, and a tick or
 * four brackets add at least eight polys. The brackets may be tinted with a port's colour (4.6: on a shared screen), so they are
 * counted by number, not by colour; the tick is the ember one. Returns how many of the three hold. */
static int sty_focus_cues(StySig rest, StySig focused)
{
    int n = 0;
    if (focused.miny <= rest.miny - 1.9f) n++;
    if (focused.ember_polys > rest.ember_polys) n++;
    if ((focused.polys - rest.polys >= 1 && focused.ember_polys - rest.ember_polys >= 2) || focused.polys - rest.polys >= 8) n++;   /* tick: one thin ember rect; brackets: eight, any colour */
    return n;
}

/* 4.3: every recorded text from index from_text on lies inside `pane` horizontally and has a legible role. -1 when all do,
 * else the index of the first offender (fake_width is half the role's size per character: a conservative stand-in). */
static int sty_text_inside(AtRect pane, int from_text)
{
    int i;
    for (i = from_text; i < REC.nt; i++) {
        float w = fake_width(NULL, REC.t[i].role, REC.t[i].s), left = REC.t[i].x;
        if (REC.t[i].align == AT_ALIGN_RIGHT) left -= w; else if (REC.t[i].align == AT_ALIGN_CENTER) left -= w * 0.5f;
        if (at_role_size(REC.t[i].role) < 12) return i;
        if (left < pane.x - 0.01f || left + w > pane.x + pane.w + 0.01f) return i;
    }
    return -1;
}

/* 4.7: colour is never the only signal. `polys_per_port` is how many polys each port's mark used: no two may be equal. 1 = distinct. */
static int sty_shapes_distinct(const int polys_per_port[4])
{
    int a, b;
    for (a = 0; a < 4; a++) for (b = a + 1; b < 4; b++) if (polys_per_port[a] == polys_per_port[b]) return 0;
    return 1;
}
#endif
