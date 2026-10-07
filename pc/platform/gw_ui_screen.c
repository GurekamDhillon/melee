#include "gw_ui_screen.h"

#include <stdio.h>
#include <string.h>

#define FAIL(...) do { snprintf(err, (size_t) errcap, __VA_ARGS__); return 0; } while (0)

/* ---- small readers; a string cut to its field counts as a warning ---------------------------------- */
static void get_str(const AtvArena *a, int t, const char *k, char *dst, int cap, int *warn)
{
    int n = atv_get(a, t, k);
    const char *s;
    if (atv_kind(a, n) != ATV_STR) return;
    s = atv_strv(a, n, "");
    if ((int) strlen(s) >= cap && warn) (*warn)++;
    snprintf(dst, (size_t) cap, "%s", s);
}
static int get_int_of(const AtvArena *a, int n)
{
    double d = atv_numv(a, n, -1.0);
    if (d != d) return -1;
    return d > 1.0e9 ? 1000000000 : (d < -1.0e9 ? -1000000000 : (int) d);
}
/* A number from a script can be NaN, infinite or huge: a bare (int) cast of those is undefined. Non-finite takes the default; the rest is clamped. */
static int get_int(const AtvArena *a, int t, const char *k, int def)
{
    double d = atv_numv(a, atv_get(a, t, k), (double) def);
    if (d != d) return def;
    if (d > 1.0e9) return 1000000000;
    if (d < -1.0e9) return -1000000000;
    return (int) d;
}
/* a model or ring index: below zero means none, and nothing past 65535 is a model */
static int to_model(int v) { return v < 0 ? AT_NO_MODEL : (v > 65535 ? 65535 : v); }
static int get_model(const AtvArena *a, int t, const char *k) { return to_model(get_int(a, t, k, AT_NO_MODEL)); }
/* an id is the refocus and handler key, so one the field would cut is an error, not a twin */
static int id_too_long(const AtvArena *a, int t) { return (int) strlen(atv_strv(a, atv_get(a, t, "id"), "")) >= AT_ID; }
static int get_fn(const AtvArena *a, int t, const char *k) { return atv_fnv(a, atv_get(a, t, k)); }
static int flag(const AtvArena *a, int f, const char *k, unsigned bit) { return atv_boolv(a, atv_get(a, f, k), 0) ? (int) bit : 0; }

static unsigned read_flags(const AtvArena *a, int f)
{
    if (atv_kind(a, f) != ATV_TABLE) return 0;
    return (unsigned) (flag(a, f, "locked", AT_CELL_LOCKED) | flag(a, f, "empty", AT_CELL_EMPTY) | flag(a, f, "merge", AT_CELL_MERGE) |
                       flag(a, f, "new", AT_CELL_NEW) | flag(a, f, "selected", AT_CELL_SELECTED) | flag(a, f, "disabled", AT_CELL_DISABLED));
}

static int button_char(const char *s)
{
    if (strcmp(s, "START") == 0) return 'S';
    if (s[0] != '\0' && s[1] == '\0' && strchr("ABXYZLR", s[0]) != NULL) return s[0];
    return 0;
}

void at_view_init(AtView *v)
{
    memset(v, 0, sizeof *v);
    v->focus.block = v->focus.index = -1;
}

