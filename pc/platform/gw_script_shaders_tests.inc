/* CPU-only Lua boundary checks for the real suite. GPU contracts use Dawn Null separately. */
static void t_shader_trim(char *s)
{
    size_t n = strlen(s);
    while (n && (s[n - 1] == 10 || s[n - 1] == 13)) s[--n] = 0;
}
static int test_script_shader_api(void)
{
    char out[4096];
    gs_init();
    t_exec("= type(gd.shader_load), type(gd.shader_set), type(gd.shader_status), "
           "type(gd.post_add), type(gd.post_set), type(gd.post_remove), type(gd.post_clear), "
           "type(gd.fx_shader), type(gd.light_set)", out, sizeof out);
    t_shader_trim(out); /* the console capture ends each printed value with a newline */
    if (strcmp(out, "function\nfunction\nfunction\nfunction\nfunction\nfunction\nfunction\nfunction\nfunction")) {
        gw_test_fail("shader APIs missing from gd: %s", out); return 1;
    }
    t_exec("= gd.post_set(2147483647,{params={amount=1}})", out, sizeof out);
    if (!strstr(out, "post handle expired or belongs to another script")) {
        gw_test_fail("expired shader post not reported: %s", out); return 1;
    }
    t_exec("= gd.light_set{dir={0,0,0},color={1,1,1},ambient={0,0,0}}", out, sizeof out);
    if (!strstr(out, "light direction must be nonzero")) {
        gw_test_fail("invalid visual light accepted: %s", out); return 1;
    }
    t_exec("= type(gd.perf().shaders), gd.perf().shaders.compiles ~= nil", out, sizeof out);
    t_shader_trim(out);
    if (strcmp(out, "table\ntrue")) { gw_test_fail("shader counters absent: %s", out); return 1; }
    return 0;
}
