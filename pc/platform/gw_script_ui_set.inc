/* gw_script_ui_set.inc - the host half of the SETTINGS screens: the tabbed pages, the remap editor and the how-to page (Atlas step 5). Included at the end of
 * gw_script_ui.inc (after gw_script_ui_sel.inc): it uses that file's slots, stack, view and mouse state.
 *
 * It is the settings' DRAWING AND INPUT LAYER over the game's tables. The pages stay FrontendItem arrays on the game side (src/melee/gm/gmfrontend_settings.inc);
 * the table walker (gmfrontend_atlas_table.h) evaluates get, visible, enabled and format there and hands this file scalars and strings through the Ui_Set* shims
 * (the game calls them unprefixed; the host defines gw_Ui_Set*). They fill ONE engine-owned screen record in place, rows in the screen's own item array (up to
 * AT_MAX_ITEMS). Nothing here holds a function pointer or a pointer into game memory, and no shim writes game memory except the poll's two out-parameters,
 * which are the game's locals and so are written BIG-ENDIAN (gs_ui_put_be32).
 *
 * The slot's owner is GS_UI_ENGINE, which every Lua door already refuses to a mod and to the console, and nothing may be pushed over it (gs_ui_open_screen).
 * The handle is (generation << 8) | slot, so a handle that outlived its screen is recognised and ignored. Only one native screen exists at a time (this or a
 * select): the stack has one top. gw_Ui_SceneExit closes it with the scene it was opened in.
 *
 * INPUT. The pad is the game's: the adapter feeds the menu input of every port (mn_80229624, the ORIGINAL layout in the frontend scene) through gw_Ui_SetIntent.
 * The keyboard and the mouse are read here, as for every menu. Events go back through a small ring the game drains with gw_Ui_SetPoll:
 *   CHANGE(slot, dir)  left or right (or the wheel) on a value row        ACCEPT(slot)  A, Enter, a click       BACK  B, Escape, a right click
 *   TAB(-, target)     L or R, Tab or Shift+Tab, a tab click (the screen does NOT switch tabs itself: the adapter submits the new rows, then SetTabs shows them)
 *   FOCUS(slot)        the focus moved (for the sound)                    ALT(slot, 'X'|'Y'|'Z')
 * Freeze drops every event (a name being typed, a controller capture) and, when it ends, primes the keys and the mouse from what is held at that instant, so a
 * key that was down when the edit ended does not also accept or go back. A dialog takes every event until it is answered; its focus starts on Cancel. */

#include "gw_ui_item.h"

#define GS_SET_Q 32
enum { GS_SETEV_CHANGE = 1, GS_SETEV_ACCEPT = 2, GS_SETEV_BACK = 3, GS_SETEV_TAB = 4, GS_SETEV_FOCUS = 5, GS_SETEV_ALT = 6 };   /* = FSS_EV_* (gmfrontend_atlas_table.h) */
enum { GS_SET_TABS, GS_SET_REMAP, GS_SET_HOWTO, GS_SET_ERASE, GS_SET_RULES, GS_SET_MORERULES, GS_SET_MATCH, GS_SET_ONLINE };

static struct {
    int gen;                 /* the handle generation: bumped per open */
    int h;                   /* the open handle, or -1 */
    struct { int type, a, b; } q[GS_SET_Q];
    int qn;
    int frozen;              /* every event is dropped (text entry, a capture) */
    int answer;              /* a dialog's answer: 0 pending, 1 the first button, 2 the second (taken once) */
    int pending;             /* the rows were submitted again: the focus is resolved by id before anyone reads it */
    char fid[AT_ID];         /* the focused row's id before the resubmit */
    int fidx;                /* and its position (the next visible row when the id is gone) */
    char help[AT_MAX_ITEMS][AT_TEXT];   /* each row's help, clamped: the explainer shows the focused one */
    char spec[96];           /* the key hints last parsed (re-parsed only on a change) */
} gs_set = { 0, -1 };

static GsUiSlot *gs_set_get(int h) {
    int slot = h & 0xFF;
    GsUiSlot *u;
    if (h < 0 || h != gs_set.h || slot >= GS_UI_SLOTS) return NULL;
    u = &gs_ui_slot[slot];
    return (u->used && u->native_set) ? u : NULL;
}

