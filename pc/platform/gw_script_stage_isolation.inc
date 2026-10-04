/* Included after gs_require_stage, before the gd registration table. */
static int l_stage_isolate(lua_State *L) {
    GsScript *s = gs_cur_script();
    int isolate;
    gw_ScriptGame_StageIsolationWatch();
    if (gs.cur == gs.console || s == NULL || !s->gameplay)
        return luaL_error(L, "stage_isolate requires a gameplay mod script");
    if (lua_isnoneornil(L, 1)) {
        lua_pushboolean(L, gw_ScriptGame_StageIsolationState(gs.cur + 1));
        return 1;
    }
    luaL_checktype(L, 1, LUA_TBOOLEAN);
    isolate = lua_toboolean(L, 1);
    if (!isolate && gw_ScriptGame_StageSlotsActive())
        return luaL_error(L, "stage slots live: unload the owning script before restoring host geometry");
    if (gs_stage_isolation_owner && gs_stage_isolation_owner != gs.cur + 1)
        return luaL_error(L, "stage isolation belongs to another script");
    if (isolate) {
        gs_require_stage(L, "stage_isolate");
        gs_rw_branch();
        if (!gw_ScriptGame_StageIsolate(gs.cur + 1, 1)) {
            lua_pushboolean(L, 0);
            return 1;
        }
        gs_stage_isolation_owner = gs.cur + 1;
    } else {
        if (gs_stage_isolation_owner == gs.cur + 1) gs_rw_branch();
        gs_stage_isolation_release_owner(gs.cur + 1);
    }
    lua_pushboolean(L, isolate && gw_ScriptGame_StageIsolationState(gs.cur + 1));
    return 1;
}

/* Compatibility wrapper: stage_isolate(false) returns the state (false),
 * while stage_hide(false) reports successful restoration (true). */
static int l_stage_hide(lua_State *L) {
    int hide, result;
    luaL_checktype(L, 1, LUA_TBOOLEAN);
    hide = lua_toboolean(L, 1);
    gs_require_stage(L, "stage_hide"); /* includes restore: never accepted online */
    result = l_stage_isolate(L);
    if (hide && !lua_toboolean(L, -1)) {
        lua_pushstring(L, "host stage isolation unavailable (audited legal retail hosts only)");
        return 2;
    }
    lua_pop(L, result);
    lua_pushboolean(L, 1);
    gw_log("script mission: stage_hide %s accepted", hide ? "true" : "false");
    return 1;
}
