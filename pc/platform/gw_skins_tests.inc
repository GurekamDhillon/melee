/* gw_skins_tests.inc - headless tests for the skin registry's engine half (included by geno_registry.c, after
 * gw_skins_boot.inc). The pure logic is covered natively (pc/tests/skins_core_test.c); the installed tables by
 * test_skins_installed_tables (gw_tests_core.c) when a run mounts skin mods. */

#ifdef _WIN32
#include <direct.h>
#define SKT_MKDIR(p) _mkdir(p)
#else
#include <sys/stat.h>
#define SKT_MKDIR(p) mkdir((p), 0777)
#endif

static char skt_msgs[40][300];
static int skt_nmsg;
static void skt_sink(const char *m) { if (skt_nmsg < 40) snprintf(skt_msgs[skt_nmsg++], 300, "%s", m); }
static int skt_said(const char *needle) {
    int i;
    for (i = 0; i < skt_nmsg; ++i) if (strstr(skt_msgs[i], needle)) return 1;
    return 0;
}
static void skt_touch(const char *root, const char *rel) {
    char path[MAX_PATH], *q;
    FILE *f;
    snprintf(path, sizeof path, "%s\\%s", root, rel);
    for (q = path + strlen(root) + 1; *q; ++q) {
        if (*q == '/' || *q == '\\') {
            char c = *q;
            *q = '\0';
            SKT_MKDIR(path);
            *q = c == '/' ? '\\' : c;
        }
    }
    f = fopen(path, "wb");
    if (f) { fputs("x", f); fclose(f); }
}
static int skt_base5(int fk) { return fk == 0 ? 5 : 0; }

static int test_skins_parse(void) {
    char root[MAX_PATH];
    const char *tmp = getenv("TEMP");
    sk_registry saved = sk_reg;
    void (*saved_sink)(const char *) = sk_sink;
    jdoc *d = (jdoc *) malloc(sizeof *d);
    int rc = 0, i;
    static const struct { const char *json, *say; } bad[] = {
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\",\"bogus\":1}]}}", "bogus" },
        { "{\"skin\":{\"format\":2,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "newer" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\",\"mex\":\"PlWf.dat\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "exactly one" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"nobody\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "not a retail fighter" },
        { "{\"skin\":{\"format\":1,\"target\":{\"mex\":\"PlZz.dat\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "not installed" },
        { "{\"skin\":{\"format\":1,\"target\":{\"geno\":\"nope\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "not installed" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"file\":\"skins/a/missing.dat\",\"joint\":\"J\"}]}}", "not in the mod's files" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"file\":\"../x.dat\",\"joint\":\"J\"}]}}", "plain relative path" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"name\":\"Caf\xC3\xA9\",\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "printable ASCII" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\",\"team\":\"purple\"}]}}", "team must be" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\",\"kirby_hat\":9}]}}", "kirby_hat" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[]}}", "at least one" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"costumes\":[{\"file\":\"skins/a/x.dat\"}]}}", "needs \"joint\"" },
        { "{\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"bogus\":1,\"costumes\":[{\"file\":\"skins/a/x.dat\",\"joint\":\"J\"}]}}", "key \"bogus\"" },
        { "{\"name\":\"no skin block\"}", "needs a \"skin\" object" },
        { "{ not json", "not valid JSON" },
    };
    if (!d) return 1;
    snprintf(root, sizeof root, "%s\\gw_skins_test_%u", tmp ? tmp : ".", (unsigned) time(NULL));
    SKT_MKDIR(root);
    skt_touch(root, "skins/a/x.dat");
    skt_touch(root, "skins/a/csp.gxtex");
    memset(&sk_reg, 0, sizeof sk_reg);
    for (i = 0; i < SK_FK_MAX; ++i) sk_reg.first[i] = -1;
    sk_sink = skt_sink;
    for (i = 0; i < (int) (sizeof bad / sizeof bad[0]); ++i) {
        int took;
        skt_nmsg = 0;
        took = sk_read_text("t-bad", root, bad[i].json, d);
        if (took != 0 || !skt_said(bad[i].say)) {
            gw_test_fail("skins: refusal %d (%s): took %d, messages %d, wanted \"%s\"", i, bad[i].say, took, skt_nmsg, bad[i].say);
            rc = 1;
        }
    }
    /* a good pack: three costumes on Mario; an art file that is missing is dropped with a message, the costume stays */
    skt_nmsg = 0;
    {
        int took = sk_read_text("t-good", root,
            "{\"name\":\"Pack\",\"version\":\"1.2.3\",\"kind\":\"skin\",\"skin\":{\"format\":1,\"target\":{\"retail\":\"mario\"},\"order\":3,\"costumes\":["
            "{\"name\":\"Red One\",\"file\":\"skins/a/x.dat\",\"joint\":\"J1\",\"matanim\":\"M1\",\"team\":\"red\",\"like\":2,\"csp\":\"skins/a/csp.gxtex\",\"stock\":\"skins/a/gone.gxtex\"},"
            "{\"file\":\"skins/a/x.dat\",\"joint\":\"J2\"},"
            "{\"name\":\"Blue\",\"file\":\"skins/a/x.dat\",\"joint\":\"J3\",\"team\":\"blue\"}]}}", d);
        if (took != 3) { gw_test_fail("skins: good pack took %d of 3", took); rc = 1; }
    }
    sk_finalize(skt_base5, sk_name_fn);
    if (sk_total(0) != 8 || sk_added(0) != 3) { gw_test_fail("skins: Mario total %d added %d", sk_total(0), sk_added(0)); rc = 1; }
    else {
        const sk_costume *r5 = sk_row(0, 5), *r6 = sk_row(0, 6), *r7 = sk_row(0, 7);
        if (strcmp(r5->name, "Red One") || strcmp(r6->name, "Pack 2") || strcmp(r7->name, "Blue") || r5->like != 2 || strcmp(r5->matanim, "M1") ||
            !r5->csp[0] || r5->stock[0] || sk_team(0, 0) != 5 || sk_team(0, 1) != 7 || sk_team(0, 2) != -1 || r5->order != 3 ||
            r5->id != sk_identity("t-good", 0, "1.2.3")) {
            gw_test_fail("skins: good pack fields wrong (%s / %s / %s)", r5->name, r6->name, r7->name);
            rc = 1;
        }
        if (!skt_said("gone.gxtex")) { gw_test_fail("skins: a missing art file must be reported"); rc = 1; }
    }
    free(sk_reg.c);
    sk_reg = saved;
    sk_sink = saved_sink;
    free(d);
    return rc;
}

void gw_skins_tests_register(void) {
    gw_test_register("skins_parse", test_skins_parse);
}
