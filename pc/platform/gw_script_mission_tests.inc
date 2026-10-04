/* Packet A: headless tests use real Lua wrappers and temporary host files.
 * They are compiled/registered with gw_script.c; no disc or renderer is needed. */
extern int gw_ScriptGame_MissionPlayerTest(void);
#ifdef _WIN32
#include <winioctl.h>
/* Directory junctions need no symlink privilege; test the actual handle path. */
static int t_mission_junction(const char *link, const char *target)
{
    struct {
        DWORD tag;
        WORD length, reserved;
        WORD substitute_offset, substitute_length, print_offset, print_length;
        WCHAR path[2 * MAX_PATH + 8];
    } reparse;
    WCHAR wide[MAX_PATH];
    int len;
    DWORD got;
    HANDLE h;
    if (!CreateDirectoryA(link, NULL)) return 0;
    len = MultiByteToWideChar(CP_ACP, 0, target, -1, wide, MAX_PATH);
    if (len < 2 || len >= MAX_PATH) { RemoveDirectoryA(link); return 0; }
    --len;
    memset(&reparse, 0, sizeof reparse);
    reparse.tag = IO_REPARSE_TAG_MOUNT_POINT;
    memcpy(reparse.path, L"\\??\\", 4 * sizeof(WCHAR));
    memcpy(reparse.path + 4, wide, (len + 1) * sizeof(WCHAR));
    reparse.substitute_length = (WORD)((len + 4) * sizeof(WCHAR));
    reparse.print_offset = (WORD)((len + 5) * sizeof(WCHAR));
    reparse.print_length = (WORD)(len * sizeof(WCHAR));
    memcpy(reparse.path + len + 5, wide, (len + 1) * sizeof(WCHAR));
    reparse.length = (WORD)(8 + (2 * len + 6) * sizeof(WCHAR));
    h = CreateFileA(link, GENERIC_WRITE, 0, NULL, OPEN_EXISTING,
                    FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS, NULL);
    if (h == INVALID_HANDLE_VALUE) { RemoveDirectoryA(link); return 0; }
    len = DeviceIoControl(h, FSCTL_SET_REPARSE_POINT, &reparse, reparse.length + 8,
                          NULL, 0, &got, NULL);
    CloseHandle(h);
    if (!len) RemoveDirectoryA(link);
    return len != 0;
}

static int t_mission_touch(const char *dir, const char *name, unsigned tick)
{
    char path[MAX_PATH];
    FILETIME time;
    HANDLE h;
    snprintf(path, sizeof path, "%s\\%s", dir, name);
    h = CreateFileA(path, FILE_WRITE_ATTRIBUTES, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h == INVALID_HANDLE_VALUE) return 0;
    time.dwHighDateTime = 0x01DA0000;
    time.dwLowDateTime = tick;
    tick = SetFileTime(h, NULL, NULL, &time);
    CloseHandle(h);
    return tick != 0;
}

static int t_mission_run(int script, const char *source)
{
    int top = lua_gettop(gs.L), rc;
    rc = luaL_loadstring(gs.L, source);
    if (!rc) {
        lua_rawgeti(gs.L, LUA_REGISTRYINDEX, gs.s[script].env_ref);
        lua_setupvalue(gs.L, -2, 1);
        rc = gs_pcall(script, 0, 0, "mission test");
    }
    if (rc) gw_test_fail("mission Lua assertion failed");
    lua_settop(gs.L, top);
    return rc;
}

