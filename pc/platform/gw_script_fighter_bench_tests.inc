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

extern int gw_ScriptGame_CpuCtlTest(void);
/* gd.cpu_pad / cpu_script / cpu_macro: the controller on a fixture, and the Lua surface's refusals */
static int test_script_cpu_ctl(void) {
    char out[1024];
    if (gw_ScriptGame_CpuCtlTest()) { gw_test_fail("CPU virtual controller fixture failed"); return 1; }
    if (t_exec("assert(type(gd.cpu_pad)=='function' and type(gd.cpu_script)=='function'); "
               "local c=gd.cpu_commands(); assert(c.press_a.code==1 and c.wait.args==1 and c.wait_for.code==c.wait.code); "
               "assert(c.lstick_toward_destination_clamped.args==2 and c.set_lstick_x.code==0x80); "
               "assert(not pcall(gd.cpu_script,1,{}) and not pcall(gd.cpu_script,1,{'nope'})); "
               "assert(not pcall(gd.cpu_script,1,{{'wait'}}) and not pcall(gd.cpu_script,1,{{'press_a',1}})); "
               "assert(not pcall(gd.cpu_script,1,{{'set_lstick_x',128}}) and not pcall(gd.cpu_script,1,{{'wait',256}})); "
               "assert(not pcall(gd.cpu_pad,1,{x=1},0) and not pcall(gd.cpu_pad,0,{})); "
               "assert(not pcall(gd.cpu_macro,1,'nope')); assert(not pcall(gd.cpu_dest,1,0/0,0)); "
               "local long={} for i=1,200 do long[i]={'wait',1} end; assert(not pcall(gd.cpu_script,1,long)); "
               "assert(not pcall(gd.cpu_goto,1,0/0,0) and not pcall(gd.cpu_goto,1,0,0,{radius=0}) and not pcall(gd.cpu_goto,1,0,0,{timeout=0}) and not pcall(gd.cpu_goto,9,0,0)); "
               "assert(not pcall(gd.cpu_goto_status,1) and not pcall(gd.cpu_cancel,1) and type(gd.cpu_goto_status)=='function')",
               out, sizeof out) != 0) {
        gw_test_fail("CPU controller Lua validation: %s", out); return 1;
    }
    return 0;
}
