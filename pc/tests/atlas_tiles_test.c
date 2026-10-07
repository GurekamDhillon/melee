/* atlas-tiles: the tiles primary (hubs, main menu), the More strip and the display screen. No game. */
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
#include "../platform/gw_ui_render.h"
#include "../platform/gw_ui_tokens.h"

static AtScreen SC; static AtView V; static AtHits H;
static const float WIDTHS[3] = { 640.0f, 853.0f, 1140.0f };

/* copied from atlas_parts_test.c:304-316: no vertex inside the top-left or bottom-right cut-away */
static int corners_clear(AtRect r, float c)
{
    int i, k;
    for (i = 0; i < REC.np; i++) for (k = 0; k < 4; k++) {
        float dx = REC.p[i].x[k] - r.x, dy = REC.p[i].y[k] - r.y, ex = r.x + r.w - REC.p[i].x[k], ey = r.y + r.h - REC.p[i].y[k];
        if (dx >= -0.01f && dy >= -0.01f && dx + dy < c - 0.01f) return 0;
        if (ex >= -0.01f && ey >= -0.01f && ex + ey < c - 0.01f) return 0;
    }
    return 1;
}
static int text_inside(AtRect r)
{
    int i;
    for (i = 0; i < REC.nt; i++) {
        const RecText *t = &REC.t[i];
        float w = fake_width(0, t->role, t->s);
        float l = t->align == AT_ALIGN_RIGHT ? t->x - w : t->align == AT_ALIGN_CENTER ? t->x - w * 0.5f : t->x;
        if (l < r.x - 0.5f || l + w > r.x + r.w + 0.5f) return 0;
    }
    return 1;
}
static void item(AtItem *it, const char *id, const char *label, const char *sub, const char *tag)
{
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id); snprintf(it->label, sizeof it->label, "%s", label);
    snprintf(it->sub, sizeof it->sub, "%s", sub); snprintf(it->tag, sizeof it->tag, "%s", tag);
}
static void solo_fixture(void)       /* the longest real SOLO hub: 7 tiles (Envoy is a mod tile) */
{
    static const char *L[7] = { "REGULAR MATCH", "EVENT MATCH", "STADIUM", "TRAINING", "LAB", "ENVOY", "MULTI-MAN MELEE" };
    int i;
    memset(&SC, 0, sizeof SC); at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "solo"); snprintf(SC.title, sizeof SC.title, "SOLO");
    SC.primary = AT_PRIMARY_TILES; SC.tile_cols = 2; SC.preset = AT_PRESET_NORMAL; SC.chapter = 1;
    for (i = 0; i < 7; i++) item(&SC.items[i], L[i], L[i], "", i >= 4 && i <= 5 ? "MOD" : "");
    SC.n_items = 7;
    V.focus.block = 0; V.focus.index = 5;
}
static void main_fixture(void)       /* five big rows with numerals, then More */
{
    static const char *L[5] = { "SOLO", "VERSUS", "ONLINE", "MODS", "SETTINGS" }, *N[5] = { "I", "II", "III", "IV", "V" };
    static const char *M[3] = { "COLLECTION", "DATA", "CREDITS" };
    int i;
    memset(&SC, 0, sizeof SC); at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "main"); snprintf(SC.title, sizeof SC.title, "MAIN MENU");
    SC.primary = AT_PRIMARY_TILES; SC.tile_cols = 1; SC.preset = AT_PRESET_NORMAL;
    for (i = 0; i < 5; i++) { item(&SC.items[i], L[i], L[i], "", ""); snprintf(SC.items[i].numeral, 6, "%s", N[i]); }
    SC.n_items = 5;
    for (i = 0; i < 3; i++) item(&SC.more[i], M[i], M[i], "", "");
    SC.n_more = 3;
    V.focus.block = 0; V.focus.index = 1;
}

static void tiles_at_three_widths(void)
{
    int w;
    for (w = 0; w < 3; w++) {
        AtSink s = rec_sink();
        AtRect t[16], m[4];
        AtLayout L;
        int n, i;
        solo_fixture();
        at_layout(WIDTHS[w], SC.preset, &L);
        at_render(&SC, &V, WIDTHS[w], 1e6, 0, &FAKE, &s, &H);
        CHECK(texts_legible());
        n = at_tiles_geometry(&SC, &L, t, 16, m, 4);
        CHECK(n == 7);
        for (i = 0; i < n; i++) {                                   /* inside primary, no overlap */
            int j;
            CHECK(t[i].x >= L.primary.x - 0.01f && t[i].x + t[i].w <= L.primary.x + L.primary.w + 0.01f);
            CHECK(t[i].y >= L.primary.y - 0.01f && t[i].y + t[i].h <= L.primary.y + L.primary.h + 0.01f);
            for (j = 0; j < i; j++) CHECK(t[i].x + t[i].w <= t[j].x + 0.01f || t[j].x + t[j].w <= t[i].x + 0.01f ||
                                          t[i].y + t[i].h <= t[j].y + 0.01f || t[j].y + t[j].h <= t[i].y + 0.01f);
        }
        for (i = 0; i < H.n; i++) if (H.h[i].kind == AT_HIT_CELL && H.h[i].a == 0) {   /* hits are the arranged rectangles */
            CHECK_NEAR(H.h[i].r.x, t[H.h[i].b].x); CHECK_NEAR(H.h[i].r.w, t[H.h[i].b].w);
        }
        CHECK(H.n == 7);
    }
}

