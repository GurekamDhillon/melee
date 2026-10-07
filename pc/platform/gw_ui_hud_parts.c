#include "gw_ui_hud_parts.h"
#include "gw_ui_tokens.h"
#include <stdio.h>
#include <string.h>

/* every text goes through the fit rule with a real width: at_fit reads max_w <= 0 as "no limit", so a box with no room draws nothing instead */
static void put(const AtSink *s, const AtTextOps *o, int role, const char *str, float x, float base, unsigned rgba, int align, float max_w)
{
    if (str == NULL || str[0] == '\0' || max_w < 8.0f) return;
    at_text(s, o, role, str, x, base, rgba, align, max_w);
}
static float twidth(const AtTextOps *o, int role, const char *s) { return o->width(o->user, role, s); }
static int clampi(int v, int lo, int hi) { return v < lo ? lo : v > hi ? hi : v; }

/* ---- the readout ---- */
static int eff_cols(const AtReadout *d, float w) { return d->cols >= 2 && w >= 380.0f ? 2 : 1; }

float at_readout_height(const AtReadout *d)
{
    int rows = d->n > 0 ? (d->n > AT_READ_ROWS ? AT_READ_ROWS : d->n) : 0, cols = d->cols >= 2 ? 2 : 1;
    int per = (rows + cols - 1) / cols;
    return 36.0f + 16.0f * (float) per + 10.0f;
}

static unsigned tone_rgba(int tone)
{
    return tone == 1 ? AT_C_JADE : tone == 2 ? AT_C_SUN : tone == 3 ? AT_C_ROSE : AT_C_IVORY;
}

void at_part_readout(const AtSink *s, const AtTextOps *o, AtRect r, const AtReadout *d)
{
    int cols = eff_cols(d, r.w), per, i, n = d->n > AT_READ_ROWS ? AT_READ_ROWS : d->n;
    float cw;
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
    put(s, o, AT_R_CAP12, d->title, r.x + 12.0f, r.y + 20.0f, AT_C_MUTED, AT_ALIGN_LEFT, r.w - 24.0f);
    per = (n + cols - 1) / cols;
    if (per < 1) per = 1;
    cw = (r.w - 24.0f) / (float) cols;
    for (i = 0; i < n; i++) {
        int c = i / per, k = i % per;
        float x = r.x + 12.0f + cw * (float) c, y = r.y + 36.0f + 16.0f * (float) k + 12.0f, gap = c + 1 < cols ? 12.0f : 0.0f;
        put(s, o, AT_R_BODY12, d->row[i].label, x, y, AT_C_MUTED, AT_ALIGN_LEFT, cw * 0.42f);
        put(s, o, AT_R_NUM12, d->row[i].value, x + cw - gap, y, tone_rgba(d->row[i].tone), AT_ALIGN_RIGHT, cw * 0.5f - gap);
    }
}

static int tone_of(const char *t) { return t == NULL ? 0 : strcmp(t, "ok") == 0 ? 1 : strcmp(t, "warn") == 0 ? 2 : strcmp(t, "bad") == 0 ? 3 : 0; }

int at_readout_from_val(const AtvArena *a, int t, AtReadout *out, char *err, int errcap)
{
    int rows, n, i;
    memset(out, 0, sizeof *out);
    if (atv_kind(a, t) != ATV_TABLE) { snprintf(err, (size_t) errcap, "%s", "gd.ui.hud: a readout is a table"); return 0; }
    snprintf(out->title, sizeof out->title, "%s", atv_strv(a, atv_get(a, t, "title"), ""));
    out->cols = (int) atv_numv(a, atv_get(a, t, "cols"), 1.0) >= 2 ? 2 : 1;
    rows = atv_get(a, t, "rows");
    n = atv_len(a, rows);
    if (n > AT_READ_ROWS) { snprintf(err, (size_t) errcap, "gd.ui.hud: a readout has at most %d rows", AT_READ_ROWS); return 0; }
    for (i = 0; i < n; i++) {
        int r = atv_at(a, rows, i + 1);
        if (atv_kind(a, r) != ATV_TABLE) { snprintf(err, (size_t) errcap, "gd.ui.hud: readout row %d is not a table", i + 1); return 0; }
        snprintf(out->row[i].label, sizeof out->row[i].label, "%s", atv_strv(a, atv_get(a, r, "label"), ""));
        snprintf(out->row[i].value, sizeof out->row[i].value, "%s", atv_strv(a, atv_get(a, r, "value"), ""));
        out->row[i].tone = tone_of(atv_strv(a, atv_get(a, r, "tone"), NULL));
    }
    out->n = n;
    return 1;
}

