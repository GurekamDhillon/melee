/* gw_ui_nucleus.inc - the NUCLEUS screens of the MODS door: browse, search, filter and install mods from SSBM Nucleus (ssbmnucleus.net), as a pure model.
 * Like gw_ui_mods.c it knows nothing of the game, the network or the disk: a source (AtNcSrc) answers for the rows, the model builds the AtScreen / AtView
 * and applies events, and says what it wants done (AtNcAction). The engine half that answers the source is gw_nucleus_engine.inc; the door that draws and
 * feeds it is gw_script_ui_mods.inc (mode 2: the list, mode 3: a mod's detail).
 *
 * Included, not compiled on its own (no new .c to hand-add to the link list): by gw_script.c through gw_script_ui_mods.inc, and by
 * pc/tests/atlas_nucleus_test.c. It needs gw_ui_screen.h / gw_ui_input.h / gw_ui_parts.h / gw_ui_layout.h in scope.
 *
 * Layout. The list has three control rows (Search, Fighter, Sort) and then the results, under a tab strip: COSTUMES / STAGES / EFFECTS / MORE / INSTALLED.
 * A is the row's action (a control row edits; a result installs or updates), Y opens the detail, X starts a search, Z refreshes, L / R change tab, B leaves.
 * A search is typed on the keyboard when there is one and by an arcade picker otherwise (up / down change the last letter, right adds one, left deletes).
 * The credit "Mods from SSBM Nucleus - ssbmnucleus.net" is on every Nucleus screen. */
#ifndef GW_UI_NUCLEUS_INC
#define GW_UI_NUCLEUS_INC

#include <ctype.h>
#include <stdio.h>
#include <string.h>

#define AT_NC_TABS 5
#define AT_NC_IDS 6000
#define AT_NC_CONTROLS 3
#define AT_NC_WINDOW AT_MAX_ITEMS
#define AT_NC_VISIBLE 7
#define AT_NC_CREDIT "Mods from SSBM Nucleus - ssbmnucleus.net"

enum { AT_NC_ST_NONE, AT_NC_ST_INSTALLED, AT_NC_ST_UPDATE, AT_NC_ST_BUSY, AT_NC_ST_REMOVED, AT_NC_ST_UNSUPPORTED };
enum { AT_NC_ACT_INSTALL = 1, AT_NC_ACT_UNINSTALL, AT_NC_ACT_PAGE, AT_NC_ACT_REFRESH };
enum { AT_NC_VIEW_BROWSE, AT_NC_VIEW_INSTALLED };

typedef struct { char text[48]; unsigned type_mask; int fighter, sort, view; } AtNcFilter;      /* fighter -1: any; type_mask 0: any */
typedef struct {
    int id;
    char title[80], author[40], fighter[28], kind[16], reason[64], updated[24], page[120], desc[320], tags[96];
    int downloads, likes, state, installable, ncostumes, tex, files;
} AtNcRow;
typedef struct { char line[100]; int syncing, done, total, offline, restart, locked, job_state; char job_msg[140]; int catalog_n; } AtNcStatus;   /* job_state 0 idle 1 running 2 ok 3 failed */

typedef struct AtNcSrc {
    void *user;
    int (*filter)(void *u, const AtNcFilter *f, int *ids, int cap, unsigned *version);     /* the matching mod ids in the filter's order; version changes when the catalog does */
    int (*row)(void *u, int id, AtNcRow *out);                                              /* 0: no such mod */
    void (*status)(void *u, AtNcStatus *out);
    int (*act)(void *u, int id, int what);                                                  /* 1: started / done; 0: refused (status.job_msg says why) */
    int (*fighters)(void *u);
    const char *(*fighter_label)(void *u, int i);
    unsigned (*version)(void *u);                                                           /* the catalog version now (0: the same as filter's last answer) */
} AtNcSrc;

