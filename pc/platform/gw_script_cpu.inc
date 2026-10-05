/* Original script-owned native CPU technical overlay; no external AI import. */
static int l_cpu_technical(lua_State *L) {
    int slot = gs_slot_arg(L, 1);
    if (lua_isnoneornil(L, 2)) {
        static const char *const fields[] = {"skill", "opportunities", "lcancel_inputs",
                                           "tech_inputs", "missed_decisions", "last_event",
                                           "reaction_frames"};
        int i;
        lua_createtable(L, 0, 9);
        for (i = 0; i < 7; ++i)
            gs_setint(L, fields[i], gw_ScriptGame_CpuTechnicalState(slot, gs.cur + 1, i));
        gs_setbool(L, "enabled", gw_ScriptGame_CpuTechnicalState(slot, gs.cur + 1, 0) != 0);
        gs_setstr(L, "policy", "original landing-cancel / hitstun ground-tech assist");
        return 1;
    } else {
        lua_Integer skill = luaL_checkinteger(L, 2);
        lua_Integer seed = luaL_optinteger(L, 3, 1);
        if (skill < 0 || skill > 3 || seed < 1 || seed > 2147483647)
            return luaL_error(L, "cpu_technical: skill 0..3 and seed 1..2147483647 required");
        gs_require_offline(L, "cpu_technical");
        gs_rw_branch();
        lua_pushboolean(L, gw_ScriptGame_CpuTechnical(slot, gs.cur + 1, (int)skill, (int)seed));
        return 1;
    }
}


/* gd.cpu_assist(port, {lcancel=0..1, perfect_shield=0..1, tech=0..1, tech_dir="in"|"away"|"toward"|"random",
 * wavedash=0..1, fast_fall=0..1, seed=n} | nil): the per-entity technique assist applied on top of the retail
 * AI (src/melee/ft/cpu_assist.inc). gd.cpu_assist(port) reads it back. Probabilities cross the native boundary as
 * permille. Offline only; the calling script owns it, scene end and script unload clear it. */
extern int gw_ScriptGame_CpuAssist(int slot, int owner, int lc, int ps, int tech, int tdir, int wd, int ff, int seed);
extern int gw_ScriptGame_CpuAssistClearSlot(int slot, int owner);
extern int gw_ScriptGame_CpuAssistState(int slot, int owner, int field);
extern int gw_ScriptGame_CpuAssistTest(void);
static const char *const gs_assist_names[5] = {"lcancel", "perfect_shield", "tech", "wavedash", "fast_fall"};
static const char *const gs_assist_dirs[4] = {"in", "away", "toward", "random"};
static int gs_assist_prob(lua_State *L, int t, const char *key) {
    lua_Number v;
    lua_getfield(L, t, key);
    if (lua_isnil(L, -1)) { lua_pop(L, 1); return 0; }
    if (!lua_isnumber(L, -1)) return luaL_error(L, "cpu_assist: %s must be a number 0..1", key);
    v = lua_tonumber(L, -1);
    lua_pop(L, 1);
    if (!(v >= 0.0 && v <= 1.0)) return luaL_error(L, "cpu_assist: %s must be a number 0..1", key);
    return (int)(v * 1000.0 + 0.5);
}
static int l_cpu_assist(lua_State *L) {
    int slot = gs_slot_arg(L, 1), owner = gs.cur + 1, i;
    if (lua_gettop(L) < 2) {
        lua_createtable(L, 0, 8);
        gs_setbool(L, "enabled", gw_ScriptGame_CpuAssistState(slot, owner, 0) != 0);
        if (gw_ScriptGame_CpuAssistState(slot, owner, 0)) {
            gs_setint(L, "seed", gw_ScriptGame_CpuAssistState(slot, owner, 7));
            gs_setstr(L, "tech_dir", gs_assist_dirs[gw_ScriptGame_CpuAssistState(slot, owner, 6) & 3]);
        }
        lua_createtable(L, 0, 5); /* probabilities */
        for (i = 0; i < 5; ++i) {
            lua_pushnumber(L, gw_ScriptGame_CpuAssistState(slot, owner, 1 + i) / 1000.0);
            lua_setfield(L, -2, gs_assist_names[i]);
        }
        lua_setfield(L, -2, "probability");
        lua_createtable(L, 0, 5);
        for (i = 0; i < 5; ++i) {
            lua_createtable(L, 0, 2);
            gs_setint(L, "opportunities", gw_ScriptGame_CpuAssistState(slot, owner, 8 + i));
            gs_setint(L, "performed", gw_ScriptGame_CpuAssistState(slot, owner, 13 + i));
            lua_setfield(L, -2, gs_assist_names[i]);
        }
        lua_setfield(L, -2, "counters");
        return 1;
    }
    gs_require_offline(L, "cpu_assist");
    gs_rw_branch();
    if (lua_isnil(L, 2)) {
        lua_pushboolean(L, gw_ScriptGame_CpuAssistClearSlot(slot, owner));
        return 1;
    }
    luaL_checktype(L, 2, LUA_TTABLE);
    {
        int lc = gs_assist_prob(L, 2, "lcancel"), ps = gs_assist_prob(L, 2, "perfect_shield"),
            tech = gs_assist_prob(L, 2, "tech"), wd = gs_assist_prob(L, 2, "wavedash"),
            ff = gs_assist_prob(L, 2, "fast_fall"), tdir = 1;
        lua_Integer seed = 1;
        lua_getfield(L, 2, "tech_dir");
        if (!lua_isnil(L, -1)) {
            const char *d = lua_type(L, -1) == LUA_TSTRING ? lua_tostring(L, -1) : NULL;
            for (tdir = -1, i = 0; d && i < 4; ++i) if (!strcmp(d, gs_assist_dirs[i])) tdir = i;
            if (tdir < 0) return luaL_error(L, "cpu_assist: tech_dir is \"in\", \"away\", \"toward\" or \"random\"");
        }
        lua_pop(L, 1);
        lua_getfield(L, 2, "seed");
        if (!lua_isnil(L, -1)) {
            if (!lua_isinteger(L, -1)) return luaL_error(L, "cpu_assist: seed must be an integer 1..2147483647");
            seed = lua_tointeger(L, -1);
            if (seed < 1 || seed > 2147483647) return luaL_error(L, "cpu_assist: seed must be an integer 1..2147483647");
        }
        lua_pop(L, 1);
        lua_pushboolean(L, gw_ScriptGame_CpuAssist(slot, owner, lc, ps, tech, tdir, wd, ff, (int)seed));
    }
    return 1;
}
