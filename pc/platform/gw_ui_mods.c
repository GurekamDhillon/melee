#include "gw_ui_mods.h"
#include "gw_ui_tokens.h"
#include <ctype.h>
#include <stdio.h>
#include <string.h>

void at_mods_state_init(AtModsState *st) { memset(st, 0, sizeof *st); }

static const char *kind_word(const char *k)
{
    if (k == NULL) return "Content";
    if (strcmp(k, "base") == 0) return "Base";
    if (strcmp(k, "fighter") == 0) return "Fighter";
    if (strcmp(k, "stage") == 0) return "Stage";
    if (strcmp(k, "script") == 0) return "Script";
    return "Content";
}

const char *at_mods_parent_label(const char *p)
{
    static char buf[4][AT_STR];
    static int k;
    char *o;
    if (strcmp(p, "main") == 0) return "Main menu";
    if (strcmp(p, "solo") == 0) return "Solo";
    if (strcmp(p, "versus") == 0) return "Versus";
    if (strcmp(p, "online") == 0) return "Online";
    if (strcmp(p, "mods") == 0) return "Mods";
    if (strcmp(p, "settings") == 0) return "Settings";
    if (strcmp(p, "more") == 0) return "More";
    if (strcmp(p, "pause") == 0) return "Pause";
    if (strcmp(p, "lab.pause") == 0) return "LAB pause";
    if (strncmp(p, "settings.", 9) == 0 && p[9] != '\0') {
        o = buf[k++ & 3];
        snprintf(o, AT_STR, "Settings > %c%s", p[9] >= 'a' && p[9] <= 'z' ? p[9] - 32 : p[9], p + 10);
        return o;
    }
    return p;
}

void at_mods_add_line(const char *parent, const char *label, char *out, int cap)
{
    snprintf(out, (size_t) cap, "%s > %s", at_mods_parent_label(parent), label);
}

static void set_tabs(AtScreen *sc, AtView *vw, int active, int n_installed, int n_conflicts)
{
    sc->n_tabs = 2;
    snprintf(sc->tabs[0].name, sizeof sc->tabs[0].name, "%s", "INSTALLED");
    snprintf(sc->tabs[1].name, sizeof sc->tabs[1].name, "%s", "CONFLICTS");
    sc->tabs[0].count = n_installed;
    sc->tabs[1].count = n_conflicts;
    vw->tab = active;
}

static void put_key(AtScreen *sc, AtView *vw, char btn, const char *label)
{
    int i = sc->n_keys;
    if (i >= AT_MAX_KEYS) return;
    sc->keys[i].btn = btn;
    sc->keys[i].fn_label = sc->keys[i].fn_when = -1;
    snprintf(sc->keys[i].label, sizeof sc->keys[i].label, "%s", label);
    snprintf(vw->key_label[i], sizeof vw->key_label[i], "%s", label);
    vw->key_shown[i] = 1;
    sc->n_keys++;
}

static int word_in(const char *list, const char *id)
{
    size_t n = strlen(id);
    const char *p = list;
    while (p != NULL && *p) {
        const char *e = strchr(p, ',');
        size_t len = e ? (size_t) (e - p) : strlen(p);
        if (len == n && strncmp(p, id, n) == 0) return 1;
        p = e ? e + 1 : NULL;
    }
    return 0;
}

static int is_locked(const AtModsSrc *s) { return s->locked != NULL && s->locked(s->user) != 0; }

static void cat(char *out, int cap, const char *fmt, const char *a)
{
    size_t l = strlen(out);
    snprintf(out + l, (size_t) cap - l, fmt, a);
}

static void row_sub(const AtModsSrc *s, int m, char *out, int cap)
{
    char adds[2][AT_STR];
    int status = s->status(s->user, m), na = s->adds != NULL ? s->adds(s->user, m, adds, 1) : 0;
    const char *pack = s->pack(s->user, m), *st = s->status_text(s->user, m);
    snprintf(out, (size_t) cap, "%s", kind_word(s->kind(s->user, m)));
    if (pack != NULL && pack[0]) cat(out, cap, " - %s", pack);
    if (na > 0) cat(out, cap, " - adds %s", adds[0]);
    if (status == AT_MOD_CONFLICT) cat(out, cap, " - %s", "CONFLICT");
    else if (status == AT_MOD_MISSING_DEP && st != NULL && st[0]) cat(out, cap, " - %s", st);
    else if (s->enabled(s->user, m) != s->active(s->user, m)) cat(out, cap, " - %s", "RESTART");
}