/* one list or tiles item (i of dst); the duplicate check runs over dst[0..i) */
static int read_item(const AtvArena *a, int in, int i, AtItem *dst, AtScreen *o, int is_more, char *err, int errcap)
{
    int k, vt;
    AtItem *it = &dst[i];
    const char *vk, *what = is_more ? "more item" : "item";
    if (atv_kind(a, in) != ATV_TABLE) FAIL("gd.ui.screen: %s %d is not a table", what, i + 1);
    if (id_too_long(a, in)) FAIL("gd.ui.screen: %s %d: id is too long (%d characters at most)", what, i + 1, AT_ID - 1);
    get_str(a, in, "id", it->id, AT_ID, &o->warnings);
    if (it->id[0] == '\0') FAIL("gd.ui.screen: %s %d has no id", what, i + 1);
    for (k = 0; k < i; k++) if (strcmp(dst[k].id, it->id) == 0) FAIL("gd.ui.screen: duplicate item id \"%s\"", it->id);
    if (is_more) for (k = 0; k < o->n_items; k++) if (strcmp(o->items[k].id, it->id) == 0) FAIL("gd.ui.screen: duplicate item id \"%s\"", it->id);
    get_str(a, in, "label", it->label, AT_STR, &o->warnings);
    get_str(a, in, "sub", it->sub, AT_STR, &o->warnings);
    get_str(a, in, "icon", it->icon, AT_ID, &o->warnings);
    get_str(a, in, "tag", it->tag, 16, &o->warnings);
    get_str(a, in, "badge", it->badge, 8, &o->warnings);
    get_str(a, in, "numeral", it->numeral, 6, &o->warnings);
    if (atv_boolv(a, atv_get(a, in, "disabled"), 0)) it->flags |= AT_CELL_DISABLED;
    if (atv_boolv(a, atv_get(a, in, "selected"), 0)) it->flags |= AT_CELL_SELECTED;
    vt = atv_get(a, in, "value");
    if (atv_kind(a, vt) == ATV_TABLE) {
        vk = atv_strv(a, atv_get(a, vt, "kind"), "");
        if (strcmp(vk, "toggle") == 0) { it->vkind = AT_VAL_TOGGLE; it->on = atv_boolv(a, atv_get(a, vt, "on"), 0); }
        else if (strcmp(vk, "choice") == 0) it->vkind = AT_VAL_CHOICE;
        else if (strcmp(vk, "slider") == 0) { it->vkind = AT_VAL_SLIDER; it->vmin = get_int(a, vt, "min", 0); it->vmax = get_int(a, vt, "max", 100); it->vval = get_int(a, vt, "value", 0);
            if (it->vmin >= it->vmax) FAIL("gd.ui.screen: item \"%s\": slider min must be below max", it->id);
            if (it->vval < it->vmin) it->vval = it->vmin;
            if (it->vval > it->vmax) it->vval = it->vmax; }
        else if (strcmp(vk, "text") == 0) it->vkind = AT_VAL_TEXT;
        else if (strcmp(vk, "counter") == 0) it->vkind = AT_VAL_COUNTER;
        else FAIL("gd.ui.screen: item \"%s\": unknown value kind \"%s\"", it->id, vk);
        get_str(a, vt, "text", it->text, AT_STR, &o->warnings);
    }
    return 1;
}