static int gs_set_is_open(void) { return gs_set.h >= 0 && gs_set_get(gs_set.h) != NULL; }

static void gs_set_push(int type, int a, int b) {
    if (gs_set.qn >= GS_SET_Q) return;                             /* the game drains it every frame; a full ring drops the newest */
    gs_set.q[gs_set.qn].type = type; gs_set.q[gs_set.qn].a = a; gs_set.q[gs_set.qn].b = b;
    gs_set.qn++;
}

/* "text" cut to cap - 1 characters; when it did not fit, the cut is at a word where one is near and the text ends in "..." (never a silent cut) */
static void gs_set_clamp(char *dst, int cap, const char *src) {
    int n = src != NULL ? (int) strlen(src) : 0, cut;
    if (n < cap) { memcpy(dst, src != NULL ? src : "", (size_t) n + 1); return; }
    cut = cap - 4;
    while (cut > cap - 28 && cut > 0 && src[cut] != ' ') cut--;
    if (src[cut] != ' ') cut = cap - 4;
    memcpy(dst, src, (size_t) cut);
    memcpy(dst + cut, "...", 4);
}

/* the focus, after the rows were submitted again: the same id, else the row at the same place (the next visible one), else none */
static void gs_set_resolve(GsUiSlot *u) {
    AtFocusPos old, p;
    if (!gs_set.pending) return;
    gs_set.pending = 0;
    if (u->sc.n_items < 1) { u->view.focus.block = 0; u->view.focus.index = -1; return; }
    old.block = 0; old.index = gs_set.fidx;
    p = at_screen_refocus(&u->sc, NULL, gs_set.fid[0] != '\0' ? gs_set.fid : NULL, old);
    u->view.focus = p;
    gs_ui_fix_scroll(u);
}

static int gs_set_steppable(const AtItem *it) {
    return (it->vkind == AT_VAL_TOGGLE || it->vkind == AT_VAL_CHOICE || it->vkind == AT_VAL_SLIDER) && !(it->iflags & AT_ITEM_RO);
}

static void gs_set_move_to(GsUiSlot *u, int index) {              /* the focus lands on a row; the game is told (the move sound) */
    if (index < 0 || index >= u->sc.n_items || index == u->view.focus.index) return;
    u->view.focus.block = 0; u->view.focus.index = index;
    gs_ui_fix_scroll(u);
    gs_set_push(GS_SETEV_FOCUS, index, 0);
}

static void gs_set_answer(GsUiSlot *u, int answer) {
    gs_set.answer = answer;
    u->view.dialog.open = 0;
}

/* one input event, from the game's intents, the keys or the mouse */
static void gs_set_event(GsUiSlot *u, const AtEvent *e) {
    AtFocusBlock fb[AT_MAX_BLOCKS];
    int nb, f;
    gs_set_resolve(u);
    if (u->view.dialog.open) {                                     /* a dialog takes every event until it is answered; B is always the second button */
        AtDialog *d = &u->view.dialog;
        if (e->type == AT_EV_ACCEPT) gs_set_answer(u, d->btn[d->focus] == 'A' ? 1 : 2);
        else if (e->type == AT_EV_BACK) gs_set_answer(u, 2);
        else if (e->type == AT_EV_MOVE && d->n == 2 && (e->a == AT_DIR_LEFT || e->a == AT_DIR_RIGHT)) d->focus = e->a == AT_DIR_LEFT ? 0 : 1;
        return;
    }
    if (gs_set.frozen) return;
    f = u->view.focus.index;
    switch (e->type) {
    case AT_EV_MOVE:
        if (e->a == AT_DIR_UP || e->a == AT_DIR_DOWN) {
            if (u->sc.n_items < 1) return;
            nb = at_screen_focus_blocks(&u->sc, fb);
            gs_set_move_to(u, at_focus_move(fb, nb, u->view.focus, e->a, 1).index);
        } else if ((e->a == AT_DIR_LEFT || e->a == AT_DIR_RIGHT) && f >= 0 && f < u->sc.n_items && gs_set_steppable(&u->sc.items[f])) {
            gs_set_push(GS_SETEV_CHANGE, f, e->a == AT_DIR_LEFT ? -1 : 1);
        }
        break;
    case AT_EV_FOCUS:
        if (e->a == 0) gs_set_move_to(u, e->b);
        break;
    case AT_EV_ACCEPT:
        if (f >= 0 && f < u->sc.n_items) gs_set_push(GS_SETEV_ACCEPT, f, 0);
        break;
    case AT_EV_BACK:
        gs_set_push(GS_SETEV_BACK, -1, 0);
        break;
    case AT_EV_PAGE:
        if (u->sc.n_tabs > 1) {
            int t = u->view.tab + (e->a < 0 ? -1 : 1);
            t = (t % u->sc.n_tabs + u->sc.n_tabs) % u->sc.n_tabs;
            gs_set_push(GS_SETEV_TAB, -1, t);
        }
        break;
    case AT_EV_ALT:
        gs_set_push(GS_SETEV_ALT, f >= 0 && f < u->sc.n_items ? f : -1, e->a);
        break;
    default: break;
    }
}

