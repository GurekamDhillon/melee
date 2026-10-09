/* gw_ui_nucleus.inc - the NUCLEUS screens of the MODS door: browse, search, filter, queue and download mods from SSBM Nucleus (ssbmnucleus.net), as a pure model.
 * Like gw_ui_mods.c it knows nothing of the game, the network or the disk: a source (AtNcSrc) answers for the rows, the model builds the AtScreen / AtView
 * and applies events, and says what it wants done (AtNcAction). The engine half that answers the source is gw_nucleus_engine.inc; the door that draws and
 * feeds it is gw_script_ui_mods.inc (mode 2: the list, mode 3: a mod's detail).
 *
 * Included, not compiled on its own (no new .c to hand-add to the link list): by gw_script.c through gw_script_ui_mods.inc, and by
 * pc/tests/atlas_nucleus_test.c. It needs gw_ui_screen.h / gw_ui_input.h / gw_ui_parts.h / gw_ui_layout.h / gw_ui_render.h in scope.
 *
 * LAYOUT. One full-width list under a tab strip: COSTUMES / STAGES / EFFECTS / MORE / INSTALLED / QUEUE. Every mod is a tall row with its PICTURE at the left (a
 * placeholder box while the picture loads), the title wrapped to two lines, a line of facts under it, and tags at the right edge (QUEUED #2, 47%, INSTALLED,
 * UPDATE). The filters (Search, Fighter, Show, Sort) are the first rows of the list; the list opens scrolled past them, and Up reveals them. While a filter is on,
 * the corner note says so. Text is wrapped and sized by the renderer from the row's real width; nothing is cut to a fixed number of characters.
 *
 * INPUT. A (click, Space): add the mod to the download queue, or take it out again. START (Enter): start the downloads, or pause them. Y: details (pictures, the
 * description). X: search (on the QUEUE tab: move the mod up). Z: refresh the catalog (on the QUEUE tab: clear the queue, after a confirmation). L / R (Tab): change
 * tab. B leaves; it never touches the queue. The queue is the engine's and is saved on every change.
 * A search is typed on the keyboard when there is one and by an arcade picker otherwise (up / down change the last letter, right adds one, left deletes).
 * The credit "Mods from SSBM Nucleus - ssbmnucleus.net" is under the list pane on every Nucleus screen. */
#ifndef GW_UI_NUCLEUS_INC
#define GW_UI_NUCLEUS_INC

#include <ctype.h>
#include <stdio.h>
#include <string.h>

#define AT_NC_TABS 6
#define AT_NC_TAB_INSTALLED 4
#define AT_NC_TAB_QUEUE 5
#define AT_NC_IDS 20000
#define AT_NC_CONTROLS 4
#define AT_NC_ROW_H 62                  /* a mod's row: the picture box is 51 px high, the title takes two lines and the facts one */
#define AT_NC_SHOT_H 86
#define AT_NC_ABOUT_H 108
#define AT_NC_CREDIT "Mods from SSBM Nucleus - ssbmnucleus.net"
#define AT_NC_SHOW_N 5

enum { AT_NC_ST_NONE, AT_NC_ST_INSTALLED, AT_NC_ST_UPDATE, AT_NC_ST_BUSY, AT_NC_ST_REMOVED, AT_NC_ST_UNSUPPORTED };
enum { AT_NC_Q_NONE = -1, AT_NC_Q_QUEUED = 0, AT_NC_Q_RUNNING = 1, AT_NC_Q_DONE = 2, AT_NC_Q_FAILED = 3 };      /* the same numbers as NQ_* in gw_nucleus_queue.h */
enum { AT_NC_ACT_QUEUE = 1, AT_NC_ACT_UNINSTALL, AT_NC_ACT_PAGE, AT_NC_ACT_REFRESH, AT_NC_ACT_START, AT_NC_ACT_Q_UP, AT_NC_ACT_Q_DOWN, AT_NC_ACT_Q_REMOVE,
       AT_NC_ACT_Q_RETRY, AT_NC_ACT_Q_CLEAR, AT_NC_ACT_Q_TIDY };
enum { AT_NC_VIEW_BROWSE, AT_NC_VIEW_INSTALLED, AT_NC_VIEW_QUEUE };
enum { AT_NC_SHOW_ALL, AT_NC_SHOW_NOT_INSTALLED, AT_NC_SHOW_INSTALLED, AT_NC_SHOW_UPDATE, AT_NC_SHOW_QUEUED };
enum { AT_NC_PIC_THUMB = 0, AT_NC_PIC_SHOT1 = 1, AT_NC_PIC_STOCK = 10 };            /* AtNcSrc.pic: 0 the list picture, 1.. the screenshots, 10 a costume's stock icon */

typedef struct { char text[48]; unsigned type_mask; int fighter, sort, view, show; } AtNcFilter;      /* fighter -1: any; type_mask 0: any; show: AT_NC_SHOW_* */
typedef struct {
    int id;
    char title[80], author[40], fighter[28], kind[16], reason[64], updated[24], page[120], desc[320], tags[96], qreason[120];
    int downloads, likes, state, installable, ncostumes, tex, files;
    int thumb;                                  /* the list picture's texture slot, or -1 (none yet, or none at all) */
    int qstate, qplace, qpct;                   /* the download queue: AT_NC_Q_* or -1 not in it; 1-based place in line; percent while running */
    int nshots;                                 /* screenshots the post lists (the detail shows up to three) */
} AtNcRow;
typedef struct {
    char line[100]; int syncing, done, total, offline, restart, locked, job_state; char job_msg[140]; int catalog_n;     /* job_state: the last message, 0 none 1 working 2 ok 3 failed */
    int q_pending, q_failed, q_processing, q_running_pct;                                                              /* the queue: entries waiting or running, failed entries, Start pressed, % of the one running */
    char q_running_title[80];
} AtNcStatus;