int at_screen_from_val(const AtvArena *a, int root, const char *owner, AtScreen *o, char *err, int errcap)
{
    int prim, blocks, items, i, j, k, ex, keys, on, alt, nb, nc, n;
    const char *kind, *id, *w;
    memset(o, 0, sizeof *o);
    if (a->overflow) FAIL("gd.ui.screen: the description is too large (it overflowed the conversion arena)");
    o->fn_provide = o->fn_accept = o->fn_back = o->fn_focus = o->fn_change = o->fn_open = o->fn_close = o->fn_counter = o->fn_page = o->fn_start = -1;
    o->fn_alt[0] = o->fn_alt[1] = o->fn_alt[2] = -1;
    o->preset = AT_PRESET_NONE;
    o->port = 1;
    if (atv_kind(a, root) != ATV_TABLE) FAIL("gd.ui.screen: a screen description is a table");
    id = atv_strv(a, atv_get(a, root, "id"), "");
    if (id[0] == '\0') FAIL("gd.ui.screen: the description has no id");
    if (owner != NULL && owner[0] != '\0') {
        size_t ol = strlen(owner);
        if (strncmp(id, owner, ol) != 0 || id[ol] != '.') FAIL("gd.ui.screen: id \"%s\" must start with \"%s.\"", id, owner);
    }
    if (strlen(id) >= sizeof o->id) FAIL("gd.ui.screen: id \"%s\" is too long (47 characters at most)", id);
    snprintf(o->id, sizeof o->id, "%s", id);
    snprintf(o->title, sizeof o->title, "%s", id);
    get_str(a, atv_get(a, root, "trail"), "title", o->title, AT_STR, &o->warnings);
    {
        int tr = atv_get(a, root, "trail"), pn = atv_len(a, tr), pi;           /* the trail's array part: the parents ("SOLO", "ENVOY") */
        for (pi = 0; pi < pn && o->n_parents < 3; pi++) {
            const char *ps = atv_strv(a, atv_at(a, tr, pi + 1), "");
            if (ps[0] != '\0') snprintf(o->parent[o->n_parents++], AT_STR, "%s", ps);
        }
    }
    o->chapter = get_int(a, root, "chapter", 0);
    if (o->chapter < 0 || o->chapter > 5) FAIL("gd.ui.screen: chapter is 0 to 5");

    prim = atv_get(a, root, "primary");
    if (atv_kind(a, prim) != ATV_TABLE) FAIL("gd.ui.screen: \"%s\" has no primary", id);
    kind = atv_strv(a, atv_get(a, prim, "kind"), "");
    if (strcmp(kind, "grid") == 0) o->primary = AT_PRIMARY_GRID;
    else if (strcmp(kind, "list") == 0) o->primary = AT_PRIMARY_LIST;
    else if (strcmp(kind, "tiles") == 0) {
        o->primary = AT_PRIMARY_TILES;
        o->tile_cols = get_int(a, prim, "cols", 0);
        if (o->tile_cols < 0 || o->tile_cols > 2) FAIL("gd.ui.screen: tiles: cols must be 1 or 2");
    }
    else if (strcmp(kind, "cards") == 0) o->primary = AT_PRIMARY_CARDS;
    else FAIL("gd.ui.screen: primary kind \"%s\" is not supported here (grid, list, tiles or cards)", kind);
    o->pause = strcmp(atv_strv(a, atv_get(a, root, "kind"), ""), "pause") == 0;
    if (o->pause && o->primary != AT_PRIMARY_LIST) FAIL("gd.ui.screen: a pause screen has a list primary");
    o->persist = atv_boolv(a, atv_get(a, root, "persist"), 0);

    if (o->primary == AT_PRIMARY_CARDS) {
        int cards = atv_get(a, prim, "cards"), nc2 = atv_len(a, cards);
        if (nc2 > AT_MAX_CARDS) FAIL("gd.ui.screen: at most %d cards (%d given)", AT_MAX_CARDS, nc2);
        if (nc2 < 1) FAIL("gd.ui.screen: a cards screen needs 1 to %d cards", AT_MAX_CARDS);
        o->n_cards = nc2;
        for (i = 0; i < nc2; i++) {
            int cn = atv_at(a, cards, i + 1), tn;
            AtCardRec *c = &o->cards[i];
            const char *lt;
            if (atv_kind(a, cn) != ATV_TABLE) FAIL("gd.ui.screen: card %d is not a table", i + 1);
            if (id_too_long(a, cn)) FAIL("gd.ui.screen: card %d: id is too long (%d characters at most)", i + 1, AT_ID - 1);
            get_str(a, cn, "id", c->id, AT_ID, &o->warnings);
            if (c->id[0] == '\0') FAIL("gd.ui.screen: card %d has no id", i + 1);
            for (k = 0; k < i; k++) if (strcmp(o->cards[k].id, c->id) == 0) FAIL("gd.ui.screen: duplicate card id %s", c->id);
            c->disabled = atv_boolv(a, atv_get(a, cn, "disabled"), 0);
            get_str(a, cn, "name", c->offer.name, AT_STR, &o->warnings);
            get_str(a, cn, "rule", c->offer.rule, AT_TEXT, &o->warnings);
            get_str(a, cn, "tag", c->offer.tag, 24, &o->warnings);
            c->offer.model = get_model(a, cn, "model");
            c->offer.ring = get_model(a, cn, "ring");
            lt = atv_strv(a, atv_get(a, cn, "letter"), "");
            c->offer.letter = lt[0];
            {
                double col = atv_numv(a, atv_get(a, cn, "rgba"), 0);
                c->offer.rgba = !(col >= 0.0) ? 0u : (col >= 4294967295.0 ? 0xFFFFFFFFu : (unsigned) col);
            }
            tn = atv_get(a, cn, "tag_tone");
            if (atv_kind(a, tn) == ATV_STR) {
                const char *tt = atv_strv(a, tn, "");
                c->offer.tag_tone = strcmp(tt, "jade") == 0 ? 1 : strcmp(tt, "ember") == 0 ? 2 : strcmp(tt, "sun") == 0 ? 3 : strcmp(tt, "rose") == 0 ? 4 : 0;
            } else {
                c->offer.tag_tone = get_int(a, cn, "tag_tone", 0);
                if (c->offer.tag_tone < 0 || c->offer.tag_tone > 4) c->offer.tag_tone = 0;
            }
        }
    } else if (o->primary == AT_PRIMARY_GRID) {
        blocks = atv_get(a, prim, "blocks");
        nb = atv_len(a, blocks);
        if (nb < 1 || nb > AT_MAX_BLOCKS) FAIL("gd.ui.screen: a grid needs 1 to %d blocks (it has %d)", AT_MAX_BLOCKS, nb);
        o->n_blocks = nb;
        for (i = 0; i < nb; i++) {
            int bn = atv_at(a, blocks, i + 1), cells;
            AtBlock *b = &o->blocks[i];
            if (atv_kind(a, bn) != ATV_TABLE) FAIL("gd.ui.screen: block %d is not a table", i + 1);
            if (id_too_long(a, bn)) FAIL("gd.ui.screen: block %d: id is too long (%d characters at most)", i + 1, AT_ID - 1);
            get_str(a, bn, "id", b->id, AT_ID, &o->warnings);
            if (b->id[0] == '\0') FAIL("gd.ui.screen: block %d has no id", i + 1);
            for (k = 0; k < i; k++) if (strcmp(o->blocks[k].id, b->id) == 0) FAIL("gd.ui.screen: duplicate block id \"%s\"", b->id);
            get_str(a, bn, "title", b->title, AT_STR, &o->warnings);
            get_str(a, bn, "count", b->count, 24, &o->warnings);
            get_str(a, bn, "note", b->note, AT_STR, &o->warnings);
            b->cols = get_int(a, bn, "cols", 1);
            if (b->cols < 1 || b->cols > AT_MAX_CELLS) FAIL("gd.ui.screen: block \"%s\": cols is 1 to %d", b->id, AT_MAX_CELLS);
            b->stones = strcmp(atv_strv(a, atv_get(a, bn, "kind"), ""), "stones") == 0;
            cells = atv_get(a, bn, "cells");
            nc = atv_len(a, cells);
            if (nc > AT_MAX_CELLS) FAIL("gd.ui.screen: block \"%s\" has %d cells (%d at most)", b->id, nc, AT_MAX_CELLS);
            b->n = nc;
            for (j = 0; j < nc; j++) {
                int cn = atv_at(a, cells, j + 1);
                AtCell *c = &b->cells[j];
                const char *og;
                if (atv_kind(a, cn) != ATV_TABLE) FAIL("gd.ui.screen: block \"%s\" cell %d is not a table", b->id, j + 1);
                if (id_too_long(a, cn)) FAIL("gd.ui.screen: block \"%s\" cell %d: id is too long (%d characters at most)", b->id, j + 1, AT_ID - 1);
                get_str(a, cn, "id", c->id, AT_ID, &o->warnings);
                if (c->id[0] == '\0') FAIL("gd.ui.screen: block \"%s\" cell %d has no id", b->id, j + 1);
                for (k = 0; k < i; k++) { int q; for (q = 0; q < o->blocks[k].n; q++) if (strcmp(o->blocks[k].cells[q].id, c->id) == 0) FAIL("gd.ui.screen: duplicate cell id \"%s\"", c->id); }
                for (k = 0; k < j; k++) if (strcmp(b->cells[k].id, c->id) == 0) FAIL("gd.ui.screen: duplicate cell id \"%s\"", c->id);
                get_str(a, cn, "name", c->name, AT_STR, &o->warnings);
                c->model = get_model(a, cn, "model");
                c->ring = get_model(a, cn, "ring");
                c->index = get_int(a, cn, "index", 0);
                if (c->index < 0) c->index = 0;
                c->pips = get_int(a, cn, "pips", 0);
                if (c->pips < 0) c->pips = 0;
                if (c->pips > 4) c->pips = 4;
                og = atv_strv(a, atv_get(a, cn, "origin"), "");
                c->origin = og[0] == 'G' || og[0] == '+' ? og[0] : 0;
                {
                    double col = atv_numv(a, atv_get(a, cn, "color"), 0);          /* NaN and negatives are 0, too large is all bits set */
                    c->rgba = !(col >= 0.0) ? 0u : (col >= 4294967295.0 ? 0xFFFFFFFFu : (unsigned) col);
                }
                w = atv_strv(a, atv_get(a, cn, "letter"), "");
                c->letter = w[0];
                c->flags = read_flags(a, atv_get(a, cn, "flags"));
            }
        }
        {
            int links = atv_get(a, prim, "links"), nl = atv_len(a, links), li;
            if (nl > AT_MAX_LINKS) FAIL("gd.ui.screen: at most %d links (%d given)", AT_MAX_LINKS, nl);
            for (li = 0; li < nl; li++) {
                int ln2 = atv_at(a, links, li + 1);
                AtLink *lk = &o->links[o->n_links];
                int fa = 0, fb = 0, bb, cc;
                if (atv_kind(a, ln2) != ATV_TABLE) FAIL("gd.ui.screen: link %d is not a table", li + 1);
                get_str(a, ln2, "a", lk->a, AT_ID, &o->warnings);
                get_str(a, ln2, "b", lk->b, AT_ID, &o->warnings);
                for (bb = 0; bb < o->n_blocks; bb++) for (cc = 0; cc < o->blocks[bb].n; cc++) {
                    if (strcmp(o->blocks[bb].cells[cc].id, lk->a) == 0) fa = 1;
                    if (strcmp(o->blocks[bb].cells[cc].id, lk->b) == 0) fb = 1;
                }
                if (!fa || !fb || strcmp(lk->a, lk->b) == 0) { memset(lk, 0, sizeof *lk); o->links_skipped++; continue; }
                {
                    double col = atv_numv(a, atv_get(a, ln2, "rgba"), 0);
                    lk->rgba = !(col >= 0.0) ? 0u : (col >= 4294967295.0 ? 0xFFFFFFFFu : (unsigned) col);
                    if (lk->rgba == 0u) lk->rgba = 0x4FD6AAFFu;
                }
                o->n_links++;
            }
        }
        {
            int ft = atv_get(a, prim, "footer");
            if (atv_kind(a, ft) == ATV_TABLE) {
                o->footer.has = 1;
                get_str(a, ft, "label", o->footer.label, 24, &o->warnings);
                o->footer.model_a = get_model(a, ft, "a");
                o->footer.model_b = get_model(a, ft, "b");
                o->footer.model_out = get_model(a, ft, "out");
                get_str(a, ft, "text", o->footer.text, AT_STR, &o->warnings);
            }
        }
    } else {
        items = atv_get(a, prim, "items");
        n = atv_len(a, items);
        if (n < 1 || n > AT_MAX_ITEMS) FAIL("gd.ui.screen: a list needs 1 to %d items (it has %d)", AT_MAX_ITEMS, n);
        o->n_items = n;
        for (i = 0; i < n; i++)
            if (!read_item(a, atv_at(a, items, i + 1), i, o->items, o, 0, err, errcap)) return 0;
        if (o->primary == AT_PRIMARY_TILES) {
            int more = atv_get(a, prim, "more"), nm = atv_len(a, more);
            if (nm > AT_MAX_MORE) FAIL("gd.ui.screen: tiles: at most %d more items (%d given)", AT_MAX_MORE, nm);
            o->n_more = nm;
            for (i = 0; i < nm; i++)
                if (!read_item(a, atv_at(a, more, i + 1), i, o->more, o, 1, err, errcap)) return 0;
        }
    }

    ex = atv_get(a, root, "explainer");
    if (atv_kind(a, ex) == ATV_STR) {
        if (strcmp(atv_strv(a, ex, ""), "none") != 0) FAIL("gd.ui.screen: explainer must be a table or \"none\"");
    } else if (atv_kind(a, ex) == ATV_TABLE) {
        w = atv_strv(a, atv_get(a, ex, "width"), "normal");
        if (strcmp(w, "narrow") == 0) o->preset = AT_PRESET_NARROW;
        else if (strcmp(w, "normal") == 0) o->preset = AT_PRESET_NORMAL;
        else if (strcmp(w, "wide") == 0) o->preset = AT_PRESET_WIDE;
        else FAIL("gd.ui.screen: explainer width \"%s\" is narrow, normal or wide", w);
        o->fn_provide = get_fn(a, ex, "provide");
    }

    keys = atv_get(a, root, "keys");
    n = atv_len(a, keys);
    if (n > AT_MAX_KEYS) FAIL("gd.ui.screen: at most %d key hints (%d given)", AT_MAX_KEYS, n);
    o->n_keys = n;
    for (i = 0; i < n; i++) {
        int kn = atv_at(a, keys, i + 1), bn, ln;
        AtKey *key = &o->keys[i];
        const char *bs;
        int ch;
        if (atv_kind(a, kn) != ATV_TABLE) FAIL("gd.ui.screen: key hint %d is not a table", i + 1);
        bn = atv_get(a, kn, "btn"); if (bn < 0) bn = atv_at(a, kn, 1);
        ln = atv_get(a, kn, "label"); if (ln < 0) ln = atv_at(a, kn, 2);
        bs = atv_strv(a, bn, "");
        ch = button_char(bs);
        if (ch == 0) FAIL("gd.ui.screen: key hint %d: unknown button \"%s\" (A B X Y Z L R START)", i + 1, bs);
        key->btn = (char) ch;
        key->fn_label = atv_fnv(a, ln);
        key->fn_when = get_fn(a, kn, "when");
        if (atv_kind(a, ln) == ATV_STR) snprintf(key->label, sizeof key->label, "%s", atv_strv(a, ln, ""));
    }

    {
        int cn = atv_get(a, root, "counter");
        if (atv_kind(a, cn) == ATV_STR) snprintf(o->counter, sizeof o->counter, "%s", atv_strv(a, cn, ""));
        else o->fn_counter = atv_fnv(a, cn);
    }
    {
        int cdn = atv_get(a, root, "countdown");
        if (atv_kind(a, cdn) == ATV_NUM) {
            o->has_countdown = 1; o->countdown = get_int(a, root, "countdown", 0);
            if (o->countdown < 0) o->countdown = 0;
            if (o->countdown > 5999) o->countdown = 5999;
        }
    }
    o->input_feed = strcmp(atv_strv(a, atv_get(a, root, "input"), "engine"), "feed") == 0;
    o->port = get_int(a, root, "port", 1);
    if (o->port < 1 || o->port > 4) FAIL("gd.ui.screen: port is 1 to 4");

    on = atv_get(a, root, "on");
    if (atv_kind(a, on) == ATV_TABLE) {
        o->fn_accept = get_fn(a, on, "accept");
        o->fn_back = get_fn(a, on, "back");
        o->fn_focus = get_fn(a, on, "focus");
        o->fn_change = get_fn(a, on, "change");
        o->fn_open = get_fn(a, on, "open");
        o->fn_close = get_fn(a, on, "close");
        o->fn_page = get_fn(a, on, "page");
        o->fn_start = get_fn(a, on, "start");
        alt = atv_get(a, on, "alt");
        o->fn_alt[0] = get_fn(a, alt, "X");
        o->fn_alt[1] = get_fn(a, alt, "Y");
        o->fn_alt[2] = get_fn(a, alt, "Z");
    }
    return 1;
}

