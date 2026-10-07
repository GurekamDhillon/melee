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
    root = atv_table(A); S(root, "id", "m.x"); prim = atv_table(A); S(prim, "kind", "wheel"); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "wheel") != NULL);
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

static int one_cell_screen(const char *bid, const char *cid, int *cellout)
{
    int root = atv_table(A), prim = atv_table(A), blocks = atv_table(A), b = block(bid, "B", 2, 0), c = cell(cid, "one", -1, 0, 0);
    S(root, "id", "m.x"); S(prim, "kind", "grid");
    atv_push(A, atv_get(A, b, "cells"), c); atv_push(A, blocks, b);
    atv_set(A, prim, "blocks", blocks); atv_set(A, root, "primary", prim);
    if (cellout) *cellout = c;
    return root;
}

static void hardening(void)
{
    static AtScreen sc;
    char err[160];
    int root, c, i, prim, blocks, items, it, v;
    /* an arena that overflowed dropped entries: never convert it as if whole */
    root = bag("m.bag");
    while (!A->overflow) atv_num(A, 1.0);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "too large") != NULL);
    atv_init(A);
    /* numbers from a script: NaN, infinity and huge values never reach an int cast */
    root = one_cell_screen("b", "c", &c);
    N(c, "model", NAN); N(c, "ring", 1e30); N(c, "pips", -INFINITY); N(c, "index", 1e300); N(c, "color", NAN);
    CHECK(at_screen_from_val(A, root, NULL, &sc, err, sizeof err));
    CHECK(sc.blocks[0].cells[0].model == AT_NO_MODEL);
    CHECK(sc.blocks[0].cells[0].ring >= AT_NO_MODEL && sc.blocks[0].cells[0].ring <= 65535);
    CHECK(sc.blocks[0].cells[0].pips == 0 && sc.blocks[0].cells[0].rgba == 0);
    CHECK(sc.blocks[0].cells[0].index >= 0);
    atv_init(A);
    root = one_cell_screen("b", "c", &c);
    N(c, "model", -7); N(c, "color", 1e30);
    CHECK(at_screen_from_val(A, root, NULL, &sc, err, sizeof err));
    CHECK(sc.blocks[0].cells[0].model == AT_NO_MODEL);                    /* below -1 means none */
    CHECK(sc.blocks[0].cells[0].rgba == 0xFFFFFFFFu);
    atv_init(A);
    /* two blocks, one id */
    root = atv_table(A); prim = atv_table(A); blocks = atv_table(A);
    S(root, "id", "m.x"); S(prim, "kind", "grid");
    atv_push(A, blocks, block("b", "B1", 2, 0)); atv_push(A, blocks, block("b", "B2", 2, 0));
    atv_set(A, prim, "blocks", blocks); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "duplicate") != NULL);
    atv_init(A);
    /* an over-long id is an error, not a truncated twin */
    root = one_cell_screen("b", "cell-id-that-is-longer-than-the-field", NULL);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "too long") != NULL);
    atv_init(A);
    root = one_cell_screen("block-id-that-is-longer-than-the-field", "c", NULL);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "too long") != NULL);
    atv_init(A);
    root = atv_table(A); prim = atv_table(A); items = atv_table(A);
    S(root, "id", "m.x"); S(prim, "kind", "list");
    it = atv_table(A); S(it, "id", "item-id-that-is-longer-than-the-field"); S(it, "label", "L"); atv_push(A, items, it);
    atv_set(A, prim, "items", items); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "too long") != NULL);
    atv_init(A);
    /* slider bounds */
    for (i = 0; i < 3; i++) {
        root = atv_table(A); prim = atv_table(A); items = atv_table(A);
        S(root, "id", "m.x"); S(prim, "kind", "list");
        it = atv_table(A); S(it, "id", "v"); S(it, "label", "V"); v = atv_table(A); S(v, "kind", "slider");
        N(v, "min", i == 0 ? 10 : 0); N(v, "max", i == 0 ? 10 : 100); N(v, "value", i == 2 ? 500 : 30);
        atv_set(A, it, "value", v); atv_push(A, items, it); atv_set(A, prim, "items", items); atv_set(A, root, "primary", prim);
        if (i == 0) { CHECK(!at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(strstr(err, "slider") != NULL); }
        else { CHECK(at_screen_from_val(A, root, NULL, &sc, err, sizeof err)); CHECK(sc.items[0].vval == (i == 2 ? 100 : 30)); }
        atv_init(A);
    }
}