/* the mouse over the settings: a hover focuses (a resting pointer never steals the focus from the pad), a click is A on the row, a right click is B, the wheel steps
 * the hovered value or moves the focus, a tab and a key hint are buttons. The first sample only primes: a button held from the menu that opened the screen
 * does not click on it. */
static void gs_set_mouse(GsUiSlot *u) {
    float x, y, wheel;
    int buttons, hit, moved, left, right, kind = 0, a = 0, b = 0;
    AtMouse *m = &u->mouse;
    gw_Mouse_ScriptRead(&x, &y, &buttons, &wheel);
    if (x < 0.0f || y < 0.0f) { m->valid = 0; m->buttons = buttons; return; }
    moved = m->valid && (x != m->x || y != m->y);
    left = (buttons & 1) && !(m->buttons & 1) && m->valid;
    right = (buttons & 2) && !(m->buttons & 2) && m->valid;
    hit = at_hit_test(&u->hits, x, y);
    if (hit >= 0) { kind = u->hits.h[hit].kind; a = u->hits.h[hit].a; b = u->hits.h[hit].b; }
    if (u->view.dialog.open) {                                     /* only the dialog's buttons are hit */
        if (left && kind == AT_HIT_DIALOG) gs_set_answer(u, a == 'A' ? 1 : 2);
        else if (right) gs_set_answer(u, 2);
    } else {
        int row = kind == AT_HIT_CELL ? b : -1;
        if (row >= 0 && (moved || left)) gs_set_move_to(u, row);
        if (left && row >= 0) gs_set_push(GS_SETEV_ACCEPT, row, 0);
        else if (left && kind == AT_HIT_TAB) gs_set_push(GS_SETEV_TAB, -1, a);
        else if (left && kind == AT_HIT_KEY) {
            AtEvent e;
            memset(&e, 0, sizeof e);
            if (a == 'A') e.type = AT_EV_ACCEPT;
            else if (a == 'B') e.type = AT_EV_BACK;
            else if (a == 'L' || a == 'R') { e.type = AT_EV_PAGE; e.a = a == 'L' ? -1 : 1; }
            else if (a == 'X' || a == 'Y' || a == 'Z') { e.type = AT_EV_ALT; e.a = a; }
            if (e.type != 0) gs_set_event(u, &e);
        }
        if (right) gs_set_push(GS_SETEV_BACK, -1, 0);
        if ((int) wheel != 0) {
            int dir = wheel > 0 ? 1 : -1;
            if (row >= 0 && gs_set_steppable(&u->sc.items[row])) {   /* the wheel steps the hovered value ... */
                gs_set_move_to(u, row);
                gs_set_push(GS_SETEV_CHANGE, row, dir);
            } else if (u->sc.n_items > 0) {                          /* ... and scrolls the list over anything else (clamped, as the legacy wheel was) */
                int to = u->view.focus.index - dir;
                gs_set_move_to(u, to < 0 ? 0 : to >= u->sc.n_items ? u->sc.n_items - 1 : to);
            }
        }
    }
    m->valid = 1;
    m->x = x;
    m->y = y;
    m->buttons = buttons;
}