int at_explainer_from_val(const AtvArena *a, int t, AtExplainer *e, char *err, int errcap)
{
    int media, with, from, i, n;
    memset(e, 0, sizeof *e);
    e->media_model = e->media_ring = AT_NO_MODEL;
    (void) err; (void) errcap;
    if (atv_kind(a, t) != ATV_TABLE) return 1;
    e->has = 1;
    get_str(a, t, "kicker", e->kicker, AT_STR, &e->warn);
    get_str(a, t, "title", e->title, AT_STR, &e->warn);
    get_str(a, t, "what", e->what, AT_TEXT, &e->warn);
    media = atv_get(a, t, "media");
    if (atv_kind(a, media) == ATV_TABLE) {
        e->media_model = get_model(a, media, "model");
        e->media_ring = get_model(a, media, "ring");
    }
    with = atv_get(a, t, "with");
    n = atv_len(a, with);
    for (i = 0; i < n && (e->n_with < AT_MAX_WITH || e->n_with_text < AT_MAX_WITH); i++) {
        int w = atv_at(a, with, i + 1);
        if (atv_kind(a, w) == ATV_STR) {                                    /* a tag such as "Melee" or "Rules" */
            if (e->n_with_text < AT_MAX_WITH) snprintf(e->with_text[e->n_with_text++], sizeof e->with_text[0], "%s", atv_strv(a, w, ""));
            continue;
        }
        if (e->n_with >= AT_MAX_WITH) continue;
        e->with_model[e->n_with++] = atv_kind(a, w) == ATV_TABLE ? get_model(a, w, "model") : to_model(get_int_of(a, w));
    }
    from = atv_get(a, t, "from");
    if (atv_kind(a, from) == ATV_TABLE) get_str(a, from, "text", e->from_text, AT_STR, &e->warn);
    return 1;
}

