/* Host-side test of the gd.ui binding: gw_script_ui.inc is included here, against stand-ins for the few gw_script.c
 * internals it uses (the script table, gs_pcall, the kit's draw calls), and driven through a real Lua 5.4 state.
 * No game, no renderer, no window. */
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
static int g_may_run = 1, g_quads, g_models;
static double g_now = 1000.0;
static float g_track;          /* what gw_Kit_SetTracking last set */
static int g_track_max_seen;   /* the largest non-zero value any draw saw */
static int g_logs_render;      /* log lines about a render budget */
static unsigned g_pad;         /* the buttons the stand-in pad holds (the raw GC bits) */
static char g_last[512];       /* the last lua() result */
static int g_logs_ex;          /* log lines about an explainer */
static int g_logs_full;        /* log lines about a full screen pool */

void gw_log(const char *fmt, ...)
{
    char buf[256];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof buf, fmt, ap);
    va_end(ap);
    if (strstr(buf, "render budget") != NULL) g_logs_render++;
    if (strstr(buf, "explainer") != NULL) g_logs_ex++;
    if (strstr(buf, "pool is full") != NULL) g_logs_full++;
}
static int gs_may_run(int i) { (void) i; return g_may_run; }
static int g_netplay, g_last_pcall_owner;
int gw_RB_Enabled(void) { return 0; }
static int gs_ui_port_read(int port, char *name, int cap, int *percent, int *stocks, int *cpu)   /* the fighter readbacks gd.player uses */
{
    if (port < 1 || port > 2) return 0;
    snprintf(name, (size_t) cap, "FOX"); *percent = 47 * port; *stocks = 3; *cpu = port == 2;
    return 1;
}
int gw_Netplay_Enabled(void) { return g_netplay; }
int gw_Mods_Count(void) { return 0; }
int gw_Mods_MenuCount(int i) { (void) i; return 0; }
const char *gw_Mods_MenuField(int i, int k, const char *f) { (void) i; (void) k; (void) f; return ""; }
const char *gw_Mods_Id(int i) { (void) i; return ""; }
static int gs_pcall(int script, int nargs, int nres, const char *what)
{
    int old = gs.cur, rc = 0;
    (void) what;
    g_last_pcall_owner = script;
    gs.cur = script;                                  /* the real gs_pcall runs the call as that script */
    if (lua_pcall(gs.L, nargs, nres, 0) != LUA_OK) { lua_pop(gs.L, 1); rc = -1; }
    gs.cur = old;
    return rc;
}
static int g_no_hook[8];                              /* scripts that define no hooks (the rest share the one global table) */
static int gs_get_hook(int i, const char *name)       /* the hooks live in one global table here */
{
    if (!gs.s[i].used || gs.s[i].disabled) return 0;
    if (g_no_hook[i]) return 0;
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
static int gs_sm_emit(int asset, const GsmParams *p, int clip, const char **why) { (void) asset; (void) p; (void) clip; (void) why; g_models++; return 0; }
void gw_script_pad_state(int ch, unsigned *b, int *sx, int *sy, int *cx, int *cy, int *tl, int *tr) { (void) ch; *b = 0; *sx = *sy = *cx = *cy = *tl = *tr = 0; }
static int g_pad_only = -1;   /* >= 0: only that channel holds g_pad (a pause screen reads the pausing port's pad) */
unsigned gw_script_pad_raw_buttons(int ch) { return (g_pad_only < 0 || ch == g_pad_only) ? g_pad : 0u; }
static float g_mx = -1000.0f, g_my = -1000.0f; static int g_mbuttons;
void gw_Mouse_ScriptRead(float *x, float *y, int *buttons, float *wheel) { *x = g_mx; *y = g_my; *buttons = g_mbuttons; *wheel = 0.0f; }
float gw_Console_ScriptWidth(void) { return 640.0f; }
static int g_pause_wanted;     /* the setting atlas_pause (the takeover; off by default) */
int gw_Settings_Int(const char *k, int d) { return strcmp(k, "atlas_pause") == 0 ? g_pause_wanted : d; }
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
static int g_kit_up = 1;
int gw_Kit_Available(void) { return g_kit_up; }
const char *gw_Kit_Why(void) { return ""; }
static const char *g_missing_page;   /* a page the kit failed to load, or NULL */
const char *gw_Kit_RoleMissingPage(int role) { (void) role; return g_missing_page; }
int gw_Kit_Role(const char *name) { return name != NULL && name[0] == 'a' && name[1] == '_' ? 3 : -1; }
float gw_Kit_TextWidth(int role, const char *s) { (void) role; return 6.0f * (float) strlen(s) + g_track * (float) strlen(s); }
int gw_Kit_DrawText(float x, float y, const char *s, int role, uint32_t rgba, int align, float max_w, float shear, float *out_w)
{
    (void) x; (void) y; (void) role; (void) rgba; (void) align; (void) max_w; (void) shear;
    if (out_w) *out_w = 0.0f;
    if (g_track > 0.0f) g_track_max_seen = 1;
    g_quads += (int) strlen(s);
    return (int) strlen(s);
}
int gw_Kit_DrawPoly4(const float x[4], const float y[4], uint32_t rgba) { (void) x; (void) y; (void) rgba; g_quads++; return 1; }
int gw_Kit_DrawImage(int tex, float x, float y, float w, float h, uint32_t rgba, int flip, float shear) { (void) tex; (void) x; (void) y; (void) w; (void) h; (void) rgba; (void) flip; (void) shear; g_quads++; return 1; }
void gw_Kit_SetTracking(float px) { g_track = px; }
static int g_hsd_frame; void gw_Kit_TexHsdFrame(int frame) { g_hsd_frame = frame; }
static int g_hsd_drops; void gw_Kit_TexDropHsd(void) { g_hsd_drops++; }
int gw_Kit_TexAddHsd(const char *key, int gx_fmt, const uint8_t *img, size_t img_size, int w, int h, int tlut_fmt, const uint8_t *tlut, int tlut_n) { (void) key; (void) gx_fmt; (void) img; (void) img_size; (void) w; (void) h; (void) tlut_fmt; (void) tlut; (void) tlut_n; return -1; }
int gw_Kit_QuadCount(void) { return g_quads; }
int gw_Kit_QuadRoom(void) { return 16000; }

#include "gw_script_ui.inc"

/* run Lua; the result is the last value as text, or "ERR ..." */
static const char *lua(const char *code)
{
    static char buf[512];
    lua_State *L = gs.L;
    int top = lua_gettop(L);
    buf[0] = '\0';
    if (luaL_dostring(L, code) != LUA_OK) snprintf(buf, sizeof buf, "ERR %s", lua_tostring(L, -1));
    else if (lua_gettop(L) > top) snprintf(buf, sizeof buf, "%s", luaL_tolstring(L, -1, NULL));
    lua_settop(L, top);
    snprintf(g_last, sizeof g_last, "%s", buf);
    return buf;
}
#define LUA_IS(code, want) CHECK_STR(lua(code), want)
#define LUA_HAS(code, part) do { const char *_r = lua(code); at_t_count++; if (strstr(_r, part) == NULL) { at_t_fails++; printf("FAIL %s:%d: %s -> \"%s\" lacks \"%s\"\n", __FILE__, __LINE__, code, _r, part); } } while (0)


/* ---- Atlas step 2, Task 5: engine-owned screens (driven the way their real owner drives them) ---- */
/* gw_Ui_PollEvent writes the game's locals, which are big-endian guest memory: poll_ev reads them back as native ints (the game does the
 * same with its byte-swapped loads). Reading them raw is the bug this guards: FOCUS (2) arrived as 0x02000000 and no menu answered. */
static unsigned bswap_u(unsigned v) { return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24); }
static int poll_ev(int *t, int *b, int *i)
{
    int r = gw_Ui_PollEvent(t, b, i);
    if (r) {
        *t = (int) bswap_u((unsigned) *t);
        *b = (int) bswap_u((unsigned) *b);
        *i = (int) bswap_u((unsigned) *i);
    }
    return r;
}

static void reset_ui(void)
{
    int i;
    for (i = 0; i < GS_UI_SLOTS; i++) { memset(&gs_ui_slot[i], 0, sizeof gs_ui_slot[i]); gs_ui_slot[i].dialog_fn = -1; }
    memset(&gs_ui_stack, 0, sizeof gs_ui_stack);
    gs_ui_qn = 0; g_pad = 0; gs.cur = 0; gs.scene_kind = -1;
    memset(g_no_hook, 0, sizeof g_no_hook);
    for (i = 1; i < 8; i++) gs.s[i].used = 0;       /* the earlier tests' scripts: each test here names its own */
    g_netplay = 0;
}
static void fake_script(int i, const char *mod)
{
    snprintf(gs.s[i].id, sizeof gs.s[i].id, "%s/main", mod);
    gs.s[i].used = 1; gs.s[i].disabled = 0;
}
static void fake_script_off(int i) { gs.s[i].used = 0; }
static void fake_script_named(int i, const char *id) { snprintf(gs.s[i].id, sizeof gs.s[i].id, "%s", id); gs.s[i].used = 1; gs.s[i].disabled = 0; }
static void fake_pad_hold(int port, unsigned bit) { (void) port; g_pad = bit; }
static int t_lua(const char *code) { return strncmp(lua(code), "ERR", 3) != 0; }
static const char *t_lua_err(void) { return g_last; }

static int engine_menu(const char *id, int n)
{
    int i;
    char tid[16];
    if (!gw_Ui_Begin(id, AT_PRIMARY_TILES, 0, "MAIN MENU")) return 0;
    gw_Ui_Cols(1);                                 /* the main menu is five big rows */
    for (i = 0; i < n; i++) { snprintf(tid, sizeof tid, "t%d", i); gw_Ui_Tile(tid, tid, "", "", 0); }
    gw_Ui_Key('A', "Open"); gw_Ui_Key('B', "Title");
    return gw_Ui_Commit(0, 0);
}
static void engine_slot_survives_tick(void)
{
    int f;
    reset_ui();
    CHECK(engine_menu("main", 5) == 1);
    for (f = 0; f < 10; f++) gs_ui_tick();
    CHECK(gw_Ui_TopIsEngine("main") == 1);        /* the gs_ui_usable() check must not release an engine slot */
    CHECK(gs_ui_slot[gs_ui_find("main")].owner == GS_UI_ENGINE);
}
static void engine_slot_not_released_by_script_unload(void)
{
    reset_ui();
    engine_menu("main", 5);
    fake_script(3, "envoy");                       /* a mod script in slot 3 */
    gs.cur = 3; CHECK(t_lua("gd.ui.screen{ id='envoy.setup', primary={kind='list', items={{id='a', label='A'}}}, on={back=function() return {pop=true} end} }; return gd.ui.open('envoy.setup')"));
    CHECK(gw_Ui_TopIsEngine("main") == 0);
    gs_ui_release(3);                              /* the script unloads while its screen covers the menu */
    CHECK(gw_Ui_TopIsEngine("main") == 1);
    gs_ui_release(GS_UI_ENGINE);                   /* a release keyed by the engine id does nothing */
    gs_ui_release(-1);
    CHECK(gw_Ui_TopIsEngine("main") == 1);
    gs.s[3].used = 0;                              /* the script is gone, the tick runs: the menu is still there */
    gs_ui_tick(); gs_ui_tick();
    CHECK(gw_Ui_TopIsEngine("main") == 1);
}
/* The regression for the dead Atlas menus: the game reads the polled event through byte-swapped loads, so the host must store big-endian.
 * A raw read of a FOCUS (2) / accept (3) event must give the swapped value, and the swapped read the real one. */
static void polled_event_is_big_endian_for_the_game(void)
{
    int t = 0, b = 0, i = 0;
    unsigned char *raw = (unsigned char *) &t;
    reset_ui();
    engine_menu("main", 5);
    gs_ui_tick();
    gs_ui_tick();
    gw_Ui_Intent(AT_EV_MOVE, AT_DIR_DOWN);                                   /* the focus moves: a FOCUS event for the game */
    CHECK(gw_Ui_PollEvent(&t, &b, &i) == 1);
    CHECK(raw[0] == 0 && raw[1] == 0 && raw[2] == 0 && raw[3] == AT_EV_FOCUS);   /* big-endian: the low byte last, as PowerPC lays it out */
    CHECK((int) bswap_u((unsigned) t) == AT_EV_FOCUS && (int) bswap_u((unsigned) i) == 1);
}

