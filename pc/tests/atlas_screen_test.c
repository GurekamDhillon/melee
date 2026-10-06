#include <stdlib.h>
#include "atlas_check.h"
#include "../platform/gw_ui_screen.h"

static AtvArena *A;
static int S(int t, const char *k, const char *v) { return atv_set(A, t, k, atv_str(A, v)); }
static int N(int t, const char *k, double v) { return atv_set(A, t, k, atv_num(A, v)); }
static int B(int t, const char *k, int v) { return atv_set(A, t, k, atv_bool(A, v)); }

static int cell(const char *id, const char *name, int model, int locked, int empty)
{
    int c = atv_table(A), f;
    S(c, "id", id); S(c, "name", name);
    if (model >= 0) N(c, "model", model);
    f = atv_table(A);
    if (locked) B(f, "locked", 1);
    if (empty) B(f, "empty", 1);
    atv_set(A, c, "flags", f);
    return c;
}

static int block(const char *id, const char *title, int cols, int stones)
{
    int b = atv_table(A), cells = atv_table(A);
    S(b, "id", id); S(b, "title", title); N(b, "cols", cols);
    if (stones) S(b, "kind", "stones");
    atv_set(A, b, "cells", cells);
    return b;
}

static int key(const char *btn, const char *label, int fn)
{
    int k = atv_table(A);
    atv_push(A, k, atv_str(A, btn));
    atv_push(A, k, fn >= 0 ? atv_fn(A, fn) : atv_str(A, label));
    return k;
}

/* the Envoy bag, as the Lua description would arrive */
static int bag(const char *id)
{
    int root = atv_table(A), trail = atv_table(A), prim = atv_table(A), blocks = atv_table(A), b, cells, i, ex = atv_table(A), keys = atv_table(A), on = atv_table(A), alt = atv_table(A);
    char buf[24];
    S(root, "id", id);
    S(trail, "title", "YOUR DRIVES"); atv_push(A, trail, atv_str(A, "SOLO")); atv_push(A, trail, atv_str(A, "ENVOY")); atv_set(A, root, "trail", trail);
    S(prim, "kind", "grid");
    b = block("eq", "EQUIPPED", 6, 0); cells = atv_get(A, b, "cells"); S(b, "count", "5 / 6");
    for (i = 0; i < 6; i++) { snprintf(buf, sizeof buf, "eq:%d", i + 1); atv_push(A, cells, i == 5 ? cell(buf, "Locked slot", -1, 1, 0) : cell(buf, "Drive", 100 + i, 0, 0)); }
    atv_push(A, blocks, b);
    b = block("bag", "BAG", 4, 0); cells = atv_get(A, b, "cells");
    for (i = 0; i < 4; i++) { snprintf(buf, sizeof buf, "bag:%d", i + 1); atv_push(A, cells, i == 3 ? cell(buf, "Empty slot", -1, 0, 1) : cell(buf, "Drive", 110 + i, 0, 0)); }
    atv_push(A, blocks, b);
    b = block("key", "KEYSTONE", 6, 1); cells = atv_get(A, b, "cells"); S(b, "note", "One held");
    for (i = 0; i < 3; i++) { snprintf(buf, sizeof buf, "key:%d", i + 1); atv_push(A, cells, cell(buf, "Keystone", -1, 0, 0)); }
    atv_push(A, blocks, b);
    atv_set(A, prim, "blocks", blocks); atv_set(A, root, "primary", prim);
    S(ex, "width", "narrow"); atv_set(A, ex, "provide", atv_fn(A, 7)); atv_set(A, root, "explainer", ex);
    atv_push(A, keys, key("A", 0, 11)); atv_push(A, keys, key("X", "Discard", -1)); atv_push(A, keys, key("Y", "More", -1)); atv_push(A, keys, key("B", "Close", -1));
    atv_set(A, root, "keys", keys);
    atv_set(A, on, "accept", atv_fn(A, 21)); atv_set(A, on, "back", atv_fn(A, 22)); atv_set(A, alt, "X", atv_fn(A, 23)); atv_set(A, on, "alt", alt);
    atv_set(A, on, "focus", atv_fn(A, 24)); atv_set(A, root, "on", on);
    S(root, "input", "feed"); N(root, "port", 2);
    return root;
}

