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
    case AT_HP_READOUT: {                                              /* 200 px wide, one column: two fit side by side between the retail timer and the corners at 4:3 */
        AtReadout one = p->data.readout;
        one.cols = 1;
        *w = 200.0f; *h = at_readout_height(&one);
        break; }
    case AT_HP_TRACK: *w = safe_w < 520.0f ? safe_w : 520.0f; *h = at_track_height(); break;
    case AT_HP_CHIPS: *w = safe_w < 400.0f ? safe_w : 400.0f; *h = at_chips_height(); break;
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
    memset(&sink, 0, sizeof sink);                                  /* a HUD never draws disc art: the image op stays NULL */
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
        case AT_HP_NOTE: at_part_note(&sink, o, r, p->text, p->tone, remaining); break;
        case AT_HP_READOUT: { AtReadout one = p->data.readout; one.cols = 1; at_part_readout(&sink, o, r, &one); break; }
        case AT_HP_TRACK: at_part_track(&sink, o, r, &p->data.track); break;
        case AT_HP_CHIPS: at_part_chips(&sink, o, r, &p->data.chips); break;
        case AT_HP_STRIP: at_part_strip(&sink, o, r, &p->strip); break;
        case AT_HP_BANNER: at_part_banner(&sink, o, r, p->btn, p->text, p->progress); break;
        case AT_HP_TOAST: at_part_toast(&sink, o, r, p->rgba != 0 ? p->rgba : AT_C_LINE2, p->text, p->rule, remaining); break;
        case AT_HP_CARD: draw_card(&sink, o, r, p); break;
        default: break;
        }
    }
    if (entries != NULL) *entries = hs.n;
}

/* ---- the Lua description -> the record (pure: the caller has already copied the table into a value tree) ---- */
#define HFAIL(...) do { snprintf(err, (size_t) errcap, __VA_ARGS__); return 0; } while (0)

static const char *const ZONE_NAMES[AT_Z_COUNT] = { "top_left", "top_center", "top_right", "bottom_left", "bottom_center", "bottom_right" };
const char *at_hud_zone_name(int z) { return (z >= 0 && z < AT_Z_COUNT) ? ZONE_NAMES[z] : NULL; }
int at_hud_zone_by_name(const char *name)
{
    int z;
    for (z = 0; z < AT_Z_COUNT; z++) if (name != NULL && strcmp(name, ZONE_NAMES[z]) == 0) return z;
    return -1;
}

static unsigned colour_of(const AtvArena *a, int n)
{
    double d = atv_numv(a, n, 0.0);
    if (d != d || d < 0.0 || d > 4294967295.0) return 0u;
    return (unsigned) d;
}
static void copy_str(const AtvArena *a, int t, const char *k, char *dst, int cap)
{
    int n = atv_get(a, t, k);
    if (atv_kind(a, n) == ATV_STR) snprintf(dst, (size_t) cap, "%s", atv_strv(a, n, ""));
}
static double clamp_secs(double s) { return s != s ? 4.0 : (s < 0.5 ? 0.5 : (s > 15.0 ? 15.0 : s)); }

