#include "atlas_lint.h"
#include "../platform/gw_ui_mods.h"
#include "../platform/gw_ui_render.h"

/* ---- a fake mods folder with the real resolver's cascade rules ---- */
#define FM_MAX 256
typedef struct { char id[40], name[64], kind[12], pack[12], desc[160], req[64], con[64], status_text[100], adds[3][64]; int status, enabled, active, nadds; char settings[40]; } FMod;
static FMod FM[FM_MAX];
static int fm_n, fm_restart, fm_save_calls, fm_save_result, fm_locked_now, fm_set_calls;

static int fm_find(const char *id) { int i; for (i = 0; i < fm_n; i++) if (strcmp(FM[i].id, id) == 0) return i; return -1; }
static int has_word(const char *list, const char *id)
{
    char buf[100], *p, *save = NULL;
    snprintf(buf, sizeof buf, "%s", list);
    for (p = strtok_s(buf, ",", &save); p != NULL; p = strtok_s(NULL, ",", &save)) if (_stricmp(p, id) == 0) return 1;
    return 0;
}
static void fm_recompute_restart(void) { int i; fm_restart = 0; for (i = 0; i < fm_n; i++) if (FM[i].enabled != FM[i].active) fm_restart = 1; }
/* enabling turns requirements on and enabled conflicting mods off (either direction); disabling turns dependents off */
static int fm_set(void *u, int i, int on)
{
    int j, changed = 0;
    (void) u;
    fm_set_calls++;
    if (i < 0 || i >= fm_n || FM[i].enabled == on) return 0;
    FM[i].enabled = on; changed++;
    for (j = 0; j < fm_n; j++) {
        if (j == i) continue;
        if (on && has_word(FM[i].req, FM[j].id) && !FM[j].enabled) { FM[j].enabled = 1; changed++; }
        if (on && FM[j].enabled && (has_word(FM[i].con, FM[j].id) || has_word(FM[j].con, FM[i].id))) { FM[j].enabled = 0; changed++; }
        if (!on && FM[j].enabled && has_word(FM[j].req, FM[i].id)) { FM[j].enabled = 0; changed++; }
    }
    fm_recompute_restart();
    return changed;
}
static int fm_count(void *u) { (void) u; return fm_n; }
#define FM_STR(field) static const char *fm_##field(void *u, int i) { (void) u; return FM[i].field; }
FM_STR(id) FM_STR(name) FM_STR(kind) FM_STR(pack) FM_STR(desc) FM_STR(status_text)
static const char *fm_version(void *u, int i) { (void) u; (void) i; return "1.0"; }
static const char *fm_requires(void *u, int i) { (void) u; return FM[i].req; }
static const char *fm_conflicts(void *u, int i) { (void) u; return FM[i].con; }
static int fm_status(void *u, int i) { (void) u; return FM[i].status; }
static int fm_enabled(void *u, int i) { (void) u; return FM[i].enabled; }
static int fm_active(void *u, int i) { (void) u; return FM[i].active; }
static int fm_save(void *u) { (void) u; fm_save_calls++; return fm_save_result; }
static int fm_restart_needed(void *u) { (void) u; return fm_restart; }
static int fm_locked(void *u) { (void) u; return fm_locked_now; }
static int fm_adds(void *u, int i, char out[][AT_STR], int cap)
{
    int k;
    (void) u;
    for (k = 0; k < FM[i].nadds && k < cap; k++) snprintf(out[k], AT_STR, "%s", FM[i].adds[k]);
    return k < FM[i].nadds ? k : FM[i].nadds;
}
static int fm_settings(void *u, int i, char *sid, int cap, char *label, int lcap)
{
    (void) u;
    if (!FM[i].settings[0]) return 0;
    snprintf(sid, (size_t) cap, "%s", FM[i].settings); snprintf(label, (size_t) lcap, "%s settings", FM[i].name);
    return 1;
}
static AtModsSrc src(void)
{
    AtModsSrc s;
    memset(&s, 0, sizeof s);
    s.count = fm_count; s.id = fm_id; s.name = fm_name; s.version = fm_version; s.kind = fm_kind; s.pack = fm_pack; s.desc = fm_desc;
    s.requires = fm_requires; s.conflicts = fm_conflicts; s.status = fm_status; s.status_text = fm_status_text; s.enabled = fm_enabled;
    s.active = fm_active; s.set_enabled = fm_set; s.save = fm_save; s.restart_needed = fm_restart_needed; s.adds = fm_adds; s.settings = fm_settings;
    s.locked = fm_locked;
    return s;
}
static void add(const char *id, const char *name, const char *kind, const char *req, const char *con, int status, int enabled, int active)
{
    FMod *m = &FM[fm_n++];
    memset(m, 0, sizeof *m);
    snprintf(m->id, sizeof m->id, "%s", id); snprintf(m->name, sizeof m->name, "%s", name); snprintf(m->kind, sizeof m->kind, "%s", kind);
    snprintf(m->req, sizeof m->req, "%s", req); snprintf(m->con, sizeof m->con, "%s", con);
    m->status = status; m->enabled = enabled; m->active = active;
    snprintf(m->desc, sizeof m->desc, "%s", "A mod.");
}
/* the eight-mod folder of the plan: a base, a fighter that needs it, a script with a Solo entry, two drive packs that conflict,
 * the LAB, a fighter whose base is not installed, and one that is simply off */
