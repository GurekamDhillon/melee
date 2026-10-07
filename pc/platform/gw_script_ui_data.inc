/* gw_script_ui_data.inc - the host half of the DATA screens: Event Match, Name Entry's list, Sound Test, Special Messages, the Records and the Results stand-in
 * (Atlas step 8). Included at the end of gw_script_ui.inc (after gw_script_ui_set.inc): it IS the settings door (gs_set: one engine-owned native list screen with
 * a handle, rows, tabs, an explainer, key hints, and an event ring the game drains), with three differences:
 *
 *   THE ADAPTER OWNS THE CURSOR. The lists are longer than the window (up to 120 name tags), so the host does not move the focus on up and down, the wheel or the
 *   keys: each is queued as GS_SETEV_MOVE(dir) and the game moves its absolute cursor, slides the window (at_data_first) and tells the host where the focus is by
 *   row id (gw_Ui_SetFocus; ids are the ABSOLUTE row number, "r<n>", so a slide keeps a row's id). A hover or a click is still FOCUS/ACCEPT with a window-relative
 *   index (the adapter adds the window's first row). L and R are GS_SETEV_PAGE(dir), not tabs.
 *   A DISABLED ROW IS NOT CLICKABLE. An accept on a row marked locked is dropped here (gs_set_event, gs_set_mouse), whatever the adapter does next.
 *   ROWS ARE VALUES. A row is a label, a value and a sub line (a read-only AtItem); the shims fill it through the pure models (gw_ui_data_models.c), so the host holds
 *   no game pointer and no callback. The only game memory read is the 8 bytes of a name tag and the glyph tables of the text decoder, both as plain bytes.
 *
 * The slot, the handle, the ring, Freeze, SetFocus, SetTabs, SetExplain, SetKeys, SetNote, SetDialog, SetClose and the poll are the settings door's, unchanged. Events
 * are cleared when a data screen opens, when it closes (gw_Ui_SetClose) and by Freeze, as the settings'. Retail text decoded by gw_Ui_SisDecode is written only into the
 * caller's buffer: nothing here logs or stores it. The Results card state (gs_res) is built from scalars by gw_Ui_Res*. */

#include "gw_ui_data_models.h"
#include "gw_ui_results.h"
#include "gw_ui_retailtext.h"

static struct {
    AtDataRow row[AT_DATA_ROWS];   /* the last row put in each slot: a later gw_Ui_DataSub refills the item from it */
    int abs[AT_DATA_ROWS];
} gs_data;

static struct { AtResInput in; AtResults r; char name[4][24]; const char *np[4]; int built; } gs_res;

static GsUiSlot *gs_data_get(int h) {
    GsUiSlot *u = gs_set_get(h);
    return (u != NULL && gs_set.data) ? u : NULL;
}

/* 1 when a data screen is the open native screen */
static int gs_data_is_open(void) { return gs_set.h >= 0 && gs_set.data && gs_set_get(gs_set.h) != NULL; }

/* id "data.<name>": a screen the game names, the engine's own. chapter: 0 the More side (Data, Records), 1 Solo, 2 Versus. t0 and t1: the trail's parents.
 * Returns a handle for the gw_Ui_Set* shims, or -1 (Atlas off, another native screen open, no free slot, an id that is not "data." + something: the caller keeps
 * the retail screen). The events of an earlier screen are gone. */
