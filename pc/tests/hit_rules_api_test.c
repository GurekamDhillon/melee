/* Real Lua parser/native packet/journal with the production game-BSS helpers.
 * Standalone source fixture; no EXE/game/snapshot integration claim. */
#define main hit_rules_state_fixture_main
#include "hit_rules_state_test.c"
#undef main
#include <stdlib.h>
#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"
#define gw_ScriptGame_HitRulesOwner ScriptGame_HitRulesOwner
#define gw_ScriptGame_HitRulesCount ScriptGame_HitRulesCount
#define gw_ScriptGame_FighterStatus ScriptGame_FighterStatus
#define gw_ScriptGame_HitRuleRead ScriptGame_HitRuleRead
#define gw_ScriptGame_HitRulesReplace ScriptGame_HitRulesReplace
#define gw_ScriptGame_HitRulesRelease ScriptGame_HitRulesRelease
/* The included state helpers already define Script_HitRulesInput; route actual
 * game helper reads to the native packet by overwriting its input in wrapper. */
static int branches;
static struct {int cur,console,n,match_active,rw_resim,rw_resim_run,rw_began,rw_head;
 struct {int used,disabled,stage_owner;} s[1];} gs;
static int gs_fbits(float f){return script_hr_bits(f);}
static void gs_setbool(lua_State* L,const char* k,int v){lua_pushboolean(L,v);lua_setfield(L,-2,k);}
static void gs_setint(lua_State* L,const char* k,int v){lua_pushinteger(L,v);lua_setfield(L,-2,k);}
static void gs_setnum(lua_State* L,const char* k,double v){lua_pushnumber(L,v);lua_setfield(L,-2,k);}
static void gs_setstr(lua_State* L,const char* k,const char* v){lua_pushstring(L,v);lua_setfield(L,-2,k);}
static int gs_slot_arg(lua_State* L,int at){int p=(int)luaL_checkinteger(L,at);if(p<1||p>6)luaL_error(L,"port");return p-1;}
static void gs_require_gameplay(lua_State* L,const char* n){(void)L;(void)n;}
static void gs_require_offline(lua_State* L,const char* n){(void)L;(void)n;}
static void gs_rw_branch(void){++branches;}
#undef gw_ScriptGame_HitRulesReplace
static int api_replace(int slot,int owner,int count,int status);
#define gw_ScriptGame_HitRulesReplace api_replace
#include "../platform/gw_script_hit_rules.inc"
static int api_replace(int slot,int owner,int count,int status){memcpy(input,gs_hit_rules_input,sizeof input);return ScriptGame_HitRulesReplace(slot,owner,count,status);}
/* This older fixture exercises hit-rule operations only. Mirror the new echo
 * payload layout for journal sizing; echo_api_test.c covers the real adapter. */
typedef struct {int count,rules[8][9];} GsEchoRules;
static int gw_ScriptGame_EchoOwner(int entity){(void)entity;return 0;}
static int gw_ScriptGame_EchoNextHandle(void){return 1;}
static void gw_ScriptGame_EchoClear(int owner){(void)owner;}
static void gs_echo_visual_release(int owner){(void)owner;}
static int gs_echo_sub(lua_State* L,int at){if(lua_isnoneornil(L,at))return 0;luaL_checktype(L,at,LUA_TBOOLEAN);return lua_toboolean(L,at);}
static void gs_echo_parse(lua_State* L,int at,GsEchoRules* rules){(void)at;(void)rules;luaL_error(L,"echo operations belong to echo_api_test");}
static void gs_echo_prepare(lua_State* L,int entity,GsEchoRules* rules,int* next){(void)L;(void)entity;(void)rules;(void)next;}
static void gs_echo_pair(lua_State* L,int a,const GsEchoRules* ar,int b,const GsEchoRules* br){(void)L;(void)a;(void)ar;(void)b;(void)br;}
static int gs_echo_apply(int entity,int owner,const GsEchoRules* rules){(void)entity;(void)owner;(void)rules;return 0;}
/* Consume real journal layout and replay application, with fixture-only game
 * services for unused operation kinds. The journal never derives fixed offsets. */