typedef struct AtNcSrc {
    void *user;
    int (*filter)(void *u, const AtNcFilter *f, int *ids, int cap, unsigned *version);     /* the matching mod ids in the filter's order; version changes when the catalog does */
    int (*row)(void *u, int id, AtNcRow *out);                                              /* 0: no such mod */
    void (*status)(void *u, AtNcStatus *out);
    int (*act)(void *u, int id, int what);                                                  /* 1: started / done; 0: refused (status.job_msg says why) */
    int (*fighters)(void *u);
    const char *(*fighter_label)(void *u, int i);
    unsigned (*version)(void *u);                                                           /* the catalog version now (0: the same as filter's last answer) */
    int (*pic)(void *u, int id, int which, int *tex);                                       /* a picture of a mod: 1 it has one (tex: its slot, or -1 while it loads), 0 none. Asked only for rows in view. */
    void (*pref_get)(void *u, AtNcFilter *f, int *tab);                                     /* the filters remembered from the last session (NULL: none kept) */
    void (*pref_set)(void *u, const AtNcFilter *f, int tab);
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
    int confirm, confirm_focus; double confirm_from;      /* the "clear the queue?" dialog */
    int dtop;                                             /* the detail's own first visible row */
    int follow;                                           /* a mod id the selection keeps following after the queue reorders it (0: none) */
    int prefs_loaded;
    char saved_text[48]; int saved_fighter, saved_show, saved_sort, saved_tab;     /* what pref_set was last given: a change is saved once, wherever it was made */
} AtNcState;

enum { AT_NA_NONE, AT_NA_BACK, AT_NA_ACT };
typedef struct { int kind, id, what; } AtNcAction;

static const char *const at_nc_tab_name[AT_NC_TABS] = { "COSTUMES", "STAGES", "EFFECTS", "MORE", "INSTALLED", "QUEUE" };
static const char *const at_nc_sort_name[5] = { "Recently updated", "Newest", "Most downloaded", "Most liked", "Title" };
static const char *const at_nc_show_name[AT_NC_SHOW_N] = { "All", "Not installed", "Installed", "Update available", "Queued" };

/* the controls at the top of a tab's list: Search, Fighter, Show, Sort; the INSTALLED tab has Search and Fighter, the QUEUE none */
static int at_nc_ctl(const AtNcState *st) { return st->tab == AT_NC_TAB_QUEUE ? 0 : st->tab == AT_NC_TAB_INSTALLED ? 2 : AT_NC_CONTROLS; }

static void at_nc_state_init(AtNcState *st) {
    memset(st, 0, sizeof *st);
    st->f.fighter = -1;
    st->f.type_mask = 1u;               /* the costumes tab */
    st->stale = 1;
    st->sel = st->top = AT_NC_CONTROLS; /* the first result has the focus, the controls are scrolled out of view above it */
}

static void at_nc_apply_tab(AtNcState *st) {
    st->f.view = st->tab == AT_NC_TAB_QUEUE ? AT_NC_VIEW_QUEUE : st->tab == AT_NC_TAB_INSTALLED ? AT_NC_VIEW_INSTALLED : AT_NC_VIEW_BROWSE;
    st->f.type_mask = st->tab == 0 ? 1u << 0 : st->tab == 1 ? 1u << 1 : st->tab == 2 ? 1u << 2 : st->tab == 3 ? ((1u << 3) | (1u << 4) | (1u << 5) | (1u << 6)) : 0;
    st->stale = 1;
    st->sel = st->top = at_nc_ctl(st);
    st->follow = 0;
}

static void at_nc_say(AtNcState *st, const char *text, int kind, double now_ms) {
    snprintf(st->note, sizeof st->note, "%s", text);
    st->note_kind = kind;
    st->note_until = now_ms + 3500.0;
}

/* the filters of the last session, once, when the screen first builds */
static void at_nc_prefs_apply(const AtNcSrc *s, AtNcState *st) {
    AtNcFilter f = st->f;
    int tab = st->tab;
    st->prefs_loaded = 1;
    if (!s->pref_get) return;
    s->pref_get(s->user, &f, &tab);
    if (tab < 0 || tab >= AT_NC_TABS) tab = 0;
    if (f.show < 0 || f.show >= AT_NC_SHOW_N) f.show = 0;
    if (f.sort < 0 || f.sort > 4) f.sort = 0;
    if (f.fighter < -1 || (s->fighters && f.fighter >= s->fighters(s->user))) f.fighter = -1;
    st->f.fighter = f.fighter; st->f.show = f.show; st->f.sort = f.sort;
    snprintf(st->f.text, sizeof st->f.text, "%s", f.text);
    st->tab = tab;
    at_nc_apply_tab(st);
    snprintf(st->saved_text, sizeof st->saved_text, "%s", st->f.text);       /* what was just loaded is what is on disk */
    st->saved_fighter = st->f.fighter; st->saved_show = st->f.show; st->saved_sort = st->f.sort; st->saved_tab = st->tab;
}

/* the filters and the tab are remembered across sessions: whatever changed them (a key, the mouse, the typed search) is saved here, once */
static void at_nc_save_prefs(const AtNcSrc *s, AtNcState *st) {
    if (!s->pref_set || (strcmp(st->saved_text, st->f.text) == 0 && st->saved_fighter == st->f.fighter && st->saved_show == st->f.show && st->saved_sort == st->f.sort && st->saved_tab == st->tab)) return;
    snprintf(st->saved_text, sizeof st->saved_text, "%s", st->f.text);
    st->saved_fighter = st->f.fighter; st->saved_show = st->f.show; st->saved_sort = st->f.sort; st->saved_tab = st->tab;
    s->pref_set(s->user, &st->f, st->tab);
}

/* The height the model assumes for the list pane: the primary place under the tab strip, less the credit strip. It is the renderer's own arithmetic. */
static float at_nc_pane_h(void) {
    AtLayout L;
    AtSplit sp;
    at_layout(853.0f, AT_PRESET_NONE, &L);
    at_layout_split(&L, 1, AT_BAND_NONE, &sp);
    return sp.grid.h - AT_CREDIT_H;
}

