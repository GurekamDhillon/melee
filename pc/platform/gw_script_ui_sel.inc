/* gw_script_ui_sel.inc - the host half of the character select, the stage select and the loading screen (Atlas step 4, Task 9). Included at the end
 * of gw_script_ui.inc: it uses that file's slots, stack, view and mouse state.
 *
 * The game side cannot hand the host a function pointer or a struct: the adapter (src/melee/gm/gmfrontend_atlas_select.inc) submits SCALARS AND C
 * STRINGS through the Ui_Sel* shims below (game code calls them unprefixed). They fill a screen record IN PLACE in one engine-owned slot:
 * its cells live in gs_sel.pool (AtBlock.ext), so the record is never copied (at_screen_copy refuses it) and nothing outlives the pool.
 * The slot's owner is GS_UI_ENGINE, which every Lua door already refuses to a script and the console; this file adds the one thing a select has
 * that a menu does not: it is a handle, (generation << 8) | slot, so a handle that outlived its screen is recognised and ignored.
 *
 * Events go the other way through a small ring the adapter drains with Ui_SelPoll: the host reads the mouse over the rendered hit rectangles itself
 * (gs_sel_tick), because a select needs to know a tab from a card from a tile, and which button, which the generic menu events do not carry. The pad
 * is the game's: it is read by the adapter, never here. Written to game memory, the poll's out-parameters are big-endian (gs_ui_put_be32). */

#include "gw_ui_css.h"
#include "gw_ui_sss.h"

static int gs_set_is_open(void);   /* gw_script_ui_set.inc: the settings screens are native too, and only one native screen exists */

static struct {
    int gen;                 /* the handle generation: bumped per open */
    int h;                   /* the open handle, or -1 */
    AtCell pool[AT_MAX_EXT_CELLS];
    struct { int kind, a, b; } q[32];
    int qn;
    char spec[96];           /* the key hints last parsed (re-parsed only on a change) */
} gs_sel = { 0, -1 };

enum { GS_SEL_CSS, GS_SEL_SSS, GS_SEL_LOAD };
enum { GS_SELEV_CELL = 1, GS_SELEV_CARD = 2, GS_SELEV_TAB = 3, GS_SELEV_KEY = 4, GS_SELEV_WHEEL = 5, GS_SELEV_POINTER = 6 };

/* the slot a handle names, or NULL when the handle is stale (the screen closed, the scene ended, another select opened since) */
static GsUiSlot *gs_sel_get(int h) {
    int slot = h & 0xFF;
    GsUiSlot *u;
    if (h < 0 || h != gs_sel.h || slot >= GS_UI_SLOTS) return NULL;
    u = &gs_ui_slot[slot];
    return (u->used && u->native_sel) ? u : NULL;
}

static void gs_sel_q_push(int kind, int a, int b) {
    if (gs_sel.qn >= 32) return;                                   /* the adapter drains it every frame; a full ring drops the newest */
    gs_sel.q[gs_sel.qn].kind = kind; gs_sel.q[gs_sel.qn].a = a; gs_sel.q[gs_sel.qn].b = b;
    gs_sel.qn++;
}

/* 1 when gw_Ui_SelOpen would succeed now: Atlas is on, no select is open, a slot is free. The game side asks BEFORE it routes a mode's state into the frontend
 * scene, so a refusal leaves the retail native screen (not the legacy kit's). */
int gw_Ui_SelCanOpen(void) {
    int i;
    if (!gw_Ui_Ready()) return 0;
    if (gs_sel.h >= 0 && gs_sel_get(gs_sel.h) != NULL) return 0;
    if (gs_set_is_open() || gs_room_is_open()) return 0;           /* the settings and the online room are native too: one native screen */
    for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) return 1;
    return 0;
}