/* on.page(dir) and on.start() are read, on.change(item, value) is kept, and every one of their references is reported */
static void page_start_change(void)
{
    static AtScreen sc;
    char err[160];
    int root = atv_table(A), prim = atv_table(A), items = atv_table(A), it = atv_table(A), on = atv_table(A), refs[24], n;
    S(root, "id", "m.pc"); S(prim, "kind", "list"); S(it, "id", "a"); S(it, "label", "A"); atv_push(A, items, it);
    atv_set(A, prim, "items", items); atv_set(A, root, "primary", prim);
    atv_set(A, on, "page", atv_fn(A, 41)); atv_set(A, on, "start", atv_fn(A, 42)); atv_set(A, on, "change", atv_fn(A, 43));
    atv_set(A, root, "on", on);
    CHECK(at_screen_from_val(A, root, NULL, &sc, err, sizeof err));
    CHECK(sc.fn_page == 41 && sc.fn_start == 42 && sc.fn_change == 43);
    n = at_screen_fn_refs(&sc, refs, 24);
    CHECK(n == 3);
}

static int tiles_desc(int cols, int nitems, int nmore)
{
    int root = atv_table(A), prim = atv_table(A), items = atv_table(A), more = atv_table(A), i;
    S(root, "id", "m.hub");
    S(prim, "kind", "tiles"); N(prim, "cols", cols);
    for (i = 0; i < nitems; i++) { int it = atv_table(A); char id[8]; snprintf(id, sizeof id, "t%d", i); S(it, "id", id); S(it, "label", id); S(it, "tag", "MOD"); S(it, "numeral", "II"); atv_push(A, items, it); }
    for (i = 0; i < nmore; i++) { int it = atv_table(A); char id[8]; snprintf(id, sizeof id, "m%d", i); S(it, "id", id); S(it, "label", id); atv_push(A, more, it); }
    atv_set(A, prim, "items", items); if (nmore > 0) atv_set(A, prim, "more", more);
    atv_set(A, root, "primary", prim);
    return root;
}
static void tiles_screen(void)
{
    static AtScreen sc;
    char err[160];
    AtFocusBlock fb[AT_MAX_BLOCKS];
    CHECK(at_screen_from_val(A, tiles_desc(2, 5, 3), NULL, &sc, err, sizeof err));
    CHECK(sc.primary == AT_PRIMARY_TILES && sc.tile_cols == 2 && sc.n_items == 5 && sc.n_more == 3);
    CHECK_STR(sc.items[1].tag, "MOD"); CHECK_STR(sc.items[1].numeral, "II"); CHECK_STR(sc.more[2].id, "m2");
    CHECK(at_screen_focus_blocks(&sc, fb) == 2 && fb[0].cols == 2 && fb[1].cols == 3 && fb[1].row0 == 3);
    CHECK_STR(at_screen_block_id(&sc, 1), "more");
    { AtFocusPos old = { 1, 2 }, p = at_screen_refocus(&sc, "more", "m1", old); CHECK(p.block == 1 && p.index == 1); }
    { AtFocusPos p = { 1, 0 }; CHECK(at_cell_accepts(&sc, p)); }
    CHECK(!at_screen_from_val(A, tiles_desc(3, 5, 0), NULL, &sc, err, sizeof err)); CHECK(strstr(err, "tiles: cols must be 1 or 2") != NULL);
    CHECK(!at_screen_from_val(A, tiles_desc(2, 5, 5), NULL, &sc, err, sizeof err)); CHECK(strstr(err, "more") != NULL);
    CHECK(at_screen_from_val(A, tiles_desc(2, 2, 4), NULL, &sc, err, sizeof err));
    CHECK(sc.n_more == 4);
    CHECK(at_screen_from_val(A, tiles_desc(0, 3, 0), NULL, &sc, err, sizeof err) && at_screen_tile_cols(&sc) == 1);   /* 0: by count */
    CHECK(at_screen_from_val(A, tiles_desc(0, 4, 0), NULL, &sc, err, sizeof err) && at_screen_tile_cols(&sc) == 2);
}