/* once per script tick, when the settings are the top screen: the keyboard, then the mouse */
static void gs_set_tick(GsUiSlot *u) {
    AtEvent ev[16];
    int n, i;
    gs_set_resolve(u);
    if (gs_set.frozen) return;                                     /* no event, and no state kept: Freeze(0) primes from what is held then */
    n = at_key_events(&u->keys, gs_ui_key_mask(), gs_now_ms(), ev, 16);
    for (i = 0; i < n; i++) gs_set_event(u, &ev[i]);
    gs_set_mouse(u);
}

/* ---- the shims ---------------------------------------------------------------------------------------------------------------- */

/* 1 when gw_Ui_SetOpen would succeed now: Atlas is on, no native screen is open, a slot is free */
int gw_Ui_SetCanOpen(void) {
    int i;
    if (!gw_Ui_Ready() || gs_set_is_open() || gs_room_is_open() || gw_Ui_SelCanOpen() == 0) return 0;
    for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) return 1;
    return 0;
}

/* kind 0: the tabbed pages, 1: the remap editor, 2: the how-to page, 3: the erase screen, 4: VERSUS > RULES, 5: MORE RULES, 6: MATCH SETUP (4 to 6 are the Versus chapter's), 7: ONLINE PLAY (the Online chapter's rows: Host, Join, Random Opponent and the rules). t0 and t1: the trail's parents (SETTINGS > CONTROLS), title: the screen's name.
 * Returns a handle, or -1 (roles missing, MELEE_ATLAS=0, another native screen open, no free slot: the caller keeps the legacy drawing). */
int gw_Ui_SetOpen(int kind, int active_tab, const char *t0, const char *t1, const char *title) {
    int i, slot = -1;
    GsUiSlot *u;
    AtScreen *sc;
    static const char *const ids[8] = { "settings.tabs", "settings.remap", "settings.howto", "settings.erase", "settings.rules", "settings.morerules", "settings.match", "online.play" };
    if (!gw_Ui_Ready() || kind < GS_SET_TABS || kind > GS_SET_ONLINE) return -1;
    if (gs_set_is_open() || gs_room_is_open() || gw_Ui_SelCanOpen() == 0) return -1;     /* one native screen at a time: the stack has one top */
    for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) { slot = i; break; }
    if (slot < 0) { gw_log("ui: no free screen slot for the settings screen"); return -1; }
    u = &gs_ui_slot[slot];
    memset(u, 0, sizeof *u);                                       /* never reused state */
    at_view_init(&u->view);
    u->dialog_fn = -1;
    u->used = 1;
    u->owner = GS_UI_ENGINE;
    u->native_set = 1;
    u->scene = gs.scene_kind;
    sc = &u->sc;
    sc->fn_provide = sc->fn_accept = sc->fn_back = sc->fn_focus = sc->fn_change = sc->fn_open = sc->fn_close = sc->fn_counter = sc->fn_page = sc->fn_start = -1;
    sc->fn_alt[0] = sc->fn_alt[1] = sc->fn_alt[2] = -1;
    snprintf(sc->id, sizeof sc->id, "%s", ids[kind]);
    snprintf(sc->title, sizeof sc->title, "%s", title != NULL ? title : sc->id);
    if (t0 != NULL && t0[0] != '\0') snprintf(sc->parent[sc->n_parents++], AT_STR, "%s", t0);
    if (t1 != NULL && t1[0] != '\0') snprintf(sc->parent[sc->n_parents++], AT_STR, "%s", t1);
    sc->chapter = kind == GS_SET_ONLINE ? 3 : kind >= GS_SET_RULES ? 2 : 5;                    /* the Versus chapter for the rules screens, the settings chapter otherwise (fa_chapter_of) */
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_NORMAL;                                 /* the explainer shows the focused row's help and value */
    sc->port = 1;
    sc->input_feed = 1;                                            /* the game feeds the pad (its menu input, every port merged) */
    u->view.tab = active_tab < 0 ? 0 : active_tab >= AT_MAX_TABS ? AT_MAX_TABS - 1 : active_tab;
    u->view.focus.block = 0; u->view.focus.index = -1;
    { int gen = gs_set.gen; memset(&gs_set, 0, sizeof gs_set); gs_set.gen = gen; }   /* a clean state; the generation only grows */
    gs_set.h = -1;
    gs_set.pending = 1; gs_set.fidx = -1;                          /* the first rows put the focus on the first row */
    if (!at_stack_push(&gs_ui_stack, slot)) { memset(u, 0, sizeof *u); u->dialog_fn = -1; return -1; }
    u->view.opened_ms = gs_now_ms();
    memset(&u->mouse, 0, sizeof u->mouse);                         /* the first sample only primes: a button held from the menu does not click here */
    u->primed_frame = gs_ui_frame + 1;                             /* an intent of this very frame is the press that opened the screen */
    gs_set.gen = (gs_set.gen + 1) & 0xFFFFF;
    gs_set.h = (gs_set.gen << 8) | slot;
    gw_log("ui: settings screen %s open (native)", sc->id);
    return gs_set.h;
}

