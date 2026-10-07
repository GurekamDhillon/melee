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

/* ---- the composite render ------------------------------------------------------------------------ */
static AtRect rc(float x, float y, float w, float h) { AtRect r; r.x = x; r.y = y; r.w = w; r.h = h; return r; }
static void e2(const AtSink *s, AtRect r) { at_plate(s, r, AT_C_PLATE2, AT_C_EDGE2, 3.0f, (float) AT_PX_CH_S); }
static unsigned port_rgba(int who) { return who == 0 ? AT_C_P1 : AT_C_P2; }

static void hit_add(AtHits *h, AtRect r, int a, int b, int *dropped)
{
    if (h == NULL) return;
    if (h->n >= AT_MAX_HITS) { (*dropped)++; return; }
    h->h[h->n].r = r; h->h[h->n].kind = AT_HIT_ROOM; h->h[h->n].a = a; h->h[h->n].b = b;
    h->n++;
}

/* a button: a 5 px chamfer plate with the key glyph and a label */
static void button(const AtSink *s, const AtTextOps *o, AtRect r, unsigned face, unsigned edge, char btn, const char *label)
{
    at_plate(s, r, face, edge, 3.0f, (float) AT_PX_CH_S);
    at_part_hint(s, o, r.x + 12.0f, r.y + r.h * 0.5f + 5.0f, btn, label);
}

/* RECONCILE (step 4): the port card. Step 4's at_part_sel_card (AtSelCard) is the character select's band card: it carries a READY or CPU
 * tag only, and the lobby needs a state word (RECONNECTING, WAITING, HOST) and a fighter line that the name must not run into. So the card
 * is drawn here in the same form (a 5 px chamfer plate, a 3 px top edge in the port colour, the port's shape and numeral, the name, a sub line)
 * with at_port_mark, the step 4 shape. Everything else in this file calls only this wrapper. who: 0 host (P1), 1 guest (P2). */
static void room_card(const AtSink *s, const AtTextOps *o, AtRect r, const AtRoomPlayer *p, int who, int is_me, const char *tag, int tone)
{
    unsigned col = port_rgba(who);
    float e = 3.0f, cx = r.x + 21.0f, cy = r.y + 3.0f + (r.h - 3.0f - e) * 0.5f, lx = r.x + 38.0f, right = r.x + r.w - 10.0f, avail = right - lx, tagw = 0.0f;
    const char *name = p->present ? (p->name[0] ? p->name : is_me ? "You" : "Opponent") : "Waiting...";
    char num[2];
    at_plate(s, r, p->present ? AT_C_PLATE2 : AT_C_GROUND2, AT_C_EDGE2, e, (float) AT_PX_CH_S);
    at_poly_rect(s, r.x + (float) AT_PX_CH_S, r.y, r.w - (float) AT_PX_CH_S, 3.0f, col);
    at_port_mark(s, cx, cy, 11.0f, who, p->present ? col : AT_C_LINE2);
    num[0] = (char) ('1' + who); num[1] = '\0';
    at_text(s, o, AT_R_CAP14, num, cx, cy + 5.0f, AT_C_INK, AT_ALIGN_CENTER, 0.0f);
    at_text(s, o, AT_R_ROW16, name, lx, r.y + 3.0f + 22.0f, p->present ? AT_C_IVORY : AT_C_MUTED, AT_ALIGN_LEFT, avail);
    if (tag[0] != '\0') {
        float want = o->width(o->user, AT_R_CAP12, tag) + 16.0f;
        tagw = want > avail * 0.8f ? avail * 0.8f : want;                           /* the tag plate is clamped to 80 % of the card: place it by the width it gets */
        at_part_tag(s, o, right - tagw, r.y + r.h - e - 24.0f, tag, tone, tagw);
    }
    if (p->present && p->fighter[0] != '\0') at_text(s, o, AT_R_BODY12, p->fighter, lx, r.y + r.h - e - 8.0f, AT_C_MUTED, AT_ALIGN_LEFT, avail - (tagw > 0.0f ? tagw + 6.0f : 0.0f));
}

