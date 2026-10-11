/* nucleus_fighter_tests.h - tests of the custom-fighter installer (gw_nucleus_fighter.h): a synthetic MexManager-style fighter package (fighter.json + a
 * fighter-shaped HSD archive, stored zip), what it writes, what it refuses, and a small fuzz of damaged zips (no crash; every outcome is a plain refusal or an
 * install). Included by nucleus_core_test.c after nucleus_stage_tests.h (it reuses nst_zip). SYNTHETIC data only. */
#ifndef NUCLEUS_FIGHTER_TESTS_H
#define NUCLEUS_FIGHTER_TESTS_H

#include "../platform/gw_nucleus_fighter.h"

static void run_fighter_tests(void) {
    static uint8_t zipbuf[16384], bad[16384], hsd[2048], aj[256], cdat[256];
    static const char *const syms[] = { "ftDataZz" };
    const char *json_ok =
        "{\"name\":\"Test Zed\",\"soundBank\":12,\"announcerCall\":510008,\"victoryTheme\":22,\"fighterMusic1\":98,\"fighterMusic2\":98,\"items\":[],"
        "\"hasKirbyCostumes\":false,\"kirbyCostumes\":[],"
        "\"costumes\":[{\"name\":\"Normal\",\"file\":{\"visibilityIndex\":0,\"fileName\":\"PlZzNr.dat\",\"jointSymbol\":\"PlyZz5K_Share_joint\",\"materialSymbol\":\"PlyZz5K_Share_matanim_joint\"}}],"
        "\"files\":{\"fighterDataPath\":\"PlZz.dat\",\"fighterDataSymbol\":\"ftDataZz\",\"animFile\":\"PlZzAJ.dat\",\"animCount\":316,\"effectFile\":\"EfKpData.dat\"},"
        "\"functions\":{\"version\":0,\"onLoad\":2148857528,\"onRespawn\":2148857408,\"moveLogicPointer\":2151232000,\"specialN\":2148544088,\"onTwoEntryTable2\":15,\"onFrame\":2148857496}}";
    char *json_far, *json_items, *json_nofile;
    size_t hn = nl_build_hsd(syms, 1, hsd), zn, i;
    nst_entry ents[5];
    nc_post post;
    nc_in_file in;
    nc_loose_env env;
    nc_loose_result res;
    char msg[400], path[400], *text;
    const char *root_dir = "nf_mods";
    int rc, fuzz_ok = 0, fuzz_n = 0;
    unsigned seed = 4242u;

    memset(aj, 0x41, sizeof aj); memset(cdat, 0x42, sizeof cdat);
    memset(&post, 0, sizeof post); memset(&in, 0, sizeof in); memset(&env, 0, sizeof env);
    post.id = 901; post.title = "Test Zed"; post.author = "Tester"; post.page = "https://ssbmnucleus.net/post/901"; post.updated = "2026-10-10T00:00:00Z"; post.installed_at = "2026-10-10T00:00:00Z";
    in.cat = NC_FC_ARCHIVE; in.filename = "pack/test-fighter.zip"; in.stem = "";
    nc_mkdirs(root_dir);

    ents[0].name = "fighter.json"; ents[0].data = (const uint8_t *) json_ok; ents[0].len = strlen(json_ok);
    ents[1].name = "PlZz.dat"; ents[1].data = hsd; ents[1].len = hn;
    ents[2].name = "PlZzAJ.dat"; ents[2].data = aj; ents[2].len = sizeof aj;
    ents[3].name = "PlZzNr.dat"; ents[3].data = cdat; ents[3].len = sizeof cdat;
    zn = nst_zip(ents, 4, zipbuf);
    in.data = zipbuf; in.len = zn;
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    if (rc != 0) printf("fighter install msg: %s | %s", msg, res.first_err);
    CHECK(rc == 0 && res.installed == 1 && !strcmp(res.mods[0], "nucleus-901-fighter"));
    snprintf(path, sizeof path, "%s/nucleus-901-fighter/mexfighter.txt", root_dir); nc_fix_slashes(path);
    text = nc_read_all(path, NULL);
    CHECK(text != NULL);
    if (text) {
        CHECK(strstr(text, "name=Test Zed") && strstr(text, "pl=PlN0385.dat") && strstr(text, "plsym=ftDataZz") && strstr(text, "anim=PlN0385AJ.dat") && strstr(text, "animcount=316"));
        CHECK(strstr(text, "c0=PlN0385C0.dat|PlyZz5K_Share_joint|PlyZz5K_Share_matanim_joint|0"));
        CHECK(strstr(text, "cb0=0x8014F6B8") && strstr(text, "cb3=0x") && strstr(text, "cb4=0x") && strstr(text, "cb23=0x") && !strstr(text, "cb30="));
        CHECK(strstr(text, "victory=22") && strstr(text, "bgm=98,98"));
        free(text);
    }
    snprintf(path, sizeof path, "%s/nucleus-901-fighter/files/PlN0385.dat", root_dir); nc_fix_slashes(path);
    CHECK(nc_file_exists(path));
    snprintf(path, sizeof path, "%s/nucleus-901-fighter/files/PlZz.dat", root_dir); nc_fix_slashes(path);
    CHECK(!nc_file_exists(path));                                    /* the zip's own name is never written (it may be a retail name) */
    snprintf(path, sizeof path, "%s/nucleus-901-fighter/mod.json", root_dir); nc_fix_slashes(path);
    text = nc_read_all(path, NULL);
    CHECK(text && strstr(text, "\"kind\": \"fighter\"") && strstr(text, "post/901") && strstr(text, "\"verdict\": \"partial\"") && strstr(text, "announcer"));
    if (text) { nc_inst inst; CHECK(nc_inst_parse("nucleus-901-fighter", text, &inst) && !strcmp(inst.cat, "cfighter") && inst.compat[0]); }
    free(text);
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);   /* reinstall / update */
    CHECK(rc == 0);

    /* refusals */
    json_far = nst_sub(json_ok, "2148857528", "3000000000");
    ents[0].data = (const uint8_t *) json_far; ents[0].len = strlen(json_far);
    in.len = nst_zip(ents, 4, bad); in.data = bad; post.id = 902;
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "another clone base") != NULL && !nc_is_dir("nf_mods/nucleus-902-fighter"));
    json_items = nst_sub(json_ok, "\"items\":[]", "\"items\":[{}]");
    ents[0].data = (const uint8_t *) json_items; ents[0].len = strlen(json_items);
    in.len = nst_zip(ents, 4, bad); post.id = 903;
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "custom items") != NULL);
    ents[0].data = (const uint8_t *) json_ok; ents[0].len = strlen(json_ok);
    in.len = nst_zip(ents, 3, bad); post.id = 904;                                             /* the costume file is missing */
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "costume file") != NULL);
    ents[1].data = (const uint8_t *) "just text"; ents[1].len = 9;                               /* not an HSD archive */
    in.len = nst_zip(ents, 4, bad); post.id = 905;
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "not a fighter file") != NULL);
    json_nofile = nst_sub(json_ok, "\"fighterDataSymbol\":\"ftDataZz\"", "\"fighterDataSymbol\":\"\"");
    ents[0].data = (const uint8_t *) json_nofile; ents[0].len = strlen(json_nofile); ents[1].data = hsd; ents[1].len = hn;
    in.len = nst_zip(ents, 4, bad); post.id = 906;
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "does not name a fighter file") != NULL);
    in.data = (const uint8_t *) "PK not really"; in.len = 13;
    rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && msg[0]);
    free(json_far); free(json_items); free(json_nofile);

    /* fuzz: single-byte flips and truncations of the good package never crash; each is a refusal or an install */
    ents[0].data = (const uint8_t *) json_ok; ents[0].len = strlen(json_ok); ents[1].data = hsd; ents[1].len = hn;
    zn = nst_zip(ents, 4, zipbuf);
    post.id = 907;
    for (i = 0; i < 400; ++i) {
        size_t cut = zn;
        memcpy(bad, zipbuf, zn);
        seed = seed * 1103515245u + 12345u;
        if (i % 4 == 3) cut = (seed >> 8) % zn; else { bad[(seed >> 8) % zn] ^= (uint8_t) (1u + (seed >> 24)); }
        in.data = bad; in.len = cut; ++fuzz_n;
        rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
        if ((rc == 0 || rc == -1) && msg[0]) ++fuzz_ok;
    }
    CHECK(fuzz_n == 400 && fuzz_ok == 400);

    CHECK(nc_loose_remove(root_dir, 901) == 1 && !nc_is_dir("nf_mods/nucleus-901-fighter"));
    nc_rmtree(root_dir);
}

#endif
