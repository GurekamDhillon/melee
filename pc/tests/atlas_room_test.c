#include "atlas_lint.h"
#include "atlas_room_fixture.h"

static void grid(void)
{
    AtGrid g;
    int i, n, cols, group[AT_ROOM_STAGES];
    /* five ungrouped stages: three columns, two rows, one page, no headers */
    memset(group, 0, sizeof group);
    at_room_grid(group, 5, 3, 3, &g);
    CHECK(g.pages == 1 && g.nhead == 0 && g.n == 5);
    CHECK(g.cell[0].col == 0 && g.cell[0].row == 0 && g.cell[2].col == 2 && g.cell[3].col == 0 && g.cell[3].row == 1);
    /* four starters and two counterpicks: a header over each group, the counterpicks start a new row */
    for (i = 0; i < 6; i++) group[i] = i >= 4;
    at_room_grid(group, 6, 4, 3, &g);
    CHECK(g.nhead == 2 && g.head[0].group == 0 && g.head[0].count == 4 && g.head[1].group == 1 && g.head[1].row == 1 && g.head[1].count == 2);
    CHECK(g.cell[4].row == 1 && g.cell[4].col == 0 && g.cell[5].col == 1);
    /* all stages: 5 starters, 27 counterpicks, six columns, three rows a page: the group's header comes back on page 2 */
    for (i = 0; i < 32; i++) group[i] = i >= 5;
    at_room_grid(group, 32, 6, 3, &g);
    CHECK(g.pages == 2);
    {
        int heads_p1 = 0;
        for (i = 0; i < g.nhead; i++) if (g.head[i].page == 1 && g.head[i].group == 1 && g.head[i].row == 0) heads_p1++;
        CHECK(heads_p1 == 1);
    }
    /* invariants over every list size and column count: unique cells, inside the grid, pages in order, every head valid */
    for (n = 0; n <= AT_ROOM_STAGES; n++) for (cols = 1; cols <= 8; cols++) {
        int split, ok = 1, lastp = 0;
        for (split = 0; split <= n; split += (n > 6 ? 5 : 1)) {
            for (i = 0; i < n; i++) group[i] = i >= split;
            at_room_grid(group, n, cols, 3, &g);
            ok = g.n == n;
            for (i = 0; i < n && ok; i++) {
                int j;
                if (g.cell[i].col < 0 || g.cell[i].col >= cols || g.cell[i].row < 0 || g.cell[i].row >= 3 || g.cell[i].page < lastp) ok = 0;
                lastp = g.cell[i].page;
                for (j = 0; j < i; j++)
                    if (g.cell[j].col == g.cell[i].col && g.cell[j].row == g.cell[i].row && g.cell[j].page == g.cell[i].page) ok = 0;
            }
            for (i = 0; i < g.nhead; i++) if (g.head[i].page >= g.pages || g.head[i].row >= 3) ok = 0;
            lastp = 0;
            CHECK(ok && (n == 0 ? g.pages == 1 : g.pages >= 1));
        }
    }
    /* the column count: three for a short ungrouped list, four compact or six wide for the rest */
    CHECK(at_room_pick_cols(5, 0, 0) == 3 && at_room_pick_cols(6, 0, 1) == 3 && at_room_pick_cols(6, 1, 0) == 4 && at_room_pick_cols(32, 1, 1) == 6);
}

static void cursor(void)
{
    AtRoomView v;
    int i;
    memset(&v, 0, sizeof v);
    CHECK(at_room_cursor_fix(&v) == -1);                       /* no list yet (a guest waits for it): no cursor */
    v.n_stages = 3; v.cursor = 7;
    for (i = 0; i < 3; i++) v.st[i].open = 1;
    CHECK(at_room_cursor_fix(&v) == 0);                        /* past the end: the first open stage */
    v.cursor = -4; CHECK(at_room_cursor_fix(&v) == 0);
    v.st[0].open = 0; v.cursor = 9; CHECK(at_room_cursor_fix(&v) == 1);
    v.cursor = 2; CHECK(at_room_cursor_fix(&v) == 2);          /* a valid cursor is left alone, open or not */
    v.st[0].open = v.st[1].open = v.st[2].open = 0; v.cursor = 9; CHECK(at_room_cursor_fix(&v) == -1);   /* nothing open */
    v.n_stages = 0; v.cursor = 0; CHECK(at_room_cursor_fix(&v) == -1);   /* the list shrank under the cursor */
}