static int test_script_mission_paths(void)
{
    static const char code[] =
        "assert(gd.mod_read('missions/a.txt')=='alpha'); "
        "local rows=assert(gd.mod_list('missions/')); "
        "assert(#rows==3 and rows[1].name=='a.txt' and not rows[1].dir "
        "and rows[2].name=='b' and rows[2].dir and rows[3].name=='z.txt'); "
        "stamp_a=gd.mod_stamp('missions/a.txt'); assert(math.type(stamp_a)=='integer'); "
        "for _,p in ipairs{'','missions','other/a','../missions/a','/missions/a',"
        "'C:/missions/a','missions/C:a','missions/a\\\\b','missions/../a',"
        "'missions/a..b','missions//a','missions/a/./b','missions/a.',"
        "'missions/a ', 'missions/a*','missions/a'..string.char(0)..'b',"
        "'missions/'..string.rep('a',300),string.char(92,92)..'server/share'} do "
        "local v,e=gd.mod_read(p); assert(v==nil and type(e)=='string',p); "
        "local v,e=gd.mod_list(p); assert(v==nil and type(e)=='string',p); "
        "assert(gd.mod_stamp(p)==nil,p) end; "
        "assert(gd.mod_read('missions/missing')==nil); "
        "assert(gd.mod_read('missions/b')==nil); "
        "local v,e=gd.mod_list('missions/a.txt'); assert(v==nil and type(e)=='string'); "
        "assert(#assert(gd.mod_read('missions/z.txt'))==1048576); "
        "assert(gd.mod_read('missions/b/large.txt')==nil); "
        "assert(gd.mod_stamp('missions/b/large.txt')==nil); "
        "assert(#assert(gd.mod_list('missions/b/empty'))==0)";
    char root[MAX_PATH], missions[MAX_PATH], nested[MAX_PATH], entry[MAX_PATH], path[MAX_PATH];
    char outside[MAX_PATH] = {0}, link[MAX_PATH], final[MAX_PATH], sibling[MAX_PATH];
    char out[256];
    char *big = NULL;
    int script = -1, rc = 1, junction = 0;
    if (t_exec("assert(type(gd.mod_read)=='function' and type(gd.mod_list)=='function' "
               "and type(gd.mod_stamp)=='function'); local v,e=gd.mod_read('missions/a'); "
               "assert(v==nil and type(e)=='string')", out, sizeof out)) return 1;
    snprintf(root, sizeof root, "%s\\gw_mission_path_test", gs.exe_dir);
    snprintf(missions, sizeof missions, "%s\\missions", root);
    snprintf(nested, sizeof nested, "%s\\scripts", root);
    CreateDirectoryA(root, NULL); CreateDirectoryA(missions, NULL); CreateDirectoryA(nested, NULL);
    snprintf(path, sizeof path, "%s\\deep", nested); CreateDirectoryA(path, NULL);
    snprintf(entry, sizeof entry, "%s\\deep\\main.lua", nested);
    snprintf(path, sizeof path, "%s\\b", missions); CreateDirectoryA(path, NULL);
    snprintf(path, sizeof path, "%s\\b\\empty", missions); CreateDirectoryA(path, NULL);
    big = malloc(1048577); if (!big) goto done;
    memset(big, 'x', 1048577);
    if (!t_model_file(root, "mod.json", "{}", 2) ||
        !t_model_file(missions, "a.txt", "alpha", 5) ||
        !t_model_file(missions, "z.txt", big, 1048576) ||
        !t_model_file(missions, "b/large.txt", big, 1048577)) goto done;
    script = t_model_owner_script("mission_paths", entry, code);
    if (script < 0) goto done;
    if (!t_model_file(missions, "a.txt", "bravo", 5) || !t_mission_touch(missions, "a.txt", 7) ||
        t_mission_run(script, "assert(gd.mod_read('missions/a.txt')=='bravo'); "
                            "assert(gd.mod_stamp('missions/a.txt')~=stamp_a)")) goto done;
    /* A sibling whose name starts with the root is still outside the root. */
    snprintf(outside, sizeof outside, "%s-outside", root); CreateDirectoryA(outside, NULL);
    snprintf(sibling, sizeof sibling, "%s\\a.txt", outside);
    snprintf(link, sizeof link, "%s\\escape", missions);
    if (!t_model_file(outside, "a.txt", "secret", 6) || !t_mission_junction(link, outside)) goto done;
    junction = 1;
    if (t_mission_run(script, "local v,e=gd.mod_read('missions/escape/a.txt'); "
                            "assert(v==nil and type(e)=='string'); "
                            "local v,e=gd.mod_list('missions/escape'); assert(v==nil and type(e)=='string'); "
                            "assert(gd.mod_stamp('missions/escape/a.txt')==nil)")) goto done;
    {
        const char *error = NULL;
        HANDLE h = gs_mission_open_native(root, sibling, GENERIC_READ, final, &error);
        if (h != INVALID_HANDLE_VALUE) { CloseHandle(h); goto done; }
        snprintf(path, sizeof path, "%s\\a.txt", link);
        h = gs_mission_open_native(root, path, GENERIC_READ, final, &error);
        if (h != INVALID_HANDLE_VALUE) { CloseHandle(h); goto done; }
        h = gs_mission_open_native(root, link, FILE_READ_ATTRIBUTES, final, &error);
        if (h != INVALID_HANDLE_VALUE) { CloseHandle(h); goto done; }
    }
    rc = 0;
done:
    if (script >= 0) gs_unload(script);
    free(big);
    if (junction) RemoveDirectoryA(link); /* remove the junction itself, never its target */
    if (outside[0]) {
        snprintf(path, sizeof path, "%s\\a.txt", outside); DeleteFileA(path);
        RemoveDirectoryA(outside);
    }
    snprintf(path, sizeof path, "%s\\b\\large.txt", missions); DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\b\\empty", missions); RemoveDirectoryA(path);
    snprintf(path, sizeof path, "%s\\b", missions); RemoveDirectoryA(path);
    snprintf(path, sizeof path, "%s\\a.txt", missions); DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\z.txt", missions); DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\mod.json", root); DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\deep", nested); RemoveDirectoryA(path);
    RemoveDirectoryA(nested); RemoveDirectoryA(missions); RemoveDirectoryA(root);
    if (rc) gw_test_fail("script_mission_paths: contained reads/listing/path refusal failed");
    return rc;
}

