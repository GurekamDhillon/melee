/* gw_ui_sss.c - the stage select's rules, moved out of gmfrontend_select.inc into pure C (see gw_ui_sss.h). The legacy functions are named at each rule;
 * the legacy code wins where they disagree. The lobby's marks on tiles are cell flags drawn by the parts and decided elsewhere: nothing about them lives here. */
#include "gw_ui_sss.h"

#include <stdio.h>
#include <string.h>

static void rebuild_vis(AtSss *s)
{
    int i, n = 0;
    for (i = 0; i < s->n; i++)
        if (s->tab == 0 || s->st[i].type == 3 || s->st[i].tab_of == s->tab) s->vis[n++] = i;
    s->n_vis = n;
}

void at_sss_open(AtSss *s, int online)
{
    memset(s, 0, sizeof *s);
    s->online = online != 0;
    s->cols = 8;
}

/* fs_sss_open's list: empty layout slots out, one Random tile at the end */
void at_sss_set_stages(AtSss *s, int n, const AtSssStage *st)
{
    int i, k = 0;
    if (st == NULL || n < 0) n = 0;
    for (i = 0; i < n && k < AT_SSS_MAX - 1; i++) {
        if (st[i].type == 0 || st[i].type == 3) continue;               /* nothing there, or a Random icon: the one Random tile is added below */
        if (st[i].type == 1 && st[i].ext <= 0) continue;                /* an empty slot of the layout */
        s->st[k] = st[i];
        s->st[k].type = st[i].type == 2 ? 2 : 1;
        s->st[k].tab_of = st[i].tab_of >= 1 && st[i].tab_of <= 3 ? st[i].tab_of : 1;
        k++;
    }
    for (i = 0; i < n; i++) if (st[i].type == 3) { s->st[k].row = st[i].row; break; }   /* the first Random icon's row (its art) */
    if (i >= n) s->st[k].row = 0;
    s->st[k].ext = -1; s->st[k].type = 3; s->st[k].ok = 1; s->st[k].tab_of = 0;
    s->n = k + 1;
    s->tab = 0;
    s->cur = 0;
    rebuild_vis(s);
}

void at_sss_set_cols(AtSss *s, int cols) { s->cols = cols > 0 ? cols : 1; }

int at_sss_tab_count(const AtSss *s, int tab)
{
    int i, n = 0;
    for (i = 0; i < s->n; i++) if (s->st[i].type != 3 && (tab == 0 || s->st[i].tab_of == tab)) n++;
    return n;
}

int at_sss_tab_offered(const AtSss *s, int tab)
{
    if (tab == 0) return 1;
    if (tab < 1 || tab > 3) return 0;
    if (at_sss_tab_count(s, tab) < 1) return 0;
    if (tab == 1) return at_sss_tab_count(s, 2) > 0 || at_sss_tab_count(s, 3) > 0;   /* RETAIL is only a different list when something else exists */
    return 1;
}

void at_sss_set_tab(AtSss *s, int tab)
{
    int old = s->cur >= 0 && s->cur < s->n_vis ? s->vis[s->cur] : 0, i, best = 0, best_d = 1 << 30;
    if (tab < 0 || tab > 3) tab = 0;
    s->tab = tab;
    rebuild_vis(s);
    for (i = 0; i < s->n_vis; i++) {
        int d = s->vis[i] - old;
        if (d < 0) d = -d;
        if (s->st[s->vis[i]].type == 3 && s->vis[i] != old) d += 1000;     /* Random only when no stage is a better stand-in */
        if (d < best_d) { best_d = d; best = i; }
    }
    s->cur = best;
}

int at_sss_tab_for_ext(int ext) { return ext >= 288 ? 2 : 1; }

void at_sss_cursor_ext(AtSss *s, int ext)
{
    int i;
    for (i = 0; i < s->n_vis; i++) if (s->st[s->vis[i]].type != 3 && s->st[s->vis[i]].ext == ext) { s->cur = i; return; }
}

static int step_tab(AtSss *s, int dir)
{
    int t = s->tab, k;
    for (k = 0; k < 4; k++) {
        t = (t + dir + 4) % 4;
        if (at_sss_tab_offered(s, t)) break;
    }
    if (t == s->tab || !at_sss_tab_offered(s, t)) return 0;
    at_sss_set_tab(s, t);
    return 1;
}

int at_sss_random_pool(const AtSss *s)
{
    int i, k = 0;
    for (i = 0; i < s->n; i++) k += s->st[i].ok && s->st[i].type != 3;
    return k;
}

/* fs_random_stage with the dice injected: the r-th unlocked stage that is not the Random tile; 31 (Battlefield) when none */
int at_sss_pick_random(const AtSss *s, int r)
{
    int k = at_sss_random_pool(s), i;
    if (k == 0) return 31;
    r %= k;
    if (r < 0) r += k;
    for (i = 0; i < s->n; i++) if (s->st[i].ok && s->st[i].type != 3 && r-- == 0) return s->st[i].ext;
    return 31;
}

/* fs_sss_frame, on the merged input of every port */
unsigned at_sss_step(AtSss *s, unsigned trig, unsigned rep, int random_pick)
{
    unsigned ev = 0;
    int cols = s->cols > 0 ? s->cols : 1, n, row, col, last_row, cur;
    if (s->done || s->n_vis <= 0) return 0;
    if (trig & AT_CI_L) { if (step_tab(s, -1)) ev |= AT_CE_MOVE; }
    else if (trig & AT_CI_R) { if (step_tab(s, 1)) ev |= AT_CE_MOVE; }
    n = s->n_vis;
    if (s->cur < 0 || s->cur >= n) s->cur = 0;
    row = s->cur / cols; col = s->cur % cols; last_row = (n - 1) / cols;
    if (rep & AT_CI_LEFT) col = col == 0 ? cols - 1 : col - 1;
    else if (rep & AT_CI_RIGHT) col = col == cols - 1 ? 0 : col + 1;
    else if (rep & AT_CI_UP) row = row == 0 ? last_row : row - 1;
    else if (rep & AT_CI_DOWN) row = row >= last_row ? 0 : row + 1;
    if (row * cols + col >= n) {
        col = n - 1 - row * cols;
        if (col < 0) { row = 0; col = 0; }
    }
    cur = row * cols + col;
    if (cur != s->cur) { s->cur = cur; ev |= AT_CE_MOVE; }
    if (trig & (AT_CI_A | AT_CI_START)) {
        const AtSssStage *g = &s->st[s->vis[s->cur]];
        if (!g->ok) {
            snprintf(s->toast, sizeof s->toast, "%s", "That stage is locked.");
            ev |= AT_CE_BACK | AT_CE_TOAST;
        } else {
            s->go = 1;
            s->pick_ext = g->type == 3 ? random_pick : g->ext;
            s->done = 1;
            ev |= AT_CE_FORWARD | AT_CE_FINISH_GO;
        }
    } else if (trig & AT_CI_B) {
        s->done = 1;
        ev |= AT_CE_BACK | AT_CE_FINISH_BACK;
    }
    return ev;
}

unsigned at_sss_mouse_bits(AtSss *s, int vis_index, int moved, int click, int wheel)
{
    unsigned b = 0;
    int t, offered = 0;
    if (vis_index >= 0 && vis_index < s->n_vis) {
        if (moved || click) s->cur = vis_index;
        if (click) b |= AT_CI_A;
    }
    if (wheel != 0) {
        for (t = 0; t < 4; t++) offered += at_sss_tab_offered(s, t);
        if (offered > 1) b |= wheel > 0 ? AT_CI_L : AT_CI_R;
    }
    return b;
}