static void fill(void)
{
    static AtScreen sc;
    static AtView vw;
    static AtRoomView rv;
    int idx;
    for (idx = 0; idx < FX_COUNT; idx += 7) {                     /* every seventh view of the space: all kinds, phases, sides */
        int i, ok;
        fx_view(idx, &rv);
        at_view_init(&vw);
        ok = at_room_fill(&rv, &sc, &vw, 1000.0);
        CHECK(ok);
        CHECK(sc.primary == AT_PRIMARY_ROOM && vw.room == &rv);
        CHECK(sc.n_keys >= 1 && sc.n_keys <= AT_MAX_KEYS && sc.title[0] != '\0' && sc.chapter == 3);
        for (i = 0; i < sc.n_keys; i++) CHECK(vw.key_shown[i] && vw.key_label[i][0] != '\0');
        CHECK(sc.preset == (rv.kind == AT_ROOM_LOBBY ? AT_PRESET_NORMAL : AT_PRESET_NONE));
        if (rv.kind == AT_ROOM_LOBBY) CHECK(vw.ex.has && vw.ex.what[0] != '\0' && strcmp(vw.ex.what, rv.instruction) == 0);
        CHECK(sc.fn_provide == -1 && sc.fn_accept == -1 && sc.fn_back == -1 && sc.fn_alt[0] == -1);   /* never a Lua reference 0 */
    }
    /* the keys name what A does now, and B asks twice before leaving */
    fx_view(0, &rv); rv.kind = AT_ROOM_LOBBY; rv.phase = AT_PH_READY; rv.can_ready = 1; rv.can_pick = 0; rv.leave_armed = 0; rv.countdown_s = 0;
    at_room_fill(&rv, &sc, &vw, 1000.0);
    CHECK(strcmp(vw.key_label[0], "Ready") == 0);
    { int b; for (b = 0; b < sc.n_keys; b++) if (sc.keys[b].btn == 'B') CHECK(strcmp(vw.key_label[b], "Leave") == 0); }
    rv.leave_armed = 1; at_room_fill(&rv, &sc, &vw, 1000.0);
    { int b; for (b = 0; b < sc.n_keys; b++) if (sc.keys[b].btn == 'B') CHECK(strcmp(vw.key_label[b], "Leave now") == 0); }
    rv.leave_armed = 0; rv.countdown_s = 3; rv.ready_mine = 1; at_room_fill(&rv, &sc, &vw, 1000.0);
    { int b; for (b = 0; b < sc.n_keys; b++) if (sc.keys[b].btn == 'B') CHECK(strcmp(vw.key_label[b], "Cancel") == 0); }   /* B in the count takes the ready back */
    /* a toast is a corner note while it is set, and gone when it is not */
    rv.kind = AT_ROOM_LOBBY; snprintf(rv.toast, sizeof rv.toast, "Press B again to leave the room."); rv.toast_bad = 1;
    at_room_fill(&rv, &sc, &vw, 1000.0);
    CHECK(strcmp(vw.note.text, rv.toast) == 0 && vw.note.kind == AT_NOTE_ERR && vw.note.until_ms > 1000.0);
    rv.toast[0] = '\0'; at_room_fill(&rv, &sc, &vw, 1000.0); CHECK(vw.note.text[0] == '\0');
    /* an unknown kind fills nothing and says so */
    rv.kind = 0; CHECK(at_room_fill(&rv, &sc, &vw, 1000.0) == 0);
}

/* the primary of one view at one width, drawn straight into a fresh recording sink */
static AtHits RH;
static AtLayout draw_room(const AtRoomView *rv, float w)
{
    AtLayout L;
    AtSink s = rec_sink();
    at_layout(w, rv->kind == AT_ROOM_LOBBY ? AT_PRESET_NORMAL : AT_PRESET_NONE, &L);
    memset(&RH, 0, sizeof RH);
    CHECK(at_room_render(rv, &L, &FAKE, &s, &RH, 0.0) == 0);                 /* no hit rectangle dropped */
    return L;
}

static int hits_of(int code)
{
    int i, n = 0;
    for (i = 0; i < RH.n; i++) n += RH.h[i].a == code;
    return n;
}

