/* Standalone production Lua parser/packet tests. Scalar services below model
 * the boundary only; collision acceptance is covered by echo_core_test. */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <limits.h>
#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"
static struct {int cur,console,n,match_active,rw_resim,rw_resim_run,rw_began,rw_head;struct {int used,disabled,stage_owner;}s[1];} gs;
static int branches,gate_closed;
static int gs_fbits(float f){union{int i;float f;}u;u.f=f;return u.i;}
static void gs_setbool(lua_State*L,const char*k,int v){lua_pushboolean(L,v);lua_setfield(L,-2,k);}
static void gs_setint(lua_State*L,const char*k,int v){lua_pushinteger(L,v);lua_setfield(L,-2,k);}
static void gs_setnum(lua_State*L,const char*k,double v){lua_pushnumber(L,v);lua_setfield(L,-2,k);}
static void gs_setstr(lua_State*L,const char*k,const char*v){lua_pushstring(L,v);lua_setfield(L,-2,k);}
static int gs_slot_arg(lua_State*L,int at){lua_Integer p=luaL_checkinteger(L,at);if(p<1||p>6)luaL_error(L,"port");return (int)p-1;}
static void gs_require_gameplay(lua_State*L,const char*n){if(gate_closed)luaL_error(L,"gate %s",n);}
static void gs_rw_branch(void){++branches;}
int gw_ScriptGame_HitRulesOwner(int e){(void)e;return 0;}
int gw_ScriptGame_HitRulesCount(int e){(void)e;return 0;}
int gw_ScriptGame_FighterStatus(int e){(void)e;return 0;}
int gw_ScriptGame_HitRuleRead(int e,int i,int f){(void)e;(void)i;(void)f;return 0;}
int gw_ScriptGame_HitRulesReplace(int e,int o,int n,int s){(void)e;(void)o;(void)n;(void)s;return 1;}
void gw_ScriptGame_HitRulesRelease(int o){(void)o;}
#include "../platform/gw_script_hit_rules.inc"
static void run(lua_State*L,const char*s){if(luaL_loadstring(L,s)||lua_pcall(L,0,0,0)){fprintf(stderr,"%s\n",lua_tostring(L,-1));assert(0);}}
#include "../platform/gw_script_echo.inc"
static GsEchoRules echo_rows[12];static int echo_owner[12],next_echo=1,history_active;
int gw_ScriptGame_FighterHistoryDepth(void){return 61;}
int gw_ScriptGame_FighterHistoryRead(int e,int a,int h,int f){(void)e;(void)a;if(!history_active)return 0;if(h<0)return f==11?77:f==0||f==10?1:0;if(h==(history_active==2?4:3)){if(f==0)return 1;if(f==15)return 1;if(f==16)return 7;if(f==17)return 1;if(f==18)return 2;if(f==19)return h;}return 0;}
int gw_ScriptGame_EchoNextHandle(void){return next_echo;}
int gw_ScriptGame_EchoOwner(int e){return echo_owner[e];}
int gw_ScriptGame_EchoRead(int e,int i,int f){return i<echo_rows[e].count?echo_rows[e].rules[i][f]:0;}
int gw_ScriptGame_EchoReplace(int e,int owner,int n){int i,j;echo_rows[e].count=n;echo_owner[e]=n?owner:0;for(i=0;i<n;++i)for(j=0;j<9;++j)echo_rows[e].rules[i][j]=gw_Script_EchoInput(i*9+j);return 1;}
int gw_ScriptGame_EchoAddWithHandle(int e,int o,int d,int m,int el,int a,int db,int kb,int ov,int once,int handle){int row[]={handle?handle:next_echo++,d,m,el,a,db,kb,ov,once};if(echo_rows[e].count==GS_ECHO_MAX)return 0;memcpy(echo_rows[e].rules[echo_rows[e].count++],row,sizeof row);echo_owner[e]=o;return row[0];}
int gw_ScriptGame_EchoRemove(int handle,int owner){int e,i;for(e=0;e<12;++e)for(i=0;i<echo_rows[e].count;++i)if(echo_rows[e].rules[i][0]==handle&&echo_owner[e]==owner){memmove(echo_rows[e].rules[i],echo_rows[e].rules[i+1],(echo_rows[e].count-i-1)*sizeof echo_rows[e].rules[0]);--echo_rows[e].count;return 1;}return 0;}
void gw_ScriptGame_EchoClear(int owner){int e;for(e=0;e<12;++e)if(echo_owner[e]==owner){echo_owner[e]=0;memset(&echo_rows[e],0,sizeof echo_rows[e]);}}
#include "../platform/gw_motion.h"
static GwMotionOptions mo;static int motion_live;static unsigned motion_owner;
static void gs_item_keys(lua_State*L,int at,const char*const*keys){int n=0;while(keys[n])++n;gs_hr_keys(L,at,keys,n);}
void gw_motion_defaults(GwMotionOptions*o,int kind){memset(o,0,sizeof *o);o->kind=kind;o->port=1;o->copies=3;o->spacing=3;o->lifetime=16;}
int gw_motion_add(unsigned owner,const GwMotionOptions*o,const char**error){(void)owner;(void)error;if(motion_live)return 0;mo=*o;motion_live=1;motion_owner=owner;return 77;}
int gw_motion_get(unsigned owner,int handle,GwMotionOptions*o){(void)owner;if(handle!=77||!motion_live||owner!=motion_owner)return 0;*o=mo;return 1;}
int gw_motion_set(unsigned owner,int h,const GwMotionOptions*o,const char**err){(void)owner;(void)h;(void)err;mo=*o;return 1;}
int gw_motion_remove(unsigned owner,int h){(void)owner;(void)h;motion_live=0;return 1;}
void gw_motion_intensity(float v){(void)v;}
void gw_motion_stats(uint64_t s[GW_MOTION_STATS_COUNT]){memset(s,0,GW_MOTION_STATS_COUNT*sizeof *s);}
const char*gw_motion_stat_name(unsigned i){(void)i;return "stat";}
#include "../platform/gw_script_motion.inc"
int gw_ScriptGame_EchoState(int e,int h,int f){(void)e;(void)h;return f==0?1:f==1?1:2;}
#include "../platform/gw_script_echo_visual.inc"
#define GS_LOG_N 2
#define GS_MAX_SCRIPTS 1
typedef struct {int stage_owner;} GsScript;
static GsScript fixture_script={1};
static GsScript*gs_cur_script(void){return &fixture_script;}
static int gw_Snap_Resimulating(void){return 0;}
int gw_ScriptGame_SimAvailable(int token){return token==1;}
int gw_ScriptGame_SimSize(int token){(void)token;return -1;}
int gw_ScriptGame_SimByte(int token,int at){(void)token;(void)at;return 0;}
int gw_ScriptGame_SimCommit(int token,int size);
void gw_ScriptGame_SimRelease(int token){(void)token;}
static int gs_ring_now(void){return 0;}
static int gs_rw_replaying(int tag){(void)tag;return 0;}
static void gs_rw_stop(void){}
static void gs_require_offline(lua_State*L,const char*n){gs_require_gameplay(L,n);}
static void gw_ScriptGame_FighterModsRelease(int owner){(void)owner;}
static int gw_ScriptGame_FighterModOwner(int slot){(void)slot;return 0;}
static int gw_ScriptGame_FighterModRead(int slot,int at){(void)slot;(void)at;return 0;}
static void gw_ScriptGame_FighterMod(int a,int b,int c,int d,int e,int f,int g,int h,int i,int j,int k){(void)a;(void)b;(void)c;(void)d;(void)e;(void)f;(void)g;(void)h;(void)i;(void)j;(void)k;}
static void gw_ScriptGame_FighterModExtra(int a,int b,int c,int d,int e){(void)a;(void)b;(void)c;(void)d;(void)e;}
static void gw_ScriptGame_SetPercent(int slot,int bits){(void)slot;(void)bits;}
#include "../platform/gw_script_sim_state.inc"
static char committed_blob[16384];static int committed_size;
int gw_ScriptGame_SimCommit(int token,int size){int i;(void)token;for(i=0;i<size;++i)committed_blob[i]=(char)gw_Script_SimByte(i);committed_size=size;return 1;}
static void gw_test_fail(const char*message,...){fprintf(stderr,"%s\n",message);}
#include "../platform/gw_script_echo_tests.inc"
static int parse_echo(lua_State *L){GsEchoRules r;gs_echo_parse(L,1,&r);assert(gs_echo_apply(0,1,&r));return 0;}
int main(void){assert(!test_script_echo_parser());lua_State *L=luaL_newstate();GsEchoRules saved;
luaL_requiref(L,"_G",luaopen_base,1);lua_pop(L,1);lua_newtable(L);
#define EAPI(name,fn) lua_pushcfunction(L,fn);lua_setfield(L,-2,name)
EAPI("afterimage_add",l_afterimage_add);EAPI("afterimage_remove",l_afterimage_remove);EAPI("echo_add",l_echo_add);EAPI("echo_remove",l_echo_remove);EAPI("fighter_history",l_fighter_history);EAPI("fighter_history_depth",l_fighter_history_depth);EAPI("sim_commit",l_sim_commit);EAPI("echoes",l_echoes);EAPI("echo_afterimage",l_echo_afterimage);EAPI("afterimage_copy",l_afterimage_copy);EAPI("afterimage_copy_set",l_afterimage_copy_set);EAPI("packet",parse_echo);lua_setglobal(L,"gd");
run(L,"assert(gd.fighter_history_depth()==61 and gd.fighter_history(1,60)==nil); assert(not pcall(gd.fighter_history,1,61)); "
"for _,r in ipairs({{delay=0},{delay=61},{delay=1,damage=0/0},{delay=1,damage=1/0},{delay=1,knockback=4.1},{match={move='bad'}},{match={element='fraction'}},{once_per_move=1},{match={airborne=1}},{bogus=true}}) do assert(not pcall(gd.echo_add,1,r)) end; "
"local h=gd.echo_add(1,{delay=60,match={move='nair',airborne=true},damage=.4,knockback=1,once_per_move=false,sub=true});assert(h>0);assert(gd.echo_remove(h)); "
"gd.packet({rules={{handle=40,delay=1,match={move='aerial'},damage=.4},{handle=41,delay=60,element='fire'}}})");
assert(echo_rows[0].rules[0][0]==40&&echo_rows[0].rules[1][1]==60);saved=echo_rows[0];memset(&echo_rows[0],0,sizeof echo_rows[0]);assert(gs_echo_apply(0,1,&saved));assert(!memcmp(&saved,&echo_rows[0],sizeof saved));
{GsEchoRules draft=saved;int next=100;draft.rules[0][0]=0;draft.rules[1][0]=0;gs_echo_prepare(L,0,&draft,&next);assert(draft.rules[0][0]==40&&draft.rules[1][0]==41&&next==100);draft.rules[0][1]=2;draft.rules[0][0]=0;gs_echo_prepare(L,0,&draft,&next);assert(draft.rules[0][0]==100&&next==101);}
gs.cur=1;run(L,"assert(not pcall(gd.echo_add,1,{delay=2}));assert(not gd.echo_remove(40))");
gs.cur=0;gw_ScriptGame_EchoClear(1);
run(L,"for i=1,80 do local e=gd.echo_afterimage(1,{copies=3,spacing=8,presentation_only=true,echoes={}});assert(e);assert(gd.afterimage_remove(e)) end");gs_echo_visual_release(1);
run(L,"local e=gd.afterimage_add(1,{copies=3,spacing=4});assert(gd.afterimage_copy(e,2).age==8);assert(gd.afterimage_copy_set(e,2,{brightness=.3}));assert(gd.afterimage_copy(e,2).brightness<.31)");gs.cur=1;
run(L,"assert(gd.afterimage_copy(77,2)==nil);assert(not pcall(gd.afterimage_copy_set,77,2,{brightness=.7}))");gs.cur=0;
run(L,"assert(gd.afterimage_remove(77));assert(gd.afterimage_copy(77,2)==nil);assert(not pcall(gd.afterimage_copy_set,77,2,{brightness=.7}))");gs_echo_visual_release(1);
run(L,"local e,h=gd.echo_afterimage(1,{copies=3,spacing=8,echoes={{copy=1,match={move='nair'},damage=.4},{copy=2,match={move='nair'},damage=.4}}});assert(e==77 and #h==2); assert(gd.echoes(1).journal and #gd.echoes(1)==2);local c=gd.afterimage_copy(e,2);assert(c.age==16 and c.armed and c.element==1 and c.flash==2);assert(gd.afterimage_copy(e,4)==nil);assert(gd.afterimage_copy_set(e,3,{brightness=.2,tint={.1,.2,.3,.4}}));assert(gd.afterimage_copy(e,3).brightness<.21)");
assert(echo_rows[0].rules[0][1]==8&&echo_rows[0].rules[1][1]==16);gs_echo_visual_release(1);motion_live=0;gw_ScriptGame_EchoClear(1);gate_closed=1;
run(L,"assert(not pcall(gd.echo_afterimage,1,{copies=3,spacing=8,echoes={{copy=1}}}));local e,h=gd.echo_afterimage(1,{copies=3,spacing=8,presentation_only=true,echoes={{copy=1},{copy=2}}});assert(e==77 and gd.afterimage_copy(e,2).echo_handle==0 and #gd.echoes(1)==0);assert(not pcall(gd.echo_add,1,{delay=4}));assert(gd.fighter_history_depth()==61)");
gate_closed=0;history_active=1;run(L,"local h=gd.fighter_history(1,0);assert(#h.hitboxes==1 and h.hitboxes[1].source_index==3 and h.grounded)");history_active=2;run(L,"local h=gd.fighter_history(1,0);assert(h.attack_id==77 and #h.hitboxes==1 and h.hitboxes[1].source_index==4 and h.hitboxes[1].owner_port==2 and not h.hitboxes[1].owner_sub and h.hitboxes[1].source_entity==2 and h.hitboxes[1].original_element==1 and h.hitboxes[1].move_tag==7 and h.hitboxes[1].grounded)");history_active=0;
gs.console=-1;gs.n=1;gs.match_active=1;gs.s[0].used=1;gs.s[0].stage_owner=1;gs_sim_window=1;gs_sim_tag=1;gs_sim_log[1].tag=1;
run(L,"gd.sim_commit('echo-checkpoint',{{op='echoes',port=1,sub=false,rules={{delay=8,match={move='aerial'},damage=.4},{delay=16,match={move='nair'},damage=.5}}}})");
saved=echo_rows[0];assert(saved.count==2&&saved.rules[0][0]>0&&saved.rules[1][0]>saved.rules[0][0]);memset(&echo_rows[0],0,sizeof echo_rows[0]);echo_owner[0]=0;committed_size=0;gs_sim_replay(1);assert(!memcmp(&saved,&echo_rows[0],sizeof saved));assert(committed_size==15&&!memcmp(committed_blob,"echo-checkpoint",15));
printf("actual production gs_sim echo journal replay PASS; op=%u commit=%u extra_echo=%u bytes\n",(unsigned)sizeof(GsSimOp),(unsigned)sizeof(GsSimCommit),(unsigned)sizeof(GsEchoRules));gs_sim_reset();assert(!gs_sim_bytes);
gs_sim_window=1;gs_sim_tag=0;gs_sim_log[0].tag=0;gs_sim_done[0]=0;
run(L,"assert(not pcall(gd.sim_commit,'bad',{{op='echoes',port=1,rules={{handle=9000,delay=4}}},{op='echoes',port=2,rules={{handle=9000,delay=8}}}}))");
assert(!memcmp(&saved,&echo_rows[0],sizeof saved));assert(echo_rows[2].count==0&&!gs_sim_bytes);
run(L,"assert(not pcall(gd.sim_commit,'bad',{{op='echoes',port=1,rules={}},{op='echoes',port=1,rules={}}}))");
assert(!memcmp(&saved,&echo_rows[0],sizeof saved));assert(!gs_sim_bytes);puts("echo duplicate commit preflight / no mutation PASS");
puts("echo production Lua parser / scalar packet deterministic replay PASS");lua_close(L);return 0;}