static void no_well(AtExplainer *e)
{
    memset(e, 0, sizeof *e);
    e->media_model = e->media_ring = AT_NO_MODEL;     /* a zeroed model would name model 0 */
    e->media_tex = -1;
    e->no_well = 1;                                   /* a mod has no picture */
}

int at_mods_build(const AtModsSrc *s, AtModsState *st, AtScreen *sc, AtView *vw, double now_ms)
{
    int i, t = st->tab, n = s->count(s->user), cur, last, locked = is_locked(s);
    const char *key_a = "Turn on";
    memset(sc, 0, sizeof *sc);
    sc->fn_provide = sc->fn_accept = sc->fn_back = sc->fn_focus = sc->fn_change = sc->fn_open = sc->fn_close = sc->fn_counter = sc->fn_page = sc->fn_start = -1;
    sc->fn_alt[0] = sc->fn_alt[1] = sc->fn_alt[2] = -1;
    memset(vw->key_label, 0, sizeof vw->key_label);
    memset(vw->key_shown, 0, sizeof vw->key_shown);
    no_well(&vw->ex);
    if (n > AT_MODS_MAX) n = AT_MODS_MAX;
    if (t < 0 || t > 1) t = st->tab = 0;
    st->n[0] = st->n[1] = 0;
    for (i = 0; i < n; i++) {
        int stt = s->status(s->user, i);
        st->list[0][st->n[0]++] = i;
        if (stt == AT_MOD_CONFLICT || stt == AT_MOD_MISSING_DEP) st->list[1][st->n[1]++] = i;
    }
    for (i = 0; i < 2; i++) {
        if (st->sel[i] >= st->n[i]) st->sel[i] = st->n[i] - 1;
        if (st->sel[i] < 0) st->sel[i] = 0;
    }
    cur = st->sel[t];
    st->top[t] = at_list_scroll(cur, st->top[t], AT_MODS_VISIBLE, st->n[t] > 0 ? st->n[t] : 1);
    st->base = st->top[t] - 12;                                       /* the window: the visible rows and room either side */
    if (st->base > st->n[t] - AT_MODS_WINDOW) st->base = st->n[t] - AT_MODS_WINDOW;
    if (st->base < 0) st->base = 0;
    snprintf(sc->id, sizeof sc->id, "%s", "mods.list");
    snprintf(sc->title, sizeof sc->title, "%s", "MODS");
    snprintf(sc->parent[0], sizeof sc->parent[0], "%s", "MAIN MENU");
    sc->n_parents = 1;
    sc->chapter = 4;                                                  /* IV Mods */
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_WIDE;
    sc->port = 1;
    sc->input_feed = 1;                                               /* the model applies events itself (at_mods_event) */
    set_tabs(sc, vw, t, st->n[0], st->n[1]);
    if (st->n[t] == 0) {
        AtItem *it = &sc->items[sc->n_items++];
        memset(it, 0, sizeof *it);
        snprintf(it->id, sizeof it->id, "%s", "none");
        snprintf(it->label, sizeof it->label, "%s", t == 0 ? "No mods installed" : "No conflicts");
        snprintf(it->sub, sizeof it->sub, "%s", t == 0 ? "Put a mod folder in the mods folder, then restart." : "Every mod that is on can start.");
        it->flags = AT_CELL_DISABLED;
        vw->focus.block = 0; vw->focus.index = 0; vw->scroll = 0;
        snprintf(vw->counter, sizeof vw->counter, "%s", "0 / 0");
        put_key(sc, vw, 'B', "Back");
    } else {
        last = st->base + AT_MODS_WINDOW;
        if (last > st->n[t]) last = st->n[t];
        for (i = st->base; i < last; i++) {
            int m = st->list[t][i];
            AtItem *it = &sc->items[sc->n_items++];
            memset(it, 0, sizeof *it);
            snprintf(it->id, sizeof it->id, "m%d", m);
            snprintf(it->label, sizeof it->label, "%s", s->name(s->user, m));
            row_sub(s, m, it->sub, (int) sizeof it->sub);
            it->vkind = AT_VAL_TOGGLE;
            it->on = s->enabled(s->user, m) != 0;
            if (locked) it->iflags |= AT_ITEM_RO;                    /* shown, never changed */
            if (s->active(s->user, m)) it->flags |= AT_CELL_SELECTED;
        }
        vw->focus.block = 0;
        vw->focus.index = cur - st->base;
        vw->scroll = st->top[t] - st->base;
        snprintf(vw->counter, sizeof vw->counter, "%d / %d", cur + 1, st->n[t]);
        {
            int m = st->list[t][cur], status = s->status(s->user, m);
            const char *d = s->desc(s->user, m), *stx = s->status_text(s->user, m), *ver = s->version(s->user, m);
            key_a = s->enabled(s->user, m) ? "Turn off" : "Turn on";
            vw->ex.has = 1;
            snprintf(vw->ex.kicker, sizeof vw->ex.kicker, "%s", status == AT_MOD_CONFLICT ? "CONFLICT" : status == AT_MOD_MISSING_DEP ? "NEEDS A MOD" : kind_word(s->kind(s->user, m)));
            snprintf(vw->ex.title, sizeof vw->ex.title, "%s", s->name(s->user, m));
            snprintf(vw->ex.what, sizeof vw->ex.what, "%s", (status == AT_MOD_CONFLICT || status == AT_MOD_MISSING_DEP) && stx != NULL && stx[0] ? stx : d);
            snprintf(vw->ex.from_text, sizeof vw->ex.from_text, "%s%s%s", s->id(s->user, m), ver != NULL && ver[0] ? " " : "", ver != NULL ? ver : "");
        }
        if (!locked) put_key(sc, vw, 'A', key_a);
        put_key(sc, vw, 'B', "Back");
        put_key(sc, vw, 'Y', "Details");
        if (!locked && (t == 1 || s->status(s->user, st->list[t][cur]) == AT_MOD_CONFLICT || s->status(s->user, st->list[t][cur]) == AT_MOD_MISSING_DEP)) put_key(sc, vw, 'X', "Resolve");
    }
    /* the corner note: the last result while it lasts, else the standing note (locked online, or the restart note) */
    if (now_ms < st->note_until && st->note[0]) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", st->note);
        vw->note.kind = st->note_kind;
        vw->note.from_ms = st->note_until - 3000.0;
        vw->note.until_ms = st->note_until;
    } else if (locked) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", "Locked while online");
        vw->note.kind = AT_NOTE_WARN;
        vw->note.from_ms = now_ms;
        vw->note.until_ms = now_ms + 1000.0;
    } else if (s->restart_needed(s->user)) {
        snprintf(vw->note.text, sizeof vw->note.text, "%s", "Applies at restart");
        vw->note.kind = AT_NOTE_INFO;
        vw->note.from_ms = now_ms;
        vw->note.until_ms = now_ms + 1000.0;
    } else {
        vw->note.text[0] = '\0';
    }
    return 1;
}