static void tile_chamfers_clear(void)
{
    AtItem it; AtRect r = { 40.0f, 60.0f, 250.0f, 62.0f };
    int st;
    item(&it, "lab", "LAB", "", "MOD");
    for (st = AT_ST_REST; st <= AT_ST_SELECTED; st++) {
        AtSink s = rec_sink();
        AtRect pr = r;
        at_part_tile(&s, &FAKE, r, &it, st, 0);
        if (st == AT_ST_FOCUS) pr.y -= 2.0f;                         /* a focused plate lifts: its chamfers lift with it */
        if (st == AT_ST_PRESS) pr.y += 1.0f;
        CHECK(corners_clear(pr, (float) AT_PX_CH_S));                /* the tag, the tick and the bar stay out of the corners */
        CHECK(text_inside(r));
    }
    snprintf(it.numeral, 6, "III");
    {
        AtSink s = rec_sink(); AtRect pr = r; pr.y -= 2.0f;
        at_part_tile(&s, &FAKE, r, &it, AT_ST_FOCUS, 1);
        CHECK(corners_clear(pr, (float) AT_PX_CH_S));
        CHECK(text_inside(r));
    }
}

static void tile_three_cues(void)
{
    AtItem it; AtRect r = { 40.0f, 60.0f, 250.0f, 62.0f };
    AtSink s;
    float rest_top, focus_top;
    item(&it, "stadium", "STADIUM", "", "");
    s = rec_sink(); at_part_tile(&s, &FAKE, r, &it, AT_ST_REST, 0);
    CHECK(count_color(AT_C_EMBER) == 0);
    rest_top = poly_miny(&REC.p[0]);
    s = rec_sink(); at_part_tile(&s, &FAKE, r, &it, AT_ST_FOCUS, 0);
    focus_top = poly_miny(&REC.p[0]);
    CHECK_NEAR(rest_top - focus_top, 2.0f);                          /* cue 1: lift 2 px */
    CHECK(count_color(AT_C_EMBER) >= 2);                             /* cue 2: the front edge; cue 3: the 4 px tick */
    {
        int i, tick = 0, edge = 0;
        for (i = 0; i < REC.np; i++) {
            if (REC.p[i].rgba == AT_C_EMBER && poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]) <= 4.01f && poly_minx(&REC.p[i]) <= r.x + 0.01f) tick = 1;
            if (REC.p[i].rgba == AT_C_EMBER && poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]) > 100.0f) edge = 1;
        }
        CHECK(tick);
        CHECK(edge);
    }
    s = rec_sink(); at_part_tile(&s, &FAKE, r, &it, AT_ST_SELECTED, 0);
    CHECK(count_color(AT_C_JADE) >= 1);
    CHECK_NEAR(poly_miny(&REC.p[0]), rest_top);                      /* selected never moves */
    s = rec_sink(); at_part_tile(&s, &FAKE, r, &it, AT_ST_SELECTED, 0);
    CHECK(count_color(AT_C_EMBER) == 0);
}

static void long_strings_fit(void)
{
    int w;
    for (w = 0; w < 3; w++) {
        AtSink s;
        AtLayout L;
        AtRect t[16], m[4];
        solo_fixture();
        snprintf(SC.items[6].label, AT_STR, "%s", "ABCDEFGHIJKLMNOPQR");   /* a mod label at the 18-character cap */
        snprintf(SC.items[6].tag, 16, "MOD");
        at_layout(WIDTHS[w], SC.preset, &L);
        at_tiles_geometry(&SC, &L, t, 16, m, 4);
        s = rec_sink();
        at_part_tile(&s, &FAKE, t[6], &SC.items[6], AT_ST_FOCUS, 0);
        CHECK(text_inside(t[6]));
        CHECK(texts_legible());
        s = rec_sink();
        snprintf(SC.items[3].label, AT_STR, "%s", "MULTI-MAN MELEE");
        at_part_tile(&s, &FAKE, t[3], &SC.items[3], AT_ST_REST, 0);
        CHECK(text_inside(t[3]));
        CHECK(texts_legible());
    }
}

