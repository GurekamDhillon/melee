/* Scalar fighter capabilities boundary. The owning script, entity bindings and
 * effect clocks live in game MEM1, not this native Lua wrapper. */
extern int gw_ScriptGame_CapsMovement(int entity,int owner,int jumps,int flags);
extern int gw_ScriptGame_CapsMovementRead(int entity,int field);
extern void gw_ScriptGame_CapsMovementRelease(int owner);
extern int gw_ScriptGame_CapsEffect(int entity,int owner,int kind,int value,int frames);
extern int gw_ScriptGame_CapsEffectRead(int entity,int kind,int field);
extern void gw_ScriptGame_CapsEffectsRelease(int owner);
extern void gw_ScriptGame_CritRelease(int owner);
extern void gw_ScriptGame_ShockRelease(int owner);
extern int gw_ScriptGame_CritSet(int entity,int owner,int slot,int chance,int mult,int mult_max,int launch,int min_percent,int force,int flags);
extern int gw_ScriptGame_CapsArmour(int entity,int owner,int damage,int knockback);
extern int gw_ScriptGame_CapsArmourRead(int entity,int field);
extern int gw_ScriptGame_CapsGiveItem(int entity,int owner,int kind);
extern const char* gw_ScriptGame_CapsGiveItemReason(int reason);
extern int gw_ScriptGame_CapsOpponent(int entity,int radius,int nth,int exclude);
extern int gw_ScriptGame_CapsStatus(int entity,int channel,int owner,int value,int frames);
extern int gw_ScriptGame_CapsStatusRead(int entity,int channel,int field);
extern void gw_ScriptGame_CapsStatusRelease(int owner);
extern int gw_ScriptGame_IntrWinSet(int entity,int owner,int frames,int exits,int flags);
extern int gw_ScriptGame_IntrWinRead(int entity,int field);
extern void gw_ScriptGame_IntrWinRelease(int owner);

