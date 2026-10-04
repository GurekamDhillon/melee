/* Leaf parser checks run without a match and without touching game BSS. */
static int gs_echo_test_rule(lua_State *L) {
    int b[GS_ECHO_FIELDS];gs_echo_rule(L,1,b);lua_pushinteger(L,b[1]);lua_pushinteger(L,b[2]);lua_pushboolean(L,b[8]);return 3;
}
static int test_script_echo_parser(void) {
    static const char script[]=
        "local d,m,o=parse{delay=60,match={move='nair',airborne=true},damage=.1,knockback=4,once_per_move=false};assert(d==60 and m==10 and not o);"
        "for _,r in ipairs({{delay=0},{delay=61},{delay=4294967296},{damage=0/0},{knockback=1/0},{match={move='projectile'}},{once_per_move=1},{unknown=true}}) do assert(not pcall(parse,r)) end";
    lua_State *L=luaL_newstate();int rc;if(!L){gw_test_fail("echo parser test Lua allocation failed");return 1;}
    luaL_requiref(L,"_G",luaopen_base,1);lua_pop(L,1);lua_pushcfunction(L,gs_echo_test_rule);lua_setglobal(L,"parse");
    rc=luaL_loadstring(L,script);if(!rc)rc=lua_pcall(L,0,0,0);
    if(rc)gw_test_fail("echo production parser: %s",lua_tostring(L,-1));lua_close(L);return rc?1:0;
}
