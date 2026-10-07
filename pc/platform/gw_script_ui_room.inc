/* gw_script_ui_room.inc - the host half of the ONLINE ROOM screens: JOIN ROOM, the WAITING ROOM and the LOBBY (Atlas step 6). Included at the end of
 * gw_script_ui.inc (after gw_script_ui_set.inc): it uses that file's slots, stack, view and mouse state.
 *
 * DRAWING ONLY. Nothing here includes a netplay header or calls a netplay function (tools/port/check_atlas_online.sh pins it). The game side
 * (src/melee/gm/gmfrontend_atlas_online.inc) fills ONE plain-data room view BY NAME through the Ui_Room* shims below (the game calls them unprefixed; the
 * host defines gw_Ui_Room*): scalars and C strings, copied and cut to their fields, phases and stage states as words, never a shared enum. The host
 * reads the keyboard and the mouse and turns them into INTENTS (gw_ui_room.c) that the game polls by name and applies as the same MenuInput bit a pad
 * press has. The pad itself is never read here and never masked: the game's own read (mn_80229624) is the pad term, untouched.
 *
 * The slot's owner is GS_UI_ENGINE (every Lua door already refuses it to a mod and to the console) and no mod screen may be pushed over it (8.4). Only
 * one native screen exists at a time: the room refuses to open over a select or the settings, and they refuse to open over it. gw_Ui_SceneExit frees the
 * slot with the scene; every call after that is refused (the view is cleared on End, and a slot that was freed under us is noticed).
 *
 * The one out-parameter this file writes into game memory (the poll's arg, the grid cell's three ints) is written BIG-ENDIAN with gs_ui_put_be32. */

#define GS_ROOM_Q 16

static struct {
    int slot;                /* the open slot, or -1 */
    AtRoomView rv;           /* the one view; the screen's AtView borrows a pointer to it */
    AtRoomIntent q[GS_ROOM_Q];
    int qn;
    int frozen;              /* every event is dropped (the first frames on a screen, a leaving screen) */
} gs_room = { -1 };

static GsUiSlot *gs_room_get(void) {
    GsUiSlot *u;
    if (gs_room.slot < 0 || gs_room.slot >= GS_UI_SLOTS) return NULL;
    u = &gs_ui_slot[gs_room.slot];
    if (u->used && u->native_room) return u;
    gs_room.slot = -1;                                              /* the slot went under us (the scene ended): the view is stale */
    memset(&gs_room.rv, 0, sizeof gs_room.rv);
    gs_room.qn = 0;
    return NULL;
}

static int gs_room_is_open(void) { return gs_room_get() != NULL; }

static void gs_room_push(const AtRoomIntent *in, int n) {
    int i;
    if (gs_room.frozen) return;
    for (i = 0; i < n; i++) if (gs_room.qn < GS_ROOM_Q) gs_room.q[gs_room.qn++] = in[i];   /* full: the newest are dropped */
}

/* once per script tick, when the room is the top screen: the keyboard (not on the code screen: the netplay layer polls it there) and the mouse */
static void gs_room_tick(GsUiSlot *u) {
    AtRoomIntent in[16];
    float x, y, wheel;
    int buttons, n;
    if (gs_room.frozen) return;
    if (at_room_takes_keys(&gs_room.rv)) {
        n = at_room_key_intents(&u->keys, gs_ui_key_mask(), gs_now_ms(), in, 16);
        gs_room_push(in, n);
    }
    gw_Mouse_ScriptRead(&x, &y, &buttons, &wheel);
    n = at_room_mouse_intents(&gs_room.rv, &u->mouse, x, y, buttons, (int) wheel, &u->hits, in, 16);
    gs_room_push(in, n);
}

int gw_Ui_RoomOn(void) { return gw_Ui_Ready(); }
int gw_Ui_RoomOpen(void) { return gs_room_is_open(); }

