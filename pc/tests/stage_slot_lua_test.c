#include <assert.h>
#include <stdio.h>
#include <stdarg.h>
#include <math.h>
#include <string.h>
#include "lua.h"
#include "lauxlib.h"
#include "lualib.h"
#include "../gameworld/script_model.h"
#define GS_SAVE_SLOTS 4
static struct {int pending_save,pending_load,scene_epoch;struct{int used,scene_epoch;}slot[4];}gs;
static int active=1,logs;
static int gw_ScriptGame_StageSlotsActive(void){return active;}
static void gw_log(const char* f,...){assert(strstr(f,"refused"));++logs;}
static void gs_require_gameplay(lua_State* L,const char* s){(void)L;(void)s;}
static int gs_fbits(float f){union{float f;int i;}v;v.f=f;return v.i;}
static float gs_stage_num(lua_State* L,int at){return (float)luaL_checknumber(L,at);}
static const char* gs_model_fields[SM_FIELDS]={"x","y","z","rot","scale","layer","visible","tint","scale_x","scale_y","scale_z","alpha","background","rot_x","rot_y"};
static int gs_push_fail(lua_State* L,const char* why);
#include "stage_slot_lua_retail.inc"
static int options(lua_State* L){int fields[SM_FIELDS]={0};int top=lua_gettop(L);
    gs_model_options(L,-1,fields);assert(lua_gettop(L)==top);
    assert(fields[SM_X]==gs_fbits(12) && fields[SM_SCALE]==gs_fbits(.8f));return 0;}
int main(int argc,char** argv){lua_State* L=luaL_newstate();luaL_openlibs(L);
    if(argc>1){assert(luaL_loadfile(L,argv[1])==LUA_OK);lua_pop(L,1);}
    lua_pushcfunction(L,options);lua_setglobal(L,"options");
    lua_pushcfunction(L,l_savestate);lua_setglobal(L,"save");
    lua_pushcfunction(L,l_loadstate);lua_setglobal(L,"load");
    assert(luaL_dostring(L,"options({x=12,scale=.8}); for _,f in ipairs{save,load} do local ok,why=f(1); assert(ok==false and type(why)=='string' and #why>20) end")==LUA_OK);
    assert(logs==2 && gs.pending_save==0 && gs.pending_load==0);
    active=0;assert(luaL_dostring(L,"save(1)")==LUA_OK && gs.pending_save==1);
    lua_close(L);puts("stage slot Lua: real VM negative-index options, clear save/load refusal and no pending writes passed");return 0;}
