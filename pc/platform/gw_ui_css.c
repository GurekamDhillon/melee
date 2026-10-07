/* gw_ui_css.c - the character select's rules, moved out of gmfrontend_select.inc into pure C (see gw_ui_css.h).
 * Each rule below names the legacy function it comes from; where the legacy code and a profile disagree the profile only ever removes a
 * capability (no CPU cards, no joining, a different minimum), it never changes what a retained rule does. */
#include "gw_ui_css.h"

#include <stdio.h>
#include <string.h>

/* ---- small helpers ---------------------------------------------------------------------------------- */

static int ncost(const AtCss *c, int ck)                 /* fs_costumes: at least 1; a Random or empty pick has one */
{
    int n = (ck >= 0 && c->ops != NULL && c->ops->costumes != NULL) ? c->ops->costumes(c->ops->user, ck) : 1;
    return n > 0 ? n : 1;
}

static int swap_of(const AtCss *c, int ck) { return (ck >= 0 && c->ops != NULL && c->ops->zelda_swap != NULL) ? c->ops->zelda_swap(c->ops->user, ck) : -1; }

/* fs_sheik_shares_zelda, for the tile ck: it has a partner (Zelda or Sheik) and the roster does not show the partner on a tile of its own */
static int shares(const AtCss *c, int ck)
{
    int partner = swap_of(c, ck), i;
    if (partner < 0) return 0;
    for (i = 0; i < c->n_slots - 1; i++) if (c->ck[i] == partner) return 0;
    return 1;
}

static int sheik_ok(const AtCss *c)                      /* fs_sheik_ok: only online does the opponent's roster matter */
{
    return !c->online || c->ops == NULL || c->ops->sheik_ok == NULL || c->ops->sheik_ok(c->ops->user);
}

static void toast(AtCss *c, const char *s) { snprintf(c->toast, sizeof c->toast, "%s", s); }

static int slot_of_vis(const AtCss *c, int vis_index)    /* the roster slot a visible index shows, or -1 */
{
    return vis_index >= 0 && vis_index < c->n_vis ? c->vis[vis_index] : -1;
}

static void rebuild_vis(AtCss *c)
{
    int s, n = 0;
    for (s = 0; s < c->n_slots; s++)
        if (c->tab == 0 || c->tab_of[s] == 0 || c->tab_of[s] == c->tab) c->vis[n++] = s;
    c->n_vis = n;
}

static void clamp_cursors(AtCss *c)
{
    int p;
    for (p = 0; p < 4; p++) if (c->p[p].cur < 0 || c->p[p].cur >= c->n_vis) c->p[p].cur = 0;
}

/* ---- opening and the roster ----------------------------------------------------------------------- */

void at_css_set_entering(AtCss *c, int port)
{
    int p;
    if (port < 0 || port > 3) return;
    for (p = 0; p < 4; p++) {
        c->p[p].kind = p == port ? AT_CSS_HMN : AT_CSS_OFF;
        c->p[p].target = p;
        c->p[p].card = -1;
        if (p != port) { c->p[p].ck = AT_CK_NONE; c->p[p].costume = 0; }
    }
}

/* Training (the retail Training setup and exit handler): the port that entered is the human, the other of ports 0 and 1 the CPU dummy; its exit handler reads
 * exactly those two slots (fe_sel_training in the legacy open). */
void at_css_set_training(AtCss *c, int human)
{
    int p, h = human >= 0 && human <= 3 ? human : 0, d = h == 0 ? 1 : 0;
    c->train_h = h;
    c->train_c = d;
    for (p = 0; p < 4; p++) {
        AtCssPort *s = &c->p[p];
        s->card = -1;
        s->target = p;
        s->kind = p == h ? AT_CSS_HMN : p == d ? AT_CSS_CPU : AT_CSS_OFF;
        if (s->kind == AT_CSS_OFF) { s->ck = AT_CK_NONE; s->costume = 0; }
        else if (s->kind == AT_CSS_CPU && s->ck < 0) s->ck = AT_CK_RANDOM;        /* the dummy is Random until its fighter is picked */
    }
}

