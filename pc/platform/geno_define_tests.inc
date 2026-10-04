/* Suite-pattern tests. No archive/disc or live alias assignment required. */
static int test_geno_define_registry(void)
{
    static gn_registry r;
    int i;
    char text[512];
    gn_registry_clear(&r);
    for (i = 0; i < 257; ++i) {
        snprintf(text, sizeof text,
            "{\"geno\":6,\"fighters\":[{\"define\":{\"key\":\"hero-%d\",\"name\":\"Hero %d\",\"base\":\"mario\",\"common\":\"melee.common.v1\",\"resources\":\"retail:mario\"},\"attributes\":{\"walk_max_vel\":1.8},\"common_states\":[{\"motion\":44,\"subaction\":46}]}]}", i, i);
        if (gn_parse_text(&r, text, "define-test") != 1 || r.n != i+1 ||
            !r.p[i].defined || r.p[i].nattr != 1 || r.p[i].ncommon != 1 ||
            r.p[i].common_row[0][2] != 46) {
            gw_test_fail("native definition parse/growth failed at profile %d", i);
            gn_registry_clear(&r); return 1;
        }
    }
    if (!r.p[31].id || !r.p[32].id || !r.p[255].id || r.p[31].id == r.p[32].id) {
        gw_test_fail("definition content identities lost across old 32-profile boundary");
        gn_registry_clear(&r); return 1;
    }
    gn_registry_clear(&r); return 0;
}
static int test_geno_define_repeated_install(void)
{
    int i, before = gw_Geno_ProfileCount(), rc = 0;
    /* A preceding test may have panicked before TestRestore. Preserve that
       caller's active/saved registries rather than compare it with its boot. */
    gn_registry entry = gn_boot, entry_saved = gn_saved;
    int entry_loaded = gn_boot_loaded, entry_saved_loaded = gn_saved_loaded;
    int entry_swapped = gn_swapped;
    memset(&gn_boot, 0, sizeof gn_boot);
    memset(&gn_saved, 0, sizeof gn_saved);
    gn_boot_loaded = 1; gn_saved_loaded = 0; gn_swapped = 0;
    for (i = 0; i < 8; ++i) {
        if (gw_Geno_TestInstall("{\"geno\":6,\"fighters\":[{\"attach\":\"mario\"}]}") != 1 ||
            gw_Geno_ProfileCount() != 1) {
            gw_test_fail("repeated temporary registry installation failed"); rc = 1; break;
        }
    }
    gw_Geno_TestRestore();
    if (gw_Geno_ProfileCount() != 0) {
        gw_test_fail("temporary registry did not restore original ownership"); rc = 1;
    }
    gn_registry_clear(&gn_boot);
    gn_registry_clear(&gn_saved);
    gn_boot = entry; gn_saved = entry_saved;
    gn_boot_loaded = entry_loaded; gn_saved_loaded = entry_saved_loaded;
    gn_swapped = entry_swapped; ++gn_pool_gen;
    if (gw_Geno_ProfileCount() != before) {
        gw_test_fail("repeated-install test changed caller registry"); rc = 1;
    }
    return rc;
}
