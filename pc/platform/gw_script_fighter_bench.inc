extern int gw_ScriptGame_FighterBenchCheck(int slot,int owner,int calling);
extern int gw_ScriptGame_FighterBench(int slot,int owner);
extern int gw_ScriptGame_FighterCall(int slot,int owner,int x,int y,int facing,int intangible);
extern int gw_ScriptGame_FighterBenched(int slot);
extern int gw_ScriptGame_FighterReserveRead(int slot,int entity,int field);
extern void gw_ScriptGame_FighterBenchRelease(int owner);
static int gs_reserve_result(lua_State* L,int rc,const char* op,int slot) {
    static const char* reasons[]={"ok","missing fighter or camera box","boss fighter",
        "dead, asleep or respawning","grabbed, grabbing, carried or thrown",
        "hitlag or externally frozen","transformation in progress",
        "unsafe action or held item (requires wait, locomotion, fall or teeter)",
        "owned by another script","fighter is not benched","fighter entity changed",
        "invalid call parameters"};
    lua_pushboolean(L,rc==0);
    if (!rc) return 1;
    if (rc<0 || rc>11) rc=1;
    gw_log("script reserve: %s refused port=%d reason=%s",op,slot+1,reasons[rc]);
    lua_pushstring(L,reasons[rc]); return 2;
}
static int l_fighter_benched(lua_State* L) {
    static const char* names[]={"present","kind","action","airborne","x","y",
        "cpu_kind","frozen","invisible","intangible","camera_excluded",
        "dormant","benched","refusal_code"};
    int slot=gs_slot_arg(L,1), entity, field, present;
    lua_pushboolean(L,gs.match_active && gw_ScriptGame_FighterBenched(slot));
    lua_newtable(L);
    for (entity=0; entity<2; ++entity) {
        lua_newtable(L);
        gs_setint(L,"entity_index",entity);
        present=gs.match_active && gw_ScriptGame_FighterReserveRead(slot,entity,0);
        gs_setbool(L,"present",present);
        if (present) for (field=1; field<14; ++field) {
            int value=gw_ScriptGame_FighterReserveRead(slot,entity,field);
            if (field==4 || field==5) gs_setnum(L,names[field],gs_camera_float(value));
            else if (field==1 || field==2 || field==6 || field==13) gs_setint(L,names[field],value);
            else gs_setbool(L,names[field],value);
        }
        lua_rawseti(L,-2,entity+1);
    }
    return 2;
}
static int l_fighter_bench(lua_State* L) {
    int slot=gs_slot_arg(L,1), rc;
    gs_require_offline(L,"fighter_bench");
    if (!gs.match_active) return gs_reserve_result(L,1,"bench",slot);
    rc=gw_ScriptGame_FighterBenchCheck(slot,gs.cur+1,0);
    if (!rc) { gs_rw_branch(); rc=gw_ScriptGame_FighterBench(slot,gs.cur+1); }
    return gs_reserve_result(L,rc,"bench",slot);
}
static int l_fighter_call(lua_State* L) {
    int slot=gs_slot_arg(L,1), rc, frames=0;
    float x=gs_stage_num(L,2), y=gs_stage_num(L,3), facing=1;
    gs_require_offline(L,"fighter_call");
    if (fabsf(x)>49000 || fabsf(y)>49000) return luaL_error(L,"reserve coordinates must be within +/-49000");
    if (!lua_isnoneornil(L,4)) {
        lua_Integer n;
        luaL_checktype(L,4,LUA_TTABLE);
        lua_getfield(L,4,"facing"); facing=(float)luaL_optnumber(L,-1,1); lua_pop(L,1);
        lua_getfield(L,4,"intangible_frames"); n=luaL_optinteger(L,-1,0); lua_pop(L,1);
        if ((facing!=1 && facing!=-1) || n<0 || n>600)
            return luaL_error(L,"facing must be -1 or 1; intangible_frames must be 0..600");
        frames=(int)n;
    }
    if (!gs.match_active) return gs_reserve_result(L,1,"call",slot);
    rc=gw_ScriptGame_FighterBenchCheck(slot,gs.cur+1,1);
    if (!rc) { gs_rw_branch(); rc=gw_ScriptGame_FighterCall(slot,gs.cur+1,gs_fbits(x),gs_fbits(y),gs_fbits(facing),frames); }
    return gs_reserve_result(L,rc,"call",slot);
}