static int test_script_mission_reload(void)
{
    unsigned char mesh[138] = {0}, tex[128] = {0};
    static const char side[] = "{\"version\":1,\"atlas\":\"kit\",\"lines\":[]}";
    char dir[MAX_PATH], path[MAX_PATH], out[256], missions[MAX_PATH], entry[MAX_PATH];
    char outside[MAX_PATH] = {0}, link[MAX_PATH];
    lua_State *L = NULL;
    int i, rc = 1, old_token, old_asset, new_asset, generated = 0;
    int script = -1, fixture = 0, junction = 0, old_match;
    unsigned char *old_mesh, *old_image;
    if (t_exec("=1", out, sizeof out) || gs_stage_nmodels) return 1;
    old_match = gs.match_active;
    snprintf(dir, sizeof dir, "%s\\gw_mission_reload_test", gs.exe_dir); CreateDirectoryA(dir, NULL);
    memcpy(mesh, "GXMS", 4); gw_w32(mesh + 4, 2); gw_w32(mesh + 8, 3); gw_w32(mesh + 12, 3);
    for (i = 16; i < 28; i += 4) gw_w32(mesh + i, (unsigned)gs_fbits(2));
    gw_w32(mesh + 28, 36); gw_w32(mesh + 32, 132);
    gw_w32(mesh + 68, (unsigned)gs_fbits(1)); gw_w32(mesh + 104, (unsigned)gs_fbits(1));
    for (i = 0; i < 3; ++i) {
        gw_w32(mesh + 36 + i * 32 + 28, (unsigned)gs_fbits(1)); gw_w16(mesh + 132 + i * 2, (uint16_t)i);
    }
    memcpy(tex, "GXTX", 4); gw_w32(tex + 4, 1); gw_w32(tex + 8, GX_TF_RGBA8);
    gw_w32(tex + 12, 4); gw_w32(tex + 16, 4); gw_w32(tex + 20, 0xFFFFFFFF);
    gw_w32(tex + 28, 64); gw_w32(tex + 36, 64); memset(tex + 64, 255, 64);
#define MISSION_RELOAD_CHECK(c) do { if (!(c)) { gw_test_fail("script_mission_reload at %d", __LINE__); goto done; } } while (0)
    MISSION_RELOAD_CHECK(t_model_file(dir, "a.gxmesh", mesh, sizeof mesh) &&
        t_model_file(dir, "a.coll.json", side, sizeof side - 1) && t_model_file(dir, "kit.gxtex", tex, sizeof tex));
    L = luaL_newstate(); MISSION_RELOAD_CHECK(L);
    luaL_requiref(L, "_G", luaopen_base, 1); lua_pop(L, 1);
    luaL_requiref(L, "string", luaopen_string, 1); lua_pop(L, 1);
    lua_pushstring(L, dir); lua_pushcclosure(L, t_model_open, 1); lua_setglobal(L, "open_model");
    MISSION_RELOAD_CHECK(!luaL_dostring(L, "a=open_model('a'); assert(a==open_model('A'))"));
    old_asset = gs_stage_nmodels - 1; old_token = gs_stage_models[old_asset].token;
    old_mesh = gs_stage_models[old_asset].mesh; old_image = gs_stage_models[old_asset].image;
    MISSION_RELOAD_CHECK(gw_ScriptGame_ModelRef(old_asset, old_token, 1));
    gw_w32(mesh + 68, (unsigned)gs_fbits(1.5f));
    MISSION_RELOAD_CHECK(t_model_file(dir, "a.gxmesh", mesh, sizeof mesh) && t_mission_touch(dir, "a.gxmesh", 1));
    MISSION_RELOAD_CHECK(!luaL_dostring(L, "b=open_model('a'); assert(b~=a and b==open_model('a'))"));
    new_asset = gs_stage_nmodels - 1;
    MISSION_RELOAD_CHECK(gs_stage_models[old_asset].mesh == old_mesh &&
        gw_rf32(old_mesh + 68) == 1 && gs_stage_models[new_asset].image == old_image &&
        gw_ScriptGame_ModelRef(old_asset, old_token, 0));
    MISSION_RELOAD_CHECK(t_mission_touch(dir, "a.coll.json", 2));
    MISSION_RELOAD_CHECK(!luaL_dostring(L, "c=open_model('a'); assert(c~=b)"));
    tex[64] = 123;
    MISSION_RELOAD_CHECK(t_model_file(dir, "kit.gxtex", tex, sizeof tex) && t_mission_touch(dir, "kit.gxtex", 3));
    MISSION_RELOAD_CHECK(!luaL_dostring(L, "d=open_model('a'); assert(d~=c)"));
    MISSION_RELOAD_CHECK(gs_stage_models[gs_stage_nmodels - 1].image != old_image && old_image[0] == 255);
    MISSION_RELOAD_CHECK(t_model_file(dir, "kit.glow.gxtex", tex, sizeof tex));
    MISSION_RELOAD_CHECK(!luaL_dostring(L, "e=open_model('a'); assert(e~=d)"));
    MISSION_RELOAD_CHECK(t_mission_touch(dir, "kit.glow.gxtex", 4));
    MISSION_RELOAD_CHECK(!luaL_dostring(L, "f=open_model('a'); assert(f~=e)"));
    snprintf(path, sizeof path, "%s\\kit.glow.gxtex", dir); DeleteFileA(path);
    MISSION_RELOAD_CHECK(!luaL_dostring(L, "assert(open_model('a')==d)"));
    /* Decision E2: releasing the last retain still keeps old bytes until reset. */
    MISSION_RELOAD_CHECK(gw_ScriptGame_ModelRef(old_asset, old_token, -1) && gs_stage_models[old_asset].mesh == old_mesh);
    /* Exercise the calling-mod resolver and real retain/spawn wrappers too. */
    snprintf(missions, sizeof missions, "%s\\missions", dir); CreateDirectoryA(missions, NULL);
    snprintf(entry, sizeof entry, "%s\\scripts\\deep\\main.lua", dir);
    MISSION_RELOAD_CHECK(t_model_file(dir, "mod.json", "{}", 2) &&
        t_model_file(missions, "a.gxmesh", mesh, sizeof mesh) &&
        t_model_file(missions, "a.coll.json", side, sizeof side - 1) &&
        t_model_file(missions, "kit.gxtex", tex, sizeof tex) &&
        t_model_file(missions, "a.dat", mesh, sizeof mesh));
    MISSION_RELOAD_CHECK(gw_ScriptGame_ModelOwnerTestFixture(1)); fixture = 1; gs.match_active = 1;
    script = t_model_owner_script("mission_assets", entry,
        "a=gd.model_load('missions/a'); assert(a==gd.model_load('missions/a.gxmesh')); "
        "h=assert(gd.model_spawn(a,{collision=false})); assert(gd.model_despawn(h)); "
        "h=assert(gd.model_spawn(a,{collision=false})); "
        "local h,e=gd.stage_add_model{file='missions/a.dat'}; assert(h==nil and type(e)=='string')");
    MISSION_RELOAD_CHECK(script >= 0 && !gs_stage_archive_bytes && !gs_stage_archive_data);
    snprintf(outside, sizeof outside, "%s-outside", dir); CreateDirectoryA(outside, NULL);
    snprintf(link, sizeof link, "%s\\escape", missions);
    MISSION_RELOAD_CHECK(t_model_file(outside, "a.gxmesh", mesh, sizeof mesh) &&
        t_model_file(outside, "a.dat", mesh, sizeof mesh) && t_mission_junction(link, outside));
    junction = 1;
    MISSION_RELOAD_CHECK(!t_mission_run(script,
        "assert(not pcall(gd.model_load,'missions/escape/a')); "
        "assert(not pcall(gd.stage_add_model,{file='missions/escape/a.dat'})); "
        "assert(not pcall(gd.model_load,'missions/../a')); "
        "assert(not pcall(gd.stage_add_model,{file='missions/../a.dat'}))"));
    gs_unload(script); script = -1;
    MISSION_RELOAD_CHECK(gw_ScriptGame_ModelOwnerTestFixture(0)); fixture = 0; gs.match_active = old_match;
    while (gs_stage_nmodels < GS_STAGE_MODELS) {
        char name[48], filename[64];
        snprintf(name, sizeof name, "part_%d", generated++);
        snprintf(filename, sizeof filename, "%s.gxmesh", name);
        MISSION_RELOAD_CHECK(t_model_file(dir, filename, mesh, sizeof mesh));
        snprintf(filename, sizeof filename, "%s.coll.json", name);
        MISSION_RELOAD_CHECK(t_model_file(dir, filename, side, sizeof side - 1));
        lua_getglobal(L, "open_model"); lua_pushstring(L, name);
        MISSION_RELOAD_CHECK(!lua_pcall(L, 1, 1, 0)); lua_pop(L, 1);
    }
    MISSION_RELOAD_CHECK(gs_stage_nmodels == 128 && !luaL_dostring(L,
        "assert(open_model('a')==d); local ok,e=pcall(open_model,'overflow'); "
        "assert(not ok and e:find('cache full') and e:find('restart the match'))"));
    rc = 0;
done:
    if (script >= 0) gs_unload(script);
    if (fixture) gw_ScriptGame_ModelOwnerTestFixture(0);
    gs.match_active = old_match;
    if (L) lua_close(L);
    if (gs_stage_nmodels) {
        while (gw_ScriptGame_ModelRef(0, gs_stage_models[0].token, 0))
            gw_ScriptGame_ModelRef(0, gs_stage_models[0].token, -1);
        gw_Script_StageModelsReset();
    }
    for (i = 0; i < 4; ++i) {
        static const char *names[] = {"a.gxmesh", "a.coll.json", "kit.gxtex", "kit.glow.gxtex"};
        snprintf(path, sizeof path, "%s\\%s", dir, names[i]); DeleteFileA(path);
    }
    for (i = 0; i < generated; ++i) {
        snprintf(path, sizeof path, "%s\\part_%d.gxmesh", dir, i); DeleteFileA(path);
        snprintf(path, sizeof path, "%s\\part_%d.coll.json", dir, i); DeleteFileA(path);
    }
    if (junction) RemoveDirectoryA(link);
    if (outside[0]) {
        snprintf(path, sizeof path, "%s\\a.gxmesh", outside); DeleteFileA(path);
        snprintf(path, sizeof path, "%s\\a.dat", outside); DeleteFileA(path);
        RemoveDirectoryA(outside);
    }
    for (i = 0; i < 4; ++i) {
        static const char *names[] = {"a.gxmesh", "a.coll.json", "kit.gxtex", "a.dat"};
        snprintf(path, sizeof path, "%s\\missions\\%s", dir, names[i]); DeleteFileA(path);
    }
    snprintf(path, sizeof path, "%s\\mod.json", dir); DeleteFileA(path);
    snprintf(path, sizeof path, "%s\\missions", dir); RemoveDirectoryA(path);
    RemoveDirectoryA(dir);
#undef MISSION_RELOAD_CHECK
    return rc;
}

