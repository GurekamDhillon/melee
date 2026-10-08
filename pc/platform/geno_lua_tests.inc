/* geno_lua_tests.inc - slice 5, registry half: the "lua" block's parse, refusals, id and the scalar functions.
 * (The sandbox itself is geno_lua_core.h's: `build.sh --native-test geno-lua`. The game half is pc/geno/geno_lua_tests.inc.) */

#define GNL_MODULE \
    "local M = {} " \
    "function M.enter(ctx) ctx.state.charge = 0 end " \
    "function M.frame(ctx) local s = ctx.state " \
    "if ctx.input.special_held and s.charge < 60 then s.charge = s.charge + 1 s.armed = s.charge >= 3 else ctx.go('Release') end end " \
    "function M.rel(ctx) ctx.velocity(1.6 + 0.03 * ctx.state.charge, 0) end " \
    "function M.relf(ctx) ctx.hitbox_damage(3, 6 + 0.15 * ctx.state.charge) end " \
    "return M"

/* one define with a lua block; `lua` replaces the block text, `states` the states array text, version is the file's */
static void gnl_json(char *out, size_t cap, int version, const char *key, const char *lua, const char *states, const char *tail)
{
    snprintf(out, cap,
             "{\"geno\":%d,\"fighters\":[{\"define\":{\"key\":\"%s\",\"name\":\"Lua T\",\"base\":\"mario\",\"common\":\"melee.common.v1\","
             "\"resources\":\"retail:mario\"}%s%s%s%s}]}",
             version, key, lua ? ",\"lua\":" : "", lua ? lua : "", states ? ",\"states\":" : "", states ? states : "", tail ? tail : "");
}

#define GNL_BLOCK "{\"source\":\"" GNL_MODULE "\",\"state\":{\"charge\":\"int\",\"armed\":\"bool\"}}"
#define GNL_STATES \
    "[{\"name\":\"Charge\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"enter\",\"frame\":\"frame\"}}," \
    "{\"name\":\"Release\",\"behavior\":\"geno.ground\",\"lua\":{\"enter\":\"rel\",\"frame\":\"relf\"}}]"

static int gnl_refuses(const char *what, const char *text)
{
    static gn_registry r;
    int n;
    gn_registry_clear(&r);
    n = gn_parse_text(&r, text, "lua-test");
    if (n != 0 || r.n != 0) {
        gw_test_fail("fighter Lua: %s must be refused (took %d entries)", what, r.n);
        gn_registry_clear(&r);
        return 1;
    }
    gn_registry_clear(&r);
    return 0;
}