int gw_Ui_SelOpen(int kind, int match_type, const char *t0, const char *t1, const char *title) {
    int i, slot = -1;
    GsUiSlot *u;
    AtScreen *sc;
    (void) match_type;
    if (!gw_Ui_Ready() || kind < GS_SEL_CSS || kind > GS_SEL_LOAD) return -1;            /* roles missing or MELEE_ATLAS=0: the caller keeps the legacy drawing */
    if (gs_sel.h >= 0 && gs_sel_get(gs_sel.h) != NULL) return -1;                          /* one select at a time: the stack has one top */
    if (gs_set_is_open() || gs_room_is_open()) return -1;                                                        /* and not over the settings, which are native too */
    for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) { slot = i; break; }
    if (slot < 0) { gw_log("ui: no free screen slot for the select screen"); return -1; }
    u = &gs_ui_slot[slot];
    memset(u, 0, sizeof *u);                                       /* never reused state */
    at_view_init(&u->view);
    u->dialog_fn = -1;
    u->used = 1;
    u->owner = GS_UI_ENGINE;
    u->native_sel = 1;
    u->scene = gs.scene_kind;
    sc = &u->sc;
    sc->fn_provide = sc->fn_accept = sc->fn_back = sc->fn_focus = sc->fn_change = sc->fn_open = sc->fn_close = sc->fn_counter = sc->fn_page = sc->fn_start = -1;
    sc->fn_alt[0] = sc->fn_alt[1] = sc->fn_alt[2] = -1;
    snprintf(sc->id, sizeof sc->id, "%s", kind == GS_SEL_CSS ? "select.css" : kind == GS_SEL_SSS ? "select.sss" : "select.load");
    snprintf(sc->title, sizeof sc->title, "%s", title != NULL ? title : sc->id);
    if (t0 != NULL && t0[0] != '\0') snprintf(sc->parent[sc->n_parents++], AT_STR, "%s", t0);
    if (t1 != NULL && t1[0] != '\0') snprintf(sc->parent[sc->n_parents++], AT_STR, "%s", t1);
    sc->chapter = (t0 != NULL && strcmp(t0, "SOLO") == 0) ? 1 : 2;
    sc->primary = AT_PRIMARY_GRID;
    sc->preset = kind == GS_SEL_CSS ? AT_PRESET_NORMAL : AT_PRESET_NONE;
    sc->port = 1;
    sc->input_feed = 1;                                            /* the game feeds the pads (the adapter reads them per port) */
    sc->grid_cols_auto = 1;
    if (kind == GS_SEL_SSS) { sc->grid_cell_min = 54; sc->grid_cell_max = 72; }
    sc->band = kind == GS_SEL_CSS ? AT_BAND_CARDS : kind == GS_SEL_LOAD ? AT_BAND_MATCHUP : AT_BAND_NONE;
    sc->n_blocks = 1;
    snprintf(sc->blocks[0].id, AT_ID, "%s", kind == GS_SEL_SSS ? "stages" : "fighters");
    memset(gs_sel.pool, 0, sizeof gs_sel.pool);
    sc->blocks[0].ext = gs_sel.pool;
    sc->blocks[0].ext_n = 0;
    for (i = 0; i < 4; i++) { sc->ports[i].port = i; sc->ports[i].ck_tex = -1; }
    gs_sel.qn = 0;
    gs_sel.spec[0] = '\0';
    if (!at_stack_push(&gs_ui_stack, slot)) { memset(u, 0, sizeof *u); u->dialog_fn = -1; return -1; }
    u->view.opened_ms = gs_now_ms();
    memset(&u->mouse, 0, sizeof u->mouse);                         /* the first sample only primes: a button held from the menu does not click here */
    gs_sel.gen = (gs_sel.gen + 1) & 0xFFFFF;
    gs_sel.h = (gs_sel.gen << 8) | slot;
    gw_log("ui: select screen %s open (native)", sc->id);
    return gs_sel.h;
}

void gw_Ui_SelClose(int h) {
    GsUiSlot *u = gs_sel_get(h);
    int slot;
    if (u == NULL) return;
    slot = (int) (u - gs_ui_slot);
    gs_sel.h = -1;
    gs_sel.qn = 0;
    gs_ui_free_engine_slot(slot);
    gs_ui_after_close();
}

void gw_Ui_SelTab(int h, int i, const char *name, int count, int active) {
    GsUiSlot *u = gs_sel_get(h);
    if (u == NULL || i < 0 || i >= AT_MAX_TABS) return;
    if (i == 0) u->sc.n_tabs = 0;                                  /* tab 0 starts a new list */
    snprintf(u->sc.tabs[i].name, sizeof u->sc.tabs[i].name, "%s", name != NULL ? name : "");
    u->sc.tabs[i].count = count;
    if (i >= u->sc.n_tabs) u->sc.n_tabs = i + 1;
    if (active) u->view.tab = i;
}