/* ---- the track ---- */
float at_track_height(void) { return 78.0f; }

static unsigned span_rgba(int id) { return id == 0 ? AT_C_ROSE : id == 1 ? AT_C_SUN : id == 2 ? AT_C_JADE : id == 3 ? AT_C_TEXT2 : AT_C_DIM; }

static void tri_up(const AtSink *s, float cx, float y, float w, unsigned c)
{
    float x[4], yy[4];
    x[0] = cx - w * 0.5f; x[1] = cx + w * 0.5f; x[2] = cx; x[3] = cx;
    yy[0] = y + w; yy[1] = y + w; yy[2] = y; yy[3] = y;
    s->poly(s->user, x, yy, c);
}
static void tri_down(const AtSink *s, float cx, float y, float w, unsigned c)
{
    float x[4], yy[4];
    x[0] = cx - w * 0.5f; x[1] = cx + w * 0.5f; x[2] = cx; x[3] = cx;
    yy[0] = y; yy[1] = y; yy[2] = y + w; yy[3] = y + w;
    s->poly(s->user, x, yy, c);
}

void at_part_track(const AtSink *s, const AtTextOps *o, AtRect r, const AtTrack *t)
{
    int len = t->len < 1 ? 1 : t->len, i, now = clampi(t->now, 1, len);
    float iw = r.w - 24.0f, x0 = r.x + 16.0f, w = r.w - 32.0f, sx = w / (float) len, by = r.y + 36.0f;
    int n_spans = t->n_spans > AT_TRACK_SPANS ? AT_TRACK_SPANS : t->n_spans, n_marks = t->n_marks > AT_TRACK_MARKS ? AT_TRACK_MARKS : t->n_marks;
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
    put(s, o, AT_R_CAP12, t->title, r.x + 12.0f, r.y + 18.0f, AT_C_IVORY, AT_ALIGN_LEFT, iw * 0.56f);
    put(s, o, AT_R_NUM12, t->right, r.x + r.w - 12.0f, r.y + 18.0f, AT_C_MUTED, AT_ALIGN_RIGHT, iw * 0.40f);
    at_poly_rect(s, x0, by, w, 8.0f, AT_C_GROUND2);                                  /* the well */
    for (i = 5; i <= len; i += 5) at_poly_rect(s, x0 + (float) (i - 1) * sx, by + 9.0f, 1.0f, i % 10 == 0 ? 4.0f : 2.0f, AT_C_LINE2);
    for (i = 0; i < n_spans; i++) {
        int a = clampi(t->span[i].from, 1, len), b = clampi(t->span[i].to, 1, len);
        float sw, left;
        if (b < a) b = a;
        sw = (float) (b - a + 1) * sx;
        left = x0 + (float) (a - 1) * sx;
        if (sw < 2.0f) sw = 2.0f;
        if (left + sw > x0 + w) sw = x0 + w - left;
        at_poly_rect(s, left, by, sw, 8.0f, span_rgba(t->span[i].id));
        {   /* the id as a number over the window when its text fits: colour is not the only signal */
            char num[12];
            snprintf(num, sizeof num, "#%d", t->span[i].id);
            if (sw >= twidth(o, AT_R_NUM12, num) + 4.0f) put(s, o, AT_R_NUM12, num, left + sw * 0.5f, by - 5.0f, span_rgba(t->span[i].id), AT_ALIGN_CENTER, sw);
        }
    }
    for (i = 0; i < n_marks; i++) {
        float mx = x0 + (float) (clampi(t->mark[i].frame, 1, len) - 1) * sx + sx * 0.5f, my = by + 14.0f;
        if (mx < x0 + 3.5f) mx = x0 + 3.5f;
        if (mx > x0 + w - 3.5f) mx = x0 + w - 3.5f;                                  /* a mark on the last frame stays inside the well */
        switch (t->mark[i].kind) {
        case AT_MK_IASA: at_poly_rect(s, mx - 1.5f, my, 3.0f, 10.0f, AT_C_JADE); break;          /* a bar */
        case AT_MK_INVINC: at_poly_rect(s, mx - 2.5f, my + 2.0f, 5.0f, 5.0f, AT_C_IVORY); break;   /* a square */
        case AT_MK_GFX: tri_up(s, mx, my + 1.0f, 7.0f, AT_C_SUN); break;                          /* a triangle up */
        case AT_MK_SFX: tri_down(s, mx, my + 1.0f, 7.0f, AT_C_ROSE); break;                       /* a triangle down */
        default: at_disc(s, mx, my + 5.0f, 3.0f, AT_C_MUTED); break;                              /* a disc */
        }
    }
    {   /* the playhead: a bar, inside the well whatever 'now' is */
        float px = x0 + (float) (now - 1) * sx + sx * 0.5f;
        if (px < x0 + 1.0f) px = x0 + 1.0f;
        if (px > x0 + w - 1.0f) px = x0 + w - 1.0f;
        at_poly_rect(s, px - 1.0f, by - 3.0f, 2.0f, 14.0f, AT_C_IVORY);
    }
    put(s, o, AT_R_BODY12, t->note, r.x + 12.0f, r.y + 72.0f, AT_C_MUTED, AT_ALIGN_LEFT, iw);
}

