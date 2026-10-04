/* Real Lua 5.4 exercising production presentation functions with deterministic
 * host time and a small post resource fixture. No game build or launch. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "../third_party/lua-5.4.7/src/lua.h"
#include "../third_party/lua-5.4.7/src/lauxlib.h"
#include "../third_party/lua-5.4.7/src/lualib.h"
#include "../platform/gw_shader.h"
#define GS_MAX_SCRIPTS 4
typedef struct {int stage_owner;} GsScript;
static GsScript script={7};
static struct {int match_active,paused;} gs={1,0};
static double now;static int online,rollback,resim,gameplay=1,removed,updates;
static double elapsed,progress;static int post_alive=1;
static GsScript* gs_cur_script(void) {return &script;}
static double gs_now_ms(void) {return now;}
static int gw_RB_Enabled(void) {return rollback;}
static int gw_Netplay_Enabled(void) {return online;}
static int gw_Snap_Resimulating(void) {return resim;}
static void gs_require_offline(lua_State* L,const char* fn)
{if(online || rollback || !gameplay)luaL_error(L,"%s refused",fn);}
int gw_Post_Set(int owner,int handle,const char* json,char* error,int cap)
{
    (void)error;(void)cap;
    if(owner!=7 || handle!=3 || !post_alive)return 0;
    assert(sscanf(json,"{\"elapsed\":%lf,\"progress\":%lf}",&elapsed,&progress)==2);
    ++updates;return 1;
}
int gw_Post_Remove(int owner,int handle) {assert(owner==7 && handle==3);post_alive=0;++removed;return 1;}
#include "../platform/gw_script_presentation.inc"
static int exec(lua_State* L,const char* text) {int r=luaL_dostring(L,text);if(r)lua_pop(L,1);return r;}
int main(void) {
 lua_State* L=luaL_newstate();luaL_requiref(L,"_G",luaopen_base,1);lua_pop(L,1);
 lua_pushcfunction(L,l_hitstop);lua_setglobal(L,"hitstop");
 lua_pushcfunction(L,l_hitstop_cancel);lua_setglobal(L,"cancel");
 now=100;assert(exec(L,"assert(hitstop(71))")==0);assert(gs_hitstop_live());
 gs.paused=1;now=1284;gs_presentation_tick();assert(!gs_hitstop_live() && gs.paused==1);
 now=2000;assert(exec(L,"assert(hitstop(60))")==0);
 script.stage_owner=8;assert(exec(L,"assert(hitstop(120))")==0);assert(exec(L,"assert(cancel())")==0);
 assert(gs_hitstop_live());
 /* A freed earlier slot must not create a second request for the same owner. */
 script.stage_owner=8;assert(exec(L,"assert(hitstop(120))")==0);
 script.stage_owner=7;assert(exec(L,"assert(cancel())")==0);
 script.stage_owner=8;assert(exec(L,"assert(hitstop(1))")==0);
 now+=20;gs_presentation_tick();assert(!gs_hitstop_live());
 now=2000;script.stage_owner=7;assert(exec(L,"assert(hitstop(60))")==0);
 assert(gs_hitstop_live());script.stage_owner=7;assert(exec(L,"assert(cancel())")==0);assert(!gs_hitstop_live());
 online=1;assert(exec(L,"hitstop(1)")!=0);online=0;rollback=1;assert(exec(L,"hitstop(1)")!=0);rollback=0;
 gameplay=0;assert(exec(L,"hitstop(1)")!=0);gameplay=1;
 assert(exec(L,"hitstop(0)")!=0 && exec(L,"hitstop(36001)")!=0);
 gs.match_active=0;assert(exec(L,"assert(not hitstop(1))")==0);gs.match_active=1;
 now=3000;assert(gs_post_timer_add(7,3,60,1));now=3500;gs_presentation_tick();
 assert(updates==2 && elapsed==0.5 && progress==0.5 && !removed);
 now=4000;gs_presentation_tick();assert(removed==1 && !post_alive);
 post_alive=1;assert(gs_post_timer_add(7,3,60,0));gs_presentation_release(8);assert(gs_post_timers[0].timer.used);
 gs_post_forget(7,3);assert(!gs_post_timers[0].timer.used);
 assert(exec(L,"assert(hitstop(60))")==0);gs.match_active=0;gs_presentation_tick();assert(!gs_hitstop_live());
 gs_presentation_reset();lua_close(L);puts("Lua hitstop/ownership/post clock: PASS");return 0;
}
