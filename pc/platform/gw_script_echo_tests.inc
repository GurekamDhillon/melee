/* Leaf parser checks run without a match and without touching game BSS. */
static int gs_echo_test_rule(lua_State *L) {
    int b[GS_ECHO_FIELDS];gs_echo_rule(L,1,b);lua_pushinteger(L,b[1]);lua_pushinteger(L,b[2]);lua_pushboolean(L,b[8]);return 3;
}
static int test_script_echo_parser(void) {
    static const char script[]=
        "local d,m,o=parse{delay=60,match={move='nair',airborne=true},damage=.1,knockback=4,once_per_move=false};assert(d==60 and m==10 and not o);"
        "for _,r in ipairs({{delay=0},{delay=61},{delay=4294967296},{damage=0/0},{knockback=1/0},{match={move='projectile'}},{once_per_move=1},{unknown=true}}) do assert(not pcall(parse,r)) end";
    lua_State *L=luaL_newstate();int rc,slot,grew;if(!L){gw_test_fail("echo parser test Lua allocation failed");return 1;}
    luaL_requiref(L,"_G",luaopen_base,1);lua_pop(L,1);lua_pushcfunction(L,gs_echo_test_rule);lua_setglobal(L,"parse");
    rc=luaL_loadstring(L,script);if(!rc)rc=lua_pcall(L,0,0,0);
    if(rc)gw_test_fail("echo production parser: %s",lua_tostring(L,-1));lua_close(L);
    /* Regression: echo rules are owned by script slot + 1; the per-frame stale-owner sweep once
     * compared that against the stage-resource token (a different number space) and deleted every
     * script-created rule within a frame. A live script slot must read as alive under its echo owner. */
    grew=gs.n;slot=gs_alloc_script();
    if(slot>=0){
        gs.s[slot].used=1;
        if(!gw_Script_EchoOwnerAlive(slot+1)){gw_test_fail("echo owner slot %d reads as dead while the script is loaded",slot+1);rc=1;}
        if(gw_Script_EchoOwnerAlive(gs.n+1)||gw_Script_EchoOwnerAlive(0)){gw_test_fail("echo owner alive for a slot that is not loaded");rc=1;}
        gs.s[slot].used=0;if(gs.n>grew&&slot==gs.n-1)gs.n--;
        if(gw_Script_EchoOwnerAlive(slot+1)){gw_test_fail("echo owner alive after its script unloaded");rc=1;}
    }
    return rc?1:0;
}
