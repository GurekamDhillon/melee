/* gw_ui_hud.c - the in-match HUD layer: zones in the title-safe box, the retail HUD's keep-out rectangles, stacking, the quiet-HUD caps,
 * expiry and the draw. Pure C: it draws only through AtSink and measures only through AtTextOps. See gw_ui_hud.h.
 *
 * KEEP-OUTS ARE ESTIMATES until measured in the game (plan step 3, Task 21 step 4): the retail HUD is drawn in the 4:3 band centred on
 * the canvas (ox = (canvas_w - 640) / 2). The damage plates and the stocks above them are one rectangle {ox + 40, 372, 560, 96}; the
 * match timer is {ox + 248, 16, 144, 52}. If the measurement differs, change these constants and atlas_hud_test.c together. */
#include "gw_ui_hud.h"
#include "gw_ui_stack.h"
#include "gw_ui_tokens.h"

#include <stdio.h>
#include <string.h>

#define KO_DAMAGE_X 40.0f
#define KO_DAMAGE_Y 372.0f
#define KO_DAMAGE_W 560.0f
#define KO_DAMAGE_H 96.0f
#define KO_TIMER_X 248.0f
#define KO_TIMER_Y 16.0f
#define KO_TIMER_W 144.0f
#define KO_TIMER_H 52.0f
#define HUD_GAP 8.0f
#define HUD_MID_Y 240.0f

AtRect at_hud_safe(float canvas_w)
{
    AtLayout L;
    AtRect r;
    at_layout(canvas_w, AT_PRESET_NONE, &L);
    r.x = L.content_x + 16.0f; r.w = L.content_w - 32.0f; r.y = 16.0f; r.h = 448.0f;
    return r;
}

void at_hud_retail_keepouts(float canvas_w, unsigned visible_mask, AtKeepOut *k)
{
    float ox = (canvas_w - 640.0f) * 0.5f;
    k->n = 0;
    if (ox < 0.0f) ox = 0.0f;
    if ((visible_mask & ((1u << AT_RE_HUD_DAMAGE) | (1u << AT_RE_HUD_STOCK))) != 0u) {      /* the plates and the stocks above them: one rectangle */
        k->r[k->n].x = ox + KO_DAMAGE_X; k->r[k->n].y = KO_DAMAGE_Y; k->r[k->n].w = KO_DAMAGE_W; k->r[k->n].h = KO_DAMAGE_H; k->n++;
    }
    if ((visible_mask & (1u << AT_RE_HUD_TIMER)) != 0u) {
        k->r[k->n].x = ox + KO_TIMER_X; k->r[k->n].y = KO_TIMER_Y; k->r[k->n].w = KO_TIMER_W; k->r[k->n].h = KO_TIMER_H; k->n++;
    }
}

int at_hud_cap_ok(const AtHud *h, char *why, int cap)
{
    int z, i, banners = 0, cards = 0, notes = 0;
    for (z = 0; z < AT_Z_COUNT; z++) {
        int toasts = 0;
        if (h->n[z] < 0 || h->n[z] > AT_HUD_PER_ZONE) { snprintf(why, (size_t) cap, "a zone holds at most %d parts", AT_HUD_PER_ZONE); return 0; }
        for (i = 0; i < h->n[z]; i++) {
            int kind = h->z[z][i].kind;
            if (kind == AT_HP_BANNER) {
                banners++;
                if (z != AT_Z_TOP_CENTER) { snprintf(why, (size_t) cap, "a banner lives in top_center only"); return 0; }
            } else if (kind == AT_HP_TOAST) {
                toasts++;
                if (z != AT_Z_TOP_LEFT && z != AT_Z_TOP_CENTER && z != AT_Z_TOP_RIGHT) { snprintf(why, (size_t) cap, "a toast lives in a top zone"); return 0; }
            } else if (kind == AT_HP_CARD) cards++;
            else if (kind == AT_HP_NOTE) notes++;
        }
        if (toasts > 1) { snprintf(why, (size_t) cap, "at most one toast per zone"); return 0; }
    }
    if (banners > 1) { snprintf(why, (size_t) cap, "at most one banner in the whole HUD"); return 0; }
    if (cards > 3) { snprintf(why, (size_t) cap, "at most three opponent cards"); return 0; }
    if (notes > 1) { snprintf(why, (size_t) cap, "at most one pickup note"); return 0; }
    return 1;
}

static int meets(AtRect a, AtRect b) { return a.w > 0.0f && b.w > 0.0f && a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h; }

static int expired(const AtHudPart *p, double now)
{
    return (p->kind == AT_HP_TOAST || p->kind == AT_HP_NOTE) && p->until_ms > 0.0 && now >= p->until_ms;
}