static void fixture(void)
{
    fm_n = 0; fm_save_calls = 0; fm_save_result = 0; fm_restart = 0; fm_locked_now = 0; fm_set_calls = 0;
    add("ace-base", "ACE Base", "base", "", "", AT_MOD_ACTIVE, 1, 1);
    add("ace-wolf", "ACE Wolf", "fighter", "ace-base", "", AT_MOD_ACTIVE, 1, 1);
    add("envoy", "Supertime Envoy", "script", "", "", AT_MOD_ACTIVE, 1, 1);
    snprintf(FM[2].adds[0], 64, "%s", "Solo > Envoy"); FM[2].nadds = 1; snprintf(FM[2].settings, sizeof FM[2].settings, "%s", "envoy.settings");
    add("envoy-drives", "Envoy Drives", "misc", "", "envoy-drives-sa2", AT_MOD_ACTIVE, 1, 1);
    add("envoy-drives-sa2", "Envoy Drives SA2", "misc", "", "envoy-drives", AT_MOD_CONFLICT, 1, 0);
    snprintf(FM[4].status_text, sizeof FM[4].status_text, "%s", "conflicts with envoy-drives");
    add("geno-lab", "Geno Lab", "script", "", "", AT_MOD_ACTIVE, 1, 1);
    snprintf(FM[5].adds[0], 64, "%s", "Solo > LAB"); FM[5].nadds = 1;
    add("orphan", "Orphan Fighter", "fighter", "missing-base", "", AT_MOD_MISSING_DEP, 1, 0);
    snprintf(FM[6].status_text, sizeof FM[6].status_text, "%s", "needs missing-base");
    add("sora", "Sora", "fighter", "", "", AT_MOD_OFF, 0, 0);
}
static void many(int n)
{
    int i;
    fm_n = 0; fm_save_calls = 0; fm_save_result = 0; fm_restart = 0; fm_locked_now = 0; fm_set_calls = 0;
    for (i = 0; i < n; i++) {
        char id[40], name[64];
        snprintf(id, sizeof id, "mod-%03d", i); snprintf(name, sizeof name, "Fighter Number %03d", i);
        add(id, name, "fighter", "", "", AT_MOD_ACTIVE, i % 3 != 0, i % 3 != 0);
    }
}

static AtModsState ST; static AtScreen SC; static AtView VW;
static AtEvent ev(int type, int a, int b) { AtEvent e; e.type = type; e.a = a; e.b = b; return e; }
static void build(const AtModsSrc *s) { CHECK(at_mods_build(s, &ST, &SC, &VW, 1000.0) == 1); }
static AtModsAction press(const AtModsSrc *s, AtEvent e) { AtModsAction a; at_mods_event(s, &ST, &e, 1000.0, &a); build(s); return a; }
static const char *focused_label(void) { return VW.focus.index >= 0 && VW.focus.index < SC.n_items ? SC.items[VW.focus.index].label : "(none)"; }

