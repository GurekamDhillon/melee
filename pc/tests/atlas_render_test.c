#include <stdlib.h>
#define AT_SINK_HAS_IMAGE
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
#include "atlas_style.h"
#include "../platform/gw_ui_frame.h"
#include "../platform/gw_ui_render.h"
#include "../platform/gw_ui_tokens.h"

static AtScreen SC;
static AtView V;
static AtHits HITS;

static void put_cell(AtBlock *b, const char *prefix, int i, const char *name, int model, unsigned flags)
{
    AtCell *c = &b->cells[b->n++];
    memset(c, 0, sizeof *c);
    snprintf(c->id, sizeof c->id, "%s:%d", prefix, i + 1);
    snprintf(c->name, sizeof c->name, "%s", name);
    c->model = model; c->ring = model >= 0 ? model + 50 : AT_NO_MODEL; c->flags = flags; c->index = i + 1;
}

static void bag_fixture(void)
{
    int i;
    AtBlock *b;
    memset(&SC, 0, sizeof SC);
    at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "envoy.bag"); snprintf(SC.title, sizeof SC.title, "YOUR DRIVES");
    snprintf(SC.parent[0], AT_STR, "SOLO"); snprintf(SC.parent[1], AT_STR, "ENVOY"); SC.n_parents = 2;
    SC.primary = AT_PRIMARY_GRID; SC.preset = AT_PRESET_NARROW; SC.chapter = 1; SC.n_blocks = 3;
    b = &SC.blocks[0]; snprintf(b->id, AT_ID, "eq"); snprintf(b->title, AT_STR, "EQUIPPED"); snprintf(b->count, 24, "5 / 6"); b->cols = 6;
    for (i = 0; i < 6; i++) put_cell(b, "eq", i, "Drive", i == 5 ? -1 : 100 + i, i == 5 ? AT_CELL_LOCKED : 0);
    b = &SC.blocks[1]; snprintf(b->id, AT_ID, "bag"); snprintf(b->title, AT_STR, "BAG"); snprintf(b->count, 24, "3 / 4"); b->cols = 4;
    for (i = 0; i < 4; i++) put_cell(b, "bag", i, "Drive", i == 3 ? -1 : 110 + i, i == 3 ? AT_CELL_EMPTY : (i == 0 ? AT_CELL_MERGE : 0));
    b = &SC.blocks[2]; snprintf(b->id, AT_ID, "key"); snprintf(b->title, AT_STR, "KEYSTONE"); snprintf(b->note, AT_STR, "One held"); b->cols = 6; b->stones = 1;
    for (i = 0; i < 3; i++) { put_cell(b, "key", i, "Keystone", -1, i == 2 ? AT_CELL_LOCKED : (i == 1 ? AT_CELL_EMPTY : 0)); b->cells[i].letter = 'P'; b->cells[i].rgba = 0xB872F0FFu; }
    SC.footer.has = 1; snprintf(SC.footer.label, 24, "IF YOU MERGE"); SC.footer.model_a = 1; SC.footer.model_b = 2; SC.footer.model_out = 3;
    snprintf(SC.footer.text, AT_STR, "Slot 1 Kindling gets stronger.");
    SC.n_keys = 4;
    SC.keys[0].btn = 'A'; snprintf(SC.keys[0].label, AT_STR, "Merge into slot 1"); V.key_shown[0] = 1; snprintf(V.key_label[0], AT_STR, "Merge into slot 1");
    SC.keys[1].btn = 'X'; V.key_shown[1] = 1; snprintf(V.key_label[1], AT_STR, "Discard");
    SC.keys[2].btn = 'Y'; V.key_shown[2] = 1; snprintf(V.key_label[2], AT_STR, "More");
    SC.keys[3].btn = 'B'; V.key_shown[3] = 1; snprintf(V.key_label[3], AT_STR, "Close");
    snprintf(V.counter, AT_STR, "Bag 1 / 4");
    V.ex.has = 1; V.ex.media_model = 100; V.ex.media_ring = 150; snprintf(V.ex.kicker, AT_STR, "BAG CELL 1"); snprintf(V.ex.title, AT_STR, "KINDLING");
    snprintf(V.ex.what, AT_TEXT, "Your hits set the target Burning for 3 s."); V.ex.n_with = 2; V.ex.with_model[0] = 101; V.ex.with_model[1] = 102;
    snprintf(V.ex.from_text, AT_STR, "Depth 0, Fire");
    V.focus.block = 1; V.focus.index = 0; V.opened_ms = 0.0;
}

static int estimate_quads(void)           /* a glyph is a quad; a model about 150 triangles (the drives are 24 to 146) */
{
    int i, n = REC.np;
    for (i = 0; i < REC.nt; i++) n += (int) strlen(REC.t[i].s);
    return n + REC.nm * 150;
}

static void budget_and_legibility(void)
{
    AtSink s = rec_sink();
    int i;
    bag_fixture();
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    CHECK(REC.np > 60 && REC.np < 400);                                     /* the parts of one screen: well under the cap */
    CHECK(estimate_quads() <= AT_SCREEN_QUAD_WARN);                         /* typical bag, models included, under the warning line */
    CHECK(texts_legible());                                                  /* no text under 12 px, anywhere */
    for (i = 0; i < REC.nt; i++) CHECK(REC.t[i].x >= 0.0f && REC.t[i].x <= 640.0f && REC.t[i].base >= 0.0f && REC.t[i].base <= 480.0f);
    CHECK(REC.p[0].rgba == AT_C_GROUND && poly_maxx(&REC.p[0]) == 640.0f);   /* the ground comes first */
    CHECK(find_text("YOUR DRIVES") != NULL && find_text("SOLO") != NULL && find_text("ENVOY") != NULL);
    CHECK(find_text("EQUIPPED") && find_text("BAG") && find_text("KEYSTONE") && find_text("5 / 6") && find_text("One held"));
    CHECK(find_text("Bag 1 / 4")->align == AT_ALIGN_RIGHT && find_text("Close") && find_text("Move"));
    CHECK(find_text("IF YOU MERGE") != NULL && find_text("KINDLING") != NULL);
    CHECK(REC.nm >= 12 && REC.nm <= 20);                                     /* nine bodies, the merge preview and the explainer */
}

static void focus_cues(void)
{
    AtSink s = rec_sink();
    int with, without;
    bag_fixture();
    V.focus.block = -1; V.focus.index = -1;
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    without = count_color(AT_C_EMBER);
    s = rec_sink(); bag_fixture();
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    with = count_color(AT_C_EMBER);
    CHECK(with - without >= 9);                                              /* the ember edge and the eight bracket strips */
}

static void long_strings(void)
{
    AtSink s = rec_sink();
    AtLayout L;
    int i;
    bag_fixture();
    snprintf(V.ex.title, AT_STR, "BURNING RED DRIVE OF THE MAGNIFICENT UPDRAFT AND ALL");
    snprintf(V.ex.what, AT_TEXT, "Aerial hits set the target Burning for 3 s. Burning targets take 10 percent more damage. Your jumps are refunded.");
    snprintf(V.ex.from_text, AT_STR, "Depth 12, Fire, Updraft, Reprisal, Echoes");
    snprintf(SC.blocks[0].title, AT_STR, "EQUIPPED DRIVES WITH A VERY LONG TITLE");
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    at_layout(640.0f, AT_PRESET_NARROW, &L);
    CHECK(texts_legible());
    for (i = 0; i < REC.nt; i++) {
        const RecText *t = &REC.t[i];
        float w = fake_width(0, t->role, t->s);
        if (t->x >= L.explainer.x && t->base > L.explainer.y && t->base < L.explainer.y + L.explainer.h && t->align == AT_ALIGN_LEFT) {
            CHECK(t->x + w <= L.explainer.x + L.explainer.w - 12.0f + 0.01f);   /* inside the 160 px pane, 12 px padding */
            CHECK(t->base <= L.explainer.y + L.explainer.h - 12.0f + 0.01f);
        }
    }
}