void gw_Ui_SetClose(int h) {
    GsUiSlot *u = gs_set_get(h);
    int slot;
    if (u == NULL) return;
    slot = (int) (u - gs_ui_slot);
    gs_set.h = -1;
    gs_set.qn = 0; gs_set.frozen = 0; gs_set.answer = 0; gs_set.pending = 0;
    gs_ui_free_engine_slot(slot);
    gs_ui_after_close();
}

/* "VIDEO,AUDIO,CONTROLS": the tab names (at most AT_MAX_TABS, each cut at 23); active is the one shown. The count is -1: a settings tab has no number. */
void gw_Ui_SetTabs(int h, int n, const char *names, int active) {
    GsUiSlot *u = gs_set_get(h);
    const char *p;
    int k = 0;
    if (u == NULL) return;
    if (n > AT_MAX_TABS) n = AT_MAX_TABS;
    for (p = names != NULL ? names : ""; k < n && *p != '\0';) {
        const char *end = strchr(p, ',');
        size_t len = end != NULL ? (size_t) (end - p) : strlen(p);
        snprintf(u->sc.tabs[k].name, sizeof u->sc.tabs[k].name, "%.*s", (int) (len < sizeof u->sc.tabs[k].name - 1 ? len : sizeof u->sc.tabs[k].name - 1), p);
        u->sc.tabs[k].count = -1;
        k++;
        p += len + (end != NULL ? 1 : 0);
    }
    u->sc.n_tabs = k;
    u->view.tab = active < 0 ? 0 : active >= k ? (k > 0 ? k - 1 : 0) : active;
}

/* the page's rows, every frame: SetRows says how many, then SetRow, SetRowVal, SetRowText and SetRowOpt fill each. A slot past the count is ignored. */
void gw_Ui_SetRows(int h, int n) {
    GsUiSlot *u = gs_set_get(h);
    if (u == NULL) return;
    if (!gs_set.pending) {                                         /* remember the focus once per submit, by id and by place */
        int f = u->view.focus.index;
        gs_set.fid[0] = '\0';
        if (f >= 0 && f < u->sc.n_items) snprintf(gs_set.fid, sizeof gs_set.fid, "%s", u->sc.items[f].id);
        gs_set.fidx = f;
        gs_set.pending = 1;
    }
    u->sc.n_items = n < 0 ? 0 : n > AT_MAX_ITEMS ? AT_MAX_ITEMS : n;
}

static AtItem *gs_set_row(GsUiSlot *u, int slot) { return (slot >= 0 && slot < u->sc.n_items) ? &u->sc.items[slot] : NULL; }

void gw_Ui_SetRow(int h, int slot, const char *id, const char *label, const char *help, int vkind) {
    GsUiSlot *u = gs_set_get(h);
    AtItem *it;
    if (u == NULL || (it = gs_set_row(u, slot)) == NULL) return;
    memset(it, 0, sizeof *it);                                     /* a row starts clean: nothing of the last page's row stays */
    snprintf(it->id, sizeof it->id, "%s", id != NULL ? id : "");
    snprintf(it->label, sizeof it->label, "%s", label != NULL ? label : "");
    it->vkind = vkind >= AT_VAL_TOGGLE && vkind <= AT_VAL_COUNTER ? vkind : AT_VAL_NONE;
    gs_set_clamp(gs_set.help[slot], AT_TEXT, help);
}

