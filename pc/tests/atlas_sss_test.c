#include "atlas_check.h"
#include "../platform/gw_ui_sss.h"

static AtSss mk(int n, int added, int tourn)
{
    AtSss s; static AtSssStage st[300]; int i;
    memset(st, 0, sizeof st);
    for (i = 0; i < n; i++) { st[i].ext = 20 + i; st[i].type = 2; st[i].ok = 1; st[i].tab_of = (unsigned char) (i >= n - added ? 2 : 1); st[i].row = i; }
    for (i = 0; i < tourn && i < n; i++) st[i].tab_of = 3;
    at_sss_open(&s, 0); at_sss_set_stages(&s, n, st); at_sss_set_cols(&s, 8);
    return s;
}
static unsigned step(AtSss *s, unsigned trig, unsigned rep) { return at_sss_step(s, trig, rep, 77); }

static void random_tile_last_and_unique(void)
{
    AtSss s; AtSssStage st[6]; int i;
    memset(st, 0, sizeof st);
    for (i = 0; i < 6; i++) { st[i].ext = 20 + i; st[i].type = 2; st[i].ok = 1; st[i].tab_of = 1; }
    st[2].type = 3; st[4].type = 3;                             /* two Random icons (an Akaneia layout) */
    at_sss_open(&s, 0); at_sss_set_stages(&s, 6, st);
    CHECK(s.n == 5 && s.st[4].type == 3 && s.st[4].ext == -1 && s.st[4].ok == 1);   /* four stages and one Random, last */
    for (i = 0; i < 4; i++) CHECK(s.st[i].type != 3);
    CHECK(s.st[0].ext == 20 && s.st[1].ext == 21 && s.st[2].ext == 23 && s.st[3].ext == 25);   /* the rest keep the disc's order */
    CHECK(s.n_vis == 5 && s.vis[4] == 4);
    /* empty layout slots (type 1 with no stage) and type 0 are dropped; no Random icon on the disc: one is still added */
    memset(st, 0, sizeof st); st[0].ext = 31; st[0].type = 1; st[0].ok = 1; st[0].tab_of = 1; st[1].ext = 0; st[1].type = 1; st[2].ext = 5; st[2].type = 0; st[3].ext = -3; st[3].type = 1;
    at_sss_open(&s, 0); at_sss_set_stages(&s, 4, st);
    CHECK(s.n == 2 && s.st[0].ext == 31 && s.st[1].type == 3);
    at_sss_open(&s, 0); at_sss_set_stages(&s, 0, NULL); CHECK(s.n == 1 && s.st[0].type == 3 && s.n_vis == 1);
}
static void locked_stage_cannot_be_picked(void)
{
    AtSss s; AtSssStage st[3]; unsigned e;
    memset(st, 0, sizeof st);
    st[0].ext = 31; st[0].type = 2; st[0].ok = 1; st[0].tab_of = 1;
    st[1].ext = 32; st[1].type = 2; st[1].ok = 0; st[1].tab_of = 1;
    at_sss_open(&s, 0); at_sss_set_stages(&s, 2, st); s.cols = 8; s.cur = 1;
    e = at_sss_step(&s, AT_CI_A, 0, 0);
    CHECK((e & AT_CE_BACK) && (e & AT_CE_TOAST) && !s.go && !s.done && strcmp(s.toast, "That stage is locked.") == 0);   /* a locked stage: no pick */
    e = at_sss_step(&s, AT_CI_START, 0, 0); CHECK((e & AT_CE_BACK) && !s.go);                               /* START on it is the same */
    s.cur = 0; e = at_sss_step(&s, AT_CI_A, 0, 0); CHECK((e & AT_CE_FINISH_GO) && (e & AT_CE_FORWARD) && s.go && s.pick_ext == 31 && s.done);
    CHECK(at_sss_step(&s, AT_CI_A, 0, 0) == 0);                                                              /* a second press in the same scene does nothing */
}
static void a_start_b(void)
{
    AtSss s = mk(10, 0, 0); unsigned e;
    s.cur = 3; e = step(&s, AT_CI_START, 0); CHECK((e & AT_CE_FINISH_GO) && s.pick_ext == 23);               /* START picks the hovered one */
    s = mk(10, 0, 0); s.cur = 4; e = step(&s, AT_CI_B, 0); CHECK((e & AT_CE_BACK) && (e & AT_CE_FINISH_BACK) && !s.go && s.done);
    s = mk(10, 0, 0); s.cur = 10; e = at_sss_step(&s, AT_CI_A, 0, 5); CHECK((e & AT_CE_FINISH_GO) && s.pick_ext == 5);   /* the Random tile: the caller's roll */
    s = mk(10, 0, 0); CHECK(step(&s, 0, 0) == 0 && !s.done);                                                 /* nothing pressed: nothing happens */
}
static void movement(void)
{
    AtSss s = mk(20, 0, 0);                                    /* 20 stages and Random: 21 tiles in 8 columns, three rows (8, 8, 5) */
    s.cur = 0;
    CHECK((step(&s, 0, AT_CI_LEFT) & AT_CE_MOVE) && s.cur == 7);
    step(&s, 0, AT_CI_RIGHT); CHECK(s.cur == 0);
    step(&s, 0, AT_CI_UP); CHECK(s.cur == 16);                                                                /* up from row 0: the last row */
    step(&s, 0, AT_CI_DOWN); CHECK(s.cur == 0);                                                               /* down from the last row: row 0 */
    s.cur = 5; step(&s, 0, AT_CI_DOWN); CHECK(s.cur == 13);
    s.cur = 7; step(&s, 0, AT_CI_DOWN); step(&s, 0, AT_CI_DOWN); CHECK(s.cur == 20 && !(step(&s, 0, 0) & AT_CE_MOVE));   /* the short last row: clamped to its last tile */
    s.cur = 20; step(&s, 0, AT_CI_RIGHT); CHECK(s.cur == 20);                                                 /* the legacy clamp */
    s.cur = 99; step(&s, 0, 0); CHECK(s.cur >= 0 && s.cur < s.n_vis);                                         /* a wild cursor is brought home */
    s.cols = 0; s.cur = 0; step(&s, 0, AT_CI_DOWN); CHECK(s.cur >= 0 && s.cur < s.n_vis);                    /* no columns yet: one column, no divide */
}
static void random_pool(void)
{
    AtSss s = mk(6, 0, 0);
    CHECK(at_sss_random_pool(&s) == 6 && at_sss_pick_random(&s, 0) == 20 && at_sss_pick_random(&s, 5) == 25);
    s.st[1].ok = 0; CHECK(at_sss_random_pool(&s) == 5 && at_sss_pick_random(&s, 1) == 22);                   /* skips locked and the Random tile */
    CHECK(at_sss_pick_random(&s, 99) >= 20 && at_sss_pick_random(&s, -4) >= 20);                              /* a roll out of range still lands on a stage */
    { int i; for (i = 0; i < s.n; i++) s.st[i].ok = 0; s.st[s.n - 1].ok = 1; CHECK(at_sss_random_pool(&s) == 0 && at_sss_pick_random(&s, 0) == 31); }   /* nothing unlocked: Battlefield */
}
static void tabs_and_256(void)
{
    AtSss s = mk(255, 40, 0);                                   /* the cap: 255 stages and Random */
    CHECK(s.n == 256 && s.n_vis == 256 && s.st[255].type == 3);
    CHECK(at_sss_tab_count(&s, 0) == 255 && at_sss_tab_count(&s, 1) == 215 && at_sss_tab_count(&s, 2) == 40 && at_sss_tab_count(&s, 1) + at_sss_tab_count(&s, 2) == at_sss_tab_count(&s, 0));
    at_sss_set_tab(&s, 2); CHECK(s.n_vis == 41 && s.vis[40] == 255 && s.st[s.vis[0]].tab_of == 2);
    at_sss_set_tab(&s, 1); CHECK(s.n_vis == 216);
    at_sss_set_tab(&s, 3); CHECK(s.n_vis == 1);                                                              /* tournament: no stages, only Random */
    { AtSssStage big[300]; int i; AtSss t; memset(big, 0, sizeof big); for (i = 0; i < 300; i++) { big[i].ext = i + 1; big[i].type = 2; big[i].ok = 1; big[i].tab_of = 1; }
      at_sss_open(&t, 0); at_sss_set_stages(&t, 300, big); CHECK(t.n == AT_SSS_MAX && t.st[AT_SSS_MAX - 1].type == 3 && t.n_vis == AT_SSS_MAX); }   /* 300 given: cut at the cap, Random kept */
    /* offered tabs */
    s = mk(30, 5, 0); CHECK(at_sss_tab_offered(&s, 0) && at_sss_tab_offered(&s, 1) && at_sss_tab_offered(&s, 2) && !at_sss_tab_offered(&s, 3));
    s = mk(30, 0, 0); CHECK(at_sss_tab_offered(&s, 0) && !at_sss_tab_offered(&s, 1) && !at_sss_tab_offered(&s, 2));
    s = mk(30, 0, 6); CHECK(at_sss_tab_offered(&s, 3) && at_sss_tab_offered(&s, 1) && at_sss_tab_count(&s, 3) == 6);   /* the tournament tab only where the adapter supplied the six */
    /* focus survives a tab switch: the cursor stays on its stage, or goes to the nearest visible, never outside vis */
    s = mk(30, 5, 0); s.cur = 30;                                                                             /* Random */
    at_sss_set_tab(&s, 2); CHECK(s.cur == s.n_vis - 1);
    s = mk(30, 5, 0); s.cur = 10; at_sss_set_tab(&s, 1); CHECK(s.vis[s.cur] == 10);
    at_sss_set_tab(&s, 2); CHECK(s.cur >= 0 && s.cur < s.n_vis && s.st[s.vis[s.cur]].tab_of == 2);
    at_sss_set_tab(&s, 7); CHECK(s.tab == 0);
    /* L and R step the offered tabs */
    s = mk(30, 5, 0);
    CHECK((step(&s, AT_CI_R, 0) & AT_CE_MOVE) && s.tab == 1); step(&s, AT_CI_R, 0); CHECK(s.tab == 2); step(&s, AT_CI_R, 0); CHECK(s.tab == 0); step(&s, AT_CI_L, 0); CHECK(s.tab == 2);
    s = mk(30, 0, 0); CHECK(!(step(&s, AT_CI_R, 0) & AT_CE_MOVE) && s.tab == 0);
    /* the cursor onto the rules' stage */
    s = mk(30, 5, 0); at_sss_cursor_ext(&s, 37); CHECK(s.st[s.vis[s.cur]].ext == 37); at_sss_cursor_ext(&s, 9999); CHECK(s.st[s.vis[s.cur]].ext == 37);
}
static void added_rule(void)
{
    CHECK(at_sss_tab_for_ext(0) == 1 && at_sss_tab_for_ext(31) == 1 && at_sss_tab_for_ext(287) == 1 && at_sss_tab_for_ext(288) == 2 && at_sss_tab_for_ext(1000) == 2);   /* the legacy rule: ext >= 288 */
}
static void mouse(void)
{
    AtSss s = mk(30, 5, 0); unsigned b;
    b = at_sss_mouse_bits(&s, 7, 1, 0, 0); CHECK(s.cur == 7 && b == 0);
    b = at_sss_mouse_bits(&s, 9, 0, 1, 0); CHECK(s.cur == 9 && (b & AT_CI_A));
    b = at_sss_mouse_bits(&s, -1, 1, 1, 0); CHECK(s.cur == 9 && b == 0);                                      /* off the tiles: nothing */
    b = at_sss_mouse_bits(&s, 2000, 1, 1, 0); CHECK(s.cur == 9 && b == 0);
    b = at_sss_mouse_bits(&s, -1, 0, 0, 1); CHECK(b & AT_CI_L);
    b = at_sss_mouse_bits(&s, -1, 0, 0, -1); CHECK(b & AT_CI_R);
    s = mk(30, 0, 0); b = at_sss_mouse_bits(&s, -1, 0, 0, 1); CHECK(b == 0);
}
static void no_decisions(void)
{
    /* the model carries no strike or ban state: its public record has no such field and the flag names are the cells' only (drawing) */
    AtSss s = mk(8, 0, 0);
    CHECK(sizeof s.st[0] <= 24);
}

int main(void)
{
    random_tile_last_and_unique(); locked_stage_cannot_be_picked(); a_start_b(); movement(); random_pool(); tabs_and_256(); added_rule(); mouse(); no_decisions();
    ATLAS_DONE("atlas sss");
}