static void ext_cells_never_copied(void)
{
    static AtScreen a, b; static AtCell pool[AT_MAX_EXT_CELLS];
    memset(&a, 0, sizeof a); memset(pool, 0, sizeof pool);
    a.primary = AT_PRIMARY_GRID; a.n_blocks = 1; a.blocks[0].ext = pool; a.blocks[0].ext_n = 200;
    CHECK(at_block_count(&a.blocks[0]) == 200);
    CHECK(at_block_cell(&a.blocks[0], 199) == &pool[199] && at_block_cell(&a.blocks[0], 200) == NULL && at_block_cell(&a.blocks[0], -1) == NULL);
    CHECK(at_screen_copy(&b, &a) == 0);                             /* a native screen is never copied: the pointer would outlive the pool */
    at_screen_clear_ext(&a); CHECK(a.blocks[0].ext == NULL && at_block_count(&a.blocks[0]) == 0);
    a.blocks[0].n = 3; CHECK(at_screen_copy(&b, &a) == 1 && b.blocks[0].n == 3);  /* an inline screen copies */
    CHECK(at_block_cell(&b.blocks[0], 2) == &b.blocks[0].cells[2] && at_block_cell(&b.blocks[0], 3) == NULL);
    /* ext_n past the pool is clamped: a bad count never indexes past AT_MAX_EXT_CELLS */
    a.blocks[0].ext = pool; a.blocks[0].ext_n = 100000; CHECK(at_block_count(&a.blocks[0]) == AT_MAX_EXT_CELLS);
    a.blocks[0].ext_n = -5; CHECK(at_block_count(&a.blocks[0]) == 0);
    /* the focus blocks read the native count */
    { AtFocusBlock fb[AT_MAX_BLOCKS]; AtFocusPos in = { 0, 128 }, out = { 0, 129 };
      a.blocks[0].ext_n = 129; a.blocks[0].cols = 8; CHECK(at_screen_focus_blocks(&a, fb) == 1 && fb[0].n == 129 && fb[0].cols == 8);
      CHECK(at_screen_cell_id(&a, in) != NULL && at_screen_cell_id(&a, out) == NULL); }
}
static void lua_door_unchanged(void)
{
    /* a Lua description with 13 more cells in one block is still refused, with the old message (documented limit 12) */
    char err[160]; static AtScreen s;
    int sc = bag("x.big"), bk = atv_at(A, atv_get(A, atv_get(A, sc, "primary"), "blocks"), 1), cells = atv_get(A, bk, "cells"), i;
    for (i = 0; i < 13; i++) { char id[16]; snprintf(id, sizeof id, "c%d", i); atv_push(A, cells, cell(id, "N", -1, 0, 0)); }
    CHECK(at_screen_from_val(A, sc, "x", &s, err, sizeof err) == 0 && strstr(err, "12") != NULL);
    /* a Lua cell has no disc art and no native storage: tex is -1 and ext is NULL, so a zeroed record can never draw texture 0 */
    sc = bag("x.ok");
    CHECK(at_screen_from_val(A, sc, "x", &s, err, sizeof err) == 1);
    CHECK(s.blocks[0].ext == NULL && s.blocks[0].cells[0].tex == -1 && s.n_tabs == 0 && s.band == AT_BAND_NONE && s.grid_cols_auto == 0);
}
static void tabs_and_cursors(void)
{
    AtView v; AtExplainer e; char err[8];
    at_view_init(&v);
    CHECK(v.tab == 0 && v.cursor[0].active == 0 && v.cursor[3].active == 0 && v.progress == 0 && v.ex.media_tex == -1);
    CHECK(v.cursor[0].card == -1 && v.cursor[0].block == 0 && v.cursor[0].index == -1);
    CHECK(at_explainer_from_val(A, -1, &e, err, sizeof err) == 1 && e.has == 0 && e.media_tex == -1 && e.stepper == 0);
    CHECK(AT_MAX_CELLS == 12 && AT_MAX_EXT_CELLS == 256 && AT_MAX_TABS == 6 && AT_MAX_CURSORS == 4);
}