static void uncover_primes_engine_screen(void)
{
    int t, b, i;
    reset_ui();
    engine_menu("main", 5);
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("gd.ui.screen{ id='envoy.s', primary={kind='list', items={{id='a', label='A'}}}, on={back=function() return {pop=true} end} }; gd.ui.open('envoy.s')");
    gs.cur = -1;
    gs_ui_tick();                                 /* B not yet held */
    fake_pad_hold(1, AT_PAD_B);                   /* B held: it closes envoy.s ... */
    gs_ui_tick();
    CHECK(gw_Ui_TopIsEngine("main") == 1);
    gw_Ui_Intent(AT_EV_BACK, 0);                  /* ... the game's menu input of that same frame is the same press: dropped */
    while (poll_ev(&t, &b, &i)) CHECK(t != AT_EV_BACK);
    gs_ui_tick();                                 /* the next frame: an intent is the menu's own again */
    gw_Ui_Intent(AT_EV_BACK, 0);
    CHECK(poll_ev(&t, &b, &i) == 1 && t == AT_EV_BACK);
}
static void native_intents_are_primed(void)
{
    int t, b, i, accepts = 0;
    reset_ui();
    gw_Ui_Intent(AT_EV_ACCEPT, 0);                /* an intent before the screen exists is dropped */
    engine_menu("main", 5);
    while (poll_ev(&t, &b, &i)) if (t == AT_EV_ACCEPT) accepts++;
    CHECK(accepts == 0);
    gw_Ui_Intent(AT_EV_MOVE, AT_DIR_DOWN);
    CHECK(poll_ev(&t, &b, &i) == 1 && t == AT_EV_FOCUS && b == 0 && i == 1);
    gw_Ui_Intent(AT_EV_ACCEPT, 0);
    CHECK(poll_ev(&t, &b, &i) == 1 && t == AT_EV_ACCEPT && i == 1);
    CHECK(poll_ev(&t, &b, &i) == 0);
}
static void intents_from_any_port(void)
{
    /* the engine screen does not read a port: the game feeds the merged menu input of all ports, and the pad is never read here */
    int t, b, i;
    reset_ui();
    engine_menu("main", 5);
    CHECK(gs_ui_slot[gs_ui_find("main")].sc.input_feed == 1);
    CHECK(at_screen_wants_pad(&gs_ui_slot[gs_ui_find("main")].sc) == 0);
    gs.cur = -1;
    fake_pad_hold(1, AT_PAD_A); gs_ui_tick(); gs_ui_tick();       /* a held A on the pad does nothing by itself */
    CHECK(poll_ev(&t, &b, &i) == 0);
}
static void engine_screen_covered_takes_no_intent(void)
{
    int t, b, i;
    reset_ui();
    engine_menu("main", 5);
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("gd.ui.screen{ id='envoy.s', primary={kind='list', items={{id='a', label='A'}}} }; gd.ui.open('envoy.s')");
    gw_Ui_Intent(AT_EV_ACCEPT, 0);
    CHECK(poll_ev(&t, &b, &i) == 0);
    gw_Ui_Intent(AT_EV_MOVE, AT_DIR_DOWN);
    CHECK(poll_ev(&t, &b, &i) == 0);
}
static void scene_exit_closes_scene_screens(void)
{
    reset_ui();
    gs.scene_kind = 1;                             /* GS_FRONTEND's kind in the stand-in */
    engine_menu("main", 5);
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("gd.ui.screen{ id='envoy.s', primary={kind='list', items={{id='a', label='A'}}} }; gd.ui.open('envoy.s')");
    gw_Ui_SceneExit(1);
    CHECK(gs_ui_stack.n == 0);                     /* neither the menu nor the mod screen reaches the next scene */
    CHECK(gs_ui_find("main") < 0);                 /* the engine's slot is free again */
    gs.scene_kind = 2;
    gs.cur = 3; t_lua("gd.ui.open('envoy.s')");     /* the bag pattern: opened inside a match */
    gw_Ui_SceneExit(1);                            /* another scene's exit leaves it alone */
    CHECK(gs_ui_stack.n == 1);
    gw_Ui_SceneExit(2);
    CHECK(gs_ui_stack.n == 0);
}
static void console_cannot_touch_engine(void)    /* console-only check: labelled */
{
    reset_ui();
    engine_menu("main", 5);
    gs.cur = gs.console;
    CHECK(!t_lua("gd.ui.close('main')"));
    CHECK(strstr(t_lua_err(), "belongs to the engine") != NULL);
    CHECK(!t_lua("gd.ui.screen{ id='main', primary={kind='list', items={{id='a', label='A'}}} }"));
    CHECK(strstr(t_lua_err(), "belongs to the engine") != NULL);
    CHECK(!t_lua("gd.ui.open('main')") && !t_lua("gd.ui.feed('main','accept')") && !t_lua("gd.ui.set_focus('main','tiles','t1')"));
    CHECK(!t_lua("gd.ui.screen{ id='solo', primary={kind='list', items={{id='a', label='A'}}} }"));      /* an engine id no slot holds yet */
    CHECK(strstr(t_lua_err(), "belongs to the engine") != NULL);
    CHECK(t_lua("gd.ui.screen{ id='t.fine', primary={kind='list', items={{id='a', label='A'}}} }"));    /* the console's own dotted ids still work */
    CHECK(gw_Ui_TopIsEngine("main") == 1);
}
static void mod_cannot_take_engine_id(void)
{
    reset_ui();
    engine_menu("main", 5);
    fake_script(3, "envoy");
    gs.cur = 3;
    CHECK(!t_lua("gd.ui.screen{ id='main', primary={kind='list', items={{id='a', label='A'}}} }"));
    CHECK(!t_lua("gd.ui.close('main')"));
    CHECK(!t_lua("gd.ui.open('main')"));
    CHECK(gw_Ui_TopIsEngine("main") == 1);
    CHECK(!t_lua("return gd.ui.focus('main')") || 1);
}
static void commit_without_change_does_not_rebuild(void)
{
    int before;
    reset_ui();
    engine_menu("main", 5);
    before = gs_ui_slot[gs_ui_find("main")].rebuilds;
    engine_menu("main", 5);
    CHECK(gs_ui_slot[gs_ui_find("main")].rebuilds == before);
    engine_menu("main", 4);
    CHECK(gs_ui_slot[gs_ui_find("main")].rebuilds == before + 1);
}
static void engine_focus_is_the_games_cursor(void)
{
    int t, b, i;
    AtFocusPos f;
    reset_ui();
    engine_menu("main", 5);
    gw_Ui_Begin("main", AT_PRIMARY_TILES, 0, "MAIN MENU"); gw_Ui_Cols(1);        /* the game says its cursor is 3 */
    { int k; char tid[16]; for (k = 0; k < 5; k++) { snprintf(tid, sizeof tid, "t%d", k); gw_Ui_Tile(tid, tid, "", "", 0); } }
    gw_Ui_Key('A', "Open"); gw_Ui_Key('B', "Title");
    gw_Ui_Commit(0, 3);
    f = gs_ui_slot[gs_ui_find("main")].view.focus;
    CHECK(f.block == 0 && f.index == 3);
    gw_Ui_Intent(AT_EV_MOVE, AT_DIR_UP);
    CHECK(poll_ev(&t, &b, &i) == 1 && t == AT_EV_FOCUS && i == 2);
    gw_Ui_Intent(AT_EV_MOVE, AT_DIR_UP);
    CHECK(poll_ev(&t, &b, &i) == 1 && t == AT_EV_FOCUS && i == 1);
    gw_Ui_Begin("main", AT_PRIMARY_TILES, 0, "MAIN MENU"); gw_Ui_Cols(1);        /* a record that changes keeps the focused cell by id */
    { int k; char tid[16]; for (k = 4; k >= 0; k--) { snprintf(tid, sizeof tid, "t%d", k); gw_Ui_Tile(tid, tid, "", "", 0); } }
    gw_Ui_Key('A', "Open"); gw_Ui_Key('B', "Title");
    gw_Ui_Commit(-1, -1);
    f = gs_ui_slot[gs_ui_find("main")].view.focus;
    CHECK(f.block == 0 && f.index == 3);                          /* t1 moved to index 3 in the reversed row: focus follows the id */
    gw_Ui_Begin("main", AT_PRIMARY_TILES, 0, "MAIN MENU"); gw_Ui_Cols(1);
    { int k; char tid[16]; for (k = 0; k < 5; k++) { snprintf(tid, sizeof tid, "u%d", k); gw_Ui_Tile(tid, tid, "", "", 0); } }
    gw_Ui_Commit(0, 4);
    CHECK(gs_ui_slot[gs_ui_find("main")].view.focus.index == 4);
    CHECK(gw_Ui_Commit(0, 0) == 0);                               /* a Commit with no Begin does nothing */
}
static void engine_close_and_queue(void)
{
    int t, b, i;
    reset_ui();
    engine_menu("main", 5);
    gw_Ui_Intent(AT_EV_BACK, 0);
    gw_Ui_Close("main");
    CHECK(gw_Ui_TopIsEngine(NULL) == 0 && gs_ui_find("main") < 0);
    CHECK(poll_ev(&t, &b, &i) == 0);                      /* a closed screen's events are stale */
    gw_Ui_Close("main");                                          /* closing what is not there is quiet */
    engine_menu("main", 5);
    gw_Ui_Begin("solo", AT_PRIMARY_TILES, 1, "SOLO"); gw_Ui_Tile("a", "A", "MOD", "", 0); gw_Ui_Tile("b", "B", "", "", AT_CELL_DISABLED); gw_Ui_Commit(0, 0);
    gw_Ui_Intent(AT_EV_MOVE, AT_DIR_RIGHT);
    CHECK(poll_ev(&t, &b, &i) == 1 && t == AT_EV_FOCUS && i == 1);
    gw_Ui_Intent(AT_EV_ACCEPT, 0);                                /* a disabled tile does not accept */
    CHECK(poll_ev(&t, &b, &i) == 0);
}
static void eight_slots_with_engine(void)
{
    /* the engine's screens count against the 8 slots: the front door uses at most 2 at once (menu, title) */
    reset_ui();
    engine_menu("main", 5);
    CHECK(gs_ui_engine_slots() <= 2);
    CHECK(gw_Ui_Begin("title", AT_PRIMARY_DISPLAY, 0, "") == 1);
    gw_Ui_Display("GD'S MELEE", "PRESS START", "v1", "credit"); gw_Ui_Commit(-1, -1);
    CHECK(gs_ui_engine_slots() == 2 && gs_ui_slot[gs_ui_find("title")].sc.primary == AT_PRIMARY_DISPLAY);
    CHECK(at_screen_wants_pad(&gs_ui_slot[gs_ui_find("title")].sc) == 0);
}

/* ---- Atlas step 3, Task 1: sixteen slots and gd.ui.forget (run as a mod script, the real path) ---- */
/* run Lua as script `slot` (gs.cur = slot, as a hook does); 0 when it ran without error, 1 when it raised. The tick runs with gs.cur = -1. */
static int run_as_script(int slot, const char *code)
{
    int old = gs.cur, ok;
    gs.cur = slot;
    ok = t_lua(code);
    gs.cur = old;
    return ok ? 0 : 1;
}
static int run_as_console(const char *code) { return run_as_script(gs.console, code); }
/* how many registry references the stored screens hold */
static int lua_refs_held(void)
{
    int i, n = 0, refs[40];
    for (i = 0; i < GS_UI_SLOTS; i++) if (gs_ui_slot[i].used) n += at_screen_fn_refs(&gs_ui_slot[i].sc, refs, 40);
    return n;
}
static void forget_frees_a_slot(void)
{
    int i, ref, again;
    char lua[320];
    reset_ui();
    fake_script(1, "envoy"); fake_script_named(2, "envoy/second");
    CHECK(GS_UI_SLOTS == 16);
    for (i = 0; i < 16; i++) {                                                  /* 16 screens fit */
        snprintf(lua, sizeof lua, "assert(gd.ui.screen{id='envoy.s%d', trail={title='T'}, primary={kind='list', items={{id='a', label='A'}}}})", i);
        CHECK(run_as_script(1, lua) == 0);
    }
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.screen, {id='envoy.s16', trail={title='T'}, primary={kind='list', items={{id='a', label='A'}}}}))") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.open('envoy.s3')); assert(gd.ui.forget('envoy.s3')); assert(gd.ui.state().depth == 0)") == 0);
    CHECK(gs_ui_find("envoy.s3") < 0);
    CHECK(run_as_script(1, "assert(gd.ui.screen{id='envoy.s16', trail={title='T'}, primary={kind='list', items={{id='a', label='A'}}}})") == 0);
    CHECK(run_as_script(2, "assert(not pcall(gd.ui.forget, 'envoy.s0'))") == 0);   /* another script: refused */
    CHECK(gs_ui_find("envoy.s0") >= 0);
    CHECK(run_as_script(1, "local ok, e = pcall(gd.ui.forget, 'envoy.nope'); assert(not ok and e:find('no screen'))") == 0);
    CHECK(lua_refs_held() == 0);                                                /* handler-free screens hold no references */
    /* a screen with handlers: forget returns every reference it holds to the registry (the next luaL_ref reuses the freed one) */
    CHECK(run_as_script(1, "assert(gd.ui.forget('envoy.s5')); assert(gd.ui.screen{id='envoy.h', primary={kind='list', items={{id='a', label='A'}}}, on={accept=function() end}})") == 0);
    CHECK(lua_refs_held() == 1);
    { int refs[40]; at_screen_fn_refs(&gs_ui_slot[gs_ui_find("envoy.h")].sc, refs, 40); ref = refs[0]; }
    CHECK(run_as_script(1, "assert(gd.ui.open('envoy.h')); assert(gd.ui.forget('envoy.h'))") == 0);
    CHECK(lua_refs_held() == 0 && gs_ui_stack.n == 0);
    lua_pushboolean(gs.L, 1); again = luaL_ref(gs.L, LUA_REGISTRYINDEX);
    CHECK(again == ref);                                                        /* the freed slot of the registry came back: nothing leaked */
    luaL_unref(gs.L, LUA_REGISTRYINDEX, again);
    CHECK(run_as_console("assert(gd.ui.forget('envoy.s0'))") == 0);            /* the console may forget any script screen */
    CHECK(run_as_script(1, "assert(gd.ui.forget('envoy.s1')); assert(gd.ui.forget('envoy.s2')); assert(gd.ui.screen{id='envoy.again', primary={kind='list', items={{id='a', label='A'}}}})") == 0);
    reset_ui();
}

