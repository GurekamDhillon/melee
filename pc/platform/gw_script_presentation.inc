/* Timed offline logic freeze and owned post lifetimes. Kept separate from manual
 * pause so expiry/unload can never resume a user's pause or another script. */
#include "gw_presentation_state.h"
#include "gw_shader.h"
static GwPresentationTimer gs_hitstop[GS_MAX_SCRIPTS+1];
static int gs_presentation_active; /* bit 1 freeze, bit 2 owned post deadlines */
static struct { int handle,clock; GwPresentationTimer timer; } gs_post_timers[32];
static int gs_presentation_owner(void) { GsScript* s=gs_cur_script();return s?s->stage_owner:0; }
static int l_hitstop(lua_State* L)
{
    lua_Integer n=luaL_checkinteger(L,1);int i,owner,slot=-1;
    gs_require_offline(L,"hitstop");
    if(n<1 || n>36000)return luaL_error(L,"gd.hitstop: frames must be 1..36000");
    if(!gs.match_active){lua_pushboolean(L,0);return 1;}
    owner=gs_presentation_owner();
    for(i=0;i<GS_MAX_SCRIPTS+1;++i) {
        if(gs_hitstop[i].used && gs_hitstop[i].owner==owner){slot=i;break;}
        if(!gs_hitstop[i].used && slot<0)slot=i;
    }
    if(slot>=0) {
        gw_presentation_start(&gs_hitstop[slot],owner,gs_now_ms(),(int)n);
        gs_presentation_active|=1;
        lua_pushboolean(L,1);return 1;
    }
    lua_pushboolean(L,0);return 1;
}
static int l_hitstop_cancel(lua_State* L)
{
    int i,changed=0;gs_require_offline(L,"hitstop_cancel");
    for(i=0;i<GS_MAX_SCRIPTS+1;++i)changed|=gw_presentation_cancel(&gs_hitstop[i],gs_presentation_owner());
    lua_pushboolean(L,changed);return 1;
}
static void gs_post_forget(int owner,int handle)
{
    int i;for(i=0;i<32;++i)if(gs_post_timers[i].timer.used && gs_post_timers[i].timer.owner==owner &&
        (!handle || gs_post_timers[i].handle==handle))gs_post_timers[i].timer.used=0;
}
static void gs_presentation_release(int owner)
{
    int i;for(i=0;i<GS_MAX_SCRIPTS+1;++i)gw_presentation_cancel(&gs_hitstop[i],owner);
    gs_post_forget(owner,0);
}
static void gs_presentation_reset(void)
{ memset(gs_hitstop,0,sizeof gs_hitstop);memset(gs_post_timers,0,sizeof gs_post_timers);gs_presentation_active=0; }
static int gs_hitstop_live(void)
{
    int i;double now;
    if(!(gs_presentation_active&1))return 0;
    now=gs_now_ms();
    if(gw_RB_Enabled() || gw_Netplay_Enabled() || gw_Snap_Resimulating())return 0;
    for(i=0;i<GS_MAX_SCRIPTS+1;++i)if(gw_presentation_live(&gs_hitstop[i],now))return 1;
    return 0;
}
static int gs_post_duration(lua_State* L,int opts)
{
    lua_Integer n=0;
    if(lua_istable(L,opts)) {lua_getfield(L,opts,"duration_frames");
        if(!lua_isnil(L,-1))n=luaL_checkinteger(L,-1);lua_pop(L,1);}
    if(n<0 || n>36000)luaL_error(L,"post duration_frames must be 1..36000 or omitted/0");
    return (int)n;
}
static int gs_post_timer_add(int owner,int handle,int frames,int clock)
{
    int i;char error[2048];
    if(!frames)return !clock;
    if(clock && !gw_Post_Set(owner,handle,"{\"elapsed\":0,\"progress\":0}",error,sizeof error))return 0;
    for(i=0;i<32;++i)if(!gs_post_timers[i].timer.used) {
        gs_post_timers[i].handle=handle;gs_post_timers[i].clock=clock;
        gw_presentation_start(&gs_post_timers[i].timer,owner,gs_now_ms(),frames);
        gs_presentation_active|=2;return 1;
    }
    return 0;
}
static void gs_presentation_tick(void)
{
    double now;int i,active=0;
    if(!gs_presentation_active)return;
    now=gs_now_ms();
    if(!gs.match_active || gw_RB_Enabled() || gw_Netplay_Enabled())memset(gs_hitstop,0,sizeof gs_hitstop);
    for(i=0;i<GS_MAX_SCRIPTS+1;++i) {
        if(!gw_presentation_live(&gs_hitstop[i],now))gs_hitstop[i].used=0;
        else active|=1;
    }
    for(i=0;i<32;++i)if(gs_post_timers[i].timer.used) {
        GwPresentationTimer* t=&gs_post_timers[i].timer;int h=gs_post_timers[i].handle;
        if(!gw_presentation_live(t,now)) {gw_Post_Remove(t->owner,h);t->used=0;}
        else if(gs_post_timers[i].clock) {
            char params[128],error[2048];
            snprintf(params,sizeof params,"{\"elapsed\":%.9g,\"progress\":%.9g}",
                (now-t->start)/1000.0,gw_presentation_progress(t,now));
            if(!gw_Post_Set(t->owner,h,params,error,sizeof error)) {gw_Post_Remove(t->owner,h);t->used=0;}
        }
        if(t->used)active|=2;
    }
    gs_presentation_active=active;
}