/* the mods that changed state between two snapshots, other than `except`, as "A, B and 2 more" */
static void changed_names(const AtModsSrc *s, const unsigned char *before, int n, int except, char *out, int cap)
{
    int i, k = 0, total = 0;
    out[0] = '\0';
    for (i = 0; i < n; i++) {
        if (i == except || (before[i] != 0) == (s->enabled(s->user, i) != 0)) continue;
        total++;
        if (k < 2) {
            size_t l = strlen(out);
            snprintf(out + l, (size_t) cap - l, "%s%s", k ? ", " : "", s->name(s->user, i));
            k++;
        }
    }
    if (total > 2) { size_t l = strlen(out); snprintf(out + l, (size_t) cap - l, " and %d more", total - 2); }
}

static void say(AtModsState *st, const char *text, int kind, double now_ms)
{
    snprintf(st->note, sizeof st->note, "%s", text);
    st->note_kind = kind;
    st->note_until = now_ms + 3000.0;
}

/* turn mod m on or off, save, and say what else moved */
static void toggle(const AtModsSrc *s, AtModsState *st, int m, int on, double now_ms)
{
    unsigned char before[AT_MODS_MAX];
    char others[AT_STR], text[AT_STR];
    int n = s->count(s->user), i, changed, saved;
    if (is_locked(s)) { say(st, "Mods cannot be changed while online.", AT_NOTE_WARN, now_ms); return; }
    if (n > AT_MODS_MAX) n = AT_MODS_MAX;
    for (i = 0; i < n; i++) before[i] = (unsigned char) (s->enabled(s->user, i) != 0);
    changed = s->set_enabled(s->user, m, on);
    saved = changed > 0 ? s->save(s->user) : 0;
    if (changed <= 0) { say(st, "Nothing changed.", AT_NOTE_INFO, now_ms); return; }
    if (saved != 0) { say(st, "Could not save mods/enabled.txt. The change is not kept.", AT_NOTE_ERR, now_ms); return; }
    changed_names(s, before, n, m, others, (int) sizeof others);
    if (others[0]) {
        /* a cascade turns mods on (requirements) or off (dependents, conflicting mods): say which, from the first other mod that moved */
        int turned_on = 0;
        for (i = 0; i < n; i++) if (i != m && (before[i] != 0) != (s->enabled(s->user, i) != 0)) { turned_on = s->enabled(s->user, i) != 0; break; }
        snprintf(text, sizeof text, "Also turned %s: %s.", turned_on ? "on" : "off", others);
    } else {
        snprintf(text, sizeof text, "%s %s. Applies at restart.", s->name(s->user, m), on ? "on" : "off");
    }
    say(st, text, AT_NOTE_OK, now_ms);
}