/* the size of a part */
static void part_size(const AtHudPart *p, const AtTextOps *o, float safe_w, float *w, float *h)
{
    switch (p->kind) {
    case AT_HP_PORT_CARD: *w = 188.0f; *h = 44.0f; break;
    case AT_HP_TIMER: *w = 96.0f; *h = 28.0f; break;
    case AT_HP_NOTE: {
        float tw = o->width(o->user, AT_R_BODY14, p->text) + 54.0f;
        *w = tw < 120.0f ? 120.0f : (tw > 360.0f ? 360.0f : tw); *h = 24.0f;
        break; }
    case AT_HP_STRIP: *w = 232.0f; *h = 20.0f; break;
    case AT_HP_BANNER: {                                               /* the glyph, one line, padding: fitted to its text, 160 to 300 */
        float tw = o->width(o->user, AT_R_CAP16, p->text) + 58.0f;
        *w = tw < 160.0f ? 160.0f : (tw > 300.0f ? 300.0f : tw); *h = 34.0f;
        break; }
    case AT_HP_TOAST: *w = 196.0f; *h = 44.0f; break;
    case AT_HP_CARD: *w = 196.0f; *h = 26.0f + 17.0f * (float) (p->n_lines < 0 ? 0 : (p->n_lines > 3 ? 3 : p->n_lines)); break;
    default: *w = 0.0f; *h = 0.0f; break;
    }
    if (*w > safe_w) *w = safe_w;
}

void at_hud_layout(const AtHud *h, float canvas_w, const AtKeepOut *k, double now_ms, const AtTextOps *o, AtHudLayout *out)
{
    AtRect safe = at_hud_safe(canvas_w), placed[AT_Z_COUNT * AT_HUD_PER_ZONE];
    int np = 0, z, i, j, order[AT_Z_COUNT] = { AT_Z_TOP_LEFT, AT_Z_TOP_RIGHT, AT_Z_TOP_CENTER, AT_Z_BOTTOM_LEFT, AT_Z_BOTTOM_RIGHT, AT_Z_BOTTOM_CENTER };
    memset(out, 0, sizeof *out);
    for (j = 0; j < AT_Z_COUNT; j++) {
        float cursor_top = safe.y, cursor_bottom = safe.y + safe.h;
        int top;
        z = order[j];
        top = z <= AT_Z_TOP_RIGHT;
        for (i = 0; i < h->n[z] && i < AT_HUD_PER_ZONE; i++) {
            const AtHudPart *p = &h->z[z][i];
            AtRect r;
            float w, hh;
            int moved, guard = 0, a;
            if (expired(p, now_ms)) continue;                          /* a gone toast or note takes no space */
            part_size(p, o, safe.w, &w, &hh);
            if (w <= 0.0f) continue;
            r.w = w; r.h = hh;
            r.x = (z % 3 == 0) ? safe.x : (z % 3 == 2 ? safe.x + safe.w - w : safe.x + (safe.w - w) * 0.5f);
            r.y = top ? cursor_top : cursor_bottom - hh;
            do {                                                       /* move past a keep-out or a part already placed, keeping the alignment */
                moved = 0;
                for (a = 0; a < k->n + np; a++) {
                    AtRect ob = a < k->n ? k->r[a] : placed[a - k->n];
                    if (!meets(r, ob)) continue;
                    r.y = top ? ob.y + ob.h + HUD_GAP : ob.y - HUD_GAP - hh;
                    moved = 1;
                }
            } while (moved && ++guard < 16);
            if (r.y < safe.y - 0.01f || r.y + r.h > safe.y + safe.h + 0.01f || (top ? r.y + r.h > HUD_MID_Y : r.y < HUD_MID_Y)) {
                out->dropped++;
                continue;
            }
            { int bad = 0; for (a = 0; a < k->n + np; a++) { AtRect ob = a < k->n ? k->r[a] : placed[a - k->n]; if (meets(r, ob)) bad = 1; }
              if (bad) { out->dropped++; continue; } }
            out->rect[z][i] = r;
            out->shown[z][i] = 1;
            placed[np++] = r;
            if (top) cursor_top = r.y + r.h + HUD_GAP; else cursor_bottom = r.y - HUD_GAP;
        }
    }
}

/* ---- the draw: a sink that counts what it forwards, stops at the cap, and scales alpha for the fade-in ---- */
typedef struct { const AtSink *real; int n, cap; float a; } HudSink;

