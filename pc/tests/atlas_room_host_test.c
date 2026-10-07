/* Host-side test of the native online room screen (gw_script_ui_room.inc): the Ui_Room* shims the game-side adapter calls, driven the way it drives them, against the real render, stack and slots.
 * gw_script_ui.inc is included against the same stand-ins atlas_settings_host_test.c uses. No game, no window. Reaches the host statics directly. */
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>
#include "lua.h"
#include "lauxlib.h"
#include "lualib.h"
#include "gw_kit.h"
#include "gw_screen_model.h"
#include "atlas_check.h"
#include "atlas_fake.h"
#include "atlas_rec.h"

typedef struct { char id[64]; int used, disabled, gameplay; } GsScript;
static struct { lua_State *L; int cur, console, n, scene_kind, match_active; GsScript s[8]; unsigned char key_now[256]; } gs;
static int g_may_run = 1, g_quads, g_art_calls, g_art_ret = 5;
static double g_now = 1000.0;
static float g_track;
static unsigned g_pad;
static char g_last[512];
static char g_art_key[64];

void gw_log(const char *fmt, ...) { (void) fmt; }
static int gs_may_run(int i) { (void) i; return g_may_run; }
int gw_RB_Enabled(void) { return 0; }
static int gs_ui_port_read(int port, char *name, int cap, int *percent, int *stocks, int *cpu)   /* the fighter readbacks gd.player uses */
{
    (void) port; (void) cap; name[0] = 0; *percent = *stocks = *cpu = 0;
    return 0;
}
int gw_Netplay_Enabled(void) { return 0; }
int gw_Mods_Count(void) { return 0; }
int gw_Mods_MenuCount(int i) { (void) i; return 0; }
const char *gw_Mods_MenuField(int i, int k, const char *f) { (void) i; (void) k; (void) f; return ""; }
const char *gw_Mods_Id(int i) { (void) i; return ""; }
static int gs_pcall(int script, int nargs, int nres, const char *what)
{
    int old = gs.cur, rc = 0;
    (void) what;
    gs.cur = script;
    if (lua_pcall(gs.L, nargs, nres, 0) != LUA_OK) { lua_pop(gs.L, 1); rc = -1; }
    gs.cur = old;
    return rc;
}
static int gs_get_hook(int i, const char *name)
{
    if (!gs.s[i].used || gs.s[i].disabled) return 0;
    lua_getglobal(gs.L, name);
    if (lua_isfunction(gs.L, -1)) return 1;
    lua_pop(gs.L, 1);
    return 0;
}
static const GsScript *gs_cur_script(void) { return &gs.s[gs.cur]; }
static double gs_now_ms(void) { return g_now; }
static double gs_ui_clock;
static struct { int token; } gs_stage_models[4];
static int gs_stage_nmodels;
int gw_ScriptGame_ModelRefOwned(int a, int b, int c, int d) { (void) a; (void) b; (void) c; (void) d; return 0; }
int gw_Script_StageResourceOwner(void) { return 0; }
static int gs_sm_emit(int asset, const GsmParams *p, int clip, const char **why) { (void) asset; (void) p; (void) clip; (void) why; return 0; }
void gw_script_pad_state(int ch, unsigned *b, int *sx, int *sy, int *cx, int *cy, int *tl, int *tr) { (void) ch; *b = 0; *sx = *sy = *cx = *cy = *tl = *tr = 0; }
unsigned gw_script_pad_raw_buttons(int ch) { (void) ch; return g_pad; }
static float g_mx = -1000.0f, g_my = -1000.0f, g_mwheel; static int g_mbuttons;
void gw_Mouse_ScriptRead(float *x, float *y, int *buttons, float *wheel) { *x = g_mx; *y = g_my; *buttons = g_mbuttons; *wheel = g_mwheel; }
float gw_Console_ScriptWidth(void) { return 640.0f; }
int gw_Settings_Int(const char *k, int d) { (void) k; return d; }
static void gs_prof_setfuncs(lua_State *L, const luaL_Reg *funcs, const char *prefix) { (void) prefix; luaL_setfuncs(L, funcs, 0); }
static int gs_kit_record(int first, int added) { (void) first; return added; }
static double gs_kit_optnum(lua_State *L, int t, const char *k, double def)
{
    double v = def;
    if (!lua_istable(L, t)) return def;
    lua_getfield(L, t, k);
    if (lua_isnumber(L, -1)) v = lua_tonumber(L, -1);
    lua_pop(L, 1);
    return v;
}
int gw_Kit_Available(void) { return 1; }
const char *gw_Kit_Why(void) { return ""; }
const char *gw_Kit_RoleMissingPage(int role) { (void) role; return NULL; }
int gw_Kit_Role(const char *name) { return name != NULL && name[0] == 'a' && name[1] == '_' ? 3 : -1; }
float gw_Kit_TextWidth(int role, const char *s) { (void) role; return 6.0f * (float) strlen(s) + g_track * (float) strlen(s); }
int gw_Kit_DrawText(float x, float y, const char *s, int role, uint32_t rgba, int align, float max_w, float shear, float *out_w)
{ (void) x; (void) y; (void) role; (void) rgba; (void) align; (void) max_w; (void) shear; if (out_w) *out_w = 0.0f; g_quads += (int) strlen(s); return (int) strlen(s); }
int gw_Kit_DrawPoly4(const float x[4], const float y[4], uint32_t rgba) { (void) x; (void) y; (void) rgba; g_quads++; return 1; }
static int g_images;
int gw_Kit_DrawImage(int tex, float x, float y, float w, float h, uint32_t rgba, int flip, float shear) { (void) tex; (void) x; (void) y; (void) w; (void) h; (void) rgba; (void) flip; (void) shear; g_quads++; g_images++; return 1; }
void gw_Kit_SetTracking(float px) { g_track = px; }
static int g_hsd_frame; void gw_Kit_TexHsdFrame(int frame) { g_hsd_frame = frame; }
static int g_hsd_drops; void gw_Kit_TexDropHsd(void) { g_hsd_drops++; }
int gw_Kit_TexAddHsd(const char *key, int gx_fmt, const uint8_t *img, size_t img_size, int w, int h, int tlut_fmt, const uint8_t *tlut, int tlut_n)
{ (void) gx_fmt; (void) img; (void) img_size; (void) w; (void) h; (void) tlut_fmt; (void) tlut; (void) tlut_n; g_art_calls++; snprintf(g_art_key, sizeof g_art_key, "%s", key); return g_art_ret; }
int gw_Kit_QuadCount(void) { return g_quads; }
int gw_Kit_QuadRoom(void) { return 16000; }

