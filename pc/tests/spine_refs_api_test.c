#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <limits.h>
#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"
static unsigned epoch=1,spawn=100;static int fighter_kind=10,calls;
static int gw_ScriptGame_EntityField(int k,int id,int f){
 if(k!=1||id<0||id>=12)return 0;
 if(f==0)return 1;if(f==1)return epoch;if(f==2)return spawn+id;
 if(f==3)return fighter_kind;if(f==4)return id+1;if(f==5)return id==10;return 0;}
static void gs_setint(lua_State*L,const char*k,int n){lua_pushinteger(L,n);lua_setfield(L,-2,k);}
static void gs_setbool(lua_State*L,const char*k,int n){lua_pushboolean(L,n);lua_setfield(L,-2,k);}
static void gs_setstr(lua_State*L,const char*k,const char*s){lua_pushstring(L,s);lua_setfield(L,-2,k);}
static int target(lua_State*L){++calls;return lua_gettop(L);}
static lua_CFunction gs_spine_call_target(const char*name){return !strcmp(name,"fighter_caps")||!strcmp(name,"fighter_history")||!strcmp(name,"fighter_status")||!strcmp(name,"hit_rules")||!strcmp(name,"echo_add")||!strcmp(name,"zones_at")?target:0;}
#include "../platform/gw_script_spine_refs.inc"
static void run(lua_State*L,const char*s){if(luaL_loadstring(L,s)||lua_pcall(L,0,0,0)){fprintf(stderr,"%s\n",lua_tostring(L,-1));assert(0);}}
int main(void){lua_State*L=luaL_newstate();luaL_requiref(L,"_G",luaopen_base,1);lua_pop(L,1);lua_newtable(L);
#define API(name,fn) lua_pushcfunction(L,fn);lua_setfield(L,-2,name)
API("entity_ref",l_entity_ref);API("entity_resolve",l_entity_resolve);API("entity_valid",l_entity_valid);API("source_ref",l_source_ref);API("source_resolve",l_source_resolve);API("entity_call",l_entity_call);lua_setglobal(L,"gd");
run(L,"r=gd.entity_ref{port=1,sub=true};assert(type(r)=='string');local e=gd.entity_resolve(r);assert(e.port==1 and e.sub and e.capability_entity==7 and e.collision_entity==1);assert(gd.entity_valid(r));"
"local a,b=gd.entity_call(r,'fighter_caps',{air_jumps=3});assert(a==7 and b.air_jumps==3);local p,s=gd.entity_call(r,'hit_rules');assert(p==1 and s==true);"
"local opts={delay=8};local p,o=gd.entity_call(r,'echo_add',opts);assert(p==1 and o.sub and opts.sub==nil);local p,s=gd.entity_call(r,'zones_at');assert(p==1 and s==1);"
"local p,age,sub=gd.entity_call(r,'fighter_history',8);assert(p==1 and age==8 and sub);local p,bits,sub=gd.entity_call(r,'fighter_status',4);assert(p==1 and bits==4 and sub);"
"assert(not pcall(gd.entity_call,r,'echo_add',{sub=false}));assert(not pcall(gd.entity_call,r,'set_percent',20));"
"src=gd.source_ref('drive','rare_42',2,r);local s=gd.source_resolve(src);assert(s.kind=='drive' and s.key=='rare_42' and s.revision==2 and s.entity_ref==r);"
"assert(not gd.entity_valid('e2:1:1:0:100:10'));assert(not gd.entity_valid('e1:1:1:0:4294967296:10'));assert(not gd.entity_valid(r..'\\0garbage'));assert(not gd.entity_valid('e1:01:1:0:100:10'));"
"assert(not pcall(gd.source_ref,'bogus','key'));assert(not pcall(gd.source_ref,'drive','../bad'));assert(not pcall(gd.source_ref,'drive','a\\0b'));"
"assert(not pcall(gd.source_ref,'drive\\0ignored','key'));assert(not pcall(gd.entity_ref,{kind='fighter\\0ignored',port=1}));assert(not pcall(gd.entity_call,r,'fighter_caps\\0ignored',{}));"
"plain=gd.source_ref('fighter','fox_profile',3);assert(gd.source_resolve(plain).key=='fox_profile')");
spawn=999;int before=calls;run(L,"assert(not gd.entity_valid(r));assert(gd.entity_resolve(r)==nil and gd.source_resolve(src)==nil);assert(not pcall(gd.entity_call,r,'fighter_caps',{}))");assert(calls==before);
spawn=100;fighter_kind=11;run(L,"assert(not gd.entity_valid(r))");fighter_kind=10;epoch=2;run(L,"assert(not gd.entity_valid(r))");epoch=1;run(L,"assert(gd.entity_valid(r) and gd.source_resolve(src))");
puts("spine native refs: codec stale/overflow/NUL source provenance adapter primary/sub safety PASS");lua_close(L);return 0;}
