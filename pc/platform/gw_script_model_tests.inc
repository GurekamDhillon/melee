/* Headless registration in gw_script.c; no stage, GPU draw, or disc required. */
static int t_model_options(lua_State *L)
{
    int field[SM_FIELDS] = {0};
    gs_model_options(L, 1, field);
    lua_pushnumber(L, gs_camera_float(field[SM_X]));
    lua_pushinteger(L, (unsigned)field[SM_TINT]);
    lua_pushnumber(L, gs_camera_float(field[SM_SX]));
    lua_pushnumber(L, gs_camera_float(field[SM_SY]));
    lua_pushnumber(L, gs_camera_float(field[SM_SZ]));
    lua_pushboolean(L, field[SM_ALPHA]);
    return 6;
}
static int t_model_open(lua_State *L)
{
    int i = gs_model_open_at(L, lua_tostring(L, lua_upvalueindex(1)), luaL_checkstring(L, 1));
    lua_pushinteger(L, i); return 1;
}
static int t_model_file(const char *dir, const char *name, const void *data, size_t size)
{
    char path[MAX_PATH]; FILE *f; int ok;
    snprintf(path, sizeof path, "%s\\%s", dir, name);
    f = fopen(path, "wb"); if (!f) return 0;
    ok = fwrite(data, 1, size, f) == size; fclose(f); return ok;
}
static int test_script_model_api(void)
{
    static const char *files[] = {"a.gxmesh", "b.gxmesh", "a.coll.json", "b.coll.json", "kit.gxtex"};
    static const char side[] = "{\"version\":1,\"atlas\":\"kit\",\"lines\":[[\"floor\",-1,0,1,0,3]]}";
    unsigned char mesh[36 + 3 * 32 + 6] = {0}, tex[128] = {0};
    char out[1024], dir[MAX_PATH], path[MAX_PATH];
    lua_State *L = NULL;
    GmCollision coll;
    int i, rc = 1, fail_line = 0, old_count = gs_stage_nmodels;
    if (t_exec("= type(gd.model_load),type(gd.model_release),type(gd.model_spawn),"
               "type(gd.model_move),type(gd.model_set),type(gd.model_despawn),type(gd.model_get),"
               "type(gd.model_instances),type(gd.stage_bounds)", out, sizeof out) ||
        strcmp(out, "function\nfunction\nfunction\nfunction\nfunction\nfunction\nfunction\nfunction\nfunction\n")) {
        gw_test_fail("model API registration: %s", out); return 1;
    }
    if (t_exec("assert(not pcall(gd.model_load, 'kit')); assert(not pcall(gd.model_spawn, 1, {})); "
               "assert(not pcall(gd.model_set, 1, {visible=false})); assert(not pcall(gd.model_move,1,0,0)); "
               "assert(not pcall(gd.model_despawn,1)); assert(not pcall(gd.model_release,1))", out, sizeof out)) {
        gw_test_fail("console model mutation was allowed"); return 1;
    }
    if (old_count) { gw_test_fail("model test requires an empty headless scene"); return 1; }
    if (t_exec("assert(#gd.model_instances()==0); assert(gd.stage_bounds()==nil)", out, sizeof out)) {
        gw_test_fail("read-only model queries outside a match: %s", out); return 1;
    }
    if (gs_stage_tex_levels(12, 12, 576) != 1 || gs_stage_tex_levels(8, 8, 320) != 2 ||
        gs_stage_tex_levels(12, 12, 720) != 0) {
        gw_test_fail("model mip/tile validation"); return 1;
    }
    if (!gm_collision_parse(side, &coll) || coll.count != 1 || coll.line[0].flags != 3 ||
        gm_collision_parse("{\"version\":1,\"lines\":[[\"floor\",1,0,0,0,0]]}", &coll) ||
        gm_collision_parse("{\"version\":1,\"lines\":[],\"unknown\":0}", &coll) ||
        gm_model_path("../escape") || gm_model_path("kit//part")) {
        gw_test_fail("model sidecar/path validation"); return 1;
    }
    snprintf(dir, sizeof dir, "%s\\gw_model_test", gs.exe_dir); CreateDirectoryA(dir, NULL);
    memcpy(mesh, "GXMS", 4); gw_w32(mesh + 4, 2); gw_w32(mesh + 8, 3); gw_w32(mesh + 12, 3);
    for (i = 16; i < 28; i += 4) gw_w32(mesh + i, (unsigned)gs_fbits(2));
    gw_w32(mesh + 28, 36); gw_w32(mesh + 32, 132);
    gw_w32(mesh + 36 + 32, (unsigned)gs_fbits(1));
    gw_w32(mesh + 36 + 64 + 4, (unsigned)gs_fbits(1));
    for (i = 0; i < 3; ++i) { gw_w32(mesh + 36 + 32 * i + 28, (unsigned)gs_fbits(1)); gw_w16(mesh + 132 + 2 * i, (uint16_t)i); }
    if (!gs_stage_mesh_valid(mesh, sizeof mesh)) { fail_line = __LINE__; goto done; }
    gw_w16(mesh + 132, 3);
    if (gs_stage_mesh_valid(mesh, sizeof mesh)) { fail_line = __LINE__; goto done; }
    gw_w16(mesh + 132, 0);
    memcpy(tex, "GXTX", 4); gw_w32(tex + 4, 1); gw_w32(tex + 8, GX_TF_RGBA8);
    gw_w32(tex + 12, 4); gw_w32(tex + 16, 4); gw_w32(tex + 20, 0xFFFFFFFF);
    gw_w32(tex + 28, 64); gw_w32(tex + 36, 64); memset(tex + 64, 255, 64);
    if (!t_model_file(dir, files[0], mesh, sizeof mesh) || !t_model_file(dir, files[1], mesh, sizeof mesh) ||
        !t_model_file(dir, files[2], side, sizeof side - 1) || !t_model_file(dir, files[3], side, sizeof side - 1) ||
        !t_model_file(dir, files[4], tex, sizeof tex)) { fail_line = __LINE__; goto done; }
    L = luaL_newstate(); if (!L) { fail_line = __LINE__; goto done; }
    luaL_requiref(L, "_G", luaopen_base, 1); lua_pop(L, 1);
    luaL_requiref(L, "math", luaopen_math, 1); lua_pop(L, 1);
    luaL_requiref(L, "string", luaopen_string, 1); lua_pop(L, 1);
    lua_pushcfunction(L, t_model_options); lua_setglobal(L, "options");
    lua_pushstring(L, dir); lua_pushcclosure(L, t_model_open, 1); lua_setglobal(L, "open_model");
    if (luaL_dostring(L,
        "local x,t=options{x=12.5,tint=0x12345678}; assert(x==12.5 and t==0x12345678) "
        "local _,_,sx,sy,sz,a=options{scale=-1,scale_x=-2,scale_y=3,scale_z=-0.5,alpha=true}; "
        "assert(sx==-2 and sy==3 and sz==-0.5 and a) "
        "for _,v in ipairs{{scale=0},{scale=101},{x=0/0},{z=math.huge},{layer=9},"
        "{scale_x=0},{scale_y=math.huge},{scale_z=-101},{alpha=1},"
        "{rot=361},{visible=1},{tint=-1},{tint=0x100000000}} do assert(not pcall(options,v)) end "
        "assert(open_model('a')==open_model('A')); assert(open_model('b')~=open_model('a'))")) {
        gw_test_fail("model Lua validation/cache: %s", lua_tostring(L, -1)); { fail_line = __LINE__; goto done; }
    }
    if (gs_stage_nmodels != 2 || gs_stage_models[0].image != gs_stage_models[1].image ||
        !gs_stage_models[0].atlas_owner || gs_stage_models[1].atlas_owner) { fail_line = __LINE__; goto done; }
    gs_stage_nmodels = GS_STAGE_MODELS;
    i = luaL_dostring(L, "local ok,e=pcall(open_model,'new'); assert(not ok and e:find('cache full'))");
    gs_stage_nmodels = 2;
    if (i) { gw_test_fail("model cache-full check: %s", lua_tostring(L, -1)); fail_line = __LINE__; goto done; }
    /* Refcounts are game-side, unlike the immutable asset cache. */
    i = gs_stage_models[0].token;
    if (!gw_ScriptGame_ModelRef(0, i, 1) || !gw_ScriptGame_ModelRef(0, i, 1) ||
        !gw_ScriptGame_ModelRef(0, i, -1) || !gw_ScriptGame_ModelRef(0, i, 0) ||
        !gw_ScriptGame_ModelRef(0, i, -1) || gw_ScriptGame_ModelRef(0, i, 0)) { fail_line = __LINE__; goto done; }
    rc = 0;
done:
    if (rc) gw_test_fail("model API validation or resource lifecycle failed (check at line %d)", fail_line);
    if (L) lua_close(L);
    if (gs_stage_nmodels) {
        int token = gs_stage_models[0].token;
        while (gw_ScriptGame_ModelRef(0, token, 0)) gw_ScriptGame_ModelRef(0, token, -1);
    }
    gw_Script_StageModelsReset();
    for (i = 0; i < sizeof files / sizeof *files; ++i) {
        snprintf(path, sizeof path, "%s\\%s", dir, files[i]); DeleteFileA(path);
    }
    RemoveDirectoryA(dir);
    return rc;
}