#include "gw_script_ui.inc"
static unsigned bswap_u(unsigned v) { return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24); }

#include "atlas_lint.h"
#include "atlas_room_fixture.h"

static void reset_ui(void)
{
    int i;
    for (i = 0; i < GS_UI_SLOTS; i++) { memset(&gs_ui_slot[i], 0, sizeof gs_ui_slot[i]); gs_ui_slot[i].dialog_fn = -1; }
    memset(&gs_ui_stack, 0, sizeof gs_ui_stack);
    gs_ui_qn = 0; g_pad = 0; gs.cur = 0; gs.scene_kind = 7;
    gs_sel.h = -1; gs_sel.qn = 0;
    gs_set.h = -1; gs_set.qn = 0;
    gs_room.slot = -1; gs_room.qn = 0; gs_room.frozen = 0; memset(&gs_room.rv, 0, sizeof gs_room.rv);
    g_mx = g_my = -1000.0f; g_mbuttons = 0; g_mwheel = 0.0f;
    memset(gs.key_now, 0, sizeof gs.key_now);
}

static GsUiSlot *room_slot(void) { return gs_room.slot >= 0 ? &gs_ui_slot[gs_room.slot] : NULL; }
static const AtRoomView *RV(void) { return &gs_room.rv; }

static void lifetime(void)
{
    char got[16];
    int arg = -1, i;
    GsUiSlot *u;
    reset_ui();
    CHECK(gw_Ui_RoomOn() == 1 && gw_Ui_RoomOpen() == 0);
    CHECK(gw_Ui_RoomSetInt("host", 1) == 0 && gw_Ui_RoomSetStr("code", "ZZZZ") == 0);   /* nothing before Begin */
    CHECK(gw_Ui_RoomBegin("lobby") == 1 && gw_Ui_RoomOpen() == 1);
    u = room_slot();
    CHECK(u != NULL && u->used && u->native_room && u->owner == GS_UI_ENGINE && at_stack_top(&gs_ui_stack) == gs_room.slot);
    CHECK(u->view.room == RV() && u->sc.primary == AT_PRIMARY_ROOM);
    CHECK(gw_Ui_RoomSetStr("code", "WXYZ") == 1 && gw_Ui_RoomSetInt("host", 1) == 1);
    CHECK(gw_Ui_RoomSetInt("no_such_key", 1) == 0 && gw_Ui_RoomSetStr("no_such_key", "x") == 0);
    CHECK(strcmp(RV()->code, "WXYZ") == 0 && RV()->host == 1);
    CHECK(gw_Ui_RoomBegin("lobby") == 1 && strcmp(RV()->code, "WXYZ") == 0);          /* the same kind again: nothing restarts */
    CHECK(gw_Ui_RoomBegin("wait") == 1);                                              /* another kind: a fresh view, the same slot */
    CHECK(room_slot() == u && RV()->code[0] == '\0' && RV()->kind == AT_ROOM_WAIT && strcmp(u->sc.id, "online.wait") == 0);
    gw_Ui_RoomSetStr("code", "ABCD");
    gw_Ui_RoomEnd();
    CHECK(gw_Ui_RoomOpen() == 0 && !gs_ui_slot[0].used && at_stack_top(&gs_ui_stack) < 0 && RV()->kind == 0 && RV()->code[0] == '\0');
    CHECK(gw_Ui_RoomSetInt("host", 1) == 0 && gw_Ui_RoomSetStr("code", "ZZZZ") == 0);   /* after End nothing is accepted */
    gw_Ui_RoomFrame();                                                                /* and a frame, a freeze, a poll with no room are harmless */
    gw_Ui_RoomFreeze(1); gw_Ui_RoomFreeze(0);
    CHECK(gw_Ui_RoomPollName(got, sizeof got, &arg) == 0);
    gw_Ui_RoomEnd();                                                                  /* End twice is harmless */
    gw_Ui_RoomBegin("lobby");
    CHECK(RV()->code[0] == '\0');                                                     /* nothing of the last room survives */
    gw_Ui_RoomEnd();
    CHECK(gw_Ui_RoomBegin("nonsense") == 0 && gw_Ui_RoomOpen() == 0 && gw_Ui_RoomBegin(NULL) == 0);
    /* a slot freed under the room (the scene ended): every call is refused and the view is stale-proof */
    gw_Ui_RoomBegin("wait"); gw_Ui_RoomSetStr("code", "QQQQ");
    gw_Ui_SceneExit(gs.scene_kind);
    CHECK(gw_Ui_RoomOpen() == 0 && gw_Ui_RoomSetStr("code", "ZZZZ") == 0 && RV()->code[0] == '\0');
    /* no free slot: refused, the game keeps its legacy drawing */
    reset_ui();
    for (i = 0; i < GS_UI_SLOTS; i++) gs_ui_slot[i].used = 1;
    CHECK(gw_Ui_RoomBegin("lobby") == 0 && gw_Ui_RoomOpen() == 0);
    reset_ui();
}

