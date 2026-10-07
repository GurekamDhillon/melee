#include "gw_ui_render.h"
#include "gw_ui_stack.h"
#include "gw_ui_tokens.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

/* ---- the entry budget: every draw goes through a counting wrapper around the caller's sink ---- */

typedef struct {
    const AtSink *inner;
    int entries, dropped, warned, capped;
} Counter;

static int glyphs(const char *s)
{
    int n = 0;
    for (; *s; s++) if (((unsigned char) *s & 0xC0) != 0x80) n++;
    return n;
}

static int spend(Counter *c, int cost)
{
    if (c->entries + cost > AT_SCREEN_QUAD_CAP) { c->capped = 1; c->dropped += cost; return 0; }
    c->entries += cost;
    if (c->entries >= AT_SCREEN_QUAD_WARN) c->warned = 1;
    return 1;
}

static void cnt_poly(void *u, const float x[4], const float y[4], unsigned rgba)
{
    Counter *c = (Counter *) u;
    if (spend(c, 1)) c->inner->poly(c->inner->user, x, y, rgba);
}
static void cnt_text(void *u, float x, float base, const char *str, int role, unsigned rgba, int align, float max_w)
{
    Counter *c = (Counter *) u;
    if (spend(c, glyphs(str))) c->inner->text(c->inner->user, x, base, str, role, rgba, align, max_w);
}
static void cnt_model(void *u, int model, int ring, float x, float y, float w, float h, int focused, int dim)
{
    Counter *c = (Counter *) u;
    if (spend(c, AT_MODEL_COST)) c->inner->model(c->inner->user, model, ring, x, y, w, h, focused, dim);
}

/* a sink that draws nothing: measures a part's advance before it is placed */
static void nul_poly(void *u, const float x[4], const float y[4], unsigned rgba) { (void) u; (void) x; (void) y; (void) rgba; }
static void nul_text(void *u, float x, float base, const char *str, int role, unsigned rgba, int align, float max_w)
{ (void) u; (void) x; (void) base; (void) str; (void) role; (void) rgba; (void) align; (void) max_w; }
static void nul_model(void *u, int model, int ring, float x, float y, float w, float h, int focused, int dim)
{ (void) u; (void) model; (void) ring; (void) x; (void) y; (void) w; (void) h; (void) focused; (void) dim; }

/* ---- hits ---- */

typedef struct { AtHits *hits; int dropped; } HitCtx;      /* hits == NULL: no hit rectangles at all (a dialog is open) */

static void hit_add(HitCtx *c, AtRect r, int kind, int a, int b)
{
    AtHits *h = c->hits;
    if (h == NULL) return;
    if (h->n >= AT_MAX_HITS) { c->dropped++; return; }
    h->h[h->n].r = r; h->h[h->n].kind = kind; h->h[h->n].a = a; h->h[h->n].b = b;
    h->n++;
}

static float draw_header(const AtScreen *sc, const AtLayout *L, const AtTextOps *o, const AtSink *s)   /* returns where the trail ends */
{
    const char *items[4];
    float end;
    int n = 0, i;
    for (i = 0; i < sc->n_parents && n < 3; i++) items[n++] = sc->parent[i];
    items[n++] = sc->title;
    end = at_part_trail(s, o, L->trail, items, n);
    if (L->wide) at_part_rail(s, o, L->rail, sc->chapter);
    else at_part_chapter(s, o, L->chapter, sc->chapter);
    at_poly_rect(s, L->rule.x, L->rule.y, L->rule.w, 1.0f, AT_C_LINE);
    return end;
}

/* ---- the grid: blocks stacked, scrolling by whole rows (global row = rows of the blocks above + the row in its block) ---- */

typedef struct { float x0, iw, top, bottom, cell; } GridBox;

static int cols_of(const AtBlock *bk) { return bk->cols > 0 ? bk->cols : 1; }

static int grid_rows(const AtScreen *sc)
{
    int b, n = 0;
    for (b = 0; b < sc->n_blocks; b++) n += (at_block_count(&sc->blocks[b]) + cols_of(&sc->blocks[b]) - 1) / cols_of(&sc->blocks[b]);
    return n;
}

