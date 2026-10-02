/* API/refusal coverage in the linked headless native harness. The synthetic
 * actual-source graph and floor-follow fixture lives in pc/tests/stage_seam_test.py. */
static int test_script_stage_link(void)
{
    static const char source[] =
        "local ok,why=gd.stage_link(2147483001,2147483002); "
        "assert(ok==false and type(why)=='string'); "
        "assert(not pcall(gd.stage_link,0,2)); assert(not pcall(gd.stage_link,1.5,2));";
    char out[512];
    int script, was_match;
    if (t_exec("assert(type(gd.stage_link)=='function'); "
               "assert(not pcall(gd.stage_link,1,2)); assert(not pcall(gd.stage_link,0,2)); "
               "assert(not pcall(gd.stage_link,1,2147483648));", out, sizeof out)) {
        gw_test_fail("stage_link registration or console validation failed: %s", out);
        return 1;
    }
    was_match = gs.match_active;
    gs.match_active = 1;
    script = t_model_owner_script("stage_link_fixture", "stage_link_fixture.lua", source);
    if (script >= 0) gs_unload(script);
    gs.match_active = was_match;
    if (script < 0) {
        gw_test_fail("stage_link Lua refusal did not return false, reason or reject invalid handles");
        return 1;
    }
    return 0;
}