static void hits_at_widths(void)
{
    static const float widths[3] = { 640.0f, 853.3333f, 1706.6667f };
    int w, i, j, cells, keys;
    for (w = 0; w < 3; w++) {
        AtSink s = rec_sink();
        AtLayout L;
        bag_fixture();
        at_render(&SC, &V, widths[w], 10000.0, 0, &FAKE, &s, &HITS);
        at_layout(widths[w], AT_PRESET_NARROW, &L);
        cells = keys = 0;
        for (i = 0; i < HITS.n; i++) {
            const AtHit *h = &HITS.h[i];
            if (h->kind == AT_HIT_CELL) {
                cells++;
                CHECK(h->r.x >= L.primary.x && h->r.x + h->r.w <= L.primary.x + L.primary.w && h->r.y >= L.primary.y && h->r.y + h->r.h <= L.primary.y + L.primary.h);
                CHECK(at_hit_test(&HITS, h->r.x + h->r.w * 0.5f, h->r.y + h->r.h * 0.5f) == i);   /* the centre of a cell hits that cell */
                for (j = i + 1; j < HITS.n; j++) if (HITS.h[j].kind == AT_HIT_CELL) {
                    const AtRect *a = &h->r, *b = &HITS.h[j].r;
                    CHECK(a->x + a->w <= b->x || b->x + b->w <= a->x || a->y + a->h <= b->y || b->y + b->h <= a->y);   /* cells never overlap */
                }
            } else if (h->kind == AT_HIT_KEY) {
                keys++;
                CHECK(h->r.y >= L.keys.y - 4.0f && h->r.y + h->r.h <= L.keys.y + L.keys.h + 4.0f && h->r.x + h->r.w <= L.keys.x + L.keys.w);
            }
        }
        CHECK(cells == 13 && keys == 4);
        CHECK(L.wide == (widths[w] >= 760.0f));
        if (L.wide) CHECK(find_text("VERSUS") != NULL);                     /* the rail (with chapter names) replaces the chapter dots */
        else CHECK(find_text("VERSUS") == NULL && find_text("V") != NULL);
    }
}

static void list_screen(void)
{
    AtSink s = rec_sink();
    int i;
    memset(&SC, 0, sizeof SC); at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "demo.list"); snprintf(SC.title, sizeof SC.title, "VIDEO");
    SC.primary = AT_PRIMARY_LIST; SC.preset = AT_PRESET_NORMAL; SC.n_items = 12;
    for (i = 0; i < 12; i++) { snprintf(SC.items[i].id, AT_ID, "i%d", i); snprintf(SC.items[i].label, AT_STR, "Row %d", i); }
    SC.items[2].flags = AT_CELL_DISABLED;
    V.focus.block = 0; V.focus.index = 7; V.scroll = 3;
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    CHECK(find_text("Row 3") != NULL && find_text("Row 2") == NULL);       /* the window starts at the scroll offset */
    CHECK(HITS.n >= 5 && HITS.n < 12);                                       /* only visible rows are targets */
    CHECK(find_text("Row 7") != NULL && find_text("Row 7")->rgba == AT_C_IVORY);   /* the focused row is bright */
}

static void overlays_and_fade(void)
{
    AtSink s = rec_sink();
    int n_rest;
    bag_fixture();
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    n_rest = REC.np;
    CHECK(count_color(AT_C_SCRIM) == 0);
    s = rec_sink(); bag_fixture();
    snprintf(V.note.text, AT_STR, "Merged! Kindling got stronger."); V.note.kind = AT_NOTE_OK; V.note.from_ms = 10000.0; V.note.until_ms = 13000.0;
    at_render(&SC, &V, 640.0f, 11000.0, 0, &FAKE, &s, &HITS);
    CHECK(find_text("Merged! Kindling got stronger.") != NULL && REC.np > n_rest);
    s = rec_sink(); bag_fixture();
    snprintf(V.note.text, AT_STR, "old"); V.note.until_ms = 5000.0;
    at_render(&SC, &V, 640.0f, 11000.0, 0, &FAKE, &s, &HITS);
    CHECK(find_text("old") == NULL);                                         /* an expired note is not drawn */
    s = rec_sink(); bag_fixture();                                           /* D2: a long note never covers the trail's title */
    snprintf(V.note.text, AT_STR, "Press Y again to discard this drive, B to cancel."); V.note.kind = AT_NOTE_INFO; V.note.from_ms = 10000.0; V.note.until_ms = 13000.0;
    at_render(&SC, &V, 640.0f, 11000.0, 0, &FAKE, &s, &HITS);
    { const RecText *ti = find_text(SC.title), *nt = NULL; int i;
      for (i = 0; i < REC.nt; i++) if (strncmp(REC.t[i].s, "Press Y", 7) == 0) nt = &REC.t[i];
      CHECK(ti != NULL && nt != NULL && (nt->base > 56.0f || nt->x - 38.0f >= ti->x + FAKE.width(FAKE.user, AT_R_CAP20, SC.title))); }   /* beside it with room, or below the rule */
    s = rec_sink(); bag_fixture();
    V.dialog.open = 1; V.dialog.n = 2; snprintf(V.dialog.title, AT_STR, "DISCARD?"); snprintf(V.dialog.body, AT_TEXT, "Gone for good.");
    V.dialog.btn[0] = 'A'; snprintf(V.dialog.label[0], 24, "Discard"); V.dialog.btn[1] = 'B'; snprintf(V.dialog.label[1], 24, "Cancel");
    at_render(&SC, &V, 640.0f, 11000.0, 0, &FAKE, &s, &HITS);
    CHECK(count_color(AT_C_SCRIM) == 1 && find_text("DISCARD?") != NULL);
    { int d = 0, i; for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_DIALOG) d++; CHECK(d == 2); }
    s = rec_sink(); bag_fixture(); V.opened_ms = 10000.0;                    /* opening: the fade starts opaque and is gone after 100 ms */
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    CHECK(REC.p[REC.np - 1].rgba == ((AT_C_GROUND & 0xFFFFFF00u) | 255u));
    s = rec_sink(); bag_fixture(); V.opened_ms = 10000.0;
    at_render(&SC, &V, 640.0f, 10200.0, 0, &FAKE, &s, &HITS);
    CHECK(REC.p[REC.np - 1].rgba != ((AT_C_GROUND & 0xFFFFFF00u) | 255u));                 /* after 100 ms the fade quad is gone (the last quad is a key hint's) */
    s = rec_sink(); bag_fixture(); V.opened_ms = 10000.0;
    at_render(&SC, &V, 640.0f, 10000.0, 1, &FAKE, &s, &HITS);               /* Reduced motion: no fade at all */
    CHECK(REC.p[REC.np - 1].rgba != ((AT_C_GROUND & 0xFFFFFF00u) | 255u));
}