/* One pass from global row `first`. Only a block's title with at least one whole row (or an empty block's title) that fits is
 * placed. Returns the last whole row placed (first - 1 if none). With emit set it draws and records hits. */
static int grid_pass(const AtScreen *sc, const AtView *v, const GridBox *g, const AtTextOps *o, const AtSink *s, HitCtx *hc, int first, int emit)
{
    const float gap = 8.0f;
    float y = g->top, ry0;
    int b, i, row0 = 0, last = first - 1, stop = 0;
    for (b = 0; b < sc->n_blocks && !stop; b++) {
        const AtBlock *bk = &sc->blocks[b];
        int bn = at_block_count(bk), cols = cols_of(bk), rows = (bn + cols - 1) / cols, start = first > row0 ? first - row0 : 0, r;
        float cw = bk->stones ? 34.0f : g->cell, ch = bk->stones ? 38.0f : g->cell;
        if (rows > 0 && start >= rows) { row0 += rows; continue; }       /* wholly scrolled off the top */
        if (y + 26.0f + (rows > 0 ? ch : 0.0f) > g->bottom) break;
        if (emit) {
            float tw = o->width(o->user, AT_R_CAP14, bk->title);
            float cnw = bk->count[0] ? o->width(o->user, AT_R_NUM12, bk->count) : 0.0f;
            at_text(s, o, AT_R_CAP14, bk->title, g->x0, y + 14.0f, AT_C_MUTED, AT_ALIGN_LEFT, g->iw - cnw - 16.0f);
            if (bk->count[0]) at_text(s, o, AT_R_NUM12, bk->count, g->x0 + g->iw, y + 14.0f, AT_C_DIM, AT_ALIGN_RIGHT, 0.0f);
            if (g->x0 + tw + 10.0f < g->x0 + g->iw - cnw - 10.0f) at_poly_rect(s, g->x0 + tw + 10.0f, y + 9.0f, g->iw - tw - cnw - 20.0f, 1.0f, AT_C_LINE);
        }
        y += 26.0f;
        ry0 = y;
        for (r = start; r < rows; r++) {
            if (y + ch > g->bottom) { stop = 1; break; }
            if (emit) for (i = r * cols; i < bn && i < (r + 1) * cols; i++) {
                AtRect rc;
                int st = (v->focus.block == b && v->focus.index == i) ? AT_ST_FOCUS : AT_ST_REST;
                rc.x = g->x0 + (float) (i % cols) * (cw + gap); rc.y = y; rc.w = cw; rc.h = ch;
                if (bk->stones) at_part_stone(s, o, rc, at_block_cell(bk, i), st, v->port_rgba);
                else at_part_cell(s, o, rc, at_block_cell(bk, i), st, v->port_rgba);
                hit_add(hc, rc, AT_HIT_CELL, b, i);
            }
            last = row0 + r;
            y += ch + gap;
        }
        if (emit && bk->stones && bk->note[0] && bn > 0) {                  /* beside the first placed row, after the stones of that row */
            int per = bn < cols ? bn : cols;
            float nx = g->x0 + (float) per * (cw + gap) + 8.0f;
            at_text(s, o, AT_R_BODY14, bk->note, nx, ry0 + 24.0f, AT_C_TEXT2, AT_ALIGN_LEFT, g->x0 + g->iw - nx);
        }
        if (!stop) y += (rows > 0 ? -gap : 0.0f) + 12.0f;
        row0 += rows;
    }
    return last;
}

