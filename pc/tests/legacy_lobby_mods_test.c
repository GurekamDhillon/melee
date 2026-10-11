/* Legacy counterpart of atlas_room_host_test.c. Runs the actual lobby builder,
 * signature, grid and mouse hit testing against a recording kit sink. A missing
 * panel, a stale row hidden by a toast, or overlapping stage cells must fail.
 * No game process, renderer, window, exported test accessor or retail assets. */
#include <stdbool.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include "atlas_check.h"
#include "legacy_lobby_capacity.h"

#define FE_STR 80
#define FP_ID 32
#define FE_NP_MAX_CK 256
#define LB_MAX_UI_STAGES 32
enum { FL_NONE, FL_CODE, FL_WAIT, FL_LOBBY, FL_LOAD, FL_CSS, FL_SSS };
enum { FE_NP_IDLE, FE_NP_WORKING, FE_NP_LOBBY, FE_NP_CONNECTED };
enum { LBP_OFF, LBP_CHAR_BLIND, LBP_STRIKE, LBP_BAN, LBP_PICK, LBP_READY };
enum { KS_VERSUS, FPR_OTHER };
typedef struct { int unused; } FjNode;
typedef struct {
    const FjNode *el, *tstates;
    float ox, oy;
    char idx[4], port[4], pre[8];
    int who;
} FlEl;

/* The capacity is read from production by the fixture runner. */
static struct {
    int art, frames, cur, coin_until, leave_until, ready_on_until, copied_until, invalid;
    char sig[LEGACY_SIG_CAP], code_msg[FE_STR];
} fl;
static struct { int ready, n, ck[4]; } fl_ld;
static struct { int ok; } fe_np_char_icon[FE_NP_MAX_CK];
static int fe_np_phase = FE_NP_LOBBY, host, nmods, stages = 6, starters = 6;
static int ready, attached, phase = LBP_STRIKE;
static char mods[16][FE_STR], mod_line[FE_STR], toast[FE_STR];
static int rows_requested, last_max_rows;
static float mouse_x, mouse_y;

typedef struct { char id[FP_ID], text[FE_STR]; float x0, y0, x1, y1; } Draw;
static Draw draw[100];
static int ndraw, nhints;
static char hint_glyph[4][24], hint_text[4][24];