/* the one word a card carries beside the fighter line: the fighter line already says "Locked in" and "Choosing..." (the legacy card's own words) */
static const char *player_tag(const AtRoomView *v, int who, int *tone)
{
    const AtRoomPlayer *p = &v->pl[who];
    *tone = AT_TAG_PLAIN;
    if (v->reconnecting) { *tone = AT_TAG_SUN; return "RECONNECTING"; }
    if (!p->present) return "WAITING";
    if (p->ready) { *tone = AT_TAG_JADE; return "READY"; }
    if (v->kind == AT_ROOM_WAIT) return who == 0 ? "HOST" : "JOINED";
    return "";
}

static int render_code(const AtRoomView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *h)
{
    AtRect P = L->primary, f;
    float fw = P.w - 48.0f, y;
    const char *msg = v->code_msg;
    int dropped = 0, i, bad = v->code_msg_bad;
    if (fw > 330.0f) fw = 330.0f;
    f = rc(P.x + (P.w - fw) * 0.5f, P.y + 54.0f, fw, 120.0f);
    at_text(s, o, AT_R_CAP12, "ROOM CODE", f.x, f.y + 4.0f, AT_C_MUTED, AT_ALIGN_LEFT, 0.0f);
    at_part_code(s, o, f, &v->code_in);
    for (i = 0; i < 4; i++) hit_add(h, at_code_slot_rect(f, i), AT_RH_CODE_SLOT, i, &dropped);
    {
        AtRect a = at_code_slot_rect(f, v->code_in.slot < 0 ? 0 : v->code_in.slot > 3 ? 3 : v->code_in.slot);
        hit_add(h, rc(a.x, a.y - AT_CODE_BAND, a.w, AT_CODE_BAND), AT_RH_CODE_UP, 0, &dropped);
        hit_add(h, rc(a.x, a.y + a.h, a.w, AT_CODE_BAND), AT_RH_CODE_DOWN, 0, &dropped);
    }
    y = f.y + f.h + 26.0f;
    at_text(s, o, AT_R_BODY14, "Up and down change a letter, left and right move between slots.", P.x + P.w * 0.5f, y, AT_C_MUTED, AT_ALIGN_CENTER, P.w - 48.0f);
    if (msg[0]) {
        float x = f.x, tw = 0.0f;
        if (bad) tw = at_part_tag(s, o, x, y + 16.0f, "FAILED", AT_TAG_ROSE, 0.0f) + 8.0f;
        at_text(s, o, AT_R_BODY14, msg, x + tw, y + 31.0f, bad ? AT_C_IVORY : AT_C_MUTED, AT_ALIGN_LEFT, f.w - tw);
    }
    return dropped;
}

static const char *stage_list_name(const AtRoomView *v) { return v->stage_all ? "All stages" : "Competitive"; }

