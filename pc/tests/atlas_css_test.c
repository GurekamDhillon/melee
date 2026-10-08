#include "atlas_check.h"
#include "../platform/gw_ui_css.h"

/* The fakes: every fighter has 4 costumes; Zelda is ck 19 and Sheik 20, shared only when the roster has no 20. */
static int g_sheik = 1;
static int g_costumes = 4;
static int f_costumes(void *u, int ck) { (void) u; (void) ck; return g_costumes; }
static int f_random(void *u, const AtCss *c) { (void) u; return c->n_slots > 0 ? c->ck[0] : 0; }
static int f_swap(void *u, int ck) { (void) u; return ck == 19 ? 20 : ck == 20 ? 19 : -1; }
static int f_sheik_ok(void *u) { (void) u; return g_sheik; }
static const AtCssOps OPS = { 0, f_costumes, f_random, f_swap, f_sheik_ok };

static void fresh(AtCss *c, int match_type, int online, int teams, int n)
{
    int ck[130], i; unsigned char ok[130], tb[130];
    for (i = 0; i < n && i < 130; i++) { ck[i] = i; ok[i] = 1; tb[i] = i < 26 ? 1 : 2; }
    at_css_open(c, at_css_profile(match_type), &OPS, online, teams);
    at_css_set_roster(c, n, ck, ok, tb);
    at_css_set_cols(c, 8);
}
static unsigned press(AtCss *c, int port, unsigned trig, unsigned rep, int frame) { AtCssIn in; in.trig = trig; in.rep = rep; return at_css_step(c, port, in, frame); }