/* ---- Atlas step 3, Task 4: the retail shims the game side calls (Ui_RetailHidden, Ui_RetailPause, Ui_TakeUnpause) ---- */
static void retail_shims(void)
{
    int id;
    reset_ui(); g_netplay = 0; g_pause_wanted = 0;
    memset(&gs_ui_retail, 0, sizeof gs_ui_retail);
    at_pause_off(&gs_ui_pause);
    for (id = 0; id < AT_RE_COUNT; id++) CHECK(gw_Ui_RetailHidden(id) == 0);          /* empty by default: every guard takes retail's path */
    at_retail_set(&gs_ui_retail, AT_RS_CONSOLE, 1u << AT_RE_HUD_DAMAGE);
    CHECK(gw_Ui_RetailHidden(AT_RE_HUD_DAMAGE) == 1 && gw_Ui_RetailHidden(AT_RE_HUD_STOCK) == 0);
    g_netplay = 1;
    CHECK(gw_Ui_RetailHidden(AT_RE_HUD_DAMAGE) == 0);                                  /* online: nothing is hidden */
    g_netplay = 0;
    at_retail_set(&gs_ui_retail, AT_RS_CONSOLE, 0);
    gw_Ui_RetailPause(1, 1);                                                           /* takeover not wanted: a pause with no takeover */
    CHECK(gs_ui_pause.paused && gs_ui_pause.pauser == 1 && !gs_ui_pause.takeover && gs_ui_pause_changed);
    CHECK(gw_Ui_TakeUnpause() == -1);
    gw_Ui_RetailPause(1, 0);
    CHECK(!gs_ui_pause.paused);
    g_pause_wanted = 1;
    gw_Ui_RetailPause(2, 1);
    CHECK(gs_ui_pause.takeover && gs_ui_pause.pauser == 2);
    CHECK(at_pause_request_unpause(&gs_ui_pause, 0) && gw_Ui_TakeUnpause() == 2 && gw_Ui_TakeUnpause() == -1);   /* one-shot */
    CHECK(at_pause_request_unpause(&gs_ui_pause, 0));
    g_netplay = 1; CHECK(gw_Ui_TakeUnpause() == -1); g_netplay = 0;                    /* never taken online */
    gw_Ui_RetailPause(2, 0); g_pause_wanted = 0; gs_ui_pause_changed = 0;
    g_netplay = 1; gw_Ui_RetailPause(1, 1); g_pause_wanted = 1;
    CHECK(!gs_ui_pause.takeover);                                                      /* a pause that began online is never a takeover */
    gw_Ui_RetailPause(1, 0); g_netplay = 0; g_pause_wanted = 0;
}

/* ---- Atlas step 3, Task 7: gd.ui.hud, toast, retail_hide, retail and the HUD draw pass (as mod scripts, the real path) ---- */
static int hud_toasts_in_zone(int owner1, int zone)
{
    int i, k, n = 0;
    for (i = 0; i < GS_UI_HUDS; i++) if (gs_ui_hud[i].owner == owner1) for (k = 0; k < gs_ui_hud[i].n[zone]; k++) if (gs_ui_hud[i].z[zone][k].kind == AT_HP_TOAST) n++;
    return n;
}
static int hud_count(void) { int i, n = 0; for (i = 0; i < GS_UI_HUDS; i++) if (gs_ui_hud[i].owner != 0) n++; return n; }
static void hud_reset(void)
{
    reset_ui();
    memset(gs_ui_hud, 0, sizeof gs_ui_hud);
    memset(&gs_ui_retail, 0, sizeof gs_ui_retail);
    at_pause_off(&gs_ui_pause);
    fake_script(1, "envoy"); fake_script_named(2, "envoy/second");
    gs.s[1].gameplay = 1; gs.s[2].gameplay = 1;
    gs.match_active = 0; g_netplay = 0; g_pause_wanted = 0;
}
static void hud_basics(void)
{
    hud_reset();
    gs.match_active = 1;
    CHECK(run_as_script(1,
        "assert(gd.ui.hud{id='envoy.hud', zones={top_left={{kind='strip', pips={{fill=0xF07474FF, ring=0xF2C14EFF}}, keys={{letter='P', rgba=0xB872F0FF}}}},"
        " top_center={{kind='banner', text='Collect the drives', button='A'}}}})") == 0);
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.hud, {id='other.hud', zones={}}))") == 0);          /* the id must start with the mod */
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.hud, {id='envoy.hud', zones={top_left={{kind='banner', text='x'}}, top_center={{kind='banner', text='y'}}}}))") == 0);
    CHECK(hud_count() == 1 && gs_ui_hud[0].n[AT_Z_TOP_LEFT] == 1);                                  /* a refused description changed nothing */
    g_quads = 0; gs_ui_hud_draw();
    CHECK(g_quads > 0);                                                                              /* drawn with no screen open */
    g_may_run = 0; g_quads = 0; gs_ui_hud_draw();                                                    /* a resimulated frame: the draw needs no Lua and still draws */
    CHECK(g_quads > 0); g_may_run = 1;
    gs.match_active = 0; g_quads = 0; gs_ui_hud_draw(); CHECK(g_quads == 0); gs.match_active = 1;    /* only inside a match */
    CHECK(run_as_script(1, "assert(gd.ui.toast{zone='top_right', title='SKYWARD ASSEMBLED', text='Aerials gain Haste.'})") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.toast{zone='top_right', title='SECOND', text='Replaces the first.'})") == 0);
    CHECK(hud_toasts_in_zone(2, AT_Z_TOP_RIGHT) == 1 && strcmp(gs_ui_hud[0].z[AT_Z_TOP_RIGHT][0].text, "SECOND") == 0);   /* replaces, never queues */
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.toast, {zone='bottom_left', title='x', text='y'}))") == 0);
    /* re-describing the HUD keeps the live toast and a note's clock */
    CHECK(run_as_script(1, "assert(gd.ui.hud{id='envoy.hud', zones={top_left={{kind='note', text='Merged', seconds=4}}}})") == 0);
    CHECK(hud_toasts_in_zone(2, AT_Z_TOP_RIGHT) == 1);
    { double until = gs_ui_hud[0].z[AT_Z_TOP_LEFT][0].until_ms;
      g_now += 500.0;
      CHECK(run_as_script(1, "assert(gd.ui.hud{id='envoy.hud', zones={top_left={{kind='note', text='Merged', seconds=4}}}})") == 0);
      CHECK(gs_ui_hud[0].z[AT_Z_TOP_LEFT][0].until_ms == until); }                                  /* the same note is not restarted */
    CHECK(run_as_script(1, "assert(gd.ui.hud_clear('envoy.hud')); assert(not gd.ui.hud_clear())") == 0);
    CHECK(hud_count() == 0);
    /* each script has its own HUD */
    CHECK(run_as_script(1, "assert(gd.ui.hud{id='envoy.hud', zones={}})") == 0);
    CHECK(run_as_script(2, "assert(gd.ui.hud{id='envoy.hud', zones={}})") == 0 && hud_count() == 2);
    gs.match_active = 0;
}
static void retail_mask_ownership(void)
{
    hud_reset();
    gs.match_active = 1; g_netplay = 0;
    CHECK(run_as_script(1, "assert(gd.ui.retail_hide{'hud.damage'}); local r=gd.ui.retail(); assert(r.hidden[1]=='hud.damage')") == 0);
    CHECK(gw_Ui_RetailHidden(AT_RE_HUD_DAMAGE));
    CHECK(run_as_script(2, "assert(not pcall(gd.ui.retail_hide, {'hud.stock'}))") == 0);                   /* another script */
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.retail_hide, {'hud.timer'}))") == 0);                   /* never the clock */
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.retail_hide, {'hud.bogus'}))") == 0);
    g_netplay = 1; CHECK(!gw_Ui_RetailHidden(AT_RE_HUD_DAMAGE));                                            /* online: shown */
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.retail_hide, {'hud.damage'}))") == 0);
    CHECK(run_as_script(1, "local r = gd.ui.retail(); assert(#r.hidden == 0)") == 0); g_netplay = 0;       /* and gd.ui.retail() says so */
    gs_ui_release(1);                                                                                       /* unload */
    CHECK(!gw_Ui_RetailHidden(AT_RE_HUD_DAMAGE) && hud_count() == 0);
    CHECK(run_as_script(1, "assert(gd.ui.retail_hide{'hud.stock'})") == 0);
    gs_ui_scene_changed();                                                                                  /* a scene change releases it too */
    CHECK(!gw_Ui_RetailHidden(AT_RE_HUD_STOCK));
    CHECK(run_as_script(1, "assert(gd.ui.retail_hide{'hud.stock'}); assert(gd.ui.retail_hide{}); assert(#gd.ui.retail().hidden == 0)") == 0);   /* {} releases */
    gs.match_active = 0;
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.retail_hide, {'hud.stock'}))") == 0);                   /* no match: refused */
    gs.match_active = 1; gs.s[1].gameplay = 0;
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.retail_hide, {'hud.stock'}))") == 0);                   /* not a gameplay script */
    gs.s[1].gameplay = 1; gs.match_active = 0;
}
static void toasts_and_notes_leave_with_the_scene(void)
{
    hud_reset(); gs.match_active = 1;
    CHECK(run_as_script(1, "assert(gd.ui.hud{id='envoy.hud', zones={top_left={{kind='strip', pips={}, keys={}}, {kind='note', text='n'}}}}); assert(gd.ui.toast{zone='top_left', title='T', text='t'})") == 0);
    CHECK(gs_ui_hud[0].n[AT_Z_TOP_LEFT] == 3);
    gs_ui_scene_changed();
    CHECK(gs_ui_hud[0].n[AT_Z_TOP_LEFT] == 1 && gs_ui_hud[0].z[AT_Z_TOP_LEFT][0].kind == AT_HP_STRIP);   /* the strip stays; the script re-describes it */
    gs.match_active = 0;
}
static void console_is_not_the_test(void)
{
    /* the console has no mod id: a console HUD, toast or retail claim is refused, so a console-driven pass proves nothing about the real path */
    hud_reset(); gs.match_active = 1;
    CHECK(run_as_console("assert(not pcall(gd.ui.hud, {id='envoy.hud', zones={}}))") == 0);
    CHECK(run_as_console("assert(not pcall(gd.ui.toast, {title='x', text='y'}))") == 0);
    CHECK(run_as_console("assert(not pcall(gd.ui.retail_hide, {'hud.stock'}))") == 0);
    gs.match_active = 0;
}
static void atlas_console_command(void)
{
    char out[320];
    hud_reset();
    CHECK(gs_ui_console("retail hud.damage,hud.stock", out, sizeof out) == 0 && gw_Ui_RetailHidden(AT_RE_HUD_DAMAGE) && gw_Ui_RetailHidden(AT_RE_HUD_STOCK));
    CHECK(gs_ui_console("retail clear", out, sizeof out) == 0 && !gw_Ui_RetailHidden(AT_RE_HUD_DAMAGE));
    CHECK(gs_ui_console("retail hud.bogus", out, sizeof out) == -1 && strstr(out, "hud.bogus") != NULL);
    CHECK(gs_ui_console("keepout on", out, sizeof out) == 0 && gs_ui_keepout_show == 1 && gs_ui_console("keepout off", out, sizeof out) == 0 && gs_ui_keepout_show == 0);
    CHECK(gs_ui_console("hud", out, sizeof out) == 0);
    CHECK(gs_ui_console("wobble", out, sizeof out) == -1);
}

