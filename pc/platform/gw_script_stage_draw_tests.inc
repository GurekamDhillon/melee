/* No match assets needed: verify draw only controls the presentation flag. */
static int gs_test_stage_draw_opts(lua_State *L) {
    int model;
    lua_pushinteger(L, gs_stage_opts(L, 1, &model));
    return 1;
}
static int test_script_stage_draw(void) {
    int i, top, flags;
    gs_init();
    if (!gs.L) return 1;
    top = lua_gettop(gs.L);
    for (i = 0; i < 4; ++i) {
        lua_pushcfunction(gs.L, gs_test_stage_draw_opts);
        lua_newtable(gs.L);
        lua_pushboolean(gs.L, 1); lua_setfield(gs.L, -2, "passthrough");
        lua_pushboolean(gs.L, 1); lua_setfield(gs.L, -2, "ledges");
        if (i == 0 || i == 1) lua_pushboolean(gs.L, i);
        else if (i == 2) lua_pushstring(gs.L, "false");
        else lua_pushnil(gs.L);
        lua_setfield(gs.L, -2, "draw");
        if (lua_pcall(gs.L, 1, 1, 0) != (i == 2 ? LUA_ERRRUN : LUA_OK)) {
            gw_test_fail("stage draw option accepted wrong type or refused boolean");
            lua_settop(gs.L, top); return 1;
        }
        flags = (int)lua_tointeger(gs.L, -1);
        if (i != 2 && flags != (i == 0 ? 7 : 3)) {
            gw_test_fail("stage draw options altered collision flags: %d", flags);
            lua_settop(gs.L, top); return 1;
        }
        lua_pop(gs.L, 1);
    }
    lua_settop(gs.L, top);
    return 0;
}