static int gs_room_kind(const char *k) {
    if (k == NULL) return 0;
    return strcmp(k, "code") == 0 ? AT_ROOM_CODE : strcmp(k, "wait") == 0 ? AT_ROOM_WAIT : strcmp(k, "lobby") == 0 ? AT_ROOM_LOBBY : 0;
}

/* Attach a room screen: 1 when it is attached (new, switched kind or already that kind), 0 when it cannot be (Atlas off, an unknown kind, another native
 * screen is open, no free slot): the game then keeps its legacy drawing. The same kind again changes nothing; another kind starts a fresh view and a fresh fade. */
int gw_Ui_RoomBegin(const char *kind) {
    int k = gs_room_kind(kind), i, slot = -1;
    GsUiSlot *u = gs_room_get();
    AtScreen *sc;
    if (!gw_Ui_Ready() || k == 0) return 0;
    if (u != NULL && gs_room.rv.kind == k) {                       /* the same kind again (a re-open): the view stays, a stale intent and a freeze do not */
        gs_room.qn = 0;
        gs_room.frozen = 0;
        return 1;
    }
    if (u == NULL) {
        if (gw_Ui_SelCanOpen() == 0) return 0;                        /* a select or the settings are up (it also checks the room: none is) */
        for (i = 0; i < GS_UI_SLOTS; i++) if (!gs_ui_slot[i].used) { slot = i; break; }
        if (slot < 0) { gw_log("ui: no free screen slot for the room screen"); return 0; }
        u = &gs_ui_slot[slot];
        memset(u, 0, sizeof *u);
        u->dialog_fn = -1;
        u->used = 1;
        u->owner = GS_UI_ENGINE;
        u->native_room = 1;
        u->scene = gs.scene_kind;
        if (!at_stack_push(&gs_ui_stack, slot)) { memset(u, 0, sizeof *u); u->dialog_fn = -1; return 0; }
        gs_room.slot = slot;
    }
    memset(&gs_room.rv, 0, sizeof gs_room.rv);
    gs_room.rv.kind = k;
    gs_room.rv.ping_ms = -1;
    gs_room.qn = 0;
    gs_room.frozen = 0;
    at_view_init(&u->view);
    memset(&u->hits, 0, sizeof u->hits);
    memset(&u->mouse, 0, sizeof u->mouse);                         /* the first sample only primes: a button held from the last screen does not click here */
    memset(&u->keys, 0, sizeof u->keys);
    u->view.opened_ms = gs_now_ms();                               /* a new screen fades in */
    u->primed_frame = gs_ui_frame + 1;
    sc = &u->sc;
    at_room_fill(&gs_room.rv, sc, &u->view, gs_now_ms());
    gw_log("ui: room screen %s open (native)", sc->id);
    return 1;
}

void gw_Ui_RoomEnd(void) {
    GsUiSlot *u = gs_room_get();
    if (u != NULL) {
        gs_ui_free_engine_slot((int) (u - gs_ui_slot));
        gs_ui_after_close();
    }
    gs_room.slot = -1;
    gs_room.frozen = 0;
    gs_room.qn = 0;
    memset(&gs_room.rv, 0, sizeof gs_room.rv);                     /* nothing of the last room survives */
}

