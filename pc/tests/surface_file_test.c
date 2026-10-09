/* Real Lua, calling-mod lookup and file loader; only renderer/mod catalogue
 * are substituted. Restoring the POSIX failure stub breaks the first load. */
#ifdef _WIN32
#include <winsock2.h>
#include <windows.h>
#else
#define MAX_PATH 260
#endif
#include <assert.h>
#include <errno.h>
#include <limits.h>
#include <math.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "../platform/gw.h"
#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"

typedef struct { char entry[MAX_PATH]; } GsScript;
static GsScript script;
static struct { int cur, console; } gs = {0, -1};
static GsScript *gs_cur_script(void) { return &script; }
static int gw_Mods_Count(void) { return 0; }
static int gw_Mods_IsActive(int i) { (void)i; return 0; }
static const char *gw_Mods_Dir(void) { return ""; }
static const char *gw_Mods_Id(int i) { (void)i; return ""; }
static uint64_t gs_fnv(uint64_t h, const void *p, size_t n)
{
    const unsigned char *s = p;
    while (n--) { h ^= *s++; h *= 1099511628211ull; }
    return h;
}
static void gs_setstr(lua_State *L, const char *key, const char *s)
{ lua_pushstring(L, s); lua_setfield(L, -2, key); }
static void gs_setbool(lua_State *L, const char *key, int b)
{ lua_pushboolean(L, b); lua_setfield(L, -2, key); }

static unsigned registrations, selections, last_slot, last_program;
static int refuse_registration;
static float last_params[16];
static const char body[] = "fn gd_surface(c: vec4f, s: GdSurfaceInput) -> vec4f { return c; }\n";
void gw_log(const char *format, ...)
{
    va_list args;
    va_start(args, format); vfprintf(stderr, format, args); va_end(args);
    fputc('\n', stderr);
}
uint32_t gw_surface_register(const char *source, const char *label, char *error, unsigned cap)
{
    size_t n = strlen(source);
    assert(!strncmp(label, "shaders/", 8));
    assert(n == strlen(body) || n == 65536);
    assert(!memcmp(source, body, strlen(body)));
    ++registrations;
    if (refuse_registration) { snprintf(error, cap, "fixture renderer refusal"); return 0; }
    return 42;
}
int gw_surface_select(unsigned slot, uint32_t program, unsigned owner, const float *params)
{
    assert(owner == 1);
    ++selections; last_slot = slot; last_program = program;
    memcpy(last_params, params, sizeof last_params);
    return 1;
}
int gw_surface_update(unsigned slot, unsigned owner, const float *params)
{ (void)slot; (void)owner; (void)params; return 1; }

#include "../platform/gw_script_mission.inc"
#include "../platform/gw_script_surface.inc"

static void run(lua_State *L, const char *source)
{
    if (luaL_loadstring(L, source) || lua_pcall(L, 0, 0, 0)) {
        fprintf(stderr, "%s\n", lua_tostring(L, -1)); exit(1);
    }
}
static void refuses(lua_State *L, const char *path)
{
    unsigned before = registrations, selected = selections;
    lua_pushstring(L, path); lua_setglobal(L, "bad_path");
    run(L, "local ok,e=gd.stage_shader(bad_path); assert(ok==nil and type(e)=='string')");
    assert(registrations == before && selections == selected);
}
int main(int argc, char **argv)
{
    lua_State *L;
    unsigned before;
    assert(argc == 2);
    assert(snprintf(script.entry, sizeof script.entry, "%s/scripts/main.lua", argv[1]) < sizeof script.entry);
    L = luaL_newstate(); assert(L);
    luaL_requiref(L, "_G", luaopen_base, 1); lua_pop(L, 1);
    luaL_requiref(L, "string", luaopen_string, 1); lua_pop(L, 1);
    lua_newtable(L);
    lua_pushcfunction(L, l_fighter_shader); lua_setfield(L, -2, "fighter_shader");
    lua_pushcfunction(L, l_stage_shader); lua_setfield(L, -2, "stage_shader");
    lua_setglobal(L, "gd");
    run(L, "local ok,e=gd.fighter_shader(1,'shaders/probe.wgsl',{params={1,2,3}}); assert(ok,e)");
    assert(registrations == 1 && selections == 1 && last_slot == 1 && last_program == 42);
    assert(last_params[0] == 1 && last_params[2] == 3 && last_params[3] == 0);
    run(L, "assert(gd.stage_shader('shaders/probe.wgsl'))");
    assert(last_slot == 7 && last_program == 42);
    run(L, "assert(gd.stage_shader('shaders/limit.wgsl'))");
    before = registrations;
    run(L, "assert(gd.fighter_shader(1,nil))");
    assert(registrations == before && last_program == 0);
    refuses(L, "shaders/../outside.wgsl");
    refuses(L, "/shaders/probe.wgsl");
    refuses(L, "shaders\\probe.wgsl");
    refuses(L, "shaders/missing.wgsl");
    refuses(L, "shaders/empty.wgsl");
    refuses(L, "shaders/large.wgsl");
    refuses(L, "shaders/nul.wgsl");
    refuses(L, "shaders/directory.wgsl");
    run(L, "local ok,e=gd.stage_shader('shaders/probe.wgsl'..string.char(0)..'ignored'); assert(ok==nil and e)");
    assert(registrations == before);
#ifndef _WIN32
    refuses(L, "shaders/escape.wgsl");
    refuses(L, "shaders/escape-dir/probe.wgsl");
    refuses(L, "shaders/prefix.wgsl");
    refuses(L, "shaders/pipe.wgsl");
    run(L, "assert(gd.stage_shader('shaders/inside.wgsl'))");
#endif
    before = selections;
    refuse_registration = 1;
    run(L, "local ok,e=gd.stage_shader('shaders/probe.wgsl'); assert(ok==nil and e=='fixture renderer refusal')");
    assert(selections == before);
    gs.cur = gs.console;
    refuses(L, "shaders/probe.wgsl");
    gs.cur = 0;
    snprintf(script.entry, sizeof script.entry, "%s/../orphan/main.lua", argv[1]);
    refuses(L, "shaders/probe.wgsl");
    lua_close(L);
    puts("surface files: real Lua loader, calling mod, bounds, containment and renderer handoff PASS");
    return 0;
}