static void draw_grid(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    GridBox g;
    const float gap = 8.0f;
    int b, maxc = 1, total = grid_rows(sc), first, fr = -1, row0 = 0, it;
    g.x0 = L->primary.x + 12.0f; g.iw = L->primary.w - 24.0f; g.top = L->primary.y + 12.0f;
    g.bottom = L->primary.y + L->primary.h - 12.0f - (sc->footer.has ? 48.0f + 8.0f : 0.0f);
    for (b = 0; b < sc->n_blocks; b++) if (!sc->blocks[b].stones && cols_of(&sc->blocks[b]) > maxc) maxc = cols_of(&sc->blocks[b]);
    g.cell = (float) floor((g.iw - (float) (maxc - 1) * gap) / (float) maxc);
    if (g.cell > 56.0f) g.cell = 56.0f;
    if (g.cell < 24.0f) g.cell = 24.0f;
    for (b = 0; b < sc->n_blocks; b++) {
        if (v->focus.block == b && v->focus.index >= 0) fr = row0 + v->focus.index / cols_of(&sc->blocks[b]);
        row0 += (at_block_count(&sc->blocks[b]) + cols_of(&sc->blocks[b]) - 1) / cols_of(&sc->blocks[b]);
    }
    first = v->scroll < 0 ? 0 : v->scroll;
    if (first >= total) first = total > 0 ? total - 1 : 0;
    for (it = 0; fr >= 0 && it <= total + 2; it++) {                    /* keep the focused cell's row visible */
        int last = grid_pass(sc, v, &g, o, s, hc, first, 0);
        if (fr < first) first = fr;
        else if (fr > last) first = at_list_scroll(fr, first, last - first + 1, total) > first ? at_list_scroll(fr, first, last - first + 1, total) : first + 1;
        else break;
    }
    grid_pass(sc, v, &g, o, s, hc, first, 1);
    if (sc->footer.has) {
        AtRect f;
        f.x = g.x0; f.y = L->primary.y + L->primary.h - 12.0f - 48.0f; f.w = g.iw; f.h = 48.0f;
        at_part_footer(s, o, f, &sc->footer);
    }
}

int at_list_visible(const AtLayout *L) { return (int) floor((L->primary.h - 28.0f) / 39.0f); }

int at_tiles_geometry_ex(const AtScreen *sc, const AtLayout *L, int scroll, AtRect *tiles, int cap, AtRect *more, int more_cap,
                         int *rows_visible, int *rows_total, AtRect *strip_out)
{
    const float gy = 6.0f, gx = 8.0f, pad = 12.0f;
    float x0 = L->primary.x + pad, iw = L->primary.w - 2.0f * pad, y0 = L->primary.y + pad, avail = L->primary.h - 2.0f * pad;
    int cols = at_screen_tile_cols(sc), big = cols == 1, rows = (sc->n_items + cols - 1) / cols, i, vis;
    float th = big ? 56.0f : 62.0f, more_h = sc->n_more > 0 ? 34.0f + gy : 0.0f, tw, bottom;
    if (rows > 0 && (float) rows * th + (float) (rows - 1) * gy + more_h > avail) {
        th = (float) floor((avail - more_h - (float) (rows - 1) * gy) / (float) rows);
        if (th < 44.0f) th = 44.0f;
    }
    vis = th + gy > 0.0f ? (int) floor((avail - more_h + gy) / (th + gy)) : rows;
    if (vis > rows) vis = rows;
    if (vis < 1) vis = 1;
    if (scroll > rows - vis) scroll = rows - vis;
    if (scroll < 0) scroll = 0;
    tw = (iw - (float) (cols - 1) * gx) / (float) cols;
    for (i = 0; i < sc->n_items && i < cap; i++) {
        int r = i / cols - scroll;
        if (r < 0 || r >= vis) { tiles[i].x = x0; tiles[i].y = y0; tiles[i].w = 0.0f; tiles[i].h = 0.0f; continue; }
        tiles[i].x = x0 + (float) (i % cols) * (tw + gx); tiles[i].y = y0 + (float) r * (th + gy); tiles[i].w = tw; tiles[i].h = th;
    }
    bottom = y0 + (float) (vis > 0 ? vis : 0) * (th + gy) - (vis > 0 ? gy : 0.0f);
    if (rows < 1) bottom = y0;
    if (sc->n_more > 0) {
        float sy = bottom + gy, slot = (iw - 70.0f) / (float) sc->n_more;
        if (strip_out != NULL) { strip_out->x = x0; strip_out->y = sy; strip_out->w = iw; strip_out->h = 34.0f; }
        for (i = 0; i < sc->n_more && i < more_cap; i++) {
            more[i].x = x0 + 70.0f + (float) i * slot; more[i].y = sy; more[i].w = slot; more[i].h = 34.0f;
        }
    }
    if (rows_visible != NULL) *rows_visible = vis;
    if (rows_total != NULL) *rows_total = rows;
    return sc->n_items < cap ? sc->n_items : cap;
}

