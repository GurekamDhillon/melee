/* Local visual APIs. Never call gs_rw_branch or write game memory. */
#include "gw_shader.h"
static int gs_shader_owner(void)
{
    GsScript *s = gs_cur_script();
    return s ? s->stage_owner : 0;
}
static int gs_shader_error(lua_State *L, const char *error)
{
    lua_pushnil(L); lua_pushstring(L, error); return 2;
}
static int gs_shader_params(lua_State *L, int idx, char out[8192])
{
    size_t used = 1;
    int first = 1;
    out[0] = '{'; out[1] = 0;
    if (lua_isnoneornil(L, idx)) { strcpy(out, "{}"); return 1; }
    if (!lua_istable(L, idx)) return 0;
    idx = lua_absindex(L, idx); lua_pushnil(L);
    while (lua_next(L, idx)) {
        const char *key; size_t len;
        int n, k;
        if (lua_type(L, -2) != LUA_TSTRING) { lua_pop(L, 2); return 0; }
        key = lua_tolstring(L, -2, &len);
        if (!len || len > 48 || strlen(key) != len || strchr(key, '"') || strchr(key, '\\') || used > 7000) { lua_pop(L, 2); return 0; }
        n = snprintf(out + used, 8192 - used, "%s\"%s\":", first ? "" : ",", key);
        if (n < 0 || (size_t)n >= 8192 - used) { lua_pop(L, 2); return 0; }
        used += (size_t)n; first = 0;
        if (lua_type(L, -1) == LUA_TNUMBER) {
            double value = lua_tonumber(L, -1);
            if (!isfinite(value) || fabs(value) > 100000) { lua_pop(L, 2); return 0; }
            n = snprintf(out + used, 8192 - used, "%.9g", value);
        } else if (lua_istable(L, -1) && lua_rawlen(L, -1) == 4) {
            double v[4];
            for (k = 0; k < 4; ++k) {
                lua_rawgeti(L, -1, k + 1);
                if (lua_type(L, -1) != LUA_TNUMBER || !isfinite(v[k] = lua_tonumber(L, -1)) || fabs(v[k]) > 100000) { lua_pop(L, 3); return 0; }
                lua_pop(L, 1);
            }
            n = snprintf(out + used, 8192 - used, "[%.9g,%.9g,%.9g,%.9g]", v[0], v[1], v[2], v[3]);
        } else { lua_pop(L, 2); return 0; }
        if (n < 0 || (size_t)n >= 8192 - used) { lua_pop(L, 2); return 0; }
        used += (size_t)n; lua_pop(L, 1);
    }
    out[used++] = '}'; out[used] = 0; return 1;
}
static int gs_shader_path(lua_State *L, const char *path, size_t len, char root[MAX_PATH], char relative[MAX_PATH])
{
    const char *slash = strchr(path, '/');
    int mod = -1;
    char id[181];
    if (!len || len >= MAX_PATH || strlen(path) != len) return 0;
    if (slash && slash - path < 180) {
        memcpy(id, path, slash - path); id[slash - path] = 0;
        mod = gw_Mods_Find(id);
    }
    if (mod >= 0 && gw_Mods_IsActive(mod)) {
        if (snprintf(root, MAX_PATH, "%s\\%s", gw_Mods_Dir(), gw_Mods_Id(mod)) >= MAX_PATH) return 0;
        snprintf(relative, MAX_PATH, "%s", slash + 1); return 1;
    }
    if (!gs_mission_script_root(root)) return 0;
    snprintf(relative, MAX_PATH, "%s", path); return 1;
}
static int gs_shader_load_arg(lua_State *L, int path_index, int opts, char error[2048])
{
    size_t len;
    const char *path = luaL_checklstring(L, path_index, &len);
    char root[MAX_PATH], relative[MAX_PATH], vertex[MAX_PATH] = "", params[8192];
    int effect = 0, h;
    if (!gs_shader_path(L, path, len, root, relative)) { snprintf(error, 2048, "shader path needs a calling mod or mounted mod-id/path"); return 0; }
    if (opts && lua_istable(L, opts)) {
        lua_getfield(L, opts, "vertex");
        if (!lua_isnil(L, -1)) {
            size_t n; const char *v = lua_tolstring(L, -1, &n);
            if (!v || n >= sizeof vertex || strlen(v) != n) { lua_pop(L, 1); snprintf(error, 2048, "invalid vertex path"); return 0; }
            memcpy(vertex, v, n + 1);
        }
        lua_pop(L, 1); lua_getfield(L, opts, "kind");
        if (!lua_isnil(L, -1)) {
            const char *kind = lua_tostring(L, -1);
            if (!kind || (strcmp(kind, "effect") && strcmp(kind, "post"))) { lua_pop(L, 1); snprintf(error, 2048, "shader kind must be post or effect"); return 0; }
            effect = !strcmp(kind, "effect");
        }
        lua_pop(L, 1); lua_getfield(L, opts, "params");
        h = gs_shader_params(L, -1, params); lua_pop(L, 1);
    } else { strcpy(params, "{}"); h = 1; }
    if (!h) { snprintf(error, 2048, "params must contain finite named floats or four-number arrays"); return 0; }
    return gw_Shader_Load(gs_shader_owner(), root, relative, vertex, effect, params, error, 2048);
}
static int l_shader_load(lua_State *L)
{
    char error[2048] = ""; int h;
    if (!lua_isnoneornil(L, 2)) luaL_checktype(L, 2, LUA_TTABLE);
    h = gs_shader_load_arg(L, 1, 2, error);
    if (!h) return gs_shader_error(L, error);
    lua_pushinteger(L, h); return 1;
}
static int l_shader_set(lua_State *L)
{
    int h = gs_stage_handle_arg(L, 1); char params[8192], error[2048] = "";
    if (!gs_shader_params(L, 2, params)) return gs_shader_error(L, "invalid shader params");
    if (!gw_Shader_Set(gs_shader_owner(), h, params, error, sizeof error)) return gs_shader_error(L, error);
    lua_pushboolean(L, 1); return 1;
}
static int l_shader_status(lua_State *L)
{
    int h = gs_stage_handle_arg(L, 1), status; char error[2048] = "";
    status = gw_Shader_Status(gs_shader_owner(), h, error, sizeof error);
    if (!status) return gs_shader_error(L, error);
    lua_createtable(L, 0, 2); gs_setbool(L, "valid", status > 0); gs_setstr(L, "error", error); return 1;
}
static int l_post_add(lua_State *L)
{
    int shader, stage = 0, half = 0, order = 0, h, duration = gs_post_duration(L,2), clock = 0;
    char error[2048] = "", params[8192];
    if (!lua_isnoneornil(L, 2)) luaL_checktype(L, 2, LUA_TTABLE);
    if (lua_istable(L, 2)) {
        lua_getfield(L, 2, "clock"); clock = lua_toboolean(L,-1); lua_pop(L,1);
        if(clock && !duration)return gs_shader_error(L,"post clock requires duration_frames");
        lua_getfield(L, 2, "stage");
        if (!lua_isnil(L, -1)) {
            const char *s = lua_tostring(L, -1);
            if (!s || (strcmp(s, "world") && strcmp(s, "final"))) { lua_pop(L, 1); return gs_shader_error(L, "stage must be world or final"); }
            stage = !strcmp(s, "final");
        }
        lua_pop(L, 1); lua_getfield(L, 2, "order");
        if (!lua_isnil(L, -1)) {
            lua_Integer n = luaL_checkinteger(L, -1);
            if (n < -100000 || n > 100000) { lua_pop(L, 1); return gs_shader_error(L, "post order out of range"); }
            order = (int)n;
        }
        lua_pop(L, 1); lua_getfield(L, 2, "half"); half = lua_toboolean(L, -1); lua_pop(L, 1);
    }
    if (lua_type(L, 1) == LUA_TNUMBER) shader = gs_stage_handle_arg(L, 1);
    else { shader = gs_shader_load_arg(L, 1, 2, error); if (!shader) return gs_shader_error(L, error); }
    h = gw_Post_Add(gs_shader_owner(), shader, order, stage, half, lua_type(L, 1) != LUA_TNUMBER, error, sizeof error);
    if (!h) return gs_shader_error(L, error);
    if (lua_type(L, 1) == LUA_TNUMBER && lua_istable(L, 2)) {
        lua_getfield(L, 2, "params");
        if (!gs_shader_params(L, -1, params) || !gw_Post_Set(gs_shader_owner(), h, params, error, sizeof error)) {
            lua_pop(L, 1); gw_Post_Remove(gs_shader_owner(), h); return gs_shader_error(L, error[0] ? error : "invalid post params");
        }
        lua_pop(L, 1);
    }
    if(duration && !gs_post_timer_add(gs_shader_owner(),h,duration,clock)) {
        gw_Post_Remove(gs_shader_owner(),h);
        return gs_shader_error(L,"post clock needs scalar elapsed/progress params, or lifetime registry is full");
    }
    lua_pushinteger(L, h); return 1;
}
static int l_post_set(lua_State *L)
{
    int h = gs_stage_handle_arg(L, 1), ok; char params[8192], error[2048] = "";
    luaL_checktype(L, 2, LUA_TTABLE); lua_getfield(L, 2, "params");
    ok = gs_shader_params(L, -1, params); lua_pop(L, 1);
    if (!ok) return gs_shader_error(L, "invalid post params");
    if (!gw_Post_Set(gs_shader_owner(), h, params, error, sizeof error)) return gs_shader_error(L, error);
    lua_pushboolean(L, 1); return 1;
}
static int l_post_remove(lua_State *L)
{
    int h=gs_stage_handle_arg(L,1);gs_post_forget(gs_shader_owner(),h);
    lua_pushboolean(L, gw_Post_Remove(gs_shader_owner(),h)); return 1;
}
static int l_post_clear(lua_State *L)
{
    int owner=gs_shader_owner();
    /* No caller is not authority to clear every script's lifetime registry. */
    if(owner>0){gs_post_forget(owner,0);gw_Post_Clear(owner);}
    return 0;
}
static int l_fx_shader(lua_State *L)
{
    const char *package = luaL_checkstring(L, 1), *emitter = luaL_checkstring(L, 2);
    int shader = lua_isnoneornil(L, 3) ? 0 : gs_stage_handle_arg(L, 3); char error[2048] = "";
    if (!gw_Shader_FxOverride(gs_shader_owner(), package, emitter, shader, error, sizeof error)) return gs_shader_error(L, error);
    lua_pushboolean(L, 1); return 1;
}
static int gs_shader_vec3(lua_State *L, int idx, float out[3])
{
    int i;
    if (!lua_istable(L, idx) || lua_rawlen(L, idx) != 3) return 0;
    idx = lua_absindex(L, idx);
    for (i = 0; i < 3; ++i) {
        double v; lua_rawgeti(L, idx, i + 1);
        if (lua_type(L, -1) != LUA_TNUMBER || !isfinite(v = lua_tonumber(L, -1)) || fabs(v) > 100000) { lua_pop(L, 1); return 0; }
        out[i] = (float)v; lua_pop(L, 1);
    }
    return 1;
}
static int l_light_set(lua_State *L)
{
    float dir[3], color[3], ambient[3], length; int ok;
    luaL_checktype(L, 1, LUA_TTABLE);
    lua_getfield(L, 1, "dir"); ok = gs_shader_vec3(L, -1, dir); lua_pop(L, 1);
    lua_getfield(L, 1, "color"); ok &= gs_shader_vec3(L, -1, color); lua_pop(L, 1);
    lua_getfield(L, 1, "ambient"); ok &= gs_shader_vec3(L, -1, ambient); lua_pop(L, 1);
    if (!ok) return gs_shader_error(L, "light_set needs dir/color/ambient arrays of three finite numbers");
    length = sqrtf(dir[0]*dir[0] + dir[1]*dir[1] + dir[2]*dir[2]);
    if (length < 1e-6f || color[0] < 0 || color[1] < 0 || color[2] < 0 || ambient[0] < 0 || ambient[1] < 0 || ambient[2] < 0)
        return gs_shader_error(L, "light direction must be nonzero and colours nonnegative");
    gw_Shader_Lights(gs_shader_owner(), dir, color, ambient); lua_pushboolean(L, 1); return 1;
}
static void gs_shader_perf(lua_State *L)
{
    GwShaderPerf p; gw_Shader_Perf(&p); lua_createtable(L, 0, 12);
    gs_setint(L, "compiles", p.compiles); gs_setint(L, "cache_hits", p.cache_hits);
    gs_setint(L, "errors", p.errors); gs_setint(L, "reloads", p.reloads);
    gs_setint(L, "draws", p.draws); gs_setint(L, "passes", p.passes); gs_setint(L, "resolves", p.resolves);
    gs_setint(L, "skipped", p.skipped); gs_setnum(L, "pixels", (double)p.pixels);
    gs_setnum(L, "uniform_bytes", (double)p.uniform_bytes); gs_setnum(L, "vertex_bytes", (double)p.vertex_bytes);
    gs_setnum(L, "record_ms", p.record_ms); lua_setfield(L, -2, "shaders");
}

static int l_post_ready(lua_State *L) {
    int result=gw_Post_Ready(gs_shader_owner(),gs_stage_handle_arg(L,1));
    if(result<0)return gs_shader_error(L,"post handle expired or belongs to another script");
    lua_pushboolean(L,result>0);return 1;
}
