#include "gw_ui_render.h"
#include "gw_ui_css.h"
#include "gw_ui_frame.h"
#include "gw_ui_room.h"
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
static void cnt_image(void *u, int tex, float x, float y, float w, float h, unsigned rgba)
{
    Counter *c = (Counter *) u;
    if (spend(c, 1)) at_sink_image(c->inner, tex, x, y, w, h, rgba);
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
    float end, cd_w = 0.0f;
    AtRect tr = L->trail;
    int n = 0, i;
    char cd[16];
    for (i = 0; i < sc->n_parents && n < 3; i++) items[n++] = sc->parent[i];
    items[n++] = sc->title;
    if (sc->has_countdown) {                                           /* 0:45 at the trail's right end; rose under 10 s; the trail gives way to it */
        snprintf(cd, sizeof cd, "%d:%02d", sc->countdown / 60, sc->countdown % 60);
        cd_w = o->width(o->user, AT_R_NUM14, cd);
        tr.w -= cd_w + 20.0f;
        if (tr.w < 120.0f) tr.w = 120.0f;
        at_text(s, o, AT_R_NUM14, cd, L->trail.x + L->trail.w, L->trail.y + L->trail.h * 0.5f + 5.0f, sc->countdown < 10 ? AT_C_ROSE : AT_C_TEXT2, AT_ALIGN_RIGHT, 0.0f);
    }
    end = at_part_trail(s, o, tr, items, n);
    if (L->wide) at_part_rail(s, o, L->rail, sc->chapter);
    else at_part_chapter(s, o, L->chapter, sc->chapter);
    at_poly_rect(s, L->rule.x, L->rule.y, L->rule.w, 1.0f, AT_C_LINE);
    return end;
}

/* ---- the grid: blocks stacked, scrolling by whole rows (global row = rows of the blocks above + the row in its block) ---- */

typedef struct { float x0, iw, top, bottom, cell; int acols; AtRect (*rects)[AT_MAX_CELLS]; unsigned char (*have)[AT_MAX_CELLS]; } GridBox;   /* acols > 0: the renderer chose the columns (grid_cols_auto) */

static int cols_in(const GridBox *g, const AtBlock *bk) { return g->acols > 0 ? g->acols : (bk->cols > 0 ? bk->cols : 1); }

/* a native block with no title takes no title row (the character select has tabs instead); every other block keeps its 26 px row */
static float title_h(const AtBlock *bk) { return (bk->ext != NULL && bk->title[0] == '\0') ? 0.0f : 26.0f; }

static int grid_rows(const AtScreen *sc, const GridBox *g)
{
    int b, n = 0;
    for (b = 0; b < sc->n_blocks; b++) n += (at_block_count(&sc->blocks[b]) + cols_in(g, &sc->blocks[b]) - 1) / cols_in(g, &sc->blocks[b]);
    return n;
}

/* the cursors on a grid cell: the ports whose cursor is on it, in port order. Fills slot_port[] and returns how many. */
static int cursors_on(const AtView *v, int block, int index, int *slot_port)
{
    int p, n = 0;
    for (p = 0; p < AT_MAX_CURSORS; p++)
        if (v->cursor[p].active && v->cursor[p].card < 0 && v->cursor[p].block == block && v->cursor[p].index == index) slot_port[n++] = p;
    return n;
}

static unsigned port_rgba(int port) { return port == 0 ? AT_C_P1 : port == 1 ? AT_C_P2 : port == 2 ? AT_C_P3 : AT_C_P4; }

/* One pass from global row `first`. Only a block's title with at least one whole row (or an empty block's title) that fits is
 * placed. Returns the last whole row placed (first - 1 if none). With emit set it draws and records hits. */