static void one_native_at_a_time(void)
{
    int h;
    reset_ui();
    CHECK(gw_Ui_RoomBegin("code") == 1);
    CHECK(gw_Ui_SelCanOpen() == 0 && gw_Ui_SetCanOpen() == 0 && gw_Ui_SetOpen(0, 0, "MAIN MENU", "", "SETTINGS") < 0);   /* not over the room */
    gw_Ui_RoomEnd();
    CHECK(gw_Ui_SetCanOpen() == 1);
    h = gw_Ui_SetOpen(0, 0, "MAIN MENU", "", "SETTINGS");
    CHECK(h >= 0);
    CHECK(gw_Ui_RoomBegin("code") == 0 && gw_Ui_RoomOpen() == 0);                      /* nor the room over the settings */
    gw_Ui_SetClose(h);
    h = gw_Ui_SetOpen(GS_SET_ONLINE, 0, "VERSUS", "", "ONLINE PLAY");                  /* ONLINE PLAY is the settings door's kind 7, in the Online chapter */
    CHECK(h >= 0 && strcmp(gs_ui_slot[h & 0xFF].sc.id, "online.play") == 0 && gs_ui_slot[h & 0xFF].sc.chapter == 3);
    gw_Ui_SetClose(h);
    CHECK(gw_Ui_RoomBegin("code") == 1);
    gw_Ui_RoomEnd();
    reset_ui();
}