static int gs_caps_entity(lua_State* L,int at)
{
    lua_Integer n=luaL_checkinteger(L,at);
    if(n<1 || n>12) luaL_error(L,"fighter entity must be 1-12 (1-6 primary, 7-12 secondary)");
    return (int)n;
}
static int gs_caps_frames(lua_State* L,int at)
{
    lua_Integer n=luaL_checkinteger(L,at);
    if(n<0 || n>3600) luaL_error(L,"fighter duration must be 0-3600 logic frames");
    return (int)n;
}
static float gs_caps_number(lua_State* L,int at,double low,double high)
{
    double n=luaL_checknumber(L,at);
    if(!isfinite(n) || n<low || n>high) luaL_error(L,"fighter value must be finite and in range");
    return (float)n;
}
static void gs_caps_write(lua_State* L,const char* name)
{
    gs_require_offline(L,name);
    if(!gs.match_active) luaL_error(L,"fighter capability requires an active match");
}
static int l_fighter_caps(lua_State* L)
{
    static const char* keys[]={"shield","air_dodge","run","grab","specials"};
    int e=gs_caps_entity(L,1),jumps=-1,flags=0,i,retail;
    if(lua_gettop(L)==1) {
        if(!gw_ScriptGame_CapsMovementRead(e,0)) lua_pushnil(L);
        else {
            lua_createtable(L,0,6);
            gs_setint(L,"air_jumps",gw_ScriptGame_CapsMovementRead(e,1));
            flags=gw_ScriptGame_CapsMovementRead(e,2);
            for(i=0;i<5;++i) { lua_pushboolean(L,(flags&(1<<i))!=0); lua_setfield(L,-2,keys[i]); }
        }
        retail=gw_ScriptGame_CapsMovementRead(e,3);
        if(retail<0) lua_pushnil(L); else lua_pushinteger(L,retail);
        return 2;
    }
    gs_caps_write(L,"fighter_caps");
    if(!lua_isnil(L,2)) {
        luaL_checktype(L,2,LUA_TTABLE); lua_pushnil(L);
        while(lua_next(L,2)) {
            const char* key=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
            if(key && !strcmp(key,"air_jumps")) {
                lua_Integer n=luaL_checkinteger(L,-1);
                if(n<0 || n>8) return luaL_error(L,"air_jumps must be 0-8");
                jumps=(int)n;
            } else {
                for(i=0;i<5;++i) if(key && !strcmp(key,keys[i])) break;
                if(i==5) return luaL_error(L,"unknown fighter capability");
                luaL_checktype(L,-1,LUA_TBOOLEAN);
                if(lua_toboolean(L,-1)) flags|=1<<i;
            }
            lua_pop(L,1);
        }
    }
    gs_rw_branch();
    lua_pushboolean(L,gw_ScriptGame_CapsMovement(e,gs.cur+1,jumps,flags));
    return 1;
}
static int l_fighter_effect(lua_State* L)
{
    static const char* kinds[]={"intangible","invincible","metal","size",NULL};
    int e=gs_caps_entity(L,1),kind=luaL_checkoption(L,2,NULL,kinds),frames;
    float value;
    if(lua_gettop(L)==2) {
        frames=gw_ScriptGame_CapsEffectRead(e,kind,0);
        if(!frames) { lua_pushnil(L); return 1; }
        lua_createtable(L,0,2); gs_setint(L,"frames",frames);
        { union{int i;float f;} v; v.i=gw_ScriptGame_CapsEffectRead(e,kind,1); gs_setnum(L,"value",v.f); }
        return 1;
    }
    gs_caps_write(L,"fighter_effect");
    value=gs_caps_number(L,3,kind==3 ? .25 : 0,kind==3 ? 4 : 1);
    frames=gs_caps_frames(L,4);
    gs_rw_branch(); lua_pushboolean(L,gw_ScriptGame_CapsEffect(e,gs.cur+1,kind,gs_fbits(value),frames));
    return 1;
}
static int l_fighter_armour(lua_State* L)
{
    int e=gs_caps_entity(L,1); float damage=0,kb=0;
    if(lua_gettop(L)==1) {
        union{int i;float f;} v;
        if(!gw_ScriptGame_CapsArmourRead(e,0)) {lua_pushnil(L);return 1;}
        lua_createtable(L,0,2);
        v.i=gw_ScriptGame_CapsArmourRead(e,1);gs_setnum(L,"damage",v.f);
        v.i=gw_ScriptGame_CapsArmourRead(e,2);gs_setnum(L,"knockback",v.f);
        return 1;
    }
    gs_caps_write(L,"fighter_armour");
    if(!lua_isnoneornil(L,2)) {
        luaL_checktype(L,2,LUA_TTABLE); lua_pushnil(L);
        while(lua_next(L,2)) {
            const char* k=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
            float v=gs_caps_number(L,-1,0,1000);
            if(k && !strcmp(k,"damage")) damage=v;
            else if(k && !strcmp(k,"knockback")) kb=v;
            else return luaL_error(L,"unknown armour threshold");
            lua_pop(L,1);
        }
    }
    gs_rw_branch(); lua_pushboolean(L,gw_ScriptGame_CapsArmour(e,gs.cur+1,gs_fbits(damage),gs_fbits(kb)));
    return 1;
}
static int l_give_item(lua_State* L)
{
    int e=gs_caps_entity(L,1),kind=-1;
    gs_caps_write(L,"give_item");
    if(lua_type(L,2)==LUA_TSTRING) {
        if(strcmp(lua_tostring(L,2),"random")) return luaL_error(L,"item must be a retail kind ID or 'random'");
    } else {
        lua_Integer n=luaL_checkinteger(L,2);
        if(n<0 || n>255) return luaL_error(L,"item kind outside retail ID range");
        kind=(int)n;
    }
    gs_rw_branch();
    {
        int r=gw_ScriptGame_CapsGiveItem(e,gs.cur+1,kind);
        lua_pushboolean(L,r>0);
        if(r>0) return 1;
        /* Second result: why it was refused (also logged natively with the entity and kind). */
        lua_pushstring(L,gw_ScriptGame_CapsGiveItemReason(r)); return 2;
    }
}
static int l_nearest_opponent(lua_State* L)
{
    int e=gs_caps_entity(L,1),target,exclude=lua_isnoneornil(L,2) ? 0 : gs_caps_entity(L,2);
    target=gs.match_active ? gw_ScriptGame_CapsOpponent(e,gs_fbits(100000),0,exclude) : 0;
    if(target) lua_pushinteger(L,target); else lua_pushnil(L);
    return 1;
}
static int l_opponents_in_radius(lua_State* L)
{
    int e=gs_caps_entity(L,1),n,id,exclude=lua_isnoneornil(L,3) ? 0 : gs_caps_entity(L,3);
    float radius=gs_caps_number(L,2,0,100000);
    lua_newtable(L);
    if(gs.match_active) for(n=1;n<=12;++n) {
        id=gw_ScriptGame_CapsOpponent(e,gs_fbits(radius),n,exclude); if(!id) break;
        lua_pushinteger(L,id); lua_rawseti(L,-2,n);
    }
    return 1;
}
static int l_fighter_timed_status(lua_State* L)
{
    int e=gs_caps_entity(L,1),channel,frames; float value;
    lua_Integer c=luaL_checkinteger(L,2);
    if(c<1 || c>4) return luaL_error(L,"fighter status channel must be 1-4");
    channel=(int)c;
    if(lua_gettop(L)==2) {
        union {int i;float f;} v;
        frames=gw_ScriptGame_CapsStatusRead(e,channel,0);
        if(!frames) {lua_pushnil(L);return 1;}
        v.i=gw_ScriptGame_CapsStatusRead(e,channel,1);
        lua_createtable(L,0,2);gs_setint(L,"frames",frames);gs_setnum(L,"value",v.f);return 1;
    }
    gs_caps_write(L,"fighter_timed_status");value=gs_caps_number(L,3,-100000,100000);
    frames=gs_caps_frames(L,4);gs_rw_branch();
    lua_pushboolean(L,gw_ScriptGame_CapsStatus(e,channel,gs.cur+1,gs_fbits(value),frames));return 1;
}
/* gd.fighter_interrupt(entity [, {frames=, exits={...}, guard=, restore_jumps=}]): the interrupt window
 * (script_fighter_interrupt.inc), opened now for the fighter's CURRENT action. Offline only: online it is
 * the Turbo match rule's, agreed by both peers, and a script cannot write it. With no table it reads:
 * nil, or {frames, exits={names}, key, opened, cancels}. A table of frames=0 closes this script's window. */