static void rows(void)
{
    AtModsSrc s = src();
    int i;
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    CHECK(SC.n_items == 8 && SC.primary == AT_PRIMARY_LIST && SC.chapter == 4 && strcmp(SC.title, "MODS") == 0);
    CHECK(SC.n_tabs == 2 && SC.tabs[0].count == 8 && SC.tabs[1].count == 2 && VW.tab == 0);
    CHECK(strcmp(SC.items[0].label, "ACE Base") == 0 && SC.items[0].vkind == AT_VAL_TOGGLE && SC.items[0].on == 1);
    CHECK(SC.items[7].on == 0 && !(SC.items[7].flags & AT_CELL_SELECTED));                      /* off: toggle off, not mounted */
    CHECK((SC.items[2].flags & AT_CELL_SELECTED) && strstr(SC.items[2].sub, "adds Solo > Envoy") != NULL);   /* mounted now: the jade bar */
    CHECK(strstr(SC.items[4].sub, "CONFLICT") != NULL && !(SC.items[4].flags & AT_CELL_SELECTED));            /* a word, not only a colour */
    CHECK(strstr(SC.items[6].sub, "needs missing-base") != NULL);
    CHECK(strstr(SC.items[0].sub, "Base") != NULL);
    for (i = 0; i < SC.n_items; i++) CHECK(strlen(SC.items[i].id) < AT_ID && SC.items[i].id[0] != '\0');
    CHECK(VW.focus.index == 0 && VW.scroll == 0);
    CHECK(strcmp(VW.counter, "1 / 8") == 0);
    CHECK(VW.note.text[0] == '\0');                                                             /* nothing to restart yet */
    /* the explainer follows the focus, and has no picture well */
    CHECK(VW.ex.has && strcmp(VW.ex.title, "ACE Base") == 0 && strstr(VW.ex.from_text, "ace-base") != NULL && VW.ex.no_well == 1);
    CHECK(VW.ex.media_model == AT_NO_MODEL && VW.ex.media_tex == -1);
    /* the keys: A names what it does to THIS mod, B backs out, Y asks for details */
    CHECK(strcmp(VW.key_label[0], "Turn off") == 0 && SC.keys[0].btn == 'A');
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
    CHECK(strcmp(focused_label(), "ACE Wolf") == 0 && strcmp(VW.counter, "2 / 8") == 0);
}

static void cascade(void)
{
    AtModsSrc s = src();
    AtModsAction a;
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    /* turning the base off turns ace-wolf off with it, and the note says so; the file is saved once */
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_MA_NONE && fm_save_calls == 1 && !FM[0].enabled && !FM[1].enabled);
    CHECK(strstr(VW.note.text, "ACE Wolf") != NULL && strstr(VW.note.text, "off") != NULL && VW.note.kind == AT_NOTE_OK);
    CHECK(SC.items[0].on == 0 && SC.items[1].on == 0);                                         /* the rows show the pending truth */
    CHECK(SC.items[0].flags & AT_CELL_SELECTED);                                               /* still mounted this boot */
    CHECK(strstr(SC.items[0].sub, "RESTART") != NULL && fm_restart == 1);
    /* the restart note is persistent and says what it is */
    CHECK(strcmp(VW.note.text, "Applies at restart") == 0 || strstr(VW.note.text, "ACE Wolf") != NULL);
    /* back on: the requirement comes with it */
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
    a = press(&s, ev(AT_EV_ACCEPT, 0, 0));                                                      /* ACE Wolf on: ace-base on too */
    CHECK(FM[1].enabled && FM[0].enabled && strstr(VW.note.text, "ACE Base") != NULL && strstr(VW.note.text, " on") != NULL);
    CHECK(strstr(SC.items[0].sub, "RESTART") == NULL && strstr(SC.items[1].sub, "RESTART") == NULL);   /* these two equal what is mounted again (the fixture's conflicting pack is on but not mounted, so the global flag stays set) */
    /* left and right toggle too (a toggle row flips on A, left or right) */
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0));
    CHECK(!FM[1].enabled);
    /* a failed save is said, and the row still shows the pending state */
    fm_save_result = -1; fm_save_calls = 0;
    press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0));
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(fm_save_calls == 1 && VW.note.kind == AT_NOTE_ERR && strstr(VW.note.text, "Could not save") != NULL);
    CHECK(SC.items[0].on == FM[0].enabled);
    /* turning a conflicting pack on turns the other off (either direction), and says which */
    fixture(); at_mods_state_init(&ST); build(&s);
    FM[4].status = AT_MOD_OFF; FM[4].enabled = 0; FM[3].enabled = 0; FM[3].status = AT_MOD_OFF; fm_recompute_restart();
    ST.sel[0] = 3; build(&s);
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(FM[3].enabled && !FM[4].enabled);
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(FM[4].enabled && !FM[3].enabled && strstr(VW.note.text, "Envoy Drives") != NULL);
}

