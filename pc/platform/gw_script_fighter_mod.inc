extern int gw_ScriptGame_FighterModOwner(int slot);
extern int gw_ScriptGame_FighterModRead(int slot,int field);
extern int gw_ScriptGame_FighterMod(int slot,int owner,int enabled,int a,int b,int c,int d,int e);
extern void gw_ScriptGame_FighterModsRelease(int owner);
static int l_fighter_mod(lua_State* L) {
    static const char* keys[5]={"damage_dealt","damage_taken","run_speed","air_speed","shield_max"};
    int slot=gs_slot_arg(L,1), i, enabled=!lua_isnoneornil(L,2), owner;
    float v[5]={1,1,1,1,1};
    if (lua_gettop(L)==1) {
        if (!gw_ScriptGame_FighterModOwner(slot)) { lua_pushnil(L); return 1; }
        lua_createtable(L,0,5);
        for (i=0;i<5;++i) {
            union { int i; float f; } u;
            u.i=gw_ScriptGame_FighterModRead(slot,i);
            gs_setnum(L,keys[i],u.f);
        }
        return 1;
    }
    gs_require_offline(L,"fighter_mod");
    if (!gs.match_active) return luaL_error(L,"fighter_mod requires an active match");
    owner=gw_ScriptGame_FighterModOwner(slot);
    if (owner && owner!=gs.cur+1) return luaL_error(L,"fighter modifiers owned by another script");
    if (enabled) {
        luaL_checktype(L,2,LUA_TTABLE); lua_pushnil(L);
        while (lua_next(L,2)) {
            const char* key=lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
            double n=luaL_checknumber(L,-1);
            for (i=0; i<5; ++i) if (key && !strcmp(key,keys[i])) break;
            if (i==5) return luaL_error(L,"unknown fighter modifier");
            if (!isfinite(n)) return luaL_error(L,"fighter modifier must be finite");
            v[i]=(float)(n<0.1 ? 0.1 : n>4 ? 4 : n); lua_pop(L,1);
        }
    }
    gs_rw_branch();
    lua_pushboolean(L,gw_ScriptGame_FighterMod(slot,gs.cur+1,enabled,gs_fbits(v[0]),
        gs_fbits(v[1]),gs_fbits(v[2]),gs_fbits(v[3]),gs_fbits(v[4])));
    return 1;
}
