/* Native Lua policy and observational payload fixture. No gameplay acceptance claim. */
/* Isolate an on_armor-only script so hook discovery is tested without other mods. */
static int test_script_armor_dispatch(void)
{
    lua_State* L=gs.L;
    GsScript saved_script=gs.s[0];GsEvent saved_event=gs.ev[0];
    int saved_n=gs.n,saved_console=gs.console,saved_cur=gs.cur,saved_match=gs.match_active;
    int saved_nev=gs.nev,saved_want=gs.want_events,saved_in_event=gs.in_event,saved_dropped=gs.ev_dropped;
    int env_ref,rc=0;char out[1024]={0};
    /* Only assert is needed: do not inherit any existing on_* hooks. */
    lua_newtable(L);
    lua_rawgeti(L,LUA_REGISTRYINDEX,gs.s[gs.console].env_ref);
    lua_getfield(L,-1,"assert");lua_setfield(L,-3,"assert");lua_pop(L,1);
    env_ref=luaL_ref(L,LUA_REGISTRYINDEX);
    gs.s[0]=gs.s[gs.console];gs.s[0].env_ref=env_ref;
    gs.s[0].used=1;gs.s[0].disabled=0;gs.s[0].gameplay=1;
    gs.n=1;gs.console=0;gs.cur=-1;gs.match_active=1;gs.nev=0;gs.ev_dropped=0;
    gs_update_want_events();
    if(gs.want_events) rc=1;
    if(!rc) rc=t_exec("armor_calls=0; function on_armor(e) "
      "assert(e.entity==7 and e.port==1 and e.subfighter and e.type=='damage_pool'); "
      "assert(not e.absorbed and e.broke and e.damage==9 and e.knockback==80); "
      "armor_calls=armor_calls+1 end",out,sizeof out);
    gs_update_want_events();
    if(!gs.want_events) rc=1;
    if(!rc) {
        gw_Script_ArmorEvent(7,6,0,1,gs_fbits(9),gs_fbits(80));
        rc=t_exec("assert(armor_calls==0)",out,sizeof out);
        if(gs.nev!=1) rc=1;
        if(!rc) {gs_dispatch_events();rc=t_exec("assert(armor_calls==1)",out,sizeof out);}
        if(gs.nev!=0) rc=1;
    }
    gs.s[0]=saved_script;gs.n=saved_n;gs.console=saved_console;gs.cur=saved_cur;
    gs.match_active=saved_match;gs.nev=saved_nev;gs.in_event=saved_in_event;
    gs.ev_dropped=saved_dropped;gs.ev[0]=saved_event;
    luaL_unref(L,LUA_REGISTRYINDEX,env_ref);
    gs_update_want_events();gs.want_events=saved_want;
    if(rc) gw_test_fail("typed armour deferred hook dispatch: %s",out);
    return rc;
}
extern int gw_ScriptGame_CapsArmorFormulaTest(void);
static int test_script_fighter_caps_armor(void)
{
    char out[2048]={0}; int saved_match=gs.match_active,rc;
    int saved_nev=gs.nev,saved_want=gs.want_events,saved_dropped=gs.ev_dropped;
    GsEvent saved_event=gs.ev[0];
    lua_State* L=gs.L;
    if(gw_ScriptGame_CapsArmorFormulaTest()) {gw_test_fail("typed armour game formula fixture failed");return 1;}
    gs.match_active=1;
    rc=t_exec("local function no(e,a) assert(not pcall(gd.fighter_armor,e,a)) end; "
      "assert(type(gd.fighter_armor)=='function'); "
      "assert(not pcall(gd.fighter_armor,0));assert(not pcall(gd.fighter_armor,13)); "
      "local rows=gd.fighter_armor(1);assert(type(rows)=='table'); "
      "local order={knockback=1,damage_threshold=2,knockback_threshold=3,super=4,hit_count=5,damage_pool=6};local last=0; "
      "for _,a in ipairs(rows) do assert(order[a.type]>last);last=order[a.type]; "
      "assert(type(a.value)=='number' and type(a.remaining)=='number' and type(a.frames)=='number' and type(a.enabled)=='boolean') end; "
      "no(1,{});no(1,{type='typo',value=1});no(1,{type='super',typo=1}); "
      "no(1,{type='knockback'});no(1,{type='knockback',value=0}); "
      "no(1,{type='super',value=0/0});no(1,{type='super',value=math.huge});no(1,{type='super',value=1001}); "
      "no(1,{type='hit_count',value=1.25});no(1,{type='hit_count',value=1.00000001});no(1,{type='hit_count',value=256}); "
      "no(1,{type='super',frames=1.5});no(1,{type='super',frames=-1});no(1,{type='super',frames=36001}); "
      "no(1,{type='super',state=1.5});no(1,{type='super',state=-2});no(1,{type='super',state=65536}); "
      "no(1,{type='super',from=-0.5});no(1,{type='super',to=0/0});no(1,{type='super',to=100001}); "
      "no(1,{type='super',from=3,to=2});no(1,{type='super',direction='side'}); "
      "no(1,{type='super',clear=1});no(1,{type='super',[1]=true})",out,sizeof out);
    gs.match_active=saved_match;
    if(rc) {gw_test_fail("typed armour Lua validation: %s",out);return 1;}
    gs.match_active=0;
    rc=t_exec("local function inactive(a) local ok,e=pcall(gd.fighter_armor,1,a); "
        "assert(not ok and tostring(e):find('active match',1,true),tostring(e)) end; "
        "inactive(nil);inactive({type='super'});inactive({type='damage_pool',clear=true})",out,sizeof out);
    gs.match_active=saved_match;
    if(rc) {gw_test_fail("typed armour inactive match policy: %s",out);return 1;}
    gs.nev=0;gs.want_events=0;
    gw_Script_ArmorEvent(8,5,1,1,gs_fbits(7.5f),gs_fbits(62.25f));
    if(gs.nev!=0) rc=1;
    gs.want_events=1;
    gw_Script_ArmorEvent(8,5,1,1,gs_fbits(7.5f),gs_fbits(62.25f));
    if(gs.nev!=1 || gs.ev[0].what!=17 || gs.ev[0].a!=8 || gs.ev[0].b!=5 || gs.ev[0].c!=3) rc=1;
    if(!rc) {
        lua_rawgeti(L,LUA_REGISTRYINDEX,gs.s[gs.console].env_ref);
        gs_armor_push(L,&gs.ev[0]);lua_setfield(L,-2,"_armor_fixture");lua_pop(L,1);
        rc=t_exec("local e=_armor_fixture;assert(e.port==2 and e.entity==8 and e.subfighter); "
            "assert(e.type=='hit_count' and e.absorbed and e.broke); "
            "assert(e.damage==7.5 and e.knockback==62.25)",out,sizeof out);
        lua_rawgeti(L,LUA_REGISTRYINDEX,gs.s[gs.console].env_ref);
        lua_pushnil(L);lua_setfield(L,-2,"_armor_fixture");lua_pop(L,1);
    }
    gs.nev=GS_MAX_EVENTS;
    gw_Script_ArmorEvent(1,1,0,0,0,0);
    if(gs.nev!=GS_MAX_EVENTS || gs.ev_dropped!=saved_dropped+1) rc=1;
    gs.nev=saved_nev;gs.want_events=saved_want;gs.ev_dropped=saved_dropped;gs.ev[0]=saved_event;
    if(rc) {gw_test_fail("typed armour event queue/payload: %s",out);return 1;}
    return test_script_armor_dispatch();
}