static void conflicts(void)
{
    AtModsSrc s = src();
    AtModsAction a;
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    press(&s, ev(AT_EV_PAGE, 1, 0));                                                            /* R: the CONFLICTS tab */
    CHECK(ST.tab == 1 && VW.tab == 1 && SC.n_items == 2 && strcmp(SC.items[0].label, "Envoy Drives SA2") == 0 && strcmp(SC.items[1].label, "Orphan Fighter") == 0);
    CHECK(VW.ex.has && strstr(VW.ex.kicker, "CONFLICT") != NULL);
    CHECK(strcmp(VW.counter, "1 / 2") == 0);
    /* X resolves the conflict: the dropped mod goes off, the tab shrinks, the focus lands on what is left */
    a = press(&s, ev(AT_EV_ALT, 'X', 0));
    CHECK(a.kind == AT_MA_NONE && !FM[4].enabled && fm_save_calls == 1);
    CHECK(SC.n_items == 2);                                                                     /* the boot status is still CONFLICT until the restart */
    /* statuses are boot facts; after a restart the row leaves the tab: simulate it and check the selection clamps */
    FM[4].status = AT_MOD_OFF; FM[6].status = AT_MOD_OFF; build(&s);
    CHECK(SC.n_items == 1 && (SC.items[0].flags & AT_CELL_DISABLED) && ST.sel[1] == 0);        /* both gone: the empty row, the selection clamped */
    FM[4].status = AT_MOD_CONFLICT; FM[6].status = AT_MOD_MISSING_DEP; ST.sel[1] = 1; build(&s);
    CHECK(strcmp(focused_label(), "Orphan Fighter") == 0);
    /* a missing dependency that is not installed cannot be fixed: the note says which one */
    a = press(&s, ev(AT_EV_ALT, 'X', 0));
    CHECK(VW.note.kind == AT_NOTE_ERR && strstr(VW.note.text, "missing-base") != NULL && strstr(VW.note.text, "not installed") != NULL);
    /* one that is installed but off is turned on by name */
    add("needs-sora", "Needs Sora", "fighter", "sora", "", AT_MOD_MISSING_DEP, 1, 0);
    build(&s);
    ST.sel[1] = 2; build(&s);
    CHECK(strcmp(focused_label(), "Needs Sora") == 0);
    press(&s, ev(AT_EV_ALT, 'X', 0));
    CHECK(FM[7].enabled == 1 && strstr(VW.note.text, "Sora") != NULL && VW.note.kind == AT_NOTE_OK);
    /* an empty tab says so instead of showing nothing, and A on it does nothing */
    FM[4].status = AT_MOD_OFF; FM[6].status = AT_MOD_OFF; FM[8].status = AT_MOD_OFF; ST.sel[1] = 0; build(&s);
    CHECK(SC.n_items == 1 && (SC.items[0].flags & AT_CELL_DISABLED) && strstr(SC.items[0].label, "No conflicts") != NULL);
    fm_save_calls = 0; press(&s, ev(AT_EV_ACCEPT, 0, 0)); CHECK(fm_save_calls == 0);
    /* each tab keeps its own selection */
    press(&s, ev(AT_EV_PAGE, -1, 0));
    CHECK(ST.tab == 0);
    ST.sel[0] = 5; build(&s); press(&s, ev(AT_EV_PAGE, 1, 0)); press(&s, ev(AT_EV_PAGE, -1, 0));
    CHECK(strcmp(focused_label(), "Geno Lab") == 0);
}

/* the rows the renderer really shows: the focus row must have a hit rectangle at every width */
static int focus_on_screen(float w)
{
    static AtHits hits; AtRenderInfo info; AtSink sk = rec_sink();
    int i;
    at_render_ex(&SC, &VW, w, 1000.0, 0, &FAKE, &sk, &hits, &info);
    for (i = 0; i < hits.n; i++) if (hits.h[i].kind == AT_HIT_CELL && hits.h[i].b == VW.focus.index) return 1;
    return 0;
}