static void at_nc_refresh(const AtNcSrc *s, AtNcState *st) {
    unsigned v = 0, cur = s->version ? s->version(s->user) : 0;
    int ctl = at_nc_ctl(st);
    if (!st->prefs_loaded) at_nc_prefs_apply(s, st);
    else at_nc_save_prefs(s, st);
    if (!st->stale && cur == st->ver) return;
    st->n = s->filter(s->user, &st->f, st->ids, AT_NC_IDS, &v);
    st->ver = v ? v : cur;
    st->stale = 0;
    if (st->follow) {                                   /* a reordered queue entry keeps the focus */
        int i;
        for (i = 0; i < st->n; ++i) if (st->ids[i] == st->follow) { st->sel = ctl + i; break; }
        st->follow = 0;
    }
    if (st->sel > ctl + (st->n > 0 ? st->n : 1) - 1) st->sel = ctl + (st->n > 0 ? st->n : 1) - 1;
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
    snprintf(sc->credit, sizeof sc->credit, "%s", AT_NC_CREDIT);
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
static int at_nc_state_tone(int state) {
    switch (state) {
    case AT_NC_ST_INSTALLED: return AT_TAG_JADE;
    case AT_NC_ST_UPDATE: return AT_TAG_SUN;
    case AT_NC_ST_REMOVED: return AT_TAG_ROSE;
    default: return AT_TAG_PLAIN;
    }
}

static void at_nc_upper(char *dst, size_t cap, const char *src) {
    size_t i;
    for (i = 0; src[i] && i + 1 < cap; ++i) dst[i] = (char) toupper((unsigned char) src[i]);
    dst[i] = '\0';
}

static void at_nc_join(char *out, size_t cap, const char *part) {
    size_t l = strlen(out);
    if (!part[0] || l + 1 >= cap) return;
    snprintf(out + l, cap - l, "%s%s", l ? " - " : "", part);
}

/* the line under a mod's title: fighter, author, size of the post, popularity. A mod that cannot be installed says why instead of the numbers. */
static void at_nc_meta(const AtNcRow *r, char *out, size_t cap) {
    char tmp[64];
    out[0] = '\0';
    if (r->state == AT_NC_ST_UNSUPPORTED && r->reason[0]) at_nc_join(out, cap, r->reason);     /* why not comes first: it is the thing to read */
    at_nc_join(out, cap, r->fighter);
    if (r->author[0]) { snprintf(tmp, sizeof tmp, "by %s", r->author); at_nc_join(out, cap, tmp); }
    if (!r->fighter[0]) at_nc_join(out, cap, r->kind);
    if (r->ncostumes > 1) { snprintf(tmp, sizeof tmp, "%d costumes", r->ncostumes); at_nc_join(out, cap, tmp); }
    snprintf(tmp, sizeof tmp, "%d downloads", r->downloads);
    at_nc_join(out, cap, tmp);
}

static void at_nc_abbr(const AtNcRow *r, char *out) {
    const char *src = r->fighter[0] ? r->fighter : r->kind;
    out[0] = src[0] ? (char) toupper((unsigned char) src[0]) : '?';
    out[1] = src[0] && src[1] ? (char) toupper((unsigned char) src[1]) : '\0';
    out[2] = '\0';
}

static const char *at_nc_qword(int q) { return q == AT_NC_Q_QUEUED ? "QUEUED" : q == AT_NC_Q_RUNNING ? "DOWNLOADING" : q == AT_NC_Q_DONE ? "DONE" : q == AT_NC_Q_FAILED ? "FAILED" : ""; }

/* the tags at a mod row's right edge: the queue first (QUEUED #2, 47%, FAILED), then the install state; only the second one when the mod is not in the queue */
static void at_nc_chips(const AtNcRow *r, AtItem *it) {
    const char *sw = at_nc_state_word(r->state);
    char *c1 = it->chip, *c2 = it->chip2;
    int *t1 = &it->chip_tone, *t2 = &it->chip2_tone;
    it->chip[0] = it->chip2[0] = '\0';
    if (r->qstate == AT_NC_Q_QUEUED) { snprintf(c1, sizeof it->chip, "QUEUED #%d", r->qplace); *t1 = AT_TAG_EMBER; }
    else if (r->qstate == AT_NC_Q_RUNNING) { snprintf(c1, sizeof it->chip, "%d%%", r->qpct); *t1 = AT_TAG_JADE; }
    else if (r->qstate == AT_NC_Q_FAILED) { snprintf(c1, sizeof it->chip, "FAILED"); *t1 = AT_TAG_ROSE; }
    else if (r->qstate == AT_NC_Q_DONE) { snprintf(c1, sizeof it->chip, "DONE"); *t1 = AT_TAG_JADE; }
    if (sw[0] && r->state != AT_NC_ST_BUSY) {
        if (c1[0]) { snprintf(c2, sizeof it->chip2, "%s", sw); *t2 = at_nc_state_tone(r->state); }
        else { snprintf(c1, sizeof it->chip, "%s", sw); *t1 = at_nc_state_tone(r->state); }
    }
}

/* one mod as a rich list row. in_queue: the QUEUE tab, where the line under the title says where the entry stands. */
static void at_nc_fill_row(const AtNcSrc *s, const AtNcRow *r, AtItem *it, int in_queue, int show_pic) {
    char meta[AT_TEXT], abbr[3];
    int tex = -1, has = 0;
    memset(it, 0, sizeof *it);
    snprintf(it->title, sizeof it->title, "%s", r->title);
    snprintf(it->label, sizeof it->label, "%.*s", (int) sizeof it->label - 1, r->title);
    it->row_h = AT_NC_ROW_H;
    it->thumb_on = 1;
    it->thumb_tex = -1;
    if (show_pic && s->pic) has = s->pic(s->user, r->id, AT_NC_PIC_THUMB, &tex);
    it->thumb_tex = has ? tex : -1;
    if (show_pic && r->ncostumes > 0 && s->pic) { int t2 = -1; if (s->pic(s->user, r->id, AT_NC_PIC_STOCK, &t2)) { it->thumb2_on = 1; it->thumb2_tex = t2; } }
    at_nc_abbr(r, abbr);
    snprintf(it->thumb_abbr, sizeof it->thumb_abbr, "%s", abbr);
    if (in_queue) {
        if (r->qstate == AT_NC_Q_FAILED) snprintf(meta, sizeof meta, "%s%s", r->qreason[0] ? r->qreason : "It could not be installed.", " A retries it.");
        else if (r->qstate == AT_NC_Q_RUNNING) snprintf(meta, sizeof meta, "Downloading and installing - %d%%", r->qpct);
        else if (r->qstate == AT_NC_Q_DONE) snprintf(meta, sizeof meta, "Installed. Restart the game to use it. A dismisses it.");
        else { at_nc_meta(r, meta, sizeof meta); }
    } else at_nc_meta(r, meta, sizeof meta);
    snprintf(it->meta, sizeof it->meta, "%s", meta);
    at_nc_chips(r, it);
    if (r->state == AT_NC_ST_INSTALLED || r->state == AT_NC_ST_UPDATE) it->flags |= AT_CELL_SELECTED;
    if (r->state == AT_NC_ST_UNSUPPORTED) it->flags |= AT_CELL_DISABLED;
}

static void at_nc_corner_set(AtView *vw, const char *text, int kind, double now_ms) {
    snprintf(vw->note.text, sizeof vw->note.text, "%s", text);
    vw->note.kind = kind;
    vw->note.from_ms = now_ms; vw->note.until_ms = now_ms + 1000.0;
}

/* what the filters currently hide, in words (empty when nothing is filtered) */
static void at_nc_filter_words(const AtNcSrc *s, const AtNcState *st, char *out, size_t cap) {
    out[0] = '\0';
    if (st->tab == AT_NC_TAB_QUEUE) return;
    if (st->tab != AT_NC_TAB_INSTALLED && st->f.show != AT_NC_SHOW_ALL) at_nc_join(out, cap, at_nc_show_name[st->f.show]);
    if (st->f.fighter >= 0) at_nc_join(out, cap, s->fighter_label(s->user, st->f.fighter));
    if (st->f.text[0]) { char t[64]; snprintf(t, sizeof t, "\"%s\"", st->f.text); at_nc_join(out, cap, t); }
}

/* the corner note, most pressing first. Returns whether the catalog is syncing (the Z hint hides then). */
static int at_nc_corner(const AtNcSrc *s, AtNcState *st, AtView *vw, double now_ms) {
    AtNcStatus ss;
    char buf[AT_STR * 2];
    memset(&ss, 0, sizeof ss);
    s->status(s->user, &ss);
    vw->note.text[0] = '\0';
    if (now_ms < st->note_until && st->note[0]) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", st->note);
        vw->note.kind = st->note_kind;
        vw->note.from_ms = st->note_until - 3500.0; vw->note.until_ms = st->note_until;
    } else if (ss.job_state == 3) at_nc_corner_set(vw, ss.job_msg, AT_NOTE_ERR, now_ms);
    else if (ss.job_state == 2) at_nc_corner_set(vw, ss.job_msg, AT_NOTE_OK, now_ms);
    else if (ss.job_state == 1) at_nc_corner_set(vw, ss.job_msg, AT_NOTE_INFO, now_ms);
    else if (ss.q_processing && ss.locked && ss.q_pending > 0) at_nc_corner_set(vw, "Downloads wait while you are online", AT_NOTE_WARN, now_ms);
    else if (ss.q_processing && ss.q_pending > 0 && ss.q_running_title[0]) {
        snprintf(buf, sizeof buf, "Installing %s %d%%", ss.q_running_title, ss.q_running_pct);
        at_nc_corner_set(vw, buf, AT_NOTE_INFO, now_ms);
    } else if (ss.q_processing && ss.q_pending > 0) {
        snprintf(buf, sizeof buf, "Downloading: %d waiting", ss.q_pending);
        at_nc_corner_set(vw, buf, AT_NOTE_INFO, now_ms);
    } else if (ss.syncing) {
        if (ss.total > 0) snprintf(buf, sizeof buf, "Syncing the catalog %d / %d", ss.done, ss.total);
        else snprintf(buf, sizeof buf, "%s", ss.line[0] ? ss.line : "Syncing the catalog");
        at_nc_corner_set(vw, buf, AT_NOTE_INFO, now_ms);
    } else if (ss.locked) at_nc_corner_set(vw, "Locked while online", AT_NOTE_WARN, now_ms);
    else if (ss.restart) at_nc_corner_set(vw, "Restart the game to use new skins", AT_NOTE_OK, now_ms);
    else if (ss.offline && ss.line[0]) at_nc_corner_set(vw, ss.line, AT_NOTE_WARN, now_ms);
    else if (st->mode == 0) {
        at_nc_filter_words(s, st, buf, sizeof buf);
        if (buf[0]) {
            char tmp[AT_STR * 2];
            snprintf(tmp, sizeof tmp, "Showing: %s", buf);
            at_nc_corner_set(vw, tmp, AT_NOTE_INFO, now_ms);
        }
    }
    return ss.syncing;
}