/* ports: P1 is a human from the start; another controller joins on A or START (fs_css_port) */
static void ports_join(void)
{
    AtCss c; fresh(&c, 0x0, 0, 0, 29);
    CHECK(c.p[0].kind == AT_CSS_HMN && c.p[1].kind == AT_CSS_OFF && c.p[2].kind == AT_CSS_OFF && c.p[3].kind == AT_CSS_OFF);
    CHECK((press(&c, 1, AT_CI_A, 0, 10) & AT_CE_FORWARD) != 0 && c.p[1].kind == AT_CSS_HMN && c.p[1].target == 1 && c.p[1].card == -1);
    CHECK(c.p[2].kind == AT_CSS_OFF); press(&c, 2, AT_CI_B, 0, 11); CHECK(c.p[2].kind == AT_CSS_OFF);   /* a stray B does not join */
    press(&c, 2, 0, AT_CI_RIGHT, 11); CHECK(c.p[2].kind == AT_CSS_OFF);                                    /* nor does a direction */
    CHECK((press(&c, 3, AT_CI_START, 0, 12) & AT_CE_FORWARD) != 0 && c.p[3].kind == AT_CSS_HMN);          /* START joins too (fs_css_port) */
    CHECK(!(press(&c, 3, AT_CI_START, 0, 13) & AT_CE_FINISH_GO));                                          /* and START as a joiner does not start the match */
    /* a CPU in a port becomes the player on A (the legacy join rule: any port whose kind is not human) */
    c.p[2].kind = AT_CSS_CPU; c.p[2].ck = 7; press(&c, 2, AT_CI_A, 0, 14); CHECK(c.p[2].kind == AT_CSS_HMN && c.p[2].ck == 7);
}
/* A picks the tile under the cursor for the port its picks go to; B undoes the pick; a second B within 150 frames leaves */
static void legacy_rules_pick_undo_back(void)
{
    AtCss c; unsigned e; fresh(&c, 0x0, 0, 0, 29);
    c.p[0].cur = 4;
    e = press(&c, 0, AT_CI_A, 0, 20); CHECK((e & AT_CE_FORWARD) && c.p[0].ck == 4);
    e = press(&c, 0, AT_CI_B, 0, 21); CHECK((e & AT_CE_BACK) && c.p[0].ck == AT_CK_NONE);
    e = press(&c, 0, AT_CI_B, 0, 22); CHECK((e & AT_CE_TOAST) && !(e & AT_CE_FINISH_BACK) && strcmp(c.toast, "Press B again to go back.") == 0);
    e = press(&c, 0, AT_CI_B, 0, 100); CHECK((e & AT_CE_FINISH_BACK) && c.done == 1);                      /* inside 150 frames */
    CHECK(press(&c, 0, AT_CI_A, 0, 101) == 0);                                                              /* the scene is ending: nothing more is read */
    fresh(&c, 0x0, 0, 0, 29);
    press(&c, 0, AT_CI_B, 0, 22); e = press(&c, 0, AT_CI_B, 0, 22 + 151); CHECK(!(e & AT_CE_FINISH_BACK) && (e & AT_CE_TOAST));   /* outside: only the toast again */
    /* B while picking for a CPU stops that first (target goes home), then undoes the pick */
    fresh(&c, 0x0, 0, 0, 29); c.p[1].kind = AT_CSS_CPU; c.p[1].ck = 3; c.p[0].target = 1; c.p[0].ck = 9;
    press(&c, 0, AT_CI_B, 0, 30); CHECK(c.p[0].target == 0 && c.p[0].ck == 9);
    press(&c, 0, AT_CI_B, 0, 31); CHECK(c.p[0].ck == AT_CK_NONE);
    /* A on the Random tile (the last visible one) picks Random, costume 0 */
    fresh(&c, 0x0, 0, 0, 29); c.p[0].cur = c.n_vis - 1; press(&c, 0, AT_CI_A, 0, 40); CHECK(c.p[0].ck == AT_CK_RANDOM && c.p[0].costume == 0);
    /* a move reports MOVE and wraps: left on column 0 goes to the last column, right on the last column to column 0, up on row 0 to the last row */
    fresh(&c, 0x0, 0, 0, 29); c.p[0].cur = 0;
    CHECK((press(&c, 0, 0, AT_CI_LEFT, 41) & AT_CE_MOVE) && c.p[0].cur == 7);
    CHECK((press(&c, 0, 0, AT_CI_RIGHT, 42) & AT_CE_MOVE) && c.p[0].cur == 0);
    CHECK((press(&c, 0, 0, AT_CI_UP, 43) & AT_CE_MOVE) && c.p[0].cur == 24);                                /* row 3, column 0 of a 30-tile grid of 8 columns */
    CHECK(!(press(&c, 0, 0, 0, 44) & AT_CE_MOVE));
    c.p[0].cur = 29; press(&c, 0, 0, AT_CI_RIGHT, 45); CHECK(c.p[0].cur == 29);                             /* the short last row has no tile past its end: the cursor stays (the legacy clamp) */
    c.p[0].cur = 29; press(&c, 0, 0, AT_CI_LEFT, 46); CHECK(c.p[0].cur == 28);
    c.p[0].cur = 3; press(&c, 0, 0, AT_CI_DOWN, 47); CHECK(c.p[0].cur == 11);
    c.p[0].cur = 28; press(&c, 0, 0, AT_CI_UP, 48); CHECK(c.p[0].cur == 20);
    c.p[0].cur = 4; press(&c, 0, 0, AT_CI_DOWN, 49); CHECK(c.p[0].cur == 12 && c.p[0].card == -1);
    c.p[0].cur = 20; press(&c, 0, 0, AT_CI_DOWN, 50); CHECK(c.p[0].cur == 28 && c.p[0].card == -1);       /* a column whose last row has a tile: down goes there */
    c.p[0].cur = 22; press(&c, 0, 0, AT_CI_DOWN, 50); CHECK(c.p[0].card == 3 && c.p[0].cur == 22);        /* a column whose last row is short: off the grid, onto the card (6 * 4 / 8) */
    fresh(&c, 0x0, 0, 0, 29); c.p[0].cur = 29; press(&c, 0, 0, AT_CI_UP, 51); CHECK(c.p[0].cur == 21);      /* up from the short last row: the same column, one row up */
}
/* down off the grid goes to the cards (never online); a CPU is added with A on an empty card and its fighter is picked next */
static void legacy_rules_cards(void)
{
    AtCss c; fresh(&c, 0x0, 0, 0, 29);
    c.p[0].cur = 24;                                    /* the last row of a 30-tile grid of 8 columns */
    press(&c, 0, 0, AT_CI_DOWN, 30); CHECK(c.p[0].card == 24 % 8 * 4 / 8);                           /* col * 4 / cols */
    c.p[0].card = -1; c.p[0].cur = 23; press(&c, 0, 0, AT_CI_DOWN, 30); CHECK(c.p[0].card == 3);     /* column 7 of 8: the fourth card */
    c.p[0].card = 1;
    press(&c, 0, AT_CI_A, 0, 31); CHECK(c.p[1].kind == AT_CSS_CPU && c.p[1].ck == AT_CK_RANDOM && c.p[0].target == 1 && c.p[0].card == -1 && c.p[1].cpu_lv == 9);
    c.p[0].cur = 7; press(&c, 0, AT_CI_A, 0, 32); CHECK(c.p[1].ck == 7 && c.p[0].target == 0);      /* the CPU got it, picks are yours again */
    c.p[0].card = 1; press(&c, 0, AT_CI_Z, 0, 33); CHECK(c.p[1].kind == AT_CSS_OFF && c.p[1].ck == AT_CK_NONE);   /* Z removes a CPU */
    fresh(&c, 0x0, 1, 0, 29); c.p[0].cur = 24; press(&c, 0, 0, AT_CI_DOWN, 34); CHECK(c.p[0].card == -1 && c.p[0].cur == 0);   /* online: wraps, no cards */
    /* on the cards: left and right cycle four, up returns to the bottom row under the card, B returns, A on your own card returns */
    fresh(&c, 0x0, 0, 0, 29); c.p[0].card = 0;
    press(&c, 0, 0, AT_CI_LEFT, 35); CHECK(c.p[0].card == 3);
    press(&c, 0, 0, AT_CI_RIGHT, 36); CHECK(c.p[0].card == 0);
    press(&c, 0, 0, AT_CI_RIGHT, 37); CHECK(c.p[0].card == 1);
    press(&c, 0, 0, AT_CI_UP, 38); CHECK(c.p[0].card == -1 && c.p[0].cur == 3 * 8 + 1 * 8 / 4);        /* row 3, column 2 */
    c.p[0].card = 2; c.p[0].cur = 0; press(&c, 0, 0, AT_CI_UP, 39); CHECK(c.p[0].cur == 3 * 8 + 2 * 8 / 4);
    c.p[0].card = 3; press(&c, 0, 0, AT_CI_UP, 40); CHECK(c.p[0].cur == 29);                           /* 3 * 8 + 6 = 30 is past the list: the last tile */
    c.p[0].card = 2; CHECK((press(&c, 0, AT_CI_B, 0, 41) & AT_CE_BACK) && c.p[0].card == -1);
    c.p[0].card = 0; press(&c, 0, AT_CI_A, 0, 42); CHECK(c.p[0].card == -1);                           /* your own card: A returns to the grid */
    /* a CPU's level: X up, Y down, wrapping 9 -> 1 and 1 -> 9; on a human card X/Y change YOUR costume */
    c.p[1].kind = AT_CSS_CPU; c.p[1].cpu_lv = 9; c.p[0].card = 1;
    press(&c, 0, AT_CI_X, 0, 43); CHECK(c.p[1].cpu_lv == 1);
    press(&c, 0, AT_CI_Y, 0, 44); CHECK(c.p[1].cpu_lv == 9);
    press(&c, 0, AT_CI_Y, 0, 45); CHECK(c.p[1].cpu_lv == 8);
    c.p[0].card = 0; c.p[0].ck = 5; c.p[0].costume = 0; press(&c, 0, AT_CI_X, 0, 46); CHECK(c.p[0].costume == 1);
    /* A on a CPU card picks that CPU's fighter next; a human's card does nothing */
    c.p[0].card = 1; press(&c, 0, AT_CI_A, 0, 47); CHECK(c.p[0].target == 1 && c.p[0].card == -1);
    c.p[0].target = 0; c.p[2].kind = AT_CSS_HMN; c.p[0].card = 2; press(&c, 0, AT_CI_A, 0, 48); CHECK(c.p[0].target == 0 && c.p[0].card == 2);
    /* Z on your own card leaves, except P1; B in a toast window is not a leave */
    c.p[2].kind = AT_CSS_HMN; c.p[2].ck = 6; c.p[2].card = 2; press(&c, 2, AT_CI_Z, 0, 49); CHECK(c.p[2].kind == AT_CSS_OFF && c.p[2].ck == AT_CK_NONE && c.p[2].card == -1);
    c.p[0].card = 0; press(&c, 0, AT_CI_Z, 0, 50); CHECK(c.p[0].kind == AT_CSS_HMN);
    /* a CPU someone was picking for goes away: the picker's target goes home (legacy fs_css_frame) */
    c.p[0].target = 1; c.p[1].kind = AT_CSS_OFF; press(&c, 0, 0, AT_CI_RIGHT, 51); CHECK(c.p[0].target == 0);
}
/* costumes: X forward, Y back, never one another port wears */
static void legacy_rules_costume(void)
{
    AtCss c; fresh(&c, 0x0, 0, 0, 29);
    c.p[1].kind = AT_CSS_HMN; c.p[1].ck = 4; c.p[1].costume = 1;
    c.p[0].cur = 4; press(&c, 0, AT_CI_A, 0, 40); CHECK(c.p[0].ck == 4 && c.p[0].costume == 0);
    press(&c, 0, AT_CI_X, 0, 41); CHECK(c.p[0].costume == 2);                                           /* 1 is taken by P2 */
    press(&c, 0, AT_CI_Y, 0, 42); CHECK(c.p[0].costume == 0);
    CHECK(at_css_free_costume(&c, 0, 4, 1, 1) == 2 && at_css_free_costume(&c, 0, 5, 1, 1) == 1);        /* another fighter: nothing is taken */
    CHECK(at_css_free_costume(&c, 1, 4, 0, 1) == 1);                                                    /* P2 asking: P1 wears 0, so 1 (its own costume is not "taken") */
    c.p[2].kind = AT_CSS_CPU; c.p[2].ck = 4; c.p[2].costume = 2; c.p[3].kind = AT_CSS_CPU; c.p[3].ck = 4; c.p[3].costume = 3;
    CHECK(at_css_free_costume(&c, 0, 4, 0, 1) == 0);                                                    /* 1, 2 and 3 are worn: only 0 is left */
    c.p[0].costume = 0; c.p[0].ck = AT_CK_NONE; press(&c, 0, AT_CI_X, 0, 43); CHECK(c.p[0].costume == 0);   /* nothing picked: X changes nothing */
    /* every costume worn by the others (three ports, four costumes: with four others this cannot happen, so wear all four through a fighter of 1) */
    c.p[1].costume = 0; c.p[2].costume = 1; c.p[3].costume = 2; c.p[0].ck = 4; CHECK(at_css_free_costume(&c, 0, 4, 0, 1) == 3);   /* 0, 1 and 2 are worn: 3 is free */
}
/* Start: blocked with a reason until every human has a fighter and two fighters are in (one online) */
static void legacy_rules_start(void)
{
    AtCss c; char b[96]; unsigned e; fresh(&c, 0x0, 0, 0, 29);
    CHECK(at_css_blocker(&c, b, sizeof b) != NULL && strcmp(b, "P1: pick a fighter.") == 0);
    e = press(&c, 0, AT_CI_START, 0, 50); CHECK((e & AT_CE_TOAST) && (e & AT_CE_BACK) && !(e & AT_CE_FINISH_GO) && strcmp(c.toast, "P1: pick a fighter.") == 0);
    c.p[0].ck = 3; CHECK(strcmp(at_css_blocker(&c, b, sizeof b), "Two fighters needed: pick one, or add a CPU below.") == 0);
    c.p[1].kind = AT_CSS_HMN; CHECK(strcmp(at_css_blocker(&c, b, sizeof b), "P2: pick a fighter.") == 0);   /* the first human without a fighter is named */
    c.p[1].kind = AT_CSS_CPU; c.p[1].ck = 5; CHECK(at_css_blocker(&c, b, sizeof b) == NULL);
    CHECK(at_css_count_in(&c) == 2);
    c.p[2].kind = AT_CSS_CPU; c.p[2].ck = AT_CK_NONE; CHECK(at_css_count_in(&c) == 2 && at_css_blocker(&c, b, sizeof b) == NULL);   /* a CPU with no fighter is not counted and does not block */
    e = press(&c, 0, AT_CI_START, 0, 51); CHECK((e & AT_CE_FINISH_GO) && (e & AT_CE_FORWARD) && c.done == 1);
    CHECK(press(&c, 0, AT_CI_START, 0, 52) == 0);                                                       /* a second START in the same scene does nothing */
    /* START from any card state starts (the blocker is checked after the move) */
    fresh(&c, 0x0, 0, 0, 29); c.p[0].ck = 3; c.p[1].kind = AT_CSS_CPU; c.p[1].ck = 5; c.p[0].card = 2;
    CHECK(press(&c, 0, AT_CI_START, 0, 53) & AT_CE_FINISH_GO);
    /* a small buffer still gets a terminated string */
    fresh(&c, 0x0, 0, 0, 29); { char tiny[8]; CHECK(at_css_blocker(&c, tiny, sizeof tiny) != NULL && strlen(tiny) < sizeof tiny); }
    /* online: one fighter is enough */
    fresh(&c, AT_MT_LOBBY, 1, 0, 29); c.p[0].ck = 2; CHECK(at_css_blocker(&c, b, sizeof b) == NULL);
    c.p[0].ck = AT_CK_NONE; CHECK(strcmp(at_css_blocker(&c, b, sizeof b), "P1: pick a fighter.") == 0);
}
/* the profiles: one player (Classic) has no CPU cards and starts with one fighter; Training has the dummy */
static void profile_rules(void)
{
    AtCss c; char b[96]; fresh(&c, 0xB, 0, 0, 29);
    c.p[0].ck = 3; CHECK(at_css_blocker(&c, b, sizeof b) == NULL);                                       /* min_to_start 1 */
    c.p[0].cur = 24; press(&c, 0, 0, AT_CI_DOWN, 60); CHECK(c.p[0].card == 0);                           /* column 0 would be card 0 anyway: use a right-hand column next */
    c.p[0].card = -1; c.p[0].cur = 23; press(&c, 0, 0, AT_CI_DOWN, 61); CHECK(c.p[0].card == 0);          /* column 7 would be card 3 in the VS family: one player has only its own */
    c.p[0].card = 1; press(&c, 0, AT_CI_A, 0, 62); CHECK(c.p[1].kind == AT_CSS_OFF);                      /* no CPU can be added */
    c.p[0].card = 0; press(&c, 0, AT_CI_Z, 0, 63); CHECK(c.p[0].kind == AT_CSS_HMN);                      /* the only player cannot leave */
    CHECK(!(press(&c, 1, AT_CI_A, 0, 64) & AT_CE_FORWARD) && c.p[1].kind == AT_CSS_OFF);                  /* nobody joins a one-player mode */
    /* entering port: the player is port 3 (css->unk_0x0 = 3 + 1) */
    fresh(&c, 0xC, 0, 0, 29); at_css_set_entering(&c, 2);
    CHECK(c.p[2].kind == AT_CSS_HMN && c.p[0].kind == AT_CSS_OFF && c.p[1].kind == AT_CSS_OFF && c.p[3].kind == AT_CSS_OFF);
    c.p[2].cur = 5; press(&c, 2, AT_CI_A, 0, 65); CHECK(c.p[2].ck == 5);
    c.p[2].card = -1; c.p[2].cur = 24; press(&c, 2, 0, AT_CI_DOWN, 66); CHECK(c.p[2].card == 2);          /* its own card is card 2 */
    CHECK(press(&c, 0, AT_CI_A, 0, 67) == 0 && c.p[0].kind == AT_CSS_OFF);
    CHECK(at_css_blocker(&c, b, sizeof b) == NULL);
    /* Training: the human plus the CPU dummy */
    fresh(&c, 0x17, 0, 0, 29);
    CHECK(c.p[c.train_c].kind == AT_CSS_CPU && c.p[c.train_c].ck == AT_CK_RANDOM && c.train_h == 0 && c.train_c == 1);
    CHECK(c.p[0].kind == AT_CSS_HMN && c.p[2].kind == AT_CSS_OFF && c.p[3].kind == AT_CSS_OFF);
    CHECK(press(&c, 1, AT_CI_A, 0, 70) == 0 && c.p[1].kind == AT_CSS_CPU);                                 /* only the human's input is read */
    CHECK(press(&c, 2, AT_CI_A, 0, 71) == 0 && c.p[2].kind == AT_CSS_OFF);
    c.p[0].ck = 3; CHECK(at_css_blocker(&c, b, sizeof b) == NULL && at_css_count_in(&c) == 2);            /* the dummy counts: Random is a fighter */
    c.p[0].cur = 0; c.p[0].card = 1; press(&c, 0, AT_CI_A, 0, 72); CHECK(c.p[0].target == 1 && c.p[1].kind == AT_CSS_CPU && c.p[0].card == -1);   /* A on the dummy's card: pick its fighter */
    c.p[0].card = 1; press(&c, 0, AT_CI_Z, 0, 73); CHECK(c.p[1].kind == AT_CSS_CPU);                      /* the dummy cannot be removed */
    c.p[0].card = 2; press(&c, 0, AT_CI_A, 0, 74); CHECK(c.p[2].kind == AT_CSS_OFF);                       /* and no CPU is added in an empty card */
    at_css_set_training(&c, 2); CHECK(c.train_h == 2 && c.train_c == 0 && c.p[2].kind == AT_CSS_HMN && c.p[0].kind == AT_CSS_CPU && c.p[1].kind == AT_CSS_OFF);
    /* teams: Y on your own card with nothing picked steps the team (red, blue, green); with a fighter it steps the costume */
    fresh(&c, 0x0, 0, 1, 29); c.p[0].card = 0; c.p[0].team = 0; press(&c, 0, AT_CI_Y, 0, 75); CHECK(c.p[0].team == 1);
    press(&c, 0, AT_CI_Y, 0, 76); press(&c, 0, AT_CI_Y, 0, 77); CHECK(c.p[0].team == 0);
    fresh(&c, 0x0, 0, 0, 29); c.p[0].card = 0; c.p[0].team = 0; press(&c, 0, AT_CI_Y, 0, 78); CHECK(c.p[0].team == 0);    /* no teams: nothing */
    fresh(&c, 0x0, 0, 1, 29); CHECK(c.p[0].team == 0 && c.p[1].team == 1 && c.p[2].team == 0 && c.p[3].team == 1);       /* the legacy default: port & 1 */
}
/* Zelda and Sheik share a tile: A on the tile again swaps them */
static void legacy_rules_zelda(void)
{
    AtCss c; fresh(&c, 0x0, 0, 0, 19 + 1);              /* ck 0..19: Zelda (19) present, Sheik (20) absent: they share */
    c.p[0].cur = 19; press(&c, 0, AT_CI_A, 0, 70); CHECK(c.p[0].ck == 19);
    press(&c, 0, AT_CI_A, 0, 71); CHECK(c.p[0].ck == 20);
    press(&c, 0, AT_CI_A, 0, 72); CHECK(c.p[0].ck == 19);
    CHECK(at_css_vis_of_ck(&c, 20) == 19 && at_css_vis_of_ck(&c, 19) == 19 && at_css_vis_of_ck(&c, 7) == 7 && at_css_vis_of_ck(&c, 99) == 0);   /* Sheik lives on Zelda's tile */
    g_sheik = 0; press(&c, 0, AT_CI_A, 0, 73); CHECK(c.p[0].ck == 20);                                     /* offline the opponent's roster does not matter */
    press(&c, 0, AT_CI_A, 0, 73);
    { AtCss d; unsigned e; fresh(&d, AT_MT_LOBBY, 1, 0, 20); d.p[0].cur = 19; press(&d, 0, AT_CI_A, 0, 73);
      e = press(&d, 0, AT_CI_A, 0, 74); CHECK(d.p[0].ck == 19 && strstr(d.toast, "Sheik") != NULL && (e & AT_CE_TOAST) && (e & AT_CE_BACK));   /* online and the opponent lacks Sheik: refused */
      g_sheik = 1; press(&d, 0, AT_CI_A, 0, 75); CHECK(d.p[0].ck == 20); }
    g_sheik = 1;
    { AtCss d; fresh(&d, 0x0, 0, 0, 22);                  /* a roster with Sheik's own tile: no sharing, A on Zelda is a plain pick */
      d.p[0].cur = 19; press(&d, 0, AT_CI_A, 0, 74); press(&d, 0, AT_CI_A, 0, 75); CHECK(d.p[0].ck == 19); CHECK(at_css_vis_of_ck(&d, 20) == 20); }
    /* the Zelda swap picks a free costume of the new fighter */
    fresh(&c, 0x0, 0, 0, 20); c.p[1].kind = AT_CSS_HMN; c.p[1].ck = 20; c.p[1].costume = 0; c.p[0].cur = 19;
    press(&c, 0, AT_CI_A, 0, 76); press(&c, 0, AT_CI_A, 0, 77); CHECK(c.p[0].ck == 20 && c.p[0].costume == 1);
}
/* tabs: switching rebuilds the visible list; a tab with no tiles is not offered; the Random tile ends every tab */
static void tabs_visible(void)
{
    AtCss c;
    fresh(&c, 0x0, 0, 0, 29);
    CHECK(c.n_vis == 30 && c.n_slots == 30 && c.ck[29] == AT_CK_RANDOM && c.vis[29] == 29);              /* 29 and Random */
    at_css_set_tab(&c, 1); CHECK(c.n_vis == 27 && c.vis[26] == 29 && c.tab == 1);                       /* 26 retail and Random */
    at_css_set_tab(&c, 2); CHECK(c.n_vis == 4 && c.vis[0] == 26 && c.vis[3] == 29);                     /* 3 added and Random */
    CHECK(at_css_tab_count(&c, 0) == 29 && at_css_tab_count(&c, 1) == 26 && at_css_tab_count(&c, 2) == 3);
    CHECK(at_css_tab_offered(&c, 0) && at_css_tab_offered(&c, 1) && at_css_tab_offered(&c, 2) && !at_css_tab_offered(&c, 3));
    fresh(&c, 0x0, 0, 0, 26); at_css_set_tab(&c, 2); CHECK(c.n_vis == 1 && c.vis[0] == 26);            /* no added fighters: only Random; the caller hides the tab */
    CHECK(at_css_tab_offered(&c, 0) && !at_css_tab_offered(&c, 1) && !at_css_tab_offered(&c, 2));
    fresh(&c, 0x0, 0, 0, 40); CHECK(c.n_vis == 41);
    /* 128 fighters and the Random tile fit; more are cut, never overrun */
    fresh(&c, 0x0, 0, 0, 128); CHECK(c.n_vis == 129 && c.n_slots == 129 && c.ck[128] == AT_CK_RANDOM);
    { int ck[200], i; for (i = 0; i < 200; i++) ck[i] = i; at_css_open(&c, at_css_profile(0), &OPS, 0, 0); at_css_set_roster(&c, 200, ck, NULL, NULL); CHECK(c.n_slots == AT_CSS_MAX_SLOTS); CHECK(c.ck[c.n_slots - 1] == AT_CK_RANDOM && c.n_vis == c.n_slots); }
    at_css_open(&c, at_css_profile(0), &OPS, 0, 0); at_css_set_roster(&c, 0, NULL, NULL, NULL); CHECK(c.n_vis == 1 && c.n_slots == 1);   /* an empty roster: Random only */
    /* the cursor follows its tile through a tab change, or goes to the nearest visible one */
    fresh(&c, 0x0, 0, 0, 29); c.p[0].cur = 10; at_css_set_tab(&c, 1); CHECK(c.p[0].cur == 10);
    at_css_set_tab(&c, 2); CHECK(c.vis[c.p[0].cur] == 26);                                                /* slot 10 is not an added fighter: the nearest is the first added */
    at_css_set_tab(&c, 0); CHECK(c.vis[c.p[0].cur] == 26);
    c.p[0].cur = 29; at_css_set_tab(&c, 1); CHECK(c.vis[c.p[0].cur] == 29);                              /* Random stays Random */
    /* L and R step the offered tabs (the legacy page turn): all, retail, added, all */
    fresh(&c, 0x0, 0, 0, 29);
    CHECK((press(&c, 0, AT_CI_R, 0, 80) & AT_CE_MOVE) && c.tab == 1); press(&c, 0, AT_CI_R, 0, 81); CHECK(c.tab == 2); press(&c, 0, AT_CI_R, 0, 82); CHECK(c.tab == 0);
    press(&c, 0, AT_CI_L, 0, 83); CHECK(c.tab == 2); press(&c, 0, AT_CI_L, 0, 84); CHECK(c.tab == 1);
    fresh(&c, 0x0, 0, 0, 26); CHECK(!(press(&c, 0, AT_CI_R, 0, 85) & AT_CE_MOVE) && c.tab == 0);        /* one tab only: nothing to step */
}
/* a port that loses its controller mid-screen: its cursor and card are dropped, nothing indexes past the list */
static void ports_leave(void)
{
    AtCss c; fresh(&c, 0x0, 0, 0, 29);
    c.p[2].kind = AT_CSS_HMN; c.p[2].cur = 29; at_css_set_tab(&c, 2);
    CHECK(c.p[2].cur >= 0 && c.p[2].cur < c.n_vis);
    press(&c, 2, 0, AT_CI_RIGHT, 80); CHECK(c.p[2].cur >= 0 && c.p[2].cur < c.n_vis);
    /* a wild cursor (past the list, or negative) is brought home before it is read */
    c.p[2].cur = 9999; press(&c, 2, 0, 0, 81); CHECK(c.p[2].cur >= 0 && c.p[2].cur < c.n_vis);
    c.p[2].cur = -7; press(&c, 2, 0, AT_CI_DOWN, 82); CHECK(c.p[2].cur >= 0 && c.p[2].cur < c.n_vis);
    /* a bad port number is ignored */
    CHECK(press(&c, -1, AT_CI_A, 0, 83) == 0 && press(&c, 4, AT_CI_A, 0, 84) == 0);
    /* no columns set yet (0): treated as one column, never a divide by zero */
    fresh(&c, 0x0, 0, 0, 29); at_css_set_cols(&c, 0); press(&c, 0, 0, AT_CI_DOWN, 85); CHECK(c.p[0].cur >= 0 && c.p[0].cur < c.n_vis);
    /* two ports on one tile: both may pick it (a cursor is not a lock) */
    fresh(&c, 0x0, 0, 0, 29); c.p[1].kind = AT_CSS_HMN; c.p[0].cur = c.p[1].cur = 5; press(&c, 0, AT_CI_A, 0, 86); press(&c, 1, AT_CI_A, 0, 87);
    CHECK(c.p[0].ck == 5 && c.p[1].ck == 5 && c.p[0].costume != c.p[1].costume);                        /* same fighter, different costumes */
}
/* the lobby profile: one card; nothing but port 1 plays; a fighter the opponent lacks cannot be picked */
static void lobby_rules(void)
{
    AtCss c; unsigned e; fresh(&c, AT_MT_LOBBY, 1, 0, 29);
    CHECK(c.p[0].kind == AT_CSS_HMN && c.p[1].kind == AT_CSS_OFF && c.online == 1);
    c.ok[5] = 0;
    c.p[0].cur = 5; e = press(&c, 0, AT_CI_A, 0, 90); CHECK((e & AT_CE_BACK) && c.p[0].ck == AT_CK_NONE && strstr(c.toast, "opponent") != NULL && (e & AT_CE_TOAST));
    CHECK(strcmp(c.toast, "Your opponent doesn't have this fighter - pick another.") == 0);
    CHECK((press(&c, 1, AT_CI_A, 0, 91) & AT_CE_FORWARD) == 0 && c.p[1].kind == AT_CSS_OFF);
    c.p[0].cur = 6; e = press(&c, 0, AT_CI_A, 0, 92); CHECK((e & AT_CE_FORWARD) && c.p[0].ck == 6);
    e = press(&c, 0, AT_CI_START, 0, 93); CHECK(e & AT_CE_FINISH_GO);
    /* Random never rolls onto a fighter the opponent lacks: the caller's roll is the op's; the Random tile itself is always available */
    fresh(&c, AT_MT_LOBBY, 1, 0, 29); c.p[0].cur = c.n_vis - 1; press(&c, 0, AT_CI_A, 0, 94); CHECK(c.p[0].ck == AT_CK_RANDOM);
}
/* the mouse: the cursor follows the pointer, a click is A, a right click B, the wheel turns costumes over a card or the tab over the grid */
static void mouse_rules(void)
{
    AtCss c; unsigned b; fresh(&c, 0x0, 0, 0, 29);
    CHECK(at_css_mouse_port(&c) == 0);
    c.p[0].kind = AT_CSS_OFF; c.p[2].kind = AT_CSS_HMN; CHECK(at_css_mouse_port(&c) == 2);               /* the first human port */
    c.p[0].kind = AT_CSS_HMN;
    b = at_css_mouse_bits(&c, 0, 7, -1, 1, 0, 0, 0); CHECK(c.p[0].cur == 7 && c.p[0].card == -1 && b == 0);   /* hovering moves the cursor */
    b = at_css_mouse_bits(&c, 0, 9, -1, 0, 1, 0, 0); CHECK(c.p[0].cur == 9 && (b & AT_CI_A));              /* a click picks */
    b = at_css_mouse_bits(&c, 0, -1, -1, 0, 0, 1, 0); CHECK(b == AT_CI_B);                                   /* a right click is B */
    b = at_css_mouse_bits(&c, 0, -1, 2, 1, 0, 0, 0); CHECK(c.p[0].card == 2 && b == 0);                     /* pointing at a card puts the cursor on it */
    b = at_css_mouse_bits(&c, 0, -1, 2, 0, 1, 0, 0); CHECK(b == AT_CI_A);                                   /* a click on another port's card: A (add a CPU) */
    c.p[0].ck = 4; b = at_css_mouse_bits(&c, 0, -1, 0, 0, 1, 0, 0); CHECK(b & AT_CI_X);                      /* a click on your own card: the next costume */
    c.p[1].kind = AT_CSS_CPU; b = at_css_mouse_bits(&c, 0, -1, 1, 0, 0, 1, 0); CHECK((b & AT_CI_Z) && !(b & AT_CI_B));   /* a right click on a CPU's card removes it */
    b = at_css_mouse_bits(&c, 0, -1, 1, 0, 0, 0, 1); CHECK(b & AT_CI_X);
    b = at_css_mouse_bits(&c, 0, -1, 1, 0, 0, 0, -1); CHECK(b & AT_CI_Y);
    b = at_css_mouse_bits(&c, 0, 3, -1, 0, 0, 0, 1); CHECK(b & AT_CI_L);                                    /* over the grid the wheel turns the tab */
    b = at_css_mouse_bits(&c, 0, 3, -1, 0, 0, 0, -1); CHECK(b & AT_CI_R);
    fresh(&c, 0x0, 0, 0, 26); b = at_css_mouse_bits(&c, 0, 3, -1, 0, 0, 0, 1); CHECK(!(b & (AT_CI_L | AT_CI_R)));   /* one tab: the wheel does nothing */
    /* a port that is not a player yet joins on a click on a tile or on its own card */
    fresh(&c, 0x0, 0, 0, 29); b = at_css_mouse_bits(&c, 1, 3, -1, 0, 1, 0, 0); CHECK(b & AT_CI_A);
    b = at_css_mouse_bits(&c, 1, -1, 1, 0, 1, 0, 0); CHECK(b & AT_CI_A);
    b = at_css_mouse_bits(&c, 1, -1, 2, 0, 1, 0, 0); CHECK(!(b & AT_CI_A));
    /* Training: the mouse never moves onto the cards of other ports, and a click on any card is the costume */
    fresh(&c, 0x17, 0, 0, 29); c.p[0].ck = 3;
    b = at_css_mouse_bits(&c, 0, -1, 1, 1, 0, 0, 0); CHECK(c.p[0].card == -1);
    b = at_css_mouse_bits(&c, 0, -1, 1, 0, 1, 0, 0); CHECK(b & AT_CI_X);
    CHECK(at_css_mouse_port(&c) == c.train_h);
    /* online: the one card; no card pointing */
    fresh(&c, AT_MT_LOBBY, 1, 0, 29); b = at_css_mouse_bits(&c, 0, -1, 0, 1, 0, 0, 0); CHECK(c.p[0].card == -1); CHECK(at_css_mouse_port(&c) == 0);
}
/* what the mode's exit handler reads is made by the adapter from the ports below; the model's contract is that these are consistent at FINISH_GO */
static void finish_state(void)
{
    AtCss c; fresh(&c, 0x0, 0, 0, 29);
    c.p[0].ck = 3; c.p[0].costume = 2; c.p[1].kind = AT_CSS_CPU; c.p[1].ck = AT_CK_RANDOM; c.p[1].cpu_lv = 7;
    CHECK(press(&c, 0, AT_CI_START, 0, 100) & AT_CE_FINISH_GO);
    CHECK(c.p[0].ck == 3 && c.p[0].costume == 2 && c.p[1].ck == AT_CK_RANDOM && c.p[1].cpu_lv == 7 && c.p[1].kind == AT_CSS_CPU && c.done);   /* nothing changed on the way out */
    CHECK(c.ops->random_ck(c.ops->user, &c) == c.ck[0]);                                                  /* the caller rolls Random through the op */
}

