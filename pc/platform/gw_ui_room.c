#include "gw_ui_room.h"
#include "gw_ui_tokens.h"
#include <stdio.h>
#include <string.h>

/* ---- the grid ---- */
void at_room_grid(const int *group, int n, int cols, int rows, AtGrid *g)
{
    int i, page = 0, row = 0, col = 0, grp = -1, has0 = 0, has1 = 0, grouped, need_head = 0, c0 = 0, c1 = 0;
    memset(g, 0, sizeof *g);
    if (cols < 1) cols = 1;
    if (cols > 8) cols = 8;
    if (rows < 1) rows = 1;
    if (n > AT_ROOM_STAGES) n = AT_ROOM_STAGES;
    if (n < 0) n = 0;
    for (i = 0; i < n; i++) { if (group[i]) { has1 = 1; c1++; } else { has0 = 1; c0++; } }
    grouped = has0 && has1;
    g->cols = cols; g->rows = rows; g->n = n; g->pages = 1;
    for (i = 0; i < n; i++) {
        int gi = group[i] ? 1 : 0, new_group = grouped && gi != grp, new_row = i > 0 && (new_group || col >= cols);
        if (new_row) { row++; col = 0; }
        if (row >= rows) { page++; row = 0; col = 0; need_head = grouped; }
        if ((new_group || need_head) && g->nhead < (int) (sizeof g->head / sizeof g->head[0])) {
            AtGridHead *h = &g->head[g->nhead++];
            h->page = page; h->row = row; h->group = gi; h->count = gi ? c1 : c0;
        }
        need_head = 0;
        g->cell[i].col = col; g->cell[i].row = row; g->cell[i].page = page;
        col++;
        grp = gi;
    }
    g->pages = page + 1;
}

int at_room_pick_cols(int n, int grouped, int wide)
{
    if (n <= 6 && !grouped) return 3;
    return wide ? 6 : 4;
}

int at_room_cursor_fix(const AtRoomView *v)
{
    int i;
    if (v->n_stages <= 0) return -1;
    if (v->cursor >= 0 && v->cursor < v->n_stages) return v->cursor;
    for (i = 0; i < v->n_stages; i++) if (v->st[i].open) return i;
    return -1;
}

/* ---- the screen record ---- */
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

static const char *a_label(const AtRoomView *v)
{
    if (v->can_pick) return "Pick";
    if (v->my_stage_turn) return v->phase == AT_PH_BAN ? "Ban" : v->phase == AT_PH_PICK ? "Pick stage" : "Strike";
    if (v->can_ready) return v->ready_mine ? NULL : "Ready";    /* A only readies: START takes it back (the legacy rule) */
    return NULL;
}

int at_room_fill(const AtRoomView *rv, AtScreen *sc, AtView *vw, double now_ms)
{
    char title[AT_STR];
    const char *a;
    memset(sc, 0, sizeof *sc);
    sc->fn_provide = sc->fn_accept = sc->fn_back = sc->fn_focus = sc->fn_change = sc->fn_open = sc->fn_close = sc->fn_counter = sc->fn_page = sc->fn_start = -1;
    sc->fn_alt[0] = sc->fn_alt[1] = sc->fn_alt[2] = -1;
    memset(vw->key_label, 0, sizeof vw->key_label);
    memset(vw->key_shown, 0, sizeof vw->key_shown);
    vw->counter[0] = '\0';
    memset(&vw->ex, 0, sizeof vw->ex);
    vw->ex.media_model = vw->ex.media_ring = AT_NO_MODEL;                 /* a zeroed explainer would name model 0 */
    vw->ex.media_tex = -1;
    vw->ex.no_well = 1;                                                   /* a room has no picture to show */
    vw->room = rv;
    if (rv->toast[0]) {                                                /* the game side owns a toast's duration: it is drawn while it is set */
        snprintf(vw->note.text, sizeof vw->note.text, "%s", rv->toast);
        vw->note.kind = rv->toast_bad ? AT_NOTE_ERR : AT_NOTE_INFO;
        vw->note.from_ms = now_ms;
        vw->note.until_ms = now_ms + 1000.0;
    } else {
        vw->note.text[0] = '\0';
    }
    sc->chapter = 3;                                                   /* III Online */
    sc->primary = AT_PRIMARY_ROOM;
    sc->preset = AT_PRESET_NONE;
    sc->input_feed = 1;                                                /* the game side owns the pad (the legacy bits); Atlas reads none */
    snprintf(sc->parent[0], sizeof sc->parent[0], "%s", "ONLINE");
    sc->n_parents = 1;
    switch (rv->kind) {
    case AT_ROOM_CODE:
        snprintf(sc->id, sizeof sc->id, "%s", "online.code");
        snprintf(sc->title, sizeof sc->title, "%s", "JOIN ROOM");
        put_key(sc, vw, 'A', "Join");
        put_key(sc, vw, 'Y', "Paste");
        put_key(sc, vw, 'B', "Back");
        return 1;
    case AT_ROOM_WAIT:
        snprintf(sc->id, sizeof sc->id, "%s", "online.wait");
        snprintf(title, sizeof title, "%s", rv->random_search ? "FINDING OPPONENT" : rv->host ? "WAITING ROOM" : "JOINING ROOM");
        snprintf(sc->title, sizeof sc->title, "%s", title);
        if (rv->host && !rv->random_search) put_key(sc, vw, 'X', "Copy code");
        put_key(sc, vw, 'B', rv->random_search ? "Stop" : rv->host ? "Close room" : "Cancel");
        return 1;
    case AT_ROOM_LOBBY:
        snprintf(sc->id, sizeof sc->id, "%s", "online.lobby");
        snprintf(sc->title, sizeof sc->title, "%s", rv->phase_title[0] ? rv->phase_title : "LOBBY");
        sc->preset = AT_PRESET_NORMAL;
        a = a_label(rv);
        if (a != NULL) put_key(sc, vw, 'A', a);
        if (rv->can_ready) put_key(sc, vw, 'S', rv->ready_mine ? "Unready" : "Ready");
        put_key(sc, vw, 'B', rv->countdown_s > 0 && rv->ready_mine ? "Cancel" : rv->leave_armed ? "Leave now" : "Leave");
        put_key(sc, vw, 'X', "Copy code");
        if (rv->grid.pages > 1 && rv->my_stage_turn) { put_key(sc, vw, 'L', "Page"); put_key(sc, vw, 'R', "Page"); }
        snprintf(vw->counter, sizeof vw->counter, "Game %d   %d - %d", rv->game, rv->pl[0].score, rv->pl[1].score);
        vw->ex.has = 1;
        snprintf(vw->ex.kicker, sizeof vw->ex.kicker, "%s", rv->phase_title);
        snprintf(vw->ex.title, sizeof vw->ex.title, "ROOM %s", rv->code);
        snprintf(vw->ex.what, sizeof vw->ex.what, "%s", rv->instruction);
        snprintf(vw->ex.from_text, sizeof vw->ex.from_text, "%s", rv->turbo && rv->envoy ? "Host sets: Turbo, Envoy" : rv->turbo ? "Host sets: Turbo" : rv->envoy ? "Host sets: Envoy" : "Host sets: standard rules");
        return 1;
    default:
        return 0;
    }
}
