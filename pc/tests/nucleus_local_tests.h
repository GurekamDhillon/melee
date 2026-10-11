/* Local archive dispatch and hostile-archive regressions. Synthetic bytes only. */
#ifndef NUCLEUS_LOCAL_TESTS_H
#define NUCLEUS_LOCAL_TESTS_H

static int nlz_commit_once(void *user) { return (*(int *) user)++ == 0; }

static void run_local_zip_tests(void) {
    static uint8_t z[16384], bad[16384], dat[2048], hps[64];
    const char *root = "nlz_mods";
    const char *const costume[] = { "PlyFox5KNr_Share_joint", "PlyFox5KNr_Share_matanim_joint" };
    const char *const stage[] = { "map_head", "coll_data", "grGroundParam" };
    nc_loose_env env = {0};
    nc_loose_result res;
    nst_entry e[6];
    char msg[600], path[800], local_id[64], *text;
    size_t zn, dn, cd;
    int rc, k;
    env.ref = nl_env_ref; env.exists = nl_env_exists;
    nc_mkdirs(root);
    dn = nl_build_hsd(costume, 2, dat);
    e[0].name = "folder/PlFxNr.dat"; e[0].data = dat; e[0].len = dn;
    e[1].name = "cover.jpeg"; e[1].data = (const uint8_t *) "art"; e[1].len = 3;
    e[2] = e[1]; e[2].name = "cover.gxtex";
    zn = nst_zip(e, 3, z);
    env.can_commit = nst_deny_commit;
    CHECK(nlz_install_data(root, "skin.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "online") && res.installed == 0);
    env.can_commit = NULL;
    CHECK(nlz_install_data(root, "skin.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) == 0);
    snprintf(path, sizeof path, "%s/%s/mod.json", root, res.mods[0]);
    text = nc_read_all(path, NULL);
    CHECK(text && strstr(text, "\"kind\": \"skin\"") && strstr(text, "\"source\": \"local file\"") && !strstr(text, "\"nucleus\""));
    free(text);
    {
        uint8_t second[2048];
        const char *const mario[] = { "PlyMario5KNr_Share_joint" };
        int commits = 0;
        e[1].name = "PlMrNr.dat"; e[1].data = second; e[1].len = nl_build_hsd(mario, 1, second);
        zn = nst_zip(e, 2, z);
        env.user = &commits; env.can_commit = nlz_commit_once;
        CHECK(nlz_install_data(root, "two-fighters.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && res.installed == 1 && strstr(msg, "then stopped") && strstr(msg, "Restart"));
        CHECK(res.nmods == 1 && nc_enabled_query(root, res.mods[0]));
        env.user = NULL; env.can_commit = NULL;
    }
    dn = nl_build_hsd(stage, 3, dat);
    e[0].name = "stage.json"; e[0].data = (const uint8_t *) "{\"name\":\"Local Hill\",\"fileName\":\"GrLocal.dat\"}"; e[0].len = strlen((const char *) e[0].data);
    e[1].name = "GrLocal.dat"; e[1].data = dat; e[1].len = dn;
    zn = nst_zip(e, 2, z);
    env.can_commit = nst_deny_commit;
    CHECK(nlz_install_data(root, "stage.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "online") && res.installed == 0);
    env.can_commit = NULL;
    rc = nlz_install_data(root, "stage.zip", z, zn, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc == 0);
    snprintf(path, sizeof path, "%s/%s/mexstage.txt", root, res.mods[0]); text = nc_read_all(path, NULL);
    CHECK(text && strstr(text, "name=Local Hill") && !strstr(text, "post=")); free(text);
    /* A second local audio mod replaces the same disc file and turns the first off. */
    nl_audio_ok = 1; memset(hps, 0, sizeof hps); memcpy(hps, " HALPST", 7);
    e[0].name = "jungle.hps"; e[0].data = hps; e[0].len = sizeof hps;
    zn = nst_zip(e, 1, z);
    env.can_commit = nst_deny_commit;
    CHECK(nlz_install_data(root, "audio.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && strstr(msg, "online") && res.installed == 0);
    env.can_commit = NULL;
    CHECK(nlz_install_data(root, "audio.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) == 0);
    nc_copy(local_id, sizeof local_id, res.mods[0]);
    hps[40] = 1; zn = nst_zip(e, 1, z);
    CHECK(nlz_install_data(root, "audio2.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) == 0);
    CHECK(nc_enabled_query(root, local_id) == 0 && nc_enabled_query(root, res.mods[0]) == 1 && res.nturned_off == 1);
    snprintf(path, sizeof path, "%s/%s/mod.json", root, res.mods[0]); text = nc_read_all(path, NULL);
    CHECK(text && strstr(text, local_id) && strstr(text, "\"local\"") && !strstr(text, "\"nucleus\"")); free(text);
    /* Valid audio alongside an invalid menu must refuse before installing either. */
    e[1].name = "MnSlChr.dat"; e[1].data = (const uint8_t *) "invalid"; e[1].len = 7;
    zn = nst_zip(e, 2, z);
    CHECK(nlz_install_data(root, "mixed.zip", z, zn, &env, &res, msg, sizeof msg, nst_log) < 0 && res.installed == 0);

    e[0].name = "readme.txt"; e[0].data = (const uint8_t *) "hello"; e[0].len = 5;
    zn = nst_zip(e, 1, z); cd = nz_u32(z + zn - 6);
    CHECK(nc_zip_list(z, zn, NULL, 0, msg, sizeof msg) == 1);
    /* A PK EOCD signature inside a valid comment is not a second directory. */
    memcpy(bad, z, zn); memset(bad + zn, 0, 24); nst_le16(bad + zn - 2, 24); nst_le32(bad + zn, 0x06054B50);
    { uint8_t *out = NULL; size_t len;
      CHECK(nc_zip_extract(bad, zn + 24, "readme.txt", &out, &len, 16, msg, sizeof msg) == 0 && len == 5); free(out); }
    memcpy(bad, z, zn); nst_le32(bad + cd + 24, NZ_MAX_FILE + 1);
    CHECK(nc_zip_list(bad, zn, NULL, 0, msg, sizeof msg) < 0 && strstr(msg, "too large"));
    CHECK(nc_zip_list(z, NZ_MAX_ARCHIVE + 1, NULL, 0, msg, sizeof msg) < 0 && strstr(msg, "160 MiB"));
    memcpy(bad, z, zn); nst_le32(bad + cd + 42, 0xFFFFFFF0u);
    CHECK(nc_zip_list(bad, zn, NULL, 0, msg, sizeof msg) < 0);
    memcpy(bad, z, zn); nst_le16(bad + cd + 8, 1);
    CHECK(nc_zip_list(bad, zn, NULL, 0, msg, sizeof msg) < 0 && strstr(msg, "encrypted"));
    memcpy(bad, z, zn); nst_le32(bad + cd + 38, 0120777u << 16);
    CHECK(nc_zip_list(bad, zn, NULL, 0, msg, sizeof msg) < 0 && strstr(msg, "symlink"));
    memcpy(bad, z, zn); bad[30 + strlen(e[0].name)] ^= 1;
    { uint8_t *out = NULL; size_t len;
      CHECK(nc_zip_extract(bad, zn, "readme.txt", &out, &len, 16, msg, sizeof msg) < 0 && strstr(msg, "checksum")); free(out); }
    e[1] = e[0]; zn = nst_zip(e, 2, z);
    CHECK(nc_zip_list(z, zn, NULL, 0, msg, sizeof msg) < 0 && strstr(msg, "duplicate"));
    { const char *paths[] = { "/absolute.dat", "C:/escape.dat", "x/../../escape.dat", "x\\..\\escape.dat", "x/file.dat:stream", "x./file.dat" };
      for (k = 0; k < 6; ++k) { e[0].name = paths[k]; zn = nst_zip(e, 1, z); CHECK(nc_zip_list(z, zn, NULL, 0, msg, sizeof msg) < 0 && strstr(msg, "path")); } }
    CHECK(!nlz_path_ok("https://example.invalid/f.zip") && !nlz_path_ok("\\\\server\\f.zip") && !nlz_path_ok("//server/f.zip"));
    nc_rmtree(root);
}
#endif
