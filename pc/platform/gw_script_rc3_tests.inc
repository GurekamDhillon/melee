/* Tests for the Linux-contained mod IO (items, calling-mod shader path). Included after gw_script_items_tests.inc. */
/* A mod's items/<name>/item.json is read and registered on every platform (a Linux Envoy run had no items: "unknown item name"
 * on every drive drop). Writes a one-item mod folder, defines it through the contained reader, and finds it by name. */
static int test_script_item_define_contained(void)
{
    char root[MAX_PATH],d1[MAX_PATH],d2[MAX_PATH],path[MAX_PATH],err[256]="";
    FILE* f;int def,def2;
    gs_init();
    snprintf(root,sizeof root,"%s\\gw_script_item_define_test",gs.exe_dir);
    snprintf(d1,sizeof d1,"%s\\items",root);
    snprintf(d2,sizeof d2,"%s\\rc3_probe",d1);
    snprintf(path,sizeof path,"%s\\item.json",d2);
    CreateDirectoryA(root,NULL);CreateDirectoryA(d1,NULL);CreateDirectoryA(d2,NULL);
    f=fopen(path,"w");
    if(!f) {gw_test_fail("item define fixture unavailable: %s (error %lu)",path,(unsigned long)GetLastError());return 1;}
    fputs("{\"version\":1,\"name\":\"rc3_probe\",\"physics\":{\"gravity\":0.12,\"terminal_velocity\":4,\"bounce\":0,\"friction\":0.15,\"floor_rest\":true,\"stage_collision\":true},"
          "\"collection\":\"touch\",\"radius\":7,\"ports\":1,\"teams\":15,\"lifetime\":900,\"blink\":120,\"count_against_cap\":false,\"held\":false,\"thrown\":false,"
          "\"payload\":{\"colour\":\"red\",\"amount\":1},\"visual\":{\"scale\":2.5}}",f);
    fclose(f);
    def=gs_item_define_root(root,"items/rc3_probe/item.json",err,sizeof err);
    def2=def>=0?gw_Geno_ItemFind("rc3_probe"):-1;
    DeleteFileA(path);RemoveDirectoryA(d2);RemoveDirectoryA(d1);RemoveDirectoryA(root);
    if(def<0) {gw_test_fail("contained item definition refused: %s",err);return 1;}
    if(def2!=def) {gw_test_fail("a defined item is not found by its name (def %d, find %d)",def,def2);return 1;}
    if(gs_item_define_root(root,"items/../item.json",err,sizeof err)>=0) {gw_test_fail("an item path outside items/<name>/ was accepted");return 1;}
    return 0;
}

/* A script in a mod folder (one holding mod.json) resolves its own shaders/ by relative path, on every platform: gd.shader_load must
 * get past the "calling mod" lookup (a Linux Envoy run answered "shader path needs a calling mod or mounted mod-id/path" for the
 * crit flash). Headless there is no render device, so the load itself may fail later; only the path error is the failure here. */
static int test_script_shader_calling_mod(void)
{
    char root[MAX_PATH],sdir[MAX_PATH],mj[MAX_PATH],lua[MAX_PATH],shader[MAX_PATH];
    FILE* f;int i,ok=1,top;char why[512]="";
    gs_init();
    if(!gs.L) {gw_test_fail("scripting unavailable");return 1;}
    top=lua_gettop(gs.L);
    snprintf(root,sizeof root,"%s\\gw_script_shader_root_test",gs.exe_dir);
    snprintf(sdir,sizeof sdir,"%s\\shaders",root);
    snprintf(mj,sizeof mj,"%s\\mod.json",root);
    snprintf(lua,sizeof lua,"%s\\main.lua",root);
    snprintf(shader,sizeof shader,"%s\\t.wgsl",sdir);
    CreateDirectoryA(root,NULL);CreateDirectoryA(sdir,NULL);
    f=fopen(mj,"w");if(f){fputs("{}",f);fclose(f);}
    f=fopen(shader,"w");if(f){fputs("// probe\n",f);fclose(f);}
    f=fopen(lua,"w");
    if(!f) {gw_test_fail("shader root fixture unavailable");ok=0;goto done;}
    fputs("local h,why=gd.shader_load('shaders/t.wgsl')\nrc3_why=tostring(why)\n",f);fclose(f);
    i=gs_load_script("shader_root_fixture",lua,"{\"gameplay\":false}","test");
    if(i<0) {gw_test_fail("shader root fixture did not load");ok=0;goto done;}
    lua_rawgeti(gs.L,LUA_REGISTRYINDEX,gs.s[i].env_ref);
    lua_getfield(gs.L,-1,"rc3_why");snprintf(why,sizeof why,"%s",lua_tostring(gs.L,-1)?lua_tostring(gs.L,-1):"");lua_pop(gs.L,2);
    if(strstr(why,"calling mod")) {gw_test_fail("a script in a mod folder has no calling mod: %s",why);ok=0;}
    gs_unload(i);
done:
    DeleteFileA(lua);DeleteFileA(shader);DeleteFileA(mj);RemoveDirectoryA(sdir);RemoveDirectoryA(root);
    lua_settop(gs.L,top);
    return ok?0:1;
}
