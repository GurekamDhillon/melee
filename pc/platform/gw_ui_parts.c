/* gw_ui_parts.c - the Atlas parts, drawn only through AtSink. Geometry follows the spec's style section
 * (tokens from gw_ui_tokens.h). Chamfers are two opposite corners; no curves; the circle is an octagon. */
#include "gw_ui_parts.h"
#include "gw_ui_tokens.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

/* tag tones that tokens.css does not carry (ATLAS/kit.css .tag.sun and .tag.rose backgrounds) */
#define AT_C_SUN_D 0x5E4A14FFu
#define AT_C_ROSE_D 0x5A1C30FFu

static void poly4(const AtSink *s, float x0, float y0, float x1, float y1, float x2, float y2, float x3, float y3, unsigned c)
{
    float px[4], py[4];
    px[0] = x0; px[1] = x1; px[2] = x2; px[3] = x3;
    py[0] = y0; py[1] = y1; py[2] = y2; py[3] = y3;
    s->poly(s->user, px, py, c);
}

void at_poly_rect(const AtSink *s, float x, float y, float w, float h, unsigned c)
{
    if ((c & 0xFFu) == 0 || w <= 0.0f || h <= 0.0f) return;
    poly4(s, x, y, x + w, y, x + w, y + h, x, y + h, c);
}

static void tri(const AtSink *s, float x0, float y0, float x1, float y1, float x2, float y2, unsigned c)
{
    poly4(s, x0, y0, x1, y1, x2, y2, x2, y2, c);
}

void at_disc(const AtSink *s, float cx, float cy, float r, unsigned c)
{
    float px[8], py[8];
    int k;
    for (k = 0; k < 8; k++) {
        double a = (22.5 + 45.0 * k) * 3.14159265358979 / 180.0;
        px[k] = cx + r * (float) cos(a);
        py[k] = cy + r * (float) sin(a);
    }
    poly4(s, px[0], py[0], px[1], py[1], px[2], py[2], px[3], py[3], c);
    poly4(s, px[0], py[0], px[3], py[3], px[4], py[4], px[5], py[5], c);
    poly4(s, px[0], py[0], px[5], py[5], px[6], py[6], px[7], py[7], c);
}

int at_plate_polys(AtRect r, float e, float c, float px[3][4], float py[3][4])
{
    float x = r.x, y = r.y, w = r.w, h = r.h;
    if (c < e) c = e;
    px[0][0] = x + c;         py[0][0] = y;
    px[0][1] = x + w;         py[0][1] = y;
    px[0][2] = x + w;         py[0][2] = y + h - c;
    px[0][3] = x + w - c + e; py[0][3] = y + h - e;
    px[1][0] = x + c;         py[1][0] = y;
    px[1][1] = x + w - c + e; py[1][1] = y + h - e;
    px[1][2] = x;             py[1][2] = y + h - e;
    px[1][3] = x;             py[1][3] = y + c;
    px[2][0] = x;             py[2][0] = y + h - e;
    px[2][1] = x + w - c + e; py[2][1] = y + h - e;
    px[2][2] = x + w - c;     py[2][2] = y + h;
    px[2][3] = x;             py[2][3] = y + h;
    return 3;
}

void at_plate(const AtSink *s, AtRect r, unsigned face, unsigned edge_rgba, float e, float c)
{
    float px[3][4], py[3][4];
    int i;
    at_plate_polys(r, e, c, px, py);
    for (i = 0; i < 3; i++) s->poly(s->user, px[i], py[i], i < 2 ? face : edge_rgba);
}

void at_text(const AtSink *s, const AtTextOps *o, int role, const char *str, float x, float base, unsigned rgba, int align, float max_w)
{
    char buf[200];
    int r;
    if (str == NULL || str[0] == '\0') return;
    r = at_fit(o, role, str, max_w, buf, sizeof buf);
    s->text(s->user, x, base, buf, r, rgba, align, 0.0f);
}

/* The one rule for budgeted text: at_fit reads max_w <= 0 as "no limit", so a box with no room must not reach it.
 * Fit when there are 8 px or more (step down one role, then truncate); otherwise draw nothing. Returns the width
 * drawn. Fixed glyphs and strings known to fit go through at_text with 0. */
static float fit_text(const AtSink *s, const AtTextOps *o, int role, const char *str, float x, float base, unsigned rgba, int align, float max_w)
{
    char buf[200];
    int r;
    if (str == NULL || str[0] == '\0' || max_w < 8.0f) return 0.0f;
    r = at_fit(o, role, str, max_w, buf, sizeof buf);
    s->text(s->user, x, base, buf, r, rgba, align, 0.0f);
    return o->width(o->user, r, buf);
}

static float mid_base(float top, float h, int role) { return top + h * 0.5f + (float) at_role_size(role) * 0.35f; }
static float twidth(const AtTextOps *o, int role, const char *s) { return o->width(o->user, role, s); }

/* ---- small glyphs, all straight lines ------------------------------------------------------------- */
static void glyph_plus(const AtSink *s, float cx, float cy, float len, float th, unsigned c)
{
    at_poly_rect(s, cx - len * 0.5f, cy - th * 0.5f, len, th, c);
    at_poly_rect(s, cx - th * 0.5f, cy - len * 0.5f, th, len, c);
}
static void glyph_lock(const AtSink *s, float cx, float cy, unsigned c)
{
    at_poly_rect(s, cx - 6.0f, cy - 1.0f, 12.0f, 9.0f, c);               /* body */
    at_poly_rect(s, cx - 4.0f, cy - 7.0f, 2.0f, 6.0f, c);                /* shackle */
    at_poly_rect(s, cx + 2.0f, cy - 7.0f, 2.0f, 6.0f, c);
    at_poly_rect(s, cx - 4.0f, cy - 8.0f, 8.0f, 2.0f, c);
}
static void outline(const AtSink *s, AtRect r, float th, unsigned c)
{
    at_poly_rect(s, r.x, r.y, r.w, th, c);
    at_poly_rect(s, r.x, r.y + r.h - th, r.w, th, c);
    at_poly_rect(s, r.x, r.y + th, th, r.h - 2.0f * th, c);
    at_poly_rect(s, r.x + r.w - th, r.y + th, th, r.h - 2.0f * th, c);
}
/* an outline of a plate whose top-left and bottom-right corners are cut by c: the strips stop at the cut */
static void outline_ch(const AtSink *s, AtRect r, float th, float c, unsigned col)
{
    at_poly_rect(s, r.x + c, r.y, r.w - c, th, col);
    at_poly_rect(s, r.x, r.y + r.h - th, r.w - c, th, col);
    at_poly_rect(s, r.x, r.y + c, th, r.h - th - c, col);
    at_poly_rect(s, r.x + r.w - th, r.y + th, th, r.h - c - th, col);
}
static void brackets(const AtSink *s, AtRect r, unsigned c)             /* four registration brackets, 4 px outside the cell */
{
    float x0 = r.x - 4.0f, y0 = r.y - 4.0f, x1 = r.x + r.w + 4.0f, y1 = r.y + r.h + 4.0f;
    at_poly_rect(s, x0, y0, 9.0f, 2.0f, c);        at_poly_rect(s, x0, y0, 2.0f, 9.0f, c);
    at_poly_rect(s, x1 - 9.0f, y0, 9.0f, 2.0f, c); at_poly_rect(s, x1 - 2.0f, y0, 2.0f, 9.0f, c);
    at_poly_rect(s, x0, y1 - 2.0f, 9.0f, 2.0f, c); at_poly_rect(s, x0, y1 - 9.0f, 2.0f, 9.0f, c);
    at_poly_rect(s, x1 - 9.0f, y1 - 2.0f, 9.0f, 2.0f, c); at_poly_rect(s, x1 - 2.0f, y1 - 9.0f, 2.0f, 9.0f, c);
}