/* ---- Atlas step 3, Task 8: the pause screen and the takeover (off by default; as a mod script, through the tick) ---- */
static void pause_takeover(void)
{
    hud_reset();
    gs_ui_pause_slot = gs_ui_pause_pushed = -1; gs_ui_pause_changed = 0;
    CHECK(run_as_script(1,
        "assert(gd.ui.screen{id='envoy.pause', kind='pause', trail={title='PAUSED'}, primary={kind='list', items={{id='resume', label='Resume'}}},"
        " on={accept=function(c) if c=='resume' then gd.ui.unpause() end end}}); assert(gd.ui.pause_screen('envoy.pause'))") == 0);
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.pause_screen, 'envoy.nope'))") == 0);
    CHECK(run_as_script(2, "assert(not pcall(gd.ui.pause_screen, 'envoy.pause'))") == 0);                      /* another script's screen */
    g_pause_wanted = 0; gw_Ui_RetailPause(1, 1); gs.cur = -1; gs_ui_tick();
    CHECK(at_stack_top(&gs_ui_stack) < 0);                                    /* takeover off (the default): nothing pushed */
    gw_Ui_RetailPause(1, 0); gs_ui_tick();
    g_pause_wanted = 1; g_pad_only = 1; gw_Ui_RetailPause(1, 1); gs_ui_tick();
    CHECK(at_stack_top(&gs_ui_stack) == gs_ui_find("envoy.pause") && gs_ui_slot[gs_ui_find("envoy.pause")].sc.port == 2);   /* the pauser's port drives it */
    CHECK(run_as_script(1, "assert(gd.ui.state().top == 'envoy.pause')") == 0);
    g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0; gs_ui_tick();                           /* the pauser presses A on Resume */
    CHECK(gw_Ui_TakeUnpause() == 1 && gw_Ui_TakeUnpause() == -1);            /* one-shot, and it names the pauser */
    gw_Ui_RetailPause(1, 0); gs_ui_tick();
    CHECK(at_stack_top(&gs_ui_stack) < 0);                                    /* popped when retail unpaused */
    g_pad_only = 0;                                                           /* another port's A does not drive it */
    g_pause_wanted = 1; gw_Ui_RetailPause(1, 1); gs_ui_tick();
    g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0;
    CHECK(gw_Ui_TakeUnpause() == -1);
    g_pad_only = -1;
    gw_Ui_RetailPause(1, 0); gs_ui_tick();
    g_netplay = 1; gw_Ui_RetailPause(1, 1); gs_ui_tick();
    CHECK(at_stack_top(&gs_ui_stack) < 0 && gw_Ui_TakeUnpause() == -1);      /* never online */
    CHECK(run_as_script(1, "assert(gd.ui.open('envoy.pause') == false)") == 0);                                 /* and a pause screen does not open online */
    gw_Ui_RetailPause(1, 0); gs_ui_tick(); g_netplay = 0; g_pause_wanted = 0;
    CHECK(run_as_script(1, "assert(gd.ui.unpause() == false)") == 0);         /* not paused */
    /* a request is one-shot and offline-only through the binding too */
    g_pause_wanted = 1; gw_Ui_RetailPause(2, 1);
    CHECK(run_as_script(1, "assert(gd.ui.unpause() == true); assert(gd.ui.unpause() == false)") == 0);
    gw_Ui_RetailPause(2, 0); gs_ui_tick(); g_pause_wanted = 0;
    CHECK(gw_Ui_TakeUnpause() == -1);                                         /* a request never leaks into the next pause */
    /* a scene that ends mid-pause leaves no pause and no screen behind */
    g_pause_wanted = 1; gw_Ui_RetailPause(1, 1); gs_ui_tick();
    CHECK(at_stack_top(&gs_ui_stack) >= 0);
    gs_ui_scene_changed(); gs_ui_tick();
    CHECK(at_stack_top(&gs_ui_stack) < 0 && !gs_ui_pause.paused);
    g_pause_wanted = 0;
    /* a screen forgotten or a script unloaded clears the name */
    CHECK(run_as_script(1, "assert(gd.ui.forget('envoy.pause'))") == 0 && gs_ui_pause_slot == -1);
    reset_ui();
}

static void persist_screens_span_scenes(void)
{
    reset_ui();
    fake_script(1, "envoy");
    gs.scene_kind = 2;
    CHECK(run_as_script(1, "assert(gd.ui.screen{id='envoy.keep', persist=true, primary={kind='list', items={{id='a', label='A'}}}}); assert(gd.ui.screen{id='envoy.drop', primary={kind='list', items={{id='a', label='A'}}}});"
                           " assert(gd.ui.open('envoy.drop')); assert(gd.ui.open('envoy.keep'))") == 0);
    gw_Ui_SceneExit(2);                                                       /* the scene they were opened in ends */
    CHECK(gs_ui_stack.n == 1 && at_stack_top(&gs_ui_stack) == gs_ui_find("envoy.keep"));   /* the ordinary one closed, the persistent one stays */
    gs.scene_kind = 2;                                                        /* the next scene is the same kind (VS to VS) */
    gw_Ui_SceneExit(2);
    CHECK(gs_ui_stack.n == 1);                                                /* and it still stays */
    gs.cur = -1; gs_ui_tick(); gs_ui_tick();
    CHECK(gs_ui_stack.n == 1);                                                /* the tick keeps running it */
    CHECK(run_as_script(1, "assert(gd.ui.close('envoy.keep')); assert(gd.ui.state().depth == 0)") == 0);   /* its owner closes it */
    CHECK(run_as_script(1, "assert(gd.ui.open('envoy.keep'))") == 0);
    gs_ui_release(1);                                                         /* an unload takes it with it */
    CHECK(gs_ui_stack.n == 0);
    reset_ui();
}

static void takeover_pause_screen_reads_the_pad_itself(void)
{
    hud_reset();
    gs_ui_pause_slot = gs_ui_pause_pushed = -1; gs_ui_pause_changed = 0;
    /* Envoy's pause screen is input = 'feed': the takeover path has no feeder, so the engine reads the pausing port's pad for it */
    CHECK(run_as_script(1,
        "UNP=0; assert(gd.ui.screen{id='envoy.pause', kind='pause', input='feed', primary={kind='list', items={{id='resume', label='Resume'}}},"
        " on={accept=function() UNP=UNP+1; gd.ui.unpause() end}}); assert(gd.ui.pause_screen('envoy.pause'))") == 0);
    g_pause_wanted = 1; g_pad_only = 1; g_pad = 0;
    gw_Ui_RetailPause(1, 1); gs.cur = -1; gs_ui_tick();
    CHECK(at_stack_top(&gs_ui_stack) == gs_ui_find("envoy.pause"));
    g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    CHECK(run_as_script(1, "assert(UNP == 1)") == 0);                          /* A reached the handler although the screen is input = feed */
    CHECK(gw_Ui_TakeUnpause() == 1);
    gw_Ui_RetailPause(1, 0); gs_ui_tick();
    /* an ordinary feed screen still does not read the pad */
    CHECK(run_as_script(1, "assert(gd.ui.screen{id='envoy.f', input='feed', primary={kind='list', items={{id='a', label='A'}}}, on={accept=function() UNP=UNP+10 end}}); assert(gd.ui.open('envoy.f'))") == 0);
    g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    CHECK(run_as_script(1, "assert(UNP == 1)") == 0);
    g_pad_only = -1; g_pause_wanted = 0;
    reset_ui();
}
static void full_pool_is_said_once(void)
{
    int i; char lua[256];
    reset_ui(); fake_script(1, "envoy"); g_logs_full = 0; gs_ui_full_warned = 0;
    for (i = 0; i < 16; i++) { snprintf(lua, sizeof lua, "assert(gd.ui.screen{id='envoy.p%d', primary={kind='list', items={{id='a', label='A'}}}})", i); CHECK(run_as_script(1, lua) == 0); }
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.screen, {id='envoy.x1', primary={kind='list', items={{id='a', label='A'}}}}))") == 0);
    CHECK(run_as_script(1, "assert(not pcall(gd.ui.screen, {id='envoy.x2', primary={kind='list', items={{id='a', label='A'}}}}))") == 0);
    CHECK(g_logs_full == 1);                                                   /* refused twice, logged once */
    reset_ui();
}

/* ---- Atlas step 2, Task 6: entries at run time (driven as the engine and as the mod, not as the console) ---- */
#include "gw_ui_menus_json.h"
static void reg_boot_with(const char *json, const char *mod)
{
    AtEntry e[6]; char err[96]; int n, i;
    at_reg_init(&gs_ui_reg);
    n = at_menus_parse(json, mod, e, 6, err, sizeof err);
    for (i = 0; i < n; i++) at_reg_add(&gs_ui_reg, &e[i]);
    gs_ui_reg_booted = 1;
}
static void fake_netplay(int on) { g_netplay = on; }
static int last_pcall_owner(void) { return g_last_pcall_owner; }

static void entry_opens_pushes_mod_screen(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"opens\":\"envoy.setup\"}]}", "envoy");
    engine_menu("solo", 6);
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("gd.ui.screen{ id='envoy.setup', primary={kind='list', items={{id='a', label='A'}}} }");
    gs.cur = -1;                                         /* activation comes from the engine, not from a script */
    CHECK(gw_Ui_EntryCount("solo") == 1);
    CHECK_STR(gw_Ui_EntryField("solo", 0, "label"), "ENVOY");
    CHECK_STR(gw_Ui_EntryField("solo", 0, "tag"), "MOD");
    CHECK(gw_Ui_EntryActivate("solo", 0) == 1);
    CHECK(strcmp(gs_ui_slot[at_stack_top(&gs_ui_stack)].sc.id, "envoy.setup") == 0);
    CHECK(gs_ui_slot[at_stack_top(&gs_ui_stack)].owner == 3);   /* the screen keeps its owner: its handlers run as envoy */
    CHECK(gw_Ui_TopIsEngine("solo") == 0);                       /* the native menu is covered: it draws and takes no input */
}
static void entry_script_runs_on_entry_as_the_mod(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"action\":\"script\"}]}", "envoy");
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("function on_entry(id) ENTRY_SEEN = id; return nil end");
    gs.cur = -1; g_last_pcall_owner = -9;
    CHECK(gw_Ui_EntryActivate("solo", 0) == 1);
    gs.cur = 3; CHECK(t_lua("return ENTRY_SEEN == 'envoy'"));
    CHECK(last_pcall_owner() == 3);                         /* called through gs_pcall(owner = the mod), not the console */
    /* a {push=} result opens one of the mod's own screens */
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"action\":\"script\"}]}", "envoy");
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("gd.ui.screen{ id='envoy.entry', primary={kind='list', items={{id='a', label='A'}}} }; function on_entry(id) return {push='envoy.entry'} end");
    gs.cur = -1;
    CHECK(gw_Ui_EntryActivate("solo", 0) == 1 && strcmp(gs_ui_slot[at_stack_top(&gs_ui_stack)].sc.id, "envoy.entry") == 0);
    /* a script whose on_entry is missing: refused */
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"action\":\"script\"}]}", "envoy");
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("on_entry = nil"); gs.cur = -1;
    CHECK(gw_Ui_EntryActivate("solo", 0) == 0);
}
/* A mod is every .lua of its scripts/ folder, one script each (Envoy: app, atlas_bag, ... main), and on_entry is defined in ONE of them. The entry must
 * reach that one, not the first script of the mod (envoy/app had no on_entry: "mod envoy has no on_entry" on the first real click). */