typedef struct {
    AtNcFilter f;
    int tab;
    int ids[AT_NC_IDS], n;
    unsigned ver; int stale;
    int sel, top, base;
    int edit; char edit_buf[48], edit_old[48];
    int mode, detail_id, detail_sel;
    char note[AT_STR]; int note_kind; double note_until;
} AtNcState;

enum { AT_NA_NONE, AT_NA_BACK, AT_NA_ACT };
typedef struct { int kind, id, what; } AtNcAction;

static const char *const at_nc_tab_name[AT_NC_TABS] = { "COSTUMES", "STAGES", "EFFECTS", "MORE", "INSTALLED" };
static const char *const at_nc_sort_name[5] = { "Recently updated", "Newest", "Most downloaded", "Most liked", "Title" };

static void at_nc_state_init(AtNcState *st) {
    memset(st, 0, sizeof *st);
    st->f.fighter = -1;
    st->f.type_mask = 1u;               /* the costumes tab */
    st->stale = 1;
    st->sel = AT_NC_CONTROLS;           /* the first result has the focus, not Search */
}

static void at_nc_apply_tab(AtNcState *st) {
    st->f.view = st->tab == 4 ? AT_NC_VIEW_INSTALLED : AT_NC_VIEW_BROWSE;
    st->f.type_mask = st->tab == 0 ? 1u << 0 : st->tab == 1 ? 1u << 1 : st->tab == 2 ? 1u << 2 : st->tab == 3 ? ((1u << 3) | (1u << 4) | (1u << 5) | (1u << 6)) : 0;
    st->stale = 1;
    st->sel = AT_NC_CONTROLS;
    st->top = 0;
}

static void at_nc_say(AtNcState *st, const char *text, int kind, double now_ms) {
    snprintf(st->note, sizeof st->note, "%s", text);
    st->note_kind = kind;
    st->note_until = now_ms + 3500.0;
}

static void at_nc_refresh(const AtNcSrc *s, AtNcState *st) {
    unsigned v = 0, cur = s->version ? s->version(s->user) : 0;
    if (!st->stale && cur == st->ver) return;
    st->n = s->filter(s->user, &st->f, st->ids, AT_NC_IDS, &v);
    st->ver = v ? v : cur;
    st->stale = 0;
    if (st->sel > AT_NC_CONTROLS + st->n - 1) st->sel = AT_NC_CONTROLS + st->n - 1;
    if (st->sel < 0) st->sel = 0;
}

static void at_nc_put_key(AtScreen *sc, AtView *vw, char btn, const char *label) {
    int i = sc->n_keys;
    if (i >= AT_MAX_KEYS) return;
    sc->keys[i].btn = btn;
    sc->keys[i].fn_label = sc->keys[i].fn_when = -1;
    snprintf(sc->keys[i].label, sizeof sc->keys[i].label, "%s", label);
    snprintf(vw->key_label[i], sizeof vw->key_label[i], "%s", label);
    vw->key_shown[i] = 1;
    sc->n_keys++;
}

static void at_nc_begin(AtScreen *sc, AtView *vw) {
    memset(sc, 0, sizeof *sc);
    sc->fn_provide = sc->fn_accept = sc->fn_back = sc->fn_focus = sc->fn_change = sc->fn_open = sc->fn_close = sc->fn_counter = sc->fn_page = sc->fn_start = -1;
    sc->fn_alt[0] = sc->fn_alt[1] = sc->fn_alt[2] = -1;
    memset(vw->key_label, 0, sizeof vw->key_label);
    memset(vw->key_shown, 0, sizeof vw->key_shown);
    memset(&vw->ex, 0, sizeof vw->ex);
    vw->ex.media_model = vw->ex.media_ring = AT_NO_MODEL;
    vw->ex.media_tex = -1;
}

static const char *at_nc_state_word(int state) {
    switch (state) {
    case AT_NC_ST_INSTALLED: return "INSTALLED";
    case AT_NC_ST_UPDATE: return "UPDATE";
    case AT_NC_ST_BUSY: return "WORKING";
    case AT_NC_ST_REMOVED: return "REMOVED";
    case AT_NC_ST_UNSUPPORTED: return "NOT YET";
    default: return "";
    }
}