static int test_geno_lua_registry(void)
{
    static gn_registry r;
    char text[4096], other[4096];
    const gn_profile *p;
    uint64_t id1, id2;
    int rc = 0;
    gn_registry_clear(&r);
    gnl_json(text, sizeof text, 10, "lua-t", GNL_BLOCK, GNL_STATES, ",\"specials\":{\"n\":\"geno:Charge\"}");
    if (gn_parse_text(&r, text, "lua-test") != 1 || r.n != 1) {
        gw_test_fail("fighter Lua: a good v9 define was not taken");
        gn_registry_clear(&r);
        return 1;
    }
    p = &r.p[0];
    if (p->lua == NULL || p->lua->nslot != 2 || strcmp(p->lua->slot_name[0], "charge") != 0 || p->lua->slot_type[0] != GLUA_INT ||
        p->lua->slot_type[1] != GLUA_BOOL) {
        gw_test_fail("fighter Lua: the state layout was not read");
        rc = 1;
    }
    if (p->st_lua[0][GENO_LUA_PHASE_ENTER] < 0 || p->st_lua[0][GENO_LUA_PHASE_FRAME] < 0 || p->st_lua[1][GENO_LUA_PHASE_FRAME] < 0 ||
        p->st_lua[0][GENO_LUA_PHASE_FRAME] == p->st_lua[1][GENO_LUA_PHASE_FRAME]) {
        gw_test_fail("fighter Lua: the per-state functions were not bound");
        rc = 1;
    }
    if (p->st_cb[0][GENO_CB_ANIM] < 0) {
        gw_test_fail("fighter Lua: a state with a \"frame\" function must get the lua anim callback");
        rc = 1;
    }
    id1 = p->id;
    gn_registry_clear(&r);
    /* the module is content: the same define with one number changed has another id; a define with no lua block is untouched */
    {
        static char changed[2048];
        const char *at = strstr(GNL_BLOCK, "< 60");
        snprintf(changed, sizeof changed, "%.*s< 61%s", (int) (at - GNL_BLOCK), GNL_BLOCK, at + 4);
        gnl_json(other, sizeof other, 10, "lua-t", changed, GNL_STATES, ",\"specials\":{\"n\":\"geno:Charge\"}");
    }
    if (gn_parse_text(&r, other, "lua-test") != 1) {
        gw_test_fail("fighter Lua: the changed module was not taken");
        rc = 1;
    } else {
        id2 = r.p[0].id;
        if (id1 == id2) {
            gw_test_fail("fighter Lua: the id does not cover the module text");
            rc = 1;
        }
    }
    gn_registry_clear(&r);
    gnl_json(text, sizeof text, 10, "lua-t", NULL, NULL, NULL);
    if (gn_parse_text(&r, text, "lua-test") != 1 || r.p[0].lua != NULL) {
        gw_test_fail("fighter Lua: a define with no lua block must have no domain");
        rc = 1;
    }
    gn_registry_clear(&r);
    /* refusals (strict, like the rest of a define) */
    gnl_json(text, sizeof text, 8, "lua-t", GNL_BLOCK, GNL_STATES, NULL);
    rc |= gnl_refuses("lua in a geno 8 file", text);
    gnl_json(text, sizeof text, 9, "lua-t", GNL_BLOCK, GNL_STATES, NULL);
    rc |= gnl_refuses("lua in a geno 9 file (the format number is 10)", text);
    snprintf(text, sizeof text, "{\"geno\":10,\"fighters\":[{\"attach\":\"kirby\",\"lua\":{\"source\":\"return {}\"}}]}");
    rc |= gnl_refuses("lua on an attach entry", text);
    gnl_json(text, sizeof text, 10, "lua-t", NULL, GNL_STATES, NULL);
    rc |= gnl_refuses("state lua functions with no lua block", text);
    gnl_json(text, sizeof text, 10, "lua-t", "{\"source\":\"return {f=function(ctx) end}\",\"script\":\"lua/x.lua\"}", NULL, NULL);
    rc |= gnl_refuses("both script and source", text);
    gnl_json(text, sizeof text, 10, "lua-t", "{}", NULL, NULL);
    rc |= gnl_refuses("neither script nor source", text);
    gnl_json(text, sizeof text, 10, "lua-t", "{\"script\":\"../x.lua\"}", NULL, NULL);
    rc |= gnl_refuses("a script path that leaves the mod", text);
    gnl_json(text, sizeof text, 10, "lua-t", "{\"source\":\"return {f=function(ctx) end}\",\"state\":{\"a\":\"string\"}}", NULL, NULL);
    rc |= gnl_refuses("a state slot of an unknown type", text);
    gnl_json(text, sizeof text, 10, "lua-t", "{\"source\":\"return {f=function(ctx) end}\",\"state\":{\"a\":\"int\",\"a\":\"int\"}}", NULL, NULL);
    rc |= gnl_refuses("a slot declared twice", text);
    {
        char many[1024] = "{\"source\":\"return {f=function(ctx) end}\",\"state\":{";
        int i;
        for (i = 0; i < GENO_LUA_STATE_SLOTS + 1; ++i) {
            char one[32];
            snprintf(one, sizeof one, "%s\"s%d\":\"int\"", i ? "," : "", i);
            strcat(many, one);
        }
        strcat(many, "}}");
        gnl_json(text, sizeof text, 10, "lua-t", many, NULL, NULL);
        rc |= gnl_refuses("more than 16 slots", text);
    }
    gnl_json(text, sizeof text, 10, "lua-t", "{\"source\":\"local n = 0 return {f=function(ctx) n = n + 1 end}\"}", NULL, NULL);
    rc |= gnl_refuses("a module that keeps state in a captured variable", text);
    gnl_json(text, sizeof text, 10, "lua-t", "{\"source\":\"return {f=function(ctx) end}\"}",
             "[{\"name\":\"A\",\"behavior\":\"geno.ground\",\"lua\":{\"frame\":\"nope\"}}]", NULL);
    rc |= gnl_refuses("a state naming a function the module lacks", text);
    gnl_json(text, sizeof text, 10, "lua-t", "{\"source\":\"return {f=function(ctx) ctx.go('A') end}\"}", NULL, NULL);
    /* (an unknown ctx.go target is a run-time fault, not a load error: the module text is not executed at load) */
    gn_registry_clear(&r);
    if (gn_parse_text(&r, text, "lua-test") != 1) {
        gw_test_fail("fighter Lua: a module whose go() target is unknown is a run-time fault, not a load error");
        rc = 1;
    }
    gn_registry_clear(&r);
    return rc;
}

