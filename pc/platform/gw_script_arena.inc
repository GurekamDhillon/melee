/* arena-hooks: native Lua validation/registration helpers. All mutable arena
 * state (including ownership) lives in script_game.c's snapshotted BSS. */
extern int gw_ScriptGame_ArenaOwner(void);
extern void gw_ScriptGame_ArenaOrigin(int owner, int x, int y, int frames);
extern void gw_ScriptGame_ArenaSpawn(int owner, int slot, int x, int y);
extern float gw_ScriptGame_ArenaSpawnPos(int slot, int axis, float fallback);
extern void gw_ScriptGame_ArenaBounds(int owner, int blast, int l, int r, int t, int b);
extern void gw_ScriptGame_ArenaRestore(void);
extern int gw_ScriptGame_ArenaRead(int field);
extern int gw_ScriptGame_ArenaGroup(int id, int field);
extern void gw_ScriptGame_ArenaGroupSet(int owner, int id, int enabled);
extern void gw_ScriptGame_ArenaRelease(int owner);

static void gs_arena_require(lua_State *L, const char *fn) {
    int owner;
    gs_require_stage(L, fn);
    owner = gw_ScriptGame_ArenaOwner();
    if (owner && owner != gs.cur + 1)
        luaL_error(L, "gd.%s: bounds are owned by another script", fn);
}
static int l_stage_set_spawn(lua_State *L) {
    int slot = (int) luaL_checkinteger(L, 1);
    float x = gs_stage_num(L, 2), y = gs_stage_num(L, 3);
    gs_arena_require(L, "stage_set_spawn");
    if (slot < 0 || slot > 7)
        return luaL_error(L, "spawn slot must be 0..7 (starts 0-3, respawns 4-7)");
    gs_rw_branch();
    gw_ScriptGame_ArenaSpawn(gs.cur + 1, slot, gs_fbits(x), gs_fbits(y));
    gw_log("script arena-hooks: spawn %d -> %g %g", slot, x, y);
    lua_pushboolean(L, 1); return 1;
}
static int l_stage_spawn(lua_State *L) {
    int slot = (int) luaL_checkinteger(L, 1);
    if (slot < 0 || slot > 7)
        return luaL_error(L, "spawn slot must be 0..7 (starts 0-3, respawns 4-7)");
    lua_pushnumber(L, gw_ScriptGame_ArenaSpawnPos(slot, 0, 0.0f));
    lua_pushnumber(L, gw_ScriptGame_ArenaSpawnPos(slot, 1, 0.0f));
    lua_pushnumber(L, gw_ScriptGame_ArenaSpawnPos(slot, 2, 0.0f));
    return 3;
}
static int l_stage_set_origin(lua_State *L) {
    float x = gs_stage_num(L, 1), y = gs_stage_num(L, 2);
    lua_Integer frames = 0;
    gs_arena_require(L, "stage_set_origin");
    if (!lua_isnoneornil(L, 3)) {
        luaL_checktype(L, 3, LUA_TTABLE);
        lua_getfield(L, 3, "frames");
        frames = luaL_optinteger(L, -1, 0);
        lua_pop(L, 1);
    }
    if (frames < 0 || frames > INT_MAX) return luaL_error(L, "frames must be in 0..INT_MAX");
    gs_rw_branch();
    gw_ScriptGame_ArenaOrigin(gs.cur + 1, gs_fbits(x), gs_fbits(y), (int)frames);
    gw_log("script arena-hooks: origin %g %g frames=%d", x, y, (int)frames);
    lua_pushboolean(L, 1); return 1;
}
static int gs_arena_bounds(lua_State *L, int blast) {
    float l = gs_stage_num(L, 1), r = gs_stage_num(L, 2);
    float t = gs_stage_num(L, 3), b = gs_stage_num(L, 4);
    gs_arena_require(L, blast ? "stage_set_blast_bounds" : "stage_set_camera_bounds");
    if (l >= r || b >= t) return luaL_error(L, "bounds require left < right and bottom < top");
    gs_rw_branch();
    gw_ScriptGame_ArenaBounds(gs.cur + 1, blast, gs_fbits(l), gs_fbits(r), gs_fbits(t), gs_fbits(b));
    gw_log("script arena-hooks: %s bounds %g %g %g %g", blast ? "blast" : "camera", l, r, t, b);
    lua_pushboolean(L, 1); return 1;
}
static int l_stage_set_camera_bounds(lua_State *L) { return gs_arena_bounds(L, 0); }
static int l_stage_set_blast_bounds(lua_State *L) { return gs_arena_bounds(L, 1); }
static int l_stage_restore_bounds(lua_State *L) {
    gs_arena_require(L, "stage_restore_bounds");
    gs_rw_branch();
    gw_ScriptGame_ArenaRestore();
    gw_log("script arena-hooks: bounds restored");
    lua_pushboolean(L, 1); return 1;
}
static int l_stage_bounds_arena(lua_State *L) {
    static const char *keys[] = {"left", "right", "top", "bottom"};
    int b, k;
    if (!gs.match_active) { lua_pushnil(L); return 1; }
    lua_newtable(L);
    for (b = 0; b < 2; ++b) {
        lua_newtable(L);
        for (k = 0; k < 4; ++k)
            gs_setnum(L, keys[k], gs_camera_float(gw_ScriptGame_ArenaRead(b * 4 + k)));
        lua_setfield(L, -2, b ? "blast" : "camera");
    }
    lua_newtable(L);
    gs_setnum(L, "x", gs_camera_float(gw_ScriptGame_ArenaRead(8)));
    gs_setnum(L, "y", gs_camera_float(gw_ScriptGame_ArenaRead(9)));
    lua_setfield(L, -2, "origin");
    gs_setnum(L, "frames", gw_ScriptGame_ArenaRead(10));
    return 1;
}
static int l_stage_collision_groups(lua_State *L) {
    int id, count = gs.match_active ? gw_ScriptGame_ArenaGroup(-1, 0) : 0;
    lua_newtable(L);
    for (id = 0; id < count; ++id) {
        lua_newtable(L);
        gs_setnum(L, "id", id);
        gs_setbool(L, "enabled", gw_ScriptGame_ArenaGroup(id, 0));
        lua_rawseti(L, -2, id + 1);
    }
    return 1;
}
static int l_stage_collision_group(lua_State *L) {
    lua_Integer id = luaL_checkinteger(L, 1);
    int owner, enabled;
    gs_require_stage(L, "stage_collision_group");
    luaL_checktype(L, 2, LUA_TBOOLEAN);
    if (id < 0 || id >= gw_ScriptGame_ArenaGroup(-1, 0))
        return luaL_error(L, "collision group id is out of range");
    owner = gw_ScriptGame_ArenaGroup((int)id, 1);
    if (owner && owner != gs.cur + 1) return luaL_error(L, "collision group is owned by another script");
    enabled = lua_toboolean(L, 2);
    gs_rw_branch();
    gw_ScriptGame_ArenaGroupSet(gs.cur + 1, (int)id, enabled);
    gw_log("script arena-hooks: collision group %d enabled=%d", (int)id, enabled);
    lua_pushboolean(L, 1); return 1;
}