static void at_nc_upper(char *dst, size_t cap, const char *src) {
    size_t i;
    for (i = 0; src[i] && i + 1 < cap; ++i) dst[i] = (char) toupper((unsigned char) src[i]);
    dst[i] = '\0';
}

static const char *at_nc_primary_word(const AtNcRow *r) {
    switch (r->state) {
    case AT_NC_ST_UPDATE: return "Update";
    case AT_NC_ST_INSTALLED: return "Installed";
    case AT_NC_ST_BUSY: return "Working";
    case AT_NC_ST_REMOVED: return "Removed";
    case AT_NC_ST_UNSUPPORTED: return "Not yet";
    default: return "Install";
    }
}

/* the status line for the corner note, most pressing first */
static int at_nc_corner(const AtNcSrc *s, AtNcState *st, AtView *vw, double now_ms) {
    AtNcStatus ss;
    char buf[AT_STR];
    memset(&ss, 0, sizeof ss);
    s->status(s->user, &ss);
    vw->note.text[0] = '\0';
    if (now_ms < st->note_until && st->note[0]) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", st->note);
        vw->note.kind = st->note_kind;
        vw->note.from_ms = st->note_until - 3500.0; vw->note.until_ms = st->note_until;
    } else if (ss.job_state == 1) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", ss.job_msg);
        vw->note.kind = AT_NOTE_INFO; vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
    } else if (ss.job_state == 2) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", ss.job_msg);
        vw->note.kind = AT_NOTE_OK; vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
    } else if (ss.job_state == 3) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", ss.job_msg);
        vw->note.kind = AT_NOTE_ERR; vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
    } else if (ss.syncing) {
        if (ss.total > 0) snprintf(buf, sizeof buf, "Syncing the catalog %d / %d", ss.done, ss.total);
        else snprintf(buf, sizeof buf, "%s", ss.line[0] ? ss.line : "Syncing the catalog");
        snprintf(vw->note.text, sizeof vw->note.text, "%s", buf);
        vw->note.kind = AT_NOTE_INFO; vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
    } else if (ss.locked) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", "Locked while online");
        vw->note.kind = AT_NOTE_WARN; vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
    } else if (ss.restart) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", "Restart the game to use new skins");
        vw->note.kind = AT_NOTE_OK; vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
    } else if (ss.offline && ss.line[0]) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", ss.line);
        vw->note.kind = AT_NOTE_WARN; vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
    }
    return ss.syncing;
}

static void at_nc_fill_ex(const AtNcRow *r, AtView *vw) {
    char kick[AT_STR], fu[28];
    vw->ex.has = 1;
    at_nc_upper(fu, sizeof fu, r->fighter);
    snprintf(kick, sizeof kick, "%s%s%s", fu, fu[0] ? " - " : "", r->kind);
    at_nc_upper(vw->ex.kicker, sizeof vw->ex.kicker, kick);
    snprintf(vw->ex.title, sizeof vw->ex.title, "%s", r->title);
    snprintf(vw->ex.what, sizeof vw->ex.what, "By %s. %d downloads, %d likes.%s%s", r->author[0] ? r->author : "unknown", r->downloads, r->likes,
             r->reason[0] ? " " : "", r->reason);
    snprintf(vw->ex.from_text, sizeof vw->ex.from_text, "%s", AT_NC_CREDIT);
    vw->ex.media_tex = r->tex;
    vw->ex.no_well = r->tex < 0;
    snprintf(vw->ex.media_abbr, sizeof vw->ex.media_abbr, "%.2s", r->fighter[0] ? r->fighter : "NC");
    vw->ex.n_with_text = 0;
    if (r->state != AT_NC_ST_NONE && r->state != AT_NC_ST_UNSUPPORTED) {
        snprintf(vw->ex.with_text[vw->ex.n_with_text++], sizeof vw->ex.with_text[0], "%s", at_nc_state_word(r->state));
    }
}