static void at_nc_dialog(const AtNcState *st, AtView *vw, int pending) {
    AtDialog *d = &vw->dialog;
    if (!st->confirm) { d->open = 0; return; }
    memset(d, 0, sizeof *d);
    snprintf(d->title, sizeof d->title, "%s", "Clear the queue?");
    snprintf(d->body, sizeof d->body, "This takes %d waiting or failed mod%s out of the queue. Mods that are already installed stay installed.", pending, pending == 1 ? "" : "s");
    d->btn[0] = 'A'; d->btn[1] = 'B';
    snprintf(d->label[0], sizeof d->label[0], "%s", "Clear");
    snprintf(d->label[1], sizeof d->label[1], "%s", "Keep");
    d->n = 2;
    d->focus = st->confirm_focus ? 1 : 0;
    d->from_ms = st->confirm_from;
    d->open = 1;
}

static void at_nc_info_row(AtScreen *sc, const char *id, const char *label, const char *sub) {
    AtItem *it;
    if (sc->n_items >= AT_MAX_ITEMS) return;
    it = &sc->items[sc->n_items++];
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id);
    snprintf(it->label, sizeof it->label, "%s", label);
    snprintf(it->sub, sizeof it->sub, "%s", sub);
    it->iflags |= AT_ITEM_RO;
}

static void at_nc_start_key(const AtNcStatus *ss, AtScreen *sc, AtView *vw) {
    char lab[24];
    if (ss->q_processing && ss->q_pending > 0) at_nc_put_key(sc, vw, 'S', "Pause");
    else if (ss->q_pending > 0) { snprintf(lab, sizeof lab, "Download %d", ss->q_pending); at_nc_put_key(sc, vw, 'S', lab); }
}

