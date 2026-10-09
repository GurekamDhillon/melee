/* gw_script_ui_mods.inc - the host half of the MODS screen (Atlas step 7). Included at the end of gw_script_ui.inc, after gw_script_ui_set.inc: it is one more kind
 * of the engine-owned settings screen (gw_Ui_SetOpen kind GS_SET_MODS), so the slot, the handle, the pad intents (gw_Ui_SetIntent), the key and mouse priming and the
 * close are the settings door's own.
 *
 * What differs: the screen has no table behind it on the game side. The model (gw_ui_mods.c) turns the mods API into the screen record and applies the events, so
 * the keys, the mouse and the pad intents all end in gs_mods_event, and the screen is rebuilt after every event and every tick (a note expires, a restart becomes
 * needed). The game side only opens it, feeds the pad, and drains one event: BACK (GS_SETEV_BACK), which leaves the screen through the settings adapter's own back.
 *
 * The screen never runs a mod's code. A mod's own settings row asks the registry to activate the entry; the registry pushes the mod's registered screen over this
 * one (a mods screen may be covered, a select or another settings screen may not) and B on it comes back here. Nothing in this file writes game memory.
 *
 * ONLINE. The accessors refuse every change while a netplay or rollback session is on (the model says so and shows the rows read-only); the screen can still be
 * opened and read. The mounted set cannot change until a restart, so what the screen shows stays true for the match. */

#include "gw_ui_mods.h"
#include "gw_ui_mods_native.h"
#include "gw_ui_nucleus.inc"      /* the NUCLEUS screens (modes 2 and 3 below): a pure model over a source, gw_nucleus_engine.inc answers it */
#include "gw_nucleus_engine.inc"

static struct {
    int mode;                   /* 0 the list, 1 a mod's detail, 2 the NUCLEUS browser, 3 a Nucleus mod's detail */
    AtNcState nc;
    AtNcSrc ncsrc;
    AtModsState st;
    AtModsDetail dt;
    AtModsSrc src;
    AtModsHooks hooks;
    int hit_base;               /* the window base of the record that was last DRAWN: the hit rectangles' row numbers are rows of that window */
} gs_mods;
static const AtModsSrc *gs_mods_test_src;   /* tests: a source over a fake folder instead of gw_Mods_* */

static int gs_mods_locked(void) { return gs_ui_netplay(); }

/* the registry holds that entry, it is visible now, and the mod that owns it has a running script (so activating it can do something) */
static int gs_mods_entry_ok(const char *id) {
    const AtEntry *e = gs_ui_entry_visible(id);
    return e != NULL && !e->builtin && gs_ui_mod_script(e->mod, NULL) >= 0;
}

/* called by the draw pass after it drew the MODS screen: the hit rectangles it just made belong to the window as it is now */
static void gs_mods_drawn(void) { gs_mods.hit_base = gs_mods.mode == 2 ? gs_mods.nc.base : gs_mods.st.base; }

static void gs_mods_rebuild(GsUiSlot *u) {
    double now = gs_now_ms();
    if (gs_mods.mode == 3 && !at_nc_detail_build(&gs_mods.ncsrc, &gs_mods.nc, &u->sc, &u->view, now)) gs_mods.mode = 2;
    if (gs_mods.mode == 2) { gs_mods.nc.mode = 0; at_nc_build(&gs_mods.ncsrc, &gs_mods.nc, &u->sc, &u->view, now); return; }
    if (gs_mods.mode == 3) return;
    if (gs_mods.mode == 1 && !at_mods_detail_build(&gs_mods.src, &gs_mods.dt, &u->sc, &u->view, now)) gs_mods.mode = 0;   /* the mod is gone: back to the list */
    if (gs_mods.mode == 0) at_mods_build(&gs_mods.src, &gs_mods.st, &u->sc, &u->view, now);
}

static void gs_mods_open(GsUiSlot *u) {
    memset(&gs_mods, 0, sizeof gs_mods);
    gs_mods.hooks.locked = gs_mods_locked;
    gs_mods.hooks.entry_ok = gs_mods_entry_ok;
    if (gs_mods_test_src != NULL) gs_mods.src = *gs_mods_test_src;
    else at_mods_native_src(&gs_mods.src, &gs_mods.hooks);
    at_mods_state_init(&gs_mods.st);
    at_nc_state_init(&gs_mods.nc);
    ne_make_src(&gs_mods.ncsrc);
    u->native_mods = 1;
    gs_mods_rebuild(u);
}