static void parse_bag(void)
{
    static AtScreen sc;
    char err[160];
    int refs[24], n;
    CHECK(at_screen_from_val(A, bag("envoy.bag"), "envoy", &sc, err, sizeof err));
    CHECK_STR(sc.id, "envoy.bag"); CHECK_STR(sc.title, "YOUR DRIVES");
    CHECK(sc.n_parents == 2); CHECK_STR(sc.parent[0], "SOLO"); CHECK_STR(sc.parent[1], "ENVOY");
    CHECK(sc.primary == AT_PRIMARY_GRID && sc.n_blocks == 3 && sc.preset == AT_PRESET_NARROW);
    CHECK(sc.blocks[0].n == 6 && sc.blocks[1].n == 4 && sc.blocks[2].n == 3);
    CHECK(sc.blocks[2].stones == 1 && sc.blocks[0].stones == 0);
    CHECK_STR(sc.blocks[0].count, "5 / 6"); CHECK_STR(sc.blocks[2].note, "One held");
    CHECK(sc.blocks[0].cells[5].flags & AT_CELL_LOCKED); CHECK(sc.blocks[1].cells[3].flags & AT_CELL_EMPTY);
    CHECK(sc.blocks[0].cells[0].model == 100 && sc.blocks[0].cells[5].model == AT_NO_MODEL);
    CHECK(sc.n_keys == 4 && sc.keys[0].btn == 'A' && sc.keys[0].fn_label == 11 && sc.keys[1].btn == 'X');
    CHECK_STR(sc.keys[1].label, "Discard");
    CHECK(sc.fn_provide == 7 && sc.fn_accept == 21 && sc.fn_back == 22 && sc.fn_alt[0] == 23 && sc.fn_alt[1] == -1 && sc.fn_focus == 24 && sc.fn_change == -1);
    CHECK(sc.input_feed == 1 && sc.port == 2);
    CHECK(!at_screen_wants_pad(&sc));                                  /* feed: the engine must not poll the pad as well */
    n = at_screen_fn_refs(&sc, refs, 24);
    CHECK(n == 6);                                                      /* provide, accept, back, alt X, focus, key A's label */
}

static void errors(void)
{
    static AtScreen sc;
    char err[160];
    int root, prim, blocks, b;
    root = atv_table(A);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "id") != NULL);
    CHECK(!at_screen_from_val(A, atv_str(A, "x"), NULL, &sc, err, sizeof err));
    CHECK(!at_screen_from_val(A, bag("other.bag"), "envoy", &sc, err, sizeof err));        /* a mod's ids start with its own id */
    CHECK(strstr(err, "envoy.") != NULL);
    root = atv_table(A); S(root, "id", "m.x"); prim = atv_table(A); S(prim, "kind", "tiles"); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "tiles") != NULL);
    root = atv_table(A); S(root, "id", "m.x"); prim = atv_table(A); S(prim, "kind", "grid"); blocks = atv_table(A);
    for (b = 0; b < 7; b++) atv_push(A, blocks, block("b", "B", 2, 0));
    atv_set(A, prim, "blocks", blocks); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "blocks") != NULL);
    root = atv_table(A); S(root, "id", "m.x"); prim = atv_table(A); S(prim, "kind", "grid"); blocks = atv_table(A);
    b = block("b", "B", 2, 0); atv_push(A, atv_get(A, b, "cells"), cell("c", "one", -1, 0, 0)); atv_push(A, atv_get(A, b, "cells"), cell("c", "two", -1, 0, 0));
    atv_push(A, blocks, b); atv_set(A, prim, "blocks", blocks); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "duplicate") != NULL);
    root = bag("m.bag"); { int k = atv_get(A, root, "keys"); atv_push(A, k, key("Q", "Nope", -1)); }
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "button") != NULL);
    root = atv_table(A); S(root, "id", "m.x"); prim = atv_table(A); S(prim, "kind", "grid"); blocks = atv_table(A);
    b = block("b", "B", 20, 0);                                                              /* more columns than cells per block */
    atv_push(A, blocks, b); atv_set(A, prim, "blocks", blocks); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "cols") != NULL);
}

static void list_screen(void)
{
    static AtScreen sc;
    char err[160];
    int root = atv_table(A), prim = atv_table(A), items = atv_table(A), it, v;
    S(root, "id", "demo.list"); S(prim, "kind", "list");
    it = atv_table(A); S(it, "id", "sync"); S(it, "label", "Sync"); v = atv_table(A); S(v, "kind", "toggle"); B(v, "on", 1); atv_set(A, it, "value", v); atv_push(A, items, it);
    it = atv_table(A); S(it, "id", "vol"); S(it, "label", "Volume"); v = atv_table(A); S(v, "kind", "slider"); N(v, "min", 0); N(v, "max", 100); N(v, "value", 30); atv_set(A, it, "value", v); atv_push(A, items, it);
    it = atv_table(A); S(it, "id", "off"); S(it, "label", "Closed"); B(it, "disabled", 1); atv_push(A, items, it);
    atv_set(A, prim, "items", items); atv_set(A, root, "primary", prim);
    CHECK(at_screen_from_val(A, root, "demo", &sc, err, sizeof err));
    CHECK(sc.primary == AT_PRIMARY_LIST && sc.n_items == 3 && sc.preset == AT_PRESET_NONE);
    CHECK(sc.items[0].vkind == AT_VAL_TOGGLE && sc.items[0].on == 1);
    CHECK(sc.items[1].vkind == AT_VAL_SLIDER && sc.items[1].vmax == 100 && sc.items[1].vval == 30);
    CHECK(sc.items[2].flags & AT_CELL_DISABLED);
}

