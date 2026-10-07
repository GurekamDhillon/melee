#define AT_SINK_HAS_IMAGE
#include <stdlib.h>
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
#include "atlas_style.h"
#include "../platform/gw_ui_render.h"

static const AtTextOps O = { fake_width, NULL };
static AtScreen SC; static AtView V; static AtCell POOL[AT_MAX_EXT_CELLS]; static AtHits HITS;

/* what the adapter submits for the character select (Task 9 uses the same field assignments: keep them in step) */
static void build_css_screen(AtScreen *sc, AtView *v, AtCell *pool, int n, int n_added, int ports_mask)
{
    int i, p;
    memset(sc, 0, sizeof *sc); at_view_init(v); memset(pool, 0, sizeof(AtCell) * AT_MAX_EXT_CELLS);
    snprintf(sc->id, sizeof sc->id, "%s", "select.css"); snprintf(sc->title, sizeof sc->title, "%s", "FIGHTERS");
    snprintf(sc->parent[0], AT_STR, "%s", "VERSUS"); snprintf(sc->parent[1], AT_STR, "%s", "MELEE"); sc->n_parents = 2; sc->chapter = 2;
    sc->primary = AT_PRIMARY_GRID; sc->preset = AT_PRESET_NORMAL; sc->grid_cols_auto = 1; sc->band = AT_BAND_CARDS;
    sc->n_blocks = 1; snprintf(sc->blocks[0].id, AT_ID, "%s", "fighters"); sc->blocks[0].ext = pool; sc->blocks[0].ext_n = n; sc->blocks[0].cols = 0;
    sc->n_tabs = n_added > 0 ? 3 : 1;
    snprintf(sc->tabs[0].name, 24, "ALL"); sc->tabs[0].count = n;
    if (n_added > 0) { snprintf(sc->tabs[1].name, 24, "RETAIL"); sc->tabs[1].count = n - n_added; snprintf(sc->tabs[2].name, 24, "ADDED"); sc->tabs[2].count = n_added; }
    for (i = 0; i < n; i++) {
        AtCell *c = &pool[i];
        snprintf(c->id, AT_ID, "f%d", i); c->model = AT_NO_MODEL; c->ring = AT_NO_MODEL; c->tex = -1;
        c->abbr[0] = (char) ('A' + i % 26); c->abbr[1] = (char) ('A' + (i * 7) % 26);
        if (i >= n - n_added) c->origin = '+';
    }
    for (p = 0; p < 4; p++) {
        AtPortCard *k = &sc->cards[p];
        k->port = p; k->kind = (ports_mask >> p) & 1 ? 1 : 0; k->ck_tex = -1;
        if (k->kind) { snprintf(k->name, AT_STR, "%s", "A FIGHTER WITH A LONG NAME"); snprintf(k->sub, AT_STR, "%s", "Costume 1"); snprintf(k->abbr, 3, "%s", "FO"); }
    }
    sc->keys[0].btn = 'A'; snprintf(sc->keys[0].label, AT_STR, "%s", "Pick"); sc->keys[1].btn = 'B'; snprintf(sc->keys[1].label, AT_STR, "%s", "Back");
    sc->keys[2].btn = 'X'; snprintf(sc->keys[2].label, AT_STR, "%s", "Costume"); sc->keys[3].btn = 'S'; snprintf(sc->keys[3].label, AT_STR, "%s", "Fight"); sc->n_keys = 4;
    for (i = 0; i < 4; i++) v->key_shown[i] = 1;
    for (i = 0; i < 4; i++) snprintf(v->key_label[i], AT_STR, "%s", sc->keys[i].label);
    snprintf(v->counter, AT_STR, "%d / %d", 1, n);
    v->ex.has = 1; snprintf(v->ex.kicker, AT_STR, "%s", "FIGHTER"); snprintf(v->ex.title, AT_STR, "%s", "A FIGHTER WITH A LONG NAME INDEED");
    snprintf(v->ex.what, AT_TEXT, "%s", "A rule line that is a little too long to fit on two lines at the narrow width, so it must be clamped.");
    snprintf(v->ex.stepper_label, 16, "%s", "COSTUME"); snprintf(v->ex.stepper_text, 24, "%s", "1 / 4"); v->ex.stepper = 1;
    snprintf(v->ex.media_abbr, 3, "%s", "FO");
    v->cursor[0].active = 1; v->cursor[0].block = 0; v->cursor[0].index = 0; v->cursor[0].card = -1;
}