static void more_row_fits_640(void)
{
    AtSink s = rec_sink();
    AtLayout L;
    AtRect t[8], m[4];
    int i;
    main_fixture();
    at_layout(640.0f, SC.preset, &L);
    CHECK(at_tiles_geometry(&SC, &L, t, 8, m, 4) == 5);
    for (i = 0; i < 3; i++) {
        CHECK(m[i].y >= t[4].y + t[4].h - 0.01f);                       /* the More row is below the five rows */
        CHECK(m[i].y + m[i].h <= L.primary.y + L.primary.h + 0.01f);
    }
    at_part_more(&s, &FAKE, m[0], SC.more, 3, 1);
    CHECK(texts_legible());
    V.focus.block = 1; V.focus.index = 2;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1e6, 0, &FAKE, &s, &H);
    CHECK(find_text("CREDITS") != NULL);
    CHECK(texts_legible());
}

static void more_strip_cues_and_corners(void)
{
    AtSink s;
    AtRect r = { 40.0f, 300.0f, 300.0f, 34.0f };
    AtItem more[3];
    int i, tick = 0;
    item(&more[0], "collection", "COLLECTION", "", ""); item(&more[1], "data", "DATA", "", ""); item(&more[2], "credits", "CREDITS", "", "");
    s = rec_sink(); at_part_more(&s, &FAKE, r, more, 3, -1);
    CHECK(count_color(AT_C_EMBER) == 0);
    CHECK(corners_clear(r, (float) AT_PX_CH_S));
    CHECK(text_inside(r));
    for (i = 0; i < 3; i++) {
        s = rec_sink(); at_part_more(&s, &FAKE, r, more, 3, i);
        CHECK(corners_clear(r, (float) AT_PX_CH_S));
        CHECK(text_inside(r));
        CHECK(count_color(AT_C_EMBER) == 2);                          /* the front edge and the tick */
        CHECK(count_color(AT_C_LIFT) == 1);                           /* the lifted plate */
    }
    s = rec_sink(); at_part_more(&s, &FAKE, r, more, 3, 0);
    for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == AT_C_LIFT) { CHECK_NEAR(poly_miny(&REC.p[i]), r.y + 2.0f); tick = 1; }
    CHECK(tick);
}

static void focus_moves_between_tiles_and_more(void)
{
    AtFocusBlock fb[AT_MAX_BLOCKS];
    int nb;
    AtFocusPos p;
    main_fixture();
    nb = at_screen_focus_blocks(&SC, fb);
    CHECK(nb == 2);
    p.block = 0; p.index = 4;                                        /* SETTINGS, the last big row */
    p = at_focus_move(fb, nb, p, AT_DIR_DOWN, 1);
    CHECK(p.block == 1);                                             /* down from the last row enters More */
    p = at_focus_move(fb, nb, p, AT_DIR_RIGHT, 1);
    CHECK(p.block == 1 && p.index == 1);
    p = at_focus_move(fb, nb, p, AT_DIR_UP, 1);
    CHECK(p.block == 0);
    CHECK_STR(at_screen_cell_id(&SC, (AtFocusPos){ 1, 2 }), "CREDITS");
    CHECK_STR(at_screen_cell_id(&SC, (AtFocusPos){ 0, 0 }), "SOLO");
    solo_fixture();
    nb = at_screen_focus_blocks(&SC, fb);
    p.block = 0; p.index = 0;
    p = at_focus_move(fb, nb, p, AT_DIR_RIGHT, 1);
    CHECK(p.index == 1);                                              /* two columns */
}

static void many_tiles_scroll_by_rows(void)
{
    AtLayout L;
    AtRect t[AT_MAX_ITEMS], m[4];
    int i, vis = 0, rows = 0, scroll;
    solo_fixture();
    for (i = 7; i < 20; i++) { char id[8]; snprintf(id, sizeof id, "x%d", i); item(&SC.items[i], id, id, "", "MOD"); }
    SC.n_items = 20;
    at_layout(640.0f, SC.preset, &L);
    at_tiles_geometry_ex(&SC, &L, 0, t, AT_MAX_ITEMS, m, 4, &vis, &rows, NULL);
    CHECK(rows == 10 && vis >= 4 && vis < rows);
    CHECK(t[0].h >= 44.0f && t[19].h == 0.0f);                      /* the last row is scrolled out */
    scroll = at_tiles_scroll(&SC, &L, 19, 0);
    CHECK(scroll == rows - vis);
    at_tiles_geometry_ex(&SC, &L, scroll, t, AT_MAX_ITEMS, m, 4, &vis, &rows, NULL);
    CHECK(t[19].h >= 44.0f && t[0].h == 0.0f);
    CHECK(t[19].y + t[19].h <= L.primary.y + L.primary.h + 0.01f);
}

