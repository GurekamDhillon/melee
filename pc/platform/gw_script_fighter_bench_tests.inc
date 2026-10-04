extern int gw_ScriptGame_FighterBenchTest(void);
extern int gw_ftCommon_PcFloorGuardTest(void);
extern int gw_ScriptGame_CpuModeTest(void);
static int test_script_fighter_bench(void) {
    char out[2048];
    if (gw_ftCommon_PcFloorGuardTest()) {
        gw_test_fail("removed grounded floor did not recover to airborne Fall"); return 1;
    }
    if (gw_ScriptGame_CpuModeTest()) {
        gw_test_fail("CPU mode pair, transformation or reserve controller fixture failed"); return 1;
    }
    if (gw_ScriptGame_FighterBenchTest()) {
        gw_test_fail("reserve safety, partner or reversible flags fixture failed"); return 1;
    }
    if (t_exec("assert(type(gd.fighter_bench)=='function' and type(gd.fighter_call)=='function'); "
               "local yes,entities=gd.fighter_benched(1); assert(not yes); "
               "assert(type(entities)=='table' and #entities==2); "
               "assert(entities[1].entity_index==0 and entities[2].entity_index==1); "
               "assert(not entities[1].present and not entities[2].present); "
               "assert(not pcall(gd.fighter_bench,0)); "
               "assert(not pcall(gd.fighter_call,7,0,0)); "
               "assert(not pcall(gd.fighter_call,1,0/0,0)); "
               "assert(not pcall(gd.fighter_call,1,0,0,{facing=0})); "
               "assert(not pcall(gd.fighter_call,1,0,0,{intangible_frames=-1}))",out,sizeof out)) {
        gw_test_fail("reserve Lua validation: %s",out); return 1;
    }
    return 0;
}