void gw_Ui_SetRowVal(int h, int slot, int vmin, int vmax, int vstep, int value, unsigned flags) {
    GsUiSlot *u = gs_set_get(h);
    AtItem *it;
    if (u == NULL || (it = gs_set_row(u, slot)) == NULL) return;
    it->vmin = vmin; it->vmax = vmax; it->vstep = vstep;
    it->vval = (vmax > vmin && it->vkind == AT_VAL_SLIDER) ? (value < vmin ? vmin : value > vmax ? vmax : value) : value;   /* a slider is pulled into its range; a choice outside its options shows nothing, never a wrong option */
    it->on = value != 0;
    it->iflags = flags;
    if (flags & AT_ITEM_DISABLED) it->flags |= AT_CELL_DISABLED;
}

void gw_Ui_SetRowText(int h, int slot, const char *text, const char *reason, const char *group) {
    GsUiSlot *u = gs_set_get(h);
    AtItem *it;
    if (u == NULL || (it = gs_set_row(u, slot)) == NULL) return;
    snprintf(it->text, sizeof it->text, "%s", text != NULL ? text : "");
    snprintf(it->reason, sizeof it->reason, "%s", reason != NULL ? reason : "");
    snprintf(it->group, sizeof it->group, "%s", group != NULL ? group : "");
}

/* option k of a choice the engine shows (the text of the row is the option of its value) */
void gw_Ui_SetRowOpt(int h, int slot, int k, const char *text) {
    GsUiSlot *u = gs_set_get(h);
    AtItem *it;
    if (u == NULL || (it = gs_set_row(u, slot)) == NULL || k < 0 || k >= AT_MAX_OPTS) return;
    snprintf(it->opt[k], sizeof it->opt[k], "%s", text != NULL ? text : "");
    if (k + 1 > it->n_opts) it->n_opts = k + 1;
    if (it->vkind == AT_VAL_CHOICE) at_item_text(AT_VAL_CHOICE, it->vval, it->vmin, (const char (*)[24]) it->opt, it->n_opts, "", it->text, (int) sizeof it->text);
}

/* the new value of a row after a step (the legacy fe_change rule: at_item_apply); cur when nothing changed. No pointer: the game reads the return value. */
int gw_Ui_ItemStep(int vkind, int vmin, int vmax, int vstep, int cur, int dir) {
    int v = cur;
    return at_item_apply(vkind, vmin, vmax, vstep, cur, dir, &v) ? v : cur;
}

/* the focus on a row by id (after a rebuild or a tab switch), else the first row */
void gw_Ui_SetFocus(int h, const char *item_id) {
    GsUiSlot *u = gs_set_get(h);
    int i, idx = -1;
    if (u == NULL) return;
    gs_set.pending = 0;
    for (i = 0; item_id != NULL && item_id[0] != '\0' && i < u->sc.n_items; i++) if (strcmp(u->sc.items[i].id, item_id) == 0) { idx = i; break; }
    if (idx < 0) idx = u->sc.n_items > 0 ? 0 : -1;
    u->view.focus.block = 0; u->view.focus.index = idx;
    if (idx >= 0) gs_ui_fix_scroll(u);
}

/* the focused row's slot, or -1 (the adapter reads it every frame: it speaks for the focused row) */
int gw_Ui_SetFocused(int h) {
    GsUiSlot *u = gs_set_get(h);
    if (u == NULL) return -1;
    gs_set_resolve(u);
    return (u->view.focus.index >= 0 && u->view.focus.index < u->sc.n_items) ? u->view.focus.index : -1;
}

/* The explainer, with no media well. A NULL kicker is the active tab's name, a NULL title the focused row's label, a NULL what its help (clamped to 159
 * characters with "...", never cut silently), a NULL now "NOW " and the value the row shows (a toggle ON or OFF, an option, a slider's text; none for a readout);
 * an empty string says nothing. */