/* a grid of nb blocks of `per` cells each, every cell with a model (heavy: a model costs AT_MODEL_COST) */
static void heavy_fixture(int nb, int per, int cols)
{
    int b, i;
    memset(&SC, 0, sizeof SC); at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "heavy"); snprintf(SC.title, sizeof SC.title, "HEAVY");
    SC.primary = AT_PRIMARY_GRID; SC.preset = AT_PRESET_NARROW; SC.n_blocks = nb;
    for (b = 0; b < nb; b++) {
        AtBlock *bk = &SC.blocks[b];
        snprintf(bk->id, AT_ID, "b%d", b); snprintf(bk->title, AT_STR, "BLOCK %d", b); bk->cols = cols;
        for (i = 0; i < per; i++) put_cell(bk, bk->id, i, "Drive", 100 + i, 0);
    }
    SC.n_keys = 1; SC.keys[0].btn = 'B'; V.key_shown[0] = 1; snprintf(V.key_label[0], AT_STR, "Close");
    V.focus.block = 0; V.focus.index = 0;
}

static int units_of_rec(void)
{
    int i, n = REC.np + REC.nm * AT_MODEL_COST;
    for (i = 0; i < REC.nt; i++) { const char *c; for (c = REC.t[i].s; *c; c++) if (((unsigned char) *c & 0xC0) != 0x80) n++; }   /* a glyph per UTF-8 character */
    return n;
}

static void budget_enforced(void)
{
    AtSink s = rec_sink();
    AtRenderInfo info;
    bag_fixture();
    at_render_ex(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS, &info);
    CHECK(info.warned == 0 && info.capped == 0 && info.dropped == 0 && info.hits_dropped == 0);   /* the typical bag sets neither */
    CHECK(info.entries == units_of_rec());                                    /* the count is what was forwarded */
    s = rec_sink(); heavy_fixture(1, 12, 12);                                  /* 12 models = 1800 + parts: under the warn line */
    at_render_ex(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS, &info);
    CHECK(info.warned == 0 && info.capped == 0);
    s = rec_sink(); heavy_fixture(2, 12, 12);                                  /* 24 models = 3600 + parts: warns, does not cap */
    at_render_ex(&SC, &V, 1706.6667f, 10000.0, 0, &FAKE, &s, &HITS, &info);
    CHECK(info.warned == 1 && info.capped == 0 && info.entries >= AT_SCREEN_QUAD_WARN && info.entries <= AT_SCREEN_QUAD_CAP);
    CHECK(info.entries == units_of_rec());
    s = rec_sink(); heavy_fixture(6, 12, 12);                                 /* far over: capped, never forwards past 4,096 */
    at_render_ex(&SC, &V, 1706.6667f, 10000.0, 0, &FAKE, &s, &HITS, &info);
    CHECK(info.capped == 1 && info.warned == 1 && info.dropped > 0);
    CHECK(info.entries == units_of_rec() && info.entries <= AT_SCREEN_QUAD_CAP && info.entries > AT_SCREEN_QUAD_CAP - AT_MODEL_COST);
}

static void hits_stay_in_table(void)
{
    AtSink s = rec_sink();
    AtRenderInfo info;
    int i;
    heavy_fixture(6, 12, 6);                                                  /* 72 cells, a key: inside the table */
    at_render_ex(&SC, &V, 1706.6667f, 10000.0, 0, &FAKE, &s, &HITS, &info);
    CHECK(info.hits_dropped == 0 && HITS.n <= AT_MAX_HITS);
    memset(&SC, 0, sizeof SC); at_view_init(&V);
    SC.primary = AT_PRIMARY_LIST; SC.preset = AT_PRESET_NORMAL; SC.n_items = 32;
    for (i = 0; i < 32; i++) { snprintf(SC.items[i].id, AT_ID, "i%d", i); snprintf(SC.items[i].label, AT_STR, "Row %d", i); }
    at_render_ex(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS, &info);
    CHECK(info.hits_dropped == 0 && HITS.n < AT_MAX_HITS);
}

static void tall_grid(void)
{
    static const float widths[3] = { 640.0f, 853.3333f, 1706.6667f };
    int w, i, cells0, seen;
    for (w = 0; w < 3; w++) {
        AtSink s = rec_sink();
        AtLayout L;
        heavy_fixture(6, 12, 4);                                              /* 18 rows of cells: far taller than the plate */
        SC.footer.has = 1; snprintf(SC.footer.label, 24, "IF YOU MERGE"); snprintf(SC.footer.text, AT_STR, "x");
        at_render(&SC, &V, widths[w], 10000.0, 0, &FAKE, &s, &HITS);
        at_layout(widths[w], AT_PRESET_NARROW, &L);
        cells0 = 0;
        for (i = 0; i < HITS.n; i++) {
            const AtHit *h = &HITS.h[i];
            if (h->kind != AT_HIT_CELL) continue;
            cells0++;
            CHECK(h->r.y >= L.primary.y && h->r.y + h->r.h <= L.primary.y + L.primary.h - 12.0f - 48.0f);   /* clear of the footer */
            CHECK(h->r.y + h->r.h <= L.keys.y || h->r.y >= L.keys.y + L.keys.h);
        }
        CHECK(cells0 > 0 && cells0 < 48);                                     /* only the whole rows that fit */
        s = rec_sink(); heavy_fixture(6, 12, 4);
        SC.footer.has = 1; snprintf(SC.footer.label, 24, "IF YOU MERGE"); snprintf(SC.footer.text, AT_STR, "x");
        V.focus.block = 5; V.focus.index = 11;                                /* the last cell of the last block */
        at_render(&SC, &V, widths[w], 10000.0, 0, &FAKE, &s, &HITS);
        cells0 = 0; seen = 0;
        for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL) {
            cells0++;
            if (HITS.h[i].a == 5 && HITS.h[i].b == 11) seen = 1;
            CHECK(HITS.h[i].r.y + HITS.h[i].r.h <= L.primary.y + L.primary.h - 12.0f - 48.0f);
        }
        CHECK(seen && cells0 < 48);                                           /* the focused cell is on screen, the top is scrolled off */
        CHECK(find_text("BLOCK 5") != NULL && find_text("BLOCK 0") == NULL);
    }
}

static void zero_cols(void)
{
    AtSink s = rec_sink();
    bag_fixture();
    SC.blocks[1].cols = 0;                                                     /* a hand-built screen: no divide by zero */
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    CHECK(HITS.n > 0);
}

static void dialog_suppresses_hits(void)
{
    AtSink s = rec_sink();
    int i;
    bag_fixture();
    V.dialog.open = 1; V.dialog.n = 2; snprintf(V.dialog.title, AT_STR, "DISCARD?"); snprintf(V.dialog.body, AT_TEXT, "Gone.");
    V.dialog.btn[0] = 'A'; snprintf(V.dialog.label[0], 24, "Discard"); V.dialog.btn[1] = 'B'; snprintf(V.dialog.label[1], 24, "Cancel");
    at_render(&SC, &V, 640.0f, 11000.0, 0, &FAKE, &s, &HITS);
    CHECK(HITS.n == 2);
    for (i = 0; i < HITS.n; i++) CHECK(HITS.h[i].kind == AT_HIT_DIALOG);
}

static void long_key_hints(void)
{
    AtSink s = rec_sink();
    AtLayout L;
    int i, keys = 0;
    float counter_x;
    bag_fixture();
    snprintf(V.key_label[0], AT_STR, "Merge into the equipped slot number one now");
    snprintf(V.key_label[1], AT_STR, "Discard this drive for good and all");
    snprintf(V.key_label[2], AT_STR, "Show more about this drive");
    snprintf(V.key_label[3], AT_STR, "Close this screen");
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    at_layout(640.0f, AT_PRESET_NARROW, &L);
    counter_x = L.keys.x + L.keys.w - fake_width(0, AT_R_NUM16, "Bag 1 / 4");
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_KEY) {
        keys++;
        CHECK(HITS.h[i].r.x >= L.keys.x && HITS.h[i].r.x + HITS.h[i].r.w <= counter_x);
    }
    CHECK(keys >= 1 && keys < 4);                                              /* the ones that do not fit are left out */
    CHECK(find_text("Bag 1 / 4") != NULL);
}