/* ---- value widgets: each draws ending at `right` and returns the width it used ---------------------- */
static float part_toggle(const AtSink *s, const AtTextOps *o, float right, float cy, int on)
{
    float w = 36.0f, h = 22.0f, x = right - 2.0f * w, y = cy - h * 0.5f;
    at_poly_rect(s, x, y, w, h, on ? AT_C_GROUND : AT_C_LINE2);
    at_poly_rect(s, x + w, y, w, h, on ? AT_C_JADE : AT_C_GROUND);
    at_text(s, o, AT_R_CAP12, "OFF", x + w * 0.5f, mid_base(y, h, AT_R_CAP12), on ? AT_C_DIM : AT_C_IVORY, AT_ALIGN_CENTER, 0.0f);
    at_text(s, o, AT_R_CAP12, "ON", x + w * 1.5f, mid_base(y, h, AT_R_CAP12), on ? AT_C_INK : AT_C_DIM, AT_ALIGN_CENTER, 0.0f);
    return 2.0f * w;
}

static float part_choice(const AtSink *s, const AtTextOps *o, float right, float cy, const char *text, int focus)
{
    float tw = twidth(o, AT_R_ROW16, text), mid = tw < 44.0f ? 44.0f : tw, total = 16.0f + 6.0f + mid + 6.0f + 16.0f, x = right - total;
    unsigned bg = focus ? AT_C_EMBER : AT_C_GROUND, fg = focus ? AT_C_INK : AT_C_MUTED;
    at_poly_rect(s, x, cy - 9.0f, 16.0f, 18.0f, bg);
    tri(s, x + 11.0f, cy - 4.0f, x + 5.0f, cy, x + 11.0f, cy + 4.0f, fg);
    fit_text(s, o, AT_R_ROW16, text, x + 22.0f + mid * 0.5f, mid_base(cy - 9.0f, 18.0f, AT_R_ROW16), AT_C_IVORY, AT_ALIGN_CENTER, mid);
    at_poly_rect(s, right - 16.0f, cy - 9.0f, 16.0f, 18.0f, bg);
    tri(s, right - 11.0f, cy - 4.0f, right - 5.0f, cy, right - 11.0f, cy + 4.0f, fg);
    return total;
}

static float part_slider(const AtSink *s, float right, float cy, int vmin, int vmax, int vval, int focus)
{
    int i, k = vmax > vmin ? (vval - vmin) * 10 / (vmax - vmin) : 0;
    float x = right - 70.0f;
    if (k < 0) k = 0;
    if (k > 9) k = 9;
    for (i = 0; i < 10; i++) {
        float tx = x + 7.0f * (float) i;
        if (i == k) at_poly_rect(s, tx, cy - 10.0f, 6.0f, 20.0f, focus ? AT_C_EMBER : AT_C_IVORY);
        else at_poly_rect(s, tx, cy - (i < k ? 6.0f : 5.0f), 5.0f, i < k ? 12.0f : 10.0f, i < k ? AT_C_TEXT2 : AT_C_GROUND);
    }
    return 70.0f;
}

void at_part_row(const AtSink *s, const AtTextOps *o, AtRect r, const AtItem *it, int state)
{
    unsigned face = AT_C_PLATE2, edge = AT_C_EDGE2, txt = AT_C_TEXT2, val = AT_C_MUTED;
    float e = 3.0f, y = r.y, right, cy, vw = 0.0f, lx, avail;
    int disabled = state == AT_ST_DISABLED || (it->flags & AT_CELL_DISABLED);
    int focus = state == AT_ST_FOCUS && !disabled, dfocus = state == AT_ST_FOCUS && disabled;   /* a focused disabled row: its own cue */
    AtRect pr;
    if (dfocus) { face = AT_C_PLATE; edge = AT_C_EMBER; txt = AT_C_DIM; val = AT_C_DIM; y -= 2.0f; }   /* disabled look kept (plate, dim), lifted with an ember edge */
    else if (disabled) { face = AT_C_PLATE; txt = AT_C_DIM; val = AT_C_DIM; }
    else if (focus) { face = AT_C_LIFT; edge = AT_C_EMBER; txt = AT_C_IVORY; val = AT_C_IVORY; y -= 2.0f; }
    else if (state == AT_ST_PRESS) { face = AT_C_PLATE; edge = AT_C_EMBER_D; txt = AT_C_IVORY; e = 1.0f; y += 1.0f; }
    pr.x = r.x; pr.y = y; pr.w = r.w; pr.h = r.h;
    at_plate(s, pr, face, edge, e, (float) AT_PX_CH_S);
    /* the tick starts below the chamfer so it never squares off the cut corner */
    if (focus || dfocus) at_poly_rect(s, r.x, y + (float) AT_PX_CH_S, 4.0f, r.h - e - (float) AT_PX_CH_S, AT_C_EMBER);
    if (state == AT_ST_PRESS) at_poly_rect(s, r.x, y + (float) AT_PX_CH_S, 4.0f, r.h - e - (float) AT_PX_CH_S, AT_C_EMBER_D);
    if (it->flags & AT_CELL_SELECTED) at_poly_rect(s, r.x + r.w - 4.0f, y, 4.0f, r.h - e, AT_C_JADE);
    right = r.x + r.w - 12.0f - ((it->flags & AT_CELL_SELECTED) ? 6.0f : 0.0f);
    cy = y + (r.h - e) * 0.5f;
    switch (it->vkind) {
    case AT_VAL_TOGGLE: vw = part_toggle(s, o, right, cy, it->on); break;
    case AT_VAL_CHOICE: vw = part_choice(s, o, right, cy, it->text, focus); break;
    case AT_VAL_SLIDER: vw = part_slider(s, right, cy, it->vmin, it->vmax, it->vval, focus); break;
    case AT_VAL_TEXT:
    case AT_VAL_COUNTER:
        at_text(s, o, AT_R_NUM14, it->text, right, mid_base(y, r.h - e, AT_R_NUM14), val, AT_ALIGN_RIGHT, 0.0f);
        vw = twidth(o, AT_R_NUM14, it->text);
        break;
    default: break;
    }
    lx = r.x + 12.0f;
    avail = right - lx - vw - 8.0f;
    {                                                                   /* no room: fit_text draws nothing rather than unfitted text */
        if (it->sub[0] != '\0') {
            fit_text(s, o, AT_R_ROW16, it->label, lx, y + 15.0f, txt, AT_ALIGN_LEFT, avail);
            fit_text(s, o, AT_R_BODY12, it->sub, lx, y + r.h - e - 4.0f, AT_C_MUTED, AT_ALIGN_LEFT, avail);
        } else {
            fit_text(s, o, AT_R_ROW16, it->label, lx, mid_base(y, r.h - e, AT_R_ROW16), txt, AT_ALIGN_LEFT, avail);
        }
    }
}

/* A hub tile or a main-menu row. Focus is three cues at once: the plate lifts 2 px, its front edge turns ember, and a 4 px tick
 * stands at the left below the chamfer. Selected adds a jade bar and never moves. Nothing is drawn inside a cut-away corner. */
