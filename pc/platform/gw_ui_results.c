#include "gw_ui_results.h"
#include <stdio.h>
#include <string.h>

static int clampi(int v, int lo, int hi) { return v < lo ? lo : v > hi ? hi : v; }

/* standing key: winner, stocks, damage (summed for a team); 1 when a is strictly ahead of b */
typedef struct { int winner, stocks, percent; } Key;
static int ahead(const Key *a, const Key *b)
{
    if (a->winner != b->winner) return a->winner > b->winner;
    if (a->stocks != b->stocks) return a->stocks > b->stocks;
    return a->percent < b->percent;
}

void at_results_build(AtResults *r, const AtResInput *in)
{
    int i, j;
    Key k[4], tk[4];
    int tvalid[4];
    memset(r, 0, sizeof *r);
    r->canceled = in->canceled != 0;
    for (i = 0; i < 4; i++) {
        const AtResPlayerIn *p = &in->p[i];
        AtResPlayer *o;
        if (p->pkind < AT_PK_HUMAN || p->pkind >= AT_PK_NONE) continue;      /* an empty slot, or a kind that is not a player */
        o = &r->players[r->n++];
        o->port = i; o->pkind = p->pkind; o->ckind = p->ckind;
        o->stocks = clampi(p->stocks, 0, 99); o->kos = clampi(p->kos, 0, 999); o->falls = clampi(p->falls, 0, 999); o->percent = clampi(p->percent, 0, 999);
        o->team = clampi(p->team, 0, 3); o->winner = p->winner != 0;
        if (o->pkind == AT_PK_HUMAN) r->humans++;
    }
    memset(tvalid, 0, sizeof tvalid); memset(tk, 0, sizeof tk);
    for (i = 0; i < r->n; i++) {
        const AtResPlayer *o = &r->players[i];
        k[i].winner = o->winner; k[i].stocks = o->stocks; k[i].percent = o->percent;
        if (in->is_teams) {
            tvalid[o->team] = 1;
            tk[o->team].winner |= o->winner; tk[o->team].stocks += o->stocks; tk[o->team].percent += o->percent;
        }
    }
    for (i = 0; i < r->n; i++) {
        int place = 1;
        if (in->is_teams) {
            for (j = 0; j < 4; j++) if (tvalid[j] && j != r->players[i].team && ahead(&tk[j], &tk[r->players[i].team])) place++;
        } else {
            for (j = 0; j < r->n; j++) if (j != i && ahead(&k[j], &k[i])) place++;
        }
        r->players[i].place = place;
    }
    if (r->humans == 0) { r->done = 1; r->exit_frames = 20; }               /* nobody has to confirm: the long countdown */
    else r->exit_frames = 10;
}

void at_results_confirm(AtResults *r, int port, int kind)
{
    int i, all = 1;
    if (r->done) return;
    for (i = 0; i < r->n; i++) {
        AtResPlayer *p = &r->players[i];
        if (p->port != port || p->pkind != AT_PK_HUMAN) continue;
        if (r->canceled) {
            if (kind == AT_RC_START) { p->confirmed = 1; r->done = 1; r->exit_frames = 0; }
            return;
        }
        if (kind == AT_RC_START || kind == AT_RC_ERR) p->confirmed = 1;
    }
    if (r->canceled) return;
    for (i = 0; i < r->n; i++) if (r->players[i].pkind == AT_PK_HUMAN && !r->players[i].confirmed) all = 0;
    if (all && r->humans > 0) { r->done = 1; r->exit_frames = 10; }
}

int at_results_done(const AtResults *r) { return r->done; }

const char *at_place_word(int place)
{
    static const char *const w[5] = { "", "1st", "2nd", "3rd", "4th" };
    return (place >= 1 && place <= 4) ? w[place] : w[0];
}

int at_results_rows(const AtResults *r, const char *const names[4], AtDataRow *rows, int cap)
{
    int order[4], n = 0, i, j;
    for (i = 0; i < r->n; i++) {                                           /* by place, ties by port (the build order): a stable insertion */
        j = n++;
        while (j > 0 && r->players[order[j - 1]].place > r->players[i].place) { order[j] = order[j - 1]; j--; }
        order[j] = i;
    }
    if (n > cap) n = cap;
    for (i = 0; i < n; i++) {
        const AtResPlayer *p = &r->players[order[i]];
        const char *nm = (names != NULL && p->port >= 0 && p->port < 4 && names[p->port] != NULL) ? names[p->port] : "";
        AtDataRow *o = &rows[i];
        memset(o, 0, sizeof *o);
        snprintf(o->label, AT_STR, "P%d%s%s%s", p->port + 1, p->pkind == AT_PK_HUMAN ? "" : " CPU", nm[0] != '\0' ? "  " : "", nm);
        if (p->winner) { o->flags |= AT_DR_WIN; snprintf(o->value, AT_STR, "%s", "WINNER"); }
        else snprintf(o->value, AT_STR, "%s", at_place_word(p->place));
        snprintf(o->sub, AT_STR, "%d KO%s, %d fall%s, %d%%", p->kos, p->kos == 1 ? "" : "s", p->falls, p->falls == 1 ? "" : "s", p->percent);
    }
    return n;
}
