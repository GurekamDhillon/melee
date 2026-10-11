/* nucleus_fighter_tests.h - tests of the custom-fighter installer (gw_nucleus_fighter.h): a synthetic MexManager-style fighter package (fighter.json + a
 * fighter-shaped HSD archive, stored zip), what it writes, what it refuses, and a small fuzz of damaged zips (no crash; every outcome is a plain refusal or an
 * install). Included by nucleus_core_test.c after nucleus_stage_tests.h (it reuses nst_zip). SYNTHETIC data only. */
#ifndef NUCLEUS_FIGHTER_TESTS_H
#define NUCLEUS_FIGHTER_TESTS_H

#include "../platform/gw_nucleus_fighter.h"
#include "../platform/gw_mods_zip.h"

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
    env.can_commit = nst_deny_commit;
    CHECK(nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "online") && !nc_is_dir("nf_mods/nucleus-901-fighter"));
    env.can_commit = NULL;
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

    /* Repeated references are separate output files. A small directory must
     * not multiply a costume past the whole-package expansion budget. */
    {
        const char *one = "{\"file\":{\"fileName\":\"PlZzNr.dat\",\"jointSymbol\":\"PlyZz5K_Share_joint\"}}";
        nj_buf many = {0};
        char *large_json;
        size_t biglen = 24u * 1024 * 1024;
        uint8_t *big = (uint8_t *) calloc(1, biglen), *large_zip = (uint8_t *) malloc(biglen + 16384);
        const char *start = strstr(json_ok, "\"costumes\":["), *end = strstr(start, "],");
        int q;
        nj_puts(&many, "\"costumes\":[");
        for (q = 0; q < 9; ++q) { if (q) nj_puts(&many, ","); nj_puts(&many, one); }
        nj_puts(&many, "]");
        large_json = (char *) malloc(strlen(json_ok) + many.n + 1);
        snprintf(large_json, strlen(json_ok) + many.n + 1, "%.*s%s%s", (int) (start - json_ok), json_ok, many.s, end + 1);
        ents[0].data = (const uint8_t *) large_json; ents[0].len = strlen(large_json);
        ents[3].data = big; ents[3].len = biglen;
        in.data = large_zip; in.len = nst_zip(ents, 4, large_zip); post.id = 908;
        rc = nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
        CHECK(rc < 0 && strstr(msg, "200 MiB") && !nc_is_dir("nf_mods/nucleus-908-fighter"));
        free(large_json); free(many.s); free(big); free(large_zip);
        ents[3].data = cdat; ents[3].len = sizeof cdat;
    }

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
    /* Local input uses the very same MexManager validator, with no fabricated post. */
    {
        char local_id[64], second_id[64];
        rc = nlz_install_data(root_dir, "My fighter.zip", zipbuf, zn, &env, &res, msg, sizeof msg, nst_log);
        CHECK(rc == 0 && res.installed == 1);
        snprintf(local_id, sizeof local_id, "%s", res.mods[0]);
        CHECK(!strncmp(local_id, "local-", 6));
        snprintf(path, sizeof path, "%s/%s/mod.json", root_dir, local_id);
        text = nc_read_all(path, NULL);
        CHECK(text && strstr(text, "\"source\": \"local file\"") && !strstr(text, "\"nucleus\"") && !strstr(text, "ssbmnucleus.net"));
        free(text);
        snprintf(path, sizeof path, "%s/%s/mexfighter.txt", root_dir, local_id);
        text = nc_read_all(path, NULL);
        CHECK(text && strstr(text, "pl=PlL") && !strstr(text, "post="));
        free(text);
        CHECK(nlz_install_data(root_dir, "renamed.zip", zipbuf, zn, &env, &res, msg, sizeof msg, nst_log) == 0);
        CHECK_STR(res.mods[0], local_id);
        aj[0] ^= 1; zn = nst_zip(ents, 4, zipbuf);
        CHECK(nlz_install_data(root_dir, "My fighter.zip", zipbuf, zn, &env, &res, msg, sizeof msg, nst_log) == 0);
        snprintf(second_id, sizeof second_id, "%s", res.mods[0]);
        CHECK(strcmp(second_id, local_id) != 0);
        aj[0] ^= 1; zn = nst_zip(ents, 4, zipbuf);

        /* Count local, Nucleus AND manually installed fighters; reinstall at cap still works. */
        for (i = 0; i < 28; ++i) {
            snprintf(path, sizeof path, "%s/manual-%u", root_dir, (unsigned) i); nc_mkdirs(path);
            snprintf(path, sizeof path, "%s/manual-%u/mexfighter.txt", root_dir, (unsigned) i);
            nc_write_file(path, "name=Fixture\n", 13);
        }
        CHECK(nlz_install_data(root_dir, "same.zip", zipbuf, zn, &env, &res, msg, sizeof msg, nst_log) == 0);
        aj[1] ^= 1; zn = nst_zip(ents, 4, zipbuf);
        CHECK(nlz_install_data(root_dir, "new.zip", zipbuf, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "31"));
        in.data = zipbuf; in.len = zn;
        CHECK(nc_fighter_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "31"));
        aj[1] ^= 1; zn = nst_zip(ents, 4, zipbuf);
        for (i = 0; i < 28; ++i) { snprintf(path, sizeof path, "%s/manual-%u", root_dir, (unsigned) i); nc_rmtree(path); }
        snprintf(path, sizeof path, "%s/%s", root_dir, local_id); nc_rmtree(path);
        snprintf(path, sizeof path, "%s/%s", root_dir, second_id); nc_rmtree(path);

        ents[4].name = "../escape.dat"; ents[4].data = hsd; ents[4].len = hn;
        zn = nst_zip(ents, 5, zipbuf);
        CHECK(nlz_install_data(root_dir, "unsafe.zip", zipbuf, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "path"));
        ents[4].name = "setup.exe"; zn = nst_zip(ents, 5, zipbuf);
        CHECK(nlz_install_data(root_dir, "unsafe.zip", zipbuf, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "file type"));
        CHECK(nlz_install_data(root_dir, "bad.zip", (const uint8_t *) "bad", 3, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "zip"));
        CHECK(nlz_install_file(root_dir, "missing.zip", &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "open"));
        zn = nst_zip(ents, 4, zipbuf);
    }
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