static const char* const gs_iw_exits[]={"jab","tilt","smash","aerial","special","grab","jump","dash","crouch","turn","walk","escape","shield","air_dodge","air_jump",NULL};
static int l_fighter_interrupt(lua_State* L)
{
    int e=gs_caps_entity(L,1),frames=0,exits=0x7FFF,flags=0,i;
    if(lua_gettop(L)==1) {
        int m;
        if(gw_ScriptGame_IntrWinRead(e,0)<=0) { lua_pushnil(L); return 1; }
        lua_createtable(L,0,6);
        gs_setint(L,"frames",gw_ScriptGame_IntrWinRead(e,0));
        gs_setint(L,"owner",gw_ScriptGame_IntrWinRead(e,1));
        gs_setint(L,"key",gw_ScriptGame_IntrWinRead(e,3));
        gs_setint(L,"opened",gw_ScriptGame_IntrWinRead(e,4));
        gs_setint(L,"cancels",gw_ScriptGame_IntrWinRead(e,5));
        m=gw_ScriptGame_IntrWinRead(e,2);
        lua_createtable(L,0,0);
        for(i=0;gs_iw_exits[i];++i) if(m&(1<<i)) { lua_pushboolean(L,1); lua_setfield(L,-2,gs_iw_exits[i]); }
        lua_setfield(L,-2,"exits");
        return 1;
    }
    gs_caps_write(L,"fighter_interrupt");
    luaL_checktype(L,2,LUA_TTABLE); lua_pushnil(L);
    while(lua_next(L,2)) {
        const char* k=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
        if(k && !strcmp(k,"frames")) {
            lua_Integer n=luaL_checkinteger(L,-1);
            if(n<0 || n>3600) return luaL_error(L,"interrupt window must be 0-3600 logic frames");
            frames=(int)n;
        } else if(k && !strcmp(k,"exits")) {
            luaL_checktype(L,-1,LUA_TTABLE); exits=0; lua_pushnil(L);
            while(lua_next(L,-2)) {
                const char* x=lua_type(L,-1)==LUA_TSTRING ? lua_tostring(L,-1) : NULL;
                for(i=0;x && gs_iw_exits[i];++i) if(!strcmp(x,gs_iw_exits[i])) break;
                if(!x || !gs_iw_exits[i]) return luaL_error(L,"unknown interrupt exit");
                exits|=1<<i; lua_pop(L,1);
            }
        } else if(k && !strcmp(k,"guard")) { luaL_checktype(L,-1,LUA_TBOOLEAN); if(lua_toboolean(L,-1)) flags|=1; }
        else if(k && !strcmp(k,"restore_jumps")) { luaL_checktype(L,-1,LUA_TBOOLEAN); if(lua_toboolean(L,-1)) flags|=2; }
        else return luaL_error(L,"unknown interrupt option");
        lua_pop(L,1);
    }
    gs_rw_branch();
    lua_pushboolean(L,gw_ScriptGame_IntrWinSet(e,gs.cur+1,frames,exits,flags));
    return 1;
}
static void gs_caps_release(int owner)
{
    gw_ScriptGame_IntrWinRelease(owner);
    gw_ScriptGame_CapsMovementRelease(owner);
    gw_ScriptGame_CapsEffectsRelease(owner);
    gw_ScriptGame_CapsStatusRelease(owner);
    gw_ScriptGame_CritRelease(owner);
    gw_ScriptGame_ShockRelease(owner);
}