int gw_Ui_DataOpen(const char *id, const char *t0, const char *t1, const char *title, int chapter) {
    int i, slot = -1;
    GsUiSlot *u;
    AtScreen *sc;
    if (!gw_Ui_Ready() || id == NULL || strncmp(id, "data.", 5) != 0 || id[5] == '\0' || strlen(id) >= sizeof sc->id) return -1;
    if (gs_set_is_open() || gs_room_is_open() || gw_Ui_SelCanOpen() == 0) return -1;     /* one native screen at a time: the stack has one top */
    for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) { slot = i; break; }
    if (slot < 0) { gw_log("ui: no free screen slot for the data screen"); return -1; }
    u = &gs_ui_slot[slot];
    memset(u, 0, sizeof *u);
    at_view_init(&u->view);
    u->dialog_fn = -1;
    u->used = 1;
    u->owner = GS_UI_ENGINE;
    u->native_set = 1;
    u->scene = gs.scene_kind;
    sc = &u->sc;
    sc->fn_provide = sc->fn_accept = sc->fn_back = sc->fn_focus = sc->fn_change = sc->fn_open = sc->fn_close = sc->fn_counter = sc->fn_page = sc->fn_start = -1;
    sc->fn_alt[0] = sc->fn_alt[1] = sc->fn_alt[2] = -1;
    snprintf(sc->id, sizeof sc->id, "%s", id);
    snprintf(sc->title, sizeof sc->title, "%s", title != NULL ? title : id);
    if (t0 != NULL && t0[0] != '\0') snprintf(sc->parent[sc->n_parents++], AT_STR, "%s", t0);
    if (t1 != NULL && t1[0] != '\0') snprintf(sc->parent[sc->n_parents++], AT_STR, "%s", t1);
    sc->chapter = chapter < 0 ? 0 : chapter > 5 ? 5 : chapter;
    sc->primary = AT_PRIMARY_LIST;
    sc->preset = AT_PRESET_NORMAL;
    sc->port = 1;
    sc->input_feed = 1;
    u->view.focus.block = 0; u->view.focus.index = -1;
    { int gen = gs_set.gen; memset(&gs_set, 0, sizeof gs_set); gs_set.gen = gen; }   /* a clean state, an empty ring; the generation only grows */
    gs_set.h = -1;
    gs_set.pending = 1; gs_set.fidx = -1;
    memset(&gs_data, 0, sizeof gs_data);
    if (!at_stack_push(&gs_ui_stack, slot)) { memset(u, 0, sizeof *u); u->dialog_fn = -1; return -1; }
    u->view.opened_ms = gs_now_ms();
    memset(&u->mouse, 0, sizeof u->mouse);                         /* the first sample only primes: a button held from the menu does not click here */
    u->primed_frame = gs_ui_frame + 1;
    gs_set.gen = (gs_set.gen + 1) & 0xFFFFF;
    gs_set.h = (gs_set.gen << 8) | slot;
    gs_set.data = 1;
    gw_log("ui: data screen %s open (native)", sc->id); /* OK-NOTEXT: an authored screen id */
    return gs_set.h;
}

/* one row into a slot: absolute row number `abs` (its id), label, value and flags (AT_DR_*). The sub line is gw_Ui_DataSub's. */
static void gs_data_put(GsUiSlot *u, int slot, int abs, const AtDataRow *r) {
    char id[AT_ID];
    if (slot < 0 || slot >= u->sc.n_items || slot >= AT_DATA_ROWS) return;
    gs_data.row[slot] = *r;
    gs_data.abs[slot] = abs;
    snprintf(id, sizeof id, "r%d", abs);
    at_data_fill_item(&u->sc.items[slot], id, r);
    gs_set.help[slot][0] = '\0';
}

void gw_Ui_DataRow(int h, int slot, int abs, const char *label, const char *value, int flags) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    if (u == NULL) return;
    memset(&r, 0, sizeof r);
    snprintf(r.label, AT_STR, "%s", label != NULL ? label : "");
    snprintf(r.value, AT_STR, "%s", value != NULL ? value : "");
    r.flags = (unsigned) flags & (AT_DR_DONE | AT_DR_LOCKED | AT_DR_NEW | AT_DR_WIN);
    gs_data_put(u, slot, abs, &r);
}

void gw_Ui_DataSub(int h, int slot, const char *sub) {
    GsUiSlot *u = gs_data_get(h);
    char id[AT_ID];
    if (u == NULL || slot < 0 || slot >= u->sc.n_items || slot >= AT_DATA_ROWS) return;
    snprintf(gs_data.row[slot].sub, AT_STR, "%s", sub != NULL ? sub : "");
    snprintf(id, sizeof id, "r%d", gs_data.abs[slot]);
    at_data_fill_item(&u->sc.items[slot], id, &gs_data.row[slot]);
}

/* the footer's right-hand text ("12 / 51"): built by the adapter, which owns the cursor */
void gw_Ui_DataCounter(int h, const char *text) {
    GsUiSlot *u = gs_data_get(h);
    if (u == NULL) return;
    snprintf(u->sc.counter, sizeof u->sc.counter, "%s", text != NULL ? text : "");
    snprintf(u->view.counter, sizeof u->view.counter, "%s", u->sc.counter);
}

int gw_Ui_DataWindow(void) { return AT_DATA_ROWS; }
int gw_Ui_DataFirst(int total, int per, int focus, int first) { return at_data_first(total, per, focus, first); }
const char *gw_Ui_EnvData(void) { const char *e = getenv("MELEE_ATLAS_DATA"); return e != NULL ? e : ""; }
int gw_Ui_NetplayActive(void) { return gs_ui_netplay(); }
/* MELEE_ATLAS_TEXTPROBE=1: the data adapter logs, once, per retail text source, how many strings, characters and glyphs do not decode (counts only) */
int gw_Ui_EnvProbe(void) { const char *e = getenv("MELEE_ATLAS_TEXTPROBE"); return e != NULL && strcmp(e, "1") == 0; }