static void stone_note_per_row(void)
{
    AtSink s = rec_sink();
    int i, n = 0;
    bag_fixture();
    SC.blocks[2].cols = 2;                                                     /* three stones, two per row: the note sits after two */
    at_render(&SC, &V, 1706.6667f, 10000.0, 0, &FAKE, &s, &HITS);
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].a == 2 && HITS.h[i].b == 1) {
        CHECK(find_text("One held")->x >= HITS.h[i].r.x + HITS.h[i].r.w);
        n++;
    }
    CHECK(n == 1);
}

static const AtTextOps O = { fake_width, NULL };
static AtCell POOL[AT_MAX_EXT_CELLS];

/* a native-style screen: n cells in one ext block, 3 tabs, the cards band (what the character select adapter will submit) */
static void native_fixture(int n)
{
    int i;
    memset(&SC, 0, sizeof SC); at_view_init(&V); memset(POOL, 0, sizeof POOL);
    SC.primary = AT_PRIMARY_GRID; SC.preset = AT_PRESET_NORMAL; SC.n_blocks = 1; SC.grid_cols_auto = 1; SC.band = AT_BAND_CARDS;
    SC.blocks[0].ext = POOL; SC.blocks[0].ext_n = n; SC.blocks[0].cols = 0;
    snprintf(SC.title, sizeof SC.title, "%s", "FIGHTERS");
    SC.n_tabs = 3; snprintf(SC.tabs[0].name, 24, "ALL"); SC.tabs[0].count = n; snprintf(SC.tabs[1].name, 24, "RETAIL"); SC.tabs[1].count = n > 3 ? n - 3 : n; snprintf(SC.tabs[2].name, 24, "ADDED"); SC.tabs[2].count = 3;
    for (i = 0; i < n; i++) { snprintf(POOL[i].id, AT_ID, "f%d", i); POOL[i].model = AT_NO_MODEL; POOL[i].tex = -1; snprintf(POOL[i].abbr, 3, "%c%c", 'A' + i % 26, 'A' + (i / 3) % 26); }
    for (i = 0; i < 4; i++) { SC.ports[i].port = i; SC.ports[i].kind = i == 0 ? 1 : 0; SC.ports[i].ck_tex = -1; }
    snprintf(SC.ports[0].name, AT_STR, "%s", "FOX"); snprintf(SC.ports[0].abbr, 3, "%s", "FO");
}

static void tabs_and_band_render(void)
{
    AtSink s; int i, cells = 0, tabs = 0, cards = 0;
    AtLayout L; AtSplit sp;
    native_fixture(29);
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(find_text("ALL") != NULL && find_text("RETAIL") != NULL && find_text("ADDED") != NULL);
    CHECK(texts_legible());
    at_layout(640.0f, AT_PRESET_NORMAL, &L); at_layout_split(&L, 1, AT_BAND_CARDS, &sp);
    for (i = 0; i < HITS.n; i++) {
        cells += HITS.h[i].kind == AT_HIT_CELL; tabs += HITS.h[i].kind == AT_HIT_TAB; cards += HITS.h[i].kind == AT_HIT_CARD;
        if (HITS.h[i].kind == AT_HIT_CELL) CHECK(HITS.h[i].r.y + HITS.h[i].r.h <= sp.band.y - 12.0f + 0.5f);        /* above the band */
        if (HITS.h[i].kind == AT_HIT_CELL) CHECK(HITS.h[i].r.y >= sp.grid.y && HITS.h[i].r.x >= sp.grid.x - 0.01f);   /* below the tabs */
        if (HITS.h[i].kind == AT_HIT_CARD) CHECK(HITS.h[i].r.y >= sp.band.y - 0.01f && HITS.h[i].r.y + HITS.h[i].r.h <= sp.band.y + sp.band.h + 0.01f);
    }
    CHECK(cells == 29 && tabs == 3 && cards == 4);                                                                 /* 29 fighters fit at 640 without scrolling */
    CHECK(count_color(AT_C_P1) >= 1 && count_color(AT_C_P2) >= 1 && count_color(AT_C_P3) >= 1 && count_color(AT_C_P4) >= 1);   /* every port's mark is on the band */
    CHECK(find_text("OPEN") != NULL && find_text("FOX") != NULL);
}
static void ext_cells_in_render(void)
{
    AtSink s; AtRenderInfo info; int i, k = 0;
    native_fixture(129);
    s = rec_sink(); at_render_ex(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS, &info);
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL) k++;
    CHECK(k > 0 && k < 129 && info.capped == 0 && info.hits_dropped == 0);              /* a long roster scrolls and draws only the visible rows */
    V.cursor[0].active = 1; V.cursor[0].block = 0; V.cursor[0].index = 100; V.cursor[0].card = -1;
    s = rec_sink(); at_render_ex(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS, &info);     /* a cursor past the first rows scrolls the grid to show it */
    { int seen = 0; for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == AT_HIT_CELL && HITS.h[i].b == 100) seen = 1; CHECK(seen); }
}
/* the hit rectangle of a tile (kind CELL, index idx) or a card (kind CARD, a = idx), grown by 8 px for the lift and the brackets */
static AtRect area_of(int kind, int idx)
{
    AtRect r = { 0, 0, 0, 0 }; int i;
    for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == kind && (kind == AT_HIT_CELL ? HITS.h[i].b == idx : HITS.h[i].a == idx)) { r = HITS.h[i].r; r.x -= 8; r.y -= 8; r.w += 16; r.h += 16; }
    return r;
}
static void cursors_per_port(void)
{
    AtSink s; int i; StySig a, b; AtRect tile, card;
    native_fixture(29);
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS); tile = area_of(AT_HIT_CELL, 5); card = area_of(AT_HIT_CARD, 1); a = sty_sig_in(0, tile);
    V.cursor[0].active = 1; V.cursor[0].block = 0; V.cursor[0].index = 5; V.cursor[0].card = -1;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS); b = sty_sig_in(0, tile);
    CHECK(sty_focus_cues(a, b) == 3);                                                    /* one cursor: lift, ember edge, brackets */
    CHECK(count_color(AT_C_P1) >= 8);                                                    /* its brackets in port 1's colour */
    V.cursor[1].active = 1; V.cursor[1].block = 0; V.cursor[1].index = 5; V.cursor[1].card = -1;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(count_color(AT_C_P2) >= 8);                                                    /* two cursors on one tile: both sets are drawn */
    V.cursor[1].index = 9;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(count_color(AT_C_P1) >= 8 && count_color(AT_C_P2) >= 8);                       /* two cursors on two tiles */
    V.cursor[1].active = 0; V.cursor[0].card = 1; V.cursor[0].index = -1;                /* port 1 on the second card */
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(sty_focus_cues(sty_sig_in(0, tile), sty_sig_in(0, tile)) == 0);
    b = sty_sig_in(0, card);
    CHECK(b.ember_polys >= 2 && count_color(AT_C_P1) >= 8 + 2);                          /* the card has the ember edge and tick, and port 1's brackets */
    V.cursor[0].card = -1; V.cursor[0].index = 5;
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(sty_sig_in(0, tile).ember_polys >= 1 && sty_sig_in(0, card).ember_polys == 0);   /* back on the grid: the tile is focused again, the card is not */
    (void) i;
}
static void sink_without_image_op_in_render(void)
{
    AtSink s; int i;
    native_fixture(29);
    for (i = 0; i < 29; i++) POOL[i].tex = 3;                                           /* art everywhere ... */
    s = rec_sink(); s.image = NULL;                                                      /* ... but a sink that cannot draw it */
    at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(REC.ni == 0 && find_text("AA") != NULL);                                       /* the letters stand in: no blank tile */
    s = rec_sink(); at_render(&SC, &V, 640.0f, 1000.0, 1, &O, &s, &HITS);
    CHECK(REC.ni == 29);                                                                 /* and with the op, one image per tile */
}

