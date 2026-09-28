/* Native half of the runtime model API. Only immutable assets live here; the
 * request buffers are read synchronously and copied by ScriptGame_Model*. */
extern int gw_ScriptGame_ModelRef(int asset, int token, int delta);
extern int gw_ScriptGame_ModelSlot(int handle);
extern int gw_ScriptGame_ModelField(int slot, int field);
extern int gw_ScriptGame_ModelSpawn(int asset, int token, int handle, int count, int first_line);
extern int gw_ScriptGame_ModelSet(int handle);
extern int gw_ScriptGame_ModelDespawn(int handle);
extern int gw_ScriptGame_ModelLineOwner(int handle);
static int gs_model_input[SM_FIELDS];
static const GmCollision *gs_model_collision;
static int gs_model_floor_flags = -1; /* spawn option: overrides every floor line's flags */

int gw_Script_ModelInput(int field, int line)
{
    const GmLine *l;
    if (line < 0) return field >= 0 && field < SM_FIELDS ? gs_model_input[field] : 0;
    if (!gs_model_collision || line >= gs_model_collision->count) return 0;
    l = &gs_model_collision->line[line];
    switch (field) {
    case 0: return gs_fbits(l->x0); case 1: return gs_fbits(l->y0);
    case 2: return gs_fbits(l->x1); case 3: return gs_fbits(l->y1);
    case 4: return l->kind;
    case 5: return l->kind == 1 && gs_model_floor_flags >= 0 ? gs_model_floor_flags : l->flags;
    default: return 0;
    }
}

void gw_Script_ModelDraw(int slot, const void *view)
{
    float local[3][4], scale, rot, c, s;
    int asset;
    if (slot < 0) { gs_model_bound = NULL; return; }
    if (!gw_ScriptGame_ModelField(slot, -1)) return;
    asset = gw_ScriptGame_ModelField(slot, -2);
    scale = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_SCALE));
    rot = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_ROT)) * 0.017453292519943295f;
    c = cosf(rot) * scale; s = sinf(rot) * scale;
    memset(local, 0, sizeof local);
    local[0][0] = c; local[0][1] = -s; local[1][0] = s; local[1][1] = c;
    local[2][2] = scale;
    local[0][3] = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_X));
    local[1][3] = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_Y));
    local[2][3] = gs_camera_float(gw_ScriptGame_ModelField(slot, SM_Z));
    gs_model_draw(asset + 1, view, local, (unsigned)gw_ScriptGame_ModelField(slot, SM_TINT));
}