/* A mod that needs a mod that is not mounting. Turn on every requirement that is in the folder and off; name the first one that is not
 * installed. (The resolver's own SetEnabled on a mod that is already on may report nothing, so each requirement is turned on itself.) */
static void resolve_missing(const AtModsSrc *s, AtModsState *st, int m, double now_ms)
{
    char need[AT_STR], gone[AT_STR], text[AT_STR + 40];
    const char *p;
    int n = s->count(s->user), turned = 0;
    gone[0] = '\0';
    snprintf(need, sizeof need, "%s", s->requires(s->user, m));
    for (p = need; p != NULL && *p; ) {
        char id[AT_STR];
        const char *e = strchr(p, ',');
        size_t len = e ? (size_t) (e - p) : strlen(p);
        int j, found = -1;
        if (len >= sizeof id) len = sizeof id - 1;
        memcpy(id, p, len);
        id[len] = '\0';
        for (j = 0; j < n && found < 0; j++) if (strcmp(s->id(s->user, j), id) == 0) found = j;
        if (found < 0) { if (!gone[0]) snprintf(gone, sizeof gone, "%s", id); }
        else if (!s->enabled(s->user, found)) { toggle(s, st, found, 1, now_ms); turned++; }
        p = e ? e + 1 : NULL;
    }
    if (turned == 0) {
        if (gone[0]) snprintf(text, sizeof text, "Nothing to turn on: %s is not installed.", gone);
        else snprintf(text, sizeof text, "%s", "Nothing to turn on: restart to apply.");
        say(st, text, AT_NOTE_ERR, now_ms);
    }
}