static void fp_copy(char *out, const char *s, int cap) { snprintf(out, (size_t) cap, "%s", s); }
static int fl_reconnecting(void) { return fe_np_phase != FE_NP_LOBBY && fe_np_phase != FE_NP_CONNECTED; }
static int fa_room_on(void) { return attached; }
static void fa_room_grid(int n, int s) { (void) n; (void) s; }
static int Netplay_LobbyInfo(int key) { return key == 9 ? stages : key == 12 ? ready : key == 0 ? 1 : 0; }
static int Netplay_LobbyStageGroup(int i) { return i >= starters; }
static int Netplay_IsHost(void) { return host; }
static int Netplay_LobbyMe(void) { return host ? 0 : 1; }
static int Netplay_LobbyPlayer(int who, int key) { (void) who; (void) key; return 0; }
static int Netplay_LobbySeq(void) { return 1; }
static int Netplay_Ping(void) { return 10; }
static void Netplay_RoomCode(char *out, int cap) { fp_copy(out, "TEST", cap); }
static void Netplay_MenuStatus(char *out, int cap) { fp_copy(out, "Waiting", cap); }
static int Netplay_RandomStatus(void) { return 0; }
static int Netplay_RandomSeconds(void) { return 0; }
static int Netplay_CodeChar(int i) { (void) i; return 'A'; }
static int Netplay_CodeSlot(void) { return 0; }
static int Netplay_Envoy(void) { return 0; }
static int Netplay_Turbo(void) { return 0; }
static int Netplay_ModsRowCount(void) { return nmods; }
static void Netplay_ModsLine(char *out, int cap) { fp_copy(out, mod_line, cap); }
static int Netplay_ModsRow(int i, int max_rows, char *out, int cap)
{
    rows_requested++; last_max_rows = max_rows;
    out[0] = '\0';
    if (i < 0 || i >= nmods || i >= max_rows) return 0;
    if (i == max_rows - 1 && nmods > max_rows)
        snprintf(out, (size_t) cap, "+%d more (see the log)", nmods - i);
    else fp_copy(out, mods[i], cap);
    return 1;
}
static int fl_phase(void) { return fl_reconnecting() ? LBP_OFF : phase; }
static int fl_can_pick_char(void) { return phase == LBP_CHAR_BLIND; }
static int fl_my_stage_turn(void) { return !fl_reconnecting() && phase == LBP_STRIKE; }
static int fl_ck(int who) { (void) who; return 0; }
static const char *fl_toast_now(void) { return toast[0] ? toast : NULL; }
static void fl_instruction(char *out) { fp_copy(out, "Strike a stage", FE_STR); }
static void fl_status_line(char *out) { fp_copy(out, toast[0] ? toast : mod_line, FE_STR); }
static void fs_css_sig(char *sig) { (void) sig; }
static void fs_sss_sig(char *sig) { (void) sig; }
static const char *fl_action_state(void) { return NULL; }
static const char *fl_tile_state(int i) { (void) i; return "free"; }
static void fl_grid_draw(void) { }
static void OSReport(const char *fmt, ...) { (void) fmt; }
static const FjNode *fj_get(const FjNode *n, const char *key) { (void) n; (void) key; return NULL; }
static const FjNode *fj_idx(const FjNode *n, int i) { (void) n; (void) i; return NULL; }
static float fj_num(const FjNode *n, float def) { (void) n; return def; }
static const FjNode *fj_load(const char *file, void **t, void **p) { (void) file; *t = *p = NULL; return NULL; }
static void fl_el(FlEl *e) { (void) e; }
static void HSD_Free(void *p) { (void) p; }
static int fms_in_rect(const float *r) { return mouse_x >= r[0] && mouse_x <= r[2] && mouse_y >= r[1] && mouse_y <= r[3]; }
static void fm_tok(const char *name, int section, float *rgb) { (void) name; (void) section; rgb[0] = rgb[1] = rgb[2] = 1; }
static int fm_panel(const char *id, int role, int tile, float x0, float y0, float x1, float y1, const float *rgb)
{
    Draw *d = &draw[ndraw++]; (void) role; (void) tile; (void) rgb;
    fp_copy(d->id, id, FP_ID); d->x0 = x0; d->y0 = y0; d->x1 = x1; d->y1 = y1;
    return ndraw - 1;
}
static int fl_text_at(const char *id, const char *role, float x, float y, const char *align, float width, const char *s, const float *rgb, int joint)
{
    Draw *d = &draw[ndraw++]; (void) role; (void) align; (void) rgb; (void) joint;
    fp_copy(d->id, id, FP_ID); fp_copy(d->text, s, FE_STR);
    d->x0 = x; d->x1 = x + width; d->y0 = y - 12; d->y1 = y + 2;
    return ndraw - 1;
}
static void fm_chrome_with(const char *title, const char **crumbs, int nc, int section, int nh, const char **hg, const char **ht)
{
    int i; (void) title; (void) crumbs; (void) nc; (void) section;
    CHECK(nh <= 4); nhints = nh;
    for (i = 0; i < nh; i++) { fp_copy(hint_glyph[i], hg[i], 24); fp_copy(hint_text[i], ht[i], 24); }
}

#include "legacy_lobby_functions.inc"

static const Draw *text(const char *s)
{
    int i;
    for (i = 0; i < ndraw; i++) if (strcmp(draw[i].text, s) == 0) return &draw[i];
    return NULL;
}
static int hint(const char *glyph)
{
    int i;
    for (i = 0; i < nhints; i++) if (strcmp(hint_glyph[i], glyph) == 0) return i;
    return -1;
}
static void build(void)
{
    char sig[LEGACY_SIG_CAP];
    memset(draw, 0, sizeof draw); ndraw = nhints = rows_requested = 0;
    fl_signature(sig);
    fl_build_lobby();
}