/* one input event, from the game's intents, the keys or the mouse */
static void gs_nucleus_act(const AtNcAction *a) {
    if (a->kind != AT_NA_ACT) return;
    if (!gs_mods.ncsrc.act(gs_mods.ncsrc.user, a->id, a->what)) gw_log("ui: nucleus action %d on %d refused", a->what, a->id);
}

static void gs_mods_event(GsUiSlot *u, const AtEvent *e) {
    AtModsAction a;
    char entry[AT_ID * 2];
    int open_entry = 0;
    if (gs_mods.mode >= 2) {
        AtNcAction na;
        if (gs_mods.mode == 3) {
            gs_mods.nc.mode = 1;
            at_nc_detail_event(&gs_mods.ncsrc, &gs_mods.nc, e, gs_now_ms(), &na);
            gs_mods.mode = gs_mods.nc.mode ? 3 : 2;
        } else {
            at_nc_event(&gs_mods.ncsrc, &gs_mods.nc, e, gs_now_ms(), &na);
            if (na.kind == AT_NA_BACK) gs_mods.mode = 0;
            else if (gs_mods.nc.mode) gs_mods.mode = 3;
        }
        gs_nucleus_act(&na);
        gs_mods_rebuild(u);
        return;
    }
    if (gs_mods.mode == 1) {
        at_mods_detail_event(&gs_mods.src, &gs_mods.dt, e, &a);
        if (a.kind == AT_MA_BACK) gs_mods.mode = 0;
        else if (a.kind == AT_MA_SETTINGS) { snprintf(entry, sizeof entry, "%s", a.id); open_entry = 1; }
    } else {
        at_mods_event(&gs_mods.src, &gs_mods.st, e, gs_now_ms(), &a);
        if (a.kind == AT_MA_BACK) gs_set_push(GS_SETEV_BACK, -1, 0);                     /* the adapter leaves the screen: it is the game's scene to leave */
        else if (a.kind == AT_MA_DETAIL) { memset(&gs_mods.dt, 0, sizeof gs_mods.dt); gs_mods.dt.mod = a.arg; gs_mods.mode = 1; }
        else if (a.kind == AT_MA_NUCLEUS) { ne_open(); gs_mods.mode = 2; gs_mods.nc.stale = 1; }
    }
    gs_mods_rebuild(u);
    if (open_entry && !gs_ui_activate_id(entry)) gw_log("ui: the mod's settings entry %s did not open", entry);
}

/* the mouse: a hover focuses (a resting pointer never steals the focus from the pad), a click is A, a right click is B, the wheel scrolls, a key hint is a button, and
 * a click on the other tab changes tab. The first sample only primes: a button held from the menu that opened the screen does not click on it. */
static void gs_mods_mouse(GsUiSlot *u) {
    float x, y, wheel;
    int buttons, n, i, hit;
    int left;
    AtEvent ev[16];
    gw_Mouse_ScriptRead(&x, &y, &buttons, &wheel);
    left = (buttons & 1) && !(u->mouse.buttons & 1) && u->mouse.valid && x >= 0.0f && y >= 0.0f;
    if (gs_mods.mode == 2 && gs_mods.nc.confirm) {                    /* the clear-the-queue dialog: only its buttons are hit, and a right click is Keep */
        int right = (buttons & 2) && !(u->mouse.buttons & 2) && u->mouse.valid && x >= 0.0f && y >= 0.0f, hit2 = left ? at_hit_test(&u->hits, x, y) : -1;
        AtEvent de;
        memset(&de, 0, sizeof de);
        if (hit2 >= 0 && u->hits.h[hit2].kind == AT_HIT_DIALOG) { gs_mods.nc.confirm_focus = u->hits.h[hit2].a == 'A' ? 0 : 1; de.type = AT_EV_ACCEPT; gs_mods_event(u, &de); }
        else if (right) { de.type = AT_EV_BACK; gs_mods_event(u, &de); }
        u->mouse.buttons = buttons; u->mouse.x = x; u->mouse.y = y; u->mouse.valid = x >= 0.0f && y >= 0.0f;
        return;
    }
    if (left && (gs_mods.mode == 0 || gs_mods.mode == 2)) {
        hit = at_hit_test(&u->hits, x, y);
        if (hit >= 0 && u->hits.h[hit].kind == AT_HIT_TAB) {
            if (gs_mods.mode == 2 && !gs_mods.nc.edit) { if (u->hits.h[hit].a >= 0 && u->hits.h[hit].a < AT_NC_TABS && u->hits.h[hit].a != gs_mods.nc.tab) { gs_mods.nc.tab = u->hits.h[hit].a; at_nc_apply_tab(&gs_mods.nc); } }
            else if (u->hits.h[hit].a != u->view.tab) { AtEvent pg; memset(&pg, 0, sizeof pg); pg.type = AT_EV_PAGE; pg.a = 1; gs_mods_event(u, &pg); }
            u->mouse.buttons = buttons; u->mouse.x = x; u->mouse.y = y;                   /* the click was this tab's: it is not also a click on what is under it */
            return;
        }
    }
    n = at_mouse_events(&u->mouse, x, y, buttons, (int) wheel, &u->hits, ev, 16);
    for (i = 0; i < n; i++) {
        /* a row the pointer is over is a row of the window as it was drawn: a key or a pad press earlier in this frame may have moved the window since */
        if (ev[i].type == AT_EV_FOCUS && gs_mods.mode == 0) ev[i].b += gs_mods.hit_base - gs_mods.st.base;
        if (ev[i].type == AT_EV_FOCUS && gs_mods.mode == 2) ev[i].b += gs_mods.hit_base - gs_mods.nc.base;
        gs_mods_event(u, &ev[i]);
    }
}