static unsigned fade(unsigned rgba, float a)
{
    unsigned al = (unsigned) ((float) (rgba & 0xFFu) * a + 0.5f);
    return (rgba & 0xFFFFFF00u) | (al > 255u ? 255u : al);
}
static void hs_poly(void *u, const float x[4], const float y[4], unsigned rgba)
{
    HudSink *h = (HudSink *) u;
    if (h->n >= h->cap) return;
    h->n++;
    h->real->poly(h->real->user, x, y, fade(rgba, h->a));
}
static void hs_text(void *u, float x, float base, const char *s, int role, unsigned rgba, int align, float max_w)
{
    HudSink *h = (HudSink *) u;
    if (h->n >= h->cap) return;
    h->n++;
    h->real->text(h->real->user, x, base, s, role, fade(rgba, h->a), align, max_w);
}
static void hs_model(void *u, int model, int ring, float x, float y, float w, float hh, int focused, int dim)
{
    HudSink *h = (HudSink *) u;
    if (h->n >= h->cap) return;
    h->n++;
    h->real->model(h->real->user, model, ring, x, y, w, hh, focused, dim);
}

static void draw_card(const AtSink *s, const AtTextOps *o, AtRect r, const AtHudPart *p)
{
    int i;
    unsigned col = p->rgba != 0 ? p->rgba : AT_C_LINE2;
    at_plate(s, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
    at_poly_rect(s, r.x + (float) AT_PX_CH_S, r.y, r.w - (float) AT_PX_CH_S, 3.0f, col);
    {
        char fit[200];
        int role = at_fit(o, AT_R_CAP14, p->text, r.w - 24.0f, fit, sizeof fit);
        if (fit[0] != '\0') s->text(s->user, r.x + 12.0f, r.y + 20.0f, fit, role, AT_C_IVORY, AT_ALIGN_LEFT, 0.0f);
    }
    for (i = 0; i < p->n_lines && i < 3; i++) {
        char fit[200];
        int role = at_fit(o, AT_R_BODY12, p->lines[i], r.w - 24.0f, fit, sizeof fit);
        if (fit[0] != '\0') s->text(s->user, r.x + 12.0f, r.y + 20.0f + 17.0f * (float) (i + 1), fit, role, AT_C_TEXT2, AT_ALIGN_LEFT, 0.0f);
    }
}

void at_hud_render(const AtHud *h, const AtHudLayout *l, double now_ms, int reduced, const AtTextOps *o, const AtSink *s, int *entries)
{
    HudSink hs;
    AtSink sink;
    int z, i;
    hs.real = s; hs.n = 0; hs.cap = AT_HUD_QUAD_CAP; hs.a = 1.0f;
    sink.user = &hs; sink.poly = hs_poly; sink.text = hs_text; sink.model = hs_model;
    for (z = 0; z < AT_Z_COUNT; z++) for (i = 0; i < h->n[z] && i < AT_HUD_PER_ZONE; i++) {
        const AtHudPart *p = &h->z[z][i];
        AtRect r = l->rect[z][i];
        float remaining = 1.0f;
        if (!l->shown[z][i]) continue;
        hs.a = 1.0f;
        if ((p->kind == AT_HP_TOAST || p->kind == AT_HP_NOTE) && p->from_ms > 0.0) {
            AtTween tw;
            at_tween_start(&tw, p->from_ms, (double) AT_MS_NOTE, reduced);
            hs.a = at_tween_value(&tw, now_ms);
            if (p->until_ms > p->from_ms) {
                remaining = (float) ((p->until_ms - now_ms) / (p->until_ms - p->from_ms));
                if (remaining < 0.0f) remaining = 0.0f;
                if (remaining > 1.0f) remaining = 1.0f;
            }
        }
        switch (p->kind) {
        case AT_HP_PORT_CARD: at_part_port_card(&sink, o, r, &p->port); break;
        case AT_HP_TIMER: {
            char t[16];
            at_plate(&sink, r, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH_S);
            snprintf(t, sizeof t, "%d:%02d", p->seconds / 60, p->seconds % 60);
            at_text(&sink, o, AT_R_NUM16, t, r.x + r.w * 0.5f, r.y + 19.0f, AT_C_IVORY, AT_ALIGN_CENTER, 0.0f);
            break; }
        case AT_HP_NOTE: at_part_note(&sink, o, r, p->text, AT_NOTE_INFO, remaining); break;
        case AT_HP_STRIP: at_part_strip(&sink, o, r, &p->strip); break;
        case AT_HP_BANNER: at_part_banner(&sink, o, r, p->btn, p->text, p->progress); break;
        case AT_HP_TOAST: at_part_toast(&sink, o, r, p->rgba != 0 ? p->rgba : AT_C_LINE2, p->text, p->rule, remaining); break;
        case AT_HP_CARD: draw_card(&sink, o, r, p); break;
        default: break;
        }
    }
    if (entries != NULL) *entries = hs.n;
}
