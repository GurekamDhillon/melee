/* Included after mission path helpers: use final-handle containment, including
 * junctions, rather than accepting an arbitrary shader path from Lua. */
#include "gw_surface.h"
#include "gw_surface_path.h"
static int gs_surface_shader(lua_State *L, unsigned slot, int path_index, int options_index)
{
    const char *error = NULL;
    char diagnostic[1024] = {0};
    uint32_t program = 0;
    float params[16] = {0};
    int owner = gs.cur + 1, n;
    size_t length = 0;
    const char *relative = lua_isnil(L, path_index) ? NULL : luaL_checklstring(L, path_index, &length);
    if (relative) {
#ifdef _WIN32
        char root[MAX_PATH], path[MAX_PATH], final[MAX_PATH];
        HANDLE h;
        LARGE_INTEGER bytes;
        DWORD got;
        char *text;
        if (!gw_surface_path_valid(relative, length)) {
            error = "surface path must be shaders/*.wgsl with no traversal";
            goto failed;
        }
        if (!gs_mission_script_root(root) || snprintf(path, sizeof path, "%s\\%s", root, relative) >= sizeof path) {
            error = "surface shader requires a calling mod folder"; goto failed;
        }
        h = gs_mission_open_native(root, path, GENERIC_READ, final, &error);
        if (h == INVALID_HANDLE_VALUE) goto failed;
        if (!GetFileSizeEx(h, &bytes) || bytes.QuadPart < 1 || bytes.QuadPart > 65536) {
            CloseHandle(h); error = "surface shader must be 1..65536 bytes"; goto failed;
        }
        text = (char *)malloc((size_t)bytes.QuadPart + 1);
        if (!text) { CloseHandle(h); error = "surface shader allocation failed"; goto failed; }
        if (!ReadFile(h, text, (DWORD)bytes.QuadPart, &got, NULL) || got != (DWORD)bytes.QuadPart) {
            free(text); CloseHandle(h); error = "surface shader read failed"; goto failed;
        }
        CloseHandle(h);
        text[got] = 0;
        if (strlen(text) != got) { free(text); error = "surface shader contains NUL"; goto failed; }
        program = gw_surface_register(text, relative, diagnostic, sizeof diagnostic);
        free(text);
        if (!program) { error = diagnostic; goto failed; }
#else
        error = "surface shader loading requires Windows"; goto failed;
#endif
    }
    if (!lua_isnoneornil(L, options_index)) {
        luaL_checktype(L, options_index, LUA_TTABLE);
        lua_getfield(L, options_index, "params");
        if (!lua_isnil(L, -1)) {
            luaL_checktype(L, -1, LUA_TTABLE);
            if (lua_rawlen(L, -1) > 16) { lua_pop(L, 1); error = "surface params has at most 16 floats"; goto failed; }
            for (n = 0; n < (int)lua_rawlen(L, -1); ++n) {
                lua_rawgeti(L, -1, n + 1);
                params[n] = (float)luaL_checknumber(L, -1);
                lua_pop(L, 1);
                if (!isfinite(params[n])) { lua_pop(L, 1); error = "surface params must be finite"; goto failed; }
            }
        }
        lua_pop(L, 1);
    }
    if (!gw_surface_select(slot, program, (unsigned)owner, params)) {
        error = "surface selection is owned by another script"; goto failed;
    }
    lua_pushboolean(L, 1);
    return 1;
failed:
    gw_log("surface shader %s: %s", relative ? relative : "clear", error ? error : "failed");
    lua_pushnil(L);
    lua_pushstring(L, error ? error : "surface shader failed");
    return 2;
}
static int l_fighter_shader(lua_State *L)
{
    lua_Integer port = luaL_checkinteger(L, 1);
    if (port < 1 || port > 6) return luaL_argerror(L, 1, "fighter port must be 1..6");
    return gs_surface_shader(L, (unsigned)port, 2, 3);
}
static int l_stage_shader(lua_State *L) { return gs_surface_shader(L, 7, 1, 2); }

/* Parameter-only updates never read a shader file or register/compile a body. */
static int l_fighter_shader_set(lua_State *L)
{
    lua_Integer port = luaL_checkinteger(L, 1);
    float params[16] = {0};
    size_t n, count;
    if (port < 1 || port > 6) return luaL_argerror(L, 1, "fighter port must be 1..6");
    luaL_checktype(L, 2, LUA_TTABLE);
    lua_getfield(L, 2, "params");
    luaL_checktype(L, -1, LUA_TTABLE);
    count = lua_rawlen(L, -1);
    if (count > 16) return luaL_error(L, "surface params has at most 16 floats");
    for (n = 0; n < count; ++n) {
        lua_rawgeti(L, -1, (lua_Integer)n + 1);
        params[n] = (float)luaL_checknumber(L, -1);
        lua_pop(L, 1);
        if (!isfinite(params[n])) return luaL_error(L, "surface params must be finite");
    }
    lua_pop(L, 1);
    if (!gw_surface_update((unsigned)port, (unsigned)(gs.cur + 1), params)) {
        lua_pushnil(L); lua_pushliteral(L, "no surface selection owned by this script"); return 2;
    }
    lua_pushboolean(L, 1); return 1;
}
