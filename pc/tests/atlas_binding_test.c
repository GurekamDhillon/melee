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
static struct { lua_State *L; int cur, console, n; GsScript s[8]; unsigned char key_now[256]; } gs;
static int g_may_run = 1, g_quads, g_models;
static double g_now = 1000.0;
static float g_track;          /* what gw_Kit_SetTracking last set */
static int g_track_max_seen;   /* the largest non-zero value any draw saw */
static int g_logs_render;      /* log lines about a render budget */
static unsigned g_pad;         /* the buttons the stand-in pad holds (the raw GC bits) */
static int g_logs_ex;          /* log lines about an explainer */

void gw_log(const char *fmt, ...)
{
    char buf[256];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, sizeof buf, fmt, ap);
    va_end(ap);
    if (strstr(buf, "render budget") != NULL) g_logs_render++;
    if (strstr(buf, "explainer") != NULL) g_logs_ex++;
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
unsigned gw_script_pad_raw_buttons(int ch) { (void) ch; return g_pad; }
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
    ATLAS_DONE("atlas binding");
}