int at_screen_fn_refs(const AtScreen *s, int *out, int cap)
{
    int n = 0, i;
    int all[16];
    int na = 0;
    all[na++] = s->fn_provide; all[na++] = s->fn_accept; all[na++] = s->fn_back; all[na++] = s->fn_focus;
    all[na++] = s->fn_change; all[na++] = s->fn_open; all[na++] = s->fn_close; all[na++] = s->fn_counter;
    all[na++] = s->fn_alt[0]; all[na++] = s->fn_alt[1]; all[na++] = s->fn_alt[2]; all[na++] = s->fn_page; all[na++] = s->fn_start;
    for (i = 0; i < na; i++) if (all[i] >= 0 && n < cap) out[n++] = all[i];
    for (i = 0; i < s->n_keys; i++) {
        if (s->keys[i].fn_label >= 0 && n < cap) out[n++] = s->keys[i].fn_label;
        if (s->keys[i].fn_when >= 0 && n < cap) out[n++] = s->keys[i].fn_when;
    }
    return n;
}

int at_screen_focus_blocks(const AtScreen *s, AtFocusBlock *fb)
{
    int b, row = 0;
    if (s->primary == AT_PRIMARY_LIST) {
        fb[0].col0 = 0; fb[0].row0 = 0; fb[0].cols = 1; fb[0].n = s->n_items; fb[0].exists = NULL;
        return 1;
    }
    if (s->primary == AT_PRIMARY_DISPLAY) return 0;
    if (s->primary == AT_PRIMARY_CARDS) {                                  /* one row of cards: left and right move between them, and wrap */
        fb[0].col0 = 0; fb[0].row0 = 0; fb[0].cols = s->n_cards > 0 ? s->n_cards : 1; fb[0].n = s->n_cards; fb[0].exists = NULL;
        return 1;
    }
    if (s->primary == AT_PRIMARY_TILES) {
        int cols = at_screen_tile_cols(s), rows = (s->n_items + cols - 1) / cols, nb = 0;
        fb[0].col0 = 0; fb[0].row0 = 0; fb[0].cols = cols; fb[0].n = s->n_items; fb[0].exists = NULL; nb = 1;
        if (s->n_more > 0) { fb[1].col0 = 0; fb[1].row0 = rows; fb[1].cols = s->n_more; fb[1].n = s->n_more; fb[1].exists = NULL; nb = 2; }
        return nb;
    }
    for (b = 0; b < s->n_blocks; b++) {
        fb[b].col0 = 0; fb[b].row0 = row; fb[b].cols = s->blocks[b].cols > 0 ? s->blocks[b].cols : 1;
        fb[b].n = s->blocks[b].n; fb[b].exists = NULL;
        row += (fb[b].n + fb[b].cols - 1) / fb[b].cols;
    }
    return s->n_blocks;
}

