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

typedef struct { char id[64]; int used, disabled, gameplay; } GsScript;
static struct { lua_State *L; int cur, console; GsScript s[8]; unsigned char key_now[256]; } gs;
static int g_may_run = 1, g_quads, g_models;
static double g_now = 1000.0;
static float g_track;          /* what gw_Kit_SetTracking last set */
static int g_track_max_seen;   /* the largest non-zero value any draw saw */
static int g_logs_render;      /* log lines about a render budget */

void gw_log(const char *fmt, ...)
{
    char buf[256];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof buf, fmt, ap);
    va_end(ap);
    if (strstr(buf, "drew") != NULL || strstr(buf, "render") != NULL) g_logs_render++;
}
static int gs_may_run(int i) { (void) i; return g_may_run; }
static int gs_pcall(int script, int nargs, int nres, const char *what)
{
    (void) script; (void) what;
    if (lua_pcall(gs.L, nargs, nres, 0) != LUA_OK) { lua_pop(gs.L, 1); return -1; }
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
unsigned gw_script_pad_raw_buttons(int ch) { (void) ch; return 0; }
void gw_Mouse_ScriptRead(float *x, float *y, int *buttons, float *wheel) { *x = -1000.0f; *y = -1000.0f; *buttons = 0; *wheel = 0.0f; }
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
void gw_Kit_SetTracking(float px) { g_track = px; }
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
    return buf;
}
#define LUA_IS(code, want) CHECK_STR(lua(code), want)
#define LUA_HAS(code, part) do { const char *_r = lua(code); at_t_count++; if (strstr(_r, part) == NULL) { at_t_fails++; printf("FAIL %s:%d: %s -> \"%s\" lacks \"%s\"\n", __FILE__, __LINE__, code, _r, part); } } while (0)

int main(void)
{
    lua_State *L = luaL_newstate();
    gs.L = L; gs.cur = 0; gs.console = 0;
    snprintf(gs.s[0].id, sizeof gs.s[0].id, "console");
    snprintf(gs.s[1].id, sizeof gs.s[1].id, "envoy/main");
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
    LUA_HAS("return gd.ui.screen{id='t.bad', primary={kind='tiles'}}", "not supported");
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
    ATLAS_DONE("atlas binding");
}