static int render_wait(const AtRoomView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *h)
{
    AtRect P = L->primary, plate, strip, leave, rules;
    float pad = 12.0f, lw = (P.w - 3.0f * pad) * 0.58f, rx = P.x + pad + lw + pad, y = P.y + pad;
    int dropped = 0, i, tone;
    const char *tag;
    char line[AT_ROOM_TEXT];
    plate = rc(P.x + pad, y, lw, 118.0f);
    e2(s, plate);
    at_text(s, o, AT_R_CAP12, v->random_search ? "SEARCHING" : v->host ? "ROOM CODE" : "JOINING ROOM", plate.x + 14.0f, plate.y + 20.0f, AT_C_MUTED, AT_ALIGN_LEFT, plate.w - 28.0f);
    if (v->random_search) {
        char t[16];
        snprintf(t, sizeof t, "%d:%02d", v->random_secs / 60, v->random_secs % 60);
        at_text(s, o, AT_R_HERO, t, plate.x + 14.0f, plate.y + 75.0f, AT_C_IVORY, AT_ALIGN_LEFT, plate.w - 28.0f);
    } else {
        at_text(s, o, AT_R_DISPLAY, v->code, plate.x + 14.0f, plate.y + 75.0f, AT_C_IVORY, AT_ALIGN_LEFT, plate.w - 28.0f - (v->host ? 100.0f : 0.0f));
    }
    at_text(s, o, AT_R_BODY14, v->random_search ? "Looking for another player." : v->host ? "Share it to invite a friend." : "The host sees you as soon as you are in.",
            plate.x + 14.0f, plate.y + 106.0f, AT_C_MUTED, AT_ALIGN_LEFT, plate.w - (v->host && !v->random_search ? 130.0f : 28.0f));
    if (v->host && !v->random_search) {
        AtRect cb = rc(plate.x + plate.w - 14.0f - 104.0f, plate.y + plate.h - 14.0f - 28.0f, 104.0f, 28.0f);
        button(s, o, cb, AT_C_PLATE, AT_C_EDGE, 'X', v->copied ? "Copied" : "Copy");
        hit_add(h, cb, AT_RH_COPY, 0, &dropped);
    }
    y = plate.y + plate.h + 8.0f;
    for (i = 0; i < 2; i++) {
        AtRect c = rc(P.x + pad, y, lw, 56.0f);
        tag = player_tag(v, i, &tone);
        room_card(s, o, c, &v->pl[i], i, i == v->me, tag, tone);
        y += 62.0f;
    }
    strip = rc(P.x + pad, y, lw, 40.0f);
    e2(s, strip);
    {
        int bars = v->ping_ms < 0 ? 0 : v->link_bars;
        float meter_w = 150.0f;                                                      /* the meter is at most 18 + 6 + the word + 8 + the number: under 150 at 640 */
        snprintf(line, sizeof line, "%s", v->status[0] ? v->status : "Connecting...");
        at_text(s, o, AT_R_BODY14, line, strip.x + 14.0f, strip.y + 25.0f, AT_C_TEXT2, AT_ALIGN_LEFT, strip.w - 28.0f - meter_w);
        at_part_linkmeter(s, o, strip.x + strip.w - 14.0f - meter_w, strip.y + 27.0f, bars, v->ping_ms, 1);
    }
    leave = rc(P.x + pad, P.y + P.h - pad - 44.0f, 190.0f, 44.0f);
    button(s, o, leave, AT_C_PLATE, AT_C_EDGE, 'B', v->random_search ? "Stop" : v->host ? "Close room" : "Cancel");
    hit_add(h, leave, AT_RH_LEAVE, 0, &dropped);
    rules = rc(rx, P.y + pad, P.x + P.w - pad - rx, P.h - 2.0f * pad);
    at_text(s, o, AT_R_CAP12, "ROOM RULES", rules.x + 4.0f, rules.y + 14.0f, AT_C_MUTED, AT_ALIGN_LEFT, rules.w * 0.5f);
    at_text(s, o, AT_R_CAP12, "HOST SETS", rules.x + rules.w - 4.0f, rules.y + 14.0f, AT_C_DIM, AT_ALIGN_RIGHT, rules.w * 0.5f);
    if (!v->rules_known) {
        at_text(s, o, AT_R_BODY14, "Set by the host.", rules.x + 4.0f, rules.y + 44.0f, AT_C_MUTED, AT_ALIGN_LEFT, rules.w - 8.0f);
        at_text(s, o, AT_R_BODY12, "You see the rules in the lobby.", rules.x + 4.0f, rules.y + 62.0f, AT_C_DIM, AT_ALIGN_LEFT, rules.w - 8.0f);
    } else {
        char val[6][24];
        static const char *const names[6] = { "Stage list", "Turbo", "Envoy", "Stocks", "Time limit", "Input delay" };
        snprintf(val[0], 24, "%s", stage_list_name(v));
        snprintf(val[1], 24, "%s", v->turbo ? "On" : "Off");
        snprintf(val[2], 24, "%s", v->envoy ? "On" : "Off");
        snprintf(val[3], 24, "%d", v->stocks);
        snprintf(val[4], 24, "%d min", v->minutes);
        snprintf(val[5], 24, "%d frames", v->delay);
        for (i = 0; i < 6; i++) {
            AtItem it;
            memset(&it, 0, sizeof it);
            snprintf(it.label, sizeof it.label, "%s", names[i]);
            it.vkind = AT_VAL_TEXT;
            it.iflags = AT_ITEM_RO;
            snprintf(it.text, sizeof it.text, "%s", val[i]);
            at_part_row(s, o, rc(rules.x, rules.y + 26.0f + (float) i * 39.0f, rules.w, 34.0f), &it, AT_ST_REST);   /* read-only: never focused */
        }
    }
    return dropped;
}