void gw_Ui_SetExplain(int h, const char *kicker, const char *title, const char *what, const char *now, const char *from) {
    GsUiSlot *u = gs_set_get(h);
    AtExplainer *e;
    const AtItem *it = NULL;
    char tmp[AT_STR], val[AT_STR];
    int f;
    if (u == NULL) return;
    gs_set_resolve(u);
    f = u->view.focus.index;
    if (f >= 0 && f < u->sc.n_items) it = &u->sc.items[f];
    e = &u->view.ex;
    memset(e, 0, sizeof *e);
    e->media_model = e->media_ring = AT_NO_MODEL;
    e->media_tex = -1;
    e->no_well = 1;
    snprintf(e->kicker, sizeof e->kicker, "%s", kicker != NULL ? kicker : (u->sc.n_tabs > 0 && u->view.tab < u->sc.n_tabs ? u->sc.tabs[u->view.tab].name : u->sc.title));
    snprintf(e->title, sizeof e->title, "%s", title != NULL ? title : (it != NULL ? it->label : ""));
    gs_set_clamp(e->what, AT_TEXT, what != NULL ? what : (it != NULL ? gs_set.help[f] : ""));
    if (now != NULL) {
        if (now[0] != '\0') { snprintf(tmp, sizeof tmp, "NOW %s", now); snprintf(e->now_text, sizeof e->now_text, "%s", tmp); }
    } else if (it != NULL && it->vkind != AT_VAL_NONE && it->vkind != AT_VAL_TEXT && it->vkind != AT_VAL_COUNTER) {
        at_item_text(it->vkind, it->vkind == AT_VAL_TOGGLE ? it->on : it->vval, it->vmin, (const char (*)[24]) it->opt, it->n_opts, it->text, val, (int) sizeof val);
        if (val[0] != '\0') { snprintf(tmp, sizeof tmp, "NOW %s", val); snprintf(e->now_text, sizeof e->now_text, "%s", tmp); }
    }
    snprintf(e->from_text, sizeof e->from_text, "%s", from != NULL ? from : "");
    e->has = e->kicker[0] != '\0' || e->title[0] != '\0';
}

/* "A:Change,L:Page,B:Back": a button (A B X Y Z L R, S = START), a colon, its label; parsed only when the text changed */
void gw_Ui_SetKeys(int h, const char *spec) {
    GsUiSlot *u = gs_set_get(h);
    const char *p;
    int n = 0, i;
    if (u == NULL || spec == NULL) return;
    if (strcmp(spec, gs_set.spec) == 0) return;
    snprintf(gs_set.spec, sizeof gs_set.spec, "%s", spec);
    memset(u->sc.keys, 0, sizeof u->sc.keys);
    memset(u->view.key_label, 0, sizeof u->view.key_label);
    memset(u->view.key_shown, 0, sizeof u->view.key_shown);
    for (p = spec; *p != '\0' && n < AT_MAX_KEYS;) {
        const char *end = strchr(p, ',');
        size_t len = end != NULL ? (size_t) (end - p) : strlen(p);
        if (len >= 3 && p[1] == ':' && strchr("ABXYZLRS", p[0]) != NULL) {
            AtKey *k = &u->sc.keys[n];
            k->btn = p[0];
            k->fn_label = k->fn_when = -1;
            snprintf(k->label, sizeof k->label, "%.*s", (int) (len - 2 < AT_STR - 1 ? len - 2 : AT_STR - 1), p + 2);
            n++;
        }
        p += len + (end != NULL ? 1 : 0);
    }
    u->sc.n_keys = n;
    for (i = 0; i < n; i++) {
        snprintf(u->view.key_label[i], AT_STR, "%s", u->sc.keys[i].label);
        u->view.key_shown[i] = u->sc.keys[i].label[0] != '\0';
    }
}