void gw_Ui_SelBeginCells(int h, int n) {
    GsUiSlot *u = gs_sel_get(h);
    if (u == NULL) return;
    u->sc.blocks[0].ext_n = n < 0 ? 0 : (n > AT_MAX_EXT_CELLS ? AT_MAX_EXT_CELLS : n);
}

void gw_Ui_SelCell(int h, int i, const char *id, const char *name, const char *abbr, int tex, unsigned flags, char origin) {
    GsUiSlot *u = gs_sel_get(h);
    AtCell *c;
    if (u == NULL || i < 0 || i >= u->sc.blocks[0].ext_n) return;
    c = &gs_sel.pool[i];
    memset(c, 0, sizeof *c);
    snprintf(c->id, sizeof c->id, "%s", id != NULL ? id : "");
    snprintf(c->name, sizeof c->name, "%s", name != NULL ? name : "");
    snprintf(c->abbr, sizeof c->abbr, "%s", abbr != NULL ? abbr : "");
    c->model = c->ring = AT_NO_MODEL;
    c->tex = tex < 0 ? -1 : tex;
    c->flags = flags;
    c->origin = origin == 'G' || origin == '+' ? origin : 0;
}

void gw_Ui_SelCard(int h, int port, int kind, const char *name, const char *sub, const char *abbr, int tex, int cpu_lv, int team, unsigned flags) {
    GsUiSlot *u = gs_sel_get(h);
    AtSelCard *k;
    if (u == NULL || port < 0 || port > 3) return;
    k = &u->sc.ports[port];
    k->port = port;
    k->kind = kind < 0 || kind > 2 ? 0 : kind;
    k->ck_tex = tex < 0 ? -1 : tex;
    snprintf(k->name, sizeof k->name, "%s", name != NULL ? name : "");
    snprintf(k->sub, sizeof k->sub, "%s", sub != NULL ? sub : "");
    snprintf(k->abbr, sizeof k->abbr, "%s", abbr != NULL ? abbr : "");
    k->cpu_lv = cpu_lv;
    k->team = team;
    k->flags = flags;
}

void gw_Ui_SelCursor(int h, int port, int active, int cell, int card) {
    GsUiSlot *u = gs_sel_get(h);
    if (u == NULL || port < 0 || port >= AT_MAX_CURSORS) return;
    u->view.cursor[port].active = active != 0;
    u->view.cursor[port].block = 0;
    u->view.cursor[port].index = cell;
    u->view.cursor[port].card = card < 0 || card > 3 ? -1 : card;
}

void gw_Ui_SelExplain(int h, const char *kicker, const char *title, const char *what, const char *abbr, int tex,
                      const char *stepper_label, const char *stepper_text, const char *from) {
    GsUiSlot *u = gs_sel_get(h);
    AtExplainer *e;
    if (u == NULL) return;
    e = &u->view.ex;
    memset(e, 0, sizeof *e);
    e->media_model = e->media_ring = AT_NO_MODEL;
    e->has = (title != NULL && title[0] != '\0') || (kicker != NULL && kicker[0] != '\0');
    snprintf(e->kicker, sizeof e->kicker, "%s", kicker != NULL ? kicker : "");
    snprintf(e->title, sizeof e->title, "%s", title != NULL ? title : "");
    snprintf(e->what, sizeof e->what, "%s", what != NULL ? what : "");
    snprintf(e->media_abbr, sizeof e->media_abbr, "%s", abbr != NULL ? abbr : "");
    e->media_tex = tex < 0 ? -1 : tex;
    snprintf(e->stepper_label, sizeof e->stepper_label, "%s", stepper_label != NULL ? stepper_label : "");
    snprintf(e->stepper_text, sizeof e->stepper_text, "%s", stepper_text != NULL ? stepper_text : "");
    e->stepper = e->stepper_label[0] != '\0';
    snprintf(e->from_text, sizeof e->from_text, "%s", from != NULL ? from : "");
}