int at_nc_build(const AtNcSrc *s, AtNcState *st, AtScreen *sc, AtView *vw, double now_ms) {
    int i, last, total, cur, syncing, ctl, nrows, rows_view;
    char buf[AT_STR];
    float pane_h = at_nc_pane_h();
    AtNcRow row;
    AtNcStatus ss;
    at_nc_refresh(s, st);
    at_nc_begin(sc, vw);
    memset(&ss, 0, sizeof ss);
    s->status(s->user, &ss);
    ctl = at_nc_ctl(st);
    nrows = st->n > 0 ? st->n : 1;                                  /* an empty list shows one note row so it is not a blank pane */
    total = ctl + nrows;
    if (st->sel >= total) st->sel = total - 1;
    if (st->sel < 0) st->sel = 0;
    cur = st->sel;
    st->base = cur - 24;
    if (st->base > total - AT_MAX_ITEMS) st->base = total - AT_MAX_ITEMS;
    if (st->base < 0) st->base = 0;
    snprintf(sc->id, sizeof sc->id, "%s", "mods.nucleus");
    snprintf(sc->title, sizeof sc->title, "%s", "NUCLEUS");
    snprintf(sc->parent[0], sizeof sc->parent[0], "%s", "MAIN MENU");
    snprintf(sc->parent[1], sizeof sc->parent[1], "%s", "MODS");
    sc->n_parents = 2;
    sc->chapter = 4;
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_NONE;                                    /* the whole width is the list: the pictures and the titles need it */
    sc->port = 1;
    sc->input_feed = 1;
    sc->n_tabs = AT_NC_TABS;
    for (i = 0; i < AT_NC_TABS; ++i) {
        snprintf(sc->tabs[i].name, sizeof sc->tabs[i].name, "%s", at_nc_tab_name[i]);
        sc->tabs[i].count = i == st->tab ? st->n : i == AT_NC_TAB_QUEUE && ss.q_pending > 0 ? ss.q_pending : -1;
    }
    vw->tab = st->tab;
    last = st->base + AT_MAX_ITEMS;
    if (last > total) last = total;
    rows_view = 0;
    {   /* which rows are on screen now: only their pictures are asked for (a scroll window is 64 rows; the screen shows four or five) */
        int vis_lo = st->top, vis_hi = st->top + 5;
        for (i = st->base; i < last; ++i) {
            AtItem *it = &sc->items[sc->n_items++];
            int showpic = i >= vis_lo && i <= vis_hi;
            memset(it, 0, sizeof *it);
            if (i < ctl) {
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
                } else if (i == 2) {
                    snprintf(it->id, sizeof it->id, "%s", "show");
                    snprintf(it->label, sizeof it->label, "%s", "Show");
                    snprintf(it->text, sizeof it->text, "%s", at_nc_show_name[st->f.show < 0 || st->f.show >= AT_NC_SHOW_N ? 0 : st->f.show]);
                    snprintf(it->sub, sizeof it->sub, "%s", "All, not installed, installed, update available or queued.");
                } else {
                    snprintf(it->id, sizeof it->id, "%s", "sort");
                    snprintf(it->label, sizeof it->label, "%s", "Sort by");
                    snprintf(it->text, sizeof it->text, "%s", at_nc_sort_name[st->f.sort < 0 || st->f.sort > 4 ? 0 : st->f.sort]);
                    snprintf(it->sub, sizeof it->sub, "%s", "Left and right change the order.");
                }
            } else if (st->n == 0) {
                snprintf(it->id, sizeof it->id, "%s", "none");
                if (st->tab == AT_NC_TAB_QUEUE) {
                    snprintf(it->label, sizeof it->label, "%s", "Your queue is empty");
                    snprintf(it->sub, sizeof it->sub, "%s", "Press A on a mod to add it, then Start to download.");
                } else if (st->tab == AT_NC_TAB_INSTALLED) {
                    snprintf(it->label, sizeof it->label, "%s", "Nothing installed yet");
                    snprintf(it->sub, sizeof it->sub, "%s", "Mods installed from SSBM Nucleus appear here.");
                } else {
                    snprintf(it->label, sizeof it->label, "%s", "Nothing matches");
                    snprintf(it->sub, sizeof it->sub, "%s", "Change Show, Fighter or Search at the top of the list.");
                }
                it->iflags |= AT_ITEM_RO;
            } else {
                int k = i - ctl;
                snprintf(it->id, sizeof it->id, "r%d", k);
                if (s->row(s->user, st->ids[k], &row)) {
                    at_nc_fill_row(s, &row, it, st->tab == AT_NC_TAB_QUEUE, showpic);
                    snprintf(it->id, sizeof it->id, "r%d", k);
                    ++rows_view;
                } else snprintf(it->label, sizeof it->label, "(gone)");
            }
        }
    }
    vw->focus.block = 0;
    vw->focus.index = cur - st->base;
    {
        int sc_rel = at_list_scroll_to(sc, pane_h, cur - st->base, st->top - st->base < 0 ? 0 : st->top - st->base);
        st->top = st->base + sc_rel;
        vw->scroll = sc_rel;
    }
    snprintf(vw->counter, sizeof vw->counter, "%d / %d", st->n ? (cur - ctl + 1 < 1 ? 0 : cur - ctl + 1) : 0, st->n);
    memset(&row, 0, sizeof row);
    row.thumb = -1;
    row.qstate = AT_NC_Q_NONE;
    if (st->edit) { at_nc_put_key(sc, vw, 'A', "Done"); at_nc_put_key(sc, vw, 'B', "Cancel"); }
    else if (cur >= ctl && st->n > 0 && s->row(s->user, st->ids[cur - ctl], &row)) {
        if (st->tab == AT_NC_TAB_QUEUE) {
            at_nc_put_key(sc, vw, 'A', row.qstate == AT_NC_Q_FAILED ? "Retry" : row.qstate == AT_NC_Q_RUNNING ? "Working" : row.qstate == AT_NC_Q_DONE ? "Dismiss" : "Remove");
            at_nc_start_key(&ss, sc, vw);
            at_nc_put_key(sc, vw, 'B', "Back");
            at_nc_put_key(sc, vw, 'X', "Move up");
            at_nc_put_key(sc, vw, 'Y', "Move down");
            at_nc_put_key(sc, vw, 'Z', "Clear");
        } else {
            at_nc_put_key(sc, vw, 'A', row.qstate == AT_NC_Q_QUEUED ? "Unqueue" : row.state == AT_NC_ST_INSTALLED ? "Details" : row.state == AT_NC_ST_UNSUPPORTED ? "Why not" : "Queue");
            at_nc_start_key(&ss, sc, vw);
            at_nc_put_key(sc, vw, 'B', "Back");
            at_nc_put_key(sc, vw, 'Y', "Details");
            at_nc_put_key(sc, vw, 'X', "Search");
        }
    } else {
        at_nc_put_key(sc, vw, 'A', cur < ctl ? (cur == 0 ? "Type" : "Change") : "OK");
        at_nc_start_key(&ss, sc, vw);
        at_nc_put_key(sc, vw, 'B', "Back");
        if (st->tab == AT_NC_TAB_QUEUE) at_nc_put_key(sc, vw, 'Z', "Clear");
        else at_nc_put_key(sc, vw, 'X', "Search");
    }
    (void) buf; (void) rows_view;
    syncing = at_nc_corner(s, st, vw, now_ms);
    if (!st->edit && !syncing && st->tab != AT_NC_TAB_QUEUE) at_nc_put_key(sc, vw, 'Z', "Refresh");
    at_nc_dialog(st, vw, ss.q_pending - (ss.q_running_pct >= 0 && ss.q_running_title[0] ? 1 : 0) + ss.q_failed);
    return 1;
}

