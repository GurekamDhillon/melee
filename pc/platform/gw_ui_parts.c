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
    at_text(s, o, AT_R_ROW16, text, x + 22.0f + mid * 0.5f, mid_base(cy - 9.0f, 18.0f, AT_R_ROW16), AT_C_IVORY, AT_ALIGN_CENTER, mid);
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
    float e = 3.0f, y = r.y, right, cy, vw = 0.0f, lx;
    int disabled = state == AT_ST_DISABLED || (it->flags & AT_CELL_DISABLED), focus = state == AT_ST_FOCUS && !disabled;
    AtRect pr;
    if (disabled) { face = AT_C_PLATE; txt = AT_C_DIM; val = AT_C_DIM; }
    else if (focus) { face = AT_C_LIFT; edge = AT_C_EMBER; txt = AT_C_IVORY; val = AT_C_IVORY; y -= 2.0f; }
    else if (state == AT_ST_PRESS) { face = AT_C_PLATE; edge = AT_C_EMBER_D; txt = AT_C_IVORY; e = 1.0f; y += 1.0f; }
    pr.x = r.x; pr.y = y; pr.w = r.w; pr.h = r.h;
    at_plate(s, pr, face, edge, e, (float) AT_PX_CH_S);
    if (focus) at_poly_rect(s, r.x, y, 4.0f, r.h - e, AT_C_EMBER);
    if (state == AT_ST_PRESS) at_poly_rect(s, r.x, y, 4.0f, r.h - e, AT_C_EMBER_D);
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
    if (it->sub[0] != '\0') {
        at_text(s, o, AT_R_ROW16, it->label, lx, y + 15.0f, txt, AT_ALIGN_LEFT, right - lx - vw - 8.0f);
        at_text(s, o, AT_R_BODY12, it->sub, lx, y + r.h - e - 4.0f, AT_C_MUTED, AT_ALIGN_LEFT, right - lx - vw - 8.0f);
    } else {
        at_text(s, o, AT_R_ROW16, it->label, lx, mid_base(y, r.h - e, AT_R_ROW16), txt, AT_ALIGN_LEFT, right - lx - vw - 8.0f);
    }
}

void at_part_tabs(const AtSink *s, const AtTextOps *o, AtRect r, const char *const *names, const int *counts, int n, int active, int focus_tab)
{
    float x = r.x, bottom = r.y + r.h;
    int i;
    for (i = 0; i < n; i++) {
        char num[16];
        float tw = twidth(o, AT_R_CAP16, names[i]), cw = 0.0f, w, h = i == active ? 30.0f : 26.0f, y = bottom - h;
        snprintf(num, sizeof num, "%d", counts != NULL ? counts[i] : 0);
        if (counts != NULL) cw = twidth(o, AT_R_NUM12, num) + 6.0f;
        w = tw + cw + 28.0f;
        at_poly_rect(s, x, y, w, h, i == active ? AT_C_PLATE : AT_C_GROUND2);
        at_text(s, o, AT_R_CAP16, names[i], x + 14.0f, mid_base(y, h, AT_R_CAP16), i == active ? AT_C_IVORY : AT_C_MUTED, AT_ALIGN_LEFT, 0.0f);
        if (counts != NULL) at_text(s, o, AT_R_NUM12, num, x + 14.0f + tw + 6.0f, mid_base(y, h, AT_R_NUM12), i == active ? AT_C_EMBER : AT_C_DIM, AT_ALIGN_LEFT, 0.0f);
        if (i == focus_tab) at_poly_rect(s, x, y, w, 2.0f, AT_C_EMBER);
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
    at_text(s, o, AT_R_CAP12, text, x + 8.0f, mid_base(y, 20.0f, AT_R_CAP12), ink, AT_ALIGN_LEFT, w - 16.0f);
    return w;
}

/* <<< Task 8 appends the second half of the parts here >>> */
