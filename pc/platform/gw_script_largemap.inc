/* largemap: offline streaming API, kept separate for batch integration.
 * Builders run synchronously (cannot yield). They construct fresh handles on
 * each load; unload destroys live content rather than hiding reserved slots.
 * Immutable assets remain scene-pinned so a snapshot can resurrect an instance. */
extern int gw_ScriptGame_AreaFind(int owner);
extern int gw_ScriptGame_AreaBegin(int owner);
extern void gw_ScriptGame_AreaEnd(int success);
extern int gw_ScriptGame_AreaUnload(int owner);
extern void gw_ScriptGame_AreaUnloadOwner(int owner);
extern int gw_ScriptGame_StageStat(int field);
extern int gw_ScriptGame_MapBound(int which, int field);
extern void gw_ScriptGame_MapBounds(int which, int left, int right, int bottom, int top);
static char gs_area_name[48];
/* Script slots are reused on reload; owner tokens must not alias areas revived
 * by an older same-scene snapshot. Like object handles, this serial is native
 * and intentionally never rewound. */
static int gs_area_serial, gs_area_owners[GS_MAX_SCRIPTS];

static int gs_area_owner(lua_State *L)
{
    if (!gs_area_owners[gs.cur]) {
        if (gs_area_serial == INT_MAX) luaL_error(L, "area owner space exhausted");
        gs_area_owners[gs.cur] = ++gs_area_serial;
    }
    return gs_area_owners[gs.cur];
}

static void gs_area_retire(int script)
{
    if (gs_area_owners[script] && gs.match_active) {
        gs_rw_branch();
        gw_ScriptGame_AreaUnloadOwner(gs_area_owners[script]);
    }
    gs_area_owners[script] = 0;
}

int gw_Script_AreaNameByte(int index)
{
    return index >= 0 && index < sizeof gs_area_name ? (unsigned char)gs_area_name[index] : 0;
}

static void gs_area_arg(lua_State *L, const char *fn)
{
    size_t n;
    const char *name;
    gs_require_stage(L, fn);
    name = luaL_checklstring(L, 1, &n);
    if (!n || n >= sizeof gs_area_name || strlen(name) != n)
        luaL_error(L, "area name must be 1..47 bytes without NUL");
    memset(gs_area_name, 0, sizeof gs_area_name);
    memcpy(gs_area_name, name, n);
}

static int l_area_loaded(lua_State *L)
{
    gs_area_arg(L, "area_loaded");
    lua_pushboolean(L, gw_ScriptGame_AreaFind(gs_area_owner(L)));
    return 1;
}

static int l_area_load(lua_State *L)
{
    int area, status;
    gs_area_arg(L, "area_load");
    luaL_checktype(L, 2, LUA_TFUNCTION);
    gs_rw_branch();
    area = gw_ScriptGame_AreaBegin(gs_area_owner(L));
    if (area < 0) return luaL_error(L, area == -1 ? "area builders cannot nest" : "loaded area pool full (64)");
    if (!area) { lua_pushboolean(L, 0); return 1; }
    lua_pushvalue(L, 2);
    status = lua_pcall(L, 0, 0, 0);
    gw_ScriptGame_AreaEnd(status == LUA_OK);
    if (status != LUA_OK) return lua_error(L);
    lua_pushboolean(L, 1);
    return 1;
}

static int l_area_unload(lua_State *L)
{
    int result;
    gs_area_arg(L, "area_unload");
    gs_rw_branch();
    result = gw_ScriptGame_AreaUnload(gs_area_owner(L));
    if (result < 0) return luaL_error(L, "cannot unload areas inside a builder");
    lua_pushboolean(L, result);
    return 1;
}

static int l_stage_stats(lua_State *L)
{
    static const char *fields[] = {"lines", "instances", "targets", "areas",
        "line_capacity", "instance_capacity", "target_capacity", "state_bytes",
        "heap_free", "collision_bytes", "visible_instances"};
    int i;
    gs_require_stage(L, "stage_stats");
    lua_newtable(L);
    for (i = 0; i < sizeof fields / sizeof fields[0]; ++i)
        gs_setint(L, fields[i], gw_ScriptGame_StageStat(i));
    gs_setint(L, "assets", gs_stage_nmodels);
    gs_setint(L, "asset_bytes", gs_model_cache_bytes());
    gs_setint(L, "asset_budget", GS_MODEL_BYTE_BUDGET);
    return 1;
}

static int l_stage_bounds(lua_State *L)
{
    static const char *rects[] = {"camera", "blast"};
    static const char *fields[] = {"left", "right", "bottom", "top"};
    float value[2][4];
    int present[2] = {0}, i, j;
    gs_require_stage(L, "stage_bounds");
    if (lua_isboolean(L, 1) && !lua_toboolean(L, 1)) {
        gs_rw_branch(); gw_ScriptGame_MapBounds(-1, 0, 0, 0, 0);
    } else if (!lua_isnoneornil(L, 1)) {
        luaL_checktype(L, 1, LUA_TTABLE);
        /* Validate both complete rectangles before committing either one. */
        for (i = 0; i < 2; ++i) {
            lua_pushstring(L, rects[i]); lua_rawget(L, 1);
            if (!lua_isnil(L, -1)) {
                int at = lua_gettop(L);
                luaL_checktype(L, at, LUA_TTABLE); present[i] = 1;
                for (j = 0; j < 4; ++j) {
                    lua_pushstring(L, fields[j]); lua_rawget(L, at);
                    value[i][j] = gs_stage_num(L, -1); lua_pop(L, 1);
                }
                if (value[i][0] >= value[i][1] || value[i][2] >= value[i][3])
                    return luaL_error(L, "stage bounds require left < right and bottom < top");
            }
            lua_pop(L, 1);
        }
        gs_rw_branch();
        for (i = 0; i < 2; ++i) if (present[i])
            gw_ScriptGame_MapBounds(i, gs_fbits(value[i][0]), gs_fbits(value[i][1]),
                                   gs_fbits(value[i][2]), gs_fbits(value[i][3]));
    }
    lua_newtable(L);
    for (i = 0; i < 2; ++i) {
        lua_newtable(L);
        for (j = 0; j < 4; ++j)
            gs_setnum(L, fields[j], gs_camera_float(gw_ScriptGame_MapBound(i, j)));
        lua_setfield(L, -2, rects[i]);
    }
    return 1;
}
