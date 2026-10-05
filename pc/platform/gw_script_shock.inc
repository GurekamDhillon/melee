/* Shock, script side (included after gw_script_crit.inc). The status is game-side (pc/gameworld/script_shock.inc);
 * this file owns gd.shock and its journal op. Events ride the skill ring: on_shock, on_shock_hit, on_shock_end. */
extern int gw_ScriptGame_ShockSet(int entity,int owner,int frames,int charges,int mult_bits,int bonus,int flags);
extern int gw_ScriptGame_ShockRead(int entity,int field);
extern void gw_ScriptGame_ShockRelease(int owner);
/* gd.shock(entity) reads {frames=, charges=, hitstun_multiplier=, bonus=, hits=, applied=, owner=} or nil;
 * gd.shock(entity, nil) clears this script's Shock; gd.shock(entity, {frames=1..3600 (default 300), charges=1..8 (default 1:
 * how many hits taken), hitstun=1..4 (a multiplier on the hit's hitstun frames, default 1.5), bonus=0..120 (flat extra
 * frames, default 0), stack=true (add charges to a running Shock instead of replacing it)}) applies it. Offline gameplay
 * write; false when another script owns the entity's Shock or the entity is missing. */
static int l_shock(lua_State* L)
{
    static const char* const keys[]={"frames","charges","hitstun","bonus","stack",NULL};
    int e=gs_caps_entity(L,1),owner=gs.cur+1,frames=300,charges=1,bonus=0,flags=1;float mult=1.5f;
    if(lua_gettop(L)==1){
        int ow=gw_ScriptGame_ShockRead(e,0);
        if(!ow){lua_pushnil(L);return 1;}
        lua_createtable(L,0,8);
        gs_setint(L,"owner",ow);gs_setint(L,"frames",gw_ScriptGame_ShockRead(e,1));gs_setint(L,"charges",gw_ScriptGame_ShockRead(e,2));
        gs_setint(L,"bonus",gw_ScriptGame_ShockRead(e,3));gs_setnum(L,"hitstun_multiplier",gs_skill_f(gw_ScriptGame_ShockRead(e,4)));
        gs_setint(L,"hits",gw_ScriptGame_ShockRead(e,5));gs_setint(L,"applied",gw_ScriptGame_ShockRead(e,6));
        return 1;
    }
    gs_caps_write(L,"shock");
    {int ow=gw_ScriptGame_ShockRead(e,0);if(ow&&ow!=owner){lua_pushboolean(L,0);return 1;}}
    if(lua_isnil(L,2)){gs_rw_branch();lua_pushboolean(L,gw_ScriptGame_ShockSet(e,owner,0,0,0,0,4));return 1;}
    luaL_checktype(L,2,LUA_TTABLE);gs_item_keys(L,2,keys);
    lua_getfield(L,2,"frames");if(!lua_isnil(L,-1)){lua_Integer n=luaL_checkinteger(L,-1);if(n<1||n>3600)luaL_error(L,"shock frames must be 1-3600");frames=(int)n;}lua_pop(L,1);
    lua_getfield(L,2,"charges");if(!lua_isnil(L,-1)){lua_Integer n=luaL_checkinteger(L,-1);if(n<1||n>8)luaL_error(L,"shock charges must be 1-8");charges=(int)n;}lua_pop(L,1);
    lua_getfield(L,2,"hitstun");if(!lua_isnil(L,-1))mult=gs_caps_number(L,-1,1,4);lua_pop(L,1);
    lua_getfield(L,2,"bonus");if(!lua_isnil(L,-1)){lua_Integer n=luaL_checkinteger(L,-1);if(n<0||n>120)luaL_error(L,"shock bonus must be 0-120");bonus=(int)n;}lua_pop(L,1);
    lua_getfield(L,2,"stack");if(!lua_isnil(L,-1)){luaL_checktype(L,-1,LUA_TBOOLEAN);if(lua_toboolean(L,-1))flags=2;}lua_pop(L,1);
    gs_rw_branch();
    lua_pushboolean(L,gw_ScriptGame_ShockSet(e,owner,frames,charges,gs_fbits(mult),bonus,flags));
    return 1;
}