typedef struct { const char *key; size_t off; } GsRoomInt;
typedef struct { const char *key; size_t off, size; } GsRoomStr;
#define GSR_I(k, f) { k, offsetof(AtRoomView, f) }
#define GSR_S(k, f) { k, offsetof(AtRoomView, f), sizeof(((AtRoomView *) 0)->f) }
static const GsRoomInt gs_room_ints[] = {
    GSR_I("host", host), GSR_I("random", random_search), GSR_I("random_secs", random_secs), GSR_I("found", found), GSR_I("rules_known", rules_known),
    GSR_I("turbo", turbo), GSR_I("envoy", envoy), GSR_I("stage_all", stage_all), GSR_I("stocks", stocks), GSR_I("minutes", minutes), GSR_I("delay", delay),
    GSR_I("copied", copied), GSR_I("me", me), GSR_I("game", game), GSR_I("turn", turn), GSR_I("left", left), GSR_I("first", first), GSR_I("countdown", countdown_s),
    GSR_I("coin", coin_on), GSR_I("reward_open", reward_open), GSR_I("reward_left", reward_left_s), GSR_I("reconnecting", reconnecting),
    GSR_I("leave_armed", leave_armed), GSR_I("my_stage_turn", my_stage_turn), GSR_I("can_pick", can_pick), GSR_I("can_ready", can_ready),
    GSR_I("ready_mine", ready_mine), GSR_I("cursor", cursor), GSR_I("code_invalid", code_in.invalid), GSR_I("code_slot", code_in.slot),
    GSR_I("code_msg_bad", code_msg_bad), GSR_I("toast_bad", toast_bad), GSR_I("fade_out", fade_out),
};
static const GsRoomStr gs_room_strs[] = {
    GSR_S("code", code), GSR_S("status", status), GSR_S("phase_title", phase_title), GSR_S("instruction", instruction), GSR_S("code_msg", code_msg), GSR_S("toast", toast),
};

/* 1 when the key is known and a room is attached; 0 (ignored) otherwise. "ping" keeps the meter's memory (the 8 ms dead band). */
int gw_Ui_RoomSetInt(const char *key, int v) {
    size_t i;
    if (gs_room_get() == NULL || key == NULL) return 0;
    if (strcmp(key, "ping") == 0) { gs_room.rv.link_bars = at_link_bars(v, gs_room.rv.link_bars); gs_room.rv.ping_ms = v; return 1; }
    for (i = 0; i < sizeof gs_room_ints / sizeof gs_room_ints[0]; i++)
        if (strcmp(gs_room_ints[i].key, key) == 0) { *(int *) ((char *) &gs_room.rv + gs_room_ints[i].off) = v; return 1; }
    return 0;
}

/* strings are copied and cut to the field (a NULL is empty). "phase" takes a word; "code_chars" the four typed characters of the code field. */
int gw_Ui_RoomSetStr(const char *key, const char *s) {
    size_t i;
    if (gs_room_get() == NULL || key == NULL) return 0;
    if (s == NULL) s = "";
    if (strcmp(key, "phase") == 0) {
        static const char *const names[] = { "off", "char_blind", "strike", "ban", "pick", "char_winner", "char_loser", "ready", "go" };
        int p;
        gs_room.rv.phase = AT_PH_OFF;
        for (p = 0; p < 9; p++) if (strcmp(s, names[p]) == 0) gs_room.rv.phase = p;
        return 1;
    }
    if (strcmp(key, "code_chars") == 0) {
        int c, len = 0;
        while (len < 4 && s[len] != '\0') len++;
        for (c = 0; c < 4; c++) gs_room.rv.code_in.c[c] = c < len ? s[c] : '\0';
        return 1;
    }
    for (i = 0; i < sizeof gs_room_strs / sizeof gs_room_strs[0]; i++)
        if (strcmp(gs_room_strs[i].key, key) == 0) { snprintf((char *) &gs_room.rv + gs_room_strs[i].off, gs_room_strs[i].size, "%s", s); return 1; }
    return 0;
}

void gw_Ui_RoomPlayer(int who, int present, int locked, int ready, int score) {
    if (gs_room_get() == NULL || who < 0 || who > 1) return;
    gs_room.rv.pl[who].present = present != 0; gs_room.rv.pl[who].locked = locked != 0; gs_room.rv.pl[who].ready = ready != 0; gs_room.rv.pl[who].score = score;
}

void gw_Ui_RoomPlayerName(int who, const char *name, const char *fighter) {
    if (gs_room_get() == NULL || who < 0 || who > 1) return;
    snprintf(gs_room.rv.pl[who].name, sizeof gs_room.rv.pl[who].name, "%s", name != NULL ? name : "");
    snprintf(gs_room.rv.pl[who].fighter, sizeof gs_room.rv.pl[who].fighter, "%s", fighter != NULL ? fighter : "");
}