static int kind_of(const char *k)
{
    return strcmp(k, "iasa") == 0 ? AT_MK_IASA : strcmp(k, "invinc") == 0 ? AT_MK_INVINC : strcmp(k, "gfx") == 0 ? AT_MK_GFX : strcmp(k, "sfx") == 0 ? AT_MK_SFX : AT_MK_VIS;
}

int at_track_from_val(const AtvArena *a, int t, AtTrack *out, char *err, int errcap)
{
    int spans, marks, n, i;
    memset(out, 0, sizeof *out);
    if (atv_kind(a, t) != ATV_TABLE) { snprintf(err, (size_t) errcap, "%s", "gd.ui.hud: a track is a table"); return 0; }
    snprintf(out->title, sizeof out->title, "%s", atv_strv(a, atv_get(a, t, "title"), ""));
    snprintf(out->right, sizeof out->right, "%s", atv_strv(a, atv_get(a, t, "right"), ""));
    snprintf(out->note, sizeof out->note, "%s", atv_strv(a, atv_get(a, t, "note"), ""));
    out->len = (int) atv_numv(a, atv_get(a, t, "len"), 1.0);
    out->now = (int) atv_numv(a, atv_get(a, t, "now"), 1.0);
    spans = atv_get(a, t, "spans"); n = atv_len(a, spans);
    if (n > AT_TRACK_SPANS) { snprintf(err, (size_t) errcap, "gd.ui.hud: a track has at most %d spans and %d marks", AT_TRACK_SPANS, AT_TRACK_MARKS); return 0; }
    for (i = 0; i < n; i++) {
        int sp = atv_at(a, spans, i + 1);
        out->span[out->n_spans].from = (int) atv_numv(a, atv_get(a, sp, "from"), 1.0);
        out->span[out->n_spans].to = (int) atv_numv(a, atv_get(a, sp, "to"), 1.0);
        out->span[out->n_spans].id = (int) atv_numv(a, atv_get(a, sp, "id"), 0.0);
        out->n_spans++;
    }
    marks = atv_get(a, t, "marks"); n = atv_len(a, marks);
    if (n > AT_TRACK_MARKS) { snprintf(err, (size_t) errcap, "gd.ui.hud: a track has at most %d spans and %d marks", AT_TRACK_SPANS, AT_TRACK_MARKS); return 0; }
    for (i = 0; i < n; i++) {
        int mk = atv_at(a, marks, i + 1);
        out->mark[out->n_marks].frame = (int) atv_numv(a, atv_get(a, mk, "frame"), 1.0);
        out->mark[out->n_marks].kind = kind_of(atv_strv(a, atv_get(a, mk, "kind"), "vis"));
        out->n_marks++;
    }
    return 1;
}

/* ---- the chip strip ---- */
float at_chips_height(void) { return 24.0f; }