static void pause_and_persist(void)
{
    char err[160]; AtScreen sc; int root, prim, items, it, grid;
    root = atv_table(A); S(root, "id", "envoy.pause"); S(root, "kind", "pause");
    prim = atv_table(A); S(prim, "kind", "list"); items = atv_table(A); it = atv_table(A); S(it, "id", "resume"); S(it, "label", "Resume"); atv_push(A, items, it);
    atv_set(A, prim, "items", items); atv_set(A, root, "primary", prim);
    CHECK(at_screen_from_val(A, root, "envoy", &sc, err, sizeof err) && sc.pause == 1 && sc.persist == 0);
    B(root, "persist", 1);
    CHECK(at_screen_from_val(A, root, "envoy", &sc, err, sizeof err) && sc.persist == 1);
    atv_init(A);                                                       /* a pause screen with a grid primary is refused */
    root = atv_table(A); S(root, "id", "envoy.pause"); S(root, "kind", "pause");
    prim = atv_table(A); S(prim, "kind", "grid"); grid = atv_table(A); atv_set(A, prim, "blocks", grid); atv_set(A, root, "primary", prim);
    CHECK(!at_screen_from_val(A, root, "envoy", &sc, err, sizeof err) && strstr(err, "a pause screen has a list primary") != NULL);
    atv_init(A);                                                       /* any other kind is an ordinary screen */
    root = atv_table(A); S(root, "id", "envoy.x"); prim = atv_table(A); S(prim, "kind", "list"); items = atv_table(A); it = atv_table(A); S(it, "id", "a"); atv_push(A, items, it);
    atv_set(A, prim, "items", items); atv_set(A, root, "primary", prim);
    CHECK(at_screen_from_val(A, root, "envoy", &sc, err, sizeof err) && sc.pause == 0);
}