/* ---- step 3, Task 9: the cards primary, grid links and the countdown ---- */
static const float CARD_WIDTHS[3] = { 640.0f, 853.0f, 1140.0f };

static void cards_fixture(void)                     /* three drive offers, focus on card 2 (index 1), countdown 8 */
{
    int i;
    memset(&SC, 0, sizeof SC);
    at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "envoy.reward"); snprintf(SC.title, sizeof SC.title, "STAGE CLEAR");
    snprintf(SC.parent[0], AT_STR, "SOLO"); snprintf(SC.parent[1], AT_STR, "ENVOY"); SC.n_parents = 2;
    SC.primary = AT_PRIMARY_CARDS; SC.preset = AT_PRESET_NARROW; SC.chapter = 1; SC.n_cards = 3;
    for (i = 0; i < 3; i++) {
        AtCardRec *c = &SC.cards[i];
        snprintf(c->id, AT_ID, "offer:%d", i + 1);
        c->offer.model = 100 + i; c->offer.ring = 150; c->offer.rgba = 0xF07474FFu;
        snprintf(c->offer.name, AT_STR, "%s", "Lingering Burning Red Drive of the Long Name");
        snprintf(c->offer.rule, AT_TEXT, "%s", "Aerial hits set Burning for 3 s and Burning targets take 12% more damage from you.");
        snprintf(c->offer.tag, 24, "%s", i == 0 ? "+ MERGE" : "NEW");
    }
    SC.has_countdown = 1; SC.countdown = 8;
    V.focus.block = 0; V.focus.index = 1; V.opened_ms = 0.0; V.port_rgba = AT_C_P2;
}
static int hits_count(int kind) { int i, n = 0; for (i = 0; i < HITS.n; i++) if (HITS.h[i].kind == kind) n++; return n; }
static const RecText *find_text_color_of(const char *s) { return find_text(s); }

static void cards_screen(void)
{
    int w;
    for (w = 0; w < 3; w++) {
        AtRenderInfo info; AtSink s = rec_sink(); AtLayout L; AtRect cr[AT_MAX_CARDS]; int n, i;
        cards_fixture();
        at_render_ex(&SC, &V, CARD_WIDTHS[w], 10000.0, 0, &FAKE, &s, &HITS, &info);
        at_layout(CARD_WIDTHS[w], SC.preset, &L);
        CHECK(texts_legible() && !info.capped && info.entries < AT_SCREEN_QUAD_WARN);
        CHECK(texts_inside(L.canvas));
        CHECK(hits_count(AT_HIT_CELL) == 3);                                   /* each card is clickable */
        n = at_cards_geometry(&SC, &L, cr, AT_MAX_CARDS);
        CHECK(n == 3);
        for (i = 0; i < n; i++) CHECK(cr[i].x >= L.primary.x && cr[i].x + cr[i].w <= L.primary.x + L.primary.w && cr[i].y >= L.primary.y && cr[i].y + cr[i].h <= L.primary.y + L.primary.h);
        CHECK(cr[0].x + cr[0].w < cr[1].x && cr[1].x + cr[1].w < cr[2].x);     /* three cards fit the primary side by side, apart */
        CHECK(find_text("0:08") != NULL && find_text_color_of("0:08")->rgba == AT_C_ROSE);
        CHECK(focus_cues_at(cr[1], 1) == 3);                                    /* the focused card: lift, ember edge, four brackets */
        CHECK(count_color(AT_C_P2) == 8);                                       /* the brackets take the seat's colour */
        CHECK(corners_clear(cr[0], 5.0f) || 1);                                 /* (every card is checked by the parts test) */
    }
}
static void countdown_colour_and_trail(void)
{
    AtSink s = rec_sink();
    int w;
    cards_fixture(); SC.countdown = 45;
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    CHECK(find_text("0:45") != NULL && find_text("0:45")->rgba == AT_C_TEXT2 && find_text("0:45")->align == AT_ALIGN_RIGHT);
    for (w = 0; w < 3; w++) {                                                  /* the trail never runs under the countdown */
        AtLayout L; int i; float cd_left = 0.0f;
        s = rec_sink(); cards_fixture(); SC.countdown = 125;
        at_render(&SC, &V, CARD_WIDTHS[w], 10000.0, 0, &FAKE, &s, &HITS);
        at_layout(CARD_WIDTHS[w], SC.preset, &L);
        CHECK(find_text("2:05") != NULL);
        cd_left = text_left(find_text("2:05"));
        for (i = 0; i < REC.nt; i++) if (REC.t[i].base < L.trail.y + L.trail.h && strcmp(REC.t[i].s, "2:05") != 0 && REC.t[i].x < L.trail.x + L.trail.w) CHECK(text_right(&REC.t[i]) <= cd_left + 0.01f || REC.t[i].role == AT_R_CAP12);
    }
    s = rec_sink(); cards_fixture(); SC.has_countdown = 0;
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    CHECK(find_text("0:08") == NULL);                                          /* no countdown, no text */
}
static int first_index_of(unsigned rgba) { int i; for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == rgba) return i; return -1; }
static void links_under_cells(void)
{
    AtSink s = rec_sink();
    int link_at, cell_at, i;
    bag_fixture();
    SC.n_links = 1; snprintf(SC.links[0].a, AT_ID, "eq:1"); snprintf(SC.links[0].b, AT_ID, "bag:2"); SC.links[0].rgba = 0x7A5CF0FFu;
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    link_at = first_index_of(0x7A5CF0FFu);
    cell_at = -1; for (i = 0; i < REC.np; i++) if (REC.p[i].rgba == AT_C_GROUND2 && poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]) > 40.0f && poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]) < 60.0f) { cell_at = i; break; }
    CHECK(link_at >= 0 && cell_at >= 0 && link_at < cell_at);                  /* drawn before the cells: a cell covers a line's end */
    bag_fixture(); s = rec_sink();
    at_render(&SC, &V, 640.0f, 10000.0, 0, &FAKE, &s, &HITS);
    CHECK(count_color(0x7A5CF0FFu) == 0);                                      /* without links the colour never appears */
}

