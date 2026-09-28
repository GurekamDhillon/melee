/* Headless storage/guard checks, registered in gw_script.c. Actual save/load,
 * PPC retarget linking and room assets still require the Windows lane. */
static int test_script_mode_blob(void) {
    lua_State *L;
    int script, saved_cur = gs.cur, saved_active = gs.match_active;
    int saved_depth = gs.rw_frames, rc = 1, slot, i;
    char bytes[SCRIPT_MODE_BYTES + 1], registration[128];
    const char *out, *why = NULL;
    size_t len;
    GsSaveSlot saved;
    if (t_exec("= type(gd.mode_blob)", registration, sizeof registration) ||
        strcmp(registration, "function\n")) {
        gw_test_fail("mode blob API registration: %s", registration); return 1;
    }
    if (gw_ScriptGame_ModeActive() || gs.rw_began) {
        gw_test_fail("mode blob test requires an empty headless scene"); return 1;
    }
    L = luaL_newstate();
    if (!L) return 1;
    script = gs_alloc_script();
    if (script < 0) { lua_close(L); return 1; }
    gs.s[script].used = gs.s[script].gameplay = 1;
    gs.s[script].src_hash = 0xABCDEF0123456789ull;
    snprintf(gs.s[script].id, sizeof gs.s[script].id, "mode_blob_test");
    gs.cur = script; gs.match_active = 1; gs.rw_frames = 0;
    for (i = 0; i < (int)sizeof bytes; ++i) bytes[i] = (char)i; /* includes NUL/high bits */
#define MODE_CHECK(test) do { if (!(test)) { gw_test_fail("mode blob check at line %d", __LINE__); goto done; } } while (0)
    lua_pushcfunction(L, l_mode_blob);
    lua_pushlstring(L, bytes, SCRIPT_MODE_BYTES);
    MODE_CHECK(lua_pcall(L, 1, 1, 0) == LUA_OK); lua_pop(L, 1);
    MODE_CHECK(gw_ScriptGame_ModeActive());
    gs_slot_capture(&saved);
    MODE_CHECK(saved.mode_session == gs_mode_session());
    MODE_CHECK(saved.mode_scripts && saved.mode_scripts == gs_mode_identity());
    gs.s[script].src_hash ^= 1;
    MODE_CHECK(saved.mode_scripts != gs_mode_identity());
    gs.s[script].src_hash ^= 1;
    slot = gs_mode_slot(gs.s[script].src_hash); MODE_CHECK(slot >= 0);
    MODE_CHECK(gs_mode_slot(gs.s[script].src_hash ^ 1) == -1);
    lua_pushcfunction(L, l_mode_blob);
    MODE_CHECK(lua_pcall(L, 0, 1, 0) == LUA_OK);
    out = lua_tolstring(L, -1, &len);
    MODE_CHECK(out && len == SCRIPT_MODE_BYTES && !memcmp(out, bytes, len)); lua_pop(L, 1);
    lua_pushcfunction(L, l_mode_blob); lua_pushlstring(L, bytes, sizeof bytes);
    MODE_CHECK(lua_pcall(L, 1, 1, 0) != LUA_OK); lua_pop(L, 1);
    MODE_CHECK(gw_ScriptGame_ModeRead(slot, 0) == SCRIPT_MODE_BYTES + 1);
    MODE_CHECK(gs_rw_request(0, &why) != 0 && why && strstr(why, "mode state"));
    lua_pushcfunction(L, l_mode_blob); lua_pushliteral(L, "x");
    MODE_CHECK(lua_pcall(L, 1, 1, 0) == LUA_OK); lua_pop(L, 1);
    MODE_CHECK(gw_ScriptGame_ModeRead(slot, 3) == 'x');
    for (i = 4; i < SCRIPT_MODE_WORDS; ++i) MODE_CHECK(!gw_ScriptGame_ModeRead(slot, i));
    gs.cur = gs.console;
    lua_pushcfunction(L, l_mode_blob);
    MODE_CHECK(lua_pcall(L, 0, 1, 0) != LUA_OK); lua_pop(L, 1);
    gs.cur = script;
    lua_pushcfunction(L, l_mode_blob); lua_pushliteral(L, "");
    MODE_CHECK(lua_pcall(L, 1, 1, 0) == LUA_OK); lua_pop(L, 1);
    MODE_CHECK(!gw_ScriptGame_ModeActive());
    rc = 0;
done:
    gs.s[script].src_hash = 0xABCDEF0123456789ull;
    gs_mode_drop(script);
    memset(&gs.s[script], 0, sizeof gs.s[script]);
    gs.s[script].env_ref = LUA_NOREF;
    gs.cur = saved_cur; gs.match_active = saved_active; gs.rw_frames = saved_depth;
    lua_close(L);
#undef MODE_CHECK
    return rc;
}