int at_tiles_geometry(const AtScreen *sc, const AtLayout *L, AtRect *tiles, int cap, AtRect *more, int more_cap)
{
    return at_tiles_geometry_ex(sc, L, 0, tiles, cap, more, more_cap, NULL, NULL, NULL);
}

int at_tiles_scroll(const AtScreen *sc, const AtLayout *L, int focus_index, int scroll)
{
    AtRect t[AT_MAX_ITEMS], m[AT_MAX_MORE];
    int vis = 1, rows = 1, cols = at_screen_tile_cols(sc);
    at_tiles_geometry_ex(sc, L, 0, t, AT_MAX_ITEMS, m, AT_MAX_MORE, &vis, &rows, NULL);
    if (focus_index < 0) return scroll < 0 ? 0 : scroll;
    return at_list_scroll(focus_index / cols, scroll, vis, rows);
}

static void draw_tiles(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    AtRect t[AT_MAX_ITEMS], m[AT_MAX_MORE];
    AtRect strip;
    int i, cols = at_screen_tile_cols(sc), vis, rows, first = v->scroll < 0 ? 0 : v->scroll;
    memset(&strip, 0, sizeof strip);
    at_tiles_geometry_ex(sc, L, first, t, AT_MAX_ITEMS, m, AT_MAX_MORE, &vis, &rows, &strip);
    for (i = 0; i < sc->n_items; i++) {
        const AtItem *it = &sc->items[i];
        int st = (it->flags & AT_CELL_DISABLED) ? AT_ST_DISABLED : (v->focus.block == 0 && v->focus.index == i) ? AT_ST_FOCUS : (it->flags & AT_CELL_SELECTED) ? AT_ST_SELECTED : AT_ST_REST;
        if (t[i].h <= 0.0f) continue;
        at_part_tile(s, o, t[i], it, st, cols == 1);
        hit_add(hc, t[i], AT_HIT_CELL, 0, i);
    }
    if (sc->n_more > 0) {
        at_part_more(s, o, (AtRect){ strip.x, strip.y, strip.w, strip.h }, sc->more, sc->n_more, v->focus.block == 1 ? v->focus.index : -1);
        for (i = 0; i < sc->n_more; i++) hit_add(hc, m[i], AT_HIT_CELL, 1, i);
    }
}

static void draw_list(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    float x0 = L->primary.x + 12.0f, y0 = L->primary.y + 14.0f, iw = L->primary.w - 24.0f;
    int visible = at_list_visible(L), first = v->scroll < 0 ? 0 : v->scroll, i;   /* the host keeps v->scroll valid */
    for (i = first; i < sc->n_items && i < first + visible; i++) {
        AtRect r;
        r.x = x0; r.y = y0 + (float) (i - first) * 39.0f; r.w = iw; r.h = 34.0f;
        at_part_row(s, o, r, &sc->items[i], (v->focus.index == i) ? AT_ST_FOCUS : AT_ST_REST);
        hit_add(hc, r, AT_HIT_CELL, 0, i);
    }
}

/* hints are laid out left to right inside the keys place, ahead of the counter; one that does not fit ends the row */
static void draw_keys(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    AtSink nul;
    float x = L->keys.x, base = L->keys.y + 18.0f, adv, limit = L->keys.x + L->keys.w;
    int i;
    nul.user = NULL; nul.poly = nul_poly; nul.text = nul_text; nul.model = nul_model;
    if (v->counter[0]) limit -= o->width(o->user, AT_R_NUM16, v->counter) + 16.0f;
    adv = at_part_hint(&nul, o, x, base, 'M', "Move");
    if (x + adv - 18.0f <= limit) { at_part_hint(s, o, x, base, 'M', "Move"); x += adv; }
    for (i = 0; i < sc->n_keys; i++) {
        AtRect r;
        if (!v->key_shown[i] || v->key_label[i][0] == '\0') continue;
        adv = at_part_hint(&nul, o, x, base, sc->keys[i].btn, v->key_label[i]);
        if (x + adv - 18.0f > limit) break;
        at_part_hint(s, o, x, base, sc->keys[i].btn, v->key_label[i]);
        r.x = x; r.y = base - 16.0f; r.w = adv - 18.0f; r.h = 22.0f;
        hit_add(hc, r, AT_HIT_KEY, sc->keys[i].btn, 0);
        x += adv;
    }
    if (v->counter[0]) at_text(s, o, AT_R_NUM16, v->counter, L->keys.x + L->keys.w, base, AT_C_DIM, AT_ALIGN_RIGHT, 0.0f);
}

