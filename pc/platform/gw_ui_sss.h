/* gw_ui_sss.h - the stage select's rules as a pure C model (the legacy rules are fs_sss_open, fs_sss_frame and fs_random_stage in
 * gmfrontend_select.inc). No game, no Lua, no libc header. The pages became tabs and scrolling. The model never decides a strike or a ban: the
 * lobby (step 6) marks cells through the cell flags it submits and this model only knows which stage is under the cursor. */
#ifndef GW_UI_SSS_H
#define GW_UI_SSS_H
#include "gw_ui_css.h"   /* the AT_CE_* result bits and AT_CI_* input bits are shared with the character select */
#ifdef __cplusplus
extern "C" {
#endif

#define AT_SSS_MAX 256
/* type: 1 an icon, 2 an icon with art, 3 the Random icon; tab_of: 1 retail, 2 added, 3 tournament (the online Stage List's legal six); ok 0 = locked */
typedef struct { int ext, type, row; unsigned char ok, tab_of; } AtSssStage;
typedef struct AtSss {
    int n, tab, cur, cols, done, go, pick_ext;
    AtSssStage st[AT_SSS_MAX];
    int vis[AT_SSS_MAX], n_vis;
    int online;                                    /* the lobby uses the same grid; strikes are drawn through cell flags, never decided here */
    char toast[48];
} AtSss;

void at_sss_open(AtSss *s, int online);
/* The stages as the disc lists them. A type 0 entry and an empty layout slot (type 1 with ext <= 0) are dropped, the Random icons collapse into ONE
 * Random tile kept last (Akaneia has two), and at most AT_SSS_MAX - 1 stages are kept. The tab is reset to ALL. */
void at_sss_set_stages(AtSss *s, int n, const AtSssStage *st);
void at_sss_set_tab(AtSss *s, int tab);            /* 0 all, 1 retail, 2 added, 3 tournament; the cursor stays on its stage or goes to the nearest */
void at_sss_set_cols(AtSss *s, int cols);
int at_sss_tab_count(const AtSss *s, int tab);     /* the stages (not Random) a tab holds; tab 0 counts every one */
int at_sss_tab_offered(const AtSss *s, int tab);   /* ALL always; the others only when they hold a stage (RETAIL only beside another tab) */
/* Which stages are "added": an m-ex external id from 288 up (fs_stage_unlocked's own rule: those stages have no unlock bits on any save, the retail 0..287 do). 1 retail, 2 added. */
int at_sss_tab_for_ext(int ext);
void at_sss_cursor_ext(AtSss *s, int ext);         /* the cursor onto the stage with this external id (the rules' stage), if it is visible */
/* One frame of the merged input of every port. Returns AT_CE_* bits; on AT_CE_FINISH_GO `go` is 1 and `pick_ext` the external id (for the Random tile the
 * `random_pick` the caller rolled with at_sss_pick_random); on AT_CE_FINISH_BACK `go` stays 0. */
unsigned at_sss_step(AtSss *s, unsigned trig, unsigned rep, int random_pick);
int at_sss_pick_random(const AtSss *s, int r);     /* the r-th unlocked, non-random stage's ext (fs_random_stage with the dice injected); 31 (Battlefield) when none */
int at_sss_random_pool(const AtSss *s);            /* how many stages a Random roll may land on: the range of r */
unsigned at_sss_mouse_bits(AtSss *s, int vis_index, int moved, int click, int wheel);   /* hover moves the cursor, a click is A, the wheel steps the tab */

#ifdef __cplusplus
}
#endif
#endif