static void window_walk(void)
{
    AtModsSrc s = src();
    int k;
    many(256); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    CHECK(SC.n_items <= AT_MAX_ITEMS && SC.tabs[0].count == 256);
    for (k = 1; k <= 255; k++) {                                                                /* down the whole folder */
        char want[64];
        press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
        snprintf(want, sizeof want, "Fighter Number %03d", k);
        CHECK(strcmp(focused_label(), want) == 0);
        CHECK(SC.n_items <= AT_MAX_ITEMS && VW.focus.index >= VW.scroll && VW.focus.index < VW.scroll + AT_MODS_VISIBLE);   /* the focus is on screen */
        if (k % 11 == 0) { CHECK(focus_on_screen(640.0f)); CHECK(focus_on_screen(1140.0f)); }       /* ... and the renderer draws that row */
    }
    press(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));                                                  /* wraps to the top */
    CHECK(strcmp(focused_label(), "Fighter Number 000") == 0 && ST.sel[0] == 0);
    press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0));                                                    /* and back to the bottom */
    CHECK(strcmp(focused_label(), "Fighter Number 255") == 0);
    for (k = 254; k >= 0; k--) {
        char want[64];
        press(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0));
        snprintf(want, sizeof want, "Fighter Number %03d", k);
        CHECK(strcmp(focused_label(), want) == 0);
    }
    /* a mouse hover reports a row of the WINDOW: it must land on that mod, not on the same index of the whole list */
    ST.sel[0] = 200; build(&s);
    press(&s, ev(AT_EV_FOCUS, 0, VW.focus.index + 2));
    CHECK(ST.sel[0] == 202 && strcmp(focused_label(), "Fighter Number 202") == 0);
    /* the wheel scrolls */
    press(&s, ev(AT_EV_SCROLL, -1, 0)); CHECK(ST.sel[0] == 201);
    /* toggling in the middle of 256 touches the right mod */
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(FM[201].enabled == (201 % 3 != 0 ? 0 : 1));
    /* an empty folder: one disabled row and a hint, no crash, no action */
    fm_n = 0; at_mods_state_init(&ST); build(&s);
    CHECK(SC.n_items == 1 && (SC.items[0].flags & AT_CELL_DISABLED) && strstr(SC.items[0].label, "No mods") != NULL);
    press(&s, ev(AT_EV_ACCEPT, 0, 0)); press(&s, ev(AT_EV_ALT, 'Y', 0)); press(&s, ev(AT_EV_ALT, 'X', 0));
}

static void actions(void)
{
    AtModsSrc s = src();
    AtModsAction a;
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    a = press(&s, ev(AT_EV_BACK, 0, 0)); CHECK(a.kind == AT_MA_BACK);
    ST.sel[0] = 2; build(&s);
    a = press(&s, ev(AT_EV_ALT, 'Y', 0)); CHECK(a.kind == AT_MA_DETAIL && a.arg == 2);
    a = press(&s, ev(AT_EV_START, 0, 0)); CHECK(a.kind == AT_MA_NONE);
}

/* a session: mods are shown, never changed. Review Focus: changing mods must never be allowed while online. */
static void online_lock(void)
{
    AtModsSrc s = src();
    AtModsAction a;
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); fm_locked_now = 1; build(&s);
    CHECK(strcmp(VW.note.text, "Locked while online") == 0 && VW.note.kind == AT_NOTE_WARN);       /* a standing note, not only a refusal */
    CHECK((SC.items[0].iflags & AT_ITEM_RO) && SC.items[0].on == 1);                                 /* the value is shown, never changed */
    {   int k, has_a = 0, has_x = 0;
        for (k = 0; k < SC.n_keys; k++) { has_a |= SC.keys[k].btn == 'A'; has_x |= SC.keys[k].btn == 'X'; }
        CHECK(!has_a && !has_x);                                                                     /* no hint promises a change */
    }
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(fm_set_calls == 0 && fm_save_calls == 0 && FM[0].enabled && FM[1].enabled);
    CHECK(strstr(VW.note.text, "online") != NULL && VW.note.kind == AT_NOTE_WARN);
    press(&s, ev(AT_EV_MOVE, AT_DIR_RIGHT, 0)); press(&s, ev(AT_EV_MOVE, AT_DIR_LEFT, 0));
    CHECK(fm_set_calls == 0 && fm_save_calls == 0);
    press(&s, ev(AT_EV_PAGE, 1, 0));
    press(&s, ev(AT_EV_ALT, 'X', 0)); CHECK(fm_set_calls == 0 && fm_save_calls == 0);              /* a conflict cannot be resolved either */
    press(&s, ev(AT_EV_PAGE, -1, 0));
    ST.sel[0] = 4; build(&s);
    press(&s, ev(AT_EV_ALT, 'X', 0)); CHECK(fm_set_calls == 0 && FM[4].enabled == 1);
    a = press(&s, ev(AT_EV_ALT, 'Y', 0)); CHECK(a.kind == AT_MA_DETAIL && a.arg == 4);               /* looking is allowed */
    a = press(&s, ev(AT_EV_BACK, 0, 0)); CHECK(a.kind == AT_MA_BACK);
    /* the lock lifting returns the keys */
    fm_locked_now = 0; build(&s);
    CHECK(!(SC.items[0].iflags & AT_ITEM_RO) && SC.keys[0].btn == 'A');
    press(&s, ev(AT_EV_ACCEPT, 0, 0)); ST.sel[0] = 0; press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(fm_set_calls >= 1);
}