static int render_lobby(const AtRoomView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *h)
{
    AtRect P = L->primary;
    float pad = 12.0f, cw = (P.w - 2.0f * pad - 8.0f) * 0.5f, y = P.y + pad, gx = P.x + pad, gw = P.w - 2.0f * pad, gy, gh, th, ay;
    int dropped = 0, i, k, tone, cur = at_room_cursor_fix(v), page = 0, rows_used = 1, nh = 0, has_action;
    int deciding = v->phase == AT_PH_STRIKE || v->phase == AT_PH_BAN || v->phase == AT_PH_PICK;   /* tiles can be acted on only now */
    const AtGrid *g = &v->grid;
    char buf[96];
    int counting = v->countdown_s > 0;                                          /* the count replaces the grid: the stage is decided and a plate over live tiles would hide their names */
    for (i = 0; i < 2; i++) {
        const char *tag = player_tag(v, i, &tone);
        room_card(s, o, rc(P.x + pad + (float) i * (cw + 8.0f), y, cw, 56.0f), &v->pl[i], i, i == v->me, tag, tone);
    }
    y += 64.0f;
    /* the coin flip, or whose turn it is */
    if (v->coin_on) {
        snprintf(buf, sizeof buf, "COIN FLIP: P%d STRIKES FIRST", v->first + 1);
        at_part_tag(s, o, gx, y, buf, AT_TAG_SUN, gw * 0.6f);
    } else if (deciding && !v->reconnecting) {
        if (v->my_stage_turn) at_part_tag(s, o, gx, y, "YOUR TURN", AT_TAG_EMBER, gw * 0.4f);
        else { snprintf(buf, sizeof buf, "P%d'S TURN", v->turn + 1); at_part_tag(s, o, gx, y, buf, AT_TAG_PLAIN, gw * 0.4f); }
    } else if (!v->reconnecting) {
        for (i = 0; i < v->n_stages && i < AT_ROOM_STAGES; i++) {
            if (v->st[i].state == AT_STAGE_PICKED) {                                 /* decided: the stage is named where the turn was */
                snprintf(buf, sizeof buf, "STAGE: %s", v->st[i].name);
                at_part_tag(s, o, gx, y, buf, AT_TAG_PLAIN, gw * 0.7f);
                break;
            }
        }
    }
    if (g->pages > 1 && !counting) {
        if (cur >= 0 && cur < g->n) page = g->cell[cur].page;
        snprintf(buf, sizeof buf, "PAGE %d / %d", page + 1, g->pages);
        at_text(s, o, AT_R_CAP12, buf, gx + gw, y + 14.0f, AT_C_MUTED, AT_ALIGN_RIGHT, gw * 0.4f);
    }
    y += 30.0f;
    has_action = (v->can_ready || v->can_pick) && !v->reconnecting;
    ay = P.y + P.h - pad - 40.0f;
    gy = y;
    gh = (has_action ? ay - 8.0f : P.y + P.h - pad) - gy;
    for (k = 0; k < g->nhead; k++) if (g->head[k].page == page) nh++;
    for (i = 0; i < g->n; i++) if (g->cell[i].page == page && g->cell[i].row + 1 > rows_used) rows_used = g->cell[i].row + 1;
    th = (gh - 6.0f * (float) rows_used - 18.0f * (float) nh) / (float) rows_used;
    if (th > 64.0f) th = 64.0f;
    if (th < 30.0f) th = 30.0f;
    for (k = 0, i = 0; !counting && k < g->nhead; k++) {
        const AtGridHead *hd = &g->head[k];
        if (hd->page != page) continue;
        at_text(s, o, AT_R_CAP12, hd->group ? "COUNTERPICKS" : "STARTERS", gx, gy + (float) hd->row * (th + 6.0f) + 18.0f * (float) i + 13.0f, AT_C_MUTED, AT_ALIGN_LEFT, gw * 0.6f);
        at_poly_rect(s, gx, gy + (float) hd->row * (th + 6.0f) + 18.0f * (float) i + 16.0f, gw, 1.0f, AT_C_LINE);
        i++;
    }
    for (i = 0; !counting && i < g->n && i < v->n_stages; i++) {
        const AtGridCell *c = &g->cell[i];
        const AtRoomStage *st = &v->st[i];
        AtCell cell;
        AtRect r;
        int heads_above = 0, focus, dim, stt;
        float tw = (gw - 6.0f * (float) (g->cols - 1)) / (float) g->cols, ty;
        const char *word = NULL;
        if (c->page != page) continue;
        for (k = 0; k < g->nhead; k++) if (g->head[k].page == page && g->head[k].row <= c->row) heads_above++;
        r = rc(gx + (float) c->col * (tw + 6.0f), gy + (float) c->row * (th + 6.0f) + 18.0f * (float) heads_above, tw, th);
        focus = i == cur && v->my_stage_turn && !v->reconnecting;
        dim = v->reconnecting || !deciding || st->state != AT_STAGE_FREE || !st->open;
        stt = focus ? AT_ST_FOCUS : dim ? AT_ST_DISABLED : AT_ST_REST;
        memset(&cell, 0, sizeof cell);
        cell.model = AT_NO_MODEL;
        cell.tex = -1;                                                              /* a zeroed cell must never name texture 0 */
        snprintf(cell.name, sizeof cell.name, "%s", st->name);
        /* step 4's strike marks: bars and BAN for a struck or banned stage (the striker's numeral and shape for a strike), a ring and PICK for the pick */
        if (st->state == AT_STAGE_STRUCK_P1) cell.flags |= AT_CELL_BANNED | AT_CELL_P1;
        else if (st->state == AT_STAGE_STRUCK_P2) cell.flags |= AT_CELL_BANNED;
        else if (st->state == AT_STAGE_BANNED) cell.flags |= AT_CELL_BANNED | AT_CELL_UNSET;
        else if (st->state == AT_STAGE_PICKED) cell.flags |= AT_CELL_PICKED | AT_CELL_UNSET | AT_CELL_SELECTED;
        at_part_cell(s, o, r, &cell, stt, port_rgba(v->me));
        if (st->state == AT_STAGE_FREE) {
            if (!st->open && deciding) word = "NOT YET";
            else if (!deciding && !v->reconnecting) word = "OUT";                   /* decided: the others are out (a word, not only a dim) */
        }
        ty = focus ? r.y - 2.0f : r.y;
        if (word != NULL) at_text(s, o, AT_R_CAP12, word, r.x + 4.0f, ty + 13.0f, AT_C_MUTED, AT_ALIGN_LEFT, r.w - 8.0f);
        hit_add(h, r, AT_RH_STAGE, i, &dropped);
    }
    if (has_action) {
        AtRect a = rc(gx, ay, gw, 40.0f);
        if (v->can_pick) {
            at_plate(s, a, AT_C_EMBER, AT_C_EMBER_D, 3.0f, (float) AT_PX_CH_S);
            at_text(s, o, AT_R_CAP20, "PICK A FIGHTER", a.x + a.w * 0.5f, a.y + 27.0f, AT_C_INK, AT_ALIGN_CENTER, a.w - 24.0f);
            hit_add(h, a, AT_RH_ACTION, 0, &dropped);
        } else if (!v->ready_mine) {
            at_plate(s, a, AT_C_EMBER, AT_C_EMBER_D, 3.0f, (float) AT_PX_CH_S);
            at_text(s, o, AT_R_CAP20, "READY", a.x + a.w * 0.5f, a.y + 27.0f, AT_C_INK, AT_ALIGN_CENTER, a.w - 24.0f);
            hit_add(h, a, AT_RH_ACTION, 0, &dropped);
        } else {
            at_plate(s, a, AT_C_PLATE2, AT_C_JADE, 3.0f, (float) AT_PX_CH_S);
            at_text(s, o, AT_R_CAP20, "READY - WAITING", a.x + a.w * 0.5f, a.y + 27.0f, AT_C_JADE, AT_ALIGN_CENTER, a.w - 24.0f);
        }
    }
    if (v->countdown_s > 0) {
        AtRect c = rc(P.x + P.w * 0.5f - 80.0f, P.y + P.h * 0.5f - 60.0f, 160.0f, 120.0f);
        at_plate(s, c, AT_C_PLATE, AT_C_EDGE, 3.0f, (float) AT_PX_CH);
        snprintf(buf, sizeof buf, "%d", v->countdown_s);
        at_text(s, o, AT_R_DISPLAY, buf, c.x + c.w * 0.5f, c.y + 72.0f, AT_C_IVORY, AT_ALIGN_CENTER, c.w - 16.0f);
        if (v->ready_mine) at_text(s, o, AT_R_CAP12, "B CANCEL", c.x + c.w * 0.5f, c.y + 104.0f, AT_C_DIM, AT_ALIGN_CENTER, c.w - 16.0f);
    }
    return dropped;
}