static int grid_pass(const AtScreen *sc, const AtView *v, const GridBox *g, const AtTextOps *o, const AtSink *s, HitCtx *hc, int first, int emit)
{
    const float gap = 8.0f;
    float y = g->top, ry0;
    int b, i, row0 = 0, last = first - 1, stop = 0;
    for (b = 0; b < sc->n_blocks && !stop; b++) {
        const AtBlock *bk = &sc->blocks[b];
        int bn = at_block_count(bk), cols = cols_in(g, bk), rows = (bn + cols - 1) / cols, start = first > row0 ? first - row0 : 0, r;
        float cw = bk->stones ? 34.0f : g->cell, ch = bk->stones ? 38.0f : g->cell, th = title_h(bk);
        if (rows > 0 && start >= rows) { row0 += rows; continue; }       /* wholly scrolled off the top */
        if (y + th + (rows > 0 ? ch : 0.0f) > g->bottom) break;
        if (emit && th > 0.0f) {
            float tw = o->width(o->user, AT_R_CAP14, bk->title);
            float cnw = bk->count[0] ? o->width(o->user, AT_R_NUM12, bk->count) : 0.0f;
            at_text(s, o, AT_R_CAP14, bk->title, g->x0, y + 14.0f, AT_C_MUTED, AT_ALIGN_LEFT, g->iw - cnw - 16.0f);
            if (bk->count[0]) at_text(s, o, AT_R_NUM12, bk->count, g->x0 + g->iw, y + 14.0f, AT_C_DIM, AT_ALIGN_RIGHT, 0.0f);
            if (g->x0 + tw + 10.0f < g->x0 + g->iw - cnw - 10.0f) at_poly_rect(s, g->x0 + tw + 10.0f, y + 9.0f, g->iw - tw - cnw - 20.0f, 1.0f, AT_C_LINE);
        }
        y += th;
        ry0 = y;
        for (r = start; r < rows; r++) {
            if (y + ch > g->bottom) { stop = 1; break; }
            for (i = r * cols; i < bn && i < (r + 1) * cols; i++) {
                AtRect rc;
                int ports[AT_MAX_CURSORS], nc = cursors_on(v, b, i, ports);
                int st = ((v->focus.block == b && v->focus.index == i) || nc > 0) ? AT_ST_FOCUS : AT_ST_REST;
                rc.x = g->x0 + (float) (i % cols) * (cw + gap); rc.y = y; rc.w = cw; rc.h = ch;
                if (g->rects != NULL && b < AT_MAX_BLOCKS && i < AT_MAX_CELLS) { g->rects[b][i] = rc; g->have[b][i] = 1; }
                if (!emit) continue;
                if (bk->stones) at_part_stone(s, o, rc, at_block_cell(bk, i), st, v->port_rgba);
                else if (nc <= 1) at_part_cell(s, o, rc, at_block_cell(bk, i), st, nc == 1 ? port_rgba(ports[0]) : v->port_rgba);
                else {                                                    /* several ports on one tile: the cues once, one set of brackets each */
                    int k;
                    at_part_cell_ex(s, o, rc, at_block_cell(bk, i), st, 0, 0);
                    for (k = 0; k < nc; k++) at_cell_brackets(s, rc, port_rgba(ports[k]), k, nc);
                }
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
    int b, maxc = 1, total, first, fr = -1, row0 = 0, it, p;
    float cmin, cmax;
    AtRect rects[AT_MAX_BLOCKS][AT_MAX_CELLS];
    unsigned char have[AT_MAX_BLOCKS][AT_MAX_CELLS];
    g.rects = NULL; g.have = NULL;
    g.x0 = L->primary.x + 12.0f; g.iw = L->primary.w - 24.0f; g.top = L->primary.y + 12.0f;
    g.bottom = L->primary.y + L->primary.h - 12.0f - (sc->footer.has ? 48.0f + 8.0f : 0.0f);
    cmin = sc->grid_cell_min > 0 ? (float) sc->grid_cell_min : 36.0f; cmax = sc->grid_cell_max > 0 ? (float) sc->grid_cell_max : 56.0f;
    if (cmax < cmin) cmax = cmin;
    g.acols = sc->grid_cols_auto ? at_grid_cols(g.iw, cmin, cmax, gap) : 0;
    for (b = 0; b < sc->n_blocks; b++) if (!sc->blocks[b].stones && cols_in(&g, &sc->blocks[b]) > maxc) maxc = cols_in(&g, &sc->blocks[b]);
    g.cell = (float) floor((g.iw - (float) (maxc - 1) * gap) / (float) maxc);
    if (g.cell > cmax) g.cell = cmax;
    if (g.cell < 24.0f) g.cell = 24.0f;
    total = grid_rows(sc, &g);
    for (p = -1; p < AT_MAX_CURSORS && fr < 0; p++) {                     /* the legacy focus first, then the lowest port's cursor on the grid */
        int cb = p < 0 ? v->focus.block : (v->cursor[p].active && v->cursor[p].card < 0 ? v->cursor[p].block : -1), ci = p < 0 ? v->focus.index : v->cursor[p].index;
        row0 = 0;
        for (b = 0; b < sc->n_blocks; b++) {
            if (cb == b && ci >= 0) { fr = row0 + ci / cols_in(&g, &sc->blocks[b]); break; }
            row0 += (at_block_count(&sc->blocks[b]) + cols_in(&g, &sc->blocks[b]) - 1) / cols_in(&g, &sc->blocks[b]);
        }
    }
    first = v->scroll < 0 ? 0 : v->scroll;
    if (first >= total) first = total > 0 ? total - 1 : 0;
    for (it = 0; fr >= 0 && it <= total + 2; it++) {                    /* keep the focused cell's row visible */
        int last = grid_pass(sc, v, &g, o, s, hc, first, 0);
        if (fr < first) first = fr;
        else if (fr > last) first = at_list_scroll(fr, first, last - first + 1, total) > first ? at_list_scroll(fr, first, last - first + 1, total) : first + 1;
        else break;
    }
    if (sc->n_links > 0) {                                              /* links go under the cells: record where the cells land, draw the lines, then the cells */
        int li, bb, cc;
        memset(have, 0, sizeof have);
        g.rects = rects; g.have = have;
        grid_pass(sc, v, &g, o, s, hc, first, 0);
        g.rects = NULL; g.have = NULL;
        for (li = 0; li < sc->n_links; li++) {
            int ab = -1, ai = -1, bbk = -1, bi = -1;
            for (bb = 0; bb < sc->n_blocks; bb++) for (cc = 0; cc < at_block_count(&sc->blocks[bb]) && cc < AT_MAX_CELLS; cc++) {
                if (strcmp(at_block_cell(&sc->blocks[bb], cc)->id, sc->links[li].a) == 0) { ab = bb; ai = cc; }
                if (strcmp(at_block_cell(&sc->blocks[bb], cc)->id, sc->links[li].b) == 0) { bbk = bb; bi = cc; }
            }
            if (ab < 0 || bbk < 0 || ab >= AT_MAX_BLOCKS || bbk >= AT_MAX_BLOCKS || !have[ab][ai] || !have[bbk][bi]) continue;   /* a cell scrolled out: no line */
            at_part_link(s, rects[ab][ai].x + rects[ab][ai].w * 0.5f, rects[ab][ai].y + rects[ab][ai].h * 0.5f,
                         rects[bbk][bi].x + rects[bbk][bi].w * 0.5f, rects[bbk][bi].y + rects[bbk][bi].h * 0.5f, 2.0f, sc->links[li].rgba);
        }
    }
    grid_pass(sc, v, &g, o, s, hc, first, 1);
    if (sc->footer.has) {
        AtRect f;
        f.x = g.x0; f.y = L->primary.y + L->primary.h - 12.0f - 48.0f; f.w = g.iw; f.h = 48.0f;
        at_part_footer(s, o, f, &sc->footer);
    }
}

/* The strip over the pane: the tab names with their counts, and one hit rectangle per tab (a = the tab index). */
static void draw_tabs(const AtScreen *sc, const AtView *v, AtRect r, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    const char *names[AT_MAX_TABS];
    int counts[AT_MAX_TABS], i, n = sc->n_tabs > AT_MAX_TABS ? AT_MAX_TABS : sc->n_tabs, with_counts = 1;
    AtRect rects[AT_MAX_TABS];
    for (i = 0; i < n; i++) { names[i] = sc->tabs[i].name; counts[i] = sc->tabs[i].count; if (counts[i] < 0) with_counts = 0; }   /* a count below zero: tabs with no numbers (settings) */
    at_part_tabs_ex(s, o, r, names, with_counts ? counts : NULL, n, v->tab, -1, rects);
    for (i = 0; i < n; i++) if (rects[i].w > 0.0f) hit_add(hc, rects[i], AT_HIT_TAB, i, 0);
}

/* The loading screen's pane: the stage (kicker and name from the view's explainer) and the warm-up bar: a track, a fill that follows view.progress (0 to 1000, clamped),
 * the state word (view.counter: WARMING UP or READY) and the percentage. Nothing here is a hit: there is nothing to click on while the game loads. */
static void draw_loading(const AtView *v, AtRect pane, const AtTextOps *o, const AtSink *s)
{
    float cx = pane.x + pane.w * 0.5f, tw = pane.w - 48.0f, bw, bx, by, ty;
    int pm = v->progress < 0 ? 0 : (v->progress > 1000 ? 1000 : v->progress);
    char pct[8];
    if (tw < 40.0f) return;
    ty = pane.y + pane.h * 0.36f;
    if (v->ex.has) {
        at_text(s, o, AT_R_CAP14, v->ex.kicker, cx, ty, AT_C_JADE, AT_ALIGN_CENTER, tw);
        at_text(s, o, AT_R_TITLE, v->ex.title, cx, ty + 40.0f, AT_C_IVORY, AT_ALIGN_CENTER, tw);
    }
    bw = tw < 360.0f ? tw : 360.0f;
    bx = cx - bw * 0.5f;
    by = pane.y + pane.h * 0.64f;
    at_poly_rect(s, bx, by, bw, 14.0f, AT_C_GROUND2);
    if (pm > 0) at_poly_rect(s, bx, by, bw * (float) pm / 1000.0f, 14.0f, AT_C_EMBER);
    at_text(s, o, AT_R_CAP12, v->counter, bx, by - 8.0f, pm >= 1000 ? AT_C_IVORY : AT_C_MUTED, AT_ALIGN_LEFT, bw - 56.0f);
    snprintf(pct, sizeof pct, "%d%%", pm / 10);
    at_text(s, o, AT_R_NUM14, pct, bx + bw, by - 8.0f, AT_C_DIM, AT_ALIGN_RIGHT, 0.0f);
}

/* The band: four port cards in equal columns, or the matchup strip. A card is focused when a port's cursor is on it (card = its index);
 * the brackets take that port's colour. A card's hit rectangle is its slot. */
static void draw_band(const AtScreen *sc, const AtView *v, AtRect r, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    if (sc->band == AT_BAND_CARDS) {
        const float gap = 8.0f;
        float w = (r.w - 3.0f * gap) / 4.0f;
        int i, p;
        for (i = 0; i < 4; i++) {
            AtRect rc;
            int focus = 0;
            rc.x = r.x + (float) i * (w + gap); rc.y = r.y; rc.w = w; rc.h = r.h;
            for (p = 0; p < AT_MAX_CURSORS && focus == 0; p++) if (v->cursor[p].active && v->cursor[p].card == i) focus = 1 + p;
            at_part_sel_card(s, o, rc, &sc->ports[i], focus);
            hit_add(hc, rc, AT_HIT_CARD, i, 0);
        }
    } else if (sc->band == AT_BAND_MATCHUP) {
        AtSelCard shown[4];
        int i, n = 0, picker = -1;
        for (i = 0; i < 4; i++) if (sc->ports[i].kind != 0) { if (sc->ports[i].flags & AT_CARD_FOCUS) picker = n; shown[n++] = sc->ports[i]; }
        at_part_matchup(s, o, r, shown, n, picker);
    }
}

/* The one place the cards row's geometry lives: n cards share the primary's width (200 px at most each, 12 px apart), as tall as the
 * primary leaves (at most 280, never under the smallest card that keeps the 12 px floor), centred. Returns the card count. */
int at_cards_geometry(const AtScreen *sc, const AtLayout *L, AtRect *out, int cap)
{
    int n = sc->n_cards < cap ? sc->n_cards : cap, i;
    float iw = L->primary.w - 24.0f, w, h, x0, y0;
    if (n < 1) return 0;
    w = (iw - 12.0f * (float) (n - 1)) / (float) n;
    if (w > 200.0f) w = 200.0f;
    h = L->primary.h - 24.0f;
    if (h > 280.0f) h = 280.0f;
    if (h < at_part_offer_min_h()) h = at_part_offer_min_h();
    x0 = L->primary.x + 12.0f + (iw - ((float) n * w + 12.0f * (float) (n - 1))) * 0.5f;
    y0 = L->primary.y + (L->primary.h - h) * 0.5f;
    for (i = 0; i < n; i++) { out[i].x = x0 + (float) i * (w + 12.0f); out[i].y = y0; out[i].w = w; out[i].h = h; }
    return n;
}

static void draw_cards(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    AtRect r[AT_MAX_CARDS];
    int n = at_cards_geometry(sc, L, r, AT_MAX_CARDS), i;
    for (i = 0; i < n; i++) {
        int st = (v->focus.block == 0 && v->focus.index == i) ? AT_ST_FOCUS : (sc->cards[i].disabled ? AT_ST_DISABLED : AT_ST_REST);
        at_part_offer(s, o, r[i], &sc->cards[i].offer, st, v->port_rgba);
        hit_add(hc, r[i], AT_HIT_CELL, 0, i);
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

/* the credit line: small, quiet, never wider than the pane (at_fit steps down a role, then cuts with an ellipsis: a last resort) */
static void fit_text_credit(const AtSink *s, const AtTextOps *o, const char *text, float x, float base, float max_w)
{
    char fit[AT_STR * 2];
    int role = at_fit(o, AT_R_CAP12, text, max_w, fit, (int) sizeof fit);
    at_text(s, o, role, fit, x, base, AT_C_MUTED, AT_ALIGN_LEFT, 0.0f);
}

/* A heading precedes row i when it names a group the row before it does not (the first row too). Headings are not rows: no focus, no hit rectangle. */
/* a row's height and the step to the next: 34 and 39 for a plain row, row_h and row_h + 5 for a rich one */
static float item_height(const AtItem *it) { return it->row_h > 34 ? (float) it->row_h : 34.0f; }
float at_item_pitch(const AtItem *it) { return item_height(it) + 5.0f; }

static int heading_before(const AtScreen *sc, int i)
{
    return sc->items[i].group[0] != '\0' && (i == 0 || strcmp(sc->items[i].group, sc->items[i - 1].group) != 0);
}

/* The rows of a list that fit a pane of height pane_h starting at `first`: a row costs 39 px (34 and a 5 px gap), a heading 22 px, with 14 px
 * above and below. With no groups this is exactly floor((pane_h - 28) / 39), the legacy at_list_visible. */
int at_list_window(const AtScreen *sc, float pane_h, int first)
{
    float avail = pane_h - 28.0f, used = 0.0f;
    int i, n = 0;
    if (first < 0) first = 0;
    for (i = first; i < sc->n_items; i++) {
        float cost = at_item_pitch(&sc->items[i]) + (heading_before(sc, i) ? 22.0f : 0.0f);
        if (used + cost > avail + 0.001f) break;
        used += cost;
        n++;
    }
    return n;
}

/* the first visible row that keeps `focus` in the window (never moves the window when the focus is already in it) */
int at_list_scroll_to(const AtScreen *sc, float pane_h, int focus, int scroll)
{
    if (sc->n_items < 1) return 0;
    if (focus < 0) focus = 0;
    if (focus >= sc->n_items) focus = sc->n_items - 1;
    if (scroll < 0) scroll = 0;
    if (scroll > focus) scroll = focus;
    while (scroll < focus && focus >= scroll + at_list_window(sc, pane_h, scroll)) scroll++;
    return scroll;
}

static void draw_list(const AtScreen *sc, const AtView *v, AtRect pane, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    float x0 = pane.x + 12.0f, y = pane.y + 14.0f, iw = pane.w - 24.0f;
    int first = v->scroll < 0 ? 0 : v->scroll, count = at_list_window(sc, pane.h, first), i;   /* the host keeps v->scroll valid */
    for (i = first; i < sc->n_items && i < first + count; i++) {
        AtRect r;
        if (heading_before(sc, i)) {
            float tw = o->width(o->user, AT_R_CAP12, sc->items[i].group);
            at_text(s, o, AT_R_CAP12, sc->items[i].group, x0 + 2.0f, y + 14.0f, AT_C_MUTED, AT_ALIGN_LEFT, iw * 0.5f);
            if (iw * 0.5f > tw + 16.0f) at_poly_rect(s, x0 + tw + 14.0f, y + 9.0f, iw - tw - 14.0f, 1.0f, AT_C_LINE);
            y += 22.0f;
        }
        r.x = x0; r.y = y; r.w = iw; r.h = item_height(&sc->items[i]);
        at_part_row(s, o, r, &sc->items[i], (v->focus.index == i) ? AT_ST_FOCUS : AT_ST_REST);
        hit_add(hc, r, AT_HIT_CELL, 0, i);
        y += at_item_pitch(&sc->items[i]);
    }
}

/* hints are laid out left to right inside the keys place, ahead of the counter; one that does not fit ends the row */
static void draw_keys(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    AtSink nul;
    float x = L->keys.x, base = L->keys.y + 18.0f, adv, limit = L->keys.x + L->keys.w;
    int i;
    memset(&nul, 0, sizeof nul);
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

/* A framed screen (gw_ui_frame.h): opaque plates in the ground colour around a window onto the retail scene, a thin line just outside the window's
 * edge, and the chrome on the plates. NOTHING is drawn inside the window: the retail 3D shows through it. No focus and no hit rectangles: retail owns
 * the pad and the mouse is not supported here. A chrome slot that would sit on the window is dropped (the counter then rides in the explainer's
 * kicker). There is no open fade: the retail scene fades in by itself, and a cut keeps Reduced Motion exact. */
static void render_frame(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s)
{
    AtFrameRect win;
    AtFrameSlots sl;
    AtRect pl[4], hole;
    HitCtx hc;
    int n, i;
    unsigned edge = sc->frame_outline ? AT_C_EMBER : AT_C_LINE2;   /* MELEE_ATLAS_FRAME_OUTLINE: the edge in ember, to compare with the retail model by looking */
    win.x = sc->has_frame ? sc->frame_x : 0.0f; win.y = sc->has_frame ? sc->frame_y : 0.0f;
    win.w = sc->has_frame ? sc->frame_w : 0.0f; win.h = sc->has_frame ? sc->frame_h : 0.0f;
    hole = at_frame_hole(L, win);
    n = at_frame_plates(L, win, pl);
    at_frame_slots(L, win, sc->preset >= AT_PRESET_NARROW && sc->preset <= AT_PRESET_WIDE ? L->explainer.w : 0.0f, &sl);
    for (i = 0; i < n; i++) at_poly_rect(s, pl[i].x, pl[i].y, pl[i].w, pl[i].h, AT_C_GROUND);
    if (hole.w > 0.5f && hole.h > 0.5f) {                               /* the window's edge: up to four lines just OUTSIDE it, where a plate is there */
        float lx = hole.x >= 2.0f ? hole.x - 2.0f : hole.x, rx = hole.x + hole.w + 2.0f <= L->canvas.w ? hole.x + hole.w + 2.0f : hole.x + hole.w;
        if (hole.y >= 2.0f) at_poly_rect(s, lx, hole.y - 2.0f, rx - lx, 2.0f, edge);
        if (hole.y + hole.h + 2.0f <= 480.0f) at_poly_rect(s, lx, hole.y + hole.h, rx - lx, 2.0f, edge);
        if (hole.x >= 2.0f) at_poly_rect(s, hole.x - 2.0f, hole.y, 2.0f, hole.h, edge);
        if (hole.x + hole.w + 2.0f <= L->canvas.w) at_poly_rect(s, hole.x + hole.w, hole.y, 2.0f, hole.h, edge);
    }
    if (sl.have_trail) {
        const char *items[4];
        int ni = 0;
        for (i = 0; i < sc->n_parents && ni < 3; i++) items[ni++] = sc->parent[i];
        items[ni++] = sc->title;
        at_part_trail(s, o, sl.trail, items, ni);
        if (L->rule.y + 1.0f <= hole.y || hole.w <= 0.5f) at_poly_rect(s, L->rule.x, L->rule.y, L->rule.w, 1.0f, AT_C_LINE);
    }
    if (sc->preset != AT_PRESET_NONE && sl.have_explainer) {
        AtExplainer ex = v->ex;
        if (!sl.have_keys && v->counter[0] != '\0') snprintf(ex.kicker, sizeof ex.kicker, "%s", v->counter);   /* no key strip to carry it */
        ex.no_well = 1;
        at_part_explainer(s, o, sl.explainer, &ex);
    }
    if (sl.have_keys) {
        hc.hits = NULL; hc.dropped = 0;                                  /* key hints are information, never clickable */
        draw_keys(sc, v, L, o, s, &hc);
    }
}


/* ---- the character select, Melee style (AT_PRIMARY_CSS) -------------------------------------------------------------------------
 * No framed pane and no explainer: a flush field of icons (the disc's own icon art, as they come, scaled so the whole roster fits when it can and
 * scrolling by rows when it cannot), a one-line caption under it, and a panel for each port across the bottom: a coloured nameplate with the
 * HMN / CPU / NA toggle, the portrait, the puck's dock, the CPU level slider, the costume stepper. A hand (a bracket and a pointer in the port's
 * colour) moves over the field or the panels and carries a puck (a coin with the port's number); a set-down puck lies on its tile. The rules are the
 * model's (gw_ui_css.c); this only draws what the adapter submitted and says where the mouse can reach (AT_HIT_CELL for a tile, AT_HIT_ZONE for a
 * panel part). The geometry is one function so the draw, the hits and the tests agree. */

#define CSV_PANEL_H 168.0f
#define CSV_CAPTION_H 22.0f
#define CSV_GAP 3.0f
#define AT_CSS_CPU_COIN 99           /* not a port: the CPU colour (unused by the screen today) */

static int csv_nports(const AtScreen *sc) { return sc->n_ports < 2 ? 4 : (sc->n_ports > AT_MAX_PORTS ? AT_MAX_PORTS : sc->n_ports); }

/* the lowest port whose hand is on the field (card < 0), or -1 */
static int csv_field_hand(const AtView *v)
{
    int p;
    for (p = 0; p < AT_MAX_CURSORS; p++) if (v->cursor[p].active && v->cursor[p].card < 0) return p;
    return -1;
}

static void csv_panel(AtCssPanel *q, AtRect box)
{
    const float pad = 6.0f;
    float w = box.w, py, ph, pw, nw, lw, bw;
    int i;
    memset(q, 0, sizeof *q);
    q->box = box;
    q->strip = (AtRect){ box.x + pad, box.y + 7.0f, w - 2.0f * pad, 18.0f };
    q->coin = (AtRect){ box.x + pad, box.y + 7.0f, 18.0f, 18.0f };
    py = box.y + 30.0f;
    ph = box.h - 30.0f - 66.0f;
    if (ph < 48.0f) ph = 48.0f;
    pw = ph * 136.0f / 188.0f;
    if (pw > w - 2.0f * pad) { pw = w - 2.0f * pad; ph = pw * 188.0f / 136.0f; }
    q->portrait = (AtRect){ box.x + w - pad - pw, py, pw, ph };
    bw = (pw - 6.0f) * 0.5f;                                                   /* the two buttons on the portrait's top corners, as retail's CSPs carry them */
    if (bw > 32.0f) bw = 32.0f;
    if (bw < 20.0f) bw = 20.0f;
    q->type = (AtRect){ q->portrait.x + q->portrait.w - bw - 2.0f, py + 2.0f, bw, 16.0f };
    q->team = (AtRect){ q->portrait.x + 2.0f, py + 2.0f, bw, 16.0f };
    q->dock = (AtRect){ box.x + pad, py, w - 2.0f * pad - pw - 4.0f, ph };            /* the left column: the puck's dock sits in it */
    if (q->dock.w < 30.0f) q->dock = (AtRect){ q->portrait.x, py + ph - 30.0f, 30.0f, 30.0f };   /* too narrow: the dock rides the portrait's lower corner */
    q->name = (AtRect){ box.x + pad, py + ph + 4.0f, w - 2.0f * pad, 16.0f };
    q->cmid = (AtRect){ box.x + pad, q->name.y + 20.0f, w - 2.0f * pad, 14.0f };
    q->cprev = (AtRect){ q->cmid.x, q->cmid.y, 22.0f, 14.0f };
    q->cnext = (AtRect){ q->cmid.x + q->cmid.w - 22.0f, q->cmid.y, 22.0f, 14.0f };
    q->level = (AtRect){ box.x + pad, q->cmid.y + 18.0f, w - 2.0f * pad, 14.0f };       /* the CPU level slider is the last row, at the panel's foot */
    lw = 34.0f;
    nw = (q->level.w - lw - 8.0f * 2.0f) / 9.0f;
    for (i = 0; i < 9; i++) q->notch[i] = (AtRect){ q->level.x + lw + (float) i * (nw + 2.0f), q->level.y, nw, 14.0f };
}

static void csv_geometry_ph(const AtScreen *sc, const AtView *v, const AtLayout *L, AtCssGeom *g, float ph)
{
    int n = csv_nports(sc), i, count = sc->n_blocks > 0 ? at_block_count(&sc->blocks[0]) : 0, c, best_c = 0, fh;
    float w = L->body.w, x0 = L->body.x, top = L->body.y, bottom = L->body.y + L->body.h, pw, gap = 10.0f, avail, tabs_h = 0.0f, px0;
    memset(g, 0, sizeof *g);
    g->n_panels = n;
    pw = (w - gap * (float) (n - 1)) / (float) n;
    if (pw > 200.0f) pw = 200.0f;
    if (pw < 80.0f) { pw = 80.0f; gap = n > 1 ? (w - pw * (float) n) / (float) (n - 1) : 0.0f; if (gap < 2.0f) gap = 2.0f; }
    px0 = x0 + (w - (pw * (float) n + gap * (float) (n - 1))) * 0.5f;
    if (px0 < x0) px0 = x0;
    for (i = 0; i < n; i++) csv_panel(&g->panel[i], (AtRect){ px0 + (float) i * (pw + gap), bottom - ph, pw, ph });
    g->caption = (AtRect){ x0, bottom - ph - CSV_CAPTION_H - 4.0f, w, CSV_CAPTION_H };
    if (sc->n_tabs > 1) tabs_h = 30.0f;
    g->tabs = (AtRect){ x0, top, w, tabs_h };
    avail = g->caption.y - 4.0f - (top + tabs_h);
    if (avail < 40.0f) avail = 40.0f;
    g->gap = CSV_GAP;
    for (c = 6; c <= 24; c++) {                                              /* the fewest columns whose icons (at most 66 wide, 56:64) all fit; else scroll */
        float iw = (w - (float) (c - 1) * g->gap) / (float) c, ih;
        int rows;
        if (iw < 44.0f) break;                                               /* no icon smaller than 44 wide: scroll instead */
        if (iw > 66.0f) iw = 66.0f;
        ih = iw * 56.0f / 64.0f;
        rows = count > 0 ? (count + c - 1) / c : 1;
        if ((float) rows * ih + (float) (rows - 1) * g->gap <= avail) { best_c = c; break; }
    }
    if (best_c == 0) {                                                       /* too many to show at once: icons no smaller than 48 wide, rows scroll */
        best_c = (int) floor((double) ((w + g->gap) / (48.0f + g->gap)));
        if (best_c < 6) best_c = 6;
    }
    g->cols = best_c;
    g->cw = (w - (float) (g->cols - 1) * g->gap) / (float) g->cols;
    if (g->cw > 66.0f) g->cw = 66.0f;
    g->ch = g->cw * 56.0f / 64.0f;
    g->rows = count > 0 ? (count + g->cols - 1) / g->cols : 1;
    g->rows_vis = (int) floor((double) ((avail + g->gap) / (g->ch + g->gap)));
    if (g->rows_vis < 1) g->rows_vis = 1;
    if (g->rows_vis > g->rows) g->rows_vis = g->rows;
    fh = csv_field_hand(v);
    g->first_row = 0;
    if (fh >= 0 && v->cursor[fh].free) {                                     /* a free hand: the host scrolls the field (at its edges, with the wheel); the rows under the hand never move by themselves */
        g->first_row = v->scroll < 0 ? 0 : v->scroll;
        if (g->first_row > g->rows - g->rows_vis) g->first_row = g->rows - g->rows_vis;
        if (g->first_row < 0) g->first_row = 0;
    } else if (fh >= 0 && g->rows > g->rows_vis) {                           /* the hand's row stays in view, in the middle when it can */
        int fr = v->cursor[fh].index / g->cols;
        g->first_row = fr - g->rows_vis / 2;
        if (g->first_row > g->rows - g->rows_vis) g->first_row = g->rows - g->rows_vis;
        if (g->first_row < 0) g->first_row = 0;
    }
    {
        float fw = (float) g->cols * g->cw + (float) (g->cols - 1) * g->gap, fh2 = (float) g->rows_vis * g->ch + (float) (g->rows_vis - 1) * g->gap;
        g->field = (AtRect){ x0 + (w - fw) * 0.5f, top + tabs_h + (avail - fh2) * 0.5f, fw, fh2 };
    }
}

void at_csv_geometry(const AtScreen *sc, const AtView *v, const AtLayout *L, AtCssGeom *g)
{
    float ph = CSV_PANEL_H, spare;
    csv_geometry_ph(sc, v, L, g, ph);
    spare = (g->caption.y - 4.0f - g->tabs.y - g->tabs.h) - g->field.h;          /* room the roster does not need: the panels take most of it (a bigger portrait) */
    if (spare > 14.0f) {
        ph += (spare - 8.0f) * 0.75f > 34.0f ? 34.0f : (spare - 8.0f) * 0.75f;
        csv_geometry_ph(sc, v, L, g, ph);
    }
}

AtRect at_csv_tile_rect(const AtCssGeom *g, int i)
{
    int r = i / g->cols - g->first_row, c = i % g->cols;
    if (r < 0 || r >= g->rows_vis) return (AtRect){ g->field.x, g->field.y, 0.0f, 0.0f };
    return (AtRect){ g->field.x + (float) c * (g->cw + g->gap), g->field.y + (float) r * (g->ch + g->gap), g->cw, g->ch };
}

static void csv_text(const AtSink *s, const AtTextOps *o, int role, const char *str, float x, float base, unsigned col, int align, float max_w)
{
    char fit[AT_STR * 2];
    int r = at_fit(o, role, str, max_w, fit, (int) sizeof fit);
    at_text(s, o, r, fit, x, base, col, align, 0.0f);
}

static void csv_tri(const AtSink *s, float x0, float y0, float x1, float y1, float x2, float y2, unsigned rgba)
{
    float px[4], py[4];
    px[0] = x0; py[0] = y0; px[1] = x1; py[1] = y1; px[2] = x2; py[2] = y2; px[3] = x2; py[3] = y2;
    s->poly(s->user, px, py, rgba);
}

static void csv_ring(const AtSink *s, AtRect r, float t, unsigned rgba)
{
    at_poly_rect(s, r.x, r.y, r.w, t, rgba);
    at_poly_rect(s, r.x, r.y + r.h - t, r.w, t, rgba);
    at_poly_rect(s, r.x, r.y, t, r.h, rgba);
    at_poly_rect(s, r.x + r.w - t, r.y, t, r.h, rgba);
}

/* ---- the hand and coin art. Masks from menu/pipeline/css_tokens.py, tinted per port; flat shapes when the art is missing ---- */
static AtCssArt csv_art = { { { -1, -1, -1, -1, -1, -1, -1, -1 }, { -1, -1, -1, -1, -1, -1, -1, -1 } }, { -1, -1, -1, -1, -1, -1, -1, -1 } };
static double csv_now;          /* the UI clock of the frame being drawn (the coin's drop) */
static int csv_reduced;

void at_css_set_art(const AtCssArt *a) { if (a != NULL) csv_art = *a; }

static int csv_art_ok(const int *t, int n) { int i; for (i = 0; i < n; i++) if (t[i] < 0) return 0; return 1; }

static unsigned csv_alpha(unsigned rgb, float a) { return (rgb & 0xFFFFFF00u) | (unsigned) (a * 255.0f + 0.5f); }
static unsigned csv_mix(unsigned a, unsigned b, float t)      /* a toward b by t, opaque */
{
    unsigned r = 0;
    int sh;
    for (sh = 24; sh >= 8; sh -= 8) {
        float x = (float) ((a >> sh) & 0xFFu), y = (float) ((b >> sh) & 0xFFu);
        r |= ((unsigned) (x + (y - x) * t + 0.5f) & 0xFFu) << sh;
    }
    return r | 0xFFu;
}
static unsigned csv_scale(unsigned c, float k) { return csv_mix(0x000000FFu, c, k); }
static unsigned csv_port_rgb(int port) { return port == AT_CSS_CPU_COIN ? AT_C_CPU : at_part_port_colour(port); }

/* a coin. r is the coin's drawn radius (its ink edge). lift 0 lies on the table, 1 is in a hand (bigger, the shadow further off) */
static void csv_coin_ex(const AtSink *s, const AtTextOps *o, float cx, float cy, float r, int port, float lift)
{
    char lab[4];
    unsigned pc = csv_port_rgb(port);
    if (csv_art_ok(csv_art.coin, AT_CSS_COIN_LAYERS) && s->image != NULL) {
        /* the texture's 64 grid: the ink edge is r 30.6 about (32, 32) */
        float k = 1.0f + 0.15f * lift, rr = r * k, side = rr * 64.0f / 30.6f, x = cx - side * 0.5f, y = cy - side * 0.5f;
        float so = (1.0f + 3.0f * (lift > 0.0f ? lift : 0.0f)) * (r / 11.0f);
        unsigned col[AT_CSS_COIN_LAYERS];
        int i;
        col[0] = csv_alpha(AT_C_INK, 0.5f - 0.15f * lift); col[1] = AT_C_INK; col[2] = csv_scale(pc, 0.62f); col[3] = AT_C_IVORY;
        col[4] = pc; col[5] = csv_alpha(AT_C_INK, 0.35f); col[6] = csv_alpha(AT_C_INK, 0.25f); col[7] = csv_alpha(0xFFFFFF00u, 0.6f);
        for (i = 0; i < AT_CSS_COIN_LAYERS; i++)
            at_sink_image(s, csv_art.coin[i], x + (i == 0 ? so : 0.0f), y + (i == 0 ? so * 1.5f : 0.0f), side, side, col[i]);
        if (port == AT_CSS_CPU_COIN) snprintf(lab, sizeof lab, "CP");
        else if (rr >= 10.0f) snprintf(lab, sizeof lab, "P%d", port + 1);
        else snprintf(lab, sizeof lab, "%d", port + 1);
        at_text(s, o, rr >= 13.0f ? AT_R_CAP14 : AT_R_CAP12, lab, cx, cy + (rr >= 13.0f ? 5.0f : 4.0f), AT_C_INK, AT_ALIGN_CENTER, 0.0f);
        return;
    }
    if (lift > 0.0f) at_disc(s, cx + 2.0f, cy + 4.0f, r, 0x00000066u);
    at_disc(s, cx, cy, r, AT_C_INK);
    at_disc(s, cx, cy, r - 2.0f, pc);
    lab[0] = (char) ('1' + (port < 0 || port == AT_CSS_CPU_COIN ? 0 : port)); lab[1] = '\0';
    at_text(s, o, r >= 11.0f ? AT_R_CAP14 : AT_R_CAP12, lab, cx, cy + (r >= 11.0f ? 5.0f : 4.0f), AT_C_INK, AT_ALIGN_CENTER, 0.0f);
}
static void csv_coin(const AtSink *s, const AtTextOps *o, float cx, float cy, float r, int port, int lifted)
{
    csv_coin_ex(s, o, cx, cy, r, port, lifted ? 1.0f : 0.0f);
}

/* an empty dock: the coin's socket, sunk into the panel: a dark recess, the chip's inlays as dim notches, a thin ring in the port's colour,
 * the port's number engraved where the coin will sit */
static void csv_socket(const AtSink *s, const AtTextOps *o, float cx, float cy, float r, int port)
{
    if (csv_art_ok(csv_art.coin, AT_CSS_COIN_LAYERS) && s->image != NULL) {
        float side = r * 64.0f / 30.6f, x = cx - side * 0.5f, y = cy - side * 0.5f;
        unsigned pc = csv_port_rgb(port);
        char lab[4];
        at_sink_image(s, csv_art.coin[1], x, y - 1.0f, side, side, csv_alpha(0xFFFFFF00u, 0.10f));      /* the lip catching the light above */
        at_sink_image(s, csv_art.coin[1], x, y, side, side, AT_C_EDGE);
        at_sink_image(s, csv_art.coin[2], x, y, side, side, AT_C_GROUND);
        at_sink_image(s, csv_art.coin[3], x, y, side, side, csv_alpha(pc, 0.30f));
        at_sink_image(s, csv_art.coin[4], x, y, side, side, AT_C_GROUND2);
        at_sink_image(s, csv_art.coin[5], x, y, side, side, csv_alpha(pc, 0.55f));
        at_sink_image(s, csv_art.coin[6], x - side * 0.04f, y - side * 0.05f, side, side, csv_alpha(AT_C_INK, 0.6f));   /* the recess's shadow falls down-right */
        snprintf(lab, sizeof lab, "P%d", port + 1);
        at_text(s, o, AT_R_CAP14, lab, cx, cy + 5.0f, csv_alpha(pc, 0.45f), AT_ALIGN_CENTER, 0.0f);
        return;
    }
    at_disc(s, cx, cy, r - 5.0f, AT_C_PLATE);
}

/* the hand's pose and how it sits: the hotspot in the texture's 64 grid (css_tokens_layout.json), the number badge's centre */
static const float csv_hot[2][2] = { { 13.36f, 13.11f }, { 13.77f, 30.81f } };
#define CSV_BADGE_X 37.42f
#define CSV_BADGE_Y 33.78f
#define CSV_HAND_SIDE 82.0f       /* the texture's side on screen at scale 1: the glove is about 60 px, Melee's hand is big */

enum { CSV_POSE_POINT = 0, CSV_POSE_GRAB = 1 };

/* a hand with its hotspot at (x, y). hover: lowered onto what it is over (the shadow closes in, a touch smaller) */
static void csv_hand_ex(const AtSink *s, const AtTextOps *o, float x, float y, int port, int pose, int hover, float scale)
{
    unsigned pc = csv_port_rgb(port);
    if (csv_art_ok(csv_art.hand[pose], AT_CSS_HAND_LAYERS) && s->image != NULL) {
        float k = scale * (hover ? 0.94f : 1.0f), side = CSV_HAND_SIDE * k, u = side / 64.0f;
        float ox = x - csv_hot[pose][0] * u, oy = y - csv_hot[pose][1] * u, so = (hover ? 1.2f : 3.0f) * k;
        unsigned col[AT_CSS_HAND_LAYERS];
        int i;
        char num[3];
        col[0] = csv_alpha(AT_C_INK, hover ? 0.55f : 0.42f); col[1] = AT_C_INK; col[2] = csv_mix(AT_C_IVORY, pc, 0.22f); col[3] = pc;
        col[4] = csv_alpha(AT_C_INK, 0.28f); col[5] = csv_alpha(0xFFFFFF00u, 0.55f); col[6] = csv_alpha(AT_C_INK, 0.85f); col[7] = pc;
        for (i = 0; i < AT_CSS_HAND_LAYERS; i++)
            at_sink_image(s, csv_art.hand[pose][i], ox + (i == 0 ? so : 0.0f), oy + (i == 0 ? so * 1.5f : 0.0f), side, side, col[i]);
        if (port == AT_CSS_CPU_COIN) { num[0] = 'C'; num[1] = '\0'; }
        else { num[0] = (char) ('1' + (port < 0 ? 0 : port)); num[1] = '\0'; }
        at_text(s, o, AT_R_CAP14, num, ox + CSV_BADGE_X * u, oy + CSV_BADGE_Y * u + 5.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
        return;
    }
    {   /* the flat pointer (no art) */
        unsigned rgba = pc;
        csv_tri(s, x + 1.0f, y + 1.0f, x + 14.0f, y + 12.0f, x + 6.0f, y + 15.0f, 0x000000AAu);
        csv_tri(s, x + 4.0f, y + 12.0f, x + 8.0f, y + 25.0f, x + 12.0f, y + 23.0f, 0x000000AAu);
        csv_tri(s, x, y, x + 12.0f, y + 10.0f, x + 5.0f, y + 13.0f, rgba);
        csv_tri(s, x + 3.0f, y + 11.0f, x + 7.0f, y + 23.0f, x + 10.0f, y + 21.0f, rgba);
    }
}

/* a hand that carries a coin: the coin under the pinch, the grabbing hand over it */
static void csv_hand_carry(const AtSink *s, const AtTextOps *o, float x, float y, int port, int coin_port, float scale)
{
    if (csv_art_ok(csv_art.hand[CSV_POSE_GRAB], AT_CSS_HAND_LAYERS) && s->image != NULL) {
        float r = 13.0f * scale;
        csv_coin_ex(s, o, x, y, r, coin_port, 1.0f);                         /* the coin under the pointer, the pinch on its upper right rim */
        csv_hand_ex(s, o, x + r * 1.0f, y - r * 0.95f, port, CSV_POSE_GRAB, 0, scale);
        return;
    }
    csv_coin(s, o, x - 13.0f, y + 13.0f, 11.0f, coin_port, 1);
    csv_hand_ex(s, o, x, y, port, CSV_POSE_POINT, 0, scale);
}

/* a set-down coin's drop: 0 at rest, else how far it still is from the table (1 the moment it was let go), with a small undershoot */
static int csv_drop_last[AT_MAX_PORTS], csv_drop_seen[AT_MAX_PORTS];
static double csv_drop_t[AT_MAX_PORTS];
static float csv_drop_lift(int p, int cell)
{
    float u;
    if (p < 0 || p >= AT_MAX_PORTS) return 0.0f;
    if (!csv_drop_seen[p] || cell < 0) { csv_drop_seen[p] = 1; csv_drop_last[p] = cell; csv_drop_t[p] = -1.0e9; return 0.0f; }
    if (cell != csv_drop_last[p]) { csv_drop_last[p] = cell; csv_drop_t[p] = csv_now; }
    if (csv_reduced) return 0.0f;
    u = (float) ((csv_now - csv_drop_t[p]) / 170.0);
    if (u >= 1.0f || u < 0.0f) return 0.0f;
    if (u < 0.65f) return 1.0f - u / 0.65f * 1.4f;                   /* falls past the table (-0.4 is a 6% squash at 0.15 per unit) */
    return -0.4f + 0.4f * (u - 0.65f) / 0.35f;                          /* and settles */
}

static const char *csv_type_word(int i) { return i == 0 ? "HMN" : i == 1 ? "CPU" : "NA"; }

/* the type button (retail's togglebtn on each CSP): the state it is in, HMN, CPU or NA; A on it (a click) gives the next one */
static void csv_type_button(const AtSink *s, const AtTextOps *o, AtRect r, int kind, int port, int ring, unsigned ring_col)
{
    unsigned face = kind == 1 ? at_part_port_colour(port) : kind == 2 ? AT_C_CPU : AT_C_LIFT;
    at_plate(s, (AtRect){ r.x, r.y + 2.0f, r.w, r.h }, AT_C_INK, AT_C_INK, 0.0f, 3.0f);
    at_plate(s, r, face, AT_C_INK, 1.0f, 3.0f);
    at_poly_rect(s, r.x + 3.0f, r.y + 1.0f, r.w - 6.0f, 1.0f, csv_alpha(0xFFFFFF00u, kind == 0 ? 0.08f : 0.35f));
    at_text(s, o, AT_R_CAP12, csv_type_word(kind == 1 ? 0 : kind == 2 ? 1 : 2), r.x + r.w * 0.5f, r.y + 12.0f, kind == 0 ? AT_C_MUTED : AT_C_INK, AT_ALIGN_CENTER, 0.0f);
    if (ring) csv_ring(s, (AtRect){ r.x - 2.0f, r.y - 2.0f, r.w + 4.0f, r.h + 4.0f }, 2.0f, ring_col);
}

static void csv_draw_panel(const AtScreen *sc, const AtView *v, const AtCssGeom *g, int idx, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    const AtSelCard *k = &sc->ports[idx];
    const AtCssPanel *q = &g->panel[idx];
    unsigned col = k->kind == 2 ? AT_C_CPU : at_part_port_colour(idx), face = k->kind == 0 ? AT_C_GROUND2 : AT_C_PLATE2, hcol = 0;
    int closed = (k->flags & AT_CARD_CLOSED) != 0, p, hand = -1, zone = -1, i;
    AtRect box = q->box;
    char lab[8];
    for (p = 0; p < AT_MAX_CURSORS; p++) if (v->cursor[p].active && v->cursor[p].card == idx && !v->cursor[p].hold) { hand = p; zone = v->cursor[p].zone; break; }
    if (hand >= 0) hcol = at_part_port_colour(hand);
    at_plate(s, (AtRect){ box.x + 3.0f, box.y + 4.0f, box.w, box.h }, AT_C_EDGE, AT_C_EDGE, 0.0f, (float) AT_PX_CH_S);   /* the kit's hard shadow */
    at_plate(s, box, face, hand >= 0 ? hcol : AT_C_EDGE, hand >= 0 ? 2.0f : 3.0f, (float) AT_PX_CH_S);
    /* the port's band across the top: its colour, the number big on it (the coin's dock below carries the coin) */
    {
        AtRect b = { box.x + 3.0f, box.y + 3.0f, box.w - 6.0f, q->strip.h + 6.0f };
        unsigned bc = closed ? AT_C_LINE : (k->kind == 0 ? csv_scale(col, 0.45f) : col);
        at_plate(s, b, bc, bc, 0.0f, (float) AT_PX_CH_S);
        at_poly_rect(s, b.x + 2.0f, b.y + b.h - 3.0f, b.w - 4.0f, 3.0f, csv_alpha(AT_C_INK, 0.35f));   /* the band's lower lip */
    }
    if (closed) {
        hit_add(hc, box, AT_HIT_PANEL, idx, 0);
        csv_text(s, o, AT_R_ROW16, "CLOSED", box.x + box.w * 0.5f, box.y + box.h * 0.5f, AT_C_DIM, AT_ALIGN_CENTER, box.w - 16.0f);
        return;
    }
    snprintf(lab, sizeof lab, "P%d", idx + 1);
    at_text(s, o, AT_R_CAP16, lab, q->strip.x + 2.0f, q->strip.y + 15.0f, k->kind == 0 ? AT_C_TEXT2 : AT_C_INK, AT_ALIGN_LEFT, 0.0f);
    /* the portrait well, and the two buttons on it */
    at_poly_rect(s, q->portrait.x, q->portrait.y, q->portrait.w, q->portrait.h, AT_C_GROUND);
    if (k->kind != 0) {
        if (k->ptex >= 0 && s->image != NULL) at_sink_image(s, k->ptex, q->portrait.x, q->portrait.y, q->portrait.w, q->portrait.h, 0xFFFFFFFFu);
        else if (k->ck_tex >= 0 && s->image != NULL) at_sink_image(s, k->ck_tex, q->portrait.x + 2.0f, q->portrait.y + (q->portrait.h - (q->portrait.w - 4.0f) * 56.0f / 64.0f) * 0.5f, q->portrait.w - 4.0f, (q->portrait.w - 4.0f) * 56.0f / 64.0f, 0xFFFFFFFFu);
        else if (k->abbr[0] != '\0') at_text(s, o, AT_R_TITLE, k->abbr, q->portrait.x + q->portrait.w * 0.5f, q->portrait.y + q->portrait.h * 0.5f + 8.0f, AT_C_DIM, AT_ALIGN_CENTER, q->portrait.w - 4.0f);
    }
    csv_ring(s, q->portrait, 1.0f, AT_C_LINE);
    hit_add(hc, box, AT_HIT_PANEL, idx, 0);                                       /* the panel's body: a free hand over it is on the panel, not on a part */
    csv_type_button(s, o, q->type, k->kind, idx, zone == AT_ZONE_TYPE, hcol);
    hit_add(hc, q->type, AT_HIT_ZONE, idx, AT_ZONE_TYPE | (2 << 4));
    if (k->kind != 0 && k->team > 0 && q->team.w >= 20.0f) {
        const char *tw = k->team == 1 ? "RED" : k->team == 2 ? "BLU" : "GRN";
        unsigned tc = k->team == 1 ? AT_C_P1 : k->team == 2 ? AT_C_P2 : AT_C_P4;
        at_plate(s, q->team, tc, AT_C_INK, 1.0f, 3.0f);
        at_text(s, o, AT_R_CAP12, tw, q->team.x + q->team.w * 0.5f, q->team.y + 12.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
        hit_add(hc, q->team, AT_HIT_ZONE, idx, AT_ZONE_TEAM | (2 << 4));
        if (zone == AT_ZONE_TEAM) csv_ring(s, (AtRect){ q->team.x - 2.0f, q->team.y - 2.0f, q->team.w + 4.0f, q->team.h + 4.0f }, 2.0f, hcol);
    }
    if (k->kind == 0) {                                                       /* NA: an empty panel; the button on it is the way in */
        csv_text(s, o, AT_R_ROW16, "OPEN", q->name.x + q->name.w * 0.5f, q->name.y + 13.0f, AT_C_MUTED, AT_ALIGN_CENTER, q->name.w);
        csv_text(s, o, AT_R_BODY12, "A on the button", q->dock.x + q->dock.w * 0.5f, q->dock.y + q->dock.h * 0.5f, AT_C_DIM, AT_ALIGN_CENTER, q->dock.w);
        return;
    }
    /* the puck's dock: a ring with the coin in it when the coin is home, the ring empty when it is in a hand or on a tile */
    {
        float cx = q->dock.x + q->dock.w * 0.5f, cy = q->dock.y + (q->dock.h > 40.0f ? 18.0f : 15.0f), r = 15.0f;
        if (r > q->dock.w * 0.5f) r = q->dock.w * 0.5f;
        if (k->puck) {
            int art = csv_art_ok(csv_art.coin, AT_CSS_COIN_LAYERS) && s->image != NULL;
            if (!art) { at_disc(s, cx, cy, r + 2.0f, AT_C_INK); at_disc(s, cx, cy, r, AT_C_GROUND2); }
            if (!k->hold && k->drop_cell < 0) csv_coin(s, o, cx, cy, art ? r + 1.0f : r - 1.0f, idx, 0);
            else csv_socket(s, o, cx, cy, art ? r + 1.0f : r, idx);                   /* the dock is empty: the puck is in a hand or on a tile */
            hit_add(hc, (AtRect){ cx - r, cy - r, 2.0f * r, 2.0f * r }, AT_HIT_ZONE, idx, AT_ZONE_PUCK | (2 << 4));
            if (zone == AT_ZONE_PUCK) csv_ring(s, (AtRect){ cx - r - 3.0f, cy - r - 3.0f, 2.0f * r + 6.0f, 2.0f * r + 6.0f }, 2.0f, hcol);
        }
    }
    /* the name */
    at_plate(s, q->name, AT_C_GROUND2, AT_C_GROUND2, 0.0f, 3.0f);
    at_poly_rect(s, q->name.x, q->name.y, 3.0f, q->name.h, col);
    if (k->name[0] != '\0') csv_text(s, o, AT_R_CAP16, k->name, q->name.x + q->name.w * 0.5f + 1.0f, q->name.y + 13.0f, AT_C_IVORY, AT_ALIGN_CENTER, q->name.w - 8.0f);
    /* the costume stepper */
    if (k->ck_tex >= 0 || k->ptex >= 0 || k->ncost > 0) {
        char t[32];
        if (k->ncost > 1) snprintf(t, sizeof t, "%d / %d", k->costume + 1, k->ncost); else snprintf(t, sizeof t, "%s", "COSTUME 1");
        if (k->ncost > 1) {
            at_text(s, o, AT_R_NUM14, "<", q->cprev.x + 11.0f, q->cprev.y + 12.0f, AT_C_MUTED, AT_ALIGN_CENTER, 0.0f);
            at_text(s, o, AT_R_NUM14, ">", q->cnext.x + 11.0f, q->cnext.y + 12.0f, AT_C_MUTED, AT_ALIGN_CENTER, 0.0f);
        }
        csv_text(s, o, AT_R_NUM12, t, q->cmid.x + q->cmid.w * 0.5f, q->cmid.y + 12.0f, AT_C_TEXT2, AT_ALIGN_CENTER, q->cmid.w - 48.0f);
        if (k->ncost > 1) {
            hit_add(hc, q->cmid, AT_HIT_ZONE, idx, AT_ZONE_COSTUME | (2 << 4));
            hit_add(hc, q->cprev, AT_HIT_ZONE, idx, AT_ZONE_COSTUME | (1 << 4));
            hit_add(hc, q->cnext, AT_HIT_ZONE, idx, AT_ZONE_COSTUME | (3 << 4));
        }
        if (zone == AT_ZONE_COSTUME) csv_ring(s, (AtRect){ q->cmid.x - 2.0f, q->cmid.y - 2.0f, q->cmid.w + 4.0f, q->cmid.h + 4.0f }, 2.0f, hcol);
    }
    /* the CPU level slider, at the foot of the panel */
    if (k->kind == 2) {
        char lv[8];
        snprintf(lv, sizeof lv, "LV %d", k->cpu_lv);
        at_text(s, o, AT_R_NUM12, lv, q->level.x, q->level.y + 12.0f, AT_C_TEXT2, AT_ALIGN_LEFT, 0.0f);
        {   /* a groove, the fill up to the level, a tick per level, the knob on the level (a coin face, grabbed by a hand while it is dragged) */
            float x0 = q->notch[0].x + q->notch[0].w * 0.5f, x1 = q->notch[8].x + q->notch[8].w * 0.5f, ty = q->level.y + 7.0f;
            int lvl = k->cpu_lv < 1 ? 1 : (k->cpu_lv > 9 ? 9 : k->cpu_lv), held = 0;
            float kx = q->notch[lvl - 1].x + q->notch[lvl - 1].w * 0.5f;
            for (p = 0; p < AT_MAX_CURSORS; p++) if (v->cursor[p].active && v->cursor[p].drag == idx) held = 1;
            at_poly_rect(s, x0 - 2.0f, ty - 3.0f, x1 - x0 + 4.0f, 6.0f, AT_C_EDGE);
            at_poly_rect(s, x0, ty - 1.0f, kx - x0, 3.0f, col);
            for (i = 0; i < 9; i++) at_poly_rect(s, q->notch[i].x + q->notch[i].w * 0.5f - 0.5f, ty + 4.0f, 1.0f, i + 1 <= lvl ? 3.0f : 2.0f, i + 1 <= lvl ? col : AT_C_LINE);
            at_disc(s, kx + 1.0f, ty + 2.0f, held ? 7.5f : 6.5f, csv_alpha(AT_C_INK, 0.6f));
            at_disc(s, kx, ty, held ? 7.0f : 6.0f, AT_C_INK);
            at_disc(s, kx, ty, held ? 5.5f : 4.5f, held ? AT_C_IVORY : col);
            at_disc(s, kx - 1.2f, ty - 1.4f, 1.6f, csv_alpha(0xFFFFFF00u, 0.7f));
        }
        hit_add(hc, q->level, AT_HIT_ZONE, idx, AT_ZONE_LEVEL | (2 << 4));
        for (i = 0; i < 9; i++) hit_add(hc, (AtRect){ q->notch[i].x - 1.0f, q->notch[i].y - 2.0f, q->notch[i].w + 2.0f, 18.0f }, AT_HIT_ZONE, idx, AT_ZONE_LEVEL | ((i + 1 + 2) << 4));
        if (zone == AT_ZONE_LEVEL) csv_ring(s, (AtRect){ q->level.x - 2.0f, q->level.y - 3.0f, q->level.w + 4.0f, q->level.h + 6.0f }, 2.0f, hcol);
    }
}

static void draw_css(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, HitCtx *hc)
{
    AtCssGeom g;
    const AtBlock *bk = sc->n_blocks > 0 ? &sc->blocks[0] : NULL;
    int count = bk != NULL ? at_block_count(bk) : 0, i, p, n = csv_nports(sc), fh;
    at_csv_geometry(sc, v, L, &g);
    if (sc->n_tabs > 1) draw_tabs(sc, v, g.tabs, o, s, hc);
    /* the icon field, flush: the icon art as it is, a thin dark edge; a tile with no art shows its letters */
    for (i = 0; i < count; i++) {
        AtRect r = at_csv_tile_rect(&g, i);
        const AtCell *c = at_block_cell(bk, i);
        int dis = (c->flags & AT_CELL_DISABLED) != 0;
        if (r.w <= 0.0f) continue;
        at_poly_rect(s, r.x, r.y, r.w, r.h, AT_C_GROUND2);
        if (c->tex >= 0 && s->image != NULL) at_sink_image(s, c->tex, r.x + 1.0f, r.y + 1.0f, r.w - 2.0f, r.h - 2.0f, dis ? 0xFFFFFF66u : 0xFFFFFFFFu);
        else {
            at_text(s, o, AT_R_CAP16, c->abbr[0] != '\0' ? c->abbr : "?", r.x + r.w * 0.5f, r.y + r.h * 0.5f + 6.0f, dis ? AT_C_DIM : AT_C_MUTED, AT_ALIGN_CENTER, r.w - 4.0f);
        }
        csv_ring(s, r, 1.0f, AT_C_EDGE);
        if (c->origin != 0) {
            char one[2];
            one[0] = c->origin; one[1] = '\0';
            at_poly_rect(s, r.x + r.w - 12.0f, r.y, 12.0f, 12.0f, c->origin == 'G' ? AT_C_JADE : AT_C_SUN);
            at_text(s, o, AT_R_CAP12, one, r.x + r.w - 6.0f, r.y + 10.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
        }
        hit_add(hc, r, AT_HIT_CELL, 0, i);
    }
    /* set-down pucks lie on their tiles, side by side when two share one; a puck not on a tile resets its drop */
    for (p = 0; p < n; p++) if (!(sc->ports[p].puck && sc->ports[p].kind != 0 && sc->ports[p].drop_cell >= 0)) (void) csv_drop_lift(p, -1);
    for (i = 0; i < count; i++) {
        AtRect r = at_csv_tile_rect(&g, i);
        int m = 0, seen = 0;
        if (r.w <= 0.0f) continue;
        for (p = 0; p < n; p++) if (sc->ports[p].puck && sc->ports[p].kind != 0 && sc->ports[p].drop_cell == i) m++;
        for (p = 0; p < n; p++) {
            if (sc->ports[p].puck && sc->ports[p].kind != 0 && sc->ports[p].drop_cell == i) {
                float rr = r.w * 0.19f < 8.0f ? 8.0f : (r.w * 0.19f > 12.0f ? 12.0f : r.w * 0.19f);
                float cx = r.x + r.w * 0.5f + ((float) seen - (float) (m - 1) * 0.5f) * rr * 1.25f;
                csv_coin_ex(s, o, cx, r.y + r.h * 0.58f, rr, p, csv_drop_lift(p, i));
                seen++;
            }
        }
    }
    /* the panels */
    for (p = 0; p < n; p++) csv_draw_panel(sc, v, &g, p, o, s, hc);
    /* the hands over the field: a bracket on the tile, a pointer at its corner, the carried coin by it */
    fh = 0;
    for (p = 0; p < AT_MAX_CURSORS; p++) {
        AtRect r;
        unsigned pc = at_part_port_colour(p);
        if (!v->cursor[p].active || v->cursor[p].card >= 0 || v->cursor[p].index < 0) continue;
        if (v->cursor[p].free && v->cursor[p].blank) continue;                  /* a free hand between tiles brackets nothing */
        r = at_csv_tile_rect(&g, v->cursor[p].index);
        if (r.w <= 0.0f) continue;
        {
            int sl = 0, of = 0, q;
            for (q = 0; q < AT_MAX_CURSORS; q++) if (v->cursor[q].active && v->cursor[q].card < 0 && v->cursor[q].index == v->cursor[p].index) { if (q < p) sl++; of++; }
            at_cell_brackets(s, r, pc, sl, of);
            if (!v->cursor[p].free) {
                if (v->cursor[p].hold) csv_hand_carry(s, o, r.x + r.w * 0.55f + (float) (fh % 3) * 3.0f, r.y + r.h * 0.45f, p, v->cursor[p].hold_port, 1.0f);
                else csv_hand_ex(s, o, r.x + r.w * 0.55f + (float) (fh % 3) * 3.0f, r.y + r.h * 0.45f, p, CSV_POSE_POINT, 1, 1.0f);
            }
        }
        fh++;
    }
    /* a puck carried over the panels rides at the pointer's panel */
    for (p = 0; p < AT_MAX_CURSORS; p++) {
        if (v->cursor[p].active && !v->cursor[p].free && v->cursor[p].card >= 0 && v->cursor[p].hold && v->cursor[p].card < n) {
            const AtCssPanel *q = &g.panel[v->cursor[p].card];
            csv_hand_carry(s, o, q->portrait.x + q->portrait.w * 0.5f, q->portrait.y + q->portrait.h * 0.4f, p, v->cursor[p].hold_port, 1.0f);
        }
    }
    /* the hand on a panel: a pointer by the part it is on */
    for (p = 0; p < AT_MAX_CURSORS; p++) {
        if (v->cursor[p].active && !v->cursor[p].free && v->cursor[p].card >= 0 && !v->cursor[p].hold && v->cursor[p].card < n) {
            const AtCssPanel *q = &g.panel[v->cursor[p].card];
            AtRect a = v->cursor[p].zone == AT_ZONE_PUCK ? (AtRect){ q->dock.x + q->dock.w * 0.5f + 8.0f, q->dock.y + 22.0f, 0.0f, 0.0f } :
                       v->cursor[p].zone == AT_ZONE_TYPE ? (AtRect){ q->type.x + q->type.w * 0.5f, q->type.y + 10.0f, 0.0f, 0.0f } :
                       v->cursor[p].zone == AT_ZONE_LEVEL ? (AtRect){ q->notch[4].x + 4.0f, q->notch[4].y + 6.0f, 0.0f, 0.0f } :
                       v->cursor[p].zone == AT_ZONE_COSTUME ? (AtRect){ q->cmid.x + q->cmid.w * 0.5f + 12.0f, q->cmid.y + 5.0f, 0.0f, 0.0f } :
                       (AtRect){ q->team.x + q->team.w * 0.5f, q->team.y + 10.0f, 0.0f, 0.0f };
            csv_hand_ex(s, o, a.x, a.y, p, v->cursor[p].drag >= 0 ? CSV_POSE_GRAB : CSV_POSE_POINT, 1, 1.0f);
        }
    }
    /* free hands, on top of everything: the pointer at the stick's position, a carried coin by it */
    for (p = 0; p < AT_MAX_CURSORS; p++) {
        if (!v->cursor[p].active || !v->cursor[p].free) continue;
        if (v->cursor[p].hold) csv_hand_carry(s, o, v->cursor[p].hx, v->cursor[p].hy, p, v->cursor[p].hold_port, 1.0f);
        else {
            int over = v->cursor[p].drag >= 0 || (v->cursor[p].card >= 0 ? (v->cursor[p].zone >= 0 && v->cursor[p].zone < AT_ZONE_N) : !v->cursor[p].blank && v->cursor[p].index >= 0);
            csv_hand_ex(s, o, v->cursor[p].hx, v->cursor[p].hy, p, v->cursor[p].drag >= 0 ? CSV_POSE_GRAB : CSV_POSE_POINT, over, 1.0f);
        }
    }
    /* the caption: the fighter under the hand (or the last one), what is missing, or Ready to Fight */
    {
        int h = csv_field_hand(v);
        AtRect cr = g.caption;
        if (h >= 0 && v->cursor[h].index >= 0 && v->cursor[h].index < count) {
            const AtCell *c = at_block_cell(bk, v->cursor[h].index);
            csv_text(s, o, AT_R_CAP16, c->name, cr.x + 2.0f, cr.y + 16.0f, AT_C_IVORY, AT_ALIGN_LEFT, cr.w * 0.4f);
        }
        if (sc->css_ready) {
            float bw = cr.w * 0.5f < 330.0f ? cr.w * 0.5f : 330.0f, bx = cr.x + cr.w - bw;
            at_plate(s, (AtRect){ bx, cr.y, bw, cr.h }, AT_C_JADE, AT_C_JADE, 0.0f, 6.0f);
            csv_text(s, o, AT_R_CAP16, "READY TO FIGHT!  PRESS START", bx + bw * 0.5f, cr.y + 16.0f, AT_C_INK, AT_ALIGN_CENTER, bw - 12.0f);
        } else if (sc->css_msg[0] != '\0') {
            csv_text(s, o, AT_R_BODY14, sc->css_msg, cr.x + cr.w, cr.y + 16.0f, AT_C_MUTED, AT_ALIGN_RIGHT, cr.w * 0.58f);
        }
    }
}

/* the trail and the rule; no rail or chapter marker (the screen is not part of a chapter list) */
static float draw_header_css(const AtScreen *sc, const AtLayout *L, const AtTextOps *o, const AtSink *s)
{
    const char *items[4];
    int n = 0, i;
    float end;
    for (i = 0; i < sc->n_parents && n < 3; i++) items[n++] = sc->parent[i];
    items[n++] = sc->title;
    end = at_part_trail(s, o, L->trail, items, n);
    at_poly_rect(s, L->rule.x, L->rule.y, L->rule.w, 1.0f, AT_C_LINE);
    return end;
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
    memset(&cs, 0, sizeof cs);
    cs.user = &cn; cs.poly = cnt_poly; cs.text = cnt_text; cs.model = cnt_model;
    cs.image = out->image != NULL ? cnt_image : NULL;                   /* no image op below: the parts see none and draw the letters instead */
    hits->n = 0;
    hc.hits = v->dialog.open ? NULL : hits;                              /* an open dialog is the only thing the mouse can reach */
    hc.dropped = 0;
    at_layout(canvas_w, sc->preset, &L);
    if (sc->primary == AT_PRIMARY_FRAME) {                                /* no ground, no focus, no hits (hits->n is already 0) */
        render_frame(sc, v, &L, o, s);
        if (info != NULL) { info->entries = cn.entries; info->dropped = cn.dropped; info->hits_dropped = 0; info->warned = cn.warned; info->capped = cn.capped; }
        return;
    }
    if (sc->backdrop == AT_BD_WORLD) {                                    /* over the frozen game: a scrim that ramps in on the UI clock (a cut with Reduced Motion), never the ground */
        AtTween wt;
        at_tween_start(&wt, v->opened_ms, 100.0, reduced);
        at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, (AT_C_SCRIM & 0xFFFFFF00u) | (unsigned) ((float) (AT_C_SCRIM & 0xFFu) * at_tween_value(&wt, now) + 0.5f));
    } else {
        at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, AT_C_GROUND);
    }
    if (sc->primary == AT_PRIMARY_DISPLAY) {                              /* a display screen: no header, no pane, no hits, no key strip */
        at_part_title(s, o, &L, sc, now, reduced, NULL);
    } else if (sc->primary == AT_PRIMARY_CSS) {
        trail_end = draw_header_css(sc, &L, o, s);
        csv_now = now; csv_reduced = reduced;
        draw_css(sc, v, &L, o, s, &hc);
        draw_keys(sc, v, &L, o, s, &hc);
    } else {
        trail_end = draw_header(sc, &L, o, s);
        if (sc->primary == AT_PRIMARY_GRID && (sc->n_tabs > 0 || sc->band != AT_BAND_NONE)) {   /* the character select: tabs over the pane, a band under it */
            AtSplit sp;
            AtLayout gl = L;
            at_layout_split(&L, sc->n_tabs > 0, sc->band, &sp);
            gl.primary = sp.grid;
            at_plate(s, sp.grid, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
            draw_grid(sc, v, &gl, o, s, &hc);
            if (sc->n_tabs > 0) draw_tabs(sc, v, sp.tabs, o, s, &hc);
            if (sc->band == AT_BAND_MATCHUP) draw_loading(v, sp.grid, o, s);
            if (sc->band != AT_BAND_NONE) draw_band(sc, v, sp.band, o, s, &hc);
        } else {
            AtSplit lsp;
            int list_tabs = sc->primary == AT_PRIMARY_LIST && sc->n_tabs > 0;      /* a list with tabs (the settings pages): the strip hangs on the pane's top edge */
            if (list_tabs) at_layout_split(&L, 1, AT_BAND_NONE, &lsp);
            AtRect lpane = list_tabs ? lsp.grid : L.primary;
            if (sc->primary == AT_PRIMARY_LIST && sc->credit[0] != '\0') {      /* the credit strip under a list pane: the pane gives up 22 px (at_list_pane_h is the model's copy of this) */
                lpane.h -= AT_CREDIT_H;
                fit_text_credit(s, o, sc->credit, lpane.x + 4.0f, lpane.y + lpane.h + 16.0f, lpane.w - 8.0f);
            }
            at_plate(s, lpane, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
            if (sc->primary == AT_PRIMARY_ROOM) hc.dropped += at_room_render(v->room, &L, o, s, hc.hits, now);   /* a NULL view draws nothing: the chrome only */
            else if (sc->primary == AT_PRIMARY_GRID) draw_grid(sc, v, &L, o, s, &hc);
            else if (sc->primary == AT_PRIMARY_TILES) draw_tiles(sc, v, &L, o, s, &hc);
            else if (sc->primary == AT_PRIMARY_CARDS) draw_cards(sc, v, &L, o, s, &hc);
            else draw_list(sc, v, lpane, o, s, &hc);
            if (list_tabs) draw_tabs(sc, v, lsp.tabs, o, s, &hc);
        }
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
    if (sc->primary == AT_PRIMARY_ROOM && v->room != NULL && v->room->fade_out > 0) {   /* and a room fades out on leaving: the game's own fade does not cover host quads */
        float f = (float) v->room->fade_out / 1000.0f;
        if (f > 1.0f) f = 1.0f;
        if (f > k) k = f;
    }
    if (sc->backdrop != AT_BD_WORLD && k > 0.004f) at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, (AT_C_GROUND & 0xFFFFFF00u) | (unsigned) (k * 255.0f + 0.5f));
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