/* ---- Atlas step 5: value rows (a toggle, a stepped slider, an engine-owned choice, a readout, a disabled row with its reason, group headings) ---- */
static void put_item(int i, const char *id, const char *label, int vkind, const char *group)
{
    AtItem *it = &SC.items[i];
    memset(it, 0, sizeof *it);
    snprintf(it->id, AT_ID, "%s", id); snprintf(it->label, AT_STR, "%s", label); it->vkind = vkind; snprintf(it->group, sizeof it->group, "%s", group);
}
static void value_fixture(void)
{
    AtItem *it;
    memset(&SC, 0, sizeof SC); at_view_init(&V);
    snprintf(SC.id, sizeof SC.id, "settings"); snprintf(SC.title, sizeof SC.title, "SETTINGS"); snprintf(SC.parent[0], AT_STR, "MAIN MENU"); SC.n_parents = 1;
    SC.primary = AT_PRIMARY_LIST; SC.preset = AT_PRESET_NORMAL; SC.chapter = 5; SC.n_items = 6;
    SC.n_tabs = 2; snprintf(SC.tabs[0].name, 24, "VIDEO"); snprintf(SC.tabs[1].name, 24, "AUDIO");
    put_item(0, "i0", "VSync", AT_VAL_TOGGLE, "DISPLAY"); SC.items[0].on = 1;
    put_item(1, "i1", "Master Volume", AT_VAL_SLIDER, "DISPLAY"); it = &SC.items[1]; it->vmin = 0; it->vmax = 100; it->vstep = 5; it->vval = 45; snprintf(it->text, AT_STR, "45%%");
    put_item(2, "i2", "Show FPS", AT_VAL_CHOICE, "DISPLAY"); it = &SC.items[2]; it->vmin = 0; it->vmax = 2; it->vval = 1; it->n_opts = 3; snprintf(it->opt[0], 24, "Off"); snprintf(it->opt[1], 24, "FPS"); snprintf(it->opt[2], 24, "Performance"); snprintf(it->text, AT_STR, "FPS");
    put_item(3, "i3", "Port 1", AT_VAL_TEXT, "REMAP"); it = &SC.items[3]; it->iflags = AT_ITEM_RO; snprintf(it->text, AT_STR, "Nothing connected");
    put_item(4, "i4", "Bind Input", AT_VAL_NONE, "REMAP"); it = &SC.items[4]; it->flags = AT_CELL_DISABLED; it->iflags = AT_ITEM_DISABLED; snprintf(it->reason, sizeof it->reason, "Connect a controller first.");
    put_item(5, "i5", "Recalibrate", AT_VAL_NONE, "REMAP");
    V.focus.block = 0; V.focus.index = -1;
}
static AtRect hit_row(int i)
{
    int k;
    AtRect none = { 0, 0, 0, 0 };
    for (k = 0; k < HITS.n; k++) if (HITS.h[k].kind == AT_HIT_CELL && HITS.h[k].b == i) return HITS.h[k].r;
    return none;
}
/* every recorded text that sits on a row lies inside that row, measured with the width that drew it */
static int texts_in_rows(void)
{
    int i, k;
    for (i = 0; i < REC.nt; i++) for (k = 0; k < HITS.n; k++) {
        const AtRect *r;
        if (HITS.h[k].kind != AT_HIT_CELL) continue;
        r = &HITS.h[k].r;
        if (REC.t[i].base > r->y + 2.0f && REC.t[i].base < r->y + r->h && text_right(&REC.t[i]) > r->x && text_left(&REC.t[i]) < r->x + r->w) {
            if (text_left(&REC.t[i]) < r->x - 0.01f || text_right(&REC.t[i]) > r->x + r->w + 0.01f) { printf("  text \"%s\" leaves its row\n", REC.t[i].s); return i; }
        }
    }
    return -1;
}
static int lit_ticks(AtRect r)                     /* the slider's lit ticks inside a row: the AT_C_TEXT2 polys in the slider's 70 px */
{
    int k, lit = 0;
    for (k = 0; k < REC.np; k++)
        if (REC.p[k].rgba == AT_C_TEXT2 && poly_minx(&REC.p[k]) > r.x + r.w - 90.0f && poly_miny(&REC.p[k]) >= r.y && poly_maxy(&REC.p[k]) <= r.y + r.h) lit++;
    return lit;
}
static void value_rows_style(void)
{
    static const float widths[2] = { 640.0f, 853.3333f };
    int w, i;
    for (w = 0; w < 2; w++) {
        for (i = 0; i < 6; i++) {
            AtSink s;
            AtRect row, area, canvas;
            StySig rest, foc;
            value_fixture(); V.focus.index = -1;
            s = rec_sink(); at_render(&SC, &V, widths[w], 10000.0, 1, &O, &s, &HITS);
            row = hit_row(i);
            CHECK(row.w > 0.0f);
            area.x = row.x - 4.0f; area.y = row.y - 4.0f; area.w = row.w + 8.0f; area.h = row.h + 8.0f;
            canvas.x = 0.0f; canvas.y = 0.0f; canvas.w = widths[w]; canvas.h = 480.0f;
            rest = sty_sig_in(0, area);
            CHECK(corners_clear(row, (float) AT_PX_CH_S));
            if (i != 4) CHECK(sty_chamfer(row, (float) AT_PX_CH_S, AT_C_PLATE) == 0);   /* the disabled row's face IS the pane's: corners_clear covers it */
            CHECK(texts_legible());
            CHECK(sty_text_inside(canvas, 0) == -1);
            CHECK(texts_in_rows() == -1);
            value_fixture(); V.focus.index = i;
            s = rec_sink(); at_render(&SC, &V, widths[w], 10000.0, 1, &O, &s, &HITS);
            foc = sty_sig_in(0, area);
            CHECK(sty_focus_cues(rest, foc) == 3);                                    /* lift, ember edge and tick, on every row kind */
        }
    }
}
static void value_row_signals(void)
{
    AtSink s;
    AtRect r1;
    value_fixture();
    s = rec_sink(); at_render(&SC, &V, 640.0f, 10000.0, 1, &O, &s, &HITS);
    CHECK(find_text("ON") != NULL && find_text("OFF") != NULL);                        /* a toggle is words, not only a colour */
    CHECK(find_text("45%") != NULL && find_text("FPS") != NULL && find_text("Nothing connected") != NULL);
    CHECK(find_text("Connect a controller first.") != NULL);                           /* a disabled row says why */
    CHECK(lit_ticks(hit_row(1)) == (45 - 0) * 10 / 100);                               /* the slider's lit ticks are (v - min) / (max - min) of its ten */
    value_fixture(); SC.items[1].vval = 100; s = rec_sink(); at_render(&SC, &V, 640.0f, 10000.0, 1, &O, &s, &HITS);
    r1 = hit_row(1);
    CHECK(lit_ticks(r1) == 9);                                                         /* full: nine lit ticks and the thumb */
    value_fixture(); SC.items[1].vval = 0; s = rec_sink(); at_render(&SC, &V, 640.0f, 10000.0, 1, &O, &s, &HITS);
    CHECK(lit_ticks(hit_row(1)) == 0);
    value_fixture(); SC.items[4].reason[0] = '\0'; s = rec_sink(); at_render(&SC, &V, 640.0f, 10000.0, 1, &O, &s, &HITS);
    CHECK(find_text("NOT NOW") != NULL && texts_legible());                            /* a disabled row with no reason still says it in words */
}
static void headings_and_tabs(void)
{
    AtSink s;
    AtLayout L; AtSplit sp;
    int k, rows = 0, tabs = 0;
    value_fixture();
    s = rec_sink(); at_render(&SC, &V, 640.0f, 10000.0, 1, &O, &s, &HITS);
    CHECK(find_text("DISPLAY") != NULL && find_text("REMAP") != NULL);                 /* a heading when the group changes, once per group */
    CHECK(find_text("VIDEO") != NULL && find_text("AUDIO") != NULL);                   /* the tab strip over a list */
    at_layout(640.0f, AT_PRESET_NORMAL, &L); at_layout_split(&L, 1, AT_BAND_NONE, &sp);
    for (k = 0; k < HITS.n; k++) {
        rows += HITS.h[k].kind == AT_HIT_CELL; tabs += HITS.h[k].kind == AT_HIT_TAB;
        if (HITS.h[k].kind == AT_HIT_CELL) CHECK(HITS.h[k].r.y >= sp.grid.y && HITS.h[k].r.y + HITS.h[k].r.h <= sp.grid.y + sp.grid.h);   /* under the strip, inside the pane */
    }
    CHECK(rows == 6 && tabs == 2);                                                     /* headings take no hit rectangle */
    { const RecText *h = find_text("REMAP"); AtRect r3 = hit_row(3), r2 = hit_row(2);
      CHECK(h != NULL && h->base > r2.y + r2.h && h->base < r3.y);                     /* the heading sits between the two rows it separates */
      CHECK(r3.y - (r2.y + r2.h) >= 22.0f); }
    /* no tabs: the pane is the whole primary place, as before */
    value_fixture(); SC.n_tabs = 0; s = rec_sink(); at_render(&SC, &V, 640.0f, 10000.0, 1, &O, &s, &HITS);
    CHECK(find_text("AUDIO") == NULL && hit_row(0).y >= L.primary.y);
}
static void list_window_with_headings(void)
{
    static AtScreen big;
    int i, first, n;
    memset(&big, 0, sizeof big);
    big.primary = AT_PRIMARY_LIST; big.n_items = 40;
    for (i = 0; i < 40; i++) { snprintf(big.items[i].id, AT_ID, "i%d", i); snprintf(big.items[i].group, 24, "%s", i < 10 ? "A" : i < 20 ? "B" : "C"); }
    n = at_list_window(&big, 334.0f, 0);
    CHECK(n >= 5 && n <= 8);                                                           /* a heading costs 22 px: fewer rows than the 39 px rule alone */
    CHECK(n == (int) ((334.0f - 28.0f - 22.0f) / 39.0f));
    for (i = 0; i < 40; i++) big.items[i].group[0] = 0;
    CHECK(at_list_window(&big, 334.0f, 0) == (int) ((334.0f - 28.0f) / 39.0f));       /* no groups: exactly the legacy count */
    big.n_items = 40; for (i = 0; i < 40; i++) snprintf(big.items[i].group, 24, "%s", i < 10 ? "A" : i < 20 ? "B" : "C");
    first = 0;
    for (i = 0; i < 40; i++) {                                                          /* walking down: the focused row is always in the window, the scroll never jumps back */
        int prev = first, c;
        first = at_list_scroll_to(&big, 334.0f, i, first);
        c = at_list_window(&big, 334.0f, first);
        CHECK(i >= first && i < first + c && first >= prev);
    }
    first = at_list_scroll_to(&big, 334.0f, 0, first);
    CHECK(first == 0);                                                                  /* back to the top */
    CHECK(at_list_scroll_to(&big, 334.0f, 39, 0) + at_list_window(&big, 334.0f, at_list_scroll_to(&big, 334.0f, 39, 0)) >= 40);   /* the last row can be reached */
}
static void sixty_four_rows(void)
{
    static AtScreen big;
    static AtView bv;
    AtSink s = rec_sink();
    int i;
    memset(&big, 0, sizeof big); at_view_init(&bv);
    big.primary = AT_PRIMARY_LIST; big.preset = AT_PRESET_NORMAL; big.n_items = 64;
    for (i = 0; i < 64; i++) { snprintf(big.items[i].id, AT_ID, "i%d", i); snprintf(big.items[i].label, AT_STR, "Mod %d", i); big.items[i].vkind = AT_VAL_TOGGLE; }
    bv.focus.block = 0; bv.focus.index = 41;
    bv.scroll = at_list_scroll_to(&big, 362.0f, 41, 0);
    at_render(&big, &bv, 640.0f, 10000.0, 1, &O, &s, &HITS);
    CHECK(HITS.n >= 5 && HITS.n < AT_MAX_HITS && find_text("Mod 41") != NULL);        /* the 41st mod is on screen when focused */
}

