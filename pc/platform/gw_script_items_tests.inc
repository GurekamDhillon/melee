/* Included before test registration, after all production item/event helpers. */
static int test_script_item_events(void)
{
    char dir[MAX_PATH],path[MAX_PATH];
    FILE* f;
    int i=-1,ok=1,top,nev,want,collect,expire,dropped,cur;unsigned collect_mask,expire_mask;
    GsEvent saved[GS_MAX_EVENTS];
    gs_init();
    if (!gs.L) { gw_test_fail("scripting unavailable"); return 1; }
    top=lua_gettop(gs.L);nev=gs.nev;want=gs.want_events;
    if(gs_item_event_family(0)!=1 || gs_item_event_family(-1)!=2 || gs_item_event_family(-238)!=4 ||
       gs_item_event_family(-75) || gs_item_event_family(-212) || gs_item_event_family(-(GENO_ART_KIND_BASE+1))) {
        gw_test_fail("item family filtering bounds failed");return 1;
    }
    if(strcmp(gs_item_spawn_error(GENO_ITEM_ERR_POOL),"item pool full") ||
       strcmp(gs_item_spawn_error(GENO_ITEM_ERR_CAP),"item cap reached") ||
       !strstr(gs_item_spawn_error(GENO_ITEM_ERR_UNSAFE),"unsafe") ||
       !strstr(gs_item_spawn_error(GENO_ITEM_ERR_UNSUPPORTED),"unsupported") ||
       !strstr(gs_item_spawn_error(GENO_ITEM_ERR_UNINITIALIZED),"uninitialized")) {
        gw_test_fail("item refusal reasons collapsed");return 1;
    }
    collect=gs_want_item_collect;expire=gs_want_item_expire;collect_mask=gs_item_collect_families;expire_mask=gs_item_expire_families;
    dropped=gs.ev_dropped;cur=gs.cur;
    memcpy(saved,gs.ev,sizeof saved);gs.nev=0;
    gs_want_item_collect=gs_want_item_expire=0;
    /* Unrelated engine hooks must not arm item producers. */
    gs.want_events=1;
    gw_ItemEvent(0,17,-1,1,gs_fbits(12.5f),gs_fbits(-3.0f),2,7);
    gw_ItemEvent(1,18,-1,-1,0,0,0,1);
    if (gs.nev) { gw_test_fail("unhooked item producer queued events");ok=0; }
    snprintf(dir,sizeof dir,"%s\\gw_script_item_events_test",gs.exe_dir);
    snprintf(path,sizeof path,"%s\\events.lua",dir);
    CreateDirectoryA(dir,NULL);
    f=fopen(path,"w");
    if (!f) { gw_test_fail("item event fixture file unavailable");ok=0;goto done; }
    fputs("collects,expires=0,0\n"
          "gd.item_events{vanilla=true}\n"
          "local ok,why=pcall(gd.item_spawn,0,0,0)\n"
          "assert(not ok and tostring(why):find('gameplay',1,true))\n"
          "function on_item_collect(e)\n"
          " assert(e.item==17 and e.kind==0 and e.layer=='vanilla' and e.name=='vanilla:0')\n"
          " assert(e.port==2 and e.x==12.5 and e.y==-3)\n"
          " assert(e.payload==nil)\n"
          " collects=collects+1\n"
          "end\n"
          "function on_item_expire(e)\n"
          " assert(e.item==18 and e.name=='vanilla:0' and e.port==nil)\n"
          " assert(e.payload==nil)\n"
          " expires=expires+1\n"
          "end\n",f);
    fclose(f);
    /* Non-gameplay source: its item_spawn must fail before any mutation. */
    i=gs_load_script("item_events_fixture",path,"{\"gameplay\":false}","test");
    if (i<0) { gw_test_fail("item event/permission fixture did not load");ok=0;goto done; }
    gs.s[i].item_event_families=0;
    if(gs_item_event_accept(i,-1)||!gs_item_event_accept(i,0)) {gw_test_fail("per-script default filter failed");ok=0;}
    gs_update_want_events();gs.nev=0;
    gw_ItemEvent(0,17,-75,1,0,0,0,0); /* Fox laser */
    gw_ItemEvent(2,18,-212,-1,0,0,0,0); /* shell */
    gw_ItemEvent(0,17,-1,1,0,0,0,0); /* unrequested capsule */
    if(gs.nev) {gw_test_fail("default hooks accepted family/projectile events");ok=0;}
    gs.s[i].item_event_families=2;gs_update_want_events();
    if(!gs_item_event_accept(i,-1)||gs_item_event_accept(i,-238)) {gw_test_fail("per-script family opt-in isolation failed");ok=0;}
    gw_ItemEvent(0,17,-75,1,0,0,0,0);
    gw_ItemEvent(2,18,-212,-1,0,0,0,0);
    if(gs.nev) {gw_test_fail("vanilla opt-in accepted projectile/shell");ok=0;}
    if (!gs_want_item_collect || !gs_want_item_expire) {
        gw_test_fail("item hooks failed to arm producers");ok=0;goto done;
    }
    gw_ItemEvent(0,17,-1,1,gs_fbits(12.5f),gs_fbits(-3.0f),2,7);
    gw_ItemEvent(1,18,-1,-1,0,0,0,1);
    if (gs.nev!=2 || gs.ev[0].what!=GS_EV_ITEM_COLLECT || gs.ev[1].what!=GS_EV_ITEM_EXPIRE) {
        gw_test_fail("item queue order or event classification failed");ok=0;goto done;
    }
    /* gs_dispatch_events calls gs_item_event_push and protected script hooks. */
    gs_dispatch_events();
    lua_rawgeti(gs.L,LUA_REGISTRYINDEX,gs.s[i].env_ref);
    lua_getfield(gs.L,-1,"collects");if(lua_tointeger(gs.L,-1)!=1)ok=0;lua_pop(gs.L,1);
    lua_getfield(gs.L,-1,"expires");if(lua_tointeger(gs.L,-1)!=1)ok=0;lua_pop(gs.L,1);
    lua_pushnil(gs.L);lua_setfield(gs.L,-2,"on_item_collect");
    lua_pushnil(gs.L);lua_setfield(gs.L,-2,"on_item_expire");lua_pop(gs.L,1);
    if (!ok) gw_test_fail("item hooks failed payload/port/name assertions");
    gs_update_want_events();
    if ((!collect && gs_want_item_collect) || (!expire && gs_want_item_expire)) {
        gw_test_fail("item queue stayed armed after removing its only hooks");ok=0;
    }
    /* Other scripts can legitimately keep their own producers armed. Temporarily
     * disable those flags to check the actual producer guard independently. */
    gs_want_item_collect=gs_want_item_expire=0;gs.nev=0;
    gw_ItemEvent(0,19,-1,0,0,0,0,1);
    gw_ItemEvent(1,19,-1,-1,0,0,0,1);
    if (gs.nev) {gw_test_fail("item producer stayed armed after hook removal");ok=0;}
done:
    if(i>=0)gs_unload(i);
    DeleteFileA(path);RemoveDirectoryA(dir);
    lua_settop(gs.L,top);gs.nev=nev;gs.want_events=want;
    gs_want_item_collect=collect;gs_want_item_expire=expire;gs_item_collect_families=collect_mask;gs_item_expire_families=expire_mask;
    gs.ev_dropped=dropped;gs.cur=cur;memcpy(gs.ev,saved,sizeof saved);
    return ok?0:1;
}
