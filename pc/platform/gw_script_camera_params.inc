extern int gw_ScriptGame_CameraParamRead(int field);
extern int gw_ScriptGame_CameraParamOwner(void);
extern int gw_ScriptGame_CameraParams(int owner, int mask, int a, int b, int c, int d,
                                     int e, int f, int g, int h, int j, int k);
extern void gw_ScriptGame_CameraParamsRelease(int owner);
static int l_camera_params(lua_State* L) {
    static const char* names[10] = {"min_dist","max_depth","fov","fixed_zoom",
        "track_smooth","track_ratio","tilt","pan","yaw_gain","pitch_gain"};
    static const float lo[10] = {1,1,1,0,0,0,1,-180,0,0};
    static const float hi[10] = {49000,49000,89,10,10,10,89,180,10,10};
    float previous[10], v[10];
    int i, mask = 0, owner;
    gs_require_offline(L, "camera_params");
    if (!gs.match_active) return luaL_error(L,"camera_params requires an active match");
    owner = gw_ScriptGame_CameraParamOwner();
    if (owner && owner != gs.cur + 1) return luaL_error(L, "camera params owned by another script");
    for (i=0; i<10; ++i) v[i] = previous[i] = gs_camera_float(gw_ScriptGame_CameraParamRead(i));
    if (!lua_isnoneornil(L, 1)) {
        luaL_checktype(L, 1, LUA_TTABLE);
        lua_pushnil(L);
        while (lua_next(L, 1)) {
            const char* key = lua_type(L,-2)==LUA_TSTRING ? lua_tostring(L,-2) : NULL;
            for (i=0; i<10; ++i) if (key && !strcmp(key,names[i])) break;
            if (i==10) return luaL_error(L,"unknown camera parameter");
            v[i] = (float)luaL_checknumber(L,-1);
            if (!(v[i]>=lo[i] && v[i]<=hi[i])) return luaL_error(L,"%s must be in %f..%f",names[i],(lua_Number)lo[i],(lua_Number)hi[i]);
            mask |= 1 << i;
            lua_pop(L,1);
        }
        if (v[0]>v[1]) return luaL_error(L,"min_dist must not exceed max_depth");
        if ((mask & (1 << 2)) && (mask & (1 << 6)) && v[2]!=v[6])
            return luaL_error(L,"tilt is the stage's FOV alias; fov and tilt must agree");
    }
    /* An empty table is a no-op; nil restores. Validate completely before branching. */
    if (mask || lua_isnoneornil(L,1)) {
        gs_rw_branch();
        if (!gw_ScriptGame_CameraParams(gs.cur+1,mask,gs_fbits(v[0]),gs_fbits(v[1]),
            gs_fbits(v[2]),gs_fbits(v[3]),gs_fbits(v[4]),gs_fbits(v[5]),
            gs_fbits(v[6]),gs_fbits(v[7]),gs_fbits(v[8]),gs_fbits(v[9])))
            return luaL_error(L,"camera params rejected");
    }
    lua_newtable(L);
    for (i=0; i<10; ++i) gs_setnum(L,names[i],previous[i]);
    return 1;
}