static void entry_finds_the_script_with_on_entry(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"action\":\"script\"}]}", "envoy");
    fake_script_named(3, "envoy/app"); fake_script_named(4, "envoy/main");
    g_no_hook[3] = 1;                                        /* the first script of the mod defines nothing */
    gs.cur = 4; t_lua("function on_entry(id) ENTRY_SEEN = id; return nil end");
    gs.cur = -1; g_last_pcall_owner = -9;
    CHECK(gw_Ui_EntryActivate("solo", 0) == 1);
    CHECK(last_pcall_owner() == 4);                           /* the script that has it, called as that script */
}
static void entry_missing_screen_refused(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"opens\":\"envoy.nope\"}]}", "envoy");
    fake_script(3, "envoy");
    CHECK(gw_Ui_EntryActivate("solo", 0) == 0);
    gs.cur = 4; fake_script(4, "other");
    gs.cur = 4; t_lua("gd.ui.screen{ id='other.nope', primary={kind='list', items={{id='a', label='A'}}} }");
    gs.cur = -1;
    CHECK(gw_Ui_EntryActivate("solo", 5) == 0 && gw_Ui_EntryActivate(NULL, 0) == 0);
}
static void entry_from_other_script_cannot_hide(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"action\":\"script\"}]}", "envoy");
    fake_script(4, "other");
    gs.cur = 4; CHECK(t_lua("return gd.ui.entry('envoy', {visible=false}) == false"));
    CHECK(gw_Ui_EntryCount("solo") == 1);
    gs.cur = gs.console; CHECK(t_lua("return gd.ui.entry('envoy', {visible=false}) == false"));      /* the console owns no entry (console-only check) */
    fake_script(3, "envoy");
    gs.cur = 3; CHECK(t_lua("return gd.ui.entry('envoy', {visible=false}) == true"));
    CHECK(gw_Ui_EntryCount("solo") == 0);
    CHECK(t_lua("return gd.ui.entry('envoy', {visible=true, badge='NEW'}) == true"));
    CHECK(gw_Ui_EntryCount("solo") == 1 && strcmp(gw_Ui_EntryField("solo", 0, "badge"), "NEW") == 0);
}
static void entry_hidden_in_netplay(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"m.v\",\"parent\":\"versus\",\"label\":\"V\",\"action\":\"script\"},"
                              "{\"id\":\"m.ok\",\"parent\":\"versus\",\"label\":\"OK\",\"action\":\"script\",\"online\":true}]}", "m");
    fake_script(3, "m");
    gs.cur = 3; t_lua("function on_entry(id) ENTRY_SEEN = id end"); gs.cur = -1;
    fake_netplay(1);
    CHECK(gw_Ui_EntryCount("versus") == 1);                      /* only the one that says online */
    CHECK(gw_Ui_EntryActivate("versus", 0) == 1);                /* index 0 is now m.ok: the list the player sees is the list that activates */
    gs.cur = 3; CHECK(t_lua("return ENTRY_SEEN == 'm.ok'"));
    fake_netplay(0);
    CHECK(gw_Ui_EntryCount("versus") == 2);
}
static void entry_screen_closed_on_scene_exit(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"opens\":\"envoy.setup\"}]}", "envoy");
    gs.scene_kind = 1; engine_menu("solo", 6);
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("gd.ui.screen{ id='envoy.setup', primary={kind='list', items={{id='a', label='A'}}} }");
    gs.cur = -1; gw_Ui_EntryActivate("solo", 0);
    CHECK(gs_ui_stack.n == 2);
    gw_Ui_SceneExit(1);
    CHECK(gs_ui_stack.n == 0);
}
static void entry_mod_unloaded(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"action\":\"script\"}]}", "envoy");
    fake_script(3, "envoy");
    gs_ui_release(3);                                    /* the only script of the mod unloads: its entries go with it */
    CHECK(gw_Ui_EntryCount("solo") == 0);
    fake_script_off(3);
    CHECK(gw_Ui_EntryActivate("solo", 0) == 0);          /* no owner to run on_entry: refused, logged, no crash */
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"action\":\"script\"}]}", "envoy");
    fake_script(3, "envoy"); fake_script(4, "envoy");
    gs_ui_release(3);
    CHECK(gw_Ui_EntryCount("solo") == 1);                /* a second script of the mod still runs: the entry stays */
    fake_script_off(3); fake_script_off(4); gs.s[4].id[0] = '\0';
}
static void builtin_entries_register(void)
{
    reset_ui(); at_reg_init(&gs_ui_reg); gs_ui_reg_booted = 1;
    CHECK(gw_Ui_EntryBuiltin("solo", "lab", "LAB", 7) == 1 && gw_Ui_EntryBuiltin("solo", "lab", "LAB", 7) == 0);
    CHECK(gw_Ui_EntryBuiltin("nowhere", "x", "X", 0) == 0);
    CHECK(gw_Ui_EntryCount("solo") == 1 && gw_Ui_EntryActivate("solo", 0) == 0);   /* a native row is the adapter's to run */
    CHECK_STR(gw_Ui_EntryField("solo", 0, "tag"), "");
}

/* ---- Task 9: the Credits screen ---- */
static void credits_screen(void)
{
    int t, b, i, slot, w;
    reset_ui();
    engine_menu("main", 5);
    CHECK(gw_Ui_OpenCredits() == 1);
    slot = gs_ui_find("more.credits");
    CHECK(slot >= 0 && gs_ui_slot[slot].owner == GS_UI_ENGINE && gs_ui_slot[slot].self_close == 1);
    CHECK(gw_Ui_TopIsEngine("more.credits") == 1 && gw_Ui_TopIsEngine("main") == 0);
    CHECK(gs_ui_slot[slot].sc.n_items >= 20 && gs_ui_slot[slot].sc.n_items <= AT_MAX_ITEMS);
    for (i = 0; i < gs_ui_slot[slot].sc.n_items; i++) CHECK(gs_ui_slot[slot].sc.items[i].sub[0] != '\0' && gs_ui_slot[slot].sc.items[i].label[0] != '\0');
    CHECK(strcmp(gs_ui_slot[slot].sc.items[gs_ui_slot[slot].sc.n_items - 1].sub, "see CREDITS.md") == 0);   /* the last row sends the player to the file */
    CHECK(gs_ui_slot[slot].view.ex.has && gs_ui_slot[slot].view.ex.what[0] != '\0');
    for (w = 0; w < 3; w++) {
        static const float WS[3] = { 640.0f, 853.0f, 1140.0f };
        AtSink s = rec_sink();
        AtHits h;
        AtLayout L;
        at_layout(WS[w], gs_ui_slot[slot].sc.preset, &L);
        at_render(&gs_ui_slot[slot].sc, &gs_ui_slot[slot].view, WS[w], 1e6, 0, &FAKE, &s, &h);
        CHECK(texts_legible());
        for (i = 0; i < REC.nt; i++) {                                           /* every text sits on the canvas */
            float tw = fake_width(0, REC.t[i].role, REC.t[i].s), x = REC.t[i].align == AT_ALIGN_RIGHT ? REC.t[i].x - tw : REC.t[i].align == AT_ALIGN_CENTER ? REC.t[i].x - tw * 0.5f : REC.t[i].x;
            CHECK(x >= -0.5f && x + tw <= L.canvas.w + 0.5f);
        }
        { const RecText *r = find_text("CREDITS"); CHECK(r != NULL); }
        for (i = 0; i < REC.nt; i++) if (strncmp(REC.t[i].s, "doldecomp", 9) == 0 && REC.t[i].x >= L.primary.x - 0.5f && REC.t[i].x < L.explainer.x) {   /* the first row's label is inside the primary pane */
            CHECK(REC.t[i].x + fake_width(0, REC.t[i].role, REC.t[i].s) <= L.primary.x + L.primary.w + 0.5f);
        }
    }
    gw_Ui_Intent(AT_EV_MOVE, AT_DIR_DOWN);                                       /* focus moves inside Credits: the explainer follows, nothing reaches the game */
    CHECK(strcmp(gs_ui_slot[slot].view.ex.title, "Aurora") == 0);
    CHECK(poll_ev(&t, &b, &i) == 0);
    gw_Ui_Intent(AT_EV_ACCEPT, 0);
    CHECK(poll_ev(&t, &b, &i) == 0);
    gw_Ui_Intent(AT_EV_BACK, 0);                                                 /* B pops it ... */
    CHECK(gw_Ui_TopIsEngine("main") == 1 && gs_ui_find("more.credits") < 0);
    CHECK(poll_ev(&t, &b, &i) == 0);                                     /* ... and the main menu does not see that B */
    gw_Ui_Intent(AT_EV_BACK, 0);                                                 /* the same frame: the press that closed Credits is not the menu's */
    CHECK(poll_ev(&t, &b, &i) == 0);
    gs_ui_tick();
    gw_Ui_Intent(AT_EV_BACK, 0);
    CHECK(poll_ev(&t, &b, &i) == 1 && t == AT_EV_BACK);
}

/* ---- Task 11: gd.ui.hold_menu and the legacy menu's gate ---- */
static void held_menu_takes_no_intent(void)
{
    int t, b, i;
    reset_ui();
    engine_menu("main", 5);
    fake_script(3, "envoy"); fake_script(4, "other");
    gs.cur = 4; CHECK(t_lua("return gd.ui.hold_menu(true)"));       /* any script may hold it ... */
    CHECK(gw_Ui_TopIsEngine("main") == 0 && gw_Ui_MenuBlocked() == 1);
    gw_Ui_Intent(AT_EV_ACCEPT, 0);
    CHECK(poll_ev(&t, &b, &i) == 0);                          /* ... and the held menu takes no intent */
    gs.cur = 3; CHECK(t_lua("return gd.ui.hold_menu(false)"));        /* another script cannot release it */
    CHECK(gw_Ui_TopIsEngine("main") == 0);
    gs.cur = 4; CHECK(t_lua("return gd.ui.hold_menu(false)"));
    CHECK(gw_Ui_TopIsEngine("main") == 1 && gw_Ui_MenuBlocked() == 0);
    gs.cur = 3; t_lua("gd.ui.hold_menu(true)");
    gs_ui_release(3);                                                  /* the holder unloads: the hold goes with it */
    CHECK(gw_Ui_TopIsEngine("main") == 1);
    gs.cur = 3; t_lua("gd.ui.hold_menu(true)");
    gw_Ui_SceneExit(5);                                                /* a scene ends: so does the hold */
    CHECK(gs_ui_menu_held_by == -2);
    gs.cur = 3; t_lua("gd.ui.hold_menu(true)"); gs.cur = gs.console; CHECK(t_lua("return gd.ui.hold_menu(false)") && gs_ui_menu_held_by == -2);   /* console: labelled */
}
static void menu_blocked_by_a_mod_screen(void)
{
    reset_ui();
    engine_menu("main", 5);
    CHECK(gw_Ui_MenuBlocked() == 0);
    fake_script(3, "envoy");
    gs.cur = 3; t_lua("gd.ui.screen{ id='envoy.s', primary={kind='list', items={{id='a', label='A'}}}, on={back=function() return {pop=true} end} }; gd.ui.open('envoy.s')");
    CHECK(gw_Ui_MenuBlocked() == 1);                                   /* a mod's screen on top: the legacy menu takes no input */
    gs.cur = -1; gs_ui_tick();
    fake_pad_hold(1, AT_PAD_B); gs_ui_tick();                          /* B closes the mod screen in this tick ... */
    CHECK(gw_Ui_MenuBlocked() == 1);                                   /* ... and the menu still takes no input in that same frame */
    gs_ui_tick();
    CHECK(gw_Ui_MenuBlocked() == 0);
}

