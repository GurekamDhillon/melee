extern int gw_ScriptGame_VirtualPadField(int slot,int field,int value,int writing);
extern void gw_ScriptGame_VirtualPadConsume(void);
extern int gw_ScriptGame_FighterRecycleCheck(int slot,int character);
extern int gw_ScriptGame_FighterRecycle(int slot,int character,int x,int y,int facing,int intangible);
static int l_fighter_recycle(lua_State* L)
{
    static const char* why[]={"ok","missing fighter or camera","character change refused: recycle keeps the launch character; use another preloaded enemy slot",
        "paired fighters/transforms or bosses cannot be recycled","enemy slot must be CPU",
        "enemy must have finished its KO (Sleep/Rebirth/RebirthWait); use a time/LAB match so the entity survives","invalid placement"};
    int slot=gs_slot_arg(L,1), character=-1, frames=0, rc;
    float x,y,facing=1;
    gs_require_offline(L,"fighter_recycle");
    luaL_checktype(L,2,LUA_TTABLE);
    lua_getfield(L,2,"x"); x=gs_stage_num(L,-1); lua_pop(L,1);
    lua_getfield(L,2,"y"); y=gs_stage_num(L,-1); lua_pop(L,1);
    lua_getfield(L,2,"facing"); facing=(float)luaL_optnumber(L,-1,1); lua_pop(L,1);
    lua_getfield(L,2,"intangible_frames");
    { lua_Integer n=luaL_optinteger(L,-1,0);
      if (n<0 || n>600) return luaL_error(L,"intangible_frames must be 0..600");
      frames=(int)n; }
    lua_pop(L,1);
    /* CharacterKind is explicit, like gd.player(port).char. Names are launch grammar. */
    lua_getfield(L,2,"character");
    if (!lua_isnil(L,-1)) {
        lua_Integer n=luaL_checkinteger(L,-1);
        if (n<0 || n>127) return luaL_error(L,"character must be a CharacterKind in 0..127");
        character=(int)n;
    }
    lua_pop(L,1);
    if (fabsf(x)>49000 || fabsf(y)>49000 || (facing!=1 && facing!=-1))
        return luaL_error(L,"recycle coordinates +/-49000; facing -1 or 1");
    rc=gs.match_active ? gw_ScriptGame_FighterRecycleCheck(slot,character) : 1;
    if (!rc) { gs_rw_branch(); rc=gw_ScriptGame_FighterRecycle(slot,character,gs_fbits(x),gs_fbits(y),gs_fbits(facing),frames); }
    lua_pushboolean(L,rc==0);
    if (!rc) return 1;
    if (rc<1 || rc>6) rc=1;
    gw_log("script recycle: refused port=%d reason=%s",slot+1,why[rc]);
    lua_pushstring(L,why[rc]); return 2;
}