static void at_nc_cycle(const AtNcSrc *s, AtNcState *st, int row, int dir) {
    if (row == 1) {
        int n = s->fighters(s->user), v = st->f.fighter + dir;
        if (v < -1) v = n - 1;
        if (v >= n) v = -1;
        st->f.fighter = v;
    } else if (row == 2) {
        st->f.show = (st->f.show + dir + AT_NC_SHOW_N) % AT_NC_SHOW_N;
    } else {
        st->f.sort = (st->f.sort + dir + 5) % 5;
    }
    st->stale = 1;
    st->sel = row;
    st->top = 0;
}

/* a typed character from the keyboard while the search is being edited: 8 backspace, 13 enter, 27 escape, else a letter, digit or space */
void at_nc_type(AtNcState *st, int ch) {
    size_t n = strlen(st->edit_buf);
    if (!st->edit) return;
    if (ch == 13) { st->edit = 0; snprintf(st->f.text, sizeof st->f.text, "%s", st->edit_buf); st->stale = 1; st->sel = st->top = at_nc_ctl(st); st->follow = 0; return; }
    if (ch == 27) { st->edit = 0; snprintf(st->edit_buf, sizeof st->edit_buf, "%s", st->edit_old); return; }
    if (ch == 8) { if (n) st->edit_buf[n - 1] = '\0'; return; }
    if (ch >= 32 && ch < 127 && n + 1 < sizeof st->edit_buf && !(ch == ' ' && (n == 0 || st->edit_buf[n - 1] == ' '))) { st->edit_buf[n] = (char) ch; st->edit_buf[n + 1] = '\0'; }
}

static void at_nc_start_edit(AtNcState *st) {
    st->edit = 1;
    snprintf(st->edit_buf, sizeof st->edit_buf, "%s", st->f.text);
    snprintf(st->edit_old, sizeof st->edit_old, "%s", st->f.text);
    st->sel = st->top = 0;
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

static void at_nc_act(AtNcAction *act, int id, int what) { act->kind = AT_NA_ACT; act->id = id; act->what = what; }

/* the events of the clear-the-queue dialog */
static void at_nc_confirm_event(AtNcState *st, const AtEvent *e, AtNcAction *act) {
    if (e->type == AT_EV_BACK) st->confirm = 0;
    else if (e->type == AT_EV_MOVE && (e->a == AT_DIR_LEFT || e->a == AT_DIR_RIGHT)) st->confirm_focus = e->a == AT_DIR_LEFT ? 0 : 1;
    else if (e->type == AT_EV_ACCEPT) {
        if (!st->confirm_focus) at_nc_act(act, 0, AT_NC_ACT_Q_CLEAR);
        st->confirm = 0;
        st->stale = 1;
    }
}

void at_nc_event(const AtNcSrc *s, AtNcState *st, const AtEvent *e, double now_ms, AtNcAction *act) {
    int ctl = at_nc_ctl(st), total = ctl + (st->n > 0 ? st->n : 1), cur = st->sel;
    AtNcRow row;
    memset(act, 0, sizeof *act);
    if (st->confirm) { at_nc_confirm_event(st, e, act); return; }
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
    if (e->type == AT_EV_START) { at_nc_act(act, 0, AT_NC_ACT_START); return; }
    if (e->type == AT_EV_PAGE) {
        st->tab = (st->tab + (e->a < 0 ? AT_NC_TABS - 1 : 1)) % AT_NC_TABS;
        at_nc_apply_tab(st);
        return;
    }
    if (e->type == AT_EV_ALT && st->tab == AT_NC_TAB_QUEUE) {                    /* the QUEUE tab: X up, Y down, Z clear */
        int id = (cur >= ctl && st->n > 0 && cur - ctl < st->n) ? st->ids[cur - ctl] : 0;
        if (e->a == 'X' && id) { at_nc_act(act, id, AT_NC_ACT_Q_UP); st->follow = id; st->stale = 1; }
        else if (e->a == 'Y' && id) { at_nc_act(act, id, AT_NC_ACT_Q_DOWN); st->follow = id; st->stale = 1; }
        else if (e->a == 'Z') {
            if (st->n > 0) { st->confirm = 1; st->confirm_focus = 1; st->confirm_from = now_ms; }
            else at_nc_say(st, "The queue is empty.", AT_NOTE_INFO, now_ms);
        }
        return;
    }
    if (e->type == AT_EV_ALT && e->a == 'X') { at_nc_start_edit(st); return; }
    if (e->type == AT_EV_ALT && e->a == 'Z') { at_nc_act(act, 0, AT_NC_ACT_REFRESH); return; }
    if (total <= 0) return;
    if (e->type == AT_EV_MOVE) {
        if (e->a == AT_DIR_UP) st->sel = (cur + total - 1) % total;
        else if (e->a == AT_DIR_DOWN) st->sel = (cur + 1) % total;
        else if (cur >= 1 && cur < ctl) at_nc_cycle(s, st, cur, e->a == AT_DIR_RIGHT ? 1 : -1);
        else if (cur == 0 && ctl > 0 && e->a == AT_DIR_RIGHT) at_nc_start_edit(st);
        return;
    }
    if (e->type == AT_EV_SCROLL) { int v = cur + (e->a > 0 ? 1 : -1); st->sel = v < 0 ? 0 : v >= total ? total - 1 : v; return; }
    if (e->type == AT_EV_FOCUS) { int v = st->base + e->b; if (v >= 0 && v < total) st->sel = v; return; }
    if (e->type == AT_EV_ACCEPT) {
        if (cur < ctl) { if (cur == 0) at_nc_start_edit(st); else at_nc_cycle(s, st, cur, 1); return; }
        if (st->n == 0) return;
        if (!s->row(s->user, st->ids[cur - ctl], &row)) return;
        if (st->tab == AT_NC_TAB_QUEUE) {
            if (row.qstate == AT_NC_Q_FAILED) at_nc_act(act, row.id, AT_NC_ACT_Q_RETRY);
            else if (row.qstate == AT_NC_Q_RUNNING) at_nc_say(st, "This one is being installed; it can be removed when it finishes.", AT_NOTE_INFO, now_ms);
            else at_nc_act(act, row.id, AT_NC_ACT_Q_REMOVE);
            return;
        }
        if (row.state == AT_NC_ST_UNSUPPORTED) at_nc_say(st, row.reason[0] ? row.reason : "This cannot be installed yet.", AT_NOTE_WARN, now_ms);
        else if (row.state == AT_NC_ST_INSTALLED && row.qstate == AT_NC_Q_NONE) { st->mode = 1; st->detail_id = row.id; st->detail_sel = 1; }   /* installed: A opens the detail, where it can be removed */
        else if (row.qstate == AT_NC_Q_RUNNING) at_nc_say(st, "This one is being installed right now.", AT_NOTE_INFO, now_ms);
        else at_nc_act(act, row.id, AT_NC_ACT_QUEUE);
        return;
    }
    if (e->type == AT_EV_ALT && e->a == 'Y' && cur >= ctl && st->n > 0) { st->mode = 1; st->detail_id = st->ids[cur - ctl]; st->detail_sel = 1; }
}

/* ---- the detail ---- */

static void at_nc_fact_row(AtScreen *sc, const char *id, const char *label, const char *value) {
    AtItem *it;
    if (sc->n_items >= AT_MAX_ITEMS || !value[0]) return;
    it = &sc->items[sc->n_items++];
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id);
    snprintf(it->label, sizeof it->label, "%s", label);
    snprintf(it->title, sizeof it->title, "%.*s", (int) sizeof it->title - 1, value);
    snprintf(it->meta, sizeof it->meta, "%s", label);
    it->row_h = 50;
    it->iflags |= AT_ITEM_RO;
}