static void copies(void)
{
    char buf[400], name[8];
    int i;
    reset_ui();
    gw_Ui_RoomBegin("wait");
    snprintf(name, sizeof name, "%s", "ABCD"); gw_Ui_RoomSetStr("code", name); name[0] = 'Z';
    CHECK(strcmp(RV()->code, "ABCD") == 0);                                            /* a copy, not a pointer into the game's buffer */
    for (i = 0; i < 399; i++) buf[i] = 'x';
    buf[399] = '\0';
    gw_Ui_RoomSetStr("status", buf);
    CHECK(strlen(RV()->status) == AT_ROOM_TEXT - 1);                                   /* cut to the field, still terminated */
    gw_Ui_RoomSetStr("status", NULL);
    CHECK(RV()->status[0] == '\0');
    gw_Ui_RoomSetStr("phase", "strike");
    CHECK(RV()->phase == AT_PH_STRIKE);
    gw_Ui_RoomSetStr("phase", "nonsense");
    CHECK(RV()->phase == AT_PH_OFF);
    gw_Ui_RoomPlayerName(1, "WWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWWW", "Locked in");
    CHECK(strlen(RV()->pl[1].name) == AT_ROOM_NAME - 1 && strcmp(RV()->pl[1].fighter, "Locked in") == 0);
    gw_Ui_RoomPlayerName(5, "x", "y"); gw_Ui_RoomPlayer(-1, 1, 1, 1, 1);               /* out of range: ignored */
    gw_Ui_RoomPlayer(0, 1, 1, 0, 3);
    CHECK(RV()->pl[0].present && RV()->pl[0].locked && !RV()->pl[0].ready && RV()->pl[0].score == 3);
    gw_Ui_RoomSetStr("code_chars", "AB");
    CHECK(RV()->code_in.c[0] == 'A' && RV()->code_in.c[1] == 'B' && RV()->code_in.c[2] == '\0' && RV()->code_in.c[3] == '\0');
    gw_Ui_RoomSetStr("code_chars", "WXYZQ");
    CHECK(memcmp(RV()->code_in.c, "WXYZ", 4) == 0);
    gw_Ui_RoomEnd();
}

static void ping(void)
{
    int k, changes = 0, last = -1;
    reset_ui();
    gw_Ui_RoomBegin("wait");
    gw_Ui_RoomSetInt("ping", 20);
    CHECK(RV()->link_bars == 4 && RV()->ping_ms == 20);
    for (k = 0; k < 100; k++) {
        gw_Ui_RoomSetInt("ping", 90 + (k & 1 ? 3 : -3));                              /* hovering on a line */
        if (last >= 0 && RV()->link_bars != last) changes++;
        last = RV()->link_bars;
    }
    CHECK(changes <= 1);
    gw_Ui_RoomSetInt("ping", -1);
    CHECK(RV()->link_bars == 0 && RV()->ping_ms == -1);
    gw_Ui_RoomEnd();
}