int at_screen_tile_cols(const AtScreen *s)
{
    if (s->tile_cols == 1 || s->tile_cols == 2) return s->tile_cols;
    return s->n_items <= 3 ? 1 : 2;
}

/* the item array a tiles block names: block 0 the tiles, block 1 the More row */
static const AtItem *tiles_cell(const AtScreen *s, AtFocusPos p)
{
    if (p.block == 0 && p.index >= 0 && p.index < s->n_items) return &s->items[p.index];
    if (p.block == 1 && p.index >= 0 && p.index < s->n_more) return &s->more[p.index];
    return NULL;
}

const char *at_screen_block_id(const AtScreen *s, int block)
{
    if (s->primary == AT_PRIMARY_TILES) return block == 0 ? "tiles" : (block == 1 && s->n_more > 0 ? "more" : NULL);
    if (s->primary == AT_PRIMARY_DISPLAY) return NULL;
    if (s->primary == AT_PRIMARY_CARDS) return block == 0 ? "cards" : NULL;
    if (s->primary == AT_PRIMARY_LIST) return block == 0 ? "list" : NULL;
    return (block >= 0 && block < s->n_blocks) ? s->blocks[block].id : NULL;
}

const char *at_screen_cell_id(const AtScreen *s, AtFocusPos p)
{
    if (s->primary == AT_PRIMARY_TILES) { const AtItem *it = tiles_cell(s, p); return it != NULL ? it->id : NULL; }
    if (s->primary == AT_PRIMARY_DISPLAY) return NULL;
    if (s->primary == AT_PRIMARY_CARDS) return (p.block == 0 && p.index >= 0 && p.index < s->n_cards) ? s->cards[p.index].id : NULL;
    if (s->primary == AT_PRIMARY_LIST) return (p.block == 0 && p.index >= 0 && p.index < s->n_items) ? s->items[p.index].id : NULL;
    if (p.block < 0 || p.block >= s->n_blocks || p.index < 0 || p.index >= s->blocks[p.block].n) return NULL;
    return s->blocks[p.block].cells[p.index].id;
}