int at_nc_build(const AtNcSrc *s, AtNcState *st, AtScreen *sc, AtView *vw, double now_ms) {
    int i, last, total, cur, syncing;
    char buf[AT_STR];
    AtNcRow row;
    at_nc_refresh(s, st);
    at_nc_begin(sc, vw);
    total = AT_NC_CONTROLS + st->n;
    if (st->sel >= total) st->sel = total - 1;
    if (st->sel < 0) st->sel = 0;
    cur = st->sel;
    st->top = at_list_scroll(cur, st->top, AT_NC_VISIBLE, total);
    st->base = st->top - 12;
    if (st->base > total - AT_NC_WINDOW) st->base = total - AT_NC_WINDOW;
    if (st->base < 0) st->base = 0;
    snprintf(sc->id, sizeof sc->id, "%s", "mods.nucleus");
    snprintf(sc->title, sizeof sc->title, "%s", "NUCLEUS");
    snprintf(sc->parent[0], sizeof sc->parent[0], "%s", "MAIN MENU");
    snprintf(sc->parent[1], sizeof sc->parent[1], "%s", "MODS");
    sc->n_parents = 2;
    sc->chapter = 4;
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_WIDE;
    sc->port = 1;
    sc->input_feed = 1;
    sc->n_tabs = AT_NC_TABS;
    for (i = 0; i < AT_NC_TABS; ++i) { snprintf(sc->tabs[i].name, sizeof sc->tabs[i].name, "%s", at_nc_tab_name[i]); sc->tabs[i].count = i == st->tab ? st->n : -1; }
    vw->tab = st->tab;
    last = st->base + AT_NC_WINDOW;
    if (last > total) last = total;
    for (i = st->base; i < last; ++i) {
        AtItem *it = &sc->items[sc->n_items++];
        memset(it, 0, sizeof *it);
        if (i < AT_NC_CONTROLS) {
            snprintf(it->group, sizeof it->group, "%s", "FILTERS");
            it->vkind = AT_VAL_STEPPER;
            if (i == 0) {
                snprintf(it->id, sizeof it->id, "%s", "search");
                snprintf(it->label, sizeof it->label, "%s", "Search");
                if (st->edit) snprintf(it->text, sizeof it->text, "%s_", st->edit_buf);
                else snprintf(it->text, sizeof it->text, "%s", st->f.text[0] ? st->f.text : "(anything)");
                snprintf(it->sub, sizeof it->sub, "%s", st->edit ? "Type, or up/down change the letter, right adds one, left deletes. A: done." : "A or X to type a title, author or fighter.");
            } else if (i == 1) {
                snprintf(it->id, sizeof it->id, "%s", "fighter");
                snprintf(it->label, sizeof it->label, "%s", "Fighter");
                snprintf(it->text, sizeof it->text, "%s", st->f.fighter < 0 ? "Any" : s->fighter_label(s->user, st->f.fighter));
                snprintf(it->sub, sizeof it->sub, "%s", "Left and right choose a fighter.");
            } else {
                snprintf(it->id, sizeof it->id, "%s", "sort");
                snprintf(it->label, sizeof it->label, "%s", "Sort by");
                snprintf(it->text, sizeof it->text, "%s", at_nc_sort_name[st->f.sort < 0 || st->f.sort > 4 ? 0 : st->f.sort]);
                snprintf(it->sub, sizeof it->sub, "%s", "Left and right change the order.");
            }
        } else {
            int k = i - AT_NC_CONTROLS;
            snprintf(it->group, sizeof it->group, "%s", "RESULTS");
            snprintf(it->id, sizeof it->id, "r%d", k);
            if (s->row(s->user, st->ids[k], &row)) {
                snprintf(it->label, sizeof it->label, "%s", row.title);
                snprintf(buf, sizeof buf, "%s%s%s", row.fighter, row.fighter[0] ? " - " : "", row.author[0] ? row.author : row.kind);
                snprintf(it->sub, sizeof it->sub, "%s", buf);
                it->vkind = AT_VAL_TEXT;
                it->iflags |= AT_ITEM_RO;
                snprintf(it->text, sizeof it->text, "%s", at_nc_state_word(row.state));
                if (row.state == AT_NC_ST_INSTALLED || row.state == AT_NC_ST_UPDATE) it->flags |= AT_CELL_SELECTED;
                if (row.state == AT_NC_ST_UNSUPPORTED) it->flags |= AT_CELL_DISABLED;
            } else {
                snprintf(it->label, sizeof it->label, "(gone)");
            }
        }
    }
    vw->focus.block = 0;
    vw->focus.index = cur - st->base;
    vw->scroll = st->top - st->base;
    snprintf(vw->counter, sizeof vw->counter, "%d / %d", st->n ? cur - AT_NC_CONTROLS + 1 < 1 ? 0 : cur - AT_NC_CONTROLS + 1 : 0, st->n);
    memset(&row, 0, sizeof row);
    row.tex = -1;
    if (cur >= AT_NC_CONTROLS && s->row(s->user, st->ids[cur - AT_NC_CONTROLS], &row)) {
        at_nc_fill_ex(&row, vw);
        if (st->edit) { at_nc_put_key(sc, vw, 'A', "Done"); at_nc_put_key(sc, vw, 'B', "Cancel"); }
        else {
            at_nc_put_key(sc, vw, 'A', row.state == AT_NC_ST_NONE ? "Install" : row.state == AT_NC_ST_UPDATE ? "Update" : "Details");
            at_nc_put_key(sc, vw, 'B', "Back");
            at_nc_put_key(sc, vw, 'Y', "Details");
            at_nc_put_key(sc, vw, 'X', "Search");
        }
    } else {
        vw->ex.has = 1;
        snprintf(vw->ex.kicker, sizeof vw->ex.kicker, "%s", "SSBM NUCLEUS");
        snprintf(vw->ex.title, sizeof vw->ex.title, "%s", st->n == 0 ? (st->tab == 4 ? "Nothing installed yet" : "Nothing matches") : "Mods from the community");
        snprintf(vw->ex.what, sizeof vw->ex.what, "%s", st->tab == 4 ? "Mods installed from SSBM Nucleus appear here." : "Costumes install as skins on a fighter's select screen. The credit and link stay with each mod.");
        snprintf(vw->ex.from_text, sizeof vw->ex.from_text, "%s", AT_NC_CREDIT);
        vw->ex.no_well = 1;
        if (st->edit) { at_nc_put_key(sc, vw, 'A', "Done"); at_nc_put_key(sc, vw, 'B', "Cancel"); }
        else {
            at_nc_put_key(sc, vw, 'A', cur == 0 ? "Type" : "Change");
            at_nc_put_key(sc, vw, 'B', "Back");
            at_nc_put_key(sc, vw, 'X', "Search");
        }
    }
    syncing = at_nc_corner(s, st, vw, now_ms);
    if (!st->edit && !syncing) at_nc_put_key(sc, vw, 'Z', "Refresh");
    return 1;
}

