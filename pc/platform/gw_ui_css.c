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


/* ---- puck mode helpers ------------------------------------------------------------------------------ */

/* a hand's start: a human with no fighter stands on its own panel at the puck (Melee: the hand starts over the coin); a port that already
 * has its fighter (back from the stage select) stands on that tile with the puck lying there */
void at_css_hand_reset(AtCss *c, int p)
{
    AtCssPort *s;
    if (p < 0 || p > 3) return;
    s = &c->p[p];
    s->hold = 0; s->zone = AT_ZONE_PUCK; s->from_ck = AT_CK_NONE;
    s->act = s->kind == AT_CSS_HMN;
    s->blank = 0; s->drag = -1; s->val = 0;
    if (c->puck && s->kind == AT_CSS_HMN && s->ck == AT_CK_NONE) s->card = p;
    else s->card = -1;
}

static int puck_slot_vis(const AtCss *c, int ck)         /* the visible index of the tile that shows ck (Random: the last tile), or -1 */
{
    int i, s = -1;
    if (ck == AT_CK_RANDOM) s = c->n_slots - 1;
    else if (ck >= 0) {
        for (i = 0; i < c->n_slots - 1 && s < 0; i++) if (c->ck[i] == ck) s = i;
        if (s < 0) {
            int partner = swap_of(c, ck);
            if (partner >= 0 && shares(c, partner)) for (i = 0; i < c->n_slots - 1 && s < 0; i++) if (c->ck[i] == partner) s = i;
        }
    }
    if (s < 0) return -1;
    for (i = 0; i < c->n_vis; i++) if (c->vis[i] == s) return i;
    return -1;
}

static int held_by(const AtCss *c, int k)                /* the hand carrying port k's puck, or -1 */
{
    int h;
    for (h = 0; h < 4; h++) if (c->p[h].hold && c->p[h].target == k) return h;
    return -1;
}

int at_css_puck_vis(const AtCss *c, int p)
{
    if (p < 0 || p > 3 || c->p[p].kind == AT_CSS_OFF || c->p[p].ck == AT_CK_NONE || held_by(c, p) >= 0) return -1;
    return puck_slot_vis(c, c->p[p].ck);
}

int at_css_zone_ok(const AtCss *c, int port, int k, int zone)
{
    const AtCssPort *kk;
    int can_add, mine;
    if (c->prof == NULL || k < 0 || k > 3 || port < 0 || port > 3) return 0;
    kk = &c->p[k];
    can_add = c->prof->cpu_cards && !c->prof->dummy_cpu;
    mine = k == port;
    switch (zone) {
    case AT_ZONE_PUCK: return kk->kind == AT_CSS_CPU || (kk->kind == AT_CSS_HMN && mine);
    case AT_ZONE_TYPE: return can_add;                                       /* the type button: every panel (retail's togglebtn is on each door, whoever's hand is there) */
    case AT_ZONE_LEVEL: return kk->kind == AT_CSS_CPU;
    case AT_ZONE_COSTUME: return (kk->kind == AT_CSS_CPU || (kk->kind == AT_CSS_HMN && mine)) && kk->ck >= 0;
    case AT_ZONE_TEAM: return c->teams && (kk->kind == AT_CSS_CPU || (kk->kind == AT_CSS_HMN && mine));
    case AT_ZONE_CONTROLS: return mine && kk->kind == AT_CSS_HMN && kk->ck >= 0 && !kk->hold && held_by(c, k) < 0;
    default: return 0;
    }
}

#include "gw_ui_css_controls.inc"

static int first_zone(const AtCss *c, int port, int k)   /* the topmost zone of card k the hand can stand on, or -1 */
{
    int z;
    for (z = 0; z < AT_ZONE_N; z++) if (at_css_zone_ok(c, port, k, z)) return z;
    return -1;
}

void at_css_set_pad(AtCss *c, int p, int ok) { if (p >= 0 && p < 4) c->pad_ok[p] = ok != 0; }