/* once per script tick, when the MODS screen is the top screen: the keyboard, then the mouse, then the rebuild (a note expires, the restart note appears) */
/* The Nucleus keyboard. Typed characters for the search (edge-detected from the host's key table; the arrows and Esc already reach the model as events), and
 * outside a search: Space is A (queue the mod), Enter is START (begin the downloads), X / Y / Z are the pad's X / Y / Z. The table of previous states is updated
 * every tick, search or not, so a key held down while the search ends is not a press for the list. */
static unsigned char gs_nc_prev[256];
static int gs_nucleus_edge(int vk) { return gs.key_now[vk] != 0 && !gs_nc_prev[vk]; }

static void gs_nucleus_typing(void) {
    int vk;
    for (vk = 8; vk < 256; vk++) {
        int ch = 0;
        if (gs_nucleus_edge(vk)) {
            if ((vk >= 'A' && vk <= 'Z') || (vk >= '0' && vk <= '9') || vk == ' ') ch = vk;
            else if (vk == VK_BACK) ch = 8;
            if (ch && gs_mods.nc.edit) at_nc_type(&gs_mods.nc, ch);
        }
    }
}

static void gs_nucleus_keys(GsUiSlot *u) {
    AtEvent e;
    memset(&e, 0, sizeof e);
    if (gs_nucleus_edge(VK_SPACE)) { e.type = AT_EV_ACCEPT; gs_mods_event(u, &e); }
    else if (gs_nucleus_edge(VK_RETURN)) { e.type = AT_EV_START; gs_mods_event(u, &e); }
    else if (gs_nucleus_edge('X')) { e.type = AT_EV_ALT; e.a = 'X'; gs_mods_event(u, &e); }
    else if (gs_nucleus_edge('Y')) { e.type = AT_EV_ALT; e.a = 'Y'; gs_mods_event(u, &e); }
    else if (gs_nucleus_edge('Z')) { e.type = AT_EV_ALT; e.a = 'Z'; gs_mods_event(u, &e); }
}

static void gs_nucleus_keys_end(void) {
    int vk;
    for (vk = 0; vk < 256; vk++) gs_nc_prev[vk] = gs.key_now[vk] != 0;
}

static void gs_mods_tick(GsUiSlot *u) {
    AtEvent ev[16];
    int n, i, nucleus = gs_mods.mode >= 2, editing = nucleus && gs_mods.mode == 2 && gs_mods.nc.edit;
    unsigned km = gs_ui_key_mask();
    ne_pump();
    if (editing) gs_nucleus_typing();
    if (nucleus && !editing) km &= ~(unsigned) AT_KEY_ENTER;           /* Enter starts the downloads here; it is A only while a search is typed */
    n = at_key_events(&u->keys, km, gs_now_ms(), ev, 16);
    for (i = 0; i < n; i++) gs_mods_event(u, &ev[i]);
    if (nucleus && !editing && !gs_mods.nc.confirm) gs_nucleus_keys(u);
    else if (nucleus && gs_mods.nc.confirm && gs_nucleus_edge(VK_RETURN)) { AtEvent a; memset(&a, 0, sizeof a); a.type = AT_EV_ACCEPT; gs_mods_event(u, &a); }
    gs_nucleus_keys_end();
    gs_mods_mouse(u);
    gs_mods_rebuild(u);
}