static void explainer(void)
{
    AtExplainer e;
    char err[160], big[400];
    int t = atv_table(A), with = atv_table(A), from = atv_table(A), media = atv_table(A);
    S(t, "kicker", "BAG CELL 1"); S(t, "title", "KINDLING"); S(t, "what", "Your hits set the target Burning for 3 s.");
    N(media, "model", 5); N(media, "ring", 6); atv_set(A, t, "media", media);
    atv_push(A, with, atv_num(A, 8)); atv_push(A, with, atv_num(A, 9)); atv_set(A, t, "with", with);
    S(from, "text", "Depth 0, Fire"); atv_set(A, t, "from", from);
    CHECK(at_explainer_from_val(A, t, &e, err, sizeof err));
    CHECK(e.has && e.media_model == 5 && e.media_ring == 6 && e.n_with == 2 && e.with_model[1] == 9 && e.warn == 0);
    CHECK_STR(e.title, "KINDLING"); CHECK_STR(e.from_text, "Depth 0, Fire");
    memset(big, 'a', sizeof big - 1); big[sizeof big - 1] = '\0';
    t = atv_table(A); S(t, "title", "T"); S(t, "what", big);
    CHECK(at_explainer_from_val(A, t, &e, err, sizeof err));
    CHECK(strlen(e.what) == AT_TEXT - 1 && e.warn == 1);                /* clamped, and said so */
    CHECK(at_explainer_from_val(A, -1, &e, err, sizeof err) && e.has == 0);   /* nothing to explain: the pane is empty */
}

static AtFocusPos P(int b, int i) { AtFocusPos p; p.block = b; p.index = i; return p; }

static void refocus(void)
{
    static AtScreen sc;
    char err[160];
    AtFocusBlock fb[AT_MAX_BLOCKS];
    AtFocusPos p;
    CHECK(at_screen_from_val(A, bag("envoy.bag"), "envoy", &sc, err, sizeof err));
    CHECK(at_screen_focus_blocks(&sc, fb) == 3);
    CHECK(fb[0].row0 == 0 && fb[1].row0 == 1 && fb[2].row0 == 2 && fb[1].cols == 4);
    CHECK_STR(at_screen_cell_id(&sc, P(1, 2)), "bag:3"); CHECK_STR(at_screen_block_id(&sc, 1), "bag");
    CHECK(at_screen_cell_id(&sc, P(1, 9)) == NULL && at_screen_cell_id(&sc, P(-1, 0)) == NULL);
    p = at_screen_refocus(&sc, "bag", "bag:3", P(1, 2)); CHECK(p.block == 1 && p.index == 2);        /* the same cell stays */
    sc.blocks[1].n = 3;                                                                              /* a discard shrank the bag */
    p = at_screen_refocus(&sc, "bag", "bag:4", P(1, 3));
    CHECK(p.block == 1 && p.index == 2);                                                              /* the gone cell: the same block, clamped */
    sc.blocks[1].n = 0;                                                                               /* the whole block went */
    p = at_screen_refocus(&sc, "bag", "bag:1", P(1, 0));
    CHECK(p.block == 0 && p.index == 0);                                                              /* never "nothing" while a cell exists */
    p = at_screen_refocus(&sc, NULL, NULL, P(-1, -1)); CHECK(p.block == 0 && p.index == 0);
    sc.blocks[0].n = 0; sc.blocks[2].n = 0;
    p = at_screen_refocus(&sc, "eq", "eq:1", P(0, 0)); CHECK(p.block == -1 && p.index == -1);          /* truly empty */
}

static void accept_semantics(void)
{
    static AtScreen sc, ls;
    char err[160];
    int root = atv_table(A), prim = atv_table(A), items = atv_table(A), it;
    CHECK(at_screen_from_val(A, bag("envoy.bag"), "envoy", &sc, err, sizeof err));
    CHECK(at_cell_accepts(&sc, P(0, 5)));                              /* a LOCKED slot still reaches the handler (it says what it is) */
    CHECK(at_cell_accepts(&sc, P(1, 3)));                              /* an EMPTY slot too */
    CHECK(!at_cell_accepts(&sc, P(7, 0)));                             /* a position that is not there accepts nothing */
    S(root, "id", "demo.list"); S(prim, "kind", "list");
    it = atv_table(A); S(it, "id", "a"); S(it, "label", "A"); atv_push(A, items, it);
    it = atv_table(A); S(it, "id", "b"); S(it, "label", "B"); B(it, "disabled", 1); atv_push(A, items, it);
    atv_set(A, prim, "items", items); atv_set(A, root, "primary", prim);
    CHECK(at_screen_from_val(A, root, NULL, &ls, err, sizeof err));
    CHECK(at_cell_accepts(&ls, P(0, 0)));
    CHECK(!at_cell_accepts(&ls, P(0, 1)));                             /* a DISABLED row is focusable but never fires */
    CHECK(at_screen_wants_pad(&ls));                                   /* default: the engine polls the pad */
}

int main(void)
{
    A = (AtvArena *) malloc(sizeof *A);
    atv_init(A); parse_bag();
    atv_init(A); errors();
    atv_init(A); list_screen();
    atv_init(A); explainer();
    atv_init(A); refocus();
    atv_init(A); accept_semantics();
    free(A);
    ATLAS_DONE("atlas screen");
}