void gw_Ui_SetNote(int h, const char *text, int kind) {
    GsUiSlot *u = gs_set_get(h);
    double now;
    if (u == NULL || text == NULL || text[0] == '\0') return;
    now = gs_now_ms();
    if (strcmp(u->view.note.text, text) == 0 && now < u->view.note.until_ms) return;   /* the same note, still showing: not restarted */
    snprintf(u->view.note.text, sizeof u->view.note.text, "%s", text);
    u->view.note.kind = kind < AT_NOTE_OK || kind > AT_NOTE_INFO ? AT_NOTE_INFO : kind;
    u->view.note.from_ms = now;
    u->view.note.until_ms = now + 2500.0;
}

/* a dialog over the rows. Its focus starts on the button named by default_button (2 = the second, Cancel in every destructive dialog); A answers the FOCUSED
 * button, B always the second (cancel), left and right move. An answer is taken once with gw_Ui_SetDialogAnswer (0 pending, 1 first, 2 second). */
void gw_Ui_SetDialog(int h, const char *title, const char *text, const char *a_label, const char *b_label, int default_button) {
    GsUiSlot *u = gs_set_get(h);
    AtDialog *d;
    if (u == NULL) return;
    d = &u->view.dialog;
    memset(d, 0, sizeof *d);
    snprintf(d->title, sizeof d->title, "%s", title != NULL ? title : "");
    snprintf(d->body, sizeof d->body, "%s", text != NULL ? text : "");
    d->btn[0] = 'A'; d->btn[1] = 'B';
    snprintf(d->label[0], sizeof d->label[0], "%s", a_label != NULL ? a_label : "");
    snprintf(d->label[1], sizeof d->label[1], "%s", b_label != NULL ? b_label : "");
    d->n = (b_label != NULL && b_label[0] != '\0') ? 2 : 1;
    d->focus = (default_button == 2 && d->n == 2) ? 1 : 0;
    d->from_ms = gs_now_ms();
    d->open = 1;
    gs_set.answer = 0;
    gs_ui_prime((int) (u - gs_ui_slot));                           /* the keys and the mouse start from what is held; an intent of this frame is dropped */
}

int gw_Ui_SetDialogAnswer(int h) {
    int a;
    if (gs_set_get(h) == NULL) return 0;
    a = gs_set.answer;
    gs_set.answer = 0;
    return a;
}

/* every event is dropped while on (a name being typed, a controller capture); a frozen screen still draws. When it ends the keys and the mouse start from what
 * is held at that instant: a key that was down when the edit ended does not also accept or go back. */
void gw_Ui_Freeze(int h, int on) {
    GsUiSlot *u = gs_set_get(h);
    if (u == NULL) return;
    on = on != 0;
    if (on == gs_set.frozen) return;
    gs_set.frozen = on;
    if (on) gs_set.qn = 0;                                         /* a Back or Enter that arrived in the frame the freeze began must not leak out after it */
    if (!on) gs_ui_prime((int) (u - gs_ui_slot));
}

/* the game's pad, as events: AT_EV_MOVE (a = AT_DIR_*), AT_EV_ACCEPT, AT_EV_BACK, AT_EV_PAGE (a = -1 or 1), AT_EV_ALT (a = 'X', 'Y', 'Z') */
void gw_Ui_SetIntent(int h, int type, int a) {
    GsUiSlot *u = gs_set_get(h);
    AtEvent e;
    if (u == NULL || at_stack_top(&gs_ui_stack) != (int) (u - gs_ui_slot)) return;
    if (u->primed_frame == gs_ui_frame + 1) return;                 /* the press that opened or uncovered it is not its own */
    if (!gs_may_run(GS_UI_ENGINE)) return;
    memset(&e, 0, sizeof e);
    e.type = type; e.a = a;
    gs_set_event(u, &e);
}

/* the next event: its type (GS_SETEV_*, 0 none); slot and arg are the game's locals, written big-endian */
int gw_Ui_SetPoll(int h, int *slot, int *arg) {
    int type, i;
    if (gs_set_get(h) == NULL || gs_set.qn <= 0 || gs_set.frozen) return 0;   /* nothing is delivered while frozen */
    type = gs_set.q[0].type;
    gs_ui_put_be32(slot, gs_set.q[0].a); gs_ui_put_be32(arg, gs_set.q[0].b);
    for (i = 1; i < gs_set.qn; i++) gs_set.q[i - 1] = gs_set.q[i];
    gs_set.qn--;
    return type;
}