static void grid(void)
{
    int i, pages, cols, group[AT_ROOM_STAGES];
    int col = 0x11111111, row = 0x22222222, page = 0x33333333;
    unsigned char raw[12];
    AtGrid g;
    reset_ui();
    gw_Ui_RoomBegin("lobby");
    gw_Ui_RoomStageCount(40);
    CHECK(RV()->n_stages == AT_ROOM_STAGES);                                           /* cut at the list cap */
    gw_Ui_RoomStage(31, "Last", "banned", 0, 1); gw_Ui_RoomStage(32, "Past the end", "free", 0, 1);
    CHECK(strcmp(RV()->st[31].name, "Last") == 0 && RV()->st[31].state == AT_STAGE_BANNED);
    pages = gw_Ui_RoomGridFor(32, 5);
    cols = at_room_pick_cols(32, 1, 0);
    for (i = 0; i < 32; i++) group[i] = i >= 5;
    at_room_grid(group, 32, cols, 3, &g);
    CHECK(pages == g.pages);
    for (i = 0; i < 32; i++) {
        CHECK(gw_Ui_RoomGridCell(i, &col, &row, &page) == 1);
        CHECK((int) bswap_u((unsigned) col) == g.cell[i].col && (int) bswap_u((unsigned) row) == g.cell[i].row && (int) bswap_u((unsigned) page) == g.cell[i].page);
    }
    /* the raw bytes the game reads: big-endian, never a native store */
    row = 0; page = 0; col = 0;
    gw_Ui_RoomGridCell(7, &col, &row, &page);
    memcpy(raw, &col, 4); memcpy(raw + 4, &row, 4); memcpy(raw + 8, &page, 4);
    CHECK(raw[0] == 0 && raw[1] == 0 && raw[2] == 0 && raw[3] == (unsigned char) g.cell[7].col);
    CHECK(raw[4] == 0 && raw[5] == 0 && raw[6] == 0 && raw[7] == (unsigned char) g.cell[7].row);
    CHECK(raw[8] == 0 && raw[9] == 0 && raw[10] == 0 && raw[11] == (unsigned char) g.cell[7].page);
    CHECK(gw_Ui_RoomGridCell(-1, &col, &row, &page) == 0 && gw_Ui_RoomGridCell(32, &col, &row, &page) == 0);
    gw_Ui_RoomEnd();
}

