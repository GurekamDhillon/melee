/* Included by geno_registry.c after geno_define_tests.inc. No disc files.
 * Catches results omitted by the strict parser, implicit Mario becoming an art
 * donor, Captain's CK 0 treated as false, and lookup mutating online identity. */
static int gn_css_art_file_is(const gn_profile* p, int kind, int costume, const char* expected)
{
    const char* actual = gn_pres_file(p, kind, costume);
    return actual != NULL && !strcmp(actual, expected);
}

static int test_geno_css_art_registry(void)
{
    static gn_registry r;
    static const struct { const char* text; int base; } cases[] = {
        {GN_T_HEAD(9, ""), 8},
        {GN_T_TMPL(10, "\"template\":\"captain\",", ""), 0},
        {GN_T_TMPL(10, "\"template\":\"mario\",", ""), 8},
        {GN_T_TMPL(10, "", ""), -1}
    };
    gn_registry boot = gn_boot, saved = gn_saved;
    int loaded = gn_boot_loaded, saved_loaded = gn_saved_loaded, swapped = gn_swapped;
    int rc = 0, i, ck, what;
    extern void gw_Skins_Suspend(int on);
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9,
            ",\"presentation\":{\"icon\":\"i.gxtex\",\"portrait\":\"p.gxtex\","
            "\"stock\":\"s.gxtex\",\"results\":[\"r0.gxtex\",\"r1.gxtex\"]}"), "results") != 1) {
        gw_test_fail("presentation.results refused"); rc = 1;
    } else if (!gn_css_art_file_is(&r.p[0], GN_PRES_RESULTS, 1, "r1.gxtex") ||
               !gn_css_art_file_is(&r.p[0], GN_PRES_RESULTS, 254, "r0.gxtex") ||
               !gn_css_art_file_is(&r.p[0], GN_PRES_PORTRAIT, 1, "p.gxtex")) {
        gw_test_fail("results/CSP selection or costume fallback wrong"); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"portrait\":\"p.gxtex\"}"), "legacy-csp") != 1 ||
        !gn_css_art_file_is(&r.p[0], GN_PRES_RESULTS, 0, "p.gxtex")) {
        gw_test_fail("legacy portrait no longer supplies results"); rc = 1;
    }
    gn_registry_clear(&r);
    if (gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"results\":[]}"), "empty") ||
        gn_parse_text(&r, GN_T_HEAD(9, ",\"presentation\":{\"results\":\"../r.gxtex\"}"), "path") ||
        gn_parse_text(&r, GN_T_HEAD(8, ",\"presentation\":{\"results\":\"r.gxtex\"}"), "version")) {
        gw_test_fail("invalid results art declaration accepted"); rc = 1;
    }
    gn_registry_clear(&r);
    gw_Skins_Suspend(1);
    memset(&gn_boot, 0, sizeof gn_boot);
    memset(&gn_saved, 0, sizeof gn_saved);
    gn_boot_loaded = 1; gn_saved_loaded = 0; gn_swapped = 0;
    for (i = 0; i < (int)(sizeof cases / sizeof cases[0]); ++i) {
        uint64_t id, digest;
        if (gn_parse_text(&gn_boot, cases[i].text, "art-test") != 1) {
            gw_test_fail("art donor fixture %d refused", i); rc = 1; break;
        }
        gn_build_kinds(&gn_boot);
        ck = gdf_alias_for_profile(&gn_boot.definitions, 0);
        id = gn_boot.p[0].id; digest = gn_boot.definitions.catalog.digest;
        if (ck < 34 || gw_Geno_DefineArtBaseCK(ck) != cases[i].base) {
            gw_test_fail("art donor fixture %d: ck %d base %d, wanted %d",
                         i, ck, gw_Geno_DefineArtBaseCK(ck), cases[i].base);
            rc = 1;
        }
        for (what = 0; what < GN_PRES_KINDS; ++what) {
            if (gw_Geno_DefineHasArt(ck, what, 254) ||
                gw_Geno_DefineArtTex(ck, what, 254) != -1 ||
                gw_Geno_DefineArtOpen(ck, what, 254) != -1) {
                gw_test_fail("undeclared package art was fabricated"); rc = 1;
            }
        }
        if (gn_boot.p[0].id != id || gn_boot.definitions.catalog.digest != digest) {
            gw_test_fail("art lookup changed fighter/catalogue identity"); rc = 1;
        }
        gn_registry_clear(&gn_boot);
    }
    if (gw_Geno_DefineArtBaseCK(0) != -1 || gw_Geno_DefineArtBaseCK(8) != -1 ||
        gw_Geno_DefineArtBaseCK(-1) != -1 || gw_Geno_DefineArtBaseCK(255) != -1) {
        gw_test_fail("non-define acquired an art donor"); rc = 1;
    }
    gn_registry_clear(&gn_boot); gn_registry_clear(&gn_saved);
    gn_boot = boot; gn_saved = saved;
    gn_boot_loaded = loaded; gn_saved_loaded = saved_loaded; gn_swapped = swapped;
    gw_Skins_Suspend(0);
    return rc;
}
