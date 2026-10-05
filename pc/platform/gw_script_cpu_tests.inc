/* Native and Lua API tests; included after gw_test.h and t_exec. */
static int test_script_cpu_technical(void) {
    char out[2048];
    int failed = gw_ScriptGame_CpuTechnicalTest();
    if (failed) {
        gw_test_fail("CPU technical native policy had %d failed cases", failed);
        return 1;
    }
    if (t_exec("assert(type(gd.cpu_technical)=='function'); "
               "assert(not pcall(gd.cpu_technical,0,1)); "
               "assert(not pcall(gd.cpu_technical,7,1)); "
               "assert(not pcall(gd.cpu_technical,1,-1)); "
               "assert(not pcall(gd.cpu_technical,1,4)); "
               "assert(not pcall(gd.cpu_technical,1,1,0)); "
               "assert(not pcall(gd.cpu_technical,1,1,2147483648)); "
               "assert(not pcall(gd.cpu_technical,1,1.5)); "
               "assert(not pcall(gd.cpu_technical,1,'hard')); "
               "local s=gd.cpu_technical(1); assert(type(s)=='table' and "
               "type(s.enabled)=='boolean' and type(s.reaction_frames)=='number'); "
               "assert(gd.cpu_technical(1,0))", out, sizeof out) != 0) {
        gw_test_fail("CPU technical API validation failed: %s", out);
        return 1;
    }
    if (gw_ScriptGame_CpuTechnical(-1, 1, 1, 1) ||
        gw_ScriptGame_CpuTechnical(6, 1, 1, 1) ||
        gw_ScriptGame_CpuTechnical(0, 0, 1, 1) ||
        gw_ScriptGame_CpuTechnical(0, 1, -1, 1) ||
        gw_ScriptGame_CpuTechnical(0, 1, 4, 1) ||
        gw_ScriptGame_CpuTechnical(0, 1, 1, 0)) {
        gw_test_fail("CPU technical accepted invalid native controls");
        return 1;
    }
    return 0;
}

static int test_script_cpu_assist(void) {
    char out[2048];
    int failed = gw_ScriptGame_CpuAssistTest();
    if (failed) {
        gw_test_fail("CPU assist native policy had %d failed cases", failed);
        return 1;
    }
    if (t_exec("assert(type(gd.cpu_assist)=='function'); "
               "assert(not pcall(gd.cpu_assist,1,{lcancel=1.5})); "
               "assert(not pcall(gd.cpu_assist,1,{tech=-0.1})); "
               "assert(not pcall(gd.cpu_assist,1,{tech='x'})); "
               "assert(not pcall(gd.cpu_assist,1,{tech_dir='up'})); "
               "assert(not pcall(gd.cpu_assist,1,{seed=0})); "
               "assert(not pcall(gd.cpu_assist,1,{seed=1.5})); "
               "assert(not pcall(gd.cpu_assist,9,{})); assert(not pcall(gd.cpu_assist,0,{})); "
               "assert(not pcall(gd.cpu_assist,1,7)); "
               "local ok,r=pcall(gd.cpu_assist,1,{lcancel=1}); assert(not ok or r==false); "  /* no match: refused */
               "local s=gd.cpu_assist(1); assert(type(s)=='table' and s.enabled==false and "
               "type(s.probability)=='table' and type(s.counters.tech.opportunities)=='number'); "
               "ok,r=pcall(gd.cpu_assist,1,nil); assert(not ok or r==false)", out, sizeof out) != 0) {
        gw_test_fail("CPU assist API validation failed: %s", out);
        return 1;
    }
    if (gw_ScriptGame_CpuAssist(-1, 1, 0, 0, 0, 0, 0, 0, 1) || gw_ScriptGame_CpuAssist(6, 1, 0, 0, 0, 0, 0, 0, 1) ||
        gw_ScriptGame_CpuAssist(0, 0, 0, 0, 0, 0, 0, 0, 1) || gw_ScriptGame_CpuAssist(0, 1, 1001, 0, 0, 0, 0, 0, 1) ||
        gw_ScriptGame_CpuAssist(0, 1, 0, 0, 0, 4, 0, 0, 1) || gw_ScriptGame_CpuAssist(0, 1, 0, 0, 0, 0, 0, 0, 0)) {
        gw_test_fail("CPU assist accepted invalid native controls");
        return 1;
    }
    return 0;
}