void at_mods_event(const AtModsSrc *s, AtModsState *st, const AtEvent *e, double now_ms, AtModsAction *act)
{
    int t = st->tab, n = st->n[t], m;
    memset(act, 0, sizeof *act);
    act->kind = AT_MA_NONE;
    if (e->type == AT_EV_PAGE) { st->tab = 1 - t; return; }                /* two tabs: L, R, Tab and Shift+Tab all switch */
    if (e->type == AT_EV_BACK) { act->kind = AT_MA_BACK; return; }
    if (n == 0) return;
    m = st->list[t][st->sel[t]];
    if (e->type == AT_EV_MOVE) {
        if (e->a == AT_DIR_UP) st->sel[t] = (st->sel[t] + n - 1) % n;
        else if (e->a == AT_DIR_DOWN) st->sel[t] = (st->sel[t] + 1) % n;
        else toggle(s, st, m, !s->enabled(s->user, m), now_ms);          /* left and right flip a toggle, as on A */
    } else if (e->type == AT_EV_SCROLL) {
        int v = st->sel[t] + (e->a > 0 ? 1 : -1);
        st->sel[t] = v < 0 ? 0 : v >= n ? n - 1 : v;
    } else if (e->type == AT_EV_FOCUS) {
        int v = st->base + e->b;                                         /* the event names a row of the WINDOW */
        if (v >= 0 && v < n) st->sel[t] = v;
    } else if (e->type == AT_EV_ACCEPT) {
        toggle(s, st, m, !s->enabled(s->user, m), now_ms);
    } else if (e->type == AT_EV_ALT && e->a == 'Y') {
        act->kind = AT_MA_DETAIL; act->arg = m;
    } else if (e->type == AT_EV_ALT && e->a == 'X') {
        int status = s->status(s->user, m);
        if (is_locked(s)) { say(st, "Mods cannot be changed while online.", AT_NOTE_WARN, now_ms); return; }
        if (status == AT_MOD_CONFLICT) {
            toggle(s, st, m, 0, now_ms);
        } else if (status == AT_MOD_MISSING_DEP) {
            resolve_missing(s, st, m, now_ms);
        }
    }
}

/* ---- the detail screen ---- */

static void name_list(const AtModsSrc *s, const char *ids, char *out, int cap)
{
    const char *p = ids;
    int n = s->count(s->user);
    out[0] = '\0';
    while (p != NULL && *p) {
        char id[AT_STR];
        const char *e = strchr(p, ',');
        size_t len = e ? (size_t) (e - p) : strlen(p);
        int j, found = -1;
        size_t l = strlen(out);
        if (len >= sizeof id) len = sizeof id - 1;
        memcpy(id, p, len);
        id[len] = '\0';
        for (j = 0; j < n && found < 0; j++) if (strcmp(s->id(s->user, j), id) == 0) found = j;
        snprintf(out + l, (size_t) cap - l, "%s%s", l ? ", " : "", found >= 0 ? s->name(s->user, found) : id);
        p = e ? e + 1 : NULL;
    }
}

static void add_row(AtScreen *sc, const char *id, const char *label, const char *sub)
{
    AtItem *it;
    if (sc->n_items >= AT_MAX_ITEMS) return;
    it = &sc->items[sc->n_items++];
    memset(it, 0, sizeof *it);
    snprintf(it->id, sizeof it->id, "%s", id);
    snprintf(it->label, sizeof it->label, "%s", label);
    snprintf(it->sub, sizeof it->sub, "%s", sub);
}