void at_part_tile(const AtSink *s, const AtTextOps *o, AtRect r, const AtItem *it, int state, int big)
{
    unsigned face = AT_C_PLATE2, edge = AT_C_EDGE2, txt = AT_C_TEXT2;
    float e = 3.0f, y = r.y, lx, right, avail, tagw = 0.0f;
    int disabled = state == AT_ST_DISABLED || (it->flags & AT_CELL_DISABLED), focus = state == AT_ST_FOCUS && !disabled;
    int role = big ? AT_R_TITLE : AT_R_CAP20, tone = AT_TAG_PLAIN;
    const char *tag = it->tag[0] != '\0' ? it->tag : it->badge;
    AtRect pr;
    if (disabled) { face = AT_C_PLATE; txt = AT_C_DIM; }
    else if (focus) { face = AT_C_LIFT; edge = AT_C_EMBER; txt = AT_C_IVORY; y -= 2.0f; }
    else if (state == AT_ST_PRESS) { face = AT_C_PLATE; edge = AT_C_EMBER_D; txt = AT_C_IVORY; e = 1.0f; y += 1.0f; }
    pr.x = r.x; pr.y = y; pr.w = r.w; pr.h = r.h;
    at_plate(s, pr, face, edge, e, (float) AT_PX_CH_S);
    if (focus) at_poly_rect(s, r.x, y + (float) AT_PX_CH_S, 4.0f, r.h - e - (float) AT_PX_CH_S, AT_C_EMBER);
    if (state == AT_ST_PRESS) at_poly_rect(s, r.x, y + (float) AT_PX_CH_S, 4.0f, r.h - e - (float) AT_PX_CH_S, AT_C_EMBER_D);
    if ((it->flags & AT_CELL_SELECTED) || state == AT_ST_SELECTED) at_poly_rect(s, r.x + (float) AT_PX_CH_S, y + r.h - 3.0f, r.w - 2.0f * (float) AT_PX_CH_S, 3.0f, AT_C_JADE);
    lx = r.x + 14.0f;
    right = r.x + r.w - 15.0f;                                       /* 10 px of padding plus the chamfer: clear of the bottom-right cut */
    if (big && it->numeral[0] != '\0') {                              /* the numeral badge: 30 x 30, a 5 px chamfer, inside the plate */
        AtRect b;
        b.x = lx; b.y = y + (r.h - e - 30.0f) * 0.5f; b.w = 30.0f; b.h = 30.0f;
        at_plate(s, b, focus ? AT_C_EMBER : AT_C_GROUND, focus ? AT_C_EMBER : AT_C_GROUND, 0.0f, (float) AT_PX_CH_S);
        fit_text(s, o, AT_R_NUM14, it->numeral, b.x + b.w * 0.5f, mid_base(b.y, b.h, AT_R_NUM14), focus ? AT_C_INK : AT_C_MUTED, AT_ALIGN_CENTER, b.w - 4.0f);
        lx += 30.0f + 12.0f;
    }
    if (tag[0] != '\0') {
        tone = strcmp(tag, "MOD") == 0 ? AT_TAG_JADE : (it->tag[0] == '\0' ? AT_TAG_EMBER : AT_TAG_PLAIN);
        tagw = twidth(o, AT_R_CAP12, tag) + 16.0f;
        if (tagw > r.w * 0.4f) tagw = r.w * 0.4f;
        at_part_tag(s, o, right - tagw, y + (r.h - e - 20.0f) * 0.5f, tag, tone, tagw);
        avail = right - tagw - 8.0f - lx;
    } else {
        avail = right - lx;
    }
    fit_text(s, o, role, it->label, lx, mid_base(y, r.h - e, role), txt, AT_ALIGN_LEFT, avail);
}

/* The More strip: a 1 px outline that skips both chamfers, the word MORE, then n quiet labels. The focused label lifts 2 px,
 * its front edge turns ember and a 4 px tick stands at its left. */
void at_part_more(const AtSink *s, const AtTextOps *o, AtRect r, const AtItem *items, int n, int focus)
{
    float slot, x0 = r.x + 70.0f;
    int i;
    outline_ch(s, r, 1.0f, (float) AT_PX_CH_S, AT_C_LINE);
    fit_text(s, o, AT_R_CAP12, "MORE", r.x + 14.0f, mid_base(r.y, r.h, AT_R_CAP12), AT_C_MUTED, AT_ALIGN_LEFT, 48.0f);
    if (n < 1) return;
    slot = (r.w - 70.0f) / (float) n;
    for (i = 0; i < n; i++) {
        float sx = x0 + (float) i * slot, ly = r.y + 4.0f, lh = r.h - 8.0f, aw = slot - 8.0f;
        int on = i == focus && !(items[i].flags & AT_CELL_DISABLED);
        unsigned ink = (items[i].flags & AT_CELL_DISABLED) ? AT_C_DIM : (on ? AT_C_IVORY : AT_C_TEXT2);
        if (on) {
            ly -= 2.0f;
            at_poly_rect(s, sx, ly, aw, lh - 2.0f, AT_C_LIFT);
            at_poly_rect(s, sx, ly + lh - 4.0f, aw, 2.0f, AT_C_EMBER);        /* the front edge */
            at_poly_rect(s, sx, ly, 4.0f, lh - 4.0f, AT_C_EMBER);             /* the tick */
        }
        fit_text(s, o, AT_R_CAP14, items[i].label, sx + 12.0f, mid_base(ly, lh - 2.0f, AT_R_CAP14), ink, AT_ALIGN_LEFT, aw - 16.0f);
    }
}

/* The title's face. The ground is the renderer's. The prompt is a focused-style plate (chamfer 5, lifted, ember edge) that pulses its
 * edge between 60 and 100 percent alpha over 1,200 ms on the UI clock; reduced motion holds it at full. */