static void render(void)
{
    static AtRoomView rv;
    static const float widths[3] = { 640.0f, 853.0f, 1140.0f };
    int idx, k, i;
    for (idx = 0; idx < FX_COUNT; idx += 5) for (k = 0; k < 3; k++) {
        AtLayout L;
        fx_view(idx, &rv);
        L = draw_room(&rv, widths[k]);
        CHECK(lint_text_inside(L.primary, 0) == 0);                          /* every text inside the primary, 12 px or more */
        if (lint_text_overlaps() != 0) { printf("view %d width %g kind %d phase %d host %d recon %d list %d\n", idx, widths[k], rv.kind, rv.phase, rv.host, rv.reconnecting, rv.n_stages); lint_dump_overlaps(); }
        CHECK(lint_text_overlaps() == 0);                                    /* and none on top of another */
        CHECK(RH.n <= AT_MAX_HITS);
        for (i = 0; i < RH.n; i++) {
            CHECK(RH.h[i].kind == AT_HIT_ROOM);
            CHECK(RH.h[i].r.x >= L.primary.x - 0.5f && RH.h[i].r.x + RH.h[i].r.w <= L.primary.x + L.primary.w + 0.5f);
            CHECK(RH.h[i].r.y >= L.primary.y - 8.0f && RH.h[i].r.y + RH.h[i].r.h <= L.primary.y + L.primary.h + 0.5f);   /* 8: a code strip above its slot */
        }
    }

    /* the lobby, my stage turn: the focused tile has three cues, the others none; the chamfer rule holds on a resting tile */
    fx_view(0, &rv); rv.kind = AT_ROOM_LOBBY; rv.phase = AT_PH_STRIKE; rv.me = 1; rv.host = 0; rv.turn = 1; rv.my_stage_turn = 1;
    rv.n_stages = 6; rv.cursor = 2; rv.reconnecting = 0; rv.coin_on = 0;
    { int g[AT_ROOM_STAGES], j; for (j = 0; j < 6; j++) { g[j] = 0; rv.st[j].open = 1; rv.st[j].state = AT_STAGE_FREE; rv.st[j].starter = 1; }
      at_room_grid(g, 6, 3, 3, &rv.grid); }
    draw_room(&rv, 640.0f);
    {
        int found = 0, stage_hits = 0;
        for (i = 0; i < RH.n; i++) if (RH.h[i].a == AT_RH_STAGE) {
            stage_hits++;
            if (RH.h[i].b == 2) { AtRect r = RH.h[i].r; found = 1; CHECK(lint_focus_cell(r, AT_C_P2) == 0); }   /* the guest's colour (me == 1) */
        }
        CHECK(found && stage_hits == 6);
        for (i = 0; i < RH.n; i++) if (RH.h[i].a == AT_RH_STAGE && RH.h[i].b != 2) CHECK(lint_plate_corners(RH.h[i].r, 3.0f) == 0);
    }
    /* not my turn, or reconnecting: no focus anywhere, and no ember (the action plate is only in the ready phase) */
    rv.my_stage_turn = 0; draw_room(&rv, 640.0f); CHECK(count_color(AT_C_EMBER) == 0);
    rv.my_stage_turn = 1; rv.reconnecting = 1; draw_room(&rv, 640.0f); CHECK(count_color(AT_C_EMBER) == 0 && find_text("RECONNECTING") != NULL);
    rv.reconnecting = 0;
    /* states have marks and words: struck and banned are crossed (the bars and BAN, with the striker's numeral), picked is ringed (PICK), not yet says so */
    rv.st[0].state = AT_STAGE_STRUCK_P1; rv.st[1].state = AT_STAGE_BANNED; rv.st[3].state = AT_STAGE_PICKED; rv.st[4].open = 0;
    draw_room(&rv, 640.0f);
    CHECK(find_text("BAN") != NULL && find_text("PICK") != NULL && find_text("NOT YET") != NULL);
    CHECK(count_color(AT_C_ROSE) >= 4);                                                         /* the bars of the two crossed tiles */
    rv.phase = AT_PH_READY; draw_room(&rv, 640.0f); CHECK(find_text("OUT") != NULL);          /* decided: the free ones say OUT */
    rv.phase = AT_PH_STRIKE;
    /* the ready phase: the one ember plate says READY, and after readying it is jade and says so */
    { int j; for (j = 0; j < 6; j++) rv.st[j].state = AT_STAGE_FREE; }                       /* the picked tile's ember ring is the pick's, not the action's */
    rv.phase = AT_PH_READY; rv.my_stage_turn = 0; rv.can_ready = 1; rv.ready_mine = 0; rv.countdown_s = 0; rv.can_pick = 0;
    draw_room(&rv, 640.0f); CHECK(count_color(AT_C_EMBER) >= 1 && find_text("READY") != NULL && hits_of(AT_RH_ACTION) == 1);
    rv.ready_mine = 1; draw_room(&rv, 640.0f); CHECK(count_color(AT_C_EMBER) == 0 && find_text("READY - WAITING") != NULL && hits_of(AT_RH_ACTION) == 0);
    /* the countdown plate */
    rv.countdown_s = 3; draw_room(&rv, 640.0f); CHECK(find_text("3") != NULL && find_text("B CANCEL") != NULL);

    /* the code screen: slots hit, strips for the active slot, an invalid code says FAILED as well as showing rose */
    fx_view(0, &rv); rv.kind = AT_ROOM_CODE; rv.code_in.slot = 1; rv.code_in.invalid = 1; snprintf(rv.code_msg, sizeof rv.code_msg, "No room with that code."); rv.code_msg_bad = 1;
    draw_room(&rv, 640.0f);
    CHECK(hits_of(AT_RH_CODE_SLOT) == 4 && hits_of(AT_RH_CODE_UP) == 1 && hits_of(AT_RH_CODE_DOWN) == 1);
    CHECK(find_text("FAILED") != NULL && count_color(AT_C_ROSE) >= 3);

    /* the waiting room: the host can copy, the guest cannot; Random shows a clock, not a code */
    fx_view(0, &rv); rv.kind = AT_ROOM_WAIT; rv.host = 1; rv.random_search = 0; draw_room(&rv, 640.0f);
    CHECK(hits_of(AT_RH_COPY) == 1);
    rv.host = 0; draw_room(&rv, 640.0f);
    CHECK(hits_of(AT_RH_COPY) == 0);
    rv.random_search = 1; rv.random_secs = 72; draw_room(&rv, 640.0f); CHECK(find_text("1:12") != NULL && find_text("ABCD") == NULL);
    /* a guest who has not seen the rules is told so instead of seeing zeros */
    rv.random_search = 0; rv.rules_known = 0; draw_room(&rv, 640.0f); CHECK(find_text("Set by the host.") != NULL);

    /* the life of the view: no room, no drawing (the chrome is the renderer's) */
    {
        AtSink s = rec_sink(); AtLayout L; AtHits h;
        at_layout(640.0f, AT_PRESET_NONE, &L); h.n = 0;
        CHECK(at_room_render(NULL, &L, &FAKE, &s, &h, 0.0) == 0 && REC.np == 0 && REC.nt == 0 && h.n == 0);
    }
    /* the whole screen through the real renderer: under the quad cap with 32 stages at the widest width, chrome and room both drawn */
    {
        static AtScreen sc; static AtView vw; static AtHits hits; AtRenderInfo info; AtSink s;
        fx_view(0, &rv); rv.kind = AT_ROOM_LOBBY; rv.phase = AT_PH_STRIKE; rv.n_stages = 32; rv.my_stage_turn = 1; rv.cursor = 7;
        { int g[AT_ROOM_STAGES], j; for (j = 0; j < 32; j++) { g[j] = j >= 5; rv.st[j].open = 1; } at_room_grid(g, 32, 6, 3, &rv.grid); }
        at_view_init(&vw); at_room_fill(&rv, &sc, &vw, 1000.0);
        s = rec_sink();
        at_render_ex(&sc, &vw, 1140.0f, 1000.0, 0, &FAKE, &s, &hits, &info);
        CHECK(info.entries < 1200 && !info.capped && info.hits_dropped == 0);
        CHECK(find_text("ROOM ABCD") != NULL && find_text("STAGE STRIKING") != NULL);                  /* the explainer, from the chrome */
        vw.room = NULL; s = rec_sink();
        at_render_ex(&sc, &vw, 640.0f, 1000.0, 0, &FAKE, &s, &hits, &info);
        CHECK(find_text("BAN") == NULL && find_text("PICK") == NULL);                                  /* chrome only, nothing dangling */
        { int j, rooms = 0; for (j = 0; j < hits.n; j++) rooms += hits.h[j].kind == AT_HIT_ROOM; CHECK(rooms == 0); }
        /* the leave fade covers the room: a ground quad over everything, the strength asked for */
        vw.room = &rv; rv.fade_out = 1000; s = rec_sink();
        at_render_ex(&sc, &vw, 640.0f, 1000.0, 0, &FAKE, &s, &hits, &info);
        CHECK(REC.np > 0 && REC.p[REC.np - 1].rgba == ((AT_C_GROUND & 0xFFFFFF00u) | 255u));
        rv.fade_out = 0;
    }
}