int at_mods_detail_build(const AtModsSrc *s, const AtModsDetail *d, AtScreen *sc, AtView *vw, double now_ms)
{
    int m = d->mod, n = s->count(s->user), j, na, i;
    char buf[AT_STR], mine[AT_STR], title[AT_STR], sid[AT_ID * 2], slabel[AT_STR], adds[6][AT_STR], rid[AT_ID];
    const char *name, *others;
    (void) now_ms;
    memset(sc, 0, sizeof *sc);
    sc->fn_provide = sc->fn_accept = sc->fn_back = sc->fn_focus = sc->fn_change = sc->fn_open = sc->fn_close = sc->fn_counter = sc->fn_page = sc->fn_start = -1;
    sc->fn_alt[0] = sc->fn_alt[1] = sc->fn_alt[2] = -1;
    memset(vw->key_label, 0, sizeof vw->key_label);
    memset(vw->key_shown, 0, sizeof vw->key_shown);
    no_well(&vw->ex);
    vw->counter[0] = '\0';
    if (m < 0 || m >= n) return 0;
    name = s->name(s->user, m);
    for (i = 0; name[i] && i < AT_STR - 1; i++) title[i] = (char) toupper((unsigned char) name[i]);
    title[i] = '\0';
    snprintf(sc->id, sizeof sc->id, "%s", "mods.detail");
    snprintf(sc->title, sizeof sc->title, "%s", title);
    snprintf(sc->parent[0], sizeof sc->parent[0], "%s", "MAIN MENU");
    snprintf(sc->parent[1], sizeof sc->parent[1], "%s", "MODS");
    sc->n_parents = 2;
    sc->chapter = 4;
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_WIDE;
    sc->port = 1;
    sc->input_feed = 1;
    snprintf(buf, sizeof buf, "%s%s%s", kind_word(s->kind(s->user, m)), s->pack(s->user, m)[0] ? " - " : "", s->pack(s->user, m));
    add_row(sc, "folder", "Folder", s->id(s->user, m));
    name_list(s, s->requires(s->user, m), mine, (int) sizeof mine);
    add_row(sc, "requires", "Requires", mine[0] ? mine : "Nothing");
    /* conflicts: this mod's own list, then every mod whose list names this one */
    name_list(s, s->conflicts(s->user, m), mine, (int) sizeof mine);
    for (j = 0; j < n; j++) {
        size_t l = strlen(mine);
        if (j == m) continue;
        others = s->conflicts(s->user, j);
        if (others != NULL && word_in(others, s->id(s->user, m)) && strstr(mine, s->name(s->user, j)) == NULL)
            snprintf(mine + l, sizeof mine - l, "%s%s", l ? ", " : "", s->name(s->user, j));
    }
    add_row(sc, "conflicts", "Conflicts", mine[0] ? mine : "Nothing");
    na = s->adds != NULL ? s->adds(s->user, m, adds, 6) : 0;
    for (i = 0; i < na; i++) { snprintf(rid, sizeof rid, "add%d", i); add_row(sc, rid, i == 0 ? "Adds to the menus" : "Also adds", adds[i]); }
    if (s->settings != NULL && s->settings(s->user, m, sid, (int) sizeof sid, slabel, (int) sizeof slabel))
        add_row(sc, "settings", slabel, "Opens the mod's own settings screen.");
    vw->focus.block = 0;
    vw->focus.index = d->sel >= 0 && d->sel < sc->n_items ? d->sel : 0;
    vw->scroll = 0;
    vw->ex.has = 1;
    snprintf(vw->ex.kicker, sizeof vw->ex.kicker, "%s", buf);
    snprintf(vw->ex.title, sizeof vw->ex.title, "%s", name);
    snprintf(vw->ex.what, sizeof vw->ex.what, "%s", s->desc(s->user, m)[0] ? s->desc(s->user, m) : "No description.");
    snprintf(vw->ex.from_text, sizeof vw->ex.from_text, "%s", s->id(s->user, m));
    if (strcmp(sc->items[vw->focus.index].id, "settings") == 0) put_key(sc, vw, 'A', "Open");
    put_key(sc, vw, 'B', "Back");
    return 1;
}

void at_mods_detail_event(const AtModsSrc *s, AtModsDetail *d, const AtEvent *e, AtModsAction *act)
{
    static AtScreen sc;
    AtView vw;
    char sid[AT_ID * 2], slabel[AT_STR];
    memset(act, 0, sizeof *act);
    memset(&vw, 0, sizeof vw);
    if (!at_mods_detail_build(s, d, &sc, &vw, 0.0)) { act->kind = AT_MA_BACK; return; }
    if (e->type == AT_EV_BACK) { act->kind = AT_MA_BACK; return; }
    if (e->type == AT_EV_MOVE && e->a == AT_DIR_UP) d->sel = (d->sel + sc.n_items - 1) % sc.n_items;
    else if (e->type == AT_EV_MOVE && e->a == AT_DIR_DOWN) d->sel = (d->sel + 1) % sc.n_items;
    else if (e->type == AT_EV_SCROLL) {
        int v = d->sel + (e->a > 0 ? 1 : -1);
        d->sel = v < 0 ? 0 : v >= sc.n_items ? sc.n_items - 1 : v;
    }
    else if (e->type == AT_EV_FOCUS && e->b >= 0 && e->b < sc.n_items) d->sel = e->b;
    else if (e->type == AT_EV_ACCEPT && d->sel >= 0 && d->sel < sc.n_items && s->settings != NULL && strcmp(sc.items[d->sel].id, "settings") == 0 &&
             s->settings(s->user, d->mod, sid, (int) sizeof sid, slabel, (int) sizeof slabel)) {
        act->kind = AT_MA_SETTINGS;
        snprintf(act->id, sizeof act->id, "%s", sid);
    }
}