int at_room_render(const AtRoomView *v, const AtLayout *L, const AtTextOps *o, const AtSink *s, AtHits *hits, double now_ms)
{
    (void) now_ms;
    if (v == NULL) return 0;
    switch (v->kind) {
    case AT_ROOM_CODE: return render_code(v, L, o, s, hits);
    case AT_ROOM_WAIT: return render_wait(v, L, o, s, hits);
    case AT_ROOM_LOBBY: return render_lobby(v, L, o, s, hits);
    default: return 0;
    }
}

/* ---- input ---------------------------------------------------------------------------------------- */
#define PUT(K, A) do { if (n < cap) { out[n].kind = (K); out[n].arg = (A); n++; } } while (0)

int at_room_takes_keys(const AtRoomView *v) { return v != NULL && v->kind != AT_ROOM_CODE; }

int at_room_key_intents(AtKeys *k, unsigned mask, double now_ms, AtRoomIntent *out, int cap)
{
    AtEvent ev[8];
    int ne = at_key_events(k, mask, now_ms, ev, 8), i, n = 0;
    for (i = 0; i < ne; i++) {
        switch (ev[i].type) {
        case AT_EV_MOVE: PUT(ev[i].a == AT_DIR_UP ? AT_RI_UP : ev[i].a == AT_DIR_DOWN ? AT_RI_DOWN : ev[i].a == AT_DIR_LEFT ? AT_RI_LEFT : AT_RI_RIGHT, 0); break;
        case AT_EV_ACCEPT: PUT(AT_RI_ACCEPT, 0); break;
        case AT_EV_BACK: PUT(AT_RI_BACK, 0); break;
        case AT_EV_PAGE: PUT(ev[i].a < 0 ? AT_RI_PAGE_L : AT_RI_PAGE_R, 0); break;
        default: break;
        }
    }
    return n;
}