static int count_hits(int kind) { int i, n = 0; for (i = 0; i < HITS.n; i++) n += HITS.h[i].kind == kind; return n; }

static void rosters_at_three_widths(void)
{
    static const int sizes[] = { 26, 29, 60, 129 }, widths[] = { 640, 853, 1140 };
    int si, wi;
    for (si = 0; si < 4; si++) for (wi = 0; wi < 3; wi++) {
        AtSink s; AtRenderInfo info; int i, cells = 0, n = sizes[si];
        AtLayout L; AtSplit sp;
        build_css_screen(&SC, &V, POOL, n, n > 26 ? n - 26 : 0, 0x3);
        s = rec_sink();
        at_render_ex(&SC, &V, (float) widths[wi], 1000.0, 1, &O, &s, &HITS, &info);
        CHECK(info.capped == 0 && info.entries < AT_SCREEN_QUAD_WARN);                     /* the 129-tile worst case stays under 3,000 entries */
        CHECK(info.hits_dropped == 0);                                                       /* every visible tile is reachable by the mouse */
        CHECK(texts_legible());
        at_layout((float) widths[wi], AT_PRESET_NORMAL, &L); at_layout_split(&L, SC.n_tabs > 0, AT_BAND_CARDS, &sp);
        for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL) cells++;
        if (n == 26 || n == 29) CHECK(cells == n);                                          /* the vanilla and ACE-sized rosters fit without scrolling at every width */
        CHECK(cells > 0);
        if (n >= 60 && widths[wi] == 640) CHECK(cells < n);                                  /* the big ones scroll at 640, and draw only the visible rows */
        if (n == 129) CHECK(cells < n);                                                      /* 128 fighters scroll at every width */
        if (n == 60 && widths[wi] == 1140) CHECK(cells <= n);                                /* 60 may fit whole in a wide window */
        CHECK(count_hits(AT_HIT_CARD) == 4 && count_hits(AT_HIT_TAB) == SC.n_tabs);
        /* no hit rectangle leaves the canvas, and no tile overlaps the band, the tabs or the keys */
        for (i = 0; i < HITS.n; i++) {
            const AtHit *h = &HITS.h[i];
            CHECK(h->r.x >= -0.01f && h->r.x + h->r.w <= (float) widths[wi] + 0.01f && h->r.y >= 0.0f && h->r.y + h->r.h <= 480.0f + 0.01f);
            if (h->kind == AT_HIT_CELL) CHECK(h->r.y >= sp.grid.y && h->r.y + h->r.h <= sp.grid.y + sp.grid.h && h->r.x >= sp.grid.x && h->r.x + h->r.w <= sp.grid.x + sp.grid.w);
            if (h->kind == AT_HIT_CELL) CHECK(h->r.y + h->r.h <= sp.band.y - 12.0f + 0.5f);
            if (h->kind == AT_HIT_KEY) CHECK(h->r.y >= L.keys.y - 24.0f && h->r.y + h->r.h <= 480.0f);
        }
        if (n == 129 && widths[wi] == 1140) { int cols = 0; float y0 = -1; for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL) { if (y0 < 0) y0 = HITS.h[i].r.y; if (HITS.h[i].r.y == y0) cols++; } CHECK(cols >= 10 && cols <= 12); }   /* a wide window: more columns, not bigger tiles */
        if (wi > 0) { AtHit *h = &HITS.h[0]; int k; for (k = 0; k < HITS.n; k++) if (HITS.h[k].kind == AT_HIT_CELL) { h = &HITS.h[k]; break; } CHECK(h->r.w <= 56.01f && h->r.w >= 24.0f); }
    }
}
static void hits_follow_layout(void)
{
    /* the same tile sits at a different x at 853 than at 640 (the arranged rectangle, not the authored one); a click on it maps to the same index */
    AtSink s; AtRect a = { 0, 0, 0, 0 }, b = { 0, 0, 0, 0 }; int i, ia = -1, ib = -1;
    build_css_screen(&SC, &V, POOL, 29, 3, 0x1);
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].b == 5) { a = HITS.h[i].r; ia = i; }
    s = rec_sink(); at_render(&SC, &V, 1706.0f, 1000.0, 1, &O, &s, &HITS);
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].b == 5) { b = HITS.h[i].r; ib = i; }
    CHECK(ia >= 0 && ib >= 0 && (a.x != b.x));
    CHECK(at_hit_test(&HITS, b.x + b.w * 0.5f, b.y + b.h * 0.5f) == ib);
    CHECK(at_hit_test(&HITS, -1000.0f, -1000.0f) == -1);                                    /* a pointer off the picture does nothing */
    /* mouse events over the arranged rectangles: hover moves the focus to that tile, a click focuses and accepts */
    { AtMouse m; AtEvent ev[8]; int n; memset(&m, 0, sizeof m);
      at_mouse_events(&m, 1.0f, 1.0f, 0, 0, &HITS, ev, 8);                                   /* the first sample only primes */
      n = at_mouse_events(&m, b.x + b.w * 0.5f, b.y + b.h * 0.5f, 0, 0, &HITS, ev, 8);
      CHECK(n == 1 && ev[0].type == AT_EV_FOCUS && ev[0].a == 0 && ev[0].b == 5);
      n = at_mouse_events(&m, b.x + b.w * 0.5f, b.y + b.h * 0.5f, 1, 0, &HITS, ev, 8);
      CHECK(n == 2 && ev[0].type == AT_EV_FOCUS && ev[1].type == AT_EV_ACCEPT); }
}
static void placeholder_when_no_art(void)
{
    AtSink s; int i;
    build_css_screen(&SC, &V, POOL, 29, 0, 0x1);
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(REC.ni == 0);                                                                       /* no image drawn: every tile is the frame with its letters */
    CHECK(find_text("AA") != NULL && find_text("BH") != NULL);
    for (i = 0; i < 29; i++) POOL[i].tex = 4 + i;                                            /* with art: one image per tile and the explainer's portrait */
    V.ex.media_tex = 99;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(REC.ni == 29 + 1 && find_text("AA") == NULL);
}
static void style_of_the_screen(void)
{
    AtSink s; AtLayout L; AtSplit sp; int i, panes = 0;
    build_css_screen(&SC, &V, POOL, 29, 3, 0xF);
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    at_layout(640.0f, AT_PRESET_NORMAL, &L); at_layout_split(&L, 1, AT_BAND_CARDS, &sp);
    CHECK(sty_chamfer(sp.grid, 8.0f, AT_C_GROUND) == 0);                                    /* the pane's two cut corners show the ground */
    CHECK(sty_chamfer(L.explainer, 8.0f, AT_C_GROUND) == 0);
    for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == AT_C_PLATE) panes++;
    CHECK(panes >= 2);                                                                       /* one primary plate and one supporting plate: everything else quiet (4.5) */
    /* every text sits inside the place it belongs to: the explainer's inside the explainer, the cards' inside their cards */
    for (i = 0; i < REC.nt; i++) {
        const RecText *t = &REC.t[i];
        float w = fake_width(NULL, t->role, t->s), left = t->x - (t->align == AT_ALIGN_RIGHT ? w : t->align == AT_ALIGN_CENTER ? w * 0.5f : 0.0f);
        CHECK(left >= -0.01f && left + w <= 640.0f + 0.01f && t->base <= 480.0f);
        if (strcmp(t->s, "A FIGHTER WITH A LONG NAME INDEED") == 0 || strncmp(t->s, "A FIGHTER WITH A", 16) == 0) { /* either the explainer's title or a card's name: each inside its box */
            int in_expl = left >= L.explainer.x - 0.01f && left + w <= L.explainer.x + L.explainer.w + 0.01f && t->base <= L.explainer.y + L.explainer.h;
            int in_card = t->base >= sp.band.y - 3.0f && t->base <= sp.band.y + sp.band.h && left >= sp.band.x - 0.01f && left + w <= sp.band.x + sp.band.w + 0.01f;
            CHECK(in_expl || in_card);
        }
    }
    /* each card's own texts are inside its slot */
    { float cw = (sp.band.w - 24.0f) / 4.0f; int k;
      for (k = 0; k < 4; k++) {
          AtRect slot; AtSink s2; AtLayout L2;
          slot.x = sp.band.x + (float) k * (cw + 8.0f); slot.y = sp.band.y; slot.w = cw; slot.h = sp.band.h;
          s2 = rec_sink(); at_poly_rect(&s2, 0, 0, 640, 480, AT_C_GROUND); at_part_port_card(&s2, &O, slot, &SC.cards[k], 0); CHECK(sty_text_inside(slot, 0) == -1 && sty_chamfer(slot, 5.0f, AT_C_GROUND) == 0);
          (void) L2;
      } }
    /* the keys strip is below the band and the tiles, inside the canvas */
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_KEY) CHECK(HITS.h[i].r.y >= sp.band.y + sp.band.h);
    /* the title row is the trail: no block title (a native block with no title takes no title row), so the first tile row is at the top of the pane */
    { float top = 1e9f; for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].r.y < top) top = HITS.h[i].r.y; CHECK(top <= sp.grid.y + 12.0f + 0.01f); }
}
static AtRect tile_area(int idx)
{
    AtRect r = { 0, 0, 0, 0 }; int i;
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].b == idx) { r = HITS.h[i].r; r.x -= 8; r.y -= 8; r.w += 16; r.h += 16; }
    return r;
}
static void focus_on_a_cell_has_three_cues(void)
{
    AtSink s; StySig a, b; AtRect area;
    build_css_screen(&SC, &V, POOL, 29, 0, 0x1);
    V.cursor[0].active = 0; s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS); area = tile_area(7); a = sty_sig_in(0, area);
    V.cursor[0].active = 1; V.cursor[0].index = 7; s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS); b = sty_sig_in(0, area);
    CHECK(sty_focus_cues(a, b) == 3);
    /* a locked tile (an unavailable fighter) is hatched or worded, not only dimmed, and still shows its letters */
    POOL[9].flags = AT_CELL_LOCKED; V.cursor[0].active = 0;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(count_color(AT_C_DIM) >= 4);
    POOL[9].flags = AT_CELL_DISABLED; POOL[10].flags = AT_CELL_DISABLED;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(count_color(AT_C_ROSE) >= 2);                                                      /* each disabled tile is struck through */
}
static void long_roster_scroll_follows_cursor(void)
{
    AtSink s; AtRenderInfo info; int i, seen = 0;
    build_css_screen(&SC, &V, POOL, 129, 5, 0x1);
    V.cursor[0].index = 128;                                                                 /* the Random tile, at the very end */
    s = rec_sink(); at_render_ex(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS, &info);
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].b == 128) seen = 1;
    CHECK(seen && info.capped == 0 && info.hits_dropped == 0);
    V.cursor[0].index = 0; seen = 0;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].b == 0) seen = 1;
    CHECK(seen);
    /* a cursor on a card, not the grid: the grid does not scroll to a stale index */
    V.cursor[0].card = 1; V.cursor[0].index = 128; V.scroll = 0;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    for (i = 0, seen = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].b == 0) seen = 1;
    CHECK(seen);
}
static void entries_stay_bounded(void)
{
    /* the worst case: 128 fighters and four cursors, with art, at the widest window */
    AtSink s; AtRenderInfo info; int i;
    build_css_screen(&SC, &V, POOL, 129, 5, 0xF);
    for (i = 0; i < 129; i++) POOL[i].tex = i;
    for (i = 0; i < 4; i++) { V.cursor[i].active = 1; V.cursor[i].block = 0; V.cursor[i].index = 20; V.cursor[i].card = -1; }
    s = rec_sink(); at_render_ex(&SC, &V, 1706.0f, 1000.0, 1, &O, &s, &HITS, &info);
    CHECK(info.capped == 0 && info.entries < AT_SCREEN_QUAD_WARN && info.hits_dropped == 0);
}

int main(void)
{
    rosters_at_three_widths(); hits_follow_layout(); placeholder_when_no_art(); style_of_the_screen(); focus_on_a_cell_has_three_cues();
    long_roster_scroll_follows_cursor(); entries_stay_bounded();
    ATLAS_DONE("atlas select render");
}