static void at_nc_cycle(const AtNcSrc *s, AtNcState *st, int row, int dir) {
    if (row == 1) {
        int n = s->fighters(s->user), v = st->f.fighter + dir;
        if (v < -1) v = n - 1;
        if (v >= n) v = -1;
        st->f.fighter = v;
    } else {
        st->f.sort = (st->f.sort + dir + 5) % 5;
    }
    st->stale = 1;
    st->top = 0;
}

/* a typed character from the keyboard while the search is being edited: 8 backspace, 13 enter, 27 escape, else a letter, digit or space */
void at_nc_type(AtNcState *st, int ch) {
    size_t n = strlen(st->edit_buf);
    if (!st->edit) return;
    if (ch == 13) { st->edit = 0; snprintf(st->f.text, sizeof st->f.text, "%s", st->edit_buf); st->stale = 1; st->sel = AT_NC_CONTROLS; st->top = 0; return; }
    if (ch == 27) { st->edit = 0; snprintf(st->edit_buf, sizeof st->edit_buf, "%s", st->edit_old); return; }
    if (ch == 8) { if (n) st->edit_buf[n - 1] = '\0'; return; }
    if (ch >= 32 && ch < 127 && n + 1 < sizeof st->edit_buf && !(ch == ' ' && (n == 0 || st->edit_buf[n - 1] == ' '))) { st->edit_buf[n] = (char) ch; st->edit_buf[n + 1] = '\0'; }
}

