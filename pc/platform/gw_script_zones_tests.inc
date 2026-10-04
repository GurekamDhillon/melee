/* The portable fixture runs the production core; failures report to the suite. */
#undef assert
#define assert(expr) do { if(!(expr)){gw_test_fail("zones: %s (line %d)",#expr,__LINE__);return 1;} } while(0)
#define SZ_TEST_ENTRY gs_zone_core_fixture
#include "../tests/zones_test.c"
#undef SZ_TEST_ENTRY
#undef assert
static int test_script_zones_core(void) {
    memset(&s,0,sizeof s);memset(&saved,0,sizeof saved);memset(&expected,0,sizeof expected);memset(in,0,sizeof in);
    return gs_zone_core_fixture();
}
extern int gw_ScriptGame_ZonesAdapterTest(int owner);
static int test_script_zones_adapter(void) {
    int rc=gw_ScriptGame_ZonesAdapterTest(gs.s[gs.console].stage_owner);
    if(rc)gw_test_fail("zones scalar/model/handle adapter fixture: %d",rc);
    return rc!=0;
}
static int test_script_zones_api(void) {
    char out[1024];int failed=0,active=gs.match_active,want=gs.want_events;
    if(gw_Script_Exec("= type(gd.zone_add)..','..type(gd.zone_set)..','..type(gd.zone_remove)..','..type(gd.zones)..','..type(gd.zones_at)..','..type(gd.zone_members)..','..type(gd.point_zones)",out,sizeof out)!=0 || !strstr(out,"function,function,function,function,function,function,function")){gw_test_fail("zone registrations: %s",out);failed=1;}
    /* No zone hook is implicit. Zone arming must not weaken the ordinary event guard. */
    if(gs.want_events!=want){gw_test_fail("ordinary event queue changed by zone registration");failed=1;}
    gs.match_active=1;
    if(t_exec("zone_fixture_h=gd.zone_add{name='zone-fixture-a',kind='room',x0=-10,y0=-10,x1=10,y1=10};"
              "assert(#gd.point_zones(0,0)>=1);"
              "assert(not pcall(gd.zone_add,{name='zone-fixture-a',x0=0,y0=0,x1=1,y1=1}));"
              "assert(not pcall(gd.zone_set,zone_fixture_h,{x0=0}));"
              "assert(not pcall(gd.zone_set,zone_fixture_h,{tags={bad='value'}}));"
              "gd.zone_set(zone_fixture_h,{label='fixture changed',tags={'fixture'}});"
              "assert(gd.point_zones(0,0)[1].label=='fixture changed');"
              "assert(not pcall(gd.zones_at,7));assert(not pcall(gd.point_zones,0/0,0));"
              "zone_fixture_wait=gd.wait_until{port=1,zone='zone-fixture-a',timeout=1};"
              "gd.zone_remove(zone_fixture_h)",out,sizeof out)!=0){gw_test_fail("zone API validation: %s",out);failed=1;}
    gw_ScriptGame_ZonesFrame(2);gs.match_active=active;
    {GsContactWait w={0};GsContactPicture p={0};
        strcpy(w.zone,"wanted-room");strcpy(p.zones,"actual-room");w.frames=60;
        gs_contact_failure(&w,&p,out,sizeof out);
        if(!strstr(out,"wanted-room") || !strstr(out,"actual-room")){gw_test_fail("zone watcher diagnostics: %s",out);failed=1;}
    }
    return failed;
}
