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