/* the scalar functions the game half calls, through the boot registry (the way a fighter reaches them) */
static int test_geno_lua_call(void)
{
    char text[4096];
    uint32_t io[GENO_LUA_IO_WORDS];
    int rc = 0, c;
    gnl_json(text, sizeof text, 10, "lua-t", GNL_BLOCK, GNL_STATES, ",\"specials\":{\"n\":\"geno:Charge\"}");
    if (gw_Geno_TestInstall(text) != 1) {
        gw_test_fail("fighter Lua: could not install the test define");
        gw_Geno_TestRestore();
        return 1;
    }
    c = 0; /* the test registry replaces the boot one: the entry is profile 0 */
    if (gw_Geno_LuaSlots(c) != 2 || gw_Geno_LuaSlotType(c, 1) != GLUA_BOOL || strcmp(gw_Geno_LuaSlotName(c, 0), "charge") != 0) {
        gw_test_fail("fighter Lua: slot accessors");
        rc = 1;
    }
    if (!gw_Geno_LuaFn(c, 0, GENO_LUA_PHASE_FRAME) || !gw_Geno_LuaFn(c, 1, GENO_LUA_PHASE_ENTER) || gw_Geno_LuaFn(c, 2, 0) || gw_Geno_LuaFn(c, 0, 5) ||
        gw_Geno_LuaFn(99, 0, 0)) {
        gw_test_fail("fighter Lua: gw_Geno_LuaFn answers");
        rc = 1;
    }
    memset(io, 0, sizeof io);
    gw_w32(&io[GENO_LUA_IO_HELD], GENO_BTN_SPECIAL);
    gw_w32(&io[GENO_LUA_IO_STATE], 2);
    if (gw_Geno_LuaCall(c, 0, GENO_LUA_PHASE_FRAME, io) != 0 || (int32_t) gw_r32(&io[GENO_LUA_IO_STATE]) != 3 || gw_r32(&io[GENO_LUA_IO_STATE + 1]) != 1) {
        gw_test_fail("fighter Lua: a frame call must count the charge (2 -> 3) and arm");
        rc = 1;
    }
    memset(io, 0, sizeof io);
    gw_w32(&io[GENO_LUA_IO_STATE], 9);
    if (gw_Geno_LuaCall(c, 0, GENO_LUA_PHASE_FRAME, io) != 0 || gw_r32(&io[GENO_LUA_IO_NCMDS]) != 1 || gw_r32(&io[GENO_LUA_IO_CMDS]) != GENO_LUA_CMD_GO ||
        gw_r32(&io[GENO_LUA_IO_CMDS + 1]) != GENO_TARGET(GENO_TGT_GENO, 1)) {
        gw_test_fail("fighter Lua: released: a go to state 1");
        rc = 1;
    }
    if (gw_Geno_LuaCall(c, 2, 0, io) != GENO_LUA_NONE) {
        gw_test_fail("fighter Lua: a state with no function is GENO_LUA_NONE, not a fault");
        rc = 1;
    }
    gw_Geno_TestRestore();
    if (gw_Geno_LuaFn(0, 0, 0)) {
        gw_test_fail("fighter Lua: the boot registry must have no Lua after the restore");
        rc = 1;
    }
    return rc;
}

/* Slice 7: the online fault policy ("hard") ends a match only on a CONFIRMED frame, and a rollback forgets a fault noted in the undone stretch. */
static int test_geno_lua_fault_policy(void)
{
    int rc = 0;
    gw_Geno_FaultSetHard(1);
    if (gw_Geno_FaultConfirmed(1000)) { gw_test_fail("fault policy: nothing noted, nothing confirmed"); rc = 1; }
    gw_Geno_FaultNote(500);
    gw_Geno_FaultNote(520); /* the earliest frame counts */
    if (gw_Geno_FaultConfirmed(499)) { gw_test_fail("fault policy: a fault ahead of the confirmed frame must not end the match"); rc = 1; }
    if (!gw_Geno_FaultConfirmed(500)) { gw_test_fail("fault policy: a fault on a confirmed frame ends the match"); rc = 1; }
    gw_Geno_FaultRolledBack(); /* the rollback loaded a snapshot before the fault */
    if (gw_Geno_FaultConfirmed(9999)) { gw_test_fail("fault policy: a rollback forgets a fault from the undone stretch"); rc = 1; }
    gw_Geno_FaultSetHard(0);
    return rc;
}