#define GS_LOG_N 2
#define GS_MAX_SCRIPTS 1
#define GsScript fixture_script
typedef struct fixture_script {int stage_owner;} fixture_script;
static GsScript script={1};
static GsScript* gs_cur_script(void){return &script;}
static int gw_Snap_Resimulating(void){return 0;}
int gw_ScriptGame_SimAvailable(int token){return token==1;}
int gw_ScriptGame_SimSize(int token){(void)token;return -1;}
int gw_ScriptGame_SimByte(int token,int at){(void)token;(void)at;return 0;}
static unsigned char sim_blob[16384];static int sim_size;
int gw_ScriptGame_SimCommit(int token,int size);
void gw_ScriptGame_SimRelease(int token){(void)token;sim_size=0;}
static int gs_ring_now(void){return 0;}
static int gs_rw_replaying(int tag){(void)tag;return 0;}
static void gs_rw_stop(void){}
static void gw_ScriptGame_FighterModsRelease(int owner){(void)owner;}
static int gw_ScriptGame_FighterModOwner(int slot){(void)slot;return 0;}
static int gw_ScriptGame_FighterModRead(int slot,int at){(void)slot;(void)at;return 0;}
static void gw_ScriptGame_FighterMod(int a,int b,int c,int d,int e,int f,int g,int h,int i,int j,int k){(void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;(void)h;(void)i;(void)j;(void)k;}
static void gw_ScriptGame_FighterModExtra(int a,int b,int c,int d,int e){(void)a;(void)b;(void)c;(void)d;(void)e;}
static void gw_ScriptGame_SetPercent(int slot,int bits){(void)slot;(void)bits;}
#include "../platform/gw_script_sim_state.inc"
int gw_ScriptGame_SimCommit(int token,int size){int i;(void)token;for(i=0;i<size;++i)sim_blob[i]=(unsigned char)gw_Script_SimByte(i);sim_size=size;return 1;}
static void run(lua_State* L,const char* text){if(luaL_loadstring(L,text)||lua_pcall(L,0,0,0)){fprintf(stderr,"%s\n",lua_tostring(L,-1));assert(0);}}
int main(void){
 lua_State* L=luaL_newstate();GsHitRules original,replayed;Fighter a={0,0,4,{0}},b={1,0,0,{0}};HitCapsule h;
 luaL_requiref(L,"_G",luaopen_base,1);lua_pop(L,1);
 lua_newtable(L);
 #define API(name,fn) lua_pushcfunction(L,fn);lua_setfield(L,-2,name)
 API("hit_rules",l_hit_rules);API("hit_rule_add",l_hit_rule_add);API("hit_rules_clear",l_hit_rules_clear);API("sim_commit",l_sim_commit);
 lua_setglobal(L,"gd");
 run(L,"assert(gd.hit_rules(1).progression and gd.hit_rules(1).percent_only); "
 "for i=1,32 do gd.hit_rule_add(1,{id=100+i,change={percent_damage=64,launch=4}},false) end; "
 "assert(#gd.hit_rules(1)==32 and gd.hit_rules(1)[32].id==132); "
 "assert(not pcall(gd.hit_rule_add,1,{change={percent_damage=1.2}},false));gd.hit_rules_clear(1,false); "
 "for _,k in ipairs({'damage','knockback_growth','knockback_base','knockback_taken'}) do "
 "assert(not pcall(gd.hit_rule_add,1,{match={incoming=true},change={[k]=.05}},false)) end; "
 "for _,v in ipairs({0/0,1/0,-1/0,1000000001,-1000000001}) do assert(not pcall(gd.hit_rule_add,1,{change={percent_damage=v}},false)) end; "
 "for _,v in ipairs({0/0,1/0,-1/0,1000000001,-1000000001}) do assert(not pcall(gd.hit_rule_add,1,{change={launch=v}},false)) end; "
 "gd.hit_rule_add(1,{change={percent_damage=1000000000,launch=-1000000000}},false);gd.hit_rules_clear(1,false); "
 "gd.hit_rule_add(2,{match={incoming=true},change={percent_damage=-.9,launch=-.9}},false); "
 "gd.hit_rule_add(2,{match={incoming=true,status_bits=8},change={percent_damage=1.6,launch=1.6}},false); "
 "assert(gd.hit_rules(2)[1].change.percent_damage < -.89)");
 gs_hr_read(2,&original);original.status=8;assert(gs_hr_apply(2,1,&original));
 h=capsule(0);ScriptGame_HitRuleCreate(&a,&h,0);ScriptGame_HitRulePercentQueue(&h,&b,10);
 assert(fabsf(ScriptGame_HitRulePercentCommit(&b,10)-1.5f)<.00001f);
 assert(fabsf(ScriptGame_HitRuleContact(&h,&b,10,1)-.5f)<.00001f);
 ScriptGame_HitRulesRelease(0);
 gs.console=-1;gs.match_active=1;gs.n=1;gs.s[0].stage_owner=1;gs.s[0].used=1;
 save_hit_state(&saved_state);
 gs_sim_window=1;gs_sim_tag=1;gs_sim_log[1].tag=1;
 run(L,"local r={} for i=1,32 do r[i]={id=200+i,change={percent_damage=2,launch=1.25}} end r[1].change={percent_damage=-.9,launch=-.9} "
 "assert(gd.sim_commit('checkpoint32',{{op='hit_rules',port=1,rules=r,status_bits=8,sub=false}})); "
 "assert(#gd.hit_rules(1)==32 and gd.hit_rules(1)[32].id==232)");
 gs_hr_read(0,&original);assert(original.rules[0][14]==gs_fbits(-.9f) && original.rules[0][15]==gs_fbits(-.9f));restore_hit_state(&saved_state);sim_size=0;gs_sim_replay(1);gs_hr_read(0,&replayed);
 assert(!memcmp(&original,&replayed,sizeof original));assert(sim_size==12 && !memcmp(sim_blob,"checkpoint32",12));
 assert(gs_sim_bytes==sizeof(GsSimCommit)+12 && gs_sim_bytes<GS_SIM_BUDGET);
 printf("native Lua API + journal32 replay PASS; GsHitRules=%u GsSimOp=%u GsSimCommit=%u allocated=%u bytes\n",(unsigned)sizeof(GsHitRules),(unsigned)sizeof(GsSimOp),(unsigned)sizeof(GsSimCommit),(unsigned)gs_sim_bytes);
 gs_sim_reset();assert(!gs_sim_bytes);ScriptGame_HitRulesRelease(0);lua_close(L);return 0;
}
