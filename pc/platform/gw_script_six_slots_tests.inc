extern int gw_ScriptGame_SixSlotsTest(void);
static int test_script_six_slots(void)
{
    char out[2048];
    if (gw_ScriptGame_SixSlotsTest()) {
        gw_test_fail("six-slot virtual input lifecycle or recycle safety failed"); return 1;
    }
    if (t_exec("assert(type(gd.fighter_recycle)=='function'); "
        "assert(gd.player(5)==nil and gd.player(6)==nil); "
        "assert(type(gd.players())=='table'); "
        "assert(gd.pad(5).buttons==0 and gd.pad(6).x==0); "
        "assert(not pcall(gd.player,0) and not pcall(gd.player,7)); "
        "assert(not pcall(gd.input,7,'A')); "
        "assert(not pcall(gd.fighter_recycle,7,{x=0,y=0})); "
        "assert(not pcall(gd.fighter_recycle,6,{x=0/0,y=0})); "
        "assert(not pcall(gd.fighter_recycle,5,{x=0,y=0,facing=0})); "
        "assert(not pcall(gd.fighter_recycle,5,{x=0,y=0,intangible_frames=601})); "
        "local ok,why=gd.fighter_recycle(6,{x=0,y=0}); assert(not ok and why:find('missing'))",
        out,sizeof out)) {
        gw_test_fail("six-slot Lua range/recycle validation: %s",out); return 1;
    }
    {
        int i, top=lua_gettop(gs.L), rc=0;
        for (i=0; i<2; ++i) {
            const char* why=gs_lab_leave_reason(i,1);
            if (!why || gs_lab_leave_reply(gs.L,why)!=2 ||
                lua_toboolean(gs.L,-2) || !lua_isstring(gs.L,-1) ||
                !strstr(lua_tostring(gs.L,-1),"six-fighter")) rc=1;
            lua_settop(gs.L,top);
        }
        if (gs_lab_leave_reason(0,0) || gs_lab_leave_reason(2,1) ||
            gs_lab_leave_reason(3,1) || gs_lab_leave_reply(gs.L,NULL)!=1 ||
            !lua_toboolean(gs.L,-1)) rc=1;
        lua_settop(gs.L,top);
        if (rc) { gw_test_fail("LAB leave reason/return arity failed"); return 1; }
    }
    return 0;
}
