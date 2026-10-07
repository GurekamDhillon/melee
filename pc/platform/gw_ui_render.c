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

/* A heading precedes row i when it names a group the row before it does not (the first row too). Headings are not rows: no focus, no hit rectangle. */
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
        float cost = 39.0f + (heading_before(sc, i) ? 22.0f : 0.0f);
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
        r.x = x0; r.y = y; r.w = iw; r.h = 34.0f;
        at_part_row(s, o, r, &sc->items[i], (v->focus.index == i) ? AT_ST_FOCUS : AT_ST_REST);
        hit_add(hc, r, AT_HIT_CELL, 0, i);
        y += 39.0f;
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
    if (sc->backdrop == AT_BD_WORLD) {                                    /* over the frozen game: a scrim that ramps in on the UI clock (a cut with Reduced Motion), never the ground */
        AtTween wt;
        at_tween_start(&wt, v->opened_ms, 100.0, reduced);
        at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, (AT_C_SCRIM & 0xFFFFFF00u) | (unsigned) ((float) (AT_C_SCRIM & 0xFFu) * at_tween_value(&wt, now) + 0.5f));
    } else {
        at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, AT_C_GROUND);
    }
    if (sc->primary == AT_PRIMARY_DISPLAY) {                              /* a display screen: no header, no pane, no hits, no key strip */
        at_part_title(s, o, &L, sc, now, reduced, NULL);
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
            at_plate(s, list_tabs ? lsp.grid : L.primary, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
            if (sc->primary == AT_PRIMARY_GRID) draw_grid(sc, v, &L, o, s, &hc);
            else if (sc->primary == AT_PRIMARY_TILES) draw_tiles(sc, v, &L, o, s, &hc);
            else if (sc->primary == AT_PRIMARY_CARDS) draw_cards(sc, v, &L, o, s, &hc);
            else draw_list(sc, v, list_tabs ? lsp.grid : L.primary, o, s, &hc);
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