int at_hud_from_val(const AtvArena *a, int root, const char *owner_mod, double now_ms, AtHud *out, char *err, int errcap)
{
    const char *id;
    int zones, z, i, k;
    memset(out, 0, sizeof *out);
    if (a->overflow) HFAIL("gd.ui.hud: the description is too large");
    if (atv_kind(a, root) != ATV_TABLE) HFAIL("gd.ui.hud: a HUD description is a table");
    id = atv_strv(a, atv_get(a, root, "id"), "");
    if (id[0] == '\0') HFAIL("gd.ui.hud: the description has no id");
    if (owner_mod != NULL && owner_mod[0] != '\0') {
        size_t ol = strlen(owner_mod);
        if (strncmp(id, owner_mod, ol) != 0 || id[ol] != '.') HFAIL("gd.ui.hud: id \"%s\" must start with \"%s.\"", id, owner_mod);
    }
    if (strlen(id) >= sizeof out->id) HFAIL("gd.ui.hud: id \"%s\" is too long (%d characters at most)", id, (int) sizeof out->id - 1);
    snprintf(out->id, sizeof out->id, "%s", id);
    zones = atv_get(a, root, "zones");
    if (zones >= 0 && atv_kind(a, zones) != ATV_TABLE) HFAIL("gd.ui.hud: zones is a table of zone names to part lists");
    for (z = 0; z < AT_Z_COUNT && zones >= 0; z++) {
        int list = atv_get(a, zones, ZONE_NAMES[z]), n;
        if (list < 0) continue;
        if (atv_kind(a, list) != ATV_TABLE) HFAIL("gd.ui.hud: zone %s is a list of parts", ZONE_NAMES[z]);
        n = atv_len(a, list);
        if (n > AT_HUD_PER_ZONE) HFAIL("gd.ui.hud: zone %s holds at most %d parts (%d given)", ZONE_NAMES[z], AT_HUD_PER_ZONE, n);
        for (i = 0; i < n; i++) {
            int pt = atv_at(a, list, i + 1), t, kn;
            const char *kind;
            AtHudPart *p = &out->z[z][i];
            if (atv_kind(a, pt) != ATV_TABLE) HFAIL("gd.ui.hud: part %d of %s is not a table", i + 1, ZONE_NAMES[z]);
            kind = atv_strv(a, atv_get(a, pt, "kind"), "");
            p->progress = -1.0f;
            if (strcmp(kind, "strip") == 0) {
                int pips = atv_get(a, pt, "pips"), keys = atv_get(a, pt, "keys");
                p->kind = AT_HP_STRIP;
                kn = atv_len(a, pips);
                if (kn > 8) HFAIL("gd.ui.hud: a strip shows at most 8 slot pips (%d given)", kn);
                for (k = 0; k < kn; k++) { t = atv_at(a, pips, k + 1); p->strip.pip_fill[k] = colour_of(a, atv_get(a, t, "fill")); p->strip.pip_ring[k] = colour_of(a, atv_get(a, t, "ring")); }
                p->strip.n_pips = kn;
                kn = atv_len(a, keys);
                if (kn > 8) HFAIL("gd.ui.hud: a strip shows at most 8 keystones (%d given)", kn);
                for (k = 0; k < kn; k++) {
                    const char *l;
                    t = atv_at(a, keys, k + 1);
                    l = atv_strv(a, atv_get(a, t, "letter"), "");
                    p->strip.key_letter[k] = l[0];
                    p->strip.key_rgba[k] = colour_of(a, atv_get(a, t, "rgba"));
                }
                p->strip.n_keys = kn;
                copy_str(a, pt, "wait", p->strip.wait, (int) sizeof p->strip.wait);
            } else if (strcmp(kind, "banner") == 0) {
                const char *b = atv_strv(a, atv_get(a, pt, "button"), "A");
                p->kind = AT_HP_BANNER;
                copy_str(a, pt, "text", p->text, AT_STR);
                p->btn = (b[0] != '\0' && strchr("ABXYZLR", b[0]) != NULL) ? b[0] : 'A';
                p->progress = (float) atv_numv(a, atv_get(a, pt, "progress"), -1.0);
                if (p->progress != p->progress) p->progress = -1.0f;
            } else if (strcmp(kind, "card") == 0) {
                int lines = atv_get(a, pt, "lines");
                p->kind = AT_HP_CARD;
                copy_str(a, pt, "title", p->text, AT_STR);
                p->rgba = colour_of(a, atv_get(a, pt, "rgba"));
                kn = atv_len(a, lines);
                if (kn > 3) HFAIL("gd.ui.hud: a card shows at most 3 lines (%d given)", kn);
                for (k = 0; k < kn; k++) snprintf(p->lines[k], AT_STR, "%s", atv_strv(a, atv_at(a, lines, k + 1), ""));
                p->n_lines = kn;
            } else if (strcmp(kind, "note") == 0) {
                const char *tn = atv_strv(a, atv_get(a, pt, "tone"), "info");
                p->kind = AT_HP_NOTE;
                p->tone = strcmp(tn, "ok") == 0 ? AT_NOTE_OK : strcmp(tn, "warn") == 0 ? AT_NOTE_WARN : strcmp(tn, "err") == 0 ? AT_NOTE_ERR : AT_NOTE_INFO;
                copy_str(a, pt, "text", p->text, AT_STR);
                p->from_ms = now_ms;
                p->until_ms = now_ms + 1000.0 * clamp_secs(atv_numv(a, atv_get(a, pt, "seconds"), 4.0));
            } else if (strcmp(kind, "port_card") == 0) {
                double port = atv_numv(a, atv_get(a, pt, "port"), 0.0);
                p->kind = AT_HP_PORT_CARD;
                if (!(port >= 1.0 && port <= 4.0)) HFAIL("gd.ui.hud: a port_card needs port 1 to 4");
                p->port.port = (int) port;
            } else if (strcmp(kind, "timer") == 0) {
                double s = atv_numv(a, atv_get(a, pt, "seconds"), 0.0);
                p->kind = AT_HP_TIMER;
                p->seconds = (s != s || s < 0.0) ? 0 : (s > 5999.0 ? 5999 : (int) s);
            } else if (strcmp(kind, "readout") == 0) {
                p->kind = AT_HP_READOUT;
                if (!at_readout_from_val(a, pt, &p->data.readout, err, errcap)) return 0;
            } else if (strcmp(kind, "track") == 0) {
                p->kind = AT_HP_TRACK;
                if (!at_track_from_val(a, pt, &p->data.track, err, errcap)) return 0;
            } else if (strcmp(kind, "chips") == 0) {
                p->kind = AT_HP_CHIPS;
                if (!at_chips_from_val(a, pt, &p->data.chips, err, errcap)) return 0;
            } else {
                HFAIL("gd.ui.hud: part %d of %s has the unknown kind \"%s\" (strip, banner, card, note, port_card, timer, readout, track, chips)", i + 1, ZONE_NAMES[z], kind);
            }
        }
        out->n[z] = n;
    }
    {
        char why[96];
        if (!at_hud_cap_ok(out, why, sizeof why)) HFAIL("gd.ui.hud: %s", why);
    }
    return 1;
}
