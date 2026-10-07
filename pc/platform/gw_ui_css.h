/* gw_ui_css.h - the character select's rules as a pure C model: four ports, cursors, cards, picks, costumes, CPU levels, teams, the start
 * blocker and the result. No game, no Lua, no renderer, no libc header (game-side files include this and the PowerPC syntax check runs with
 * -nostdinc). The legacy rules are gmfrontend_select.inc's fs_css_port, fs_pick, fs_css_blocker and fs_free_costume: each is one named check in
 * pc/tests/atlas_css_test.c and the legacy code wins where they disagree. A mode's differences are data (gw_ui_css_profile.h).
 *
 * The model returns what a step did as bits (AT_CE_*): the caller turns them into sound, a toast and the end of the scene. It reads nothing
 * from the game: the fighters' costume counts, the Random roll, the Zelda/Sheik pair and the online roster check come in through AtCssOps. */
#ifndef GW_UI_CSS_H
#define GW_UI_CSS_H
#include "gw_ui_css_profile.h"
#ifdef __cplusplus
extern "C" {
#endif

#define AT_CSS_MAX_SLOTS 130          /* 128 fighters (FS_MAX_FIGHTERS) and the Random tile */
enum { AT_CSS_OFF, AT_CSS_HMN, AT_CSS_CPU };
enum { AT_CK_NONE = -1, AT_CK_RANDOM = -2 };
/* input bits for one port and one frame: held-with-repeat directions and edge-triggered buttons (the legacy FsIn.rep and .trig) */
enum { AT_CI_LEFT = 1, AT_CI_RIGHT = 2, AT_CI_UP = 4, AT_CI_DOWN = 8, AT_CI_A = 16, AT_CI_B = 32, AT_CI_X = 64, AT_CI_Y = 128,
       AT_CI_Z = 256, AT_CI_L = 512, AT_CI_R = 1024, AT_CI_START = 2048 };
typedef struct { unsigned trig, rep; } AtCssIn;
/* what one step did, for the caller to turn into sound, toast and scene end (MOVE: sfxMove, FORWARD: sfxForward, BACK: sfxBack) */
enum { AT_CE_MOVE = 1, AT_CE_FORWARD = 2, AT_CE_BACK = 4, AT_CE_TOAST = 8, AT_CE_FINISH_GO = 16, AT_CE_FINISH_BACK = 32 };
/* cur: an index into the visible list; card -1 = on the grid, else the band card (0..3) the cursor is on; target: the port its picks go to */
typedef struct { int kind, ck, costume, cpu_lv, team, cur, card, target; } AtCssPort;

struct AtCss;
typedef struct AtCssOps {
    void *user;
    int (*costumes)(void *u, int ck);                   /* how many costumes the fighter has (fs_costumes), at least 1 */
    int (*random_ck)(void *u, const struct AtCss *c);   /* a fighter for the Random tile (fs_random_ck) */
    int (*zelda_swap)(void *u, int ck);                 /* the other of the Zelda/Sheik pair for either of them, else -1 */
    int (*sheik_ok)(void *u);                           /* online: the opponent has Sheik */
} AtCssOps;

typedef struct AtCss {
    const AtCssProfile *prof;
    int online, teams;
    /* the roster: n_slots tiles, the last one the Random tile (ck AT_CK_RANDOM, in every tab). tab_of: 1 retail, 2 added, 0 every tab. */
    int n_slots, ck[AT_CSS_MAX_SLOTS];
    unsigned char ok[AT_CSS_MAX_SLOTS], tab_of[AT_CSS_MAX_SLOTS];
    int tab;                                            /* 0 = all, 1 retail, 2 added */
    int vis[AT_CSS_MAX_SLOTS], n_vis, cols;             /* the visible list for the tab (slots), and the grid's columns (set by the caller) */
    AtCssPort p[4];
    int train_h, train_c;                               /* Training: the human port and the dummy's port */
    int done, back_until; char toast[80];
    const AtCssOps *ops;
} AtCss;

void at_css_open(AtCss *c, const AtCssProfile *prof, const AtCssOps *ops, int online, int teams);
/* n fighters (at most AT_CSS_MAX_SLOTS - 1); the Random tile is added; the tab is reset to ALL. tab_of may be NULL (all retail), ok may be NULL (all available). */
void at_css_set_roster(AtCss *c, int n, const int *ck, const unsigned char *ok, const unsigned char *tab_of);
void at_css_set_tab(AtCss *c, int tab);                 /* rebuilds vis; every cursor goes to the same tile, or the nearest visible one */
void at_css_set_cols(AtCss *c, int cols);
int at_css_tab_count(const AtCss *c, int tab);          /* the fighters (not Random) a tab holds; tab 0 counts every one */
int at_css_tab_offered(const AtCss *c, int tab);        /* ALL always; RETAIL and ADDED only when the roster has added fighters (a tab with no tiles is not offered) */
void at_css_set_entering(AtCss *c, int port);           /* a one-player mode: only this port plays (the others are off) */
void at_css_set_training(AtCss *c, int human);          /* Training: this port is the human, the other of ports 0 and 1 the CPU dummy (Random until picked) */
int at_css_vis_of_ck(const AtCss *c, int ck);           /* the index in the visible list of the tile showing ck (Sheik: Zelda's tile when they share), else 0 */
unsigned at_css_step(AtCss *c, int port, AtCssIn in, int frame);              /* one port, one frame: AT_CE_* bits */
const char *at_css_blocker(const AtCss *c, char *buf, int cap);               /* NULL when the match can start */
int at_css_free_costume(const AtCss *c, int port, int ck, int from, int dir); /* fs_free_costume */
int at_css_count_in(const AtCss *c);
/* fs_mouse_css without the hit test: the caller says which visible tile (-1) and which band card (-1) the pointer is on and what the mouse did
 * this frame; the cursor is moved here (hover and click) and the buttons the mouse stands for are returned (OR them into that port's trig). */
unsigned at_css_mouse_bits(AtCss *c, int port, int vis_slot, int card, int moved, int click, int rclick, int wheel);
int at_css_mouse_port(const AtCss *c);                  /* the one port the mouse plays for (fs_mouse_port): the first human, the human in Training, 0 online */

#ifdef __cplusplus
}
#endif
#endif