static int button_intent(int c)
{
    switch (c) {
    case 'A': return AT_RI_ACCEPT;
    case 'B': return AT_RI_BACK;
    case 'X': return AT_RI_COPY;
    case 'Y': return AT_RI_PASTE;
    case 'S': return AT_RI_START;
    case 'L': return AT_RI_PAGE_L;
    case 'R': return AT_RI_PAGE_R;
    default: return AT_RI_NONE;
    }
}

int at_room_mouse_intents(const AtRoomView *v, AtMouse *m, float x, float y, int buttons, int wheel, const AtHits *hits, AtRoomIntent *out, int cap)
{
    int n = 0, hit, moved, left, right;
    if (x < 0.0f || y < 0.0f) {                              /* off the picture (-1000): never an intent */
        m->valid = 0;
        m->buttons = buttons;
        return 0;
    }
    moved = m->valid && (x != m->x || y != m->y);            /* a pointer at rest never acts */
    left = (buttons & 1) && !(m->buttons & 1) && m->valid;
    right = (buttons & 2) && !(m->buttons & 2) && m->valid;
    hit = at_hit_test(hits, x, y);
    if (hit >= 0 && moved && hits->h[hit].kind == AT_HIT_ROOM && hits->h[hit].a == AT_RH_STAGE && v->my_stage_turn &&
        hits->h[hit].b >= 0 && hits->h[hit].b < v->n_stages && v->st[hits->h[hit].b].open)
        PUT(AT_RI_STAGE_AT, hits->h[hit].b);                  /* hover moves the cursor onto an open stage, nothing more */
    if (left && hit >= 0) {
        const AtHit *h = &hits->h[hit];
        if (h->kind == AT_HIT_KEY) {
            int b = button_intent(h->a);
            if (b != AT_RI_NONE) PUT(b, 0);
        } else if (h->kind == AT_HIT_ROOM) {
            switch (h->a) {
            case AT_RH_STAGE:                                  /* cursor there AND confirm, as ONE intent; only on my stage turn (a tile click in the ready phase must not ready me) and only on an open stage (a click on a closed one must not confirm another) */
                if (v->my_stage_turn && h->b >= 0 && h->b < v->n_stages && v->st[h->b].open) PUT(AT_RI_STAGE_CLICK, h->b);
                break;
            case AT_RH_ACTION: PUT(AT_RI_ACCEPT, 0); break;
            case AT_RH_LEAVE: PUT(AT_RI_BACK, 0); break;
            case AT_RH_COPY: PUT(AT_RI_COPY, 0); break;
            case AT_RH_CODE_SLOT: PUT(AT_RI_CODE_SLOT, h->b); break;
            case AT_RH_CODE_UP: PUT(AT_RI_UP, 0); break;
            case AT_RH_CODE_DOWN: PUT(AT_RI_DOWN, 0); break;
            default: break;
            }
        }
    }
    if (right) PUT(AT_RI_BACK, 0);
    if (wheel != 0) {
        if (v->kind == AT_ROOM_CODE) PUT(wheel > 0 ? AT_RI_UP : AT_RI_DOWN, 0);
        else if (v->kind == AT_ROOM_LOBBY && v->my_stage_turn) PUT(wheel > 0 ? AT_RI_PAGE_L : AT_RI_PAGE_R, 0);
    }
    m->valid = 1; m->x = x; m->y = y; m->buttons = buttons;
    return n;
}
#undef PUT

const char *at_room_intent_name(int kind)
{
    static const char *const names[] = { "none", "up", "down", "left", "right", "accept", "back", "start", "copy", "paste", "page_l", "page_r", "stage_at", "code_slot", "stage_click" };
    return kind >= 0 && kind < (int) (sizeof names / sizeof names[0]) ? names[kind] : "?";
}