AtFocusPos at_screen_refocus(const AtScreen *s, const char *block_id, const char *cell_id, AtFocusPos old)
{
    AtFocusBlock fb[AT_MAX_BLOCKS];
    int nb = at_screen_focus_blocks(s, fb), b, i;
    AtFocusPos p = { -1, -1 };
    if (s->primary == AT_PRIMARY_TILES) {
        for (i = 0; cell_id != NULL && i < s->n_items; i++) if (strcmp(s->items[i].id, cell_id) == 0) { p.block = 0; p.index = i; return p; }
        for (i = 0; cell_id != NULL && i < s->n_more; i++) if (strcmp(s->more[i].id, cell_id) == 0) { p.block = 1; p.index = i; return p; }
        if (block_id != NULL && strcmp(block_id, "more") == 0 && s->n_more > 0) { p.block = 1; p.index = old.index < 0 ? 0 : (old.index >= s->n_more ? s->n_more - 1 : old.index); return p; }
        if (s->n_items > 0) { p.block = 0; p.index = old.block == 0 && old.index >= 0 ? (old.index >= s->n_items ? s->n_items - 1 : old.index) : 0; }
        return p;
    }
    if (s->primary == AT_PRIMARY_DISPLAY) return p;
    if (s->primary == AT_PRIMARY_CARDS) {
        for (i = 0; cell_id != NULL && i < s->n_cards; i++) if (strcmp(s->cards[i].id, cell_id) == 0) { p.block = 0; p.index = i; return p; }
        if (s->n_cards > 0) { p.block = 0; p.index = old.index < 0 ? 0 : (old.index >= s->n_cards ? s->n_cards - 1 : old.index); }
        return p;
    }
    if (s->primary == AT_PRIMARY_LIST) {
        for (i = 0; cell_id != NULL && i < s->n_items; i++) if (strcmp(s->items[i].id, cell_id) == 0) { p.block = 0; p.index = i; return p; }
        if (s->n_items > 0) { p.block = 0; p.index = old.index < 0 ? 0 : (old.index >= s->n_items ? s->n_items - 1 : old.index); }
        return p;
    }
    for (b = 0; block_id != NULL && cell_id != NULL && b < s->n_blocks; b++) {
        if (strcmp(s->blocks[b].id, block_id) != 0) continue;
        for (i = 0; i < s->blocks[b].n; i++) if (strcmp(s->blocks[b].cells[i].id, cell_id) == 0) { p.block = b; p.index = i; return p; }
    }
    for (b = 0; block_id != NULL && b < s->n_blocks; b++) {          /* the cell is gone: the same block, the same place, clamped */
        if (strcmp(s->blocks[b].id, block_id) != 0 || s->blocks[b].n < 1) continue;
        p.block = b;
        p.index = old.index < 0 ? 0 : (old.index >= s->blocks[b].n ? s->blocks[b].n - 1 : old.index);
        return p;
    }
    return at_focus_first(fb, nb);
}

int at_cell_accepts(const AtScreen *s, AtFocusPos p)
{
    if (at_screen_cell_id(s, p) == NULL) return 0;
    if (s->primary == AT_PRIMARY_TILES) return !(tiles_cell(s, p)->flags & AT_CELL_DISABLED);
    if (s->primary == AT_PRIMARY_CARDS) return !s->cards[p.index].disabled;
    if (s->primary == AT_PRIMARY_LIST) return !(s->items[p.index].flags & AT_CELL_DISABLED);
    return !(s->blocks[p.block].cells[p.index].flags & AT_CELL_DISABLED);
}

int at_screen_wants_pad(const AtScreen *s) { return !s->input_feed && s->primary != AT_PRIMARY_DISPLAY; }
