/* Production Geno define admission, hashing and controls lookup. */
static int test_geno_controls_schema(void)
{
    static gn_registry r;
    static const char text[] =
        "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"controls-fixture\","
        "\"name\":\"Controls Fixture\",\"base\":\"mario\",\"common\":\"melee.common.v1\","
        "\"resources\":\"retail:mario\"},\"controls\":{\"layouts\":[{\"name\":\"Swap attacks\","
        "\"buttons\":{\"A\":\"B\",\"B\":\"A\",\"Y\":\"B\"}}],\"extras\":["
        "{\"name\":\"Y boost\",\"buttons\":{\"Y\":[\"A\",\"B\"]}}]}}]}";
    unsigned short map[GW_FC_BUTTONS];
    uint64_t good_id;
    int rc=0;
    gn_registry_clear(&r);
    if(gn_parse_text(&r,text,"controls-test")!=1 || r.p[0].controls.layouts!=1 || r.p[0].controls.extras!=1) {
        gw_test_fail("define controls schema missing or layout0 shifted"); rc=1; goto done;
    }
    good_id=r.p[0].id;
    gw_fc_build(&r.p[0].controls,0,map);
    if(gw_fc_apply(map,0x100)!=0x200 || gw_fc_apply(map,0x800)!=0x200) {
        gw_test_fail("default must apply first declared layout with extras off"); rc=1; goto done;
    }
    gw_fc_build(&r.p[0].controls,1u|0x100u,map);
    if(gw_fc_apply(map,0x800)!=0x300) {
        gw_test_fail("explicit Y boost lost when jump is X"); rc=1; goto done;
    }
    gn_registry_clear(&r);
    if(gn_parse_text(&r,
        "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"controls-fixture\","
        "\"name\":\"Controls Fixture\",\"base\":\"mario\",\"common\":\"melee.common.v1\","
        "\"resources\":\"retail:mario\"},\"controls\":{\"layouts\":[{\"name\":\"Broken\","
        "\"buttons\":{\"Y\":\"boost\"}}]}}]}","controls-test")!=1 ||
        r.p[0].controls.layouts || r.p[0].controls.extras || r.p[0].id==good_id) {
        gw_test_fail("bad controls must fall back atomically, and controls must salt fighter identity"); rc=1;
    }
done:
    gn_registry_clear(&r); return rc;
}
static int test_geno_controls_lookup(void)
{
    gn_registry boot=gn_boot, saved=gn_saved;
    int loaded=gn_boot_loaded, saved_loaded=gn_saved_loaded, swapped=gn_swapped;
    const GwFighterControlsSchema* s;
    int rc=0, ck;
    memset(&gn_boot,0,sizeof gn_boot); memset(&gn_saved,0,sizeof gn_saved);
    gn_boot_loaded=1; gn_saved_loaded=0; gn_swapped=0;
    if(gn_parse_text(&gn_boot,
        "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"controls-fixture\","
        "\"name\":\"Controls Fixture\",\"base\":\"mario\",\"common\":\"melee.common.v1\","
        "\"resources\":\"retail:mario\"},\"controls\":{\"extras\":[{\"name\":\"Y boost\","
        "\"buttons\":{\"Y\":[\"A\",\"B\"]}}]}}]}","controls-test")!=1) {
        gw_test_fail("extra-only controls fixture refused"); rc=1; goto done;
    }
    gn_build_kinds(&gn_boot); ck=gdf_alias_for_profile(&gn_boot.definitions,0);
    s=gw_Geno_ControlsSchema(ck);
    if(ck<34 || !s || s->layouts || s->extras!=1 || gw_Geno_ControlsSchema(8) ||
       gw_Geno_ControlsSchema(-1) || gw_Geno_ControlsSchema(255)) {
        gw_test_fail("define controls lookup/absent schema behavior wrong"); rc=1;
    }
done:
    gn_registry_clear(&gn_boot); gn_registry_clear(&gn_saved);
    gn_boot=boot; gn_saved=saved;
    gn_boot_loaded=loaded; gn_saved_loaded=saved_loaded; gn_swapped=swapped;
    return rc;
}