static int test_script_mission_player(void)
{
    if (gw_ScriptGame_MissionPlayerTest()) {
        gw_test_fail("script_mission_player: fake fighter collision reads failed"); return 1;
    }
    return 0;
}

static int test_script_mission_stage_hide(void)
{
    char out[512];
    int script = -1, old_match, old_owner, failed = 0;
    if (t_exec("assert(type(gd.stage_hide)=='function'); "
               "assert(not pcall(gd.stage_hide,true)); assert(not pcall(gd.stage_hide,false)); "
               "assert(not pcall(gd.stage_hide)); assert(not pcall(gd.stage_hide,1))", out, sizeof out) ||
        gw_mpScriptStageIsolationTest()) {
        gw_test_fail("script_mission_stage_hide: owner/type refusal or isolation regression"); return 1;
    }
    old_match = gs.match_active; old_owner = gs_stage_isolation_owner;
    gs.match_active = 1; /* no host map: unsupported, not an actual game match */
    script = t_model_owner_script("mission_hide", "mission_hide.lua", "local ok,e=gd.stage_hide(true); "
                                 "assert(ok==false and type(e)=='string'); assert(gd.stage_hide(false)==true); "
                                 "assert(gd.stage_isolate()==false)");
    if (script < 0) failed = 1;
    if (script >= 0) {
        gs_stage_isolation_owner = gs.cur + GS_MAX_SCRIPTS + 2;
        if (t_mission_run(script, "assert(not pcall(gd.stage_hide,true)); assert(not pcall(gd.stage_hide,false))")) failed = 1;
        gs_stage_isolation_owner = 0;
        gs_unload(script);
    }
    gs.match_active = old_match; gs_stage_isolation_owner = old_owner;
    return failed;
}