void at_css_open(AtCss *c, const AtCssProfile *prof, const AtCssOps *ops, int online, int teams)
{
    int p;
    memset(c, 0, sizeof *c);
    c->prof = prof;
    c->ops = ops;
    c->online = online != 0;
    c->teams = teams != 0 && !c->online;                       /* fcs.teams: the rules' teams, never online */
    for (p = 0; p < 4; p++) {
        AtCssPort *s = &c->p[p];
        s->kind = AT_CSS_OFF;
        s->ck = AT_CK_NONE;
        s->cpu_lv = 9;
        s->team = p & 1;
        s->card = -1;
        s->target = p;
    }
    c->p[0].kind = AT_CSS_HMN;                                 /* whoever came here is P1 */
    c->train_h = 0; c->train_c = 1;
    if (prof != NULL && prof->dummy_cpu) at_css_set_training(c, 0);
    else if (prof != NULL && prof->entering_port_only) at_css_set_entering(c, 0);
}

void at_css_set_roster(AtCss *c, int n, const int *ck, const unsigned char *ok, const unsigned char *tab_of)
{
    int i;
    if (n < 0 || ck == NULL) n = 0;
    if (n > AT_CSS_MAX_SLOTS - 1) n = AT_CSS_MAX_SLOTS - 1;
    for (i = 0; i < n; i++) {
        c->ck[i] = ck[i];
        c->ok[i] = ok == NULL ? 1 : (ok[i] != 0);
        c->tab_of[i] = tab_of != NULL && tab_of[i] == 2 ? 2 : 1;
    }
    c->ck[n] = AT_CK_RANDOM; c->ok[n] = 1; c->tab_of[n] = 0;   /* the Random tile ends every tab */
    c->n_slots = n + 1;
    c->tab = 0;
    rebuild_vis(c);
    clamp_cursors(c);
}

void at_css_set_cols(AtCss *c, int cols) { c->cols = cols > 0 ? cols : 1; }

int at_css_tab_count(const AtCss *c, int tab)
{
    int i, n = 0;
    for (i = 0; i < c->n_slots - 1; i++) if (tab == 0 || c->tab_of[i] == tab) n++;
    return n;
}

int at_css_tab_offered(const AtCss *c, int tab)
{
    if (tab == 0) return 1;
    if (tab == 2) return at_css_tab_count(c, 2) > 0;
    if (tab == 1) return at_css_tab_count(c, 1) > 0 && at_css_tab_count(c, 2) > 0;
    return 0;
}

void at_css_set_tab(AtCss *c, int tab)
{
    int old_slot[4], p, i;
    if (tab < 0 || tab > 2) tab = 0;
    for (p = 0; p < 4; p++) { old_slot[p] = slot_of_vis(c, c->p[p].cur); if (old_slot[p] < 0) old_slot[p] = 0; }
    c->tab = tab;
    rebuild_vis(c);
    for (p = 0; p < 4; p++) {                                  /* the same tile when it is still there, else the nearest slot that is */
        int best = 0, best_d = 1 << 30;
        for (i = 0; i < c->n_vis; i++) {
            int d = c->vis[i] - old_slot[p], last = c->vis[i] == c->n_slots - 1;
            if (d < 0) d = -d;
            if (last && c->vis[i] != old_slot[p]) d += 1000;    /* Random only when no fighter is a better stand-in */
            if (d < best_d) { best_d = d; best = i; }
        }
        c->p[p].cur = best;
    }
}

int at_css_vis_of_ck(const AtCss *c, int ck)
{
    int s = -1, i;
    if (ck >= 0) {
        for (i = 0; i < c->n_slots - 1 && s < 0; i++) if (c->ck[i] == ck) s = i;
        if (s < 0) {                                           /* Sheik lives on Zelda's tile when the roster has no tile of her own */
            int partner = swap_of(c, ck);
            if (partner >= 0 && shares(c, partner)) for (i = 0; i < c->n_slots - 1 && s < 0; i++) if (c->ck[i] == partner) s = i;
        }
    }
    if (s < 0) return 0;
    for (i = 0; i < c->n_vis; i++) if (c->vis[i] == s) return i;
    return 0;
}

/* ---- the rules ------------------------------------------------------------------------------------- */

int at_css_count_in(const AtCss *c)
{
    int p, k = 0;
    for (p = 0; p < 4; p++) k += c->p[p].kind != AT_CSS_OFF && c->p[p].ck != AT_CK_NONE;
    return k;
}

