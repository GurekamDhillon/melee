#include "gw_ui_render.h"
#include "gw_ui_stack.h"
#include "gw_ui_tokens.h"

#include <math.h>
#include <stdio.h>
#include <string.h>

static void hit_add(AtHits *h, AtRect r, int kind, int a, int b)
{
    if (h->n >= AT_MAX_HITS) return;
    h->h[h->n].r = r; h->h[h->n].kind = kind; h->h[h->n].a = a; h->h[h->n].b = b;
    h->n++;
}

static void draw_header(const AtScreen *sc, const AtLayout *L, const AtTextOps *o, const AtSink *s)
{
    const char *items[4];
    int n = 0, i;
    for (i = 0; i < sc->n_parents && n < 3; i++) items[n++] = sc->parent[i];
    items[n++] = sc->title;
    at_part_trail(s, o, L->trail, items, n);
    if (L->wide) at_part_rail(s, o, L->rail, sc->chapter);
    else at_part_chapter(s, o, L->chapter, sc->chapter);
    at_poly_rect(s, L->rule.x, L->rule.y, L->rule.w, 1.0f, AT_C_LINE);
}

static void draw_grid(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *hits)
{
    float x0 = L->primary.x + 12.0f, y = L->primary.y + 12.0f, iw = L->primary.w - 24.0f, gap = 8.0f, cell;
    int b, i, maxc = 1;
    for (b = 0; b < sc->n_blocks; b++) if (!sc->blocks[b].stones && sc->blocks[b].cols > maxc) maxc = sc->blocks[b].cols;
    cell = (float) floor((iw - (float) (maxc - 1) * gap) / (float) maxc);
    if (cell > 56.0f) cell = 56.0f;
    if (cell < 24.0f) cell = 24.0f;
    for (b = 0; b < sc->n_blocks; b++) {
        const AtBlock *bk = &sc->blocks[b];
        int rows = (bk->n + bk->cols - 1) / bk->cols;
        float cw = bk->stones ? 34.0f : cell, ch = bk->stones ? 38.0f : cell, tw, cnw;
        if (rows < 1) rows = 1;
        tw = o->width(o->user, AT_R_CAP14, bk->title);
        cnw = bk->count[0] ? o->width(o->user, AT_R_NUM12, bk->count) : 0.0f;
        at_text(s, o, AT_R_CAP14, bk->title, x0, y + 14.0f, AT_C_MUTED, AT_ALIGN_LEFT, iw - cnw - 16.0f);
        if (bk->count[0]) at_text(s, o, AT_R_NUM12, bk->count, x0 + iw, y + 14.0f, AT_C_DIM, AT_ALIGN_RIGHT, 0.0f);
        if (x0 + tw + 10.0f < x0 + iw - cnw - 10.0f) at_poly_rect(s, x0 + tw + 10.0f, y + 9.0f, iw - tw - cnw - 20.0f, 1.0f, AT_C_LINE);
        y += 26.0f;
        for (i = 0; i < bk->n; i++) {
            AtRect r;
            int st = (v->focus.block == b && v->focus.index == i) ? AT_ST_FOCUS : AT_ST_REST;
            r.x = x0 + (float) (i % bk->cols) * (cw + gap);
            r.y = y + (float) (i / bk->cols) * (ch + gap);
            r.w = cw; r.h = ch;
            if (bk->stones) at_part_stone(s, o, r, &bk->cells[i], st, v->port_rgba);
            else at_part_cell(s, o, r, &bk->cells[i], st, v->port_rgba);
            hit_add(hits, r, AT_HIT_CELL, b, i);
        }
        if (bk->stones && bk->note[0]) at_text(s, o, AT_R_BODY14, bk->note, x0 + (float) bk->n * (cw + gap) + 8.0f, y + 24.0f, AT_C_TEXT2, AT_ALIGN_LEFT, iw - (float) bk->n * (cw + gap) - 8.0f);
        y += (float) rows * (ch + gap) - gap + 12.0f;
    }
    if (sc->footer.has) {
        AtRect f;
        f.x = x0; f.y = L->primary.y + L->primary.h - 12.0f - 48.0f; f.w = iw; f.h = 48.0f;
        at_part_footer(s, o, f, &sc->footer);
    }
}