static void budget(void)
{
    AtSink s = rec_sink();
    AtRenderInfo info;
    solo_fixture();
    at_render_ex(&SC, &V, 1140.0f, 1e6, 0, &FAKE, &s, &H, &info);
    CHECK(info.entries < AT_SCREEN_QUAD_WARN);
    CHECK(info.capped == 0);
}

static void title_fixture(void)
{
    memset(&SC, 0, sizeof SC); at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "title");
    SC.primary = AT_PRIMARY_DISPLAY; SC.preset = AT_PRESET_NONE;
    snprintf(SC.hero, AT_STR, "GD'S MELEE"); snprintf(SC.prompt, AT_STR, "PRESS START");
    snprintf(SC.foot_left, 24, "v0.1.7"); snprintf(SC.foot_right, AT_STR, "Original menu art");
}
static void title_fits(void)
{
    int w;
    for (w = 0; w < 3; w++) {
        AtSink s = rec_sink();
        AtLayout L;
        const RecText *h, *p;
        title_fixture();
        at_layout(WIDTHS[w], AT_PRESET_NONE, &L);
        at_render(&SC, &V, WIDTHS[w], 1e6, 0, &FAKE, &s, &H);
        h = find_text("GD'S MELEE"); p = find_text("PRESS START");
        CHECK(h != NULL && p != NULL);
        CHECK(h != NULL && at_role_size(h->role) >= 44);                 /* hero or display */
        CHECK(texts_legible());
        CHECK(text_inside(L.canvas));
        CHECK(h != NULL && p != NULL && p->base > h->base);
        /* the longest wordmark the title could carry still fits its box, stepping down then truncating */
        snprintf(SC.hero, AT_STR, "%s", "A VERY LONG WORDMARK THAT CANNOT FIT ONE LINE AT ANY ROLE");
        s = rec_sink(); at_render(&SC, &V, WIDTHS[w], 1e6, 0, &FAKE, &s, &H);
        CHECK(texts_legible() && text_inside(L.canvas));
    }
}
static void title_prompt_chamfers_and_pulse(void)
{
    AtSink s;
    AtLayout L;
    AtRect pr;
    float a0, a1;
    int i, k;
    title_fixture();
    at_layout(853.0f, AT_PRESET_NONE, &L);
    s = rec_sink(); at_part_title(&s, &FAKE, &L, &SC, 0.0, 0, &pr);
    CHECK(pr.w > 0.0f && corners_clear(pr, (float) AT_PX_CH_S));      /* the START glyph and the label stay out of the cut corners */
    CHECK(count_color(AT_C_LIFT) == 2);                              /* the plate: lifted face (two quads) */
    a0 = a1 = -1.0f;
    for (k = 0; k < 2; k++) {
        s = rec_sink(); at_part_title(&s, &FAKE, &L, &SC, k == 0 ? 0.0 : 600.0, 0, &pr);
        for (i = 0; i < REC.np; i++) if ((REC.p[i].rgba & 0xFFFFFF00u) == (AT_C_EMBER & 0xFFFFFF00u) && poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]) > 100.0f && poly_maxy(&REC.p[i]) - poly_miny(&REC.p[i]) > 2.5f)
            { if (k == 0) a0 = (float) (REC.p[i].rgba & 0xFFu); else a1 = (float) (REC.p[i].rgba & 0xFFu); }
    }
    CHECK(a0 > a1 && a1 >= 0.59f * 255.0f);                          /* the edge pulses between 60 and 100 percent */
    s = rec_sink(); at_part_title(&s, &FAKE, &L, &SC, 600.0, 1, &pr);
    for (i = 0; i < REC.np; i++) if ((REC.p[i].rgba & 0xFFFFFF00u) == (AT_C_EMBER & 0xFFFFFF00u) && poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]) > 100.0f && poly_maxy(&REC.p[i]) - poly_miny(&REC.p[i]) > 2.5f)
        CHECK((REC.p[i].rgba & 0xFFu) == 0xFFu);                     /* reduced motion: no pulse */
}
static void title_has_no_hits(void)
{
    AtSink s = rec_sink();
    AtFocusBlock fb[AT_MAX_BLOCKS];
    title_fixture();
    at_render(&SC, &V, 853.0f, 1e6, 0, &FAKE, &s, &H);
    CHECK(H.n == 0);                                                     /* the mouse cannot act on the title */
    CHECK(at_screen_focus_blocks(&SC, fb) == 0);
    CHECK(at_screen_wants_pad(&SC) == 0);                                /* the title never reads the pad: retail does */
}

int main(void)
{
    tiles_at_three_widths(); tile_chamfers_clear(); tile_three_cues(); long_strings_fit();
    more_row_fits_640(); more_strip_cues_and_corners(); focus_moves_between_tiles_and_more(); many_tiles_scroll_by_rows(); budget();
    title_fits(); title_prompt_chamfers_and_pulse(); title_has_no_hits();
    ATLAS_DONE("atlas-tiles");
}