/* every row's texts must sit inside its row rectangle, at three widths, with the worst strings the folder can hold */
static void long_strings(void)
{
    AtModsSrc s = src();
    static const float widths[3] = { 640.0f, 853.0f, 1140.0f };
    int w, i, j;
    fixture();
    memset(FM[0].name, 'W', 63); FM[0].name[63] = '\0';
    memset(FM[2].adds[0], 'A', 63); FM[2].adds[0][63] = '\0';
    memset(FM[4].status_text, 's', 99); FM[4].status_text[99] = '\0';
    memset(FM[1].desc, 'd', 159); FM[1].desc[159] = '\0';
    for (w = 0; w < 3; w++) {
        static AtHits hits; AtRenderInfo info; AtSink sk;
        at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
        sk = rec_sink();
        at_render_ex(&SC, &VW, widths[w], 1000.0, 0, &FAKE, &sk, &hits, &info);
        CHECK(info.entries < 1500 && !info.capped);
        for (i = 0; i < hits.n; i++) {
            if (hits.h[i].kind != AT_HIT_CELL) continue;
            for (j = 0; j < REC.nt; j++) {
                float x0, x1, y0, y1;
                lint_text_box(&REC.t[j], &x0, &x1, &y0, &y1);
                if (REC.t[j].base < hits.h[i].r.y || REC.t[j].base > hits.h[i].r.y + hits.h[i].r.h) continue;
                if (x0 < hits.h[i].r.x - 0.5f) continue;                                         /* left of the row: the margin or the rail */
                if (x0 > hits.h[i].r.x + hits.h[i].r.w) continue;                                /* right of it: the explainer */
                CHECK(x1 <= hits.h[i].r.x + hits.h[i].r.w + 0.5f && at_role_size(REC.t[j].role) >= 12);
            }
        }
        press(&s, ev(AT_EV_PAGE, 1, 0)); press(&s, ev(AT_EV_PAGE, -1, 0));
    }
    /* the counter and the restart note against the key hints: the hints that do not fit are left off, never overlapped */
    FM[0].enabled = 0; fm_recompute_restart();
    build(&s);
    CHECK(VW.note.text[0] != '\0');
}

static void parents(void)
{
    char out[64];
    CHECK(strcmp(at_mods_parent_label("solo"), "Solo") == 0 && strcmp(at_mods_parent_label("versus"), "Versus") == 0);
    CHECK(strcmp(at_mods_parent_label("online"), "Online") == 0 && strcmp(at_mods_parent_label("lab.pause"), "LAB pause") == 0);
    CHECK(strcmp(at_mods_parent_label("settings.video"), "Settings > Video") == 0 && strcmp(at_mods_parent_label("pause"), "Pause") == 0);
    CHECK(strcmp(at_mods_parent_label("whatever"), "whatever") == 0);                           /* unknown: shown as given, never hidden */
    at_mods_add_line("solo", "Envoy", out, sizeof out);
    CHECK(strcmp(out, "Solo > Envoy") == 0);
}

static AtModsDetail DT;
static void dopen(const AtModsSrc *s, int mod) { memset(&DT, 0, sizeof DT); DT.mod = mod; CHECK(at_mods_detail_build(s, &DT, &SC, &VW, 1000.0) == 1); }
static AtModsAction dpress(const AtModsSrc *s, AtEvent e) { AtModsAction a; at_mods_detail_event(s, &DT, &e, &a); at_mods_detail_build(s, &DT, &SC, &VW, 1000.0); return a; }
static const char *sub_of(const char *label) { int i; for (i = 0; i < SC.n_items; i++) if (strcmp(SC.items[i].label, label) == 0) return SC.items[i].sub; return NULL; }

