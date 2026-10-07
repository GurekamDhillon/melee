/* Owner-scoped visual APIs. Includes native Lua validation only; never calls
 * gameplay permission gates, gs_rw_branch, or a game-state mutator. */
#include "gw_motion.h"
static void gs_item_keys(lua_State*,int,const char* const*);
static const char* gs_motion_text(lua_State* L,int index)
{
    size_t n;const char* text=luaL_checklstring(L,index,&n);
    if(strlen(text)!=n)luaL_error(L,"malformed motion name");return text;
}
static int gs_motion_enum(lua_State* L,int table,const char* key,const char* const* names,int count,int current)
{
    int n;lua_getfield(L,table,key);
    if(lua_isnil(L,-1)){lua_pop(L,1);return current;}
    const char* text=gs_motion_text(L,-1);
    for(n=0;n<count;++n)if(!strcmp(text,names[n])){lua_pop(L,1);return n;}
    return luaL_error(L,"invalid motion %s",key);
}
static void gs_motion_number(lua_State* L,int table,const char* key,float* value,float lo,float hi)
{
    lua_getfield(L,table,key);
    if(!lua_isnil(L,-1)){double n=luaL_checknumber(L,-1);if(!isfinite(n)||n<lo||n>hi)luaL_error(L,"motion %s out of range",key);*value=(float)n;}
    lua_pop(L,1);
}
static void gs_motion_integer(lua_State* L,int table,const char* key,int* value,int lo,int hi)
{
    lua_getfield(L,table,key);
    if(!lua_isnil(L,-1)){lua_Integer n=luaL_checkinteger(L,-1);if(n<lo||n>hi)luaL_error(L,"motion %s out of range",key);*value=(int)n;}
    lua_pop(L,1);
}
static void gs_motion_bool(lua_State* L,int table,const char* key,int* value)
{
    lua_getfield(L,table,key);if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);*value=lua_toboolean(L,-1);}lua_pop(L,1);
}
static void gs_motion_array(lua_State* L,int table,const char* key,float* value,int count,float lo,float hi)
{
    int i;lua_getfield(L,table,key);
    if(!lua_isnil(L,-1)){
        luaL_checktype(L,-1,LUA_TTABLE);if(lua_rawlen(L,-1)!=(size_t)count)luaL_error(L,"motion %s needs %d floats",key,count);
        for(i=0;i<count;++i){lua_rawgeti(L,-1,i+1);double n=luaL_checknumber(L,-1);if(!isfinite(n)||n<lo||n>hi)luaL_error(L,"invalid motion %s",key);value[i]=(float)n;lua_pop(L,1);}
    }lua_pop(L,1);
}
static int gs_motion_joint(lua_State* L,int index)
{
    static const char* names[]={"left_hand","right_hand","left_foot","right_foot","head"};
    int i;if(lua_type(L,index)==LUA_TNUMBER){lua_Integer n=luaL_checkinteger(L,index);if(n<0||n>254)luaL_error(L,"joint index 0..254");return (int)n;}
    const char* name=gs_motion_text(L,index);
    for(i=0;i<5;++i)if(!strcmp(name,names[i]))return -i-1;
    return luaL_error(L,"unknown joint alias; use an explicit joint index for tail or custom skeletons");
}
static void gs_motion_options(lua_State* L,int table,GwMotionOptions* o)
{
    static const char* keys[]={"port","sub","anchor","copies","spacing","lifetime","length","smoothing","blend","trigger","flag","follow","clear_on_respawn","surface","shader","depth","speed","scale","fade","width","taper","intensity","params","offset","tint","tail","edge","debug",NULL};
    static const char* blends[]={"alpha","additive"},*triggers[]={"always","moving","flag"},*surfaces[]={"own","silhouette","gradient"};
    static const char* shaders[]={"solid","glow","fire","electric","frost","dark"};
    luaL_checktype(L,table,LUA_TTABLE);gs_item_keys(L,table,keys);
    gs_motion_integer(L,table,"port",&o->port,1,6);gs_motion_bool(L,table,"sub",&o->sub);
    gs_motion_integer(L,table,"copies",&o->copies,1,GW_MOTION_MAX_COPIES);gs_motion_integer(L,table,"spacing",&o->spacing,1,8);
    gs_motion_integer(L,table,"lifetime",&o->lifetime,2,GW_MOTION_MAX_AGE);gs_motion_integer(L,table,"length",&o->length,2,GW_MOTION_MAX_AGE);
    gs_motion_integer(L,table,"smoothing",&o->smoothing,1,8);
    o->blend=gs_motion_enum(L,table,"blend",blends,2,o->blend);o->trigger=gs_motion_enum(L,table,"trigger",triggers,3,o->trigger);
    o->surface=gs_motion_enum(L,table,"surface",surfaces,3,o->surface);o->shader=gs_motion_enum(L,table,"shader",shaders,6,o->shader);
    gs_motion_bool(L,table,"flag",&o->flag);gs_motion_bool(L,table,"follow",&o->follow);
    {   /* no always-on afterimages: a continuous trigger is a labelled debug view, never a default */
        int debug=0;gs_motion_bool(L,table,"debug",&debug);lua_getfield(L,table,"trigger");
        if(!lua_isnil(L,-1)&&o->kind==GW_MOTION_AFTERIMAGE&&o->trigger!=2&&!debug)
            luaL_error(L,"afterimages must be earned: use trigger=\"flag\" with gd.afterimage_bind/gd.afterimage_window (trigger \"always\"/\"moving\" needs debug=true, a debug view)");
        lua_pop(L,1);
    }
    gs_motion_bool(L,table,"clear_on_respawn",&o->clear_on_respawn);gs_motion_bool(L,table,"depth",&o->depth);
    gs_motion_number(L,table,"speed",&o->speed,0,100);gs_motion_number(L,table,"scale",&o->scale,0.25f,2);
    gs_motion_number(L,table,"fade",&o->curve,0.25f,4);gs_motion_number(L,table,"width",&o->width,0.05f,24);
    gs_motion_number(L,table,"taper",&o->taper,0,4);gs_motion_number(L,table,"intensity",&o->intensity,0,1);
    gs_motion_array(L,table,"tint",o->tint,4,0,1);gs_motion_array(L,table,"tail",o->tail,4,0,1);gs_motion_array(L,table,"edge",o->edge,4,0,1);
    gs_motion_array(L,table,"params",o->params,4,0,10);gs_motion_array(L,table,"offset",o->offset,3,-100,100);
    lua_getfield(L,table,"anchor");
    if(!lua_isnil(L,-1)){
        if(lua_type(L,-1)==LUA_TTABLE){
            static const char* akeys[]={"joint","hitbox","item",NULL};int count=0;
            gs_item_keys(L,lua_gettop(L),akeys);
            lua_getfield(L,-1,"joint");if(!lua_isnil(L,-1)){o->anchor=GW_ANCHOR_JOINT;o->index=gs_motion_joint(L,-1);++count;}lua_pop(L,1);
            lua_getfield(L,-1,"hitbox");if(!lua_isnil(L,-1)){lua_Integer n=luaL_checkinteger(L,-1);if(n<0||n>4)luaL_error(L,"hitbox 0..4");o->anchor=GW_ANCHOR_HITBOX;o->index=(int)n;++count;}lua_pop(L,1);
            lua_getfield(L,-1,"item");if(!lua_isnil(L,-1)){lua_Integer id=luaL_checkinteger(L,-1);if(id<=0||id>INT_MAX)luaL_error(L,"invalid item serial");o->anchor=GW_ANCHOR_ITEM;o->item=(int)id;++count;}lua_pop(L,1);
            if(count!=1)luaL_error(L,"anchor table needs exactly one of joint, hitbox, item");
        }else{
            const char* a=lua_type(L,-1)==LUA_TSTRING?gs_motion_text(L,-1):NULL;
            if(a&&!strcmp(a,"held_item"))o->anchor=GW_ANCHOR_HELD;
            else if(a&&!strcmp(a,"sword_tip"))o->anchor=GW_ANCHOR_SWORD;
            else if(a&&!strcmp(a,"active_hitboxes"))o->anchor=GW_ANCHOR_HITS;
            else {o->anchor=GW_ANCHOR_JOINT;o->index=gs_motion_joint(L,-1);}
        }
    }lua_pop(L,1);
}
static int gs_motion_result(lua_State* L,int result,const char* error)
{
    if(result){lua_pushinteger(L,result);return 1;}lua_pushnil(L);lua_pushstring(L,error?error:"motion failed");return 2;
}
static int l_afterimage_add(lua_State* L)
{
    GwMotionOptions o;const char* error=NULL;gw_motion_defaults(&o,GW_MOTION_AFTERIMAGE);
    lua_Integer port=luaL_checkinteger(L,1);if(port<1||port>6)return luaL_argerror(L,1,"port 1..6");o.port=(int)port;
    gs_motion_options(L,2,&o);if(o.port!=port)return luaL_error(L,"options port differs from argument");
    int h=gw_motion_add((unsigned)gs.cur+1,&o,&error);return gs_motion_result(L,h,error);
}
static int l_tracer_add(lua_State* L)
{
    GwMotionOptions o;const char* error=NULL;gw_motion_defaults(&o,GW_MOTION_TRACER);gs_motion_options(L,1,&o);
    int h=gw_motion_add((unsigned)gs.cur+1,&o,&error);return gs_motion_result(L,h,error);
}
static int l_tracer_hitboxes(lua_State* L)
{
    GwMotionOptions o;const char* error=NULL;gw_motion_defaults(&o,GW_MOTION_TRACER);
    lua_Integer p=luaL_checkinteger(L,1);if(p<1||p>6)return luaL_argerror(L,1,"port 1..6");o.port=(int)p;
    if(!lua_isnoneornil(L,2))gs_motion_options(L,2,&o);
    if(o.port!=p)return luaL_error(L,"options port differs from argument");o.anchor=GW_ANCHOR_HITS;
    int h=gw_motion_add((unsigned)gs.cur+1,&o,&error);return gs_motion_result(L,h,error);
}
static int gs_motion_handle(lua_State* L,int kind,GwMotionOptions* o)
{
    lua_Integer h=luaL_checkinteger(L,1);
    if(h<1||h>INT_MAX||!gw_motion_get((unsigned)gs.cur+1,(int)h,o)||o->kind!=kind)luaL_error(L,"stale or foreign motion handle");return (int)h;
}
static int gs_motion_set(lua_State* L,int kind)
{
    GwMotionOptions o;const char* error=NULL;int h=gs_motion_handle(L,kind,&o);gs_motion_options(L,2,&o);
    int ok=gw_motion_set((unsigned)gs.cur+1,h,&o,&error);if(!ok)return gs_motion_result(L,0,error);lua_pushboolean(L,1);return 1;
}
static int l_afterimage_set(lua_State* L){return gs_motion_set(L,GW_MOTION_AFTERIMAGE);}
static int l_tracer_set(lua_State* L){return gs_motion_set(L,GW_MOTION_TRACER);}
static int gs_motion_remove(lua_State* L,int kind){GwMotionOptions o;int h=gs_motion_handle(L,kind,&o);lua_pushboolean(L,gw_motion_remove((unsigned)gs.cur+1,h));return 1;}
static int l_afterimage_remove(lua_State* L){return gs_motion_remove(L,GW_MOTION_AFTERIMAGE);}
static int l_tracer_remove(lua_State* L){return gs_motion_remove(L,GW_MOTION_TRACER);}
static int l_motion_intensity(lua_State* L){float value=(float)luaL_checknumber(L,1);if(!isfinite(value)||value<0||value>1)return luaL_argerror(L,1,"intensity 0..1");gw_motion_intensity(value);return 0;}
static int l_motion_stats(lua_State* L){
    uint64_t stats[GW_MOTION_STATS_COUNT];int k;gw_motion_stats(stats);lua_newtable(L);for(k=0;k<GW_MOTION_STATS_COUNT;++k)gs_setint(L,gw_motion_stat_name((unsigned)k),(lua_Integer)stats[k]);return 1;
}