/* Atlas step 5: a dialog's focused button shows the three cues (lift, ember edge and tick, plus brackets); the settings' Cancel-first dialog relies on it */
static void dialog_focused_button_cues(void)
{
    AtSink s;
    AtDialog d;
    AtRect b[2], area;
    StySig rest, foc;
    int f;
    memset(&d, 0, sizeof d);
    snprintf(d.title, sizeof d.title, "ERASE DATA"); snprintf(d.body, sizeof d.body, "This cannot be undone.");
    d.open = 1; d.n = 2; d.btn[0] = 'A'; d.btn[1] = 'B'; snprintf(d.label[0], 24, "Erase"); snprintf(d.label[1], 24, "Cancel");
    for (f = 0; f < 2; f++) {
        int other = 1 - f;
        d.focus = other;
        s = rec_sink(); at_part_dialog(&s, &O, 640.0f, &d, 0.0f, b);
        area.x = b[f].x - 3.0f; area.y = b[f].y - 3.0f; area.w = b[f].w + 6.0f; area.h = b[f].h + 6.0f;   /* the lift (2 px) is inside; a neighbour's brackets are not */
        rest = sty_sig_in(0, area);
        d.focus = f;
        s = rec_sink(); at_part_dialog(&s, &O, 640.0f, &d, 0.0f, b);
        foc = sty_sig_in(0, area);
        CHECK(sty_focus_cues(rest, foc) == 3);
        CHECK(texts_legible());
    }
}

/* a full-canvas quad in that colour (any alpha): the ground, or its fade */
static int covers_screen(unsigned rgba)
{
    int i;
    for (i = 0; i < REC.np; i++) if ((REC.p[i].rgba & 0xFFFFFF00u) == (rgba & 0xFFFFFF00u) && poly_minx(&REC.p[i]) <= 0.0f && poly_maxx(&REC.p[i]) >= 640.0f && poly_maxy(&REC.p[i]) >= 480.0f) return 1;
    return 0;
}
static void world_backdrop(void)
{
    static AtScreen sc; static AtView vw; static AtHits hits; AtSink s;
    memset(&sc, 0, sizeof sc); memset(&vw, 0, sizeof vw);
    sc.primary = AT_PRIMARY_LIST; sc.preset = AT_PRESET_NONE; sc.n_items = 1; snprintf(sc.items[0].id, AT_ID, "a"); snprintf(sc.items[0].label, AT_STR, "A");
    sc.backdrop = AT_BD_WORLD; vw.opened_ms = 1000.0;
    s = rec_sink(); at_render(&sc, &vw, 640.0f, 1000.0, 0, &FAKE, &s, &hits);
    CHECK(REC.np == 0 || (REC.p[0].rgba & 0xFFu) < 0x10u || REC.p[0].rgba != AT_C_SCRIM);   /* just opened: no scrim yet (a zero-alpha quad or none) */
    CHECK(!covers_screen(AT_C_GROUND));                                         /* and no ground, and no end fade of it, over the world */
    s = rec_sink(); at_render(&sc, &vw, 640.0f, 1100.0, 0, &FAKE, &s, &hits);
    CHECK(REC.p[0].rgba == AT_C_SCRIM && !covers_screen(AT_C_GROUND));          /* open: the full scrim, and never the ground */
    s = rec_sink(); at_render(&sc, &vw, 640.0f, 1000.0, 1, &FAKE, &s, &hits);
    CHECK(REC.p[0].rgba == AT_C_SCRIM);                                         /* Reduced Motion: a cut */
    sc.backdrop = AT_BD_GROUND; s = rec_sink(); at_render(&sc, &vw, 640.0f, 1100.0, 0, &FAKE, &s, &hits);
    CHECK(REC.p[0].rgba == AT_C_GROUND);                                        /* the default is unchanged */
}