/* A reference model of what the legacy lobby does with each intent (fl_frame_lobby), used only to compare streams.
 * It returns the NETPLAY-VISIBLE actions: the things that call into the protocol. Cursor moves and arming B are local. */
typedef struct { char act[24]; int arg; } NetAct;
typedef struct { const AtRoomView *v; int cursor, armed; NetAct out[16]; int n; } Model;
static void emit(Model *m, const char *a, int arg) { if (m->n < 16) { snprintf(m->out[m->n].act, 24, "%s", a); m->out[m->n].arg = arg; m->n++; } }
static void consume(Model *m, AtRoomIntent in)
{
    const AtRoomView *v = m->v;
    switch (in.kind) {
    case AT_RI_STAGE_AT: m->cursor = in.arg; break;
    case AT_RI_RIGHT: m->cursor = (m->cursor + 1) % v->n_stages; break;
    case AT_RI_LEFT: m->cursor = (m->cursor + v->n_stages - 1) % v->n_stages; break;
    case AT_RI_ACCEPT:
        if (v->can_ready && !v->ready_mine) emit(m, "READY", 1);
        else if (v->can_pick) emit(m, "PICK_CHAR", 0);
        else if (v->my_stage_turn) emit(m, "STAGE_ACT", m->cursor);
        break;
    case AT_RI_START: if (v->can_ready) emit(m, "READY", !v->ready_mine); break;
    case AT_RI_BACK:
        if (v->countdown_s > 0 && v->ready_mine) emit(m, "READY", 0);
        else if (m->armed) emit(m, "LEAVE", 0);
        else m->armed = 1;
        break;
    default: break;
    }
}
static int same(const Model *a, const Model *b)
{
    int i;
    if (a->n != b->n) return 0;
    for (i = 0; i < a->n; i++) if (strcmp(a->out[i].act, b->out[i].act) != 0 || a->out[i].arg != b->out[i].arg) return 0;
    return 1;
}