static int test_script_mission_limits(void)
{
    GmCollision coll;
    char json[4096], out[512];
    int i, at = 0;
    if (SCRIPT_MESH_ASSETS != 128 || SCRIPT_MESH_LINES != 64 || GM_COLLISION_LINES != 64) return 1;
    at += snprintf(json + at, sizeof json - at, "{\"version\":1,\"lines\":[");
    for (i = 0; i < 64; ++i) at += snprintf(json + at, sizeof json - at, "%s[\"floor\",-1,0,1,0,0]", i ? "," : "");
    snprintf(json + at, sizeof json - at, "]}");
    if (!gm_collision_parse(json, &coll) || coll.count != 64) return 1;
    snprintf(json + at, sizeof json - at, ",[\"floor\",-1,0,1,0,0]]}");
    if (gm_collision_parse(json, &coll)) return 1;
    if (t_exec("for _,xy in ipairs{{49001,0},{-49001,0},{0,49001},{0,-49001},"
               "{0/0,0},{0,math.huge}} do local ok,e=pcall(gd.fly_target,1,xy[1],xy[2]); "
               "assert(not ok and e:find('49000',1,true)) end; "
               "for _,xy in ipairs{{49000,0},{-49000,0},{0,49000},{0,-49000},{10001,0}} do "
               "local ok,e=pcall(gd.fly_target,1,xy[1],xy[2]); "
               "assert(ok or not e:find('requires finite',1,true)) end", out, sizeof out)) return 1;
    return 0;
}
static int test_script_mission_archive(void)
{
    unsigned char data[5] = {1, 2, 3, 4, 5};
    int failed;
    if (gs_stage_archive_data || gs_stage_archive_bytes) return 1;
    gs_stage_archive_data = data; gs_stage_archive_bytes = sizeof data;
    gs_stage_archive_stamp = 0x123456789ABCDEF0ull;
    failed = gw_Script_StageArchiveInput(0, 0) != 5 ||
             (unsigned)gw_Script_StageArchiveInput(1, 0) != 0x12345678u ||
             (unsigned)gw_Script_StageArchiveInput(2, 0) != 0x9ABCDEF0u ||
             gw_Script_StageArchiveInput(3, 0) != 0x01020304 ||
             gw_Script_StageArchiveInput(4, 4) != 5 ||
             gw_Script_StageArchiveInput(3, 2) || gw_Script_StageArchiveInput(4, 5);
    gs_stage_archive_data = NULL; gs_stage_archive_bytes = 0; gs_stage_archive_stamp = 0;
    if (failed) gw_test_fail("script_mission_archive: scalar BE word/tail transfer failed");
    return failed;
}
#else
static int test_script_mission_player(void) { return gw_ScriptGame_MissionPlayerTest(); }
#endif