static void at_nc_start_edit(AtNcState *st) {
    st->edit = 1;
    snprintf(st->edit_buf, sizeof st->edit_buf, "%s", st->f.text);
    snprintf(st->edit_old, sizeof st->edit_old, "%s", st->f.text);
    st->sel = 0;
}

static void at_nc_pick(AtNcState *st, int dir) {
    static const char set[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    size_t n = strlen(st->edit_buf), len = sizeof set - 1, k;
    if (n == 0) { if (n + 1 < sizeof st->edit_buf) { st->edit_buf[0] = dir > 0 ? set[0] : set[len - 1]; st->edit_buf[1] = '\0'; } return; }
    for (k = 0; k < len && set[k] != toupper((unsigned char) st->edit_buf[n - 1]); ++k) {}
    if (k == len) k = dir > 0 ? len - 1 : 0;
    k = (k + len + (size_t) (dir > 0 ? 1 : len - 1)) % len;
    st->edit_buf[n - 1] = set[k];
}

void at_nc_event(const AtNcSrc *s, AtNcState *st, const AtEvent *e, double now_ms, AtNcAction *act) {
    int total = AT_NC_CONTROLS + st->n, cur = st->sel;
    AtNcRow row;
    memset(act, 0, sizeof *act);
    if (st->edit) {
        if (e->type == AT_EV_BACK) at_nc_type(st, 27);
        else if (e->type == AT_EV_ACCEPT) at_nc_type(st, 13);
        else if (e->type == AT_EV_MOVE) {
            if (e->a == AT_DIR_UP) at_nc_pick(st, 1);
            else if (e->a == AT_DIR_DOWN) at_nc_pick(st, -1);
            else if (e->a == AT_DIR_RIGHT) at_nc_type(st, 'A');
            else if (e->a == AT_DIR_LEFT) at_nc_type(st, 8);
        }
        return;
    }
    if (e->type == AT_EV_BACK) { act->kind = AT_NA_BACK; return; }
    if (e->type == AT_EV_PAGE) { st->tab = (st->tab + (e->a < 0 ? AT_NC_TABS - 1 : 1)) % AT_NC_TABS; at_nc_apply_tab(st); return; }
    if (e->type == AT_EV_ALT && e->a == 'X') { at_nc_start_edit(st); return; }
    if (e->type == AT_EV_ALT && e->a == 'Z') { act->kind = AT_NA_ACT; act->what = AT_NC_ACT_REFRESH; return; }
    if (total <= 0) return;
    if (e->type == AT_EV_MOVE) {
        if (e->a == AT_DIR_UP) st->sel = (cur + total - 1) % total;
        else if (e->a == AT_DIR_DOWN) st->sel = (cur + 1) % total;
        else if (cur == 1 || cur == 2) at_nc_cycle(s, st, cur, e->a == AT_DIR_RIGHT ? 1 : -1);
        else if (cur == 0 && e->a == AT_DIR_RIGHT) at_nc_start_edit(st);
        return;
    }
    if (e->type == AT_EV_SCROLL) { int v = cur + (e->a > 0 ? 1 : -1); st->sel = v < 0 ? 0 : v >= total ? total - 1 : v; return; }
    if (e->type == AT_EV_FOCUS) { int v = st->base + e->b; if (v >= 0 && v < total) st->sel = v; return; }
    if (e->type == AT_EV_ACCEPT) {
        if (cur == 0) { at_nc_start_edit(st); return; }
        if (cur < AT_NC_CONTROLS) { at_nc_cycle(s, st, cur, 1); return; }
        if (!s->row(s->user, st->ids[cur - AT_NC_CONTROLS], &row)) return;
        if (row.state == AT_NC_ST_NONE || row.state == AT_NC_ST_UPDATE) { act->kind = AT_NA_ACT; act->id = row.id; act->what = AT_NC_ACT_INSTALL; }
        else if (row.state == AT_NC_ST_UNSUPPORTED) at_nc_say(st, row.reason[0] ? row.reason : "This cannot be installed yet.", AT_NOTE_WARN, now_ms);
        else { st->mode = 1; st->detail_id = row.id; st->detail_sel = 0; }    /* installed: A opens the detail, where it can be removed */
        (void) now_ms;
        return;
    }
    if (e->type == AT_EV_ALT && e->a == 'Y' && cur >= AT_NC_CONTROLS) { st->mode = 1; st->detail_id = st->ids[cur - AT_NC_CONTROLS]; st->detail_sel = 0; }
}

/* ---- the detail ---- */

static void at_nc_add_row(AtScreen *sc, const char *id, const char *label, const char *sub, const char *text) {
    AtItem *it;
    if (sc->n_items >= AT_MAX_ITEMS) return;
    it = &sc->items[sc->n_items++];
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id);
    snprintf(it->label, sizeof it->label, "%s", label);
    snprintf(it->sub, sizeof it->sub, "%s", sub);
    if (text && text[0]) { it->vkind = AT_VAL_TEXT; it->iflags |= AT_ITEM_RO; snprintf(it->text, sizeof it->text, "%s", text); }
}