static int card_desc(int cards_list, const char *id, const char *name, int disabled)
{
    int c = atv_table(A);
    S(c, "id", id); S(c, "name", name); S(c, "rule", "One short rule."); N(c, "model", 7);
    if (disabled) B(c, "disabled", 1);
    atv_push(A, cards_list, c);
    return c;
}
static int cards_root(int n, int disabled_second)
{
    int root = atv_table(A), prim = atv_table(A), cards = atv_table(A), i;
    char id[16];
    S(root, "id", "envoy.reward"); S(prim, "kind", "cards");
    for (i = 0; i < n; i++) { snprintf(id, sizeof id, "offer:%d", i + 1); card_desc(cards, id, "Drive", i == 1 && disabled_second); }
    atv_set(A, prim, "cards", cards); atv_set(A, root, "primary", prim);
    return root;
}
static void cards_and_countdown(void)
{
    char err[160]; AtScreen sc; AtFocusBlock fb[AT_MAX_BLOCKS]; AtFocusPos p;
    int root = cards_root(3, 1), nb;
    CHECK(at_screen_from_val(A, root, "envoy", &sc, err, sizeof err) && sc.primary == AT_PRIMARY_CARDS && sc.n_cards == 3);
    CHECK_STR(sc.cards[0].id, "offer:1"); CHECK(sc.cards[0].offer.model == 7 && sc.cards[1].disabled == 1);
    nb = at_screen_focus_blocks(&sc, fb);
    CHECK(nb == 1 && fb[0].n == 3 && fb[0].cols == 3);                       /* one block named cards, one row */
    CHECK_STR(at_screen_block_id(&sc, 0), "cards");
    p = at_focus_first(fb, nb);
    CHECK(p.block == 0 && p.index == 0 && strcmp(at_screen_cell_id(&sc, p), "offer:1") == 0);
    p = at_focus_move(fb, nb, p, AT_DIR_RIGHT, 1); CHECK(p.index == 1);
    p = at_focus_move(fb, nb, p, AT_DIR_RIGHT, 1); p = at_focus_move(fb, nb, p, AT_DIR_RIGHT, 1); CHECK(p.index == 0);   /* right wraps */
    p = at_focus_move(fb, nb, p, AT_DIR_LEFT, 1); CHECK(p.index == 2);                                                   /* and so does left */
    p.index = 1; CHECK(!at_cell_accepts(&sc, p));                              /* a disabled card does not accept */
    p.index = 0; CHECK(at_cell_accepts(&sc, p));
    p = at_screen_refocus(&sc, "cards", "offer:3", p); CHECK(p.index == 2);
    atv_init(A);
    CHECK(!at_screen_from_val(A, cards_root(5, 0), "envoy", &sc, err, sizeof err) && strstr(err, "at most 4 cards") != NULL);   /* a fifth card is refused */
    atv_init(A);
    { int r = cards_root(2, 0), c0 = atv_at(A, atv_get(A, atv_get(A, r, "primary"), "cards"), 1); S(c0, "tag_tone", "jade"); N(c0, "rgba", (double) 0xB872F0FFu); S(c0, "letter", "P");
      CHECK(at_screen_from_val(A, r, "envoy", &sc, err, sizeof err) && sc.cards[0].offer.tag_tone == 1 && sc.cards[0].offer.rgba == 0xB872F0FFu && sc.cards[0].offer.letter == 'P'); }
    atv_init(A);
    { int r = cards_root(2, 0), c1 = atv_at(A, atv_get(A, atv_get(A, r, "primary"), "cards"), 2); S(c1, "id", "offer:1");   /* a second id of the same name */
      (void) c1; CHECK(at_screen_from_val(A, r, "envoy", &sc, err, sizeof err)); }                                          /* the first id wins in a table: no twin */
    atv_init(A);
    { int r = cards_root(2, 0); N(r, "countdown", 45);
      CHECK(at_screen_from_val(A, r, "envoy", &sc, err, sizeof err) && sc.has_countdown == 1 && sc.countdown == 45);
      atv_init(A); r = cards_root(2, 0);
      CHECK(at_screen_from_val(A, r, "envoy", &sc, err, sizeof err) && sc.has_countdown == 0);
      atv_init(A); r = cards_root(2, 0); N(r, "countdown", -3);
      CHECK(at_screen_from_val(A, r, "envoy", &sc, err, sizeof err) && sc.countdown == 0); }
}
static void grid_links(void)
{
    char err[160]; AtScreen sc; int root, i, prim, links, l1, l2, l3;
    atv_init(A);
    root = bag("envoy.bag"); prim = atv_get(A, root, "primary"); links = atv_table(A);
    l1 = atv_table(A); S(l1, "a", "eq:2"); S(l1, "b", "bag:1"); N(l1, "rgba", (double) 0x7A5CF0FFu); atv_push(A, links, l1);
    l2 = atv_table(A); S(l2, "a", "eq:1"); S(l2, "b", "nope:9"); atv_push(A, links, l2);              /* a missing cell: skipped, counted */
    l3 = atv_table(A); S(l3, "a", "eq:1"); S(l3, "b", "eq:1"); atv_push(A, links, l3);                /* a cell to itself: skipped too */
    atv_set(A, prim, "links", links);
    CHECK(at_screen_from_val(A, root, "envoy", &sc, err, sizeof err) && sc.n_links == 1 && sc.links_skipped == 2);
    CHECK_STR(sc.links[0].a, "eq:2"); CHECK_STR(sc.links[0].b, "bag:1"); CHECK(sc.links[0].rgba == 0x7A5CF0FFu);
    atv_init(A);
    root = bag("envoy.bag"); prim = atv_get(A, root, "primary"); links = atv_table(A);
    for (i = 0; i < 17; i++) { l1 = atv_table(A); S(l1, "a", "eq:1"); S(l1, "b", "eq:2"); atv_push(A, links, l1); }
    atv_set(A, prim, "links", links);
    CHECK(!at_screen_from_val(A, root, "envoy", &sc, err, sizeof err) && strstr(err, "at most 16 links") != NULL);
}