static const char *gs_model_fields[SM_FIELDS] = { "x", "y", "z", "rot", "scale", "layer", "visible", "tint" };
static void gs_model_options(lua_State *L, int at, int field[SM_FIELDS])
{
    int i;
    if (lua_isnoneornil(L, at)) return;
    luaL_checktype(L, at, LUA_TTABLE);
    for (i = 0; i < SM_FIELDS; ++i) {
        /* Raw fields avoid arbitrary script callbacks during a transaction. */
        lua_pushstring(L, gs_model_fields[i]); lua_rawget(L, at);
        if (!lua_isnil(L, -1)) {
            if (i <= SM_SCALE) {
                float v = gs_stage_num(L, -1);
                if (i == SM_SCALE && (v < 0.001f || v > 100)) luaL_error(L, "model scale must be 0.001..100");
                if (i == SM_ROT && (v < -360 || v > 360)) luaL_error(L, "model rot must be -360..360 degrees");
                field[i] = gs_fbits(v);
            } else if (i == SM_VISIBLE) {
                luaL_checktype(L, -1, LUA_TBOOLEAN); field[i] = lua_toboolean(L, -1);
            } else {
                lua_Integer v = luaL_checkinteger(L, -1);
                if (i == SM_LAYER && (v < -8 || v > 8)) luaL_error(L, "model layer must be -8..8");
                if (i == SM_TINT && (v < 0 || (lua_Unsigned)v > 0xFFFFFFFFu)) luaL_error(L, "model tint must be RGBA 0x00000000..0xFFFFFFFF");
                field[i] = (int)(unsigned)v;
            }
        }
        lua_pop(L, 1);
    }
}
static int gs_model_asset(lua_State *L, int at)
{
    int token = gs_stage_handle_arg(L, at), i;
    for (i = 0; i < gs_stage_nmodels; ++i)
        if (gs_stage_models[i].token == token && gw_ScriptGame_ModelRef(i, token, 0)) return i;
    return luaL_error(L, "stale or released model handle");
}
static int l_model_load(lua_State *L)
{
    int i;
    const char *path = luaL_checkstring(L, 1);
    if (strlen(path) != lua_rawlen(L, 1)) return luaL_error(L, "model path contains NUL");
    gs_require_stage(L, "model_load");
    i = gs_model_open(L, path) - 1;
    gs_rw_branch();
    if (!gw_ScriptGame_ModelRef(i, gs_stage_models[i].token, 1)) return luaL_error(L, "model reference limit exceeded");
    gw_log("script model: retain asset=%d handle=%d", i, gs_stage_models[i].token);
    lua_pushinteger(L, gs_stage_models[i].token); return 1;
}
static int l_model_release(lua_State *L)
{
    int i;
    gs_require_stage(L, "model_release"); i = gs_model_asset(L, 1);
    gs_rw_branch();
    gw_ScriptGame_ModelRef(i, gs_stage_models[i].token, -1);
    gw_log("script model: release handle=%d (scene pin retained)", gs_stage_models[i].token);
    return 0;
}
static int l_model_spawn(lua_State *L)
{
    int i, handle, result, first_line, count, floor_flags = -1, field[SM_FIELDS] = {0};
    gs_require_stage(L, "model_spawn"); i = gs_model_asset(L, 1);
    field[SM_SCALE] = gs_fbits(1); field[SM_VISIBLE] = 1; field[SM_TINT] = -1;
    gs_model_options(L, 2, field);
    count = gs_stage_models[i].collision.count;
    if (lua_istable(L, 2)) {
        /* Spawn-only: collision = false places the model visual-only; floor_flags (0..3:
         * 1 pass-through, 2 ledges) replaces the sidecar's flags on every floor line. */
        lua_pushstring(L, "collision"); lua_rawget(L, 2);
        if (!lua_isnil(L, -1)) { luaL_checktype(L, -1, LUA_TBOOLEAN); if (!lua_toboolean(L, -1)) count = 0; }
        lua_pop(L, 1);
        lua_pushstring(L, "floor_flags"); lua_rawget(L, 2);
        if (!lua_isnil(L, -1)) {
            lua_Integer v = luaL_checkinteger(L, -1);
            if (v < 0 || v > 3) luaL_error(L, "model floor_flags must be 0..3");
            floor_flags = (int)v;
        }
        lua_pop(L, 1);
    }
    if (gs_stage_handle_serial > INT_MAX - SCRIPT_MESH_LINES - 1)
        return luaL_error(L, "model instance handle space exhausted");
    handle = gs_stage_next_handle(L); first_line = gs_stage_handle_serial + 1;
    gs_stage_handle_serial += count;
    gs_rw_branch(); memcpy(gs_model_input, field, sizeof field);
    gs_model_collision = &gs_stage_models[i].collision; gs_model_floor_flags = floor_flags;
    result = gw_ScriptGame_ModelSpawn(i, gs_stage_models[i].token, handle, count, first_line);
    gs_model_collision = NULL; gs_model_floor_flags = -1;
    if (result < 0) {
        lua_pushnil(L);
        lua_pushstring(L, result == -2 ? "model instance pool full (128)" : result == -3 ?
                       "collision line capacity unavailable" : result == -4 ?
                       "model transform reverses collision kind or exceeds world bounds" : "model scene renderer unavailable");
        return 2;
    }
    lua_pushinteger(L, result); return 1;
}
static int gs_model_update(lua_State *L, int move)
{
    int handle, slot, i, result, field[SM_FIELDS];
    gs_require_stage(L, move ? "model_move" : "model_set");
    handle = gs_stage_handle_arg(L, 1); slot = gw_ScriptGame_ModelSlot(handle);
    if (slot < 0) return luaL_error(L, "stale model instance");
    for (i = 0; i < SM_FIELDS; ++i) field[i] = gw_ScriptGame_ModelField(slot, i);
    if (move && !lua_istable(L, 2)) {
        field[SM_X] = gs_fbits(gs_stage_num(L, 2)); field[SM_Y] = gs_fbits(gs_stage_num(L, 3));
        if (!lua_isnoneornil(L, 4)) field[SM_Z] = gs_fbits(gs_stage_num(L, 4));
    } else { luaL_checktype(L, 2, LUA_TTABLE); gs_model_options(L, 2, field); }
    gs_rw_branch(); memcpy(gs_model_input, field, sizeof field);
    result = gw_ScriptGame_ModelSet(handle);
    if (result < 0) return luaL_error(L, "model transform reverses collision kind or exceeds world bounds");
    lua_pushboolean(L, result); return 1;
}
static int l_model_move(lua_State *L) { return gs_model_update(L, 1); }
static int l_model_set(lua_State *L) { return gs_model_update(L, 0); }
static int l_model_despawn(lua_State *L)
{
    int h = gs_stage_handle_arg(L, 1);
    gs_require_stage(L, "model_despawn"); gs_rw_branch();
    lua_pushboolean(L, gw_ScriptGame_ModelDespawn(h)); return 1;
}
static int l_model_get(lua_State *L)
{
    int slot, i;
    gs_require_stage(L, "model_get"); slot = gw_ScriptGame_ModelSlot(gs_stage_handle_arg(L, 1));
    if (slot < 0) { lua_pushnil(L); return 1; }
    lua_newtable(L);
    for (i = 0; i < SM_FIELDS; ++i) {
        int v = gw_ScriptGame_ModelField(slot, i);
        if (i <= SM_SCALE) lua_pushnumber(L, gs_camera_float(v));
        else if (i == SM_VISIBLE) lua_pushboolean(L, v);
        else lua_pushinteger(L, i == SM_TINT ? (lua_Integer)(unsigned)v : v);
        lua_setfield(L, -2, gs_model_fields[i]);
    }
    return 1;
}