static void detail(void)
{
    AtModsSrc s = src();
    AtModsAction a;
    int i;
    fixture(); memset(&VW, 0, sizeof VW);
    /* a mod with an entry and its own settings screen */
    dopen(&s, 2);
    CHECK(strcmp(SC.id, "mods.detail") == 0 && strcmp(SC.title, "SUPERTIME ENVOY") == 0);
    CHECK(sub_of("Requires") != NULL && strcmp(sub_of("Requires"), "Nothing") == 0);
    CHECK(sub_of("Conflicts") != NULL && strcmp(sub_of("Conflicts"), "Nothing") == 0);
    CHECK(sub_of("Adds to the menus") != NULL && strcmp(sub_of("Adds to the menus"), "Solo > Envoy") == 0);
    CHECK(sub_of("Supertime Envoy settings") != NULL);                                          /* the mod's own settings entry */
    CHECK(VW.ex.has && strstr(VW.ex.what, "A mod.") != NULL && VW.ex.no_well == 1);
    for (i = 0; i < SC.n_items; i++) CHECK(SC.items[i].vkind == AT_VAL_NONE);                   /* read-only rows: nothing to flip here */
    /* A on a plain row does nothing; A on the settings row hands off to the entry */
    a = dpress(&s, ev(AT_EV_ACCEPT, 0, 0)); CHECK(a.kind == AT_MA_NONE);
    for (i = 0; i < SC.n_items; i++) if (strstr(SC.items[i].label, "settings") != NULL) DT.sel = i;
    a = dpress(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(a.kind == AT_MA_SETTINGS && strcmp(a.id, "envoy.settings") == 0);
    CHECK(SC.keys[0].btn == 'A');                                                               /* the A hint appears on the row that has an action */
    a = dpress(&s, ev(AT_EV_BACK, 0, 0)); CHECK(a.kind == AT_MA_BACK);
    /* a conflict is listed from both sides, by name */
    dopen(&s, 3);
    CHECK(strstr(sub_of("Conflicts"), "Envoy Drives SA2") != NULL);
    dopen(&s, 4);
    CHECK(strcmp(sub_of("Conflicts"), "Envoy Drives") == 0);                               /* its own list names the other pack; the reverse scan finds the same one once */
    /* requirements by name where the mod is known, by id where it is not */
    dopen(&s, 6);
    CHECK(strstr(sub_of("Requires"), "missing-base") != NULL);
    dopen(&s, 1);
    CHECK(strstr(sub_of("Requires"), "ACE Base") != NULL);
    /* nothing to add, no settings entry: only the plain rows, and A and B are safe */
    dopen(&s, 7);
    CHECK(sub_of("Adds to the menus") == NULL && SC.n_items >= 3);
    a = dpress(&s, ev(AT_EV_ACCEPT, 0, 0)); CHECK(a.kind == AT_MA_NONE);
    a = dpress(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0)); a = dpress(&s, ev(AT_EV_MOVE, AT_DIR_UP, 0)); CHECK(a.kind == AT_MA_NONE);
    /* the six-entry cap: six adds fit the record with everything else */
    fixture(); FM[2].nadds = 3; snprintf(FM[2].adds[1], 64, "%s", "Versus > Envoy Online"); snprintf(FM[2].adds[2], 64, "%s", "Online > Envoy Room");
    dopen(&s, 2); CHECK(SC.n_items <= AT_MAX_ITEMS);
    /* ten rows (folder, requires, conflicts, six adds, settings) do not all fit: the focus is always in the rows shown, down to the last */
    fixture(); FM[2].nadds = 3; snprintf(FM[2].adds[1], 64, "%s", "Versus > Envoy Online"); snprintf(FM[2].adds[2], 64, "%s", "Online > Envoy Room");
    dopen(&s, 2);
    for (i = 0; i < SC.n_items + 2; i++) {
        static AtHits hits; AtRenderInfo info; AtSink sk = rec_sink(); int k, on = 0;
        CHECK(VW.focus.index >= VW.scroll && VW.focus.index < VW.scroll + AT_MODS_DETAIL_VISIBLE);
        at_render_ex(&SC, &VW, 640.0f, 1000.0, 0, &FAKE, &sk, &hits, &info);
        for (k = 0; k < hits.n; k++) if (hits.h[k].kind == AT_HIT_CELL && hits.h[k].b == VW.focus.index) on = 1;
        CHECK(on);
        dpress(&s, ev(AT_EV_MOVE, AT_DIR_DOWN, 0));
    }
    /* a long list and long names are cut by the fit rule inside their rows */
    fixture(); memset(FM[3].con, 'c', 63); FM[3].con[63] = '\0';
    dopen(&s, 3);
    {
        static AtHits hits; AtRenderInfo info; AtSink sk = rec_sink();
        at_render_ex(&SC, &VW, 640.0f, 1000.0, 0, &FAKE, &sk, &hits, &info);
        CHECK(info.entries < 1500 && !info.capped && hits.n >= 3);
    }
}