/* ---- rows through the pure models (the game passes what the retail getters returned) ---- */

void gw_Ui_RowMisc(int h, int slot, int row, unsigned value, const char *fighter) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    if (u == NULL) return;
    at_misc_fill(row, value, fighter, &r);
    gs_data_put(u, slot, row, &r);
}
const char *gw_Ui_MiscLabel(int row) {
    static char label[32];
    AtMiscRow m = at_misc_row(row);
    snprintf(label, sizeof label, "%s", m.label);
    return label;
}

/* the last display index a retail event list reaches: first_bound is mnEvent_8024CE74(), debug the retail debug-level test */
int gw_Ui_EventLastOpen(int first_bound, int debug) { return at_event_last_open(first_bound, debug); }
/* flags: 1 cleared, 2 locked, 4 timed */
void gw_Ui_RowEvent(int h, int slot, int event, int flags, unsigned best, const char *name) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    if (u == NULL) return;
    at_event_row(event, (flags & 1) != 0, (flags & 2) != 0, (flags & 4) != 0, best, name, &r);
    gs_data_put(u, slot, event, &r);
}

/* a message row; ymd is year * 10000 + month * 100 + day, 0 for no date */
void gw_Ui_RowMsg(int h, int slot, int k, int ymd) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    if (u == NULL) return;
    at_msg_row(k, ymd > 0, ymd / 10000, ymd / 100 % 100, ymd % 100, &r);
    gs_data_put(u, slot, k, &r);
}
void gw_Ui_RowBonus(int h, int slot, int k) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    if (u == NULL) return;
    at_bonus_row(k, &r);
    gs_data_put(u, slot, k, &r);
}
/* the unlocked messages in retail's order: unlocked[] and date[] are the game's 66 values written one by one (gw_Ui_MsgSet), the result read by position */
static struct { int un[72]; unsigned date[72]; int ord[72]; int n; } gs_msg;
void gw_Ui_MsgSet(int id, int unlocked, unsigned date) {
    if (id < 0 || id >= 72) return;
    gs_msg.un[id] = unlocked != 0; gs_msg.date[id] = date;
}
int gw_Ui_MsgOrder(int n) {
    if (n < 0) n = 0;
    if (n > 72) n = 72;
    gs_msg.n = at_msg_order(gs_msg.un, gs_msg.date, n, gs_msg.ord);
    return gs_msg.n;
}
int gw_Ui_MsgAt(int k) { return (k >= 0 && k < gs_msg.n) ? gs_msg.ord[k] : -1; }

void gw_Ui_RowSound(int h, int slot, int i, const char *name, int playing) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    if (u == NULL) return;
    at_sound_row(i, name, playing, &r);
    gs_data_put(u, slot, i, &r);
}
void gw_Ui_RowSfx(int h, int slot, int group, int sub, int count) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    if (u == NULL) return;
    at_sfx_row(group, sub, count, &r);
    gs_data_put(u, slot, group, &r);
}
int gw_Ui_SoundToggle(int playing, int row) { return at_sound_toggle(playing, row); }

/* a ranking row: statflags is stat | (AT_SF_* << 8); value is the (already converted) number */
void gw_Ui_RowStat(int h, int slot, int rank, const char *name, int statflags, unsigned value) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    char text[AT_STR];
    if (u == NULL) return;
    at_stat_text(at_stat_info(statflags & 0xFF).kind, value, statflags >> 8, text, (int) sizeof text);
    at_rank_row(rank, name, text, &r);
    gs_data_put(u, slot, rank, &r);
}
int gw_Ui_StatKind(int stat) { return at_stat_info(stat).kind; }
const char *gw_Ui_StatLabel(int stat) {
    static char label[32];
    AtStatInfo s = at_stat_info(stat);
    snprintf(label, sizeof label, "%s", s.label);
    return label;
}
/* one stat's text, for the explainer's line about a ranked entry */
const char *gw_Ui_StatText(int statflags, unsigned value) {
    static char text[AT_STR];
    at_stat_text(at_stat_info(statflags & 0xFF).kind, value, statflags >> 8, text, (int) sizeof text);
    return text;
}
/* a ranking by values the game wrote with gw_Ui_RankSet: n entries, the result read by position (entity index) */
static struct { unsigned val[AT_DATA_ROWS * 4]; int ord[AT_DATA_ROWS * 4]; int n; } gs_rank;
void gw_Ui_RankSet(int i, unsigned value) { if (i >= 0 && i < AT_DATA_ROWS * 4) gs_rank.val[i] = value; }
int gw_Ui_RankBuild(int n) {
    if (n < 0) n = 0;
    if (n > AT_DATA_ROWS * 4) n = AT_DATA_ROWS * 4;
    gs_rank.n = at_rank(gs_rank.val, n, gs_rank.ord);
    return gs_rank.n;
}
int gw_Ui_RankAt(int k) { return (k >= 0 && k < gs_rank.n) ? gs_rank.ord[k] : -1; }