/* headings are a property of the row below them (item.group), not rows: focus never lands on one, and Up from the first row wraps to the last */
static void refocus_skips_headings(void)
{
    static AtScreen sc;
    AtFocusBlock fb[AT_MAX_BLOCKS];
    AtFocusPos p, q;
    int i, nb;
    memset(&sc, 0, sizeof sc);
    sc.primary = AT_PRIMARY_LIST; sc.n_items = 6;
    for (i = 0; i < 6; i++) { snprintf(sc.items[i].id, AT_ID, "i%d", i); snprintf(sc.items[i].group, 24, "%s", i < 3 ? "DISPLAY" : "REMAP"); }
    nb = at_screen_focus_blocks(&sc, fb);
    CHECK(nb == 1 && fb[0].n == 6);                                       /* six rows, two headings: the focus block counts only the rows */
    p.block = 0; p.index = 0;
    q = at_focus_move(fb, nb, p, AT_DIR_UP, 1);
    CHECK(q.block == 0 && q.index == 5);                                  /* Up from the first row wraps to the last, never onto a heading */
    p.index = 2; q = at_focus_move(fb, nb, p, AT_DIR_DOWN, 1);
    CHECK(q.index == 3);                                                  /* across a heading: the next row */
    q = at_screen_refocus(&sc, NULL, "i4", p);
    CHECK(q.block == 0 && q.index == 4);                                  /* refocus by id */
    q = at_screen_refocus(&sc, NULL, "gone", p);
    CHECK(q.block == 0 && q.index == 2);                                  /* an id that left: the same place, clamped */
    sc.n_items = 2;
    q = at_screen_refocus(&sc, NULL, "gone", p);
    CHECK(q.block == 0 && q.index == 1);                                  /* fewer rows: the last one, never past the end */
}