void at_part_chips(const AtSink *s, const AtTextOps *o, AtRect r, const AtChips *d)
{
    float x = r.x, right_w = d->right[0] ? twidth(o, AT_R_NUM12, d->right) + 16.0f : 0.0f, limit;
    int i, n = d->n > AT_CHIPS_MAX ? AT_CHIPS_MAX : d->n;
    if (right_w > r.w * 0.45f) right_w = 0.0f;                                       /* the chips matter more than the counter: on a narrow strip it is left off */
    limit = r.x + r.w - right_w;
    for (i = 0; i < n; i++) {
        const AtChip *c = &d->c[i];
        float tw = twidth(o, AT_R_CAP12, c->text), cw = tw + (c->on >= 0 ? 30.0f : 20.0f);
        AtRect cr;
        unsigned face = AT_C_PLATE, edge = AT_C_EDGE, ink = c->on == 0 ? AT_C_DIM : AT_C_IVORY;
        if (c->text[0] == '\0') continue;
        if (x + cw > limit) break;                                                   /* a chip that does not fit ends the row: none is cut or overlapped */
        cr.x = x; cr.y = r.y; cr.w = cw; cr.h = r.h;
        at_plate(s, cr, face, edge, 2.0f, (float) AT_PX_CH_XS);
        if (c->tone == 1) at_poly_rect(s, cr.x + (float) AT_PX_CH_XS, cr.y + r.h - 4.0f, cr.w - (float) AT_PX_CH_XS, 2.0f, AT_C_JADE);   /* the mode: a bar under it and its name */
        else if (c->tone == 2) at_poly_rect(s, cr.x + (float) AT_PX_CH_XS, cr.y + r.h - 4.0f, cr.w - (float) AT_PX_CH_XS, 2.0f, AT_C_SUN);
        if (c->on >= 0) {                                                            /* a toggle: a filled square when on, an outline when off */
            float sx = cr.x + 10.0f, sy = cr.y + r.h * 0.5f - 4.0f;
            if (c->on) at_poly_rect(s, sx, sy, 8.0f, 8.0f, AT_C_JADE);
            else { at_poly_rect(s, sx, sy, 8.0f, 1.0f, AT_C_DIM); at_poly_rect(s, sx, sy + 7.0f, 8.0f, 1.0f, AT_C_DIM); at_poly_rect(s, sx, sy + 1.0f, 1.0f, 6.0f, AT_C_DIM); at_poly_rect(s, sx + 7.0f, sy + 1.0f, 1.0f, 6.0f, AT_C_DIM); }
            put(s, o, AT_R_CAP12, c->text, cr.x + 22.0f, cr.y + r.h * 0.5f + 4.0f, ink, AT_ALIGN_LEFT, cr.w - 26.0f);
        } else {
            put(s, o, AT_R_CAP12, c->text, cr.x + 10.0f, cr.y + r.h * 0.5f + 4.0f, ink, AT_ALIGN_LEFT, cr.w - 14.0f);
        }
        x += cw + 6.0f;
    }
    if (right_w > 0.0f) put(s, o, AT_R_NUM12, d->right, r.x + r.w - 4.0f, r.y + r.h * 0.5f + 4.0f, AT_C_MUTED, AT_ALIGN_RIGHT, right_w - 8.0f);
}

int at_chips_from_val(const AtvArena *a, int t, AtChips *out, char *err, int errcap)
{
    int items, n, i;
    memset(out, 0, sizeof *out);
    if (atv_kind(a, t) != ATV_TABLE) { snprintf(err, (size_t) errcap, "%s", "gd.ui.hud: a chip strip is a table"); return 0; }
    snprintf(out->right, sizeof out->right, "%s", atv_strv(a, atv_get(a, t, "right"), ""));
    items = atv_get(a, t, "items");
    n = atv_len(a, items);
    if (n > AT_CHIPS_MAX) { snprintf(err, (size_t) errcap, "gd.ui.hud: a chip strip has at most %d chips", AT_CHIPS_MAX); return 0; }
    for (i = 0; i < n; i++) {
        int c = atv_at(a, items, i + 1), on;
        const char *tn;
        if (atv_kind(a, c) != ATV_TABLE) { snprintf(err, (size_t) errcap, "gd.ui.hud: chip %d is not a table", i + 1); return 0; }
        snprintf(out->c[i].text, sizeof out->c[i].text, "%s", atv_strv(a, atv_get(a, c, "text"), ""));
        tn = atv_strv(a, atv_get(a, c, "tone"), "");
        out->c[i].tone = strcmp(tn, "ok") == 0 ? 1 : strcmp(tn, "warn") == 0 ? 2 : 0;
        on = atv_get(a, c, "on");
        out->c[i].on = atv_kind(a, on) == ATV_BOOL ? (atv_boolv(a, on, 0) ? 1 : 0) : -1;
    }
    out->n = n;
    return 1;
}
