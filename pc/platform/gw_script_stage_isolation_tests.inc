/* Included after gw_test.h / t_exec: native raycasts plus API/owner refusal. */
static int test_script_stage_isolate(void) {
    char out[2048];
    int saved_owner = gs_stage_isolation_owner;
    int failed = gw_mpScriptStageIsolationTest();
    if (failed) {
        gw_test_fail("stage isolation native floor/remap restoration failed (%d cases)", failed);
        return 1;
    }
    if (t_exec("assert(type(gd.stage_isolate)=='function'); "
               "assert(not pcall(gd.stage_isolate)); "
               "assert(not pcall(gd.stage_isolate,true)); "
               "assert(not pcall(gd.stage_isolate,false)); "
               "assert(not pcall(gd.stage_isolate,'true'))", out, sizeof out) != 0) {
        gw_test_fail("stage isolation allowed console ownership: %s", out);
        return 1;
    }
    if (gw_ScriptGame_StageIsolate(0, 1) || gw_ScriptGame_StageIsolate(-1, 1) ||
        gw_Script_StageIsolationAllowed(0) || gw_Script_StageIsolationAllowed(-1)) {
        gw_test_fail("stage isolation accepted an invalid owner"); return 1;
    }
    gs_stage_isolation_owner = 77;
    gs_stage_isolation_release_owner(78);
    if (gs_stage_isolation_owner != 77) failed++;
    gw_Script_StageIsolationReleased(78);
    if (gs_stage_isolation_owner != 77) failed++;
    gw_Script_StageIsolationReleased(77);
    if (gs_stage_isolation_owner) failed++;
    gs_stage_isolation_owner = saved_owner;
    if (failed) gw_test_fail("stage isolation released another script's ownership");
    return failed != 0;
}