int at_css_ready(const AtCss *c)
{
    char buf[96];
    int h;
    if (!c->puck) return 0;
    for (h = 0; h < 4; h++) if (c->p[h].hold) return 0;
    return at_css_blocker(c, buf, sizeof buf) == NULL;
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
    c->controls_port = -1;
    c->controls_block_frame = -1;
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
    c->puck = !c->online;
    for (p = 0; p < 4; p++) c->pad_ok[p] = 1;
    c->free = 0;
    c->rng = 0x9E3779B9u;
    c->train_h = 0; c->train_c = 1;
    if (prof != NULL && prof->dummy_cpu) at_css_set_training(c, 0);
    else if (prof != NULL && prof->entering_port_only) at_css_set_entering(c, 0);
    for (p = 0; p < 4; p++) at_css_hand_reset(c, p);
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
    if (c->controls_port >= 0) { snprintf(buf, (size_t) cap, "Close Controls to start."); return buf; }
    for (p = 0; p < 4; p++) {
        if (c->p[p].kind == AT_CSS_HMN && c->p[p].ck == AT_CK_NONE) {
            if (c->puck) snprintf(buf, (size_t) cap, "P%d: pick up your puck and drop it on a fighter.", p + 1);
            else snprintf(buf, (size_t) cap, "P%d: pick a fighter.", p + 1);
            return buf;
        }
    }
    if (c->puck) {
        for (p = 0; p < 4; p++) {
            if (c->p[p].kind == AT_CSS_CPU && c->p[p].ck == AT_CK_NONE) {
                snprintf(buf, (size_t) cap, "P%d CPU has no fighter: pick up its puck and drop it on one.", p + 1);
                return buf;
            }
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
    if (c->puck && me->from_ck != AT_CK_NONE && partner >= 0 && shares(c, ck) && (me->from_ck == ck || me->from_ck == partner)) {
        int want = me->from_ck == ck ? partner : ck;           /* a puck lifted from Zelda's tile and set down on it again turns into Sheik, and back */
        if (want == partner && !sheik_ok(c)) {
            toast(c, "Your opponent doesn't have Sheik.");
            return AT_CE_BACK | AT_CE_TOAST;
        }
        ck = want;
        partner = swap_of(c, ck);
    }
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
    if (c->puck) t->costume = ck >= 0 ? at_css_free_costume(c, me->target, ck, t->costume < ncost(c, ck) ? t->costume : 0, 1) : 0;   /* the puck keeps its costume wherever it lands */
    else t->costume = ck >= 0 ? at_css_free_costume(c, me->target, ck, 0, 1) : 0;
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


/* ---- puck mode: Melee's own rules ---------------------------------------------------------------------
 * Each port has a puck. It lives docked on its panel, in a hand, or set down on a tile. A hand (a human's cursor) picks a puck up with A on it (on the
 * panel, or on the tile it lies on), carries it over the roster, and sets it down with A, which is the pick (fs_pick: a CPU's puck picks the CPU's
 * fighter). B takes a set-down puck back into the hand, then docks it. A panel has parts a hand moves between with Up and Down: the puck, the type
 * (CPU or not), the CPU level slider, the costume, the team. mncharsel.c is the reference for each. */

static void shift_costume(AtCss *c, int k, int dir)
{
    AtCssPort *t = &c->p[k];
    if (t->ck >= 0) t->costume = at_css_free_costume(c, k, t->ck, t->costume + dir, dir);
}

static void puck_grab(AtCss *c, int port, int k)
{
    AtCssPort *s = &c->p[port], *t = &c->p[k];
    int v = at_css_puck_vis(c, k);
    s->hold = 1; s->target = k; s->from_ck = t->ck; s->card = -1;
    if (v >= 0) s->cur = v;
    t->ck = AT_CK_NONE;
}

static void puck_dock(AtCss *c, int port)
{
    AtCssPort *s = &c->p[port];
    int k = s->target;
    s->hold = 0; s->from_ck = AT_CK_NONE;
    s->target = port;
    s->card = k >= 0 && k <= 3 ? k : port;
    s->zone = first_zone(c, port, s->card);
    if (s->zone < 0) { s->zone = AT_ZONE_PUCK; s->card = port; }
}

static int next_card(const AtCss *c, int port, int from, int dir)    /* the next card with something to stand on, wrapping; -1 when none */
{
    int i, k = from;
    for (i = 0; i < 4; i++) {
        k = (k + dir + 4) % 4;
        if (first_zone(c, port, k) >= 0) return k;
    }
    return -1;
}

static void card_to(AtCss *c, int port, int k)                       /* the hand moves to card k, keeping its zone when the card has it */
{
    AtCssPort *s = &c->p[port];
    int z = at_css_zone_ok(c, port, k, s->zone) ? s->zone : first_zone(c, port, k);
    s->card = k;
    s->zone = z < 0 ? AT_ZONE_PUCK : z;
}

/* Where each part of a panel sits (the screen draws them there): the team button and the type button over the portrait's top corners (left, right),
 * the puck's dock in the column beside the portrait, the costume stepper under the name, the CPU level slider at the bottom. A direction moves the
 * hand to the part that lies that way, the way Melee's free hand would reach it. */
enum { NAV_GRID = -1, NAV_STAY = -2, NAV_PREV = -3, NAV_NEXT = -4 };
enum { DIR_LEFT, DIR_RIGHT, DIR_UP, DIR_DOWN };

static int pick_zone(const AtCss *c, int port, int k, const int *list, int n)
{
    int i;
    for (i = 0; i < n; i++) if (at_css_zone_ok(c, port, k, list[i])) return list[i];
    return -1;
}

static int nav_zone(const AtCss *c, int port, int k, int z, int dir)
{
    static const int l_team_type[] = { AT_ZONE_TEAM, AT_ZONE_TYPE }, l_type[] = { AT_ZONE_TYPE }, l_team_puck[] = { AT_ZONE_TEAM, AT_ZONE_PUCK };
    static const int l_down_top[] = { AT_ZONE_COSTUME, AT_ZONE_LEVEL, AT_ZONE_PUCK }, l_cost_level[] = { AT_ZONE_COSTUME, AT_ZONE_LEVEL }, l_level[] = { AT_ZONE_LEVEL, AT_ZONE_CONTROLS };
    static const int l_up_cost[] = { AT_ZONE_PUCK, AT_ZONE_TEAM, AT_ZONE_TYPE }, l_up_level[] = { AT_ZONE_COSTUME, AT_ZONE_PUCK, AT_ZONE_TEAM, AT_ZONE_TYPE };
    int r;
    switch (dir) {
    case DIR_UP:
        if (z == AT_ZONE_CONTROLS) return AT_ZONE_COSTUME;
        if (z == AT_ZONE_PUCK) return NAV_GRID;                                  /* the dock is in the left column: up from it leaves the panel */
        if (z == AT_ZONE_COSTUME) { r = pick_zone(c, port, k, l_up_cost, 3); return r < 0 ? NAV_GRID : r; }
        if (z == AT_ZONE_LEVEL) { r = pick_zone(c, port, k, l_up_level, 4); return r < 0 ? NAV_GRID : r; }
        return NAV_GRID;
    case DIR_DOWN:
        if (z == AT_ZONE_TYPE || z == AT_ZONE_TEAM) { r = pick_zone(c, port, k, l_down_top, 3); return r < 0 ? NAV_STAY : r; }   /* down through the portrait: the costume stepper, the slider */
        if (z == AT_ZONE_PUCK) { r = pick_zone(c, port, k, l_cost_level, 2); return r < 0 ? NAV_STAY : r; }
        if (z == AT_ZONE_COSTUME) { r = pick_zone(c, port, k, l_level, 2); return r < 0 ? NAV_STAY : r; }
        return NAV_STAY;
    case DIR_LEFT:
        if (z == AT_ZONE_TYPE) { r = pick_zone(c, port, k, l_team_puck, 2); return r < 0 ? NAV_PREV : r; }
        return NAV_PREV;
    default:
        if (z == AT_ZONE_TEAM || z == AT_ZONE_PUCK) { r = pick_zone(c, port, k, l_type, 1); return r < 0 ? NAV_NEXT : r; }
        return NAV_NEXT;
    }
}

/* the hand leaves a panel for the roster, at the column that card sits under. Melee: a hand that reaches the roster with its own coin not placed
 * carries the coin (mnCharSel_CursorThink: `sel_icon >= MNCS_NUM_U` after `cy9 > 0.2f`), and a NA port wakes up as a human. */
static unsigned enter_roster(AtCss *c, int port)
{
    AtCssPort *s = &c->p[port];
    unsigned ev = 0;
    if (s->kind == AT_CSS_OFF) { s->kind = AT_CSS_HMN; s->ck = AT_CK_NONE; }
    if (!s->hold && s->ck == AT_CK_NONE && s->kind != AT_CSS_OFF) { puck_grab(c, port, port); ev |= AT_CE_FORWARD; }
    return ev;
}

static unsigned go_grid(AtCss *c, int port, int k)
{
    AtCssPort *s = &c->p[port];
    int cols = c->cols > 0 ? c->cols : 1, n = c->n_vis;
    unsigned ev = AT_CE_MOVE;
    s->cur = ((n - 1) / cols) * cols + k * cols / 4;
    if (s->cur >= n) s->cur = n - 1;
    s->card = -1;
    ev |= enter_roster(c, port);
    return ev;
}

/* the type button (mnCharSel_CursorThink, the togglebtn box: p_kind + 1; HMN to CPU to NA to HMN, to CPU when that port has no controller).
 * A port that becomes a CPU with no fighter takes the Random tile (retail rolls a tile: mnCharSel_8025FB50). */
static int roll_fighter(AtCss *c)                                   /* a CPU's fighter: a random tile that can be picked (mnCharSel_8025FB50) */
{
    int tries, slot = 0, n = c->n_slots - 1;
    if (n < 1) return AT_CK_RANDOM;
    for (tries = 0; tries < 64; tries++) {
        c->rng = c->rng * 1664525u + 1013904223u;
        slot = (int) ((c->rng >> 8) % (unsigned) n);
        if (c->ok[slot] && c->ck[slot] >= 0) return c->ck[slot];
    }
    return c->ck[0] >= 0 ? c->ck[0] : AT_CK_RANDOM;
}

static void cycle_kind(AtCss *c, int k)
{
    AtCssPort *t = &c->p[k];
    int h = held_by(c, k);
    if (h >= 0) c->p[h].hold = 0, c->p[h].target = h;
    if (t->kind == AT_CSS_HMN) { t->kind = AT_CSS_CPU; t->cpu_lv = t->cpu_lv < 1 ? 9 : t->cpu_lv; if (t->ck == AT_CK_NONE) { t->ck = roll_fighter(c); t->costume = t->ck >= 0 ? at_css_free_costume(c, k, t->ck, 0, 1) : 0; } }
    else if (t->kind == AT_CSS_CPU) { t->kind = AT_CSS_OFF; t->ck = AT_CK_NONE; t->costume = 0; }
    else if (c->pad_ok[k]) { t->kind = AT_CSS_HMN; t->ck = AT_CK_NONE; t->costume = 0; at_css_hand_reset(c, k); }
    else { t->kind = AT_CSS_CPU; t->cpu_lv = 9; t->ck = roll_fighter(c); t->costume = t->ck >= 0 ? at_css_free_costume(c, k, t->ck, 0, 1) : 0; }
}

static unsigned puck_cards(AtCss *c, int port, AtCssIn in)
{
    AtCssPort *s = &c->p[port];
    unsigned ev = 0;
    int k = s->card, z = s->zone, dir = -1, r;
    if (k < 0 || k > 3) { s->card = -1; return 0; }
    if (in.rep & AT_CI_LEFT) dir = DIR_LEFT; else if (in.rep & AT_CI_RIGHT) dir = DIR_RIGHT; else if (in.rep & AT_CI_UP) dir = DIR_UP; else if (in.rep & AT_CI_DOWN) dir = DIR_DOWN;
    if (c->free) dir = -1;                                           /* a free hand goes where the stick takes it */
    if (c->free && z == AT_ZONE_NONE) return ev;                      /* over the panel's body, not a part of it: nothing to press */
    if (s->hold) {                                                  /* a puck in the hand over the panels: sideways between them, up to the roster, A or B puts it home */
        if (dir == DIR_LEFT || dir == DIR_RIGHT) { int nk = next_card(c, port, k, dir == DIR_RIGHT ? 1 : -1); s->card = nk < 0 ? k : nk; ev |= AT_CE_MOVE; }
        else if (dir == DIR_UP) ev |= go_grid(c, port, k);
        else if (in.trig & (AT_CI_A | AT_CI_B)) { puck_dock(c, port); ev |= AT_CE_BACK; }
        return ev;
    }
    if (!at_css_zone_ok(c, port, k, z)) {
        if (c->free) return ev;
        z = first_zone(c, port, k);
        if (z < 0) { s->card = -1; return 0; }
        s->zone = z;
    }
    if (dir == DIR_LEFT || dir == DIR_RIGHT) {
        int d = dir == DIR_RIGHT ? 1 : -1;
        if (z == AT_ZONE_LEVEL) {
            int lv = c->p[k].cpu_lv + d;
            if (lv >= 1 && lv <= 9) { c->p[k].cpu_lv = lv; ev |= AT_CE_MOVE; }
        } else if (z == AT_ZONE_COSTUME) { shift_costume(c, k, d); ev |= AT_CE_MOVE; }
        else {
            r = nav_zone(c, port, k, z, dir);
            if (r >= 0) { s->zone = r; ev |= AT_CE_MOVE; }
            else { int nk = next_card(c, port, k, d); if (nk >= 0 && nk != k) { card_to(c, port, nk); ev |= AT_CE_MOVE; } }
        }
    } else if (dir == DIR_UP || dir == DIR_DOWN) {
        r = nav_zone(c, port, k, z, dir);
        if (r >= 0) { s->zone = r; ev |= AT_CE_MOVE; }
        else if (r == NAV_GRID) ev |= go_grid(c, port, k);
    } else if (in.trig & AT_CI_A) {
        switch (z) {
        case AT_ZONE_PUCK: puck_grab(c, port, k); ev |= AT_CE_FORWARD; break;
        case AT_ZONE_TYPE: cycle_kind(c, k); ev |= c->p[k].kind == AT_CSS_OFF ? AT_CE_BACK : AT_CE_FORWARD; break;
        case AT_ZONE_LEVEL:
            if (c->free) {                                              /* the handle: A takes hold when the hand is on it, the stick drags it, A lets go (updateGrabbedSlider) */
                int d = s->val - c->p[k].cpu_lv;
                if (s->val >= 1 && d >= -1 && d <= 1) { s->drag = k; ev |= AT_CE_FORWARD; }
            } else { c->p[k].cpu_lv = c->p[k].cpu_lv >= 9 ? 1 : c->p[k].cpu_lv + 1; ev |= AT_CE_MOVE; }
            break;
        case AT_ZONE_COSTUME: shift_costume(c, k, 1); ev |= AT_CE_MOVE; break;
        case AT_ZONE_TEAM: c->p[k].team = (c->p[k].team + 1) % 3; ev |= AT_CE_MOVE; break;
        case AT_ZONE_CONTROLS: ev |= at_css_controls_open(c, port); break;
        default: break;
        }
    } else if (in.trig & (AT_CI_X | AT_CI_Y)) {
        if (c->p[k].ck >= 0 && (c->p[k].kind == AT_CSS_CPU || k == port)) { shift_costume(c, k, (in.trig & AT_CI_X) ? 1 : -1); ev |= AT_CE_MOVE; }
    } else if ((in.trig & AT_CI_Z) && !c->prof->dummy_cpu) {
        if (c->p[k].kind == AT_CSS_CPU && k != port) { c->p[k].kind = AT_CSS_OFF; c->p[k].ck = AT_CK_NONE; ev |= AT_CE_BACK; }
        else if (k == port && port != 0 && c->prof->max_humans > 1) {
            s->kind = AT_CSS_OFF; s->ck = AT_CK_NONE; s->card = -1; s->act = 0; ev |= AT_CE_BACK;
        }
    } else if (in.trig & AT_CI_B) {
        s->card = -1;
        ev |= AT_CE_BACK;
    }
    return ev;
}

/* the hand on the roster (mnCharSel_CursorThink):
 *  - holding a coin, A on a fighter sets it down: the pick, the announcer and the coin lies on the tile (the `icons[i]` bounds loop, `ckind = char_kind`);
 *    A where no fighter is available (a locked icon) is the error blip (`lbAudioAx_80024030(3)`); B puts the coin back (mnCharSel_8025FDEC);
 *    X and Y change the costume while the coin is over a fighter (the `cancel_or_costume` label);
 *  - a free hand: A takes the nearest set-down coin, yours or a CPU's (the `closest` loop: the hand is within about 3 units of it), and A anywhere else
 *    does nothing; B takes your own set-down coin; X and Y change the costume of the fighter you have picked (mnCharSel_CostumeChange, not in team games);
 *  - moving the hand down onto a panel with a coin in it puts the coin home by itself (`cursor->x10 < 0.2f` in the held-coin case). */
static unsigned puck_grid(AtCss *c, int port, AtCssIn in, int frame)
{
    AtCssPort *s = &c->p[port];
    unsigned ev = 0;
    int cols = c->cols > 0 ? c->cols : 1, n = c->n_vis, row, col, last_row, cur;
    if (s->kind == AT_CSS_OFF) { s->kind = AT_CSS_HMN; s->ck = AT_CK_NONE; }
    row = s->cur / cols; col = s->cur % cols; last_row = (n - 1) / cols;
    if (c->free) in.rep = 0;
    if (in.rep & AT_CI_LEFT) col = col == 0 ? cols - 1 : col - 1;
    else if (in.rep & AT_CI_RIGHT) col = col == cols - 1 ? 0 : col + 1;
    else if (in.rep & AT_CI_UP) row = row == 0 ? last_row : row - 1;
    else if (in.rep & AT_CI_DOWN) {
        if (row + 1 > last_row || (row + 1) * cols + col >= n) {
            if (!c->online) {                                   /* down off the roster: onto the panels */
                int k = col * 4 / cols, own = port;
                if (first_zone(c, port, k) < 0) k = first_zone(c, port, own) >= 0 ? own : next_card(c, port, k, 1);
                if (s->hold) { puck_dock(c, port); return ev | AT_CE_BACK; }
                if (k >= 0) { s->card = k; s->zone = first_zone(c, port, k); return ev | AT_CE_MOVE; }
                row = 0;
            } else row = 0;
        } else row++;
    }
    if (row * cols + col >= n) {
        col = n - 1 - row * cols;
        if (col < 0) { row = 0; col = 0; }
    }
    cur = row * cols + col;
    if (cur != s->cur) ev |= AT_CE_MOVE;
    s->cur = cur;
    if (in.trig & AT_CI_A) {
        if (s->hold && c->free && s->blank) {                      /* a coin set down where there is no fighter: the error blip */
            ev |= AT_CE_BACK;
        } else if (s->hold) {
            unsigned r = pick(c, port, c->vis[s->cur]);
            if (r & AT_CE_FORWARD) { s->hold = 0; s->from_ck = AT_CK_NONE; s->target = port; }
            ev |= r;
        } else {
            int o, found = -1;
            for (o = 0; o < 4 && found < 0 && !(c->free && s->blank); o++) {
                int q = (port + o) % 4;                          /* your own coin first, then a CPU's */
                if ((q == port || c->p[q].kind == AT_CSS_CPU) && at_css_puck_vis(c, q) == s->cur) found = q;
            }
            if (found >= 0) { puck_grab(c, port, found); ev |= AT_CE_FORWARD; }
        }
    } else if (in.trig & (AT_CI_X | AT_CI_Y)) {
        int d = (in.trig & AT_CI_X) ? 1 : -1;
        if (c->free && s->blank && s->hold) {
        } else if (s->hold) {
            int ck = c->ck[c->vis[s->cur]];                     /* the coin over a fighter previews its costume */
            AtCssPort *t = &c->p[s->target];
            if (ck >= 0) { t->costume = at_css_free_costume(c, s->target, ck, (t->costume < ncost(c, ck) ? t->costume : 0) + d, d); ev |= AT_CE_MOVE; }
        } else if (s->ck >= 0) { shift_costume(c, port, d); ev |= AT_CE_MOVE; }
    } else if (in.trig & AT_CI_B) {
        ev |= AT_CE_BACK;
        if (s->hold) { puck_dock(c, port); }
        else if (s->ck != AT_CK_NONE) { puck_grab(c, port, port); }
        else if (frame < c->back_until) { ev |= AT_CE_FINISH_BACK; c->done = 1; }
        else { c->back_until = frame + 150; toast(c, "Press B again to go back."); ev |= AT_CE_TOAST; }
    }
    return ev;
}

unsigned at_css_hand_over(AtCss *c, int port, int region, int a, int b, int val)
{
    AtCssPort *s;
    unsigned ev = 0;
    if (c->prof == NULL || !c->puck || port < 0 || port > 3 || c->controls_port >= 0) return 0;
    s = &c->p[port];
    if (s->kind != AT_CSS_HMN && !s->act) return 0;
    if (s->drag >= 0) return 0;                                     /* the hand is locked to the slider it holds */
    if (region == 0) {
        if (s->card >= 0) { ev |= enter_roster(c, port); s->card = -1; }   /* from a panel onto the roster: your own coin comes with the hand, a NA port wakes */
        if (a >= 0 && a < c->n_vis) { if (s->cur != a) ev |= AT_CE_MOVE; s->cur = a; s->blank = 0; }
        else s->blank = 1;
    } else {
        if (a < 0 || a > 3) return ev;
        if (s->card < 0 && s->hold) { puck_dock(c, port); ev |= AT_CE_BACK; }  /* a coin in the hand moved onto the panels goes home by itself */
        if (s->card != a) ev |= AT_CE_MOVE;
        s->card = a;
        s->zone = b >= 0 && b < AT_ZONE_N && at_css_zone_ok(c, port, a, b) ? b : AT_ZONE_NONE;
        s->val = s->zone == AT_ZONE_LEVEL ? val : 0;
        s->blank = 0;
    }
    return ev;
}

void at_css_drag_to(AtCss *c, int port, int level)
{
    AtCssPort *s;
    if (port < 0 || port > 3) return;
    s = &c->p[port];
    if (s->drag < 0 || s->drag > 3) return;
    c->p[s->drag].cpu_lv = level < 1 ? 1 : (level > 9 ? 9 : level);
}

unsigned at_css_mouse_zone(AtCss *c, int port, int card, int zone, int value, int moved, int click, int rclick, int wheel)
{
    AtCssPort *s;
    unsigned b = rclick ? AT_CI_B : 0u;
    if (c->controls_port >= 0) return 0;
    if (zone == AT_ZONE_CONTROLS && at_css_zone_ok(c, port, card, zone)) {
        if (moved || click) { c->p[port].card = card; c->p[port].zone = zone; }
        return b | (click ? AT_CI_A : 0u);
    }
    if (!c->puck || c->prof == NULL || port < 0 || port > 3 || card < 0 || card > 3) return b;
    s = &c->p[port];
    if (s->kind != AT_CSS_HMN && !s->act) { if (click && card == port) b |= AT_CI_A; return b; }
    if (s->hold) {                                              /* carrying: over a panel, a click puts it home when it is that panel's */
        if (moved || click) s->card = card;
        if (click && card == s->target) b |= AT_CI_A;
        return b;
    }
    if (!at_css_zone_ok(c, port, card, zone)) return b;
    if (moved || click || wheel != 0) { s->card = card; s->zone = zone; }
    if (click) {
        if (zone == AT_ZONE_LEVEL && value >= 1 && value <= 9) { c->p[card].cpu_lv = value; b |= AT_CI_SOUND; }
        else if (zone == AT_ZONE_COSTUME && value != 0) { shift_costume(c, card, value > 0 ? 1 : -1); b |= AT_CI_SOUND; }
        else b |= AT_CI_A;
    }
    if (wheel != 0) {
        if (zone == AT_ZONE_LEVEL) { int lv = c->p[card].cpu_lv + (wheel > 0 ? 1 : -1); if (lv >= 1 && lv <= 9) { c->p[card].cpu_lv = lv; b |= AT_CI_SOUND; } }
        else if (zone == AT_ZONE_COSTUME) { shift_costume(c, card, wheel > 0 ? 1 : -1); b |= AT_CI_SOUND; }
    }
    return b;
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
    if (c->controls_port >= 0) {
        unsigned ce = controls_step(c, port, in);
        c->controls_block_frame = frame;
        return ce;
    }
    if (c->controls_block_frame == -2) c->controls_block_frame = frame;
    if (c->controls_block_frame == frame) return 0;
    s = &c->p[port];
    if ((c->online && (in.trig & AT_CI_Z)) || ((in.trig & AT_CI_A) && s->card == port && s->zone == AT_ZONE_CONTROLS)) {
        unsigned opened = at_css_controls_open(c, port);
        if (opened) return opened;
    }
    cols = c->cols > 0 ? c->cols : 1;
    n = c->n_vis;
    if (s->kind != AT_CSS_HMN && !(c->puck && s->act)) {
        if (!c->online && c->prof->max_humans > 1 && (in.trig & (AT_CI_A | AT_CI_START))) {
            s->kind = AT_CSS_HMN; s->target = port; s->card = -1; ev |= AT_CE_FORWARD;   /* a controller joins (a CPU in its port becomes this player) */
            if (c->puck) at_css_hand_reset(c, port);
        }
        return ev;
    }
    if (in.trig & AT_CI_L) { if (step_tab(c, -1)) ev |= AT_CE_MOVE; }
    else if (in.trig & AT_CI_R) { if (step_tab(c, 1)) ev |= AT_CE_MOVE; }
    n = c->n_vis;
    if (s->cur < 0 || s->cur >= n) s->cur = 0;
    if (s->target < 0 || s->target > 3 || (s->target != port && c->p[s->target].kind != AT_CSS_CPU)) { s->target = port; s->hold = 0; }   /* a CPU someone was picking for went away */
    if (c->puck && c->free) in.rep = 0;
    if (c->puck && s->drag >= 0) {                                  /* a slider in the hand: A lets go (B too), nothing else is read */
        if (in.trig & (AT_CI_A | AT_CI_B)) { s->drag = -1; ev |= AT_CE_FORWARD; }
        return ev;
    }
    if (c->puck) {
        if (s->card < 0) ev |= puck_grid(c, port, in, frame);
        else ev |= puck_cards(c, port, in);
        if (c->done) return ev;
    } else if (s->card < 0) {
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
    if (c->controls_port >= 0) return 0;
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