/* a name tag row: bytes is the 8-byte namedata of the tag in game memory (read as plain bytes), tag < 0 the "NEW TAG" row */
void gw_Ui_RowTag(int h, int slot, int tag, const unsigned char *bytes, int index) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow r;
    char text[24];
    if (u == NULL) return;
    if (tag < 0) at_tag_row(-1, NULL, &r);
    else {
        text[0] = '\0';
        at_tag_text(bytes, 8, text, (int) sizeof text);
        at_tag_row(tag, text, &r);
    }
    gs_data_put(u, slot, index, &r);
}
/* the tag's text for the explainer; "" when it is not Latin */
const char *gw_Ui_TagText(const unsigned char *bytes) {
    static char text[24];
    text[0] = '\0';
    at_tag_text(bytes, 8, text, (int) sizeof text);
    return text;
}

/* ---- the retail text decoder (gw_ui_retailtext.c): the tables and the stream are game memory read as bytes, the text goes only into the caller's buffer ---- */
int gw_Ui_SisDecode(const unsigned char *glyph, const unsigned char *sjis, int npairs, const unsigned char *sis, char *out, int cap, int *st5) {
    AtSisStats st;
    int r = at_sis_decode(glyph, sjis, npairs, sis, out, cap, &st);
    if (st5 != NULL) {
        gs_ui_put_be32(&st5[0], st.chars); gs_ui_put_be32(&st5[1], st.unknown); gs_ui_put_be32(&st5[2], st.controls);
        gs_ui_put_be32(&st5[3], st.jumps); gs_ui_put_be32(&st5[4], st.bad);
    }
    return r;
}

/* ---- the Results stand-in ---- */

void gw_Ui_ResBegin(int outcome, int is_teams, int canceled) {
    memset(&gs_res, 0, sizeof gs_res);
    gs_res.in.outcome = outcome; gs_res.in.is_teams = is_teams != 0; gs_res.in.canceled = canceled != 0;
    { int i; for (i = 0; i < 4; i++) { gs_res.in.p[i].pkind = AT_PK_NONE; gs_res.np[i] = gs_res.name[i]; } }
}
void gw_Ui_ResPlayer(int port, int pkind, int ckind, int stocks, int kos, int falls) {
    AtResPlayerIn *p;
    if (port < 0 || port > 3) return;
    p = &gs_res.in.p[port];
    p->pkind = pkind; p->ckind = ckind; p->stocks = stocks; p->kos = kos; p->falls = falls;
}
void gw_Ui_ResPlayer2(int port, int percent, int team, int winner, const char *name) {
    AtResPlayerIn *p;
    if (port < 0 || port > 3) return;
    p = &gs_res.in.p[port];
    p->percent = percent; p->team = team; p->winner = winner;
    snprintf(gs_res.name[port], sizeof gs_res.name[port], "%s", name != NULL ? name : "");
}
void gw_Ui_ResCommit(void) { at_results_build(&gs_res.r, &gs_res.in); gs_res.built = 1; }
void gw_Ui_ResConfirm(int port, int kind) { if (gs_res.built) at_results_confirm(&gs_res.r, port, kind); }
/* frames to wait after the last confirm, -1 while someone has not confirmed */
int gw_Ui_ResDone(void) { return (gs_res.built && at_results_done(&gs_res.r)) ? gs_res.r.exit_frames : -1; }
/* the card rows into an open data screen, in place order; returns the count */
int gw_Ui_ResFill(int h) {
    GsUiSlot *u = gs_data_get(h);
    AtDataRow rows[4];
    int n, i;
    if (u == NULL || !gs_res.built) return 0;
    n = at_results_rows(&gs_res.r, gs_res.np, rows, 4);
    gw_Ui_SetRows(h, n);
    for (i = 0; i < n; i++) {
        gs_data_put(u, i, i, &rows[i]);
    }
    return n;
}