int at_nc_detail_build(const AtNcSrc *s, AtNcState *st, AtScreen *sc, AtView *vw, double now_ms) {
    AtNcRow r;
    char buf[AT_STR], up[AT_STR];
    int ok;
    at_nc_begin(sc, vw);
    memset(&r, 0, sizeof r);
    r.tex = -1;
    ok = s->row(s->user, st->detail_id, &r);
    if (!ok) return 0;
    at_nc_upper(up, sizeof up, r.title);
    snprintf(sc->id, sizeof sc->id, "%s", "mods.nucleus.detail");
    snprintf(sc->title, sizeof sc->title, "%s", up);
    snprintf(sc->parent[0], sizeof sc->parent[0], "%s", "MODS");
    snprintf(sc->parent[1], sizeof sc->parent[1], "%s", "NUCLEUS");
    sc->n_parents = 2;
    sc->chapter = 4;
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_WIDE;
    sc->port = 1;
    sc->input_feed = 1;
    at_nc_add_row(sc, "act", at_nc_primary_word(&r),
                  r.state == AT_NC_ST_UNSUPPORTED ? r.reason : r.state == AT_NC_ST_UPDATE ? "A newer version is on SSBM Nucleus." :
                  r.state == AT_NC_ST_INSTALLED ? "Already installed." : r.state == AT_NC_ST_REMOVED ? "Taken down from SSBM Nucleus; your copy stays." : "Downloads it and adds it as a skin.", "");
    if (r.state == AT_NC_ST_INSTALLED || r.state == AT_NC_ST_UPDATE || r.state == AT_NC_ST_REMOVED) at_nc_add_row(sc, "rm", "Uninstall", "Deletes the skin from your mods folder.", "");
    at_nc_add_row(sc, "page", "Open the page", r.page[0] ? r.page : "ssbmnucleus.net", "");
    at_nc_add_row(sc, "who", "Author", r.author[0] ? r.author : "unknown", "");
    at_nc_add_row(sc, "fighter", "Fighter", r.fighter[0] ? r.fighter : "-", r.kind);
    snprintf(buf, sizeof buf, "%d costume file%s", r.ncostumes, r.ncostumes == 1 ? "" : "s");
    at_nc_add_row(sc, "files", "Contains", buf, "");
    snprintf(buf, sizeof buf, "%d downloads, %d likes", r.downloads, r.likes);
    at_nc_add_row(sc, "pop", "On SSBM Nucleus", buf, "");
    at_nc_add_row(sc, "upd", "Updated", r.updated, "");
    at_nc_add_row(sc, "credit", "Credit", AT_NC_CREDIT, "");
    if (st->detail_sel >= sc->n_items) st->detail_sel = sc->n_items - 1;
    if (st->detail_sel < 0) st->detail_sel = 0;
    vw->focus.block = 0;
    vw->focus.index = st->detail_sel;
    vw->scroll = st->detail_sel >= AT_MODS_DETAIL_VISIBLE ? st->detail_sel - AT_MODS_DETAIL_VISIBLE + 1 : 0;
    at_nc_fill_ex(&r, vw);
    snprintf(vw->ex.what, sizeof vw->ex.what, "%s", r.desc[0] ? r.desc : "No description.");
    if (!strcmp(sc->items[st->detail_sel].id, "act") && (r.state == AT_NC_ST_NONE || r.state == AT_NC_ST_UPDATE)) at_nc_put_key(sc, vw, 'A', r.state == AT_NC_ST_UPDATE ? "Update" : "Install");
    else if (!strcmp(sc->items[st->detail_sel].id, "rm")) at_nc_put_key(sc, vw, 'A', "Uninstall");
    else if (!strcmp(sc->items[st->detail_sel].id, "page")) at_nc_put_key(sc, vw, 'A', "Open");
    at_nc_put_key(sc, vw, 'B', "Back");
    at_nc_corner(s, st, vw, now_ms);
    return 1;
}

