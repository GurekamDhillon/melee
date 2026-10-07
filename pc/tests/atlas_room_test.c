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

int main(void)
{
    grid();
    cursor();
    fill();
    ATLAS_DONE("atlas room");
}