/* fs_css_blocker: who is still missing, or NULL when the match can start. min_to_start replaces (online ? 1 : 2). */
const char *at_css_blocker(const AtCss *c, char *buf, int cap)
{
    int p, min = c->prof != NULL ? c->prof->min_to_start : 2;
    if (cap < 1) return "";
    for (p = 0; p < 4; p++) {
        if (c->p[p].kind == AT_CSS_HMN && c->p[p].ck == AT_CK_NONE) {
            snprintf(buf, (size_t) cap, "P%d: pick a fighter.", p + 1);
            return buf;
        }
    }
    if (at_css_count_in(c) < min) {
        snprintf(buf, (size_t) cap, "%s", min >= 2 ? "Two fighters needed: pick one, or add a CPU below." : "Pick a fighter.");
        return buf;
    }
    return NULL;
}

/* fs_free_costume: the first costume of `ck` nobody else in the match wears (from `from`, stepping by dir) */
int at_css_free_costume(const AtCss *c, int port, int ck, int from, int dir)
{
    int n = ncost(c, ck), k, q;
    for (k = 0; k < n; k++) {
        int taken = 0, cs = ((from + dir * k) % n + n) % n;
        for (q = 0; q < 4; q++)
            if (q != port && c->p[q].kind != AT_CSS_OFF && c->p[q].ck == ck && c->p[q].costume == cs) taken = 1;
        if (!taken) return cs;
    }
    return from;
}

/* fs_pick: the slot under the cursor for the port this cursor picks for */
static unsigned pick(AtCss *c, int port, int slot)
{
    AtCssPort *me = &c->p[port], *t = &c->p[me->target];
    int ck, partner;
    if (slot < 0 || slot >= c->n_slots) return 0;
    ck = c->ck[slot];
    if (!c->ok[slot]) {
        toast(c, "Your opponent doesn't have this fighter - pick another.");
        return AT_CE_BACK | AT_CE_TOAST;
    }
    partner = swap_of(c, ck);
    if (partner >= 0 && shares(c, ck) && (t->ck == ck || t->ck == partner)) {
        /* A on the Zelda tile again: swap Zelda and Sheik */
        if (t->ck == ck && !sheik_ok(c)) {
            toast(c, "Your opponent doesn't have Sheik.");
            return AT_CE_BACK | AT_CE_TOAST;
        }
        t->ck = t->ck == ck ? partner : ck;
        t->costume = at_css_free_costume(c, me->target, t->ck, t->costume < ncost(c, t->ck) ? t->costume : 0, 1);
        if (me->target != port) me->target = port;
        return AT_CE_FORWARD;
    }
    t->ck = ck;
    t->costume = ck >= 0 ? at_css_free_costume(c, me->target, ck, 0, 1) : 0;
    if (me->target != port) me->target = port;                 /* the CPU has its fighter: back to picking for yourself */
    return AT_CE_FORWARD;
}

/* the legacy "on the cards" block, line for line (left/right cycle four cards, up returns to the bottom row under the card, A adds or picks a
 * CPU, X/Y step a CPU's level or your costume, Z removes a CPU or leaves, B returns to the grid) */