static void at_nc_action_row(AtScreen *sc, const char *id, const char *label, const char *sub) {
    AtItem *it;
    if (sc->n_items >= AT_MAX_ITEMS) return;
    it = &sc->items[sc->n_items++];
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id);
    snprintf(it->label, sizeof it->label, "%s", label);
    snprintf(it->sub, sizeof it->sub, "%s", sub);
}

/* the detail: the mod's picture and title, what A does, the screenshots, the description, the facts. */
int at_nc_detail_build(const AtNcSrc *s, AtNcState *st, AtScreen *sc, AtView *vw, double now_ms) {
    AtNcRow r;
    AtNcStatus ss;
    AtItem *it;
    char buf[AT_TEXT], kick[AT_STR];
    int ok, i, tex;
    float pane_h = at_nc_pane_h() + 30.0f;           /* no tab strip on the detail */
    at_nc_begin(sc, vw);
    memset(&r, 0, sizeof r);
    r.thumb = -1; r.tex = -1; r.qstate = AT_NC_Q_NONE;
    ok = s->row(s->user, st->detail_id, &r);
    if (!ok) return 0;
    memset(&ss, 0, sizeof ss);
    s->status(s->user, &ss);
    at_nc_upper(kick, sizeof kick, r.title);
    snprintf(sc->id, sizeof sc->id, "%s", "mods.nucleus.detail");
    snprintf(sc->title, sizeof sc->title, "%s", "DETAILS");
    snprintf(sc->parent[0], sizeof sc->parent[0], "%s", "MODS");
    snprintf(sc->parent[1], sizeof sc->parent[1], "%s", "NUCLEUS");
    sc->n_parents = 2;
    sc->chapter = 4;
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_NONE;
    sc->port = 1;
    sc->input_feed = 1;
    /* 0: the mod itself, as in the list */
    at_nc_fill_row(s, &r, &sc->items[sc->n_items++], 0, 1);
    snprintf(sc->items[0].id, sizeof sc->items[0].id, "%s", "hero");
    sc->items[0].iflags |= AT_ITEM_RO;
    /* the actions */
    if (r.state == AT_NC_ST_UNSUPPORTED) at_nc_action_row(sc, "act", "Cannot be installed yet", r.reason[0] ? r.reason : "This kind of mod is shown here but not installed.");
    else if (r.qstate == AT_NC_Q_QUEUED) at_nc_action_row(sc, "act", "Take it out of the queue", "It will not be downloaded.");
    else if (r.qstate == AT_NC_Q_RUNNING) { snprintf(buf, sizeof buf, "Installing - %d%%", r.qpct); at_nc_action_row(sc, "act", buf, "It can be removed when it finishes."); }
    else if (r.qstate == AT_NC_Q_FAILED) at_nc_action_row(sc, "act", "Try again", r.qreason[0] ? r.qreason : "The last attempt failed.");
    else if (r.state == AT_NC_ST_UPDATE) at_nc_action_row(sc, "act", "Queue the update", "A newer version is on SSBM Nucleus.");
    else if (r.state == AT_NC_ST_INSTALLED) at_nc_action_row(sc, "act", "Installed", "Already installed.");
    else if (r.state == AT_NC_ST_REMOVED) at_nc_action_row(sc, "act", "Taken down", "Taken down from SSBM Nucleus; your copy stays.");
    else at_nc_action_row(sc, "act", "Add to the queue", "Downloads it with the others when you press Start.");
    if (ss.q_pending > 0 || ss.q_processing) {
        if (ss.q_processing) at_nc_action_row(sc, "start", "Pause the downloads", "The one being installed finishes first.");
        else { snprintf(buf, sizeof buf, "%d in the queue. Starts them one after another.", ss.q_pending); at_nc_action_row(sc, "start", "Start the downloads", buf); }
    }
    if (r.state == AT_NC_ST_INSTALLED || r.state == AT_NC_ST_UPDATE || r.state == AT_NC_ST_REMOVED) at_nc_action_row(sc, "rm", "Uninstall", "Deletes the skin from your mods folder.");
    at_nc_action_row(sc, "page", "Open the page", r.page[0] ? r.page : "ssbmnucleus.net");
    /* the screenshots */
    for (i = 0; i < r.nshots && i < 3; ++i) {
        tex = -1;
        if (sc->n_items >= AT_MAX_ITEMS) break;
        it = &sc->items[sc->n_items++];
        memset(it, 0, sizeof *it);
        snprintf(it->id, sizeof it->id, "shot%d", i);
        snprintf(it->label, sizeof it->label, "Screenshot %d", i + 1);
        snprintf(it->title, sizeof it->title, "Screenshot %d of %d", i + 1, r.nshots > 3 ? 3 : r.nshots);
        it->row_h = AT_NC_SHOT_H;
        it->thumb_on = 1;
        it->thumb_w = 128;
        it->thumb_tex = (s->pic && s->pic(s->user, r.id, AT_NC_PIC_SHOT1 + i, &tex)) ? tex : -1;
        snprintf(it->thumb_abbr, sizeof it->thumb_abbr, "%s", "...");
        it->iflags |= AT_ITEM_RO;
    }
    /* the description, wrapped by the renderer to the row's width */
    if (r.desc[0] && sc->n_items < AT_MAX_ITEMS) {
        it = &sc->items[sc->n_items++];
        memset(it, 0, sizeof *it);
        snprintf(it->id, sizeof it->id, "%s", "about");
        snprintf(it->label, sizeof it->label, "%s", "About");
        snprintf(it->body, sizeof it->body, "%s", r.desc);
        it->row_h = AT_NC_ABOUT_H;
        it->iflags |= AT_ITEM_RO;
    }
    /* the facts */
    at_nc_fact_row(sc, "who", "Author", r.author[0] ? r.author : "unknown");
    snprintf(buf, sizeof buf, "%s", r.fighter);
    at_nc_join(buf, sizeof buf, r.kind);
    if (r.ncostumes > 0) { char t[40]; snprintf(t, sizeof t, "%d costume file%s", r.ncostumes, r.ncostumes == 1 ? "" : "s"); at_nc_join(buf, sizeof buf, t); }
    at_nc_fact_row(sc, "what", "Contains", buf);
    if (r.tags[0]) at_nc_fact_row(sc, "tags", "Tags", r.tags);
    snprintf(buf, sizeof buf, "%d downloads, %d likes", r.downloads, r.likes);
    if (r.updated[0]) { char t[40]; snprintf(t, sizeof t, "updated %s", r.updated); at_nc_join(buf, sizeof buf, t); }
    at_nc_fact_row(sc, "pop", "On SSBM Nucleus", buf);
    at_nc_fact_row(sc, "credit", "Credit", AT_NC_CREDIT);
    if (st->detail_sel >= sc->n_items) st->detail_sel = sc->n_items - 1;
    if (st->detail_sel < 1) st->detail_sel = sc->n_items > 1 ? 1 : 0;
    vw->focus.block = 0;
    vw->focus.index = st->detail_sel;
    vw->scroll = at_list_scroll_to(sc, pane_h, st->detail_sel, st->dtop > 0 && st->dtop < sc->n_items ? st->dtop : 0);
    st->dtop = vw->scroll;
    {
        const char *fid = sc->items[st->detail_sel].id;
        if (!strcmp(fid, "act") && r.state != AT_NC_ST_UNSUPPORTED && r.state != AT_NC_ST_REMOVED && !(r.state == AT_NC_ST_INSTALLED && r.qstate == AT_NC_Q_NONE) && r.qstate != AT_NC_Q_RUNNING)
            at_nc_put_key(sc, vw, 'A', r.qstate == AT_NC_Q_QUEUED ? "Unqueue" : r.qstate == AT_NC_Q_FAILED ? "Retry" : "Queue");
        else if (!strcmp(fid, "start")) at_nc_put_key(sc, vw, 'A', ss.q_processing ? "Pause" : "Start");
        else if (!strcmp(fid, "rm")) at_nc_put_key(sc, vw, 'A', "Uninstall");
        else if (!strcmp(fid, "page")) at_nc_put_key(sc, vw, 'A', "Open");
    }
    if (!ss.q_processing && ss.q_pending > 0) { snprintf(buf, sizeof buf, "Download %d", ss.q_pending); at_nc_put_key(sc, vw, 'S', buf); }
    else if (ss.q_processing && ss.q_pending > 0) at_nc_put_key(sc, vw, 'S', "Pause");
    at_nc_put_key(sc, vw, 'B', "Back");
    snprintf(vw->counter, sizeof vw->counter, "%d / %d", st->detail_sel + 1, sc->n_items);
    at_nc_corner(s, st, vw, now_ms);
    (void) kick;
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
    if (e->type == AT_EV_START) { at_nc_act(act, 0, AT_NC_ACT_START); return; }
    if (e->type == AT_EV_MOVE && e->a == AT_DIR_UP) st->detail_sel = (st->detail_sel + sc.n_items - 1) % sc.n_items;
    else if (e->type == AT_EV_MOVE && e->a == AT_DIR_DOWN) st->detail_sel = (st->detail_sel + 1) % sc.n_items;
    else if (e->type == AT_EV_SCROLL) { int v = st->detail_sel + (e->a > 0 ? 1 : -1); st->detail_sel = v < 0 ? 0 : v >= sc.n_items ? sc.n_items - 1 : v; }
    else if (e->type == AT_EV_FOCUS && e->b >= 0 && e->b < sc.n_items) st->detail_sel = e->b;
    else if (e->type == AT_EV_ACCEPT && st->detail_sel >= 0 && st->detail_sel < sc.n_items) {
        id = sc.items[st->detail_sel].id;
        if (!strcmp(id, "act")) {
            AtNcRow r;
            memset(&r, 0, sizeof r);
            r.qstate = AT_NC_Q_NONE;
            if (!s->row(s->user, st->detail_id, &r)) return;
            if (r.state == AT_NC_ST_UNSUPPORTED) at_nc_say(st, r.reason[0] ? r.reason : "This cannot be installed yet.", AT_NOTE_WARN, now_ms);
            else if (r.qstate == AT_NC_Q_RUNNING) at_nc_say(st, "This one is being installed right now.", AT_NOTE_INFO, now_ms);
            else if (r.qstate == AT_NC_Q_FAILED) at_nc_act(act, r.id, AT_NC_ACT_Q_RETRY);
            else if (r.state == AT_NC_ST_NONE || r.state == AT_NC_ST_UPDATE || r.qstate == AT_NC_Q_QUEUED) at_nc_act(act, r.id, AT_NC_ACT_QUEUE);
            else at_nc_say(st, r.reason[0] ? r.reason : "Nothing to do.", AT_NOTE_INFO, now_ms);
        }
        else if (!strcmp(id, "start")) at_nc_act(act, 0, AT_NC_ACT_START);
        else if (!strcmp(id, "rm")) at_nc_act(act, st->detail_id, AT_NC_ACT_UNINSTALL);
        else if (!strcmp(id, "page")) at_nc_act(act, st->detail_id, AT_NC_ACT_PAGE);
    }
}

#endif