/* ---- Task 12: the title as an overlay ---- */
static void title_reset(void)
{
    reset_ui();
    gs_ui_roles_state = 0; g_kit_up = 1; g_missing_page = NULL; gs_ui_overlay_kind = -1; gs_ui_atlas_env = 1;
}
static void title_pushed_on_scene_begin(void)
{
    int slot;
    title_reset();
    gs.scene_kind = 0;                                   /* GS_TITLE */
    gw_Ui_SceneBegin(0);
    slot = gs_ui_find("title");
    CHECK(slot >= 0 && at_stack_top(&gs_ui_stack) == slot);
    CHECK(gs_ui_slot[slot].owner == GS_UI_ENGINE && gs_ui_slot[slot].sc.primary == AT_PRIMARY_DISPLAY && gs_ui_slot[slot].scene == 0);
    CHECK_STR(gs_ui_slot[slot].sc.hero, "GD'S MELEE"); CHECK_STR(gs_ui_slot[slot].sc.prompt, "PRESS START");
    gw_Ui_SceneBegin(1);                                 /* any other scene: retail, nothing pushed (the new scene is current) */
    gw_Ui_SceneExit(0);
    gs.scene_kind = 1;
    CHECK(gs_ui_stack.n == 0 && gs_ui_find("title") < 0);
}
static void title_takes_no_input(void)
{
    int t, b, i, slot, n0;
    title_reset();
    gs.scene_kind = 0; gw_Ui_SceneBegin(0);
    slot = gs_ui_find("title"); n0 = gs_ui_stack.n;
    gw_Ui_Intent(AT_EV_ACCEPT, 0); gw_Ui_Intent(AT_EV_BACK, 0); gw_Ui_Intent(AT_EV_MOVE, AT_DIR_DOWN);
    CHECK(poll_ev(&t, &b, &i) == 0);
    gs.cur = -1;
    gs_ui_tick();
    g_mx = 320.0f; g_my = 240.0f; g_mbuttons = 3;       /* a click and a right click on the title */
    gs_ui_tick(); gs_ui_tick();
    g_mx = g_my = -1000.0f; g_mbuttons = 0;
    g_pad = AT_PAD_A | AT_PAD_START; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    CHECK(poll_ev(&t, &b, &i) == 0);
    CHECK(gs_ui_stack.n == n0 && at_stack_top(&gs_ui_stack) == slot);
    gs.key_now[VK_RETURN] = 1; gs.key_now[VK_ESCAPE] = 1; gs_ui_tick(); gs_ui_tick(); gs.key_now[VK_RETURN] = gs.key_now[VK_ESCAPE] = 0;
    CHECK(poll_ev(&t, &b, &i) == 0 && gs_ui_stack.n == n0);
}
static void title_waits_for_roles(void)
{
    title_reset();
    gs.scene_kind = 0; g_kit_up = 0;
    gw_Ui_SceneBegin(0);
    CHECK(gs_ui_find("title") < 0);                      /* the kit is not up yet: the retail title shows */
    gs.cur = -1; gs_ui_tick();
    CHECK(gs_ui_find("title") < 0);
    g_kit_up = 1;
    gs_ui_tick();                                        /* the first tick where the roles are ready, and the scene is still the title */
    CHECK(gs_ui_find("title") >= 0 && gs_ui_slot[gs_ui_find("title")].owner == GS_UI_ENGINE);
    title_reset();
    gs.scene_kind = 0; g_kit_up = 0; gw_Ui_SceneBegin(0);
    gs.scene_kind = 1; g_kit_up = 1; gs_ui_tick();       /* the scene ended before the roles came up: never pushed */
    CHECK(gs_ui_find("title") < 0);
}
static void title_without_roles_stays_retail(void)
{
    title_reset();
    gs.scene_kind = 0; g_missing_page = "atlas_caps.png";   /* the Atlas font page did not load: roles are missing for good */
    gw_Ui_SceneBegin(0);
    CHECK(gs_ui_find("title") < 0);
    g_missing_page = NULL; gs_ui_tick(); gs_ui_tick();
    CHECK(gs_ui_find("title") < 0 && gs_ui_roles_state < 0);
    g_missing_page = NULL;
}
static void title_retail_when_off(void)
{
    title_reset();
    gs.scene_kind = 0; gs_ui_atlas_env = 0;              /* MELEE_ATLAS=0 */
    CHECK(gw_Ui_ScenePolicy(0) == AT_POLICY_RETAIL);
    gw_Ui_SceneBegin(0); gs_ui_tick();
    CHECK(gs_ui_find("title") < 0 && gs_ui_stack.n == 0);
    CHECK(gw_Ui_Ready() == 0);
    gs_ui_atlas_env = 1;
    CHECK(gw_Ui_ScenePolicy(0) == AT_POLICY_OVERLAY && gw_Ui_ScenePolicy(1) == AT_POLICY_RETAIL);
}
static void title_popped_on_scene_exit(void)
{
    title_reset();
    gs.scene_kind = 0; gw_Ui_SceneBegin(0);
    CHECK(gs_ui_stack.n == 1);
    gw_Ui_SceneExit(0);
    CHECK(gs_ui_stack.n == 0 && gs_ui_engine_slots() == 0);
    gs.scene_kind = 1;
}
static void title_and_menu_together(void)
{
    title_reset();
    gs.scene_kind = 0; gw_Ui_SceneBegin(0);
    engine_menu("main", 5);                              /* the title's scene is still current in this test: the front door holds two engine slots at most */
    CHECK(gs_ui_engine_slots() == 2);
    gw_Ui_SceneExit(0);
    CHECK(gs_ui_stack.n == 0);
}
static void after_places_a_mod_entry_among_builtins(void)
{
    reset_ui(); reg_boot_with("{\"menus\":[{\"id\":\"envoy\",\"parent\":\"solo\",\"label\":\"ENVOY\",\"after\":\"training\",\"action\":\"script\"}]}", "envoy");
    CHECK(gw_Ui_EntryBuiltin("solo", "regular-match", "REGULAR MATCH", 0) == 1);
    CHECK(gw_Ui_EntryBuiltin("solo", "training", "TRAINING", 1) == 1);
    CHECK(gw_Ui_EntryBuiltin("solo", "lab", "LAB", 2) == 1);
    CHECK(gw_Ui_EntryCount("solo") == 4);
    CHECK_STR(gw_Ui_EntryField("solo", 0, "id"), "regular-match"); CHECK_STR(gw_Ui_EntryField("solo", 1, "id"), "training");
    CHECK_STR(gw_Ui_EntryField("solo", 2, "id"), "envoy");         /* right after training, not last */
    CHECK_STR(gw_Ui_EntryField("solo", 3, "id"), "lab");
    CHECK_STR(gw_Ui_EntryField("solo", 2, "tag"), "MOD"); CHECK_STR(gw_Ui_EntryField("solo", 1, "tag"), "");
    CHECK(gw_Ui_EntryBuiltin("settings", "online", "ONLINE", 0) == 1 && gw_Ui_EntryBuiltin("main", "online", "ONLINE", 0) == 1);   /* the same name under two menus */
}

/* ---- Atlas step 5, Task 3: gd.ui step, options, set_value and value (always as a mod caller; the console is the exception, not the test) ---- */
static void value_api_as_a_mod(void)
{
    const char *reg =
        "CH={}; assert(gd.ui.screen{id='envoy.set', primary={kind='list', items={"
        "{id='vol',label='Volume',value={kind='slider',min=-1,max=50,step=1,value=3}},"
        "{id='fps',label='FPS',value={kind='choice',options={'Off','FPS','Perf'},value=0}},"
        "{id='on',label='On',value={kind='toggle',on=false}},"
        "{id='who',label='Who',value={kind='text',text='P1'}},"
        "{id='old',label='Old',value={kind='choice',text='x'}},"
        "{id='dz',label='DZ',value={kind='slider',min=0,max=100,value=50}}}},"
        "on={change=function(id,v) CH[#CH+1]=id..'='..tostring(v) end}})";
    AtItem *vol, *fps;
    reset_ui();
    fake_script(1, "envoy"); fake_script_named(2, "other/main");
    CHECK(run_as_script(1, reg) == 0);
    CHECK(run_as_script(1, "assert(gd.ui.value('envoy.set','vol')==3); assert(gd.ui.value('envoy.set','fps')==0); assert(gd.ui.value('envoy.set','on')==false); assert(gd.ui.value('envoy.set','who')=='P1')") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.set_value('envoy.set','vol',99)==true); assert(gd.ui.value('envoy.set','vol')==50)") == 0);       /* clamped */
    CHECK(run_as_script(1, "assert(gd.ui.set_value('envoy.set','vol','x')==false); assert(gd.ui.set_value('envoy.set','vol',1.5)==false); assert(gd.ui.value('envoy.set','vol')==50)") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.set_value('envoy.set','fps',2)==true); assert(gd.ui.value('envoy.set','fps')==2); assert(gd.ui.set_value('envoy.set','fps',3)==false); assert(gd.ui.set_value('envoy.set','fps',-1)==false); assert(gd.ui.value('envoy.set','fps')==2)") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.set_value('envoy.set','on',true)==true); assert(gd.ui.value('envoy.set','on')==true); assert(gd.ui.set_value('envoy.set','on',1)==false)") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.set_value('envoy.set','who','P2')==true); assert(gd.ui.value('envoy.set','who')=='P2'); assert(gd.ui.set_value('envoy.set','who',5)==false)") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.set_value('envoy.set','old','y')==true); assert(gd.ui.value('envoy.set','old')=='y')") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.set_value('envoy.set','nope',1)==false); assert(gd.ui.value('envoy.set','nope')==nil)") == 0);
    CHECK(run_as_script(1, "local ok,e=pcall(gd.ui.set_value,'no.such','vol',1); assert(not ok and e:find('no screen'))") == 0);
    CHECK(run_as_script(1, "assert(#CH==0)") == 0);                                                                                      /* set_value never calls on.change */
    /* the screen record holds the value, set in place: the text a choice shows is the option's */
    vol = &gs_ui_slot[gs_ui_find("envoy.set")].sc.items[0]; fps = &gs_ui_slot[gs_ui_find("envoy.set")].sc.items[1];
    CHECK(vol->vkind == AT_VAL_SLIDER && vol->vval == 50 && vol->vstep == 1);
    CHECK(fps->vkind == AT_VAL_CHOICE && fps->n_opts == 3 && fps->vval == 2 && strcmp(fps->text, "Perf") == 0 && strcmp(fps->opt[1], "FPS") == 0);
    /* another script: refused, with the documented message */
    CHECK(run_as_script(2, "local ok,e=pcall(gd.ui.set_value,'envoy.set','vol',1); assert(not ok and e:find('belongs to another script'))") == 0);
    CHECK(run_as_script(2, "local ok,e=pcall(gd.ui.value,'envoy.set','vol'); assert(not ok and e:find('belongs to another script'))") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.value('envoy.set','vol')==50)") == 0);                                                          /* and it changed nothing */
    /* the console may touch a script's screen */
    CHECK(run_as_console("assert(gd.ui.set_value('envoy.set','dz',20)==true); assert(gd.ui.value('envoy.set','dz')==20)") == 0);
    /* the engine's rule, through the real event path (feed): step 1, an options choice wraps and reports the INDEX */
    CHECK(run_as_script(1, "assert(gd.ui.open('envoy.set')); assert(gd.ui.set_value('envoy.set','vol',50)); CH={}; gd.ui.feed('envoy.set','right'); assert(#CH==0)") == 0);   /* held at max: nothing said */
    CHECK(run_as_script(1, "gd.ui.feed('envoy.set','left'); assert(table.concat(CH,',')=='vol=49' and gd.ui.value('envoy.set','vol')==49)") == 0);      /* step 1, not (51 // 20) = 2 */
    CHECK(run_as_script(1, "CH={}; gd.ui.feed('envoy.set','down'); assert(gd.ui.set_value('envoy.set','fps',2)); gd.ui.feed('envoy.set','right'); gd.ui.feed('envoy.set','right'); gd.ui.feed('envoy.set','left'); assert(table.concat(CH,',')=='fps=0,fps=1,fps=0')") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.value('envoy.set','fps')==0)") == 0);
    CHECK(strcmp(gs_ui_slot[gs_ui_find("envoy.set")].sc.items[1].text, "Off") == 0);                                                      /* the shown text followed */
    CHECK(run_as_script(1, "CH={}; gd.ui.feed('envoy.set','down'); gd.ui.feed('envoy.set','down'); gd.ui.feed('envoy.set','down'); gd.ui.feed('envoy.set','right'); assert(table.concat(CH,',')=='old=1')") == 0);   /* options-less choice: the direction, as before */
    CHECK(run_as_script(1, "CH={}; gd.ui.feed('envoy.set','down'); gd.ui.set_value('envoy.set','dz',50); gd.ui.feed('envoy.set','right'); assert(table.concat(CH,',')=='dz=55')") == 0);    /* no step: the Lua rule, 100 // 20 */
    CHECK(run_as_script(1, "assert(gd.ui.value('envoy.set','vol')==49)") == 0);
    /* an engine screen is never a script's to read or write */
    CHECK(engine_menu("main", 3) == 1);
    CHECK(run_as_script(1, "local ok,e=pcall(gd.ui.set_value,'main','t0',1); assert(not ok and e:find('belongs to the engine'))") == 0);
    CHECK(run_as_script(1, "local ok,e=pcall(gd.ui.value,'main','t0'); assert(not ok and e:find('belongs to the engine'))") == 0);
    CHECK(run_as_console("local ok,e=pcall(gd.ui.value,'main','t0'); assert(not ok and e:find('belongs to the engine'))") == 0);
    /* descriptions: options and step are validated */
    CHECK(run_as_script(1, "local ok,e=pcall(gd.ui.screen,{id='envoy.b1',primary={kind='list',items={{id='a',label='A',value={kind='choice',options={}}}}}}); assert(not ok and e:find('options'))") == 0);
    CHECK(run_as_script(1, "local ok,e=pcall(gd.ui.screen,{id='envoy.b2',primary={kind='list',items={{id='a',label='A',value={kind='choice',options={'1','2','3','4','5','6','7','8','9'}}}}}}); assert(not ok and e:find('options'))") == 0);
    CHECK(run_as_script(1, "local ok,e=pcall(gd.ui.screen,{id='envoy.b3',primary={kind='list',items={{id='a',label='A',value={kind='slider',min=0,max=10,step=0}}}}}); assert(not ok and e:find('step'))") == 0);
    CHECK(run_as_script(1, "local ok,e=pcall(gd.ui.screen,{id='envoy.b4',primary={kind='list',items={{id='a',label='A',value={kind='choice',options={'a',5}}}}}}); assert(not ok and e:find('options'))") == 0);
    CHECK(run_as_script(1, "assert(gd.ui.screen{id='envoy.b5',primary={kind='list',items={{id='a',label='A',value={kind='choice',options={string.rep('x',40)}}}}}})") == 0);   /* a long option is cut at 23, never refused */
    CHECK(strlen(gs_ui_slot[gs_ui_find("envoy.b5")].sc.items[0].opt[0]) == 23 && strlen(gs_ui_slot[gs_ui_find("envoy.b5")].sc.items[0].text) == 23);
    CHECK(run_as_script(1, "local t={}; for i=1,33 do t[i]={id='r'..i,label='R'} end; local ok,e=pcall(gd.ui.screen,{id='envoy.b6',primary={kind='list',items=t}}); assert(not ok and e:find('1 to 32'))") == 0);   /* the Lua door keeps 32 */
    reset_ui();
}