void at_nc_detail_event(const AtNcSrc *s, AtNcState *st, const AtEvent *e, double now_ms, AtNcAction *act) {
    static AtScreen sc;
    AtView vw;
    const char *id;
    memset(act, 0, sizeof *act);
    memset(&vw, 0, sizeof vw);
    if (!at_nc_detail_build(s, st, &sc, &vw, 0.0)) { st->mode = 0; return; }
    if (e->type == AT_EV_BACK) { st->mode = 0; return; }
    if (e->type == AT_EV_MOVE && e->a == AT_DIR_UP) st->detail_sel = (st->detail_sel + sc.n_items - 1) % sc.n_items;
    else if (e->type == AT_EV_MOVE && e->a == AT_DIR_DOWN) st->detail_sel = (st->detail_sel + 1) % sc.n_items;
    else if (e->type == AT_EV_SCROLL) { int v = st->detail_sel + (e->a > 0 ? 1 : -1); st->detail_sel = v < 0 ? 0 : v >= sc.n_items ? sc.n_items - 1 : v; }
    else if (e->type == AT_EV_FOCUS && e->b >= 0 && e->b < sc.n_items) st->detail_sel = e->b;
    else if (e->type == AT_EV_ACCEPT && st->detail_sel >= 0 && st->detail_sel < sc.n_items) {
        id = sc.items[st->detail_sel].id;
        act->id = st->detail_id;
        if (!strcmp(id, "act")) { AtNcRow r; if (s->row(s->user, st->detail_id, &r) && (r.state == AT_NC_ST_NONE || r.state == AT_NC_ST_UPDATE)) { act->kind = AT_NA_ACT; act->what = AT_NC_ACT_INSTALL; } else at_nc_say(st, r.reason[0] ? r.reason : "Nothing to do.", AT_NOTE_INFO, now_ms); }
        else if (!strcmp(id, "rm")) { act->kind = AT_NA_ACT; act->what = AT_NC_ACT_UNINSTALL; }
        else if (!strcmp(id, "page")) { act->kind = AT_NA_ACT; act->what = AT_NC_ACT_PAGE; }
    }
}

#endif
