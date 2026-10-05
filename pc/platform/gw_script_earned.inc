/* Earned afterimages and tracers (included right after gw_script_motion.inc).
 *
 * Owner's hard rule: no always-on afterimages. Every afterimage therefore has a start and an end, and the engine
 * enforces it: afterimages default to the "flag" trigger (off until something raises the flag), trigger "always" or
 * "moving" is refused unless the options say debug=true, and the two calls below bind the flag to something that ends
 * by itself:
 *
 *   gd.afterimage_bind(handle, {status=1..4, tint=, tail=})   on while that timed-status channel (gd.fighter_timed_status)
 *                                                             of the emitter's fighter has frames left, in the colour the
 *                                                             binding names. The status lives in snapshotted game state, so a
 *                                                             rewind restores exactly when the afterimage is on.
 *   gd.afterimage_window(handle, frames [, {tint=, tail=}])   on for `frames` live logic frames, then off by itself. A
 *                                                             presentation window: it is not rewound, use a status for
 *                                                             anything a rewind must reproduce.
 *   gd.afterimage_unbind(handle)                              drop the binding and lower the flag.
 *
 * The same three exist for tracers as gd.tracer_bind / tracer_window / tracer_unbind: a tracer with a flag trigger is
 * shown only while its status or window is on ("this hit carries something"); bind it to a status granted on on_crit or
 * on a hit tag, or open a window of the hitbox's length. Nothing here writes the simulation. */