void at_render_ex(const AtScreen *sc, const AtView *v, float canvas_w, double now, int reduced,
                  const AtTextOps *o, const AtSink *out, AtHits *hits, AtRenderInfo *info)
{
    AtLayout L;
    AtTween open;
    Counter cn;
    AtSink cs;
    const AtSink *s = &cs;
    HitCtx hc;
    float k, trail_end = 0.0f;
    memset(&cn, 0, sizeof cn);
    cn.inner = out;
    cs.user = &cn; cs.poly = cnt_poly; cs.text = cnt_text; cs.model = cnt_model;
    hits->n = 0;
    hc.hits = v->dialog.open ? NULL : hits;                              /* an open dialog is the only thing the mouse can reach */
    hc.dropped = 0;
    at_layout(canvas_w, sc->preset, &L);
    at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, AT_C_GROUND);
    if (sc->primary == AT_PRIMARY_DISPLAY) {                              /* a display screen: no header, no pane, no hits, no key strip */
        at_part_title(s, o, &L, sc, now, reduced, NULL);
    } else {
        trail_end = draw_header(sc, &L, o, s);
        at_plate(s, L.primary, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
        if (sc->primary == AT_PRIMARY_GRID) draw_grid(sc, v, &L, o, s, &hc);
        else if (sc->primary == AT_PRIMARY_TILES) draw_tiles(sc, v, &L, o, s, &hc);
        else draw_list(sc, v, &L, o, s, &hc);
        if (sc->preset != AT_PRESET_NONE) at_part_explainer(s, o, L.explainer, &v->ex);
        draw_keys(sc, v, &L, o, s, &hc);
    }
    if (v->note.text[0] != '\0' && now < v->note.until_ms) {
        AtRect r;
        float w = o->width(o->user, AT_R_BODY14, v->note.text) + 54.0f, right = L.wide ? L.header.x + L.header.w : L.chapter.x - 12.0f;
        if (w > 300.0f) w = 300.0f;
        r.y = 22.0f;
        if (w > right - (trail_end + 16.0f)) {                         /* it would cover the trail's title: it drops under the rule instead */
            right = L.header.x + L.header.w;
            r.y = L.rule.y + 6.0f;
        }
        r.x = right - w; r.w = w; r.h = 30.0f;
        at_part_note(s, o, r, v->note.text, v->note.kind, v->note.until_ms > v->note.from_ms ? (float) ((v->note.until_ms - now) / (v->note.until_ms - v->note.from_ms)) : 0.0f);
    }
    if (v->dialog.open) {
        AtRect b[2];
        AtTween rise;
        int n, i;
        hc.hits = hits;
        at_tween_start(&rise, v->dialog.from_ms, 140.0, reduced);
        n = at_part_dialog(s, o, L.canvas.w, &v->dialog, 6.0f * (1.0f - at_tween_value(&rise, now)), b);
        for (i = 0; i < n; i++) hit_add(&hc, b[i], AT_HIT_DIALOG, v->dialog.btn[i], 0);
    }
    at_tween_start(&open, v->opened_ms, 100.0, reduced);
    k = 1.0f - at_tween_value(&open, now);                               /* the screen fades in from the ground colour */
    if (k > 0.004f) at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, (AT_C_GROUND & 0xFFFFFF00u) | (unsigned) (k * 255.0f + 0.5f));
    if (info != NULL) {
        info->entries = cn.entries; info->dropped = cn.dropped; info->hits_dropped = hc.dropped;
        info->warned = cn.warned; info->capped = cn.capped;
    }
}

void at_render(const AtScreen *sc, const AtView *v, float canvas_w, double now, int reduced,
               const AtTextOps *o, const AtSink *s, AtHits *hits)
{
    at_render_ex(sc, v, canvas_w, now, reduced, o, s, hits, NULL);
}