/* Task 10: what each group's finish hands the (unchanged, game-side) legacy writer. The writer (fs_css_finish) writes slot_type, ckind, color, cpu_level and team of the
 * ports that are in, marks the others NA, and never touches stocks or nametag (tools/port/test_css_hooks.py pins that on the source); the model's contract is that
 * at FINISH_GO the ports are exactly what the mode asked for. Each group is checked with its own profile. */
static void finish_leaves_mode_fields_group1(void)
{
    AtCss c; int mt;
    for (mt = 0; mt <= 0xA; mt++) {                                      /* the VS family: four ports, CPUs, teams from the rules */
        fresh(&c, mt, 0, 1, 29);
        c.p[0].ck = 3; c.p[0].costume = 1; c.p[1].kind = AT_CSS_CPU; c.p[1].ck = 8; c.p[1].cpu_lv = 5; c.p[1].team = 2;
        c.p[2].kind = AT_CSS_HMN; c.p[2].ck = AT_CK_NONE;                 /* a human with no fighter blocks */
        CHECK(!(press(&c, 0, AT_CI_START, 0, 100) & AT_CE_FINISH_GO) && !c.done);
        c.p[2].kind = AT_CSS_OFF;
        CHECK(press(&c, 0, AT_CI_START, 0, 101) & AT_CE_FINISH_GO);
        CHECK(c.p[0].kind == AT_CSS_HMN && c.p[0].ck == 3 && c.p[0].costume == 1 && c.p[1].kind == AT_CSS_CPU && c.p[1].ck == 8 && c.p[1].cpu_lv == 5 && c.p[1].team == 2);
        CHECK(c.p[2].kind == AT_CSS_OFF && c.p[3].kind == AT_CSS_OFF && c.teams == 1);
    }
    fresh(&c, 0x17, 0, 0, 29);                                           /* Training and the LAB profile: the human and the dummy, nothing else */
    c.p[0].ck = 5; press(&c, 0, AT_CI_START, 0, 102);
    CHECK(c.done && c.p[0].kind == AT_CSS_HMN && c.p[0].ck == 5 && c.p[1].kind == AT_CSS_CPU && c.p[1].ck == AT_CK_RANDOM && c.p[2].kind == AT_CSS_OFF && c.p[3].kind == AT_CSS_OFF);
}
static void finish_leaves_mode_fields_one_player(void)
{
    AtCss c; int mt, port;
    for (mt = 0xB; mt <= 0x16; mt++) {                                    /* Classic, Adventure, All-Star, Event, every Stadium mode */
        for (port = 0; port < 4; port++) {
            fresh(&c, mt, 0, 0, 29);
            at_css_set_entering(&c, port);
            c.p[port].cur = 6; press(&c, port, AT_CI_A, 0, 110);
            CHECK(c.p[port].ck == 6);
            CHECK(press(&c, port, AT_CI_START, 0, 111) & AT_CE_FINISH_GO);
            CHECK(c.p[port].kind == AT_CSS_HMN && c.p[port].ck == 6);     /* the slot the exit handler reads: its fighter and costume */
            CHECK(at_css_count_in(&c) == 1);                              /* nobody else is in: the legacy writer marks every other port NA */
        }
        fresh(&c, mt, 0, 0, 29); at_css_set_entering(&c, 1);              /* Back: B twice leaves; the state's exit handler sees pending_scene_change 2 */
        press(&c, 1, AT_CI_B, 0, 120); CHECK(press(&c, 1, AT_CI_B, 0, 121) & AT_CE_FINISH_BACK);
    }
    /* a one-player mode that arrives with a fighter already chosen (the mode's last pick) can start at once */
    fresh(&c, 0xB, 0, 0, 29); at_css_set_entering(&c, 0); c.p[0].ck = 9; c.p[0].costume = 2;
    CHECK(press(&c, 0, AT_CI_START, 0, 130) & AT_CE_FINISH_GO);
    CHECK(c.p[0].ck == 9 && c.p[0].costume == 2);
}

static void skins255_stepper(void)
{
    AtCss c; int i;
    g_costumes = 255;
    fresh(&c, 0x0, 0, 0, 29);
    c.p[0].ck = 0;
    for (i = 0; i < 255; ++i) {
        CHECK(at_css_free_costume(&c, 0, 0, i, 1) == i);
        c.p[0].costume = i;
    }
    CHECK(at_css_free_costume(&c, 0, 0, 255, 1) == 0);
    CHECK(at_css_free_costume(&c, 0, 0, -1, -1) == 254);
    c.p[1].kind = AT_CSS_CPU; c.p[1].ck = 0; c.p[1].costume = 128;
    CHECK(at_css_free_costume(&c, 0, 0, 128, 1) == 129);
    g_costumes = 4;
}
int main(void)
{
    skins255_stepper();
    ports_join(); legacy_rules_pick_undo_back(); legacy_rules_cards(); legacy_rules_costume(); legacy_rules_start();
    profile_rules(); legacy_rules_zelda(); tabs_visible(); ports_leave(); lobby_rules(); mouse_rules(); finish_state(); finish_leaves_mode_fields_group1(); finish_leaves_mode_fields_one_player();
    ATLAS_DONE("atlas css");
}