extern int gw_ScriptGame_CapsStatusRead(int entity,int channel,int field);
static float gs_caps_number(lua_State* L,int at,double low,double high); /* gw_script_fighter_caps.inc */
#define GS_EARNED_MAX 32
typedef struct { unsigned owner; int handle, kind, entity, channel, window, on, retint; float tint[4], tail[4]; } GsEarned;
static GsEarned gs_earned[GS_EARNED_MAX];
static GsEarned* gs_earned_find(unsigned owner,int handle,int create)
{
    int i;GsEarned* free_slot=NULL;
    for(i=0;i<GS_EARNED_MAX;++i){
        if(gs_earned[i].owner==owner&&gs_earned[i].handle==handle) return &gs_earned[i];
        if(!gs_earned[i].owner&&!free_slot) free_slot=&gs_earned[i];
    }
    if(create&&free_slot){memset(free_slot,0,sizeof *free_slot);free_slot->owner=owner;free_slot->handle=handle;return free_slot;}
    return NULL;
}
static int gs_earned_entity(const GwMotionOptions* o){return o->port+(o->sub?6:0);}
/* Raise or lower the flag, applying the cause's colour when it rises. Returns false if the emitter is gone. */
static int gs_earned_apply(GsEarned* b,int on)
{
    GwMotionOptions o;const char* error=NULL;
    if(!gw_motion_get(b->owner,b->handle,&o)) return 0;
    if(!!o.flag==!!on&&!(on&&b->retint)) return 1;
    o.flag=on?1:0;o.trigger=2;
    if(on&&b->retint){memcpy(o.tint,b->tint,sizeof o.tint);memcpy(o.tail,b->tail,sizeof o.tail);}
    b->retint=0;
    gw_log("earned: %s handle %d %s (%s)",o.kind==GW_MOTION_AFTERIMAGE?"afterimage":"tracer",b->handle,on?"on":"off",b->channel>0?"status":"window");
    return gw_motion_set(b->owner,b->handle,&o,&error);
}
static void gs_earned_frame(void)
{
    int i;
    for(i=0;i<GS_EARNED_MAX;++i){
        GsEarned* b=&gs_earned[i];int on=0;
        if(!b->owner) continue;
        if(b->channel>0) on=gw_ScriptGame_CapsStatusRead(b->entity,b->channel,0)>0;
        if(b->window>0){on=1;--b->window;}
        if(!gs_earned_apply(b,on)){memset(b,0,sizeof *b);continue;}
        b->on=on;
        if(!on&&b->channel<=0&&b->window<=0) memset(b,0,sizeof *b); /* a finished window frees its slot */
    }
}
static void gs_earned_release(unsigned owner)
{
    int i;for(i=0;i<GS_EARNED_MAX;++i) if(!owner||gs_earned[i].owner==owner) memset(&gs_earned[i],0,sizeof gs_earned[i]);
}
static void gs_earned_colours(lua_State* L,int at,GsEarned* b)
{
    if(lua_isnoneornil(L,at)) return;
    luaL_checktype(L,at,LUA_TTABLE);
    lua_getfield(L,at,"tint");
    if(!lua_isnil(L,-1)){int i;luaL_checktype(L,-1,LUA_TTABLE);if(lua_rawlen(L,-1)!=4)luaL_error(L,"tint needs 4 components");
        for(i=0;i<4;++i){lua_rawgeti(L,-1,i+1);b->tint[i]=gs_caps_number(L,-1,0,1);lua_pop(L,1);}b->retint=1;
        if(memcmp(b->tail,(float[4]){0,0,0,0},16)==0){b->tail[0]=b->tint[0]*0.4f;b->tail[1]=b->tint[1]*0.4f;b->tail[2]=b->tint[2]*0.4f;b->tail[3]=0;}}
    lua_pop(L,1);
    lua_getfield(L,at,"tail");
    if(!lua_isnil(L,-1)){int i;luaL_checktype(L,-1,LUA_TTABLE);if(lua_rawlen(L,-1)!=4)luaL_error(L,"tail needs 4 components");
        for(i=0;i<4;++i){lua_rawgeti(L,-1,i+1);b->tail[i]=gs_caps_number(L,-1,0,1);lua_pop(L,1);}b->retint=1;}
    lua_pop(L,1);
}
static int gs_earned_bind(lua_State* L,int kind,int window)
{
    static const char* const keys[]={"status","tint","tail",NULL};
    GwMotionOptions o;int h=gs_motion_handle(L,kind,&o);GsEarned* b;int status=0,frames=0;
    if(window){lua_Integer n=luaL_checkinteger(L,2);if(n<1||n>3600)return luaL_error(L,"window frames must be 1-3600");frames=(int)n;}
    else{luaL_checktype(L,2,LUA_TTABLE);gs_item_keys(L,2,keys);lua_getfield(L,2,"status");
        {lua_Integer n=luaL_checkinteger(L,-1);if(n<1||n>4)return luaL_error(L,"status channel must be 1-4");status=(int)n;}lua_pop(L,1);}
    b=gs_earned_find((unsigned)gs.cur+1,h,1);
    if(!b) return luaL_error(L,"earned binding capacity (32) exhausted");
    b->kind=kind;b->entity=gs_earned_entity(&o);
    if(window) b->window=frames; else b->channel=status;
    if(window&&!lua_isnoneornil(L,3)){static const char* const wkeys[]={"tint","tail",NULL};luaL_checktype(L,3,LUA_TTABLE);gs_item_keys(L,3,wkeys);}
    gs_earned_colours(L,window?3:2,b);
    gs_earned_apply(b,window?1:gw_ScriptGame_CapsStatusRead(b->entity,b->channel,0)>0);
    lua_pushboolean(L,1);return 1;
}
static int gs_earned_unbind(lua_State* L,int kind)
{
    GwMotionOptions o;int h=gs_motion_handle(L,kind,&o);GsEarned* b=gs_earned_find((unsigned)gs.cur+1,h,0);
    if(b){gs_earned_apply(b,0);memset(b,0,sizeof *b);}
    lua_pushboolean(L,1);return 1;
}
static int l_afterimage_bind(lua_State* L){return gs_earned_bind(L,GW_MOTION_AFTERIMAGE,0);}
static int l_afterimage_window(lua_State* L){return gs_earned_bind(L,GW_MOTION_AFTERIMAGE,1);}
static int l_afterimage_unbind(lua_State* L){return gs_earned_unbind(L,GW_MOTION_AFTERIMAGE);}
static int l_tracer_bind(lua_State* L){return gs_earned_bind(L,GW_MOTION_TRACER,0);}
static int l_tracer_window(lua_State* L){return gs_earned_bind(L,GW_MOTION_TRACER,1);}
static int l_tracer_unbind(lua_State* L){return gs_earned_unbind(L,GW_MOTION_TRACER);}