static void draw_list(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *hits)
{
    float x0 = L->primary.x + 12.0f, y0 = L->primary.y + 14.0f, iw = L->primary.w - 24.0f;
    int visible = (int) floor((L->primary.h - 28.0f) / 39.0f), first = v->scroll < 0 ? 0 : v->scroll, i;   /* the host keeps v->scroll valid */
    for (i = first; i < sc->n_items && i < first + visible; i++) {
        AtRect r;
        r.x = x0; r.y = y0 + (float) (i - first) * 39.0f; r.w = iw; r.h = 34.0f;
        at_part_row(s, o, r, &sc->items[i], (v->focus.index == i) ? AT_ST_FOCUS : AT_ST_REST);
        hit_add(hits, r, AT_HIT_CELL, 0, i);
    }
}

static void draw_keys(const AtScreen *sc, const AtView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *hits)
{
    float x = L->keys.x, base = L->keys.y + 18.0f, adv;
    int i;
    adv = at_part_hint(s, o, x, base, 'M', "Move");
    x += adv;
    for (i = 0; i < sc->n_keys; i++) {
        AtRect r;
        if (!v->key_shown[i] || v->key_label[i][0] == '\0') continue;
        adv = at_part_hint(s, o, x, base, sc->keys[i].btn, v->key_label[i]);
        r.x = x; r.y = base - 16.0f; r.w = adv - 18.0f; r.h = 22.0f;
        hit_add(hits, r, AT_HIT_KEY, sc->keys[i].btn, 0);
        x += adv;
    }
    if (v->counter[0]) at_text(s, o, AT_R_NUM16, v->counter, L->keys.x + L->keys.w, base, AT_C_DIM, AT_ALIGN_RIGHT, 0.0f);
}

void at_render(const AtScreen *sc, const AtView *v, float canvas_w, double now, int reduced,
               const AtTextOps *o, const AtSink *s, AtHits *hits)
{
    AtLayout L;
    AtTween open;
    float k;
    hits->n = 0;
    at_layout(canvas_w, sc->preset, &L);
    at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, AT_C_GROUND);
    draw_header(sc, &L, o, s);
    at_plate(s, L.primary, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
    if (sc->primary == AT_PRIMARY_GRID) draw_grid(sc, v, &L, o, s, hits);
    else draw_list(sc, v, &L, o, s, hits);
    if (sc->preset != AT_PRESET_NONE) at_part_explainer(s, o, L.explainer, &v->ex);
    draw_keys(sc, v, &L, o, s, hits);
    if (v->note.text[0] != '\0' && now < v->note.until_ms) {
        AtRect r;
        float w = o->width(o->user, AT_R_BODY14, v->note.text) + 54.0f, right = L.wide ? L.header.x + L.header.w : L.chapter.x - 12.0f;
        if (w > 300.0f) w = 300.0f;
        r.x = right - w; r.y = 22.0f; r.w = w; r.h = 30.0f;
        at_part_note(s, o, r, v->note.text, v->note.kind, v->note.until_ms > v->note.from_ms ? (float) ((v->note.until_ms - now) / (v->note.until_ms - v->note.from_ms)) : 0.0f);
    }
    if (v->dialog.open) {
        AtRect b[2];
        AtTween rise;
        int n, i;
        at_tween_start(&rise, v->dialog.from_ms, 140.0, reduced);
        n = at_part_dialog(s, o, L.canvas.w, &v->dialog, 6.0f * (1.0f - at_tween_value(&rise, now)), b);
        for (i = 0; i < n; i++) hit_add(hits, b[i], AT_HIT_DIALOG, v->dialog.btn[i], 0);
    }
    at_tween_start(&open, v->opened_ms, 100.0, reduced);
    k = 1.0f - at_tween_value(&open, now);                               /* the screen fades in from the ground colour */
    if (k > 0.004f) at_poly_rect(s, 0.0f, 0.0f, L.canvas.w, 480.0f, (AT_C_GROUND & 0xFFFFFF00u) | (unsigned) (k * 255.0f + 0.5f));
}