/* "A:Pick,B:Back,X:Costume,S:Fight": a button (A B X Y Z L R, S = START), a colon, its label; parsed only when the text changed */
void gw_Ui_SelKeys(int h, const char *spec) {
    GsUiSlot *u = gs_sel_get(h);
    const char *p;
    int n = 0, i;
    if (u == NULL || spec == NULL) return;
    if (strcmp(spec, gs_sel.spec) == 0) return;
    snprintf(gs_sel.spec, sizeof gs_sel.spec, "%s", spec);
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

void gw_Ui_SelCounter(int h, const char *text) {
    GsUiSlot *u = gs_sel_get(h);
    if (u != NULL) snprintf(u->view.counter, sizeof u->view.counter, "%s", text != NULL ? text : "");
}

void gw_Ui_SelNote(int h, const char *text, int kind) {
    GsUiSlot *u = gs_sel_get(h);
    double now;
    if (u == NULL || text == NULL || text[0] == '\0') return;
    if (strcmp(u->view.note.text, text) == 0 && gs_now_ms() < u->view.note.until_ms) return;   /* the same toast, still showing: not restarted */
    now = gs_now_ms();
    snprintf(u->view.note.text, sizeof u->view.note.text, "%s", text);
    u->view.note.kind = kind < AT_NOTE_OK || kind > AT_NOTE_INFO ? AT_NOTE_INFO : kind;
    u->view.note.from_ms = now;
    u->view.note.until_ms = now + 2500.0;                          /* the legacy toast: 150 frames */
}

void gw_Ui_SelProgress(int h, int permille) {
    GsUiSlot *u = gs_sel_get(h);
    if (u != NULL) u->view.progress = permille < 0 ? 0 : (permille > 1000 ? 1000 : permille);
}

/* the columns the layout chose for the grid pane (the model needs them to move on the grid) */
int gw_Ui_SelCols(int h) {
    GsUiSlot *u = gs_sel_get(h);
    AtLayout L;
    AtSplit sp;
    float cmin, cmax;
    if (u == NULL) return 8;
    at_layout(gs_ui_stat.canvas_w > 0.0f ? gs_ui_stat.canvas_w : 640.0f, u->sc.preset, &L);
    at_layout_split(&L, u->sc.n_tabs > 0, u->sc.band, &sp);
    cmin = u->sc.grid_cell_min > 0 ? (float) u->sc.grid_cell_min : 36.0f;
    cmax = u->sc.grid_cell_max > 0 ? (float) u->sc.grid_cell_max : 56.0f;
    if (cmax < cmin) cmax = cmin;
    return at_grid_cols(sp.grid.w - 24.0f, cmin, cmax, 8.0f);
}

int gw_Ui_SelPoll(int h, int *kind, int *a, int *b) {
    int i;
    if (gs_sel_get(h) == NULL || gs_sel.qn <= 0) return 0;
    gs_ui_put_be32(kind, gs_sel.q[0].kind); gs_ui_put_be32(a, gs_sel.q[0].a); gs_ui_put_be32(b, gs_sel.q[0].b);
    for (i = 1; i < gs_sel.qn; i++) gs_sel.q[i - 1] = gs_sel.q[i];
    gs_sel.qn--;
    return 1;
}

/* disc art: a GX texture in game memory, decoded now into the kit's pool (nothing is kept of the caller's pointers) */
int gw_Ui_ArtDecode(const char *key, int gx_fmt, const void *img, int size, int w, int h, int tlut_fmt, const void *tlut, int tlut_n) {
    return gw_Kit_TexAddHsd(key, gx_fmt, (const uint8_t *) img, size < 0 ? 0u : (size_t) size, w, h, tlut_fmt, (const uint8_t *) tlut, tlut_n);
}

/* the disc art is dropped when the roster it was decoded for changed (a mod toggled): the key names a fighter or a stage, not a disc, and the disc is fixed for the process */
void gw_Ui_ArtDrop(void) { gw_Kit_TexDropHsd(); }

/* ---- the mouse over a select screen: the same priming rules as at_mouse_events (the first sample only primes; off the picture is never an event;
 * a button held from the previous scene does not click), but the events keep what they hit: a tile, a card, a tab, a key hint, the wheel ---- */
static void gs_sel_tick(GsUiSlot *u) {
    float x, y, wheel;
    int buttons, hit, moved, left, right, kind = 0, idx = -1;
    AtMouse *m = &u->mouse;
    gw_Mouse_ScriptRead(&x, &y, &buttons, &wheel);
    if (x < 0.0f || y < 0.0f) { m->valid = 0; m->buttons = buttons; return; }
    moved = m->valid && (x != m->x || y != m->y);
    left = (buttons & 1) && !(m->buttons & 1) && m->valid;
    right = (buttons & 2) && !(m->buttons & 2) && m->valid;
    hit = at_hit_test(&u->hits, x, y);
    if (hit >= 0) { kind = u->hits.h[hit].kind; idx = kind == AT_HIT_CELL ? u->hits.h[hit].b : u->hits.h[hit].a; }
    if (kind == AT_HIT_CELL && (moved || left || right)) gs_sel_q_push(GS_SELEV_CELL, idx, left ? 1 : right ? 2 : 0);
    else if (kind == AT_HIT_CARD && (moved || left || right)) gs_sel_q_push(GS_SELEV_CARD, idx, left ? 1 : right ? 2 : 0);
    else if (kind == AT_HIT_TAB && left) gs_sel_q_push(GS_SELEV_TAB, idx, 0);
    else if (kind == AT_HIT_KEY && left) gs_sel_q_push(GS_SELEV_KEY, idx, 0);
    else if (right) gs_sel_q_push(GS_SELEV_CELL, -1, 2);           /* a right click on nothing: still B */
    else if (moved) gs_sel_q_push(GS_SELEV_POINTER, 0, 0);
    if ((int) wheel != 0) gs_sel_q_push(GS_SELEV_WHEEL, wheel > 0 ? 1 : -1, kind == AT_HIT_CARD ? idx : -1);
    m->valid = 1;
    m->x = x;
    m->y = y;
    m->buttons = buttons;
}

/* ---- the models, host side ------------------------------------------------------------------------------------------------
 * The rules (gw_ui_css.c, gw_ui_sss.c) are native C structs; game code is retargeted (its memory is big-endian), so it can neither hold an AtCss nor
 * hand one over. The model lives HERE, one instance of each, and the game-side adapter drives it through scalar shims: it pushes in what only the
 * game knows (the roster, each fighter's costume count, the Zelda and Sheik ids, whether the opponent has Sheik, the ports' starting state) and
 * reads back what the model decided. The only strings that cross are NUL-terminated and written byte by byte (no endianness in a byte). */
static AtCss gs_selm_css;
static AtSss gs_selm_sss;
static int gs_selm_cost[256];          /* costumes per fighter id, pushed by the adapter (0 = unknown: one) */
static int gs_selm_zelda = -1, gs_selm_sheik = -1, gs_selm_sheik_ok = 1;
static int gs_selm_ck[AT_CSS_MAX_SLOTS];
static unsigned char gs_selm_ok[AT_CSS_MAX_SLOTS], gs_selm_tab[AT_CSS_MAX_SLOTS];
static int gs_selm_n;
static AtSssStage gs_selm_st[AT_SSS_MAX + 8];
static int gs_selm_nst;

static int gs_selm_costumes(void *u, int ck) { (void) u; return ck >= 0 && ck < 256 && gs_selm_cost[ck] > 0 ? gs_selm_cost[ck] : 1; }
static int gs_selm_random(void *u, const struct AtCss *c) { (void) u; return c->n_slots > 1 ? c->ck[0] : 0; }
static int gs_selm_swap(void *u, int ck) { (void) u; return ck >= 0 && ck == gs_selm_zelda ? gs_selm_sheik : ck >= 0 && ck == gs_selm_sheik ? gs_selm_zelda : -1; }
static int gs_selm_sheik_ok_op(void *u) { (void) u; return gs_selm_sheik_ok; }
static const AtCssOps gs_selm_ops = { 0, gs_selm_costumes, gs_selm_random, gs_selm_swap, gs_selm_sheik_ok_op };

/* 1 when the match type has a profile (the adapter asks before it routes a mode's state into the frontend scene) */
int gw_Ui_CssKnown(int match_type) { return at_css_profile(match_type) != NULL; }

/* 1 when the match type has a profile and the model is open; 0 when it has none (the caller keeps the retail screen: never guess) */
int gw_Ui_CssOpen(int match_type, int online, int teams) {
    const AtCssProfile *p = at_css_profile(match_type);
    if (p == NULL) return 0;
    memset(gs_selm_cost, 0, sizeof gs_selm_cost);
    gs_selm_zelda = gs_selm_sheik = -1;
    gs_selm_sheik_ok = 1;
    at_css_open(&gs_selm_css, p, &gs_selm_ops, online, teams);
    gs_selm_n = 0;
    return 1;
}
void gw_Ui_CssPair(int zelda_ck, int sheik_ck) { gs_selm_zelda = zelda_ck; gs_selm_sheik = sheik_ck; }
void gw_Ui_CssSheikOk(int ok) { gs_selm_sheik_ok = ok != 0; }
void gw_Ui_CssCostumes(int ck, int n) { if (ck >= 0 && ck < 256) gs_selm_cost[ck] = n; }
void gw_Ui_CssRosterBegin(void) { gs_selm_n = 0; }
void gw_Ui_CssRosterAdd(int ck, int ok, int tab_of) {
    if (gs_selm_n >= AT_CSS_MAX_SLOTS - 1) return;
    gs_selm_ck[gs_selm_n] = ck; gs_selm_ok[gs_selm_n] = ok != 0; gs_selm_tab[gs_selm_n] = (unsigned char) tab_of;
    gs_selm_n++;
}
void gw_Ui_CssRosterEnd(void) { at_css_set_roster(&gs_selm_css, gs_selm_n, gs_selm_ck, gs_selm_ok, gs_selm_tab); }
void gw_Ui_CssPort(int p, int kind, int ck, int costume, int cpu_lv, int team) {
    AtCssPort *s;
    if (p < 0 || p > 3) return;
    s = &gs_selm_css.p[p];
    s->kind = kind < 0 || kind > 2 ? 0 : kind;
    s->ck = ck < AT_CK_RANDOM ? AT_CK_NONE : ck;
    s->costume = costume < 0 ? 0 : costume;
    s->cpu_lv = cpu_lv < 1 || cpu_lv > 9 ? 9 : cpu_lv;
    s->team = team < 0 || team > 2 ? 0 : team;
    s->card = -1;
    s->target = p;
    s->cur = s->ck >= 0 ? at_css_vis_of_ck(&gs_selm_css, s->ck) : 0;
}
void gw_Ui_CssSetPortCk(int p, int ck, int costume) {
    if (p < 0 || p > 3) return;
    gs_selm_css.p[p].ck = ck < AT_CK_RANDOM ? AT_CK_NONE : ck;
    gs_selm_css.p[p].costume = costume < 0 ? 0 : costume;
}
void gw_Ui_CssEntering(int port) { at_css_set_entering(&gs_selm_css, port); }
void gw_Ui_CssTraining(int human) { at_css_set_training(&gs_selm_css, human); }
void gw_Ui_CssCols(int cols) { at_css_set_cols(&gs_selm_css, cols); }
void gw_Ui_CssSetTab(int tab) { at_css_set_tab(&gs_selm_css, tab); }
void gw_Ui_CssSetCursor(int port, int vis) {
    if (port < 0 || port > 3) return;
    gs_selm_css.p[port].cur = vis >= 0 && vis < gs_selm_css.n_vis ? vis : 0;
    gs_selm_css.p[port].card = -1;
}
unsigned gw_Ui_CssStep(int port, unsigned trig, unsigned rep, int frame) {
    AtCssIn in;
    in.trig = trig; in.rep = rep;
    return at_css_step(&gs_selm_css, port, in, frame);
}
unsigned gw_Ui_CssMouse(int port, int vis, int card, int moved, int click, int rclick, int wheel) {
    return at_css_mouse_bits(&gs_selm_css, port, vis, card, moved, click, rclick, wheel);
}
enum { GS_CSSG_KIND, GS_CSSG_CK, GS_CSSG_COSTUME, GS_CSSG_CPU_LV, GS_CSSG_TEAM, GS_CSSG_CUR, GS_CSSG_CARD, GS_CSSG_TARGET,
       GS_CSSG_DONE = 10, GS_CSSG_N_VIS, GS_CSSG_TAB, GS_CSSG_COUNT_IN, GS_CSSG_TRAIN_H, GS_CSSG_TRAIN_C, GS_CSSG_N_SLOTS, GS_CSSG_ONLINE, GS_CSSG_TEAMS,
       GS_CSSG_TAB_OFFERED = 20, GS_CSSG_TAB_COUNT, GS_CSSG_VIS_SLOT, GS_CSSG_SLOT_CK, GS_CSSG_SLOT_OK, GS_CSSG_SLOT_TAB, GS_CSSG_VIS_OF_CK, GS_CSSG_MOUSE_PORT,
       GS_CSSG_MAX_HUMANS = 30, GS_CSSG_CPU_CARDS, GS_CSSG_MIN_TO_START, GS_CSSG_ENTERING, GS_CSSG_DUMMY, GS_CSSG_HAS_SSS };
/* what: GS_CSSG_*; i: the port (0..3), the tab, the visible index, the slot or the fighter id, by what */
int gw_Ui_CssGet(int what, int i) {
    const AtCss *c = &gs_selm_css;
    if (what >= GS_CSSG_KIND && what <= GS_CSSG_TARGET) {
        const AtCssPort *s;
        if (i < 0 || i > 3) return 0;
        s = &c->p[i];
        switch (what) {
        case GS_CSSG_KIND: return s->kind;
        case GS_CSSG_CK: return s->ck;
        case GS_CSSG_COSTUME: return s->costume;
        case GS_CSSG_CPU_LV: return s->cpu_lv;
        case GS_CSSG_TEAM: return s->team;
        case GS_CSSG_CUR: return s->cur;
        case GS_CSSG_CARD: return s->card;
        default: return s->target;
        }
    }
    switch (what) {
    case GS_CSSG_DONE: return c->done;
    case GS_CSSG_N_VIS: return c->n_vis;
    case GS_CSSG_TAB: return c->tab;
    case GS_CSSG_COUNT_IN: return at_css_count_in(c);
    case GS_CSSG_TRAIN_H: return c->train_h;
    case GS_CSSG_TRAIN_C: return c->train_c;
    case GS_CSSG_N_SLOTS: return c->n_slots;
    case GS_CSSG_ONLINE: return c->online;
    case GS_CSSG_TEAMS: return c->teams;
    case GS_CSSG_TAB_OFFERED: return at_css_tab_offered(c, i);
    case GS_CSSG_TAB_COUNT: return at_css_tab_count(c, i);
    case GS_CSSG_VIS_SLOT: return i >= 0 && i < c->n_vis ? c->vis[i] : -1;
    case GS_CSSG_SLOT_CK: return i >= 0 && i < c->n_slots ? c->ck[i] : AT_CK_NONE;
    case GS_CSSG_SLOT_OK: return i >= 0 && i < c->n_slots ? c->ok[i] : 0;
    case GS_CSSG_SLOT_TAB: return i >= 0 && i < c->n_slots ? c->tab_of[i] : 0;
    case GS_CSSG_VIS_OF_CK: return at_css_vis_of_ck(c, i);
    case GS_CSSG_MOUSE_PORT: return at_css_mouse_port(c);
    case GS_CSSG_MAX_HUMANS: return c->prof != NULL ? c->prof->max_humans : 0;
    case GS_CSSG_CPU_CARDS: return c->prof != NULL ? c->prof->cpu_cards : 0;
    case GS_CSSG_MIN_TO_START: return c->prof != NULL ? c->prof->min_to_start : 2;
    case GS_CSSG_ENTERING: return c->prof != NULL ? c->prof->entering_port_only : 0;
    case GS_CSSG_DUMMY: return c->prof != NULL ? c->prof->dummy_cpu : 0;
    case GS_CSSG_HAS_SSS: return c->prof != NULL ? c->prof->has_sss : 0;
    default: return 0;
    }
}
/* NUL-terminated text into the game's buffer: 1 when there is text (the blocker: the reason the match cannot start) */
static int gs_selm_put(char *out, int cap, const char *s) {
    int n;
    if (out == NULL || cap < 1) return 0;
    for (n = 0; s != NULL && s[n] != '\0' && n < cap - 1; n++) out[n] = s[n];
    out[n] = '\0';
    return n;
}
int gw_Ui_CssBlocker(char *out, int cap) {
    char buf[96];
    const char *why = at_css_blocker(&gs_selm_css, buf, sizeof buf);
    if (why == NULL) { gs_selm_put(out, cap, ""); return 0; }
    gs_selm_put(out, cap, why);
    return 1;
}
int gw_Ui_CssToast(char *out, int cap) { return gs_selm_put(out, cap, gs_selm_css.toast); }

void gw_Ui_SssOpen(int online) { at_sss_open(&gs_selm_sss, online); gs_selm_nst = 0; }
void gw_Ui_SssStagesBegin(void) { gs_selm_nst = 0; }
void gw_Ui_SssStageAdd(int ext, int type, int row, int ok, int tab_of) {
    AtSssStage *s;
    if (gs_selm_nst >= AT_SSS_MAX + 8) return;
    s = &gs_selm_st[gs_selm_nst++];
    s->ext = ext; s->type = type; s->row = row; s->ok = ok != 0; s->tab_of = (unsigned char) tab_of;
}
void gw_Ui_SssStagesEnd(void) { at_sss_set_stages(&gs_selm_sss, gs_selm_nst, gs_selm_st); }
void gw_Ui_SssCols(int cols) { at_sss_set_cols(&gs_selm_sss, cols); }
void gw_Ui_SssCursorExt(int ext) { at_sss_cursor_ext(&gs_selm_sss, ext); }
void gw_Ui_SssSetTab(int tab) { at_sss_set_tab(&gs_selm_sss, tab); }
unsigned gw_Ui_SssStep(unsigned trig, unsigned rep, int random_pick) { return at_sss_step(&gs_selm_sss, trig, rep, random_pick); }
unsigned gw_Ui_SssMouse(int vis, int moved, int click, int wheel) { return at_sss_mouse_bits(&gs_selm_sss, vis, moved, click, wheel); }
int gw_Ui_SssRandomPool(void) { return at_sss_random_pool(&gs_selm_sss); }
int gw_Ui_SssPickRandom(int r) { return at_sss_pick_random(&gs_selm_sss, r); }
enum { GS_SSSG_N, GS_SSSG_TAB, GS_SSSG_CUR, GS_SSSG_DONE, GS_SSSG_GO, GS_SSSG_PICK_EXT, GS_SSSG_N_VIS, GS_SSSG_VIS_SLOT, GS_SSSG_EXT, GS_SSSG_TYPE, GS_SSSG_OK,
       GS_SSSG_ROW, GS_SSSG_TAB_OFFERED, GS_SSSG_TAB_COUNT, GS_SSSG_TAB_OF, GS_SSSG_CUR_TYPE, GS_SSSG_TAB_FOR_EXT };
int gw_Ui_SssGet(int what, int i) {
    const AtSss *s = &gs_selm_sss;
    const AtSssStage *g = i >= 0 && i < s->n ? &s->st[i] : NULL;
    switch (what) {
    case GS_SSSG_N: return s->n;
    case GS_SSSG_TAB: return s->tab;
    case GS_SSSG_CUR: return s->cur;
    case GS_SSSG_DONE: return s->done;
    case GS_SSSG_GO: return s->go;
    case GS_SSSG_PICK_EXT: return s->pick_ext;
    case GS_SSSG_N_VIS: return s->n_vis;
    case GS_SSSG_VIS_SLOT: return i >= 0 && i < s->n_vis ? s->vis[i] : -1;
    case GS_SSSG_EXT: return g != NULL ? g->ext : -1;
    case GS_SSSG_TYPE: return g != NULL ? g->type : 0;
    case GS_SSSG_OK: return g != NULL ? g->ok : 0;
    case GS_SSSG_ROW: return g != NULL ? g->row : 0;
    case GS_SSSG_TAB_OFFERED: return at_sss_tab_offered(s, i);
    case GS_SSSG_TAB_COUNT: return at_sss_tab_count(s, i);
    case GS_SSSG_TAB_OF: return g != NULL ? g->tab_of : 0;
    case GS_SSSG_CUR_TYPE: return s->cur >= 0 && s->cur < s->n_vis ? s->st[s->vis[s->cur]].type : 0;
    case GS_SSSG_TAB_FOR_EXT: return at_sss_tab_for_ext(i);
    default: return 0;
    }
}
int gw_Ui_SssToast(char *out, int cap) { return gs_selm_put(out, cap, gs_selm_sss.toast); }