static unsigned card_step(AtCss *c, int port, AtCssIn in)
{
    AtCssPort *s = &c->p[port], *k;
    unsigned ev = 0;
    int cols = c->cols > 0 ? c->cols : 1, n = c->n_vis, can_add = c->prof != NULL && c->prof->cpu_cards && !c->prof->dummy_cpu;
    if (s->card < 0 || s->card > 3) { s->card = -1; return 0; }
    k = &c->p[s->card];
    if (in.rep & AT_CI_LEFT) { s->card = (s->card + 3) % 4; ev |= AT_CE_MOVE; }
    else if (in.rep & AT_CI_RIGHT) { s->card = (s->card + 1) % 4; ev |= AT_CE_MOVE; }
    else if (in.rep & AT_CI_UP) {
        int last_row = (n - 1) / cols;
        s->cur = last_row * cols + s->card * cols / 4;
        if (s->cur >= n) s->cur = n - 1;
        s->card = -1;
        ev |= AT_CE_MOVE;
    } else if (in.trig & AT_CI_A) {
        if (s->card == port) {
            s->card = -1;                                      /* your own card: back to the grid */
        } else if (k->kind == AT_CSS_OFF && can_add) {
            k->kind = AT_CSS_CPU;                              /* add a CPU, and pick its fighter now */
            k->ck = AT_CK_RANDOM;
            k->cpu_lv = 9;
            s->target = s->card;
            s->card = -1;
            ev |= AT_CE_FORWARD;
        } else if (k->kind == AT_CSS_CPU) {
            s->target = s->card;                               /* pick this CPU's fighter */
            s->card = -1;
            ev |= AT_CE_FORWARD;
        }
    } else if (in.trig & (AT_CI_X | AT_CI_Y)) {
        int dir = (in.trig & AT_CI_X) ? 1 : -1;
        if (k->kind == AT_CSS_CPU) {
            k->cpu_lv = k->cpu_lv + dir < 1 ? 9 : k->cpu_lv + dir > 9 ? 1 : k->cpu_lv + dir;
            ev |= AT_CE_MOVE;
        } else if (s->card == port && s->ck >= 0) {
            s->costume = at_css_free_costume(c, port, s->ck, s->costume + dir, dir);
            ev |= AT_CE_MOVE;
        }
    } else if ((in.trig & AT_CI_Z) && !c->prof->dummy_cpu) {
        if (k->kind == AT_CSS_CPU && s->card != port) {
            k->kind = AT_CSS_OFF;
            k->ck = AT_CK_NONE;
            ev |= AT_CE_BACK;
        } else if (s->card == port && port != 0 && c->prof->max_humans > 1) {
            s->kind = AT_CSS_OFF;                              /* leave */
            s->ck = AT_CK_NONE;
            s->card = -1;
            ev |= AT_CE_BACK;
        }
    } else if (in.trig & AT_CI_B) {
        s->card = -1;
        ev |= AT_CE_BACK;
    }
    if (c->teams && (in.trig & AT_CI_Y) && s->card == port && s->ck < 0) s->team = (s->team + 1) % 3;
    return ev;
}

/* the legacy page turn on L and R: here the tabs (the pages became tabs and scrolling) */
static int step_tab(AtCss *c, int dir)
{
    int t = c->tab, k;
    for (k = 0; k < 3; k++) {
        t = (t + dir + 3) % 3;
        if (at_css_tab_offered(c, t)) break;
    }
    if (t == c->tab || !at_css_tab_offered(c, t)) return 0;
    at_css_set_tab(c, t);
    return 1;
}

unsigned at_css_step(AtCss *c, int port, AtCssIn in, int frame)
{
    AtCssPort *s;
    unsigned ev = 0;
    int cols, n, row, col, last_row, cur;
    if (port < 0 || port > 3) return 0;
    if (c->done || c->n_vis <= 0 || c->prof == NULL) return 0;
    if (c->online && port != 0) return 0;                       /* the adapter folds the other controllers into port 0 */
    if (c->prof->dummy_cpu && port != c->train_h) return 0;     /* Training: the human's input only */
    s = &c->p[port];
    cols = c->cols > 0 ? c->cols : 1;
    n = c->n_vis;
    if (s->kind != AT_CSS_HMN) {
        if (!c->online && c->prof->max_humans > 1 && (in.trig & (AT_CI_A | AT_CI_START))) {
            s->kind = AT_CSS_HMN; s->target = port; s->card = -1; ev |= AT_CE_FORWARD;   /* a controller joins (a CPU in its port becomes this player) */
        }
        return ev;
    }
    if (in.trig & AT_CI_L) { if (step_tab(c, -1)) ev |= AT_CE_MOVE; }
    else if (in.trig & AT_CI_R) { if (step_tab(c, 1)) ev |= AT_CE_MOVE; }
    n = c->n_vis;
    if (s->cur < 0 || s->cur >= n) s->cur = 0;
    if (s->target < 0 || s->target > 3 || (s->target != port && c->p[s->target].kind != AT_CSS_CPU)) s->target = port;   /* a CPU someone was picking for went away */
    if (s->card < 0) {
        row = s->cur / cols; col = s->cur % cols; last_row = (n - 1) / cols;
        if (in.rep & AT_CI_LEFT) col = col == 0 ? cols - 1 : col - 1;
        else if (in.rep & AT_CI_RIGHT) col = col == cols - 1 ? 0 : col + 1;
        else if (in.rep & AT_CI_UP) row = row == 0 ? last_row : row - 1;
        else if (in.rep & AT_CI_DOWN) {
            if (row + 1 > last_row || (row + 1) * cols + col >= n) {
                if (!c->online) {                               /* down off the grid: onto the cards (a one-player mode has only its own) */
                    s->card = (c->prof->max_humans == 1 && !c->prof->cpu_cards && !c->prof->dummy_cpu) ? port : col * 4 / cols;
                    return ev | AT_CE_MOVE;
                }
                row = 0;
            } else row++;
        }
        if (row * cols + col >= n) {
            col = n - 1 - row * cols;
            if (col < 0) { row = 0; col = 0; }
        }
        cur = row * cols + col;
        if (cur != s->cur) ev |= AT_CE_MOVE;
        s->cur = cur;
        if (in.trig & AT_CI_A) ev |= pick(c, port, c->vis[s->cur]);
        else if (in.trig & (AT_CI_X | AT_CI_Y)) {
            AtCssPort *t = &c->p[s->target];
            if (t->ck >= 0) {
                int dir = (in.trig & AT_CI_X) ? 1 : -1;
                t->costume = at_css_free_costume(c, s->target, t->ck, t->costume + dir, dir);
                ev |= AT_CE_MOVE;
            }
        } else if (in.trig & AT_CI_B) {
            ev |= AT_CE_BACK;
            if (s->target != port) s->target = port;            /* stop picking for that CPU */
            else if (s->ck != AT_CK_NONE) s->ck = AT_CK_NONE;   /* undo */
            else if (frame < c->back_until) { ev |= AT_CE_FINISH_BACK; c->done = 1; return ev; }
            else { c->back_until = frame + 150; toast(c, "Press B again to go back."); ev |= AT_CE_TOAST; }
        }
    } else {
        ev |= card_step(c, port, in);
    }
    if ((in.trig & AT_CI_START) && !c->done) {
        char buf[96];
        const char *why = at_css_blocker(c, buf, sizeof buf);
        if (why == NULL) { ev |= AT_CE_FORWARD | AT_CE_FINISH_GO; c->done = 1; }
        else { toast(c, why); ev |= AT_CE_BACK | AT_CE_TOAST; }
    }
    return ev;
}

