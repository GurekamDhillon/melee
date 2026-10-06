#include <stdlib.h>
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"
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

int main(void)
{
    budget_and_legibility(); focus_cues(); long_strings(); hits_at_widths(); list_screen(); overlays_and_fade();
    ATLAS_DONE("atlas render");
}