static void intents(void)
{
    char name[16];
    int arg = -9, i, n;
    unsigned char raw[4];
    AtSink s;
    GsUiSlot *u;
    reset_ui();
    gw_Ui_RoomBegin("lobby");
    u = room_slot();
    gw_Ui_RoomSetInt("my_stage_turn", 1); gw_Ui_RoomSetStr("phase", "strike"); gw_Ui_RoomSetInt("me", 1);
    gw_Ui_RoomStageCount(6);
    for (i = 0; i < 6; i++) gw_Ui_RoomStage(i, "Stage", "free", 1, 1);
    gw_Ui_RoomGridFor(6, 6);
    gw_Ui_RoomSetInt("cursor", 1);
    gw_Ui_RoomFrame();
    s = rec_sink();                                                                   /* the draw pass learns the hit rectangles */
    at_render_ex(&u->sc, &u->view, 640.0f, g_now, 0, &FAKE, &s, &u->hits, NULL);
    CHECK(u->hits.n > 0);
    gs.key_now[VK_RIGHT] = 1; gs_room_tick(u); gs.key_now[VK_RIGHT] = 0;
    CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 1 && strcmp(name, "right") == 0);
    CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 0);
    /* a click on a tile: stage_at then accept, in order; the pointer arrives first (a first sample never clicks) */
    {
        const AtHit *t = NULL;
        for (i = 0; i < u->hits.n; i++) if (u->hits.h[i].kind == AT_HIT_ROOM && u->hits.h[i].a == AT_RH_STAGE && u->hits.h[i].b == 3) t = &u->hits.h[i];
        CHECK(t != NULL);
        g_mx = 400.0f; g_my = 300.0f; g_mbuttons = 0; gs_room_tick(u);                 /* the first sample only primes */
        CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 0);
        g_mx = t->r.x + 4.0f; g_my = t->r.y + 4.0f; gs_room_tick(u);
        n = gw_Ui_RoomPollName(name, sizeof name, &arg);
        CHECK(n == 1 && strcmp(name, "stage_at") == 0);                               /* hover onto an open stage on my turn: the cursor, nothing else */
        CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 0);
        g_mbuttons = 1; gs_room_tick(u); g_mbuttons = 0;
        CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 1 && strcmp(name, "stage_at") == 0);
        memcpy(raw, &arg, 4);                                                         /* the arg is big-endian in the game's local */
        CHECK(raw[0] == 0 && raw[1] == 0 && raw[2] == 0 && raw[3] == 3);
        CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 1 && strcmp(name, "accept") == 0);
        gs_room_tick(u);
    }
    /* the queue is bounded: a held-down storm of presses keeps 16 and drops the rest, never overruns */
    for (n = 0; n < 60; n++) { memset(gs.key_now, 0, sizeof gs.key_now); gs.key_now[VK_RETURN] = (n & 1) ? 0 : 1; gs_room_tick(u); }
    memset(gs.key_now, 0, sizeof gs.key_now);
    for (n = 0; gw_Ui_RoomPollName(name, sizeof name, &arg); n++) CHECK(strcmp(name, "accept") == 0);
    CHECK(n == 16);
    /* freeze: the ring is emptied, nothing arrives while it lasts, and a key down when it ends does not fire */
    gs.key_now[VK_RETURN] = 1; gs_room_tick(u);
    gw_Ui_RoomFreeze(1);
    CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 0 && gs_room.qn == 0);
    gs.key_now[VK_ESCAPE] = 1; gs_room_tick(u);
    CHECK(gs_room.qn == 0);
    gw_Ui_RoomFreeze(0);                                                              /* Enter and Escape are still down: primed, not pressed */
    gs_room_tick(u);
    CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 0);
    memset(gs.key_now, 0, sizeof gs.key_now);
    /* the code screen: the keyboard is the netplay layer's, so keys make no intents; the mouse still does */
    gw_Ui_RoomBegin("code");
    u = room_slot();
    gs.key_now[VK_LEFT] = 1; gs.key_now[VK_RETURN] = 1; gs_room_tick(u);
    CHECK(gw_Ui_RoomPollName(name, sizeof name, &arg) == 0);
    memset(gs.key_now, 0, sizeof gs.key_now);
    gw_Ui_RoomEnd();
    g_mx = g_my = -1000.0f;
    reset_ui();
}

static void draw_through_host(void)
{
    /* the host's own draw pass: one room frame through gs_ui_draw's renderer path stays under the entry cap with 32 stages at 1140 wide */
    GsUiSlot *u;
    AtSink s;
    AtRenderInfo info;
    int i;
    reset_ui();
    gw_Ui_RoomBegin("lobby");
    u = room_slot();
    gw_Ui_RoomSetStr("phase", "strike"); gw_Ui_RoomSetInt("my_stage_turn", 1); gw_Ui_RoomStageCount(32);
    for (i = 0; i < 32; i++) gw_Ui_RoomStage(i, "Princess Peach's Castle", i == 3 ? "banned" : "free", i < 5, 1);
    gw_Ui_RoomGridFor(32, 5);
    gw_Ui_RoomSetInt("fade_out", 0);
    gw_Ui_RoomFrame();
    s = rec_sink();
    memset(&info, 0, sizeof info);
    at_render_ex(&u->sc, &u->view, 1140.0f, g_now, 0, &FAKE, &s, &u->hits, &info);
    CHECK(info.entries < 1200 && !info.capped && info.hits_dropped == 0 && u->hits.n > 10);
    gw_Ui_RoomEnd();
    s = rec_sink();
    reset_ui();
}

int main(void)
{
    lua_State *L = luaL_newstate();
    gs.L = L; gs.cur = 0; gs.console = 0;
    snprintf(gs.s[0].id, sizeof gs.s[0].id, "console");
    gs.n = 8; gs.s[0].used = 1;
    luaL_openlibs(L);
    lifetime(); one_native_at_a_time(); copies(); ping(); grid(); intents(); draw_through_host();
    lua_close(L);
    ATLAS_DONE("atlas room host");
}
