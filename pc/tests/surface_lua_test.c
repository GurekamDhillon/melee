#include <winsock2.h>
#include <windows.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <assert.h>
#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"
static struct { int cur; } gs;
static int calls, last_slot, last_id;
void gw_log(const char *format, ...) { (void)format; }
uint32_t gw_surface_register(const char *source, const char *label, char *error, unsigned n) {
    assert(source && strstr(source, "gd_surface")); (void)label; (void)error; (void)n; return 1;
}
int gw_surface_select(unsigned slot, uint32_t program, unsigned owner, const float *params) {
    assert(owner == 1 && params); ++calls; last_slot = (int)slot; last_id = (int)program; return 1;
}
int gw_surface_update(unsigned slot, unsigned owner, const float *params) {
    assert(owner == 1 && params); return slot == 1;
}
static int gs_mission_script_root(char out[MAX_PATH]) {
    return GetFullPathNameA("melee/pc/scripts/examples/surface-shaders", MAX_PATH, out, NULL) < MAX_PATH;
}
#include "surface-mission-open.inc"
#include "../platform/gw_script_surface.inc"
static void run(lua_State *L, const char *source) {
    if (luaL_loadstring(L, source) || lua_pcall(L, 0, 0, 0)) {
        fprintf(stderr, "%s\n", lua_tostring(L, -1)); exit(1);
    }
}
int main(void) {
    lua_State *L = luaL_newstate();
    luaL_requiref(L, "_G", luaopen_base, 1); lua_pop(L, 1);
    lua_newtable(L);
    lua_pushcfunction(L, l_fighter_shader); lua_setfield(L, -2, "fighter_shader");
    lua_pushcfunction(L, l_fighter_shader_set); lua_setfield(L, -2, "fighter_shader_set");
    lua_pushcfunction(L, l_stage_shader); lua_setfield(L, -2, "stage_shader");
    lua_setglobal(L, "gd");
    run(L, "assert(gd.fighter_shader(1, 'shaders/rim-light.wgsl', {params={1,2,3}}))");
    assert(calls == 1 && last_slot == 1 && last_id == 1);
    run(L, "assert(gd.fighter_shader_set(1,{params={2,3}})); local ok,e=gd.fighter_shader_set(2,{params={}}); assert(ok==nil and e)");
    run(L, "assert(not pcall(gd.fighter_shader_set,1,{params={1/0}})); assert(not pcall(gd.fighter_shader_set,0,{params={}}))");
    assert(calls == 1); /* no registration/file selection on parameter updates */
    run(L, "assert(gd.stage_shader('shaders/cel-outline.wgsl'))");
    assert(last_slot == 7);
    run(L, "assert(gd.fighter_shader(1,nil))"); assert(last_id == 0);
    run(L, "local ok,e = gd.fighter_shader(1,'shaders/../x.wgsl'); assert(ok==nil and e)");
    run(L, "local ok,e = gd.stage_shader('shaders/rim-light.wgsl',{params={1/0}}); assert(ok==nil and e)");
    run(L, "local ok,e = gd.stage_shader('shaders/rim-light.wgsl',{params={0/0}}); assert(ok==nil and e)");
    run(L, "assert(not pcall(gd.fighter_shader,4294967297,nil))");
    run(L, "assert(not pcall(gd.fighter_shader,0,nil))");
    if (luaL_loadfile(L,"melee/pc/scripts/examples/surface-shaders/scripts/main.lua") || lua_pcall(L,0,0,0)) return 2;
    run(L, "assert(type(on_match_start)=='function' and type(on_match_end)=='function'); on_match_start(); on_match_end()");
    assert(last_id == 0);
    lua_close(L);
    puts("surface Lua loading, containment, invalid params/ports, sample hooks: PASS");
    return 0;
}
