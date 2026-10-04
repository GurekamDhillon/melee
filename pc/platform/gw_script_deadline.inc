/* Progress is opt-in: frame presentations cannot prove that staging completed.
 * Deadlines count completed live logic frames, not PADReads, wall time, rewind
 * or paused draws. Host diagnostics deliberately survive savestate loads.
 * Namespace = script slot + its stage-owner generation. Unload cancels it.
 */
#include "gw_hang_policy.h"
#define GS_DEADLINES 128
static struct { int used,owner,generation,reported; unsigned waited,limit,serial; char name[128]; } gs_deadlines[GS_DEADLINES];
static unsigned gs_deadline_serial;
static int gs_deadline_active,gs_deadline_expired_count;
static char gs_deadline_last[128];
static void gs_deadline_publish(void) {
    int i,pending=0,expired=0;
    for(i=0;i<GS_DEADLINES;++i)if(gs_deadlines[i].used) {
        if(gs_deadlines[i].reported)expired++;else pending++;
    }
    gs_deadline_active=pending+expired;gs_deadline_expired_count=expired;
    gw_hang_script_state(pending,expired,gs_deadline_last);
}
static int l_deadline(lua_State *L) {
    const char *name=luaL_checkstring(L,1);lua_Integer n=luaL_checkinteger(L,2);
    int i,slot=-1,generation=gs.s[gs.cur].stage_owner;
    if(!*name || strlen(name)>=sizeof gs_deadlines[0].name || n<1 || n>INT_MAX)
        return luaL_error(L,"gd.deadline: name 1..127 bytes, frames 1..2147483647");
    for(i=0;i<GS_DEADLINES;++i) {
        if(gs_deadlines[i].used && gs_deadlines[i].owner==gs.cur && gs_deadlines[i].generation==generation && !strcmp(gs_deadlines[i].name,name)) {
            /* Repeated calls in staging must not silently extend its deadline. */
            lua_pushboolean(L,0);return 1;
        }
        if(!gs_deadlines[i].used && slot<0)slot=i;
    }
    if(slot<0)return luaL_error(L,"gd.deadline: table full (128 deadlines)");
    memset(&gs_deadlines[slot],0,sizeof gs_deadlines[slot]);
    gs_deadlines[slot].used=1;gs_deadlines[slot].owner=gs.cur;gs_deadlines[slot].generation=generation;
    gs_deadlines[slot].limit=(unsigned)n;gs_deadlines[slot].serial=++gs_deadline_serial;
    snprintf(gs_deadlines[slot].name,sizeof gs_deadlines[slot].name,"%s",name);
    gs_deadline_publish();lua_pushboolean(L,1);return 1;
}
static int l_deadline_done(lua_State *L) {
    const char *name=luaL_checkstring(L,1);int i;
    for(i=0;i<GS_DEADLINES;++i)if(gs_deadlines[i].used && gs_deadlines[i].owner==gs.cur &&
        gs_deadlines[i].generation==gs.s[gs.cur].stage_owner && !strcmp(gs_deadlines[i].name,name)) {
        gs_deadlines[i].used=0;gs_deadline_publish();lua_pushboolean(L,1);return 1;
    }
    lua_pushboolean(L,0);return 1;
}
static void gs_deadline_release(int owner) {
    int i;for(i=0;i<GS_DEADLINES;++i)if(gs_deadlines[i].used && gs_deadlines[i].owner==owner)gs_deadlines[i].used=0;
    gs_deadline_publish();
}
static void gs_deadline_frame(void) {
    int i;unsigned cutoff=gs_deadline_serial;
    if(!gs_deadline_active)return;
    for(i=0;i<GS_DEADLINES;++i)if(gs_deadlines[i].used && !gs_deadlines[i].reported && gs_deadlines[i].serial<=cutoff) {
        int owner=gs_deadlines[i].owner;
        if(!gs.s[owner].used || gs.s[owner].stage_owner!=gs_deadlines[i].generation){gs_deadlines[i].used=0;continue;}
        if(gw_deadline_expired(++gs_deadlines[i].waited,gs_deadlines[i].limit,gs_deadlines[i].reported)) {
            char name[128];unsigned waited=gs_deadlines[i].waited;
            snprintf(name,sizeof name,"%s",gs_deadlines[i].name);
            gs_deadlines[i].reported=1;
            snprintf(gs_deadline_last,sizeof gs_deadline_last,"%s",name);
            gw_log("script [%s] deadline expired name=%s frames_waited=%u",gs_script_id(owner),name,waited);
            /* Publish before invoking user code: callback may stall/cancel/re-arm. */
            gs_deadline_publish();
            if(gs_get_hook(owner,"on_deadline")) {
                lua_pushstring(gs.L,name);lua_pushinteger(gs.L,waited);
                gs_pcall(owner,2,0,"on_deadline");
            }
        }
    }
    gs_deadline_publish();
}

/* Errors and explicit API false/nil + reason returns, per callback, even when
 * the script catches the refusal itself. A plain false predicate is not a
 * refusal. Disabled scripts already get a terminal error summary.
 */
static struct { int used,owner; char callback[64]; double first,last,report; unsigned count; } gs_callback_fail[128];
static int gs_watch_api_result(lua_State *L,int n) {
    if(n>=2 && (lua_isnil(L,-n) || (lua_isboolean(L,-n) && !lua_toboolean(L,-n))) && lua_type(L,-n+1)==LUA_TSTRING)
        gs_callback_refused=1;
    return n;
}
static void gs_callback_result(int owner,const char *what,int failed) {
    int i,slot=-1;double now=gs_now_ms(),limit=10000;
    const char *v=getenv("MELEE_SCRIPT_WATCHDOG_SECS");
    if(v){double n=strtod(v,NULL);if(n>0 && n<=86400)limit=n*1000;}
    for(i=0;i<128;++i) {
        if(gs_callback_fail[i].used && gs_callback_fail[i].owner==owner && !strcmp(gs_callback_fail[i].callback,what)){slot=i;break;}
        if(!gs_callback_fail[i].used && slot<0)slot=i;
    }
    if(slot<0)return;
    if(!failed){gs_callback_fail[slot].used=0;return;}
    if(!gs_callback_fail[slot].used || now-gs_callback_fail[slot].last>2000) {
        memset(&gs_callback_fail[slot],0,sizeof gs_callback_fail[slot]);
        gs_callback_fail[slot].used=1;gs_callback_fail[slot].owner=owner;gs_callback_fail[slot].first=gs_callback_fail[slot].report=now;
        snprintf(gs_callback_fail[slot].callback,sizeof gs_callback_fail[slot].callback,"%s",what);
    }
    gs_callback_fail[slot].last=now;gs_callback_fail[slot].count++;
    if(now-gs_callback_fail[slot].first>=limit && now-gs_callback_fail[slot].report>=limit) {
        gw_log("script [%s] watchdog: callback=%s failing/refusing continuously %.1fs calls=%u",gs_script_id(owner),what,
            (now-gs_callback_fail[slot].first)/1000,gs_callback_fail[slot].count);
        gs_callback_fail[slot].report=now;
    }
}
static void gs_callback_release(int owner) {
    int i;for(i=0;i<128;++i)if(gs_callback_fail[i].owner==owner)gs_callback_fail[i].used=0;
}