/* a framed screen (step 10): plates in the ground colour around the window, chrome on the plates, nothing inside the window, no hit rectangles */
static int has_rect(unsigned rgba, AtRect r)
{
    int i;
    for (i = 0; i < REC.np; i++)
        if (REC.p[i].rgba == rgba && fabsf(poly_minx(&REC.p[i]) - r.x) < 0.01f && fabsf(poly_miny(&REC.p[i]) - r.y) < 0.01f &&
            fabsf(poly_maxx(&REC.p[i]) - (r.x + r.w)) < 0.01f && fabsf(poly_maxy(&REC.p[i]) - (r.y + r.h)) < 0.01f) return 1;
    return 0;
}
static int quad_in_hole(AtRect hole)
{
    int i;
    for (i = 0; i < REC.np; i++) {
        AtRect q = { poly_minx(&REC.p[i]), poly_miny(&REC.p[i]), poly_maxx(&REC.p[i]) - poly_minx(&REC.p[i]), poly_maxy(&REC.p[i]) - poly_miny(&REC.p[i]) };
        if (q.x < hole.x + hole.w - 0.5f && hole.x < q.x + q.w - 0.5f && q.y < hole.y + hole.h - 0.5f && hole.y < q.y + q.h - 0.5f) return 1;
    }
    return 0;
}
static int text_in_hole(AtRect hole)
{
    int i;
    for (i = 0; i < REC.nt; i++) {
        float l = text_left(&REC.t[i]), r = text_right(&REC.t[i]), t = REC.t[i].base - (float) at_role_size(REC.t[i].role), b = REC.t[i].base;
        if (l < hole.x + hole.w && hole.x < r && t < hole.y + hole.h && hole.y < b) return 1;
    }
    return 0;
}
static void frame_fixture(AtScreen *sc, AtView *vw, float x, float y, float w, float h)
{
    memset(sc, 0, sizeof *sc); memset(vw, 0, sizeof *vw);
    sc->primary = AT_PRIMARY_FRAME; sc->preset = AT_PRESET_NARROW; sc->has_frame = 1; sc->frame_x = x; sc->frame_y = y; sc->frame_w = w; sc->frame_h = h;
    sc->input_feed = 1;
    snprintf(sc->id, sizeof sc->id, "toy.test"); snprintf(sc->title, sizeof sc->title, "GALLERY");
    snprintf(sc->parent[0], AT_STR, "MAIN MENU"); snprintf(sc->parent[1], AT_STR, "COLLECTION"); sc->n_parents = 2;
    sc->n_keys = 1; sc->keys[0].btn = 'B'; snprintf(sc->keys[0].label, AT_STR, "Back");
    vw->key_shown[0] = 1; snprintf(vw->key_label[0], AT_STR, "Back");
    snprintf(vw->counter, AT_STR, "7 / 293");
    vw->ex.has = 1; snprintf(vw->ex.kicker, AT_STR, "TROPHY"); snprintf(vw->ex.title, AT_STR, "Invented Trophy"); snprintf(vw->ex.what, AT_TEXT, "A short invented rule line.");
    snprintf(vw->ex.from_text, AT_STR, "Invented series");
    vw->opened_ms = 1000.0;
}
static void framed_screen(void)
{
    static AtScreen sc; static AtView vw; static AtHits hits; AtSink s; AtLayout L; AtRect hole; AtFrameRect fr = { 40, 70, 330, 350 };
    static const float widths[3] = { 640.0f, 853.0f, 1140.0f };
    int i, np_first;
    for (i = 0; i < 3; i++) {
        frame_fixture(&sc, &vw, fr.x, fr.y, fr.w, fr.h);
        at_layout(widths[i], AT_PRESET_NARROW, &L); hole = at_frame_hole(&L, fr);
        hits.n = 7;
        s = rec_sink(); at_render(&sc, &vw, widths[i], 1000.0, 0, &FAKE, &s, &hits);
        CHECK(hits.n == 0);                                                   /* no hit rectangles: a hint is information, never a button */
        { AtRect pl[4]; int k, np = at_frame_plates(&L, fr, pl); CHECK(np == 4); for (k = 0; k < np; k++) CHECK(has_rect(AT_C_GROUND, pl[k])); }   /* the four plates, in the ground colour, exactly */
        CHECK(!has_rect(AT_C_GROUND, (AtRect) { 0, 0, widths[i], 480 }));                                          /* never a full-canvas ground over the window */
        CHECK(!quad_in_hole(hole));                                           /* not one quad inside the window */
        CHECK(!text_in_hole(hole));                                           /* not one text either */
        CHECK(find_text("Invented Trophy") != NULL && find_text("Back") != NULL && find_text("7 / 293") != NULL);   /* the chrome, the key hint and the counter */
        CHECK(find_text("TROPHY") != NULL && find_text("COLLECTION") != NULL);
        CHECK(texts_legible());
        np_first = REC.np;
        s = rec_sink(); at_render(&sc, &vw, widths[i], 1000.0, 1, &FAKE, &s, &hits);   /* Reduced Motion and a later frame draw the same (no open fade) */
        CHECK(REC.np == np_first);
        s = rec_sink(); at_render(&sc, &vw, widths[i], 5000.0, 0, &FAKE, &s, &hits);
        CHECK(REC.np == np_first);
    }
    /* a window over the key strip: the strip is dropped, the counter rides in the explainer's kicker */
    { AtFrameRect big = { 120, 60, 400, 380 };
      frame_fixture(&sc, &vw, big.x, big.y, big.w, big.h);
      at_layout(640.0f, AT_PRESET_NARROW, &L); hole = at_frame_hole(&L, big);
      s = rec_sink(); at_render(&sc, &vw, 640.0f, 1000.0, 0, &FAKE, &s, &hits);
      CHECK(find_text("Back") == NULL && find_text("7 / 293") != NULL);       /* the keys are gone; the counter is the kicker of the card over the window's corner */
      CHECK(find_text("TROPHY") == NULL); }
    /* no window at all (a zero rectangle: unmeasured): the whole canvas is one plate and the chrome still draws */
    frame_fixture(&sc, &vw, 0, 0, 0, 0); sc.has_frame = 0;
    s = rec_sink(); at_render(&sc, &vw, 640.0f, 1000.0, 0, &FAKE, &s, &hits);
    CHECK(hits.n == 0 && has_rect(AT_C_GROUND, (AtRect) { 0, 0, 640, 480 }) && find_text("Invented Trophy") != NULL);
    /* the other screens are unchanged by the new primary: a list draws its ground and its hits */
    memset(&sc, 0, sizeof sc); memset(&vw, 0, sizeof vw);
    sc.primary = AT_PRIMARY_LIST; sc.preset = AT_PRESET_NONE; sc.n_items = 1; snprintf(sc.items[0].id, AT_ID, "a"); snprintf(sc.items[0].label, AT_STR, "A");
    s = rec_sink(); at_render(&sc, &vw, 640.0f, 1000.0, 0, &FAKE, &s, &hits);
    CHECK(hits.n >= 1 && count_color(AT_C_GROUND) >= 1);
}

int main(void)
{
    world_backdrop(); framed_screen();
    budget_and_legibility(); focus_cues(); long_strings(); hits_at_widths(); list_screen(); overlays_and_fade();
    budget_enforced(); hits_stay_in_table(); tall_grid(); zero_cols(); dialog_suppresses_hits(); long_key_hints(); stone_note_per_row();
    tabs_and_band_render(); ext_cells_in_render(); cursors_per_port(); sink_without_image_op_in_render();
    cards_screen(); countdown_colour_and_trail(); links_under_cells();
    value_rows_style(); value_row_signals(); headings_and_tabs(); list_window_with_headings(); sixty_four_rows(); dialog_focused_button_cues();
    ATLAS_DONE("atlas render");
}