/* review follow-ups: ids match without regard to case (like gw_Mods_Find), whole-word names, a failed save rolls back, mixed cascades, no restart note when back */
static void review_followups(void)
{
    AtModsSrc s = src();
    int i;
    char before[FM_MAX];
    /* 1. a requirement spelled in another case is the same mod */
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW);
    snprintf(FM[6].req, sizeof FM[6].req, "%s", "MISSING-BASE");
    add("needs-sora", "Needs Sora", "fighter", "SORA", "", AT_MOD_MISSING_DEP, 1, 0);
    build(&s); press(&s, ev(AT_EV_PAGE, 1, 0));
    ST.sel[1] = 2; build(&s);
    CHECK(strcmp(focused_label(), "Needs Sora") == 0);
    press(&s, ev(AT_EV_ALT, 'X', 0));
    CHECK(FM[7].enabled == 1 && strstr(VW.note.text, "not installed") == NULL);                 /* SORA found sora */
    dopen(&s, 8);
    CHECK(strstr(sub_of("Requires"), "Sora") != NULL);                                           /* named, not shown as the raw id */
    /* 2. a reverse conflict is added by whole name, never skipped because its name is inside another */
    fixture();
    add("envoy-lite", "Envoy", "misc", "", "", AT_MOD_ACTIVE, 1, 1);
    snprintf(FM[3].con, sizeof FM[3].con, "%s", "envoy-lite,envoy-drives-sa2");              /* mod 3 lists the lite one (Envoy) and sa2 */
    snprintf(FM[8].con, sizeof FM[8].con, "%s", "envoy-drives");                               /* the lite one lists mod 3 too: reverse side */
    dopen(&s, 8);
    CHECK(strstr(sub_of("Conflicts"), "Envoy Drives") != NULL);
    fixture();
    add("envoy-lite", "Envoy", "misc", "", "", AT_MOD_ACTIVE, 1, 1);
    snprintf(FM[3].con, sizeof FM[3].con, "%s", "envoy-drives-sa2");                           /* mod 3 ("Envoy Drives") lists SA2 ... */
    snprintf(FM[4].con, sizeof FM[4].con, "%s", "envoy-drives");
    snprintf(FM[8].con, sizeof FM[8].con, "%s", "envoy-drives");                               /* ... and "Envoy" lists mod 3: the name "Envoy" is inside "Envoy Drives" */
    dopen(&s, 3);
    CHECK(strstr(sub_of("Conflicts"), "Envoy Drives SA2") != NULL && strstr(sub_of("Conflicts"), ", Envoy") != NULL);   /* "Envoy" is added: whole words, not substrings */
    /* 3. a failed save rolls the toggle back: the screen matches the disk */
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    for (i = 0; i < fm_n; i++) before[i] = (char) FM[i].enabled;
    fm_save_result = -1;
    press(&s, ev(AT_EV_ACCEPT, 0, 0));                                                         /* ACE Base off would have turned ACE Wolf off too */
    for (i = 0; i < fm_n; i++) CHECK((char) FM[i].enabled == before[i]);
    CHECK(VW.note.kind == AT_NOTE_ERR && strstr(VW.note.text, "Could not save") != NULL && strstr(VW.note.text, "Nothing was changed") != NULL);
    CHECK(SC.items[0].on == 1 && SC.items[1].on == 1);
    fm_save_result = 0;
    /* 4. a mixed cascade says both: a requirement on, a conflicting mod off */
    fixture(); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW);
    add("mixer", "Mixer", "misc", "sora", "envoy", AT_MOD_OFF, 0, 0);
    build(&s); ST.sel[0] = 8; build(&s);
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(FM[8].enabled && FM[7].enabled && !FM[2].enabled);
    CHECK(strstr(VW.note.text, "turned on: Sora") != NULL && strstr(VW.note.text, "turned off: Supertime Envoy") != NULL);
    /* 5. back to what is mounted: no "applies at restart" in the note, and none standing */
    many(5); at_mods_state_init(&ST); memset(&VW, 0, sizeof VW); build(&s);
    ST.sel[0] = 1; build(&s);
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(FM[1].enabled == 0 && strstr(VW.note.text, "Applies at restart") != NULL);
    press(&s, ev(AT_EV_ACCEPT, 0, 0));
    CHECK(FM[1].enabled == 1 && strstr(VW.note.text, "restart") == NULL);
}

int main(void)
{
    rows();
    cascade();
    conflicts();
    window_walk();
    actions();
    online_lock();
    long_strings();
    parents();
    detail();
    review_followups();
    ATLAS_DONE("atlas mods");
}