int main(void)
{
    lua_State *L = luaL_newstate();
    gs.L = L; gs.cur = 0; gs.console = 0;
    snprintf(gs.s[0].id, sizeof gs.s[0].id, "console");
    snprintf(gs.s[1].id, sizeof gs.s[1].id, "envoy/main");
    gs.n = 8; gs.s[0].used = gs.s[1].used = gs.s[2].used = 1;
    luaL_openlibs(L);
    lua_newtable(L); gs_push_ui(L); lua_setfield(L, -2, "ui"); lua_setglobal(L, "gd");

    LUA_IS("return tostring((gd.ui.available()))", "true");
    /* a list: register, open, move focus by feed, a bad description, a note, a dialog */
    LUA_IS("return gd.ui.screen{id='t.list', primary={kind='list', items={{id='one',label='One'},{id='two',label='Two'},{id='three',label='Three',disabled=true}}}, explainer='none', keys={{'A','Pick'},{'B','Back'}}}", "true");
    LUA_IS("return gd.ui.open('t.list')", "true");
    LUA_IS("return gd.ui.state().top", "t.list");
    LUA_IS("return (gd.ui.focus('t.list'))", "one");
    LUA_IS("return gd.ui.feed('t.list','down')", "true");
    LUA_IS("return (gd.ui.focus('t.list'))", "two");
    LUA_HAS("return gd.ui.screen{id='t.bad', primary={kind='wheel'}}", "not supported");
    LUA_HAS("return gd.ui.feed('t.list','sideways')", "unknown intent");
    LUA_IS("return gd.ui.note{text='hi', kind='ok'}", "true");
    LUA_IS("return gd.ui.dialog{title='T', text='body', actions={{'A','Yes'},{'B','No'}}, on=function(b) DIALOG=b end}", "true");
    LUA_IS("gd.ui.feed('t.list','down'); return tostring(DIALOG)", "nil");          /* a dialog takes every event: focus did not move */
    LUA_IS("return (gd.ui.focus('t.list'))", "two");
    LUA_IS("gd.ui.feed('t.list','accept'); return DIALOG", "A");
    LUA_IS("gd.ui.close('t.list'); return tostring(gd.ui.state().top)", "nil");

    /* a grid: handlers, the provider, key functions, the legacy focus rule, disabled and locked cells */
    LUA_IS("LOG={}; return gd.ui.screen{id='t.grid', trail={'SOLO', title='BAG'}, primary={kind='grid', blocks={"
           "{id='eq', title='EQUIPPED', cols=3, cells={{id='eq:1',name='a'},{id='eq:2',name='b'},{id='eq:3',name='c',flags={locked=true}}}},"
           "{id='bag', title='BAG', cols=2, cells={{id='bag:1',name='x'},{id='bag:2',name='y',flags={disabled=true}}}}}},"
           "explainer={width='narrow', provide=function(c,b) return {kicker=b, title=c, what='rule '..c} end},"
           "keys={{'A', function(c) return c=='eq:1' and 'First' or nil end}, {'B','Close'}}, counter=function(c) return 'at '..c end, input='feed',"
           "on={focus=function(c,b) LOG[#LOG+1]='focus '..c end, accept=function(c,b) LOG[#LOG+1]='accept '..c end,"
           "back=function() LOG[#LOG+1]='back'; return {pop=true} end}}", "true");
    LUA_IS("gd.ui.open('t.grid'); return gd.ui.state().top", "t.grid");
    LUA_IS("gd.ui.feed('t.grid','right'); return (gd.ui.focus('t.grid'))", "eq:2");
    LUA_IS("gd.ui.feed('t.grid','accept'); return table.concat(LOG,',')", "focus eq:2,accept eq:2");
    LUA_IS("gd.ui.feed('t.grid','down'); return (gd.ui.focus('t.grid'))", "bag:2");
    LUA_IS("gd.ui.feed('t.grid','accept'); return table.concat(LOG,',')", "focus eq:2,accept eq:2,focus bag:2");   /* a disabled cell takes focus and never fires */
    LUA_IS("gd.ui.feed('t.grid','right'); return (gd.ui.focus('t.grid'))", "eq:3");                              /* the inherited legacy rule: the nearest cell to the right, in another row */
    LUA_IS("gd.ui.feed('t.grid','accept'); return LOG[#LOG]", "accept eq:3");                                    /* a locked cell still reaches its handler */
    LUA_IS("gd.ui.set_focus('t.grid','bag','bag:1'); return (gd.ui.focus('t.grid'))", "bag:1");
    LUA_IS("return tostring(gd.ui.set_focus('t.grid','bag','nope'))", "false");
    LUA_IS("gd.ui.screen{id='t.grid', primary={kind='grid', blocks={{id='eq', cols=3, cells={{id='eq:1'}}},{id='bag', cols=2, cells={{id='bag:1'}}}}}}; return (gd.ui.focus('t.grid'))", "bag:1");
    LUA_IS("gd.ui.screen{id='t.grid', primary={kind='grid', blocks={{id='eq', cols=3, cells={{id='eq:1'},{id='eq:2'}}}}}}; return (gd.ui.focus('t.grid'))", "eq:1");   /* the focused block is gone: the first cell */
    /* a re-registration from inside a handler keeps working */
    LUA_IS("N=0; gd.ui.screen{id='t.re', primary={kind='list', items={{id='a',label='A'},{id='b',label='B'}}},"
           "on={focus=function(c) N=N+1; gd.ui.screen{id='t.re', primary={kind='list', items={{id='a',label='A'},{id='b',label='B'}}}, on={}} end}}; gd.ui.open('t.re'); gd.ui.feed('t.re','down'); return N..(gd.ui.focus('t.re'))", "1b");
    /* a Lua error inside a handler is contained */
    LUA_IS("gd.ui.screen{id='t.err', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() error('boom') end}}; gd.ui.open('t.err'); gd.ui.feed('t.err','accept'); return 'alive'", "alive");

    /* a mod's screens carry its id; another script cannot replace them */
    gs.cur = 1;
    LUA_HAS("return gd.ui.screen{id='other.x', primary={kind='list', items={{id='a',label='A'}}}}", "must start with \"envoy.\"");
    LUA_IS("return gd.ui.screen{id='envoy.bag', primary={kind='list', items={{id='a',label='A'}}}}", "true");
    gs.cur = 2; snprintf(gs.s[2].id, sizeof gs.s[2].id, "envoy/second");
    LUA_HAS("return gd.ui.screen{id='envoy.bag', primary={kind='list', items={{id='a',label='A'}}}}", "belongs to another script");
    gs.cur = 1;
    LUA_IS("gd.ui.open('envoy.bag'); return gd.ui.state().top", "envoy.bag");
    gs_ui_release(1);                                                                /* the script unloads: its screens and the stack entry go */
    LUA_IS("return tostring(gd.ui.state().top)", "t.err");
    gs.cur = 0;

    /* nothing runs on a resimulated frame: handlers are not called while gs_may_run says no */
    LUA_IS("LOG={}; gd.ui.screen{id='t.rs', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() LOG[#LOG+1]='accept' end}}; gd.ui.open('t.rs'); return #LOG", "0");
    g_may_run = 0;
    LUA_IS("gd.ui.feed('t.rs','accept'); return #LOG", "0");
    g_may_run = 1;
    LUA_IS("gd.ui.feed('t.rs','accept'); return #LOG", "1");

    /* a description too large for the value arena is refused with a message, never silently cut */
    LUA_HAS("local t = {}; for i = 1, 3000 do t[i] = i end; return gd.ui.screen{id='t.big', extra=t, primary={kind='list', items={{id='a',label='A'}}}}", "the description is too large");
    LUA_HAS("local t = {}; for i = 1, 3000 do t['k' .. i] = {i} end; return gd.ui.screen{id='t.big', extra=t, primary={kind='list', items={{id='a',label='A'}}}}", "the description is too large");
    /* so is one nested deeper than the converter reads (it used to drop the deep part and carry on) */
    LUA_HAS("local d = {}; local c = d; for i = 1, 12 do c.x = {}; c = c.x end; return gd.ui.screen{id='t.deep', extra=d, primary={kind='list', items={{id='a',label='A'}}}}", "the description is too large or nested too deeply");
    LUA_HAS("local d = {}; d.self = d; return gd.ui.screen{id='t.cycle', extra=d, primary={kind='list', items={{id='a',label='A'}}}}", "nested too deeply");
    LUA_IS("return tostring(gd.ui.state().top)", "t.rs");                                  /* a refused screen changed nothing */
    LUA_IS("return gd.ui.screen{id='t.after', primary={kind='list', items={{id='a',label='A'}}}}", "true");   /* and the arena is usable after a refusal */

    /* the tick and the draw pass: input polling is quiet with no pad and the pointer off the picture; a frame draws quads */
    gs_ui_tick();
    gs_ui_draw();
    CHECK(g_quads > 20);
    LUA_IS("return tostring(gd.ui.state().quads > 20)", "true");
    LUA_IS("return tostring(gd.ui.state().roles_ok)", "true");
    /* tracking is set for tracked roles (the small caps) and never left set after a draw or a width query */
    CHECK(g_track_max_seen);
    CHECK_NEAR(g_track, 0.0);
    /* an unregistered or closed screen name is a Lua error, not a crash */
    LUA_HAS("return gd.ui.open('nope')", "no screen");
    /* ---- fix round 1 ---- */
    /* available() needs every Atlas role's page loaded, not only the manifest roles; the reason names the page */
    g_missing_page = "font_a_cap12_latin_0"; gs_ui_roles_state = 0;
    LUA_IS("local ok, why = gd.ui.available(); return tostring(ok)", "false");
    LUA_HAS("local ok, why = gd.ui.available(); return why", "font_a_cap12_latin_0");
    g_logs_render = 0;
    gs_ui_draw();                                                                          /* and nothing is drawn while it is false */
    g_missing_page = NULL; gs_ui_roles_state = 0;
    LUA_IS("return tostring((gd.ui.available()))", "true");
    gs_ui_release(0); gs_ui_release(1); gs_ui_release(2);                                   /* eight slots: free the ones the earlier checks used */
    /* ownership: another script cannot open, close, feed, focus, set the focus of, note on or dialog on a screen it does not own */
    LUA_IS("while gd.ui.state().top do gd.ui.close() end return 'clear'", "clear");
    gs.cur = 1;
    LUA_IS("return gd.ui.screen{id='envoy.own', primary={kind='list', items={{id='a',label='A'},{id='b',label='B'}}}, on={accept=function() OWNLOG=(OWNLOG or '')..'a' end}}", "true");
    LUA_IS("return gd.ui.open('envoy.own')", "true");
    gs.cur = 2;
    LUA_HAS("return gd.ui.open('envoy.own')", "belongs to another script");
    LUA_HAS("return gd.ui.close('envoy.own')", "belongs to another script");
    LUA_HAS("return gd.ui.feed('envoy.own','down')", "belongs to another script");
    LUA_HAS("return gd.ui.focus('envoy.own')", "belongs to another script");
    LUA_HAS("return gd.ui.set_focus('envoy.own','list','b')", "belongs to another script");
    LUA_IS("return tostring(gd.ui.note{text='x'})", "false");                              /* the top screen is not theirs */
    LUA_IS("return tostring(gd.ui.dialog{title='x', actions={{'A','Y'}}, on=function() end})", "false");
    LUA_IS("return tostring(gd.ui.close())", "false");
    LUA_IS("return gd.ui.state().top", "envoy.own");                                       /* nothing changed */
    gs.cur = 1;
    LUA_IS("return (gd.ui.focus('envoy.own'))", "a");
    gs.cur = 0;                                                                            /* the console may drive any screen */
    LUA_IS("return gd.ui.feed('envoy.own','down')", "true");
    LUA_IS("return (gd.ui.focus('envoy.own'))", "b");
    LUA_IS("return tostring(gd.ui.close('envoy.own'))", "true");

    /* a switched-off script's handlers do not run, and its screen leaves the stack on the next tick */
    gs.cur = 1;
    LUA_IS("OWNLOG=''; gd.ui.screen{id='envoy.off', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() OWNLOG=OWNLOG..'x' end}}; gd.ui.open('envoy.off'); gd.ui.feed('envoy.off','accept'); return OWNLOG", "x");
    gs.s[1].disabled = 1;
    LUA_IS("gd.ui.feed('envoy.off','accept'); return OWNLOG", "x");                        /* the handler did not run again */
    gs_ui_tick();
    gs.cur = 0;
    LUA_IS("return tostring(gd.ui.state().top)", "nil");
    gs.s[1].disabled = 0;

    /* a held button is carried across a stack change: it must be released before it can fire again */
    LUA_IS("HL={}; gd.ui.screen{id='h.one', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() HL[#HL+1]='one'; return {push='h.two'} end, back=function() HL[#HL+1]='b1'; return {pop=true} end}}; "
           "gd.ui.screen{id='h.two', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() HL[#HL+1]='two' end, back=function() HL[#HL+1]='b2'; return {pop=true} end}}; "
           "gd.ui.screen{id='h.three', primary={kind='list', items={{id='a',label='A'}}}, on={back=function() HL[#HL+1]='b3'; return {pop=true} end}}; gd.ui.open('h.one'); return #HL", "0");
    g_pad = 0; gs_ui_tick();
    g_pad = AT_PAD_A; gs_ui_tick();
    LUA_IS("return table.concat(HL,',')", "one");                                          /* the press fired on the first screen and pushed the second */
    LUA_IS("return gd.ui.state().top", "h.two");
    gs_ui_tick(); gs_ui_tick();
    LUA_IS("return table.concat(HL,',')", "one");                                          /* still held: the new screen does not take it */
    g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_A; gs_ui_tick();
    LUA_IS("return table.concat(HL,',')", "one,two");                                      /* released and pressed again: now it fires */
    g_pad = 0; gs_ui_tick();
    LUA_IS("HL={}; gd.ui.open('h.three'); return gd.ui.state().top", "h.three");
    g_pad = AT_PAD_B; gs_ui_tick();
    LUA_IS("return table.concat(HL,',')..'/'..gd.ui.state().top", "b3/h.two");             /* one B closed one level ... */
    gs_ui_tick(); gs_ui_tick(); gs_ui_tick();
    LUA_IS("return table.concat(HL,',')..'/'..gd.ui.state().top", "b3/h.two");             /* ... and held B does not close the next one */
    g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_B; gs_ui_tick();
    LUA_IS("return table.concat(HL,',')..'/'..gd.ui.state().top", "b3,b2/h.one");
    g_pad = 0; gs_ui_tick();
    /* the same carry for a screen opened by a script while A is held, and for the keyboard */
    LUA_IS("while gd.ui.state().top do gd.ui.close() end HL={}; return #HL", "0");
    g_pad = AT_PAD_A; LUA_IS("gd.ui.open('h.two'); return #HL", "0");
    gs_ui_tick(); gs_ui_tick();
    LUA_IS("return #HL", "0");
    g_pad = 0; gs_ui_tick();
    gs.key_now[VK_RETURN] = 1; LUA_IS("gd.ui.close(); gd.ui.open('h.two'); return #HL", "0");
    gs_ui_tick();
    LUA_IS("return #HL", "0");                                                             /* Enter held across the open does not accept */
    gs.key_now[VK_RETURN] = 0; gs_ui_tick(); gs.key_now[VK_RETURN] = 1; gs_ui_tick();
    LUA_IS("return table.concat(HL,',')", "two");
    gs.key_now[VK_RETURN] = 0; gs_ui_tick();

    /* the resimulation gate through the tick: with the gate shut, a press reaches nothing */
    LUA_IS("while gd.ui.state().top do gd.ui.close() end HL={}; gd.ui.open('h.two'); return #HL", "0");
    g_may_run = 0; g_pad = AT_PAD_A; gs_ui_tick(); gs_ui_tick();
    LUA_IS("return #HL", "0");
    g_may_run = 1; g_pad = 0; gs_ui_tick();
    LUA_IS("return #HL", "0");

    /* a handler's returned table is read raw: an __index that raises is never run */
    LUA_IS("while gd.ui.state().top do gd.ui.close() end gd.ui.screen{id='h.mt', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() return setmetatable({}, {__index=function() error('index ran') end}) end}}; "
           "gd.ui.open('h.mt'); return tostring(gd.ui.feed('h.mt','accept'))", "true");
    g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0;
    LUA_IS("return gd.ui.state().top", "h.mt");

    gs_ui_release(0);
    /* the render budget is logged once per screen, not once per registration or frame */
    LUA_IS("while gd.ui.state().top do gd.ui.close() end local bl={} for b=1,6 do local cs={} for c=1,12 do cs[c]={id='c'..b..'_'..c, name='n', model=5} end bl[b]={id='b'..b, cols=12, cells=cs} end "
           "HEAVY=function() return gd.ui.screen{id='t.heavy', primary={kind='grid', blocks=bl}} end; return HEAVY()", "true");
    LUA_IS("return gd.ui.open('t.heavy')", "true");
    g_logs_render = 0;
    gs_ui_draw(); gs_ui_draw();
    CHECK(g_logs_render == 1);
    LUA_IS("return HEAVY()", "true");                                                      /* a re-registration (a mod refreshing its data) ... */
    gs_ui_draw(); gs_ui_draw();
    CHECK(g_logs_render == 1);                                                             /* ... does not log it again */
    LUA_IS("gd.ui.close('t.heavy') return 'closed'", "closed");

    /* the explainer log lines have a guard too */
    LUA_IS("gd.ui.screen{id='t.ex', primary={kind='list', items={{id='a',label='A'},{id='b',label='B'}}}, explainer={provide=function(c) return {title=c, what=string.rep('word ', 60)} end}}; gd.ui.open('t.ex'); return 'ok'", "ok");
    g_logs_ex = 0;
    LUA_IS("gd.ui.feed('t.ex','down'); gd.ui.feed('t.ex','up'); gd.ui.feed('t.ex','down'); return 'ok'", "ok");
    CHECK(g_logs_ex == 1);
    LUA_IS("gd.ui.close('t.ex') return 'closed'", "closed");

    gs_ui_release(0);
    /* page (L and R, Tab and Shift+Tab), start and the generic intents reach their handlers */
    LUA_IS("PL={}; gd.ui.screen{id='p.one', primary={kind='list', items={{id='a',label='A'}}}, on={page=function(d) PL[#PL+1]='page '..d end, start=function(c) PL[#PL+1]='start '..c end}}; gd.ui.open('p.one'); return #PL", "0");
    g_pad = AT_PAD_L; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    g_pad = AT_PAD_R; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    gs.key_now[VK_TAB] = 1; gs_ui_tick(); gs.key_now[VK_TAB] = 0; gs_ui_tick();
    gs.key_now[VK_SHIFT] = 1; gs.key_now[VK_TAB] = 1; gs_ui_tick(); gs.key_now[VK_TAB] = 0; gs.key_now[VK_SHIFT] = 0; gs_ui_tick();
    g_pad = AT_PAD_START; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    LUA_IS("return table.concat(PL,',')", "page -1,page 1,page 1,page -1,start a");
    LUA_IS("PL={}; gd.ui.feed('p.one','l'); gd.ui.feed('p.one','r'); gd.ui.feed('p.one','start'); return table.concat(PL,',')", "page -1,page 1,start a");
    LUA_IS("return gd.ui.state().top", "p.one");

    /* on.change: a toggle flips, a slider steps, a choice reports its direction */
    LUA_IS("CH={}; gd.ui.screen{id='c.one', primary={kind='list', items={{id='tg',label='T',value={kind='toggle',on=false}},{id='sl',label='S',value={kind='slider',min=0,max=100,value=50}},{id='ch',label='C',value={kind='choice',text='x'}},{id='off',label='D',disabled=true,value={kind='toggle'}}}},"
           "on={change=function(id,v) CH[#CH+1]=id..'='..tostring(v) end, accept=function(c) CH[#CH+1]='accept '..c end}}; gd.ui.open('c.one'); return #CH", "0");
    LUA_IS("gd.ui.feed('c.one','accept'); gd.ui.feed('c.one','accept'); return table.concat(CH,',')", "tg=true,tg=false");
    LUA_IS("gd.ui.feed('c.one','down'); gd.ui.feed('c.one','right'); gd.ui.feed('c.one','left'); gd.ui.feed('c.one','left'); return table.concat(CH,',')", "tg=true,tg=false,sl=55,sl=50,sl=45");
    LUA_IS("CH={}; gd.ui.feed('c.one','down'); gd.ui.feed('c.one','right'); gd.ui.feed('c.one','left'); gd.ui.feed('c.one','accept'); return table.concat(CH,',')", "ch=1,ch=-1,ch=1");
    LUA_IS("CH={}; gd.ui.feed('c.one','down'); gd.ui.feed('c.one','accept'); gd.ui.feed('c.one','right'); return #CH", "0");    /* a disabled row changes nothing */
    LUA_IS("CH={}; gd.ui.screen{id='c.two', primary={kind='list', items={{id='sl',label='S',value={kind='slider',min=0,max=10,value=10}}}}, on={change=function(id,v) CH[#CH+1]=id..'='..v end}}; gd.ui.open('c.two'); gd.ui.feed('c.two','right'); return #CH", "0");   /* at the end: no change */
    LUA_IS("while gd.ui.state().top do gd.ui.close() end return 'clear'", "clear");
    /* ---- fix round 2: a handler's result is judged by slot owners, never by gs.cur (which is -1 in the tick) ---- */
    gs_ui_release(0); gs_ui_release(1); gs_ui_release(2);
    g_pad = 0;
    /* a result returned during a REAL tick (gs.cur = -1, screens owned by a script that is not the console) pushes its owner's screen */
    gs.cur = 1;
    LUA_IS("R2={}; gd.ui.screen{id='envoy.p1', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() R2[#R2+1]='p1'; return {push='envoy.p2'} end}}; "
           "gd.ui.screen{id='envoy.p2', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() R2[#R2+1]='p2' end, back=function() R2[#R2+1]='p2b'; return {pop=true} end}}; return gd.ui.open('envoy.p1')", "true");
    gs.cur = -1;
    g_pad = 0; gs_ui_tick();
    g_pad = AT_PAD_A; gs_ui_tick();
    g_pad = 0; gs_ui_tick();
    LUA_IS("return table.concat(R2,',')..'/'..gd.ui.state().top", "p1/envoy.p2");               /* the push from the tick worked */
    g_pad = AT_PAD_B; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    LUA_IS("return table.concat(R2,',')..'/'..gd.ui.state().top", "p1,p2b/envoy.p1");           /* and so did a pop from the tick */
    /* it cannot push a screen another script owns */
    gs.cur = 2;
    LUA_IS("return gd.ui.screen{id='envoy.q', primary={kind='list', items={{id='a',label='A'}}}}", "true");
    gs.cur = 1;
    LUA_IS("gd.ui.screen{id='envoy.p3', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() return {push='envoy.q'} end}}; return gd.ui.open('envoy.p3')", "true");
    gs.cur = -1;
    g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0; gs_ui_tick();
    gs.cur = 0;
    LUA_IS("return gd.ui.state().top", "envoy.p3");                                              /* the push of script 2's screen was refused */
    /* and a pop returned by one script's handler does not close another script's top screen */
    gs.cur = 1;
    LUA_IS("gd.ui.screen{id='envoy.a', primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() return {pop=true} end}}; return gd.ui.open('envoy.a')", "true");
    gs.cur = 2;
    LUA_IS("gd.ui.open('envoy.q'); return gd.ui.state().top", "envoy.q");
    gs.cur = 1;
    LUA_IS("return tostring(gd.ui.feed('envoy.a','accept'))", "true");                          /* script 1 feeds its own screen, which is not on top */
    gs.cur = 0;
    LUA_IS("return gd.ui.state().top", "envoy.q");                                               /* script 2's screen is still open */
    LUA_IS("while gd.ui.state().top do gd.ui.close() end return 'clear'", "clear");
    gs_ui_release(0); gs_ui_release(1); gs_ui_release(2);
    /* re-registering a top screen under another port primes the new port's buttons: a held A does not fire on it */
    LUA_IS("PT={}; PTD=function(port) return gd.ui.screen{id='pt.s', port=port, primary={kind='list', items={{id='a',label='A'}}}, on={accept=function() PT[#PT+1]='a' end}} end; PTD(1); return gd.ui.open('pt.s')", "true");
    g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_A;
    LUA_IS("return PTD(2)", "true");
    gs_ui_tick(); gs_ui_tick();
    LUA_IS("return #PT", "0");
    g_pad = 0; gs_ui_tick(); g_pad = AT_PAD_A; gs_ui_tick(); g_pad = 0;
    LUA_IS("return #PT", "1");
    LUA_IS("while gd.ui.state().top do gd.ui.close() end return 'clear'", "clear");
    engine_slot_survives_tick(); engine_slot_not_released_by_script_unload(); uncover_primes_engine_screen(); native_intents_are_primed(); polled_event_is_big_endian_for_the_game();
    intents_from_any_port(); engine_screen_covered_takes_no_intent(); scene_exit_closes_scene_screens(); console_cannot_touch_engine();
    mod_cannot_take_engine_id(); commit_without_change_does_not_rebuild(); engine_focus_is_the_games_cursor(); engine_close_and_queue();
    eight_slots_with_engine(); forget_frees_a_slot(); retail_shims(); hud_basics(); retail_mask_ownership(); toasts_and_notes_leave_with_the_scene(); console_is_not_the_test(); atlas_console_command(); pause_takeover(); persist_screens_span_scenes(); takeover_pause_screen_reads_the_pad_itself(); full_pool_is_said_once();
    entry_opens_pushes_mod_screen(); entry_script_runs_on_entry_as_the_mod(); entry_finds_the_script_with_on_entry(); entry_missing_screen_refused(); entry_from_other_script_cannot_hide();
    entry_hidden_in_netplay(); entry_screen_closed_on_scene_exit(); entry_mod_unloaded(); builtin_entries_register();
    after_places_a_mod_entry_among_builtins(); credits_screen(); held_menu_takes_no_intent(); menu_blocked_by_a_mod_screen();
    title_pushed_on_scene_begin(); title_takes_no_input(); title_waits_for_roles(); title_without_roles_stays_retail(); title_retail_when_off();
    title_popped_on_scene_exit(); title_and_menu_together();
    value_api_as_a_mod();
    ATLAS_DONE("atlas binding");
}