/* Atlas step 7: a stepper value, the world backdrop, tabs from Lua, an explainer with no picture well */
static int list_root(const char *id, int items_t)
{
    int root = atv_table(A), prim = atv_table(A);
    S(root, "id", id); S(prim, "kind", "list"); atv_set(A, prim, "items", items_t); atv_set(A, root, "primary", prim);
    return root;
}
static int one_item(const char *id, const char *label)
{
    int it = atv_table(A);
    S(it, "id", id); S(it, "label", label);
    return it;
}
static void step7(void)
{
    AtScreen sc; char err[160]; int root, items, it, val, tabs, t1, t2, on, refs[32], n, k, seen = 0;
    /* a stepper value converts with its text */
    items = atv_table(A); it = one_item("r1", "Row"); val = atv_table(A); S(val, "kind", "stepper"); S(val, "text", "Stand");
    atv_set(A, it, "value", val); atv_push(A, items, it); root = list_root("t.one", items);
    CHECK(at_screen_from_val(A, root, "t", &sc, err, sizeof err) == 1);
    CHECK(sc.items[0].vkind == AT_VAL_STEPPER && strcmp(sc.items[0].text, "Stand") == 0 && sc.backdrop == AT_BD_GROUND);
    /* the backdrop (a key is set once per tree: each case is a tree of its own) */
    { int r2 = list_root("t.two", items); S(r2, "backdrop", "world"); CHECK(at_screen_from_val(A, r2, "t", &sc, err, sizeof err) == 1 && sc.backdrop == AT_BD_WORLD); }
    { int r2 = list_root("t.two", items); S(r2, "backdrop", "sky"); CHECK(at_screen_from_val(A, r2, "t", &sc, err, sizeof err) == 0 && strstr(err, "backdrop") != NULL); }
    { int r2 = list_root("t.two", items); S(r2, "backdrop", "ground"); CHECK(at_screen_from_val(A, r2, "t", &sc, err, sizeof err) == 1 && sc.backdrop == AT_BD_GROUND); }
    /* tabs from Lua: names, a count, the tab asked for, on.tab as a reference that is released with the screen */
    tabs = atv_table(A); t1 = atv_table(A); t2 = atv_table(A);
    S(t1, "name", "PLAY"); S(t2, "name", "DISPLAY"); N(t2, "count", 4);
    atv_push(A, tabs, t1); atv_push(A, tabs, t2); atv_set(A, root, "tabs", tabs); N(root, "tab", 2);
    on = atv_table(A); atv_set(A, on, "tab", atv_fn(A, 41)); atv_set(A, root, "on", on);
    CHECK(at_screen_from_val(A, root, "t", &sc, err, sizeof err) == 1);
    CHECK(sc.n_tabs == 2 && strcmp(sc.tabs[1].name, "DISPLAY") == 0 && sc.tabs[1].count == 4 && sc.tabs[0].count == -1 && sc.tab0 == 1);
    n = at_screen_fn_refs(&sc, refs, 32);
    for (k = 0; k < n; k++) seen += refs[k] == 41;
    CHECK(seen == 1);                                                         /* on.tab is released with the screen */
    {   AtScreen z; int k2;                                                   /* an engine screen's record is zeroed: on.tab, kept as a reference PLUS ONE, names none */
        memset(&z, 0, sizeof z);
        z.fn_provide = z.fn_accept = z.fn_back = z.fn_focus = z.fn_change = z.fn_open = z.fn_close = z.fn_counter = z.fn_page = z.fn_start = -1;
        z.fn_alt[0] = z.fn_alt[1] = z.fn_alt[2] = -1;
        CHECK(at_screen_fn_refs(&z, refs, 32) == 0);
        (void) k2;
    }
    { int r2 = list_root("t.two", items); atv_set(A, r2, "tabs", atv_table(A));   /* an empty tabs table is refused */
      CHECK(at_screen_from_val(A, r2, "t", &sc, err, sizeof err) == 0 && strstr(err, "tabs") != NULL); }
    {   int many = atv_table(A), i, r2 = list_root("t.two", items);
        for (i = 0; i < AT_MAX_TABS + 1; i++) { int tt = atv_table(A); S(tt, "name", "T"); atv_push(A, many, tt); }
        atv_set(A, r2, "tabs", many);
        CHECK(at_screen_from_val(A, r2, "t", &sc, err, sizeof err) == 0 && strstr(err, "tabs") != NULL);
    }
    /* the explainer: well = false is a screen row with no picture; WITH strings are tags */
    {   AtExplainer e; int ex = atv_table(A), with = atv_table(A);
        atv_push(A, with, atv_str(A, "B Boxes")); atv_push(A, with, atv_str(A, "L Labels"));
        atv_set(A, ex, "with", with); atv_set(A, ex, "well", atv_bool(A, 0)); S(ex, "title", "Mode");
        CHECK(at_explainer_from_val(A, ex, &e, err, sizeof err) == 1 && e.no_well == 1 && e.n_with_text == 2 && strcmp(e.with_text[1], "L Labels") == 0);
        ex = atv_table(A); S(ex, "title", "Mode"); atv_set(A, ex, "well", atv_bool(A, 1));
        CHECK(at_explainer_from_val(A, ex, &e, err, sizeof err) == 1 && e.no_well == 0);
    }
}

int main(void)
{
    A = (AtvArena *) malloc(sizeof *A);
    atv_init(A); step7();
    atv_init(A); parse_bag();
    atv_init(A); errors();
    atv_init(A); list_screen();
    atv_init(A); tiles_screen();
    atv_init(A); explainer();
    atv_init(A); refocus();
    refocus_skips_headings();
    atv_init(A); pause_and_persist();
    atv_init(A); cards_and_countdown();
    atv_init(A); grid_links();
    atv_init(A); accept_semantics();
    atv_init(A); hardening();
    atv_init(A); page_start_change();
    atv_init(A); ext_cells_never_copied();
    atv_init(A); lua_door_unchanged();
    atv_init(A); tabs_and_cursors();
    free(A);
    ATLAS_DONE("atlas screen");
}