int main(void)
{
    int i;
    char a[LEGACY_SIG_CAP], b[LEGACY_SIG_CAP];
    const Draw *row;
    fl.art = FL_LOBBY; ready = 1;
    for (i = 0; i < 16; i++) snprintf(mods[i], FE_STR, "Mod %c - different version", 'A' + i);
    fp_copy(mod_line, "Y: match the host's mods", FE_STR);
    nmods = 2;
    build();
    CHECK(text("MODS DIFFER FROM THE OTHER PLAYER'S") != NULL);
    CHECK(text(mods[0]) != NULL && text(mods[1]) != NULL);
    CHECK(last_max_rows == 5);
    CHECK(hint("glyph_y") >= 0 && hint("glyph_b") >= 0 && hint("glyph_a") >= 0);
    row = text(mods[1]);
    if (row) CHECK(flg.r[0][1] > row->y1);
    for (i = 0; i < stages; i++) {
        CHECK(flg.r[i][1] >= 280 && flg.r[i][3] <= 394);
        mouse_x = (flg.r[i][0] + flg.r[i][2]) / 2;
        mouse_y = (flg.r[i][1] + flg.r[i][3]) / 2;
        CHECK(fl_mouse_tile(i) == i);
    }

    /* Five literal rows fit; the sixth changes the last line to the accessor's summary. */
    nmods = 5; build();
    CHECK(text(mods[4]) != NULL);
    nmods = 9; build();
    CHECK(text(mods[3]) != NULL && text("+5 more (see the log)") != NULL);
    CHECK(text(mods[4]) == NULL);
    CHECK(flg.r[0][1] >= 344 && flg.r[0][3] <= 394);
    host = 1; build();
    CHECK(text(mods[0]) != NULL && hint("glyph_y") < 0);

    /* Cosmetic-only diffs have rows but no actionable ModsLine: do not promise Y. */
    host = 0; nmods = 2; mod_line[0] = '\0';
    fp_copy(mods[0], "Mod A - only you have it (cosmetic, fine)", FE_STR);
    build();
    CHECK(text(mods[0]) != NULL && hint("glyph_y") < 0);
    CHECK(text("Y: Match the host's mods (restart and rejoin).") == NULL);
    CHECK(text("These differences are fine; a match can start.") != NULL);
    fp_copy(mod_line, "Y: match the host's mods", FE_STR);

    /* A toast masks the one-line status but must never freeze a changing mod list. */
    host = 0; nmods = 5;
    fp_copy(toast, "Press B again to leave the room.", FE_STR);
    for (i = 0; i < 5; i++) { memset(mods[i], 'x', FE_STR - 1); mods[i][FE_STR - 1] = '\0'; }
    fl_signature(a);
    mods[4][FE_STR - 2] = 'y';
    fl_signature(b);
    CHECK(strcmp(a, b) != 0);
    nmods = 0; mod_line[0] = '\0'; build();
    CHECK(ndraw == 0 && hint("glyph_y") < 0);
    CHECK_NEAR(flg.r[0][1], 214);

    /* Reconnect clears a stale peer list; a sync status with no rows keeps the action visible. */
    nmods = 2; fp_copy(mod_line, "Mods differ", FE_STR);
    fe_np_phase = FE_NP_WORKING; build();
    CHECK(ndraw == 0 && hint("glyph_y") < 0);
    fe_np_phase = FE_NP_LOBBY; nmods = 0;
    build(); CHECK(ndraw == 0 && hint("glyph_y") >= 0);

    /* Long grouped lists stay paginated, bounded and clickable below the panel. */
    nmods = 9; stages = 32; starters = 5; build();
    CHECK(flg.n == 32 && flg.pages > 1);
    for (i = 0; i < stages; i++) {
        CHECK(flg.r[i][1] >= 344 && flg.r[i][3] <= 394);
        mouse_x = (flg.r[i][0] + flg.r[i][2]) / 2;
        mouse_y = (flg.r[i][1] + flg.r[i][3]) / 2;
        CHECK(fl_mouse_tile(i) == i);
    }
    ATLAS_DONE("legacy-lobby-mods");
}