void gw_Ui_RoomStageCount(int n) {
    if (gs_room_get() == NULL) return;
    gs_room.rv.n_stages = n < 0 ? 0 : n > AT_ROOM_STAGES ? AT_ROOM_STAGES : n;
}

void gw_Ui_RoomStage(int i, const char *name, const char *state, int starter, int open) {
    static const char *const names[] = { "free", "p1", "p2", "banned", "picked" };
    int s;
    AtRoomStage *st;
    if (gs_room_get() == NULL || i < 0 || i >= gs_room.rv.n_stages) return;
    st = &gs_room.rv.st[i];
    snprintf(st->name, sizeof st->name, "%s", name != NULL ? name : "");
    st->state = AT_STAGE_FREE;
    for (s = 0; s < 5; s++) if (state != NULL && strcmp(state, names[s]) == 0) st->state = s;
    st->starter = starter != 0; st->open = open != 0;
}

/* the one grid model, for drawing and for the legacy cursor code: n stages, the first `starters` are the starters. Returns the page count. */
int gw_Ui_RoomGridFor(int n, int starters) {
    int group[AT_ROOM_STAGES], i, wide = gw_Console_ScriptWidth() >= 760.0f;
    if (gs_room_get() == NULL) return 1;
    if (n < 0) n = 0;
    if (n > AT_ROOM_STAGES) n = AT_ROOM_STAGES;
    for (i = 0; i < n; i++) group[i] = i >= starters;
    at_room_grid(group, n, at_room_pick_cols(n, starters > 0 && starters < n, wide), 3, &gs_room.rv.grid);
    return gs_room.rv.grid.pages;
}

/* the cell of stage i: 1 when it exists. col, row and page are the game's locals: written big-endian. */
int gw_Ui_RoomGridCell(int i, int *col, int *row, int *page) {
    if (gs_room_get() == NULL || i < 0 || i >= gs_room.rv.grid.n) return 0;
    gs_ui_put_be32(col, gs_room.rv.grid.cell[i].col);
    gs_ui_put_be32(row, gs_room.rv.grid.cell[i].row);
    gs_ui_put_be32(page, gs_room.rv.grid.cell[i].page);
    return 1;
}

/* once a frame, after the fields are set: the screen record is rebuilt from the view (keys, counter, explainer, the toast as a corner note) */
void gw_Ui_RoomFrame(void) {
    GsUiSlot *u = gs_room_get();
    if (u == NULL) return;
    at_room_fill(&gs_room.rv, &u->sc, &u->view, gs_now_ms());
}

/* every event is dropped while on (the first frames on a screen, a leaving one) and the ring is emptied; when it ends the keys and the mouse start from
 * what is held at that instant, so a press that was down does not fire as the freeze ends. */
void gw_Ui_RoomFreeze(int on) {
    GsUiSlot *u = gs_room_get();
    if (u == NULL) return;
    on = on != 0;
    if (on == gs_room.frozen) return;
    gs_room.frozen = on;
    if (on) gs_room.qn = 0;
    else gs_ui_prime((int) (u - gs_ui_slot));
}

/* the next intent's name ("up", "accept", "stage_at" ...): 1 with out filled, 0 when none. arg is the game's local: written big-endian. */
int gw_Ui_RoomPollName(char *out, int cap, int *arg) {
    int i;
    if (gs_room_get() == NULL || gs_room.qn == 0 || gs_room.frozen || cap < 2) return 0;
    snprintf(out, (size_t) cap, "%s", at_room_intent_name(gs_room.q[0].kind));
    gs_ui_put_be32(arg, gs_room.q[0].arg);
    for (i = 1; i < gs_room.qn; i++) gs_room.q[i - 1] = gs_room.q[i];
    gs_room.qn--;
    return 1;
}