static int mouse_step(const AtRoomView *v, AtMouse *m, float x, float y, int buttons, int wheel, AtRoomIntent *out, int cap)
{
    return at_room_mouse_intents(v, m, x, y, buttons, wheel, &RH, out, cap);
}
static float tx_of(int hit) { return RH.h[hit].r.x + 5.0f; }
static float ty_of(int hit) { return RH.h[hit].r.y + 5.0f; }

static void intents(void)
{
    static AtRoomView rv;
    AtRoomIntent in[8];
    AtMouse m; AtKeys k;
    Model pad, mouse, kbd;
    int i, n, tile3 = -1, action = -1;
    memset(&m, 0, sizeof m); memset(&k, 0, sizeof k);

    /* a lobby on my stage turn with six open stages; the pointer targets are the recorded hit rectangles */
    fx_view(0, &rv); rv.kind = AT_ROOM_LOBBY; rv.phase = AT_PH_STRIKE; rv.me = 1; rv.host = 0; rv.turn = 1; rv.my_stage_turn = 1; rv.n_stages = 6; rv.cursor = 1;
    rv.coin_on = 0; rv.countdown_s = 0;
    { int g[AT_ROOM_STAGES], j; for (j = 0; j < 6; j++) { g[j] = 0; rv.st[j].open = 1; rv.st[j].state = AT_STAGE_FREE; } at_room_grid(g, 6, 3, 3, &rv.grid); }
    draw_room(&rv, 640.0f);
    for (i = 0; i < RH.n; i++) if (RH.h[i].a == AT_RH_STAGE && RH.h[i].b == 3) tile3 = i;
    CHECK(tile3 >= 0);
    {
        float tx = tx_of(tile3), ty = ty_of(tile3);
        /* the same job three ways: strike stage 3 */
        memset(&pad, 0, sizeof pad); pad.v = &rv; pad.cursor = 1;
        { AtRoomIntent a = { AT_RI_RIGHT, 0 }, b = { AT_RI_RIGHT, 0 }, c = { AT_RI_ACCEPT, 0 }; consume(&pad, a); consume(&pad, b); consume(&pad, c); }   /* pad: right, right, A */
        memset(&mouse, 0, sizeof mouse); mouse.v = &rv; mouse.cursor = 1;
        n = mouse_step(&rv, &m, 400.0f, 300.0f, 0, 0, in, 8);                      /* the pointer arrives somewhere: nothing */
        for (i = 0; i < n; i++) consume(&mouse, in[i]);
        n = mouse_step(&rv, &m, tx, ty, 0, 0, in, 8);                              /* hover over tile 3 */
        CHECK(n == 1 && in[0].kind == AT_RI_STAGE_AT && in[0].arg == 3);           /* hover moves the cursor... */
        for (i = 0; i < n; i++) consume(&mouse, in[i]);
        CHECK(mouse.n == 0);                                                       /* ...and does nothing netplay-visible */
        n = mouse_step(&rv, &m, tx, ty, 1, 0, in, 8);                              /* left click */
        CHECK(n == 2 && in[0].kind == AT_RI_STAGE_AT && in[1].kind == AT_RI_ACCEPT);
        for (i = 0; i < n; i++) consume(&mouse, in[i]);
        memset(&kbd, 0, sizeof kbd); kbd.v = &rv; kbd.cursor = 1;
        n = at_room_key_intents(&k, AT_KEY_RIGHT, 0.0, in, 8); for (i = 0; i < n; i++) consume(&kbd, in[i]);
        n = at_room_key_intents(&k, 0, 10.0, in, 8);
        n = at_room_key_intents(&k, AT_KEY_RIGHT, 20.0, in, 8); for (i = 0; i < n; i++) consume(&kbd, in[i]);
        n = at_room_key_intents(&k, AT_KEY_RIGHT | AT_KEY_ENTER, 30.0, in, 8); for (i = 0; i < n; i++) consume(&kbd, in[i]);
        CHECK(pad.n == 1 && strcmp(pad.out[0].act, "STAGE_ACT") == 0 && pad.out[0].arg == 3);
        CHECK(same(&pad, &mouse));                                                 /* pad, mouse and keyboard: one netplay-visible action */
        CHECK(kbd.n == 1 && strcmp(kbd.out[0].act, "STAGE_ACT") == 0);
    }
    /* hover alone: 100 pointer moves over every rectangle with no button, nothing netplay-visible; a resting pointer is silent */
    {
        Model hv; int steps;
        memset(&hv, 0, sizeof hv); hv.v = &rv; hv.cursor = 1; memset(&m, 0, sizeof m);
        for (steps = 0; steps < 100; steps++) {
            const AtHit *h = &RH.h[steps % RH.n];
            n = mouse_step(&rv, &m, h->r.x + 3.0f + (float) (steps % 3), h->r.y + 3.0f, 0, 0, in, 8);
            for (i = 0; i < n; i++) consume(&hv, in[i]);
        }
        CHECK(hv.n == 0);
        n = mouse_step(&rv, &m, 123.0f, 45.0f, 0, 0, in, 8); n = mouse_step(&rv, &m, 123.0f, 45.0f, 0, 0, in, 8); CHECK(n == 0);   /* resting: silent */
        n = mouse_step(&rv, &m, -1000.0f, -1000.0f, 1, 0, in, 8); CHECK(n == 0);                                                   /* off the picture: silent */
        n = mouse_step(&rv, &m, tx_of(tile3), ty_of(tile3), 0, 0, in, 8);                                                          /* back on, no click yet */
        CHECK(n <= 1);
    }
    /* the ready phase: a click on the action plate readies (as A does), a click elsewhere does nothing, START and A agree */
    rv.my_stage_turn = 0; rv.phase = AT_PH_READY; rv.can_ready = 1; rv.ready_mine = 0; draw_room(&rv, 640.0f);
    for (i = 0; i < RH.n; i++) if (RH.h[i].a == AT_RH_ACTION) action = i;
    CHECK(action >= 0);
    memset(&m, 0, sizeof m);
    mouse_step(&rv, &m, RH.h[action].r.x + 4.0f, RH.h[action].r.y + 4.0f, 0, 0, in, 8);
    n = mouse_step(&rv, &m, RH.h[action].r.x + 4.0f, RH.h[action].r.y + 4.0f, 1, 0, in, 8);
    CHECK(n == 1 && in[0].kind == AT_RI_ACCEPT);
    mouse_step(&rv, &m, 3.0f, 3.0f, 0, 0, in, 8);
    n = mouse_step(&rv, &m, 3.0f, 3.0f, 1, 0, in, 8); CHECK(n == 0);                                    /* a click on nothing */
    n = mouse_step(&rv, &m, 3.0f, 3.0f, 2, 0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_BACK);       /* right click: back */
    {   /* the deliberate narrowing: a click on a stage tile when it is not my stage turn is nothing (it must not ready me) */
        int tile = -1;
        for (i = 0; i < RH.n; i++) if (RH.h[i].a == AT_RH_STAGE) tile = i;
        CHECK(tile >= 0);
        mouse_step(&rv, &m, tx_of(tile), ty_of(tile), 0, 0, in, 8);
        n = mouse_step(&rv, &m, tx_of(tile), ty_of(tile), 1, 0, in, 8); CHECK(n == 0);
        n = mouse_step(&rv, &m, tx_of(tile), ty_of(tile), 0, 0, in, 8); CHECK(n == 0);
    }

    /* the Envoy reward pick open does not change one intent or one key (the lobby keeps today's input behaviour) */
    {
        AtRoomIntent a[8], b[8]; int na, nb; AtMouse m1, m2;
        memset(&m1, 0, sizeof m1); memset(&m2, 0, sizeof m2);
        rv.reward_open = 0; draw_room(&rv, 640.0f);
        mouse_step(&rv, &m1, RH.h[action].r.x + 4.0f, RH.h[action].r.y + 4.0f, 0, 0, a, 8);
        na = mouse_step(&rv, &m1, RH.h[action].r.x + 4.0f, RH.h[action].r.y + 4.0f, 1, 0, a, 8);
        rv.reward_open = 1; rv.reward_left_s = 20; draw_room(&rv, 640.0f);
        mouse_step(&rv, &m2, RH.h[action].r.x + 4.0f, RH.h[action].r.y + 4.0f, 0, 0, b, 8);
        nb = mouse_step(&rv, &m2, RH.h[action].r.x + 4.0f, RH.h[action].r.y + 4.0f, 1, 0, b, 8);
        CHECK(na == nb && (na == 0 || a[0].kind == b[0].kind));
        {
            AtKeys k1, k2; AtRoomIntent ka[8], kb[8]; int xa, xb;
            memset(&k1, 0, sizeof k1); memset(&k2, 0, sizeof k2);
            rv.reward_open = 0; xa = at_room_key_intents(&k1, AT_KEY_ENTER, 0.0, ka, 8);
            rv.reward_open = 1; xb = at_room_key_intents(&k2, AT_KEY_ENTER, 0.0, kb, 8);
            CHECK(xa == xb && xa == 1 && ka[0].kind == kb[0].kind);
        }
        rv.reward_open = 0;
    }
    /* B twice leaves, the same for the pad and for a click on the B key hint (the chrome records that hit rectangle: add one by hand) */
    {
        Model mb, pb;
        AtRoomIntent back = { AT_RI_BACK, 0 };
        AtHit key; int hb;
        memset(&key, 0, sizeof key); key.r.x = 200.0f; key.r.y = 440.0f; key.r.w = 80.0f; key.r.h = 22.0f; key.kind = AT_HIT_KEY; key.a = 'B';
        hb = RH.n; RH.h[RH.n++] = key;
        memset(&mb, 0, sizeof mb); mb.v = &rv; memset(&pb, 0, sizeof pb); pb.v = &rv;
        rv.countdown_s = 0;
        consume(&pb, back); consume(&pb, back);
        memset(&m, 0, sizeof m);
        for (i = 0; i < 2; i++) {
            mouse_step(&rv, &m, RH.h[hb].r.x + 4.0f, RH.h[hb].r.y + 4.0f, 0, 0, in, 8);
            n = mouse_step(&rv, &m, RH.h[hb].r.x + 4.0f, RH.h[hb].r.y + 4.0f, 1, 0, in, 8);
            CHECK(n == 1 && in[0].kind == AT_RI_BACK);
            consume(&mb, in[0]);
            mouse_step(&rv, &m, RH.h[hb].r.x + 4.0f, RH.h[hb].r.y + 4.0f, 0, 0, in, 8);   /* release */
        }
        CHECK(same(&pb, &mb) && pb.n == 1 && strcmp(pb.out[0].act, "LEAVE") == 0);
        RH.n--;
    }
    /* the code screen: wheel steps the letter, a click on the active slot's strip steps it, a click on another slot moves there */
    fx_view(0, &rv); rv.kind = AT_ROOM_CODE; rv.code_in.slot = 1; draw_room(&rv, 640.0f);
    memset(&m, 0, sizeof m);
    mouse_step(&rv, &m, 300.0f, 200.0f, 0, 0, in, 8);
    n = mouse_step(&rv, &m, 300.0f, 200.0f, 0, 1, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_UP);
    n = mouse_step(&rv, &m, 300.0f, 200.0f, 0, -1, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_DOWN);
    {
        int up = -1, down = -1, slot2 = -1;
        for (i = 0; i < RH.n; i++) { if (RH.h[i].a == AT_RH_CODE_UP) up = i; if (RH.h[i].a == AT_RH_CODE_DOWN) down = i; if (RH.h[i].a == AT_RH_CODE_SLOT && RH.h[i].b == 2) slot2 = i; }
        CHECK(up >= 0 && down >= 0 && slot2 >= 0);
        mouse_step(&rv, &m, tx_of(up), ty_of(up), 0, 0, in, 8);
        n = mouse_step(&rv, &m, tx_of(up), ty_of(up), 1, 0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_UP);
        mouse_step(&rv, &m, tx_of(down), ty_of(down), 0, 0, in, 8);
        n = mouse_step(&rv, &m, tx_of(down), ty_of(down), 1, 0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_DOWN);
        mouse_step(&rv, &m, tx_of(slot2), ty_of(slot2), 0, 0, in, 8);
        n = mouse_step(&rv, &m, tx_of(slot2), ty_of(slot2), 1, 0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_CODE_SLOT && in[0].arg == 2);
    }
    /* the code screen reads the keyboard itself (the netplay layer's own key poll): the room does not turn keys into intents there, or an arrow would act twice */
    CHECK(!at_room_takes_keys(&rv));
    rv.kind = AT_ROOM_WAIT; CHECK(at_room_takes_keys(&rv));
    rv.kind = AT_ROOM_LOBBY; CHECK(at_room_takes_keys(&rv));
    /* the lobby on my stage turn: the wheel turns the page (L before R) */
    rv.kind = AT_ROOM_LOBBY; rv.my_stage_turn = 1; rv.phase = AT_PH_BAN; draw_room(&rv, 640.0f);
    n = mouse_step(&rv, &m, 300.0f, 200.0f, 0, 1, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_PAGE_L);
    n = mouse_step(&rv, &m, 300.0f, 200.0f, 0, -1, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_PAGE_R);
    /* keyboard: arrows move, Enter accepts, Escape goes back, Tab and Shift+Tab page; typed letters are not here at all */
    memset(&k, 0, sizeof k);
    n = at_room_key_intents(&k, AT_KEY_UP, 0.0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_UP);
    at_room_key_intents(&k, 0, 10.0, in, 8);
    n = at_room_key_intents(&k, AT_KEY_ESC, 20.0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_BACK);
    at_room_key_intents(&k, 0, 30.0, in, 8);
    n = at_room_key_intents(&k, AT_KEY_TAB, 40.0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_PAGE_R);
    at_room_key_intents(&k, 0, 50.0, in, 8);
    n = at_room_key_intents(&k, AT_KEY_TAB | AT_KEY_SHIFT, 60.0, in, 8); CHECK(n == 1 && in[0].kind == AT_RI_PAGE_L);
    /* every intent has a distinct name, and the names are the ones the adapter's table reads */
    {
        static const char *const want[] = { "none", "up", "down", "left", "right", "accept", "back", "start", "copy", "paste", "page_l", "page_r", "stage_at", "code_slot" };
        for (i = 0; i <= AT_RI_CODE_SLOT; i++) CHECK(strcmp(at_room_intent_name(i), want[i]) == 0);
        CHECK(strcmp(at_room_intent_name(99), "?") == 0 && strcmp(at_room_intent_name(-1), "?") == 0);
    }
}

int main(void)
{
    grid();
    cursor();
    fill();
    render();
    intents();
    ATLAS_DONE("atlas room");
}
