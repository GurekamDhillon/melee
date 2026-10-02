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
