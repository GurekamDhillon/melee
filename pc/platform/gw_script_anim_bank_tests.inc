/* Exercise the exported Lua rejection path: no game/disc assets needed. */
static int test_script_anim_bank_api(void)
{
    char out[512];
    if (t_exec("assert(type(gd.reload_anim_bank)=='function'); "
               "assert(type(gd.reload_anim_bank_status)=='function'); "
               "local ok,why=gd.reload_anim_bank('__missing_define__'); "
               "assert(ok==false and type(why)=='string' and #why>0); "
               "local s=gd.reload_anim_bank_status(); assert(type(s.phase)=='string' and type(s.text)=='string')",
               out, sizeof out) != 0) {
        gw_test_fail("animation bank Lua contract: %s", out); return 1;
    }
    return 0;
}
static int test_workbench_anim_bank_gate(void)
{
    lua_State* L = luaL_newstate();
    int rc;
    if (!L) return 1;
    lua_pushcfunction(L, l_wb_reload_anim);
    lua_pushliteral(L, "__missing_define__");
    rc = lua_pcall(L, 1, 1, 0);
    if (rc == LUA_OK) {
        gw_test_fail("Workbench animation reload accepted without its live fighter");
        lua_close(L); return 1;
    }
    lua_close(L); return 0;
}