/* ---- the mouse ---------------------------------------------------------------------------------------- */

/* fs_mouse_port: online the one card, in Training the human, else the first human port */
int at_css_mouse_port(const AtCss *c)
{
    int p;
    if (c->online) return 0;
    if (c->prof != NULL && c->prof->dummy_cpu) return c->train_h;
    for (p = 0; p < 4; p++) if (c->p[p].kind == AT_CSS_HMN) return p;
    return 0;
}

unsigned at_css_mouse_bits(AtCss *c, int port, int vis_slot, int card, int moved, int click, int rclick, int wheel)
{
    AtCssPort *s;
    unsigned b = rclick ? AT_CI_B : 0u;                         /* fms_pad_bits: a right click is B */
    int dummy;
    if (port < 0 || port > 3 || c->prof == NULL) return b;
    s = &c->p[port];
    dummy = c->prof->dummy_cpu;
    if (vis_slot >= c->n_vis) vis_slot = -1;
    if (card > 3 || (c->online && card > 0)) card = -1;
    if (s->kind != AT_CSS_HMN) {
        if (click && (vis_slot >= 0 || card == port)) b |= AT_CI_A;   /* join */
        return b;
    }
    if (vis_slot >= 0 && (moved || click)) { s->card = -1; s->cur = vis_slot; }
    if (vis_slot >= 0 && click) b |= AT_CI_A;
    if (card >= 0 && !c->online && !dummy && (moved || click) && s->card != card) s->card = card;
    if (card >= 0 && click) {
        if (card == port || c->online || dummy) { if (s->target >= 0 && s->target < 4 && c->p[s->target].ck >= 0) b |= AT_CI_X; }   /* your card: the next costume */
        else b |= AT_CI_A;
    }
    if (card >= 0 && rclick && card != port && c->p[card].kind == AT_CSS_CPU && !dummy) b = (b & ~AT_CI_B) | AT_CI_Z;
    if (wheel != 0) {
        if (card >= 0) b |= wheel > 0 ? AT_CI_X : AT_CI_Y;
        else {                                                  /* over the grid: the legacy page turn, here the tab */
            int t, offered = 0;
            for (t = 0; t < 3; t++) offered += at_css_tab_offered(c, t);
            if (offered > 1) b |= wheel > 0 ? AT_CI_L : AT_CI_R;
        }
    }
    return b;
}