void at_part_title(const AtSink *s, const AtTextOps *o, const AtLayout *L, const AtScreen *sc, double now_ms, int reduced, AtRect *prompt_out)
{
    float cx = L->canvas.w * 0.5f, max_w = L->content_w - 64.0f, tw, gw, lw, pw, pulse = 1.0f;
    AtRect pr;
    unsigned edge;
    if (sc->hero[0] != '\0') at_text(s, o, AT_R_DISPLAY, sc->hero, cx, 208.0f, AT_C_IVORY, AT_ALIGN_CENTER, max_w);
    tw = twidth(o, AT_R_CAP16, "PC PORT");
    at_text(s, o, AT_R_CAP16, "PC PORT", cx, 240.0f, AT_C_MUTED, AT_ALIGN_CENTER, 0.0f);
    at_poly_rect(s, cx - tw * 0.5f - 12.0f - 70.0f, 235.0f, 70.0f, 2.0f, AT_C_EMBER);
    at_poly_rect(s, cx + tw * 0.5f + 12.0f, 235.0f, 70.0f, 2.0f, AT_C_EMBER);
    if (sc->prompt[0] != '\0') {
        float lab = max_w - 32.0f - 60.0f;
        char fit[200];
        int role = at_fit(o, AT_R_CAP16, sc->prompt, lab, fit, sizeof fit);
        gw = twidth(o, AT_R_CAP12, "START") + 14.0f;
        lw = twidth(o, role, fit);
        pw = 16.0f + gw + 10.0f + lw + 16.0f;
        pr.x = cx - pw * 0.5f; pr.y = 330.0f - 20.0f - 2.0f; pr.w = pw; pr.h = 40.0f;      /* the plate is lifted 2 px, as a focused row */
        if (!reduced) pulse = 0.8f + 0.2f * (float) cos(now_ms * 6.28318530718 / 1200.0);
        edge = (AT_C_EMBER & 0xFFFFFF00u) | (unsigned) (pulse * 255.0f + 0.5f);
        at_plate(s, pr, AT_C_LIFT, edge, 3.0f, (float) AT_PX_CH_S);
        at_part_hint(s, o, pr.x + 16.0f, pr.y + 20.0f + 5.0f, 'S', "");
        s->text(s->user, pr.x + 16.0f + gw + 10.0f, pr.y + (pr.h - 3.0f) * 0.5f + (float) at_role_size(role) * 0.35f, fit, role, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
        if (prompt_out != NULL) *prompt_out = pr;
    } else if (prompt_out != NULL) {
        memset(prompt_out, 0, sizeof *prompt_out);
    }
    if (sc->foot_left[0] != '\0') fit_text(s, o, AT_R_NUM12, sc->foot_left, L->content_x + 32.0f, 456.0f, AT_C_MUTED, AT_ALIGN_LEFT, (L->content_w - 64.0f) * 0.3f);
    if (sc->foot_right[0] != '\0') fit_text(s, o, AT_R_BODY12, sc->foot_right, L->content_x + L->content_w - 32.0f, 456.0f, AT_C_DIM, AT_ALIGN_RIGHT, (L->content_w - 64.0f) * 0.6f);
}

/* Tabs fit inside r.w: the names step down one role (CAP16 to CAP14), then are truncated in proportion; when even the
 * padding and counts do not fit, the counts are dropped, and a name with no room left is not drawn. */
void at_part_tabs(const AtSink *s, const AtTextOps *o, AtRect r, const char *const *names, const int *counts, int n, int active, int focus_tab)
{
    float x = r.x, bottom = r.y + r.h, gaps = n > 1 ? 2.0f * (float) (n - 1) : 0.0f, fixed, names_w, scale = 1.0f;
    int i, role = AT_R_CAP16, with_counts = counts != NULL, pass;
    for (pass = 0; pass < 3; pass++) {
        fixed = gaps; names_w = 0.0f;
        for (i = 0; i < n; i++) {
            char num[16];
            snprintf(num, sizeof num, "%d", counts != NULL ? counts[i] : 0);
            fixed += 28.0f + (with_counts ? twidth(o, AT_R_NUM12, num) + 6.0f : 0.0f);
            names_w += twidth(o, role, names[i]);
        }
        if (fixed + names_w <= r.w) break;
        if (pass == 0) role = AT_R_CAP14;
        else if (pass == 1 && with_counts && r.w - fixed < 8.0f * (float) n) with_counts = 0;
        else break;
    }
    if (fixed + names_w > r.w) scale = names_w > 0.0f && r.w > fixed ? (r.w - fixed) / names_w : 0.0f;
    for (i = 0; i < n; i++) {
        char num[16], fit[200];
        int fr = role;
        float tw = twidth(o, role, names[i]), cw = 0.0f, w, h = i == active ? 30.0f : 26.0f, y = bottom - h, nw, e = 3.0f;
        int draw_name;
        unsigned face = i == active ? AT_C_PLATE : AT_C_GROUND2;
        if (i == focus_tab) y -= 2.0f;                                    /* the focused tab lifts like a focused row */
        snprintf(num, sizeof num, "%d", with_counts ? counts[i] : 0);
        if (with_counts) cw = twidth(o, AT_R_NUM12, num) + 6.0f;
        draw_name = tw * scale >= 8.0f;
        if (draw_name) { fr = at_fit(o, role, names[i], tw * scale, fit, sizeof fit); nw = twidth(o, fr, fit); } else nw = 0.0f;
        w = nw + cw + 28.0f;
        if (x + w > r.x + r.w) w = r.x + r.w - x;
        if (w <= 0.0f) break;
        at_poly_rect(s, x, y, w, h, face);
        if (draw_name) at_text(s, o, fr, fit, x + 14.0f, mid_base(y, h, fr), i == active ? AT_C_IVORY : AT_C_MUTED, AT_ALIGN_LEFT, 0.0f);
        if (with_counts) at_text(s, o, AT_R_NUM12, num, x + 14.0f + nw + 6.0f, mid_base(y, h, AT_R_NUM12), i == active ? AT_C_EMBER : AT_C_DIM, AT_ALIGN_LEFT, 0.0f);
        if (i == focus_tab) {                                             /* three cues: the lift, an ember front edge, an ember tick */
            at_poly_rect(s, x, y + h - e, w, e, AT_C_EMBER);
            at_poly_rect(s, x, y, 4.0f, h - e, AT_C_EMBER);
        }
        x += w + 2.0f;
    }
}

float at_part_tag(const AtSink *s, const AtTextOps *o, float x, float y, const char *text, int tone, float max_w)
{
    unsigned face = AT_C_GROUND, ink = AT_C_TEXT2;
    AtRect r;
    float tw = twidth(o, AT_R_CAP12, text), w = tw + 16.0f;
    if (tone == AT_TAG_JADE) { face = AT_C_JADE_D; ink = AT_C_IVORY; }
    else if (tone == AT_TAG_EMBER) { face = AT_C_EMBER; ink = AT_C_INK; }
    else if (tone == AT_TAG_SUN) { face = AT_C_SUN_D; ink = AT_C_SUN; }
    else if (tone == AT_TAG_ROSE) { face = AT_C_ROSE_D; ink = 0xFFB3C9FFu; }
    if (max_w > 0.0f && w > max_w) w = max_w;
    r.x = x; r.y = y; r.w = w; r.h = 20.0f;
    at_plate(s, r, face, face, 0.0f, (float) AT_PX_CH_S);
    fit_text(s, o, AT_R_CAP12, text, x + 8.0f, mid_base(y, 20.0f, AT_R_CAP12), ink, AT_ALIGN_LEFT, w - 16.0f);
    return w;
}

void at_part_cell(const AtSink *s, const AtTextOps *o, AtRect r, const AtCell *c, int state, unsigned focus_rgba)
{
    int disabled = state == AT_ST_DISABLED || (c->flags & AT_CELL_DISABLED), focus = state == AT_ST_FOCUS;   /* a focused disabled or locked cell keeps its dim face and gains the cues */
    int press = state == AT_ST_PRESS && !disabled, has_model = c->model != AT_NO_MODEL;
    float y = focus ? r.y - 2.0f : press ? r.y + 1.0f : r.y, cc = (float) AT_PX_CH_XS;
    AtRect pr;
    pr.x = r.x; pr.y = y; pr.w = r.w; pr.h = r.h;
    if (c->flags & AT_CELL_EMPTY) {
        outline(s, pr, 1.0f, focus ? AT_C_EMBER : AT_C_LINE);
        if (focus) at_poly_rect(s, r.x + 1.0f, y + r.h - 3.0f, r.w - 2.0f, 2.0f, AT_C_EMBER);      /* the front edge, thickened */
        glyph_plus(s, r.x + r.w * 0.5f, y + r.h * 0.5f, 14.0f, 2.0f, AT_C_LINE2);
    } else {
        unsigned face = (c->flags & AT_CELL_LOCKED) || disabled || press ? AT_C_PLATE : (focus ? AT_C_LIFT : (has_model ? AT_C_GROUND2 : AT_C_PLATE2));
        at_plate(s, pr, face, focus ? AT_C_EMBER : press ? AT_C_EMBER_D : AT_C_EDGE2, press ? 1.0f : 3.0f, cc);
        if (c->flags & AT_CELL_LOCKED) {
            glyph_lock(s, r.x + r.w * 0.5f, y + r.h * 0.5f, AT_C_DIM);
        } else if (has_model) {
            at_poly_rect(s, r.x + r.w * 0.18f, y + r.h * 0.77f, r.w * 0.64f, r.h * 0.12f, 0x00000073u);   /* the floor shadow */
            s->model(s->user, c->model, c->ring, r.x + 3.0f, y + 3.0f, r.w - 6.0f, r.h - 8.0f, focus && !disabled, disabled ? 1 : 0);
        } else {
            /* pips take their own strip along the bottom edge; the name sits above it, never under them */
            fit_text(s, o, AT_R_BODY12, c->name, r.x + r.w * 0.5f, y + r.h - (c->pips > 0 ? 16.0f : 8.0f), disabled ? AT_C_DIM : AT_C_IVORY, AT_ALIGN_CENTER, r.w - 6.0f);
        }
        if (c->index > 0) {
            char num[12];
            snprintf(num, sizeof num, "%d", c->index);
            at_text(s, o, AT_R_NUM12, num, r.x + 4.0f, y + 13.0f, AT_C_DIM, AT_ALIGN_LEFT, 0.0f);
        }
        if (c->origin != 0) {
            char one[2];
            one[0] = c->origin; one[1] = '\0';
            at_poly_rect(s, r.x + r.w - 14.0f, y, 14.0f, 14.0f, c->origin == 'G' ? AT_C_JADE : AT_C_SUN);
            at_text(s, o, AT_R_CAP12, one, r.x + r.w - 7.0f, y + 11.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
        } else if (c->flags & AT_CELL_NEW) {
            at_poly_rect(s, r.x + r.w - 9.0f, y + 3.0f, 6.0f, 6.0f, AT_C_EMBER_D);
        }
        if (c->pips > 0) {
            int p;
            float px = r.x + (r.w - (7.0f * (float) c->pips - 2.0f)) * 0.5f;                /* centred along the bottom edge */
            for (p = 0; p < c->pips; p++) at_poly_rect(s, px + 7.0f * (float) p, y + r.h - 9.0f, 5.0f, 5.0f, AT_C_JADE);
        }
        if (c->flags & (AT_CELL_MERGE | AT_CELL_SELECTED)) outline_ch(s, pr, 2.0f, cc, AT_C_JADE);
        if (c->flags & AT_CELL_MERGE) {
            at_poly_rect(s, r.x, y + r.h - 18.0f, r.w, 15.0f, AT_C_JADE);
            fit_text(s, o, AT_R_CAP12, "+ MERGE", r.x + r.w * 0.5f, y + r.h - 7.0f, AT_C_INK, AT_ALIGN_CENTER, r.w - 4.0f);
        }
    }
    if (focus) brackets(s, pr, focus_rgba != 0 ? focus_rgba : AT_C_EMBER);
}

void at_part_stone(const AtSink *s, const AtTextOps *o, AtRect r, const AtCell *c, int state, unsigned focus_rgba)
{
    int focus = state == AT_ST_FOCUS;
    float y = focus ? r.y - 2.0f : r.y;
    AtRect pr;
    unsigned face = c->rgba != 0 ? c->rgba : AT_C_LINE2;
    char one[2];
    pr.x = r.x; pr.y = y; pr.w = r.w; pr.h = r.h;
    if (c->flags & AT_CELL_EMPTY) face = AT_C_GROUND;
    else if (c->flags & AT_CELL_LOCKED) face = AT_C_PLATE;
    poly4(s, r.x + r.w * 0.2f, y, r.x + r.w * 0.8f, y, r.x + r.w, y + r.h, r.x, y + r.h, face);     /* the arch stone */
    if (c->flags & AT_CELL_EMPTY) glyph_plus(s, r.x + r.w * 0.5f, y + r.h * 0.6f, 12.0f, 2.0f, AT_C_LINE2);
    else if (c->flags & AT_CELL_LOCKED) glyph_lock(s, r.x + r.w * 0.5f, y + r.h * 0.6f, AT_C_DIM);
    else if (c->letter != 0) {
        one[0] = c->letter; one[1] = '\0';
        at_text(s, o, AT_R_CAP20, one, r.x + r.w * 0.5f, y + r.h - 9.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
    }
    if (focus) {                                                         /* three cues: the lift, an ember front edge, brackets */
        at_poly_rect(s, r.x, y + r.h - 3.0f, r.w, 3.0f, AT_C_EMBER);
        brackets(s, pr, focus_rgba != 0 ? focus_rgba : AT_C_EMBER);
    }
}

/* a key hint has no box of its own: its label is one short phrase, capped so a row of hints cannot outgrow the keys strip */
#define AT_HINT_MAX_W 280.0f

float at_part_hint(const AtSink *s, const AtTextOps *o, float x, float base, char btn, const char *label)
{
    float cy = base - 5.0f, gw = 18.0f, lw;
    char one[2];
    one[0] = btn; one[1] = '\0';
    if (btn == 'A' || btn == 'B' || btn == 'X' || btn == 'Y') {
        at_disc(s, x + 9.0f, cy, 9.0f, btn == 'A' ? AT_C_PAD_A : btn == 'B' ? AT_C_PAD_B : AT_C_PAD_X);
        at_text(s, o, AT_R_CAP12, one, x + 9.0f, mid_base(cy - 9.0f, 18.0f, AT_R_CAP12), btn == 'B' ? AT_C_IVORY : AT_C_INK, AT_ALIGN_CENTER, 0.0f);
    } else if (btn == 'Z') {
        gw = 20.0f;
        at_poly_rect(s, x, cy - 9.0f, gw, 18.0f, AT_C_PAD_Z);
        at_text(s, o, AT_R_CAP12, one, x + gw * 0.5f, mid_base(cy - 9.0f, 18.0f, AT_R_CAP12), AT_C_IVORY, AT_ALIGN_CENTER, 0.0f);
    } else if (btn == 'L' || btn == 'R') {
        gw = 22.0f;
        at_poly_rect(s, x, cy - 8.0f, gw, 16.0f, AT_C_MUTED);
        at_text(s, o, AT_R_CAP12, one, x + gw * 0.5f, mid_base(cy - 8.0f, 16.0f, AT_R_CAP12), AT_C_INK, AT_ALIGN_CENTER, 0.0f);
    } else if (btn == 'S') {
        gw = twidth(o, AT_R_CAP12, "START") + 14.0f;
        at_poly_rect(s, x, cy - 8.0f, gw, 16.0f, AT_C_MUTED);
        at_text(s, o, AT_R_CAP12, "START", x + gw * 0.5f, mid_base(cy - 8.0f, 16.0f, AT_R_CAP12), AT_C_INK, AT_ALIGN_CENTER, 0.0f);
    } else {                                                             /* 'M': the d-pad cross, for "Move" */
        at_poly_rect(s, x, cy - 3.0f, 18.0f, 6.0f, AT_C_TEXT2);
        at_poly_rect(s, x + 6.0f, cy - 9.0f, 6.0f, 18.0f, AT_C_TEXT2);
    }
    lw = fit_text(s, o, AT_R_BODY14, label, x + gw + 6.0f, base, AT_C_TEXT2, AT_ALIGN_LEFT, AT_HINT_MAX_W);
    return gw + 6.0f + lw + 18.0f;
}

float at_part_trail(const AtSink *s, const AtTextOps *o, AtRect r, const char *const *items, int n)
{
    float x = r.x, cy = r.y + r.h * 0.5f, sep = 18.0f, total, ell = twidth(o, AT_R_CAP16, "\xE2\x80\xA6"), base = cy + 7.0f, right = r.x + r.w;
    int i, first = 0;
    poly4(s, x + 11.0f, cy - 11.0f, x + 22.0f, cy, x + 11.0f, cy + 11.0f, x, cy, AT_C_EMBER);        /* the mark: a diamond ring */
    poly4(s, x + 11.0f, cy - 6.0f, x + 17.0f, cy, x + 11.0f, cy + 6.0f, x + 5.0f, cy, AT_C_GROUND);
    x += 34.0f;
    for (;;) {                                                           /* too wide: the earliest parents drop out, the ellipsis is in the budget */
        total = first > 0 ? ell + sep : 0.0f;
        for (i = first; i < n; i++) total += twidth(o, i == n - 1 ? AT_R_CAP20 : AT_R_CAP16, items[i]) + (i > first ? sep : 0.0f);
        if (total <= right - x || first >= n - 1) break;
        first++;
    }
    if (first > 0) {
        float w = fit_text(s, o, AT_R_CAP16, "\xE2\x80\xA6", x, base, AT_C_DIM, AT_ALIGN_LEFT, right - x);
        x += w > 0.0f ? w + sep : 0.0f;
    }
    for (i = first; i < n; i++) {
        int here = i == n - 1, role = here ? AT_R_CAP20 : AT_R_CAP16;
        float w = fit_text(s, o, role, items[i], x, base, here ? AT_C_IVORY : AT_C_MUTED, AT_ALIGN_LEFT, right - x);
        x += w;
        if (!here) {
            if (x + sep > right) break;
            at_text(s, o, AT_R_CAP14, ">", x + sep * 0.5f, base, AT_C_DIM, AT_ALIGN_CENTER, 0.0f);
            x += sep;
        }
    }
    return x < right ? x : right;
}

static const char *const ROMAN[5] = { "I", "II", "III", "IV", "V" };
static const char *const CHAPTER_NAME[5] = { "SOLO", "VERSUS", "ONLINE", "MODS", "SETTINGS" };

void at_part_chapter(const AtSink *s, const AtTextOps *o, AtRect r, int active)
{
    int i;
    for (i = 0; i < 5; i++) {
        float x = r.x + 23.0f * (float) i;
        int on = active == i + 1;
        at_poly_rect(s, x, r.y, 20.0f, 20.0f, on ? AT_C_EMBER : AT_C_GROUND2);
        at_text(s, o, AT_R_CAP12, ROMAN[i], x + 10.0f, mid_base(r.y, 20.0f, AT_R_CAP12), on ? AT_C_INK : AT_C_DIM, AT_ALIGN_CENTER, 0.0f);
    }
}

void at_part_rail(const AtSink *s, const AtTextOps *o, AtRect r, int active)
{
    int i;
    for (i = 0; i < 5; i++) {
        AtRect row;
        int on = active == i + 1;
        row.x = r.x; row.y = r.y + 34.0f * (float) i; row.w = r.w; row.h = 30.0f;
        if (on) at_plate(s, row, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
        at_poly_rect(s, row.x + 8.0f, row.y + 5.0f, 20.0f, 20.0f, on ? AT_C_EMBER : AT_C_GROUND2);
        at_text(s, o, AT_R_CAP12, ROMAN[i], row.x + 18.0f, mid_base(row.y + 5.0f, 20.0f, AT_R_CAP12), on ? AT_C_INK : AT_C_DIM, AT_ALIGN_CENTER, 0.0f);
        fit_text(s, o, AT_R_CAP14, CHAPTER_NAME[i], row.x + 36.0f, mid_base(row.y, 28.0f, AT_R_CAP14), on ? AT_C_IVORY : AT_C_DIM, AT_ALIGN_LEFT, r.w - 40.0f);
    }
}

void at_part_explainer(const AtSink *s, const AtTextOps *o, AtRect r, const AtExplainer *e)
{
    float x = r.x + 12.0f, w = r.w - 24.0f, y = r.y + 12.0f, bottom = r.y + r.h - 12.0f;
    char lines[5][96];
    int n, i, clamped = 0;
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
    if (!e->has) return;
    at_poly_rect(s, x, y, w, 96.0f, AT_C_GROUND2);                       /* the media well */
    if (e->media_model != AT_NO_MODEL) s->model(s->user, e->media_model, e->media_ring, x + 8.0f, y + 4.0f, w - 16.0f, 88.0f, 1, 0);
    y += 106.0f;
    fit_text(s, o, AT_R_CAP14, e->kicker, x, y + 11.0f, AT_C_JADE, AT_ALIGN_LEFT, w);
    y += 18.0f;
    fit_text(s, o, AT_R_TITLE, e->title, x, y + 24.0f, AT_C_IVORY, AT_ALIGN_LEFT, w);
    y += 36.0f;
    n = at_wrap(o, AT_R_BODY14, e->what, w, 4, lines, &clamped);
    for (i = 0; i < n && y + 18.0f <= bottom; i++) {
        at_text(s, o, AT_R_BODY14, lines[i], x, y + 13.0f, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
        y += 18.0f;
    }
    y += 8.0f;
    if (e->n_with > 0 && y + 44.0f <= bottom) {                          /* a label, then its content under it, so a narrow pane still fits */
        at_text(s, o, AT_R_CAP12, "WITH", x, y + 11.0f, AT_C_DIM, AT_ALIGN_LEFT, 0.0f);
        for (i = 0; i < e->n_with; i++) {
            float cx = x + 26.0f * (float) i;
            at_poly_rect(s, cx, y + 16.0f, 22.0f, 22.0f, AT_C_GROUND2);
            if (e->with_model[i] != AT_NO_MODEL) s->model(s->user, e->with_model[i], AT_NO_MODEL, cx + 1.0f, y + 17.0f, 20.0f, 20.0f, 0, 0);
        }
        y += 44.0f;
    }
    if (e->from_text[0] != '\0' && y + 40.0f <= bottom) {
        at_text(s, o, AT_R_CAP12, "FROM", x, y + 11.0f, AT_C_DIM, AT_ALIGN_LEFT, 0.0f);
        at_part_tag(s, o, x, y + 16.0f, e->from_text, AT_TAG_JADE, w);
    }
}

void at_part_footer(const AtSink *s, const AtTextOps *o, AtRect r, const AtFooter *f)
{
    float x = r.x + 14.0f, cy = r.y + r.h * 0.5f, mx, tx, right = r.x + r.w - 10.0f;
    int eq;
    at_plate(s, r, AT_C_PLATE2, AT_C_EDGE2, 3.0f, (float) AT_PX_CH_S);
    fit_text(s, o, AT_R_CAP14, f->label, x, mid_base(r.y, r.h - 3.0f, AT_R_CAP14), AT_C_MUTED, AT_ALIGN_LEFT, 76.0f);
    mx = x + 88.0f;
    eq = mx + 134.0f <= right;                                           /* the combine strip needs its full 134 px, else it is left out */
    if (eq) {
        if (f->model_a != AT_NO_MODEL) s->model(s->user, f->model_a, AT_NO_MODEL, mx, cy - 14.0f, 28.0f, 28.0f, 0, 0);
        glyph_plus(s, mx + 40.0f, cy, 8.0f, 2.0f, AT_C_MUTED);
        if (f->model_b != AT_NO_MODEL) s->model(s->user, f->model_b, AT_NO_MODEL, mx + 52.0f, cy - 14.0f, 28.0f, 28.0f, 1, 0);
        tri(s, mx + 90.0f, cy - 5.0f, mx + 100.0f, cy, mx + 90.0f, cy + 5.0f, AT_C_JADE);
        if (f->model_out != AT_NO_MODEL) s->model(s->user, f->model_out, AT_NO_MODEL, mx + 106.0f, cy - 14.0f, 28.0f, 28.0f, 0, 0);
    }
    tx = eq ? mx + 146.0f : mx;
    fit_text(s, o, AT_R_BODY14, f->text, tx, mid_base(r.y, r.h - 3.0f, AT_R_BODY14), AT_C_TEXT2, AT_ALIGN_LEFT, right - tx);
}

void at_part_note(const AtSink *s, const AtTextOps *o, AtRect r, const char *text, int kind, float remaining)
{
    unsigned tone = kind == AT_NOTE_OK ? AT_C_JADE : kind == AT_NOTE_WARN ? AT_C_SUN : kind == AT_NOTE_ERR ? AT_C_ROSE : AT_C_LINE2;
    AtRect ic;
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
    ic.x = r.x; ic.y = r.y; ic.w = 30.0f; ic.h = r.h - 3.0f;
    at_poly_rect(s, ic.x, ic.y + (float) AT_PX_CH_S, ic.w, ic.h - (float) AT_PX_CH_S, tone);          /* the icon follows the chamfered corner */
    at_poly_rect(s, ic.x + (float) AT_PX_CH_S, ic.y, ic.w - (float) AT_PX_CH_S, (float) AT_PX_CH_S, tone);
    glyph_plus(s, ic.x + 15.0f, ic.y + ic.h * 0.5f, 10.0f, 2.0f, AT_C_INK);
    fit_text(s, o, AT_R_BODY14, text, r.x + 38.0f, mid_base(r.y, r.h - 3.0f, AT_R_BODY14), AT_C_IVORY, AT_ALIGN_LEFT, r.w - 46.0f);
    if (remaining < 0.0f) remaining = 0.0f;
    if (remaining > 1.0f) remaining = 1.0f;
    at_poly_rect(s, r.x, r.y + r.h - 5.0f, (r.w - 8.0f) * remaining, 2.0f, tone);                      /* the timer rule drains */
}

int at_part_dialog(const AtSink *s, const AtTextOps *o, float canvas_w, const AtDialog *d, float rise, AtRect btn[2])
{
    float w = 332.0f, pad = 16.0f, x, y, h, bx, by;
    char lines[5][96];
    int n, i, clamped = 0, nb = d->n > 2 ? 2 : d->n;
    at_poly_rect(s, 0.0f, 0.0f, canvas_w, 480.0f, AT_C_SCRIM);
    n = at_wrap(o, AT_R_BODY14, d->body, w - 2.0f * pad, 4, lines, &clamped);
    h = 14.0f + 24.0f + 6.0f + 18.0f * (float) n + 14.0f + 34.0f + 16.0f;
    x = (canvas_w - w) * 0.5f;
    y = (480.0f - h) * 0.5f + rise;
    {
        AtRect pr;
        pr.x = x; pr.y = y; pr.w = w; pr.h = h;
        at_plate(s, pr, AT_C_PLATE, AT_C_EDGE, 6.0f, (float) AT_PX_CH);
    }
    fit_text(s, o, AT_R_CAP20, d->title, x + pad, y + 34.0f, AT_C_IVORY, AT_ALIGN_LEFT, w - 2.0f * pad);
    for (i = 0; i < n; i++) at_text(s, o, AT_R_BODY14, lines[i], x + pad, y + 57.0f + 18.0f * (float) i, AT_C_TEXT2, AT_ALIGN_LEFT, 0.0f);
    by = y + h - 16.0f - 34.0f;
    bx = x + w - pad;
    for (i = nb - 1; i >= 0; i--) {
        char fit[200];
        float cap = (w - 2.0f * pad - 8.0f * (float) (nb - 1) - 58.0f * (float) nb) / (float) nb, tw = 0.0f, bw, bt;
        int f = d->focus == i, fr = AT_R_CAP16;
        AtRect br, pr;
        fit[0] = '\0';
        if (cap >= 8.0f) { fr = at_fit(o, AT_R_CAP16, d->label[i], cap, fit, sizeof fit); tw = twidth(o, fr, fit); }
        bw = tw + 18.0f + 22.0f + 18.0f;
        bx -= bw;
        br.x = bx; br.y = by; br.w = bw; br.h = 34.0f;
        bt = f ? by - 2.0f : by;                                         /* focus: lifted 2 px, an ember front edge and tick, brackets */
        pr.x = br.x; pr.y = bt; pr.w = bw; pr.h = 34.0f;
        at_plate(s, pr, f ? AT_C_LIFT : AT_C_PLATE2, f ? AT_C_EMBER : AT_C_EDGE2, 3.0f, (float) AT_PX_CH_S);
        if (f) {
            at_poly_rect(s, br.x, bt + (float) AT_PX_CH_S, 4.0f, 34.0f - 3.0f - (float) AT_PX_CH_S, AT_C_EMBER);
            brackets(s, pr, AT_C_EMBER);
        }
        at_disc(s, br.x + 18.0f, bt + 15.0f, 7.0f, d->btn[i] == 'A' ? AT_C_PAD_A : d->btn[i] == 'B' ? AT_C_PAD_B : AT_C_PAD_X);
        if (fit[0] != '\0') s->text(s->user, br.x + 18.0f + 14.0f + 8.0f, mid_base(bt, 31.0f, AT_R_CAP16), fit, fr, f ? AT_C_IVORY : AT_C_TEXT2, AT_ALIGN_LEFT, 0.0f);
        btn[i] = br;
        bx -= 8.0f;
    }
    return nb;
}

/* ---- step 3: the offer card and the in-match HUD parts ------------------------------------------------------------------------
 * Every plate is at_plate(.., 3, 5): nothing else places a vertex in a cut corner (top-left and bottom-right), so tags, emblems and
 * wells are inset by the chamfer. Text goes only through fit_text (one line) or at_wrap (a stated number of lines). HUD parts
 * (port card, strip, banner, toast, link) never take focus and never draw a focus cue: no lift, no ember, no brackets. */

#define AT_OFFER_PAD 8.0f
#define AT_OFFER_NAME_H 22.0f
#define AT_OFFER_RULE_LINES 2
#define AT_OFFER_RULE_H 15.0f
#define AT_OFFER_TAG_H 20.0f

float at_part_offer_min_h(void)
{
    /* pad, the model well (48), a gap, the name row, two rule lines, a gap, the tag, the front edge and a pad */
    return AT_OFFER_PAD + 48.0f + 6.0f + AT_OFFER_NAME_H + AT_OFFER_RULE_LINES * AT_OFFER_RULE_H + 6.0f + AT_OFFER_TAG_H + 3.0f + 6.0f;
}

/* an offer card: the model well (or a keystone arch stone with its letter), the name, ONE rule (two lines at most) and a bottom tag.
 * Focus is three cues: lift 2 px, the ember front edge, four brackets in the focusing seat's colour. */
void at_part_offer(const AtSink *s, const AtTextOps *o, AtRect r, const AtOffer *c, int state, unsigned focus_rgba)
{
    int disabled = state == AT_ST_DISABLED, focus = state == AT_ST_FOCUS;
    float y = focus ? r.y - 2.0f : r.y, inner = r.w - 2.0f * AT_OFFER_PAD, well_h, ty, wx, wy;
    unsigned face = focus ? AT_C_LIFT : (disabled ? AT_C_PLATE2 : AT_C_PLATE);
    char lines[AT_OFFER_RULE_LINES][96];
    int n, i, clamped = 0;
    AtRect pr;
    pr.x = r.x; pr.y = y; pr.w = r.w; pr.h = r.h;
    at_plate(s, pr, face, focus ? AT_C_EMBER : AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
    /* the well takes what the text rows leave, between 48 and 150 px */
    well_h = r.h - (AT_OFFER_PAD + 6.0f + AT_OFFER_NAME_H + AT_OFFER_RULE_LINES * AT_OFFER_RULE_H + 6.0f + AT_OFFER_TAG_H + 3.0f + 6.0f);
    if (well_h < 48.0f) well_h = 48.0f;
    if (well_h > 150.0f) well_h = 150.0f;
    wx = r.x + AT_OFFER_PAD; wy = y + AT_OFFER_PAD;
    at_poly_rect(s, wx, wy, inner, well_h, AT_C_GROUND2);
    if (c->model >= 0) {
        s->model(s->user, c->model, c->ring, wx + 2.0f, wy + 2.0f, inner - 4.0f, well_h - 4.0f, focus, disabled ? 1 : 0);
    } else if (c->letter != 0) {                                        /* a keystone: its arch stone and letter, in the keystone's colour */
        float sw = 40.0f, sh = 46.0f, sx = wx + (inner - sw) * 0.5f, sy = wy + (well_h - sh) * 0.5f;
        char one[2];
        poly4(s, sx + sw * 0.2f, sy, sx + sw * 0.8f, sy, sx + sw, sy + sh, sx, sy + sh, c->rgba != 0 ? c->rgba : AT_C_LINE2);
        one[0] = c->letter; one[1] = '\0';
        at_text(s, o, AT_R_CAP20, one, sx + sw * 0.5f, sy + sh - 12.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
    }
    ty = wy + well_h + 6.0f;
    fit_text(s, o, AT_R_CAP16, c->name, wx, ty + 15.0f, disabled ? AT_C_DIM : AT_C_IVORY, AT_ALIGN_LEFT, inner);
    ty += AT_OFFER_NAME_H;
    n = at_wrap(o, AT_R_BODY12, c->rule, inner, AT_OFFER_RULE_LINES, lines, &clamped);
    for (i = 0; i < n; i++) at_text(s, o, AT_R_BODY12, lines[i], wx, ty + 11.0f + AT_OFFER_RULE_H * (float) i, disabled ? AT_C_DIM : AT_C_TEXT2, AT_ALIGN_LEFT, 0.0f);
    if (c->tag[0] != '\0') at_part_tag(s, o, wx, y + r.h - 3.0f - 6.0f - AT_OFFER_TAG_H, c->tag, c->tag_tone, inner);
    if (focus) brackets(s, pr, focus_rgba != 0 ? focus_rgba : AT_C_EMBER);
}

static unsigned port_rgba(int port, int cpu)
{
    static const unsigned P[4] = { AT_C_P1, AT_C_P2, AT_C_P3, AT_C_P4 };
    if (cpu) return AT_C_CPU;
    return (port >= 1 && port <= 4) ? P[port - 1] : AT_C_CPU;
}

/* an opponent or player card: a 3 px top edge in the port colour, the name, the percent and the stock pips. No focus. */
void at_part_port_card(const AtSink *s, const AtTextOps *o, AtRect r, const AtPortCard *c)
{
    char pct[16];
    float x, pw;
    int i, st = c->stocks < 0 ? 0 : (c->stocks > 5 ? 5 : c->stocks);
    unsigned col = port_rgba(c->port, c->cpu);
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
    at_poly_rect(s, r.x + (float) AT_PX_CH_S, r.y, r.w - (float) AT_PX_CH_S, 3.0f, col);        /* from the cut, never over it */
    snprintf(pct, sizeof pct, "%d%%", c->percent);
    pw = twidth(o, AT_R_NUM16, pct);
    at_text(s, o, AT_R_NUM16, pct, r.x + r.w - 12.0f, r.y + 24.0f, AT_C_IVORY, AT_ALIGN_RIGHT, 0.0f);
    fit_text(s, o, AT_R_CAP14, c->name, r.x + 12.0f, r.y + 20.0f, AT_C_IVORY, AT_ALIGN_LEFT, r.w - 36.0f - pw);
    x = r.x + 12.0f;
    for (i = 0; i < st; i++) { at_poly_rect(s, x, r.y + r.h - 3.0f - 10.0f - 4.0f, 8.0f, 10.0f, col); x += 12.0f; }
}

/* the build strip: slot pips (a 12 px square, the drive's colour inside a 2 px rarity ring), at most six keystone arch stones with
 * their letters, "+n" for the rest, then "n waiting". No plate, no focus. */
void at_part_strip(const AtSink *s, const AtTextOps *o, AtRect r, const AtStrip *st)
{
    float x = r.x, right = r.x + r.w, w;
    int i, shown = st->n_keys > 6 ? 6 : st->n_keys, np = st->n_pips > 8 ? 8 : st->n_pips;
    for (i = 0; i < np; i++) {
        at_poly_rect(s, x, r.y + 4.0f, 12.0f, 12.0f, st->pip_ring[i]);
        at_poly_rect(s, x + 2.0f, r.y + 6.0f, 8.0f, 8.0f, st->pip_fill[i]);
        x += 16.0f;
    }
    if (np > 0 && shown > 0) x += 6.0f;
    for (i = 0; i < shown; i++) {
        char one[2];
        poly4(s, x + 3.0f, r.y + 2.0f, x + 11.0f, r.y + 2.0f, x + 14.0f, r.y + 18.0f, x, r.y + 18.0f, st->key_rgba[i]);
        one[0] = st->key_letter[i]; one[1] = '\0';
        if (one[0] != '\0') at_text(s, o, AT_R_CAP12, one, x + 7.0f, r.y + 15.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
        x += 18.0f;
    }
    if (st->n_keys > shown) {
        char more[12];
        snprintf(more, sizeof more, "+%d", st->n_keys - shown);
        w = fit_text(s, o, AT_R_NUM12, more, x, r.y + 15.0f, AT_C_MUTED, AT_ALIGN_LEFT, right - x);
        x += w + 6.0f;
    }
    if (st->wait[0] != '\0' && right - x - 4.0f >= 24.0f)         /* under 24 px a fitted word is only an ellipsis: leave it out */
        fit_text(s, o, AT_R_BODY12, st->wait, x + 4.0f, r.y + 15.0f, AT_C_MUTED, AT_ALIGN_LEFT, right - x - 4.0f);
}

/* a banner: the pad glyph, one line. progress >= 0 draws a thin sun-coloured rule along the bottom (the leave progress). No focus. */
void at_part_banner(const AtSink *s, const AtTextOps *o, AtRect r, char btn, const char *text, float progress)
{
    float x = r.x + 14.0f, adv;
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
    adv = at_part_hint(s, o, x, r.y + (r.h - 3.0f) * 0.5f + 5.0f, btn, "");
    fit_text(s, o, AT_R_CAP16, text, x + adv - 12.0f, mid_base(r.y, r.h - 3.0f, AT_R_CAP16), AT_C_IVORY, AT_ALIGN_LEFT, r.x + r.w - 14.0f - (x + adv - 12.0f));
    if (progress >= 0.0f) {
        if (progress > 1.0f) progress = 1.0f;
        at_poly_rect(s, r.x, r.y + r.h - 5.0f, (r.w - 8.0f) * progress, 2.0f, AT_C_SUN);
    }
}

/* a toast: the emblem square, a title line, one rule line (fitted, never wrapped) and the drain rule along the bottom. No focus. */
void at_part_toast(const AtSink *s, const AtTextOps *o, AtRect r, unsigned emblem_rgba, const char *title, const char *rule, float remaining)
{
    float tx = r.x + 8.0f + 22.0f + 8.0f, tw = r.x + r.w - 8.0f - tx;
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
    at_poly_rect(s, r.x + 8.0f, r.y + 8.0f, 22.0f, 22.0f, emblem_rgba);
    fit_text(s, o, AT_R_CAP14, title, tx, r.y + 18.0f, AT_C_IVORY, AT_ALIGN_LEFT, tw);
    fit_text(s, o, AT_R_BODY12, rule, tx, r.y + 33.0f, AT_C_TEXT2, AT_ALIGN_LEFT, tw);
    if (remaining < 0.0f) remaining = 0.0f;
    if (remaining > 1.0f) remaining = 1.0f;
    at_poly_rect(s, r.x, r.y + r.h - 5.0f, (r.w - 8.0f) * remaining, 2.0f, emblem_rgba);
}

/* a flat quad along a segment, th thick (a grid link between two cells, a synergy chain): nothing for a zero-length segment */
void at_part_link(const AtSink *s, float x0, float y0, float x1, float y1, float th, unsigned rgba)
{
    float dx = x1 - x0, dy = y1 - y0, len = (float) sqrt((double) (dx * dx + dy * dy)), nx, ny;
    if (len < 0.001f) return;
    nx = -dy / len * th * 0.5f; ny = dx / len * th * 0.5f;
    poly4(s, x0 + nx, y0 + ny, x1 + nx, y1 + ny, x1 - nx, y1 - ny, x0 - nx, y0 - ny, rgba);
}
