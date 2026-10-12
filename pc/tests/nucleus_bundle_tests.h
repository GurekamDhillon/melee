/* nucleus_bundle_tests.h - the fixes for the Goku post (https://ssbmnucleus.net/post/17621): a post the bulk list captured before Nucleus processed it (no files, no pictures)
 * is refetched alone and merged, a GD mod zip (melee-pc.exe + mods/<id>/ folders) installs its mod folders and never the exe, and the pictures the site serves (JPEG, big PNG)
 * decode. Synthetic bytes and a fake API only; no network. Included by nucleus_core_test.c after its CHECK macros. */
#ifndef NUCLEUS_BUNDLE_TESTS_H
#define NUCLEUS_BUNDLE_TESTS_H

#include "nucleus_image_fixture.h"

/* ---- a fake API: the list shows post 17621 as the bulk sync saw it (files_total 0); GET /mods/17621 has the archive ---- */
static int bt_calls_single_17621, bt_calls_single_18000, bt_calls_list;
static int64_t bt_clock;

static const char *const bt_thin =
    "{\"id\":17621,\"type\":\"other\",\"title\":\"Goku Custom Character\",\"author\":\"peacerino\",\"description\":\"Goku\",\"tags\":[\"Assets/Other\",\"custom fighter\"],"
    "\"page_url\":\"https://ssbmnucleus.net/post/17621/goku\",\"screenshots\":[],\"created_at\":\"2026-10-07T09:28:20.495\",\"updated_at\":\"2026-10-07T09:34:55.483\","
    "\"download_count\":1,\"like_count\":0,\"files_total\":0,\"files\":[]}";
static const char *const bt_full =
    "{\"id\":17621,\"type\":\"other\",\"title\":\"Goku Custom Character\",\"author\":\"peacerino\",\"description\":\"Goku\",\"tags\":[\"Assets/Other\",\"custom fighter\"],"
    "\"page_url\":\"https://ssbmnucleus.net/post/17621/goku\",\"screenshots\":[\"https://media.ssbmnucleus.net/posts/2026/10/post_17621/shot_0.jpg\"],"
    "\"created_at\":\"2026-10-07T09:28:20.495321Z\",\"updated_at\":\"2026-10-07T09:34:55.483711Z\",\"download_count\":1,\"like_count\":0,\"files\":[],"
    "\"archive_files\":[{\"filename\":\"melee-pc.exe\",\"size\":12651520},{\"filename\":\"mods/goku-xv2/mod.json\",\"size\":900},{\"filename\":\"mods/goku-xv2/files/GnGoku.dat\",\"size\":5},"
    "{\"filename\":\"mods/geno-lab/mod.json\",\"size\":300},{\"filename\":\"mods/geno-lab/scripts/lab.lua\",\"size\":10},{\"filename\":\"mods/bad name/mod.json\",\"size\":1},"
    "{\"filename\":\"GOKU-README.txt\",\"size\":20},{\"filename\":\"nucleus_media/x/screenshot_0.png\",\"size\":9}]}";
static const char *const bt_empty_18000 =
    "{\"id\":18000,\"type\":\"other\",\"title\":\"Empty post\",\"author\":\"x\",\"description\":\"\",\"tags\":[],\"page_url\":\"\",\"screenshots\":[],"
    "\"created_at\":\"2026-10-07T10:00:00.000\",\"updated_at\":\"2026-10-07T10:00:00.000\",\"download_count\":0,\"like_count\":0,\"files_total\":0,\"files\":[]}";

static int bt_fetch(void *u, const char *url, nc_resp *r) {
    char *body;
    (void) u;
    r->status = 200;
    if (strstr(url, "/mods/17621")) { ++bt_calls_single_17621; body = (char *) malloc(strlen(bt_full) + 16); sprintf(body, "{\"data\":%s}", bt_full); }
    else if (strstr(url, "/mods/18000")) { ++bt_calls_single_18000; body = (char *) malloc(strlen(bt_empty_18000) + 16); sprintf(body, "{\"data\":%s}", bt_empty_18000); }
    else if (strstr(url, "/mods/removed")) body = strdup("{\"data\":[],\"next_cursor\":null}");
    else if (strstr(url, "updated_since=")) body = strdup("{\"data\":[],\"next_cursor\":null,\"total\":2}");
    else { ++bt_calls_list; body = (char *) malloc(strlen(bt_thin) + strlen(bt_empty_18000) + 64); sprintf(body, "{\"data\":[%s,%s],\"next_cursor\":null,\"total\":2}", bt_thin, bt_empty_18000); }
    r->body = body; r->len = strlen(body);
    return 0;
}
static int64_t bt_now(void *u) { (void) u; return bt_clock; }
static void bt_sleep(void *u, int ms) { (void) u; (void) ms; }

/* ---- a GD game folder as a zip ---- */
static size_t bt_exe(uint8_t *out, uint32_t stamp) {          /* the first bytes of an exe: MZ, e_lfanew 0x80, "PE\0\0", TimeDateStamp at +8 */
    memset(out, 0, 256);
    out[0] = 'M'; out[1] = 'Z'; out[0x3C] = 0x80;
    memcpy(out + 0x80, "PE\0\0", 4);
    out[0x88] = (uint8_t) stamp; out[0x89] = (uint8_t) (stamp >> 8); out[0x8A] = (uint8_t) (stamp >> 16); out[0x8B] = (uint8_t) (stamp >> 24);
    return 256;
}
static int bt_engine_miss(const char *json, char *why, size_t cap) {
    if (strstr(json, "future_feature")) { snprintf(why, cap, "needs a newer game (engine future_feature 9)"); return 1; }
    return 0;
}

static void run_bundle_tests(void) {
    static uint8_t z[65536], exe[256], big[4096];
    nc_catalog cat;
    nc_sync st;
    nc_sync_io io;
    nc_mod *m;
    int at, i, rc;
    char msg[600], path[800], *text;

    /* ---- 1. the thin list record, then the single-post record ---- */
    memset(&cat, 0, sizeof cat);
    memset(&st, 0, sizeof st);
    memset(&io, 0, sizeof io);
    io.fetch = bt_fetch; io.now = bt_now; io.sleep_ms = bt_sleep;
    bt_clock = 1791360000LL + 3600;                                 /* 2026-10-07 + a bit: the posts are "recent" */
    {
        nj_doc d;
        int root = nj_parse(&d, bt_thin);
        CHECK(root >= 0 && nc_cat_apply(&cat, &d, root) == 1);
        nj_free(&d);
        at = nc_cat_find(&cat, 17621);
        CHECK(at >= 0 && cat.m[at].nf == 0 && cat.m[at].nf_total == 0 && !cat.m[at].bundle[0]);
        CHECK_STR(nc_why_not(&cat, &cat.m[at]), "Nucleus lists no files for this post yet.");
        root = nj_parse(&d, bt_full);
        CHECK(root >= 0 && nc_cat_apply(&cat, &d, root) == 2);
        nj_free(&d);
        at = nc_cat_find(&cat, 17621);
        m = &cat.m[at];
        CHECK_STR(m->bundle, "goku-xv2,geno-lab");                  /* "bad name" is not a mod folder id; the root exe and the media dir are not mods */
        CHECK(m->bexe == 1 && m->nshots == 1 && nc_installable_mod(&cat, m) && m->tab == NC_TAB_OTHER);
        /* the bulk list delivers the thin copy again (the 15-minute overlap): it must not wipe what the refetch found */
        root = nj_parse(&d, bt_thin);
        CHECK(root >= 0 && nc_cat_apply(&cat, &d, root) == 2);
        nj_free(&d);
        m = &cat.m[nc_cat_find(&cat, 17621)];
        CHECK_STR(m->bundle, "goku-xv2,geno-lab");
        CHECK(m->nshots == 1 && nc_installable_mod(&cat, m));
        /* the cache round trip keeps the bundle */
        CHECK(nc_cat_save(&cat, "nb_cat.jsonl") == 0);
        nc_cat_free(&cat);
        memset(&cat, 0, sizeof cat);
        CHECK(nc_cat_load(&cat, "nb_cat.jsonl") == 1);
        m = &cat.m[nc_cat_find(&cat, 17621)];
        CHECK_STR(m->bundle, "goku-xv2,geno-lab");
        CHECK(m->bexe == 1 && nc_installable_mod(&cat, m));
        remove("nb_cat.jsonl");
        nc_cat_free(&cat);
        memset(&cat, 0, sizeof cat);
    }

    /* ---- 2. the sync re-asks for recent posts that listed no files, each at most NC_RECHECK_TRIES times ---- */
    st.full = 0;
    bt_calls_single_17621 = bt_calls_single_18000 = bt_calls_list = 0;
    CHECK(nc_sync_run(&st, &cat, &io, 1) == 0);
    at = nc_cat_find(&cat, 17621);
    CHECK(at >= 0 && cat.m[at].bundle[0] && cat.m[at].nf == 0 && bt_calls_single_17621 == 1);
    CHECK(bt_calls_single_18000 == 1);
    for (i = 0; i < 5; ++i) { bt_clock += 700; CHECK(nc_sync_run(&st, &cat, &io, 0) == 0); }
    CHECK(bt_calls_single_17621 == 1);                              /* it has its files now: never asked again */
    CHECK(bt_calls_single_18000 == NC_RECHECK_TRIES);               /* a post that stays empty: three tries, then left alone */
    CHECK(bt_calls_list == 1);
    nc_cat_free(&cat);

    /* ---- 3. the pictures the site serves ---- */
    {
        uint8_t *px = NULL, *gx = NULL;
        size_t gl = 0;
        int w = 0, h = 0;
        char err[100];
        CHECK(ni_decode(fx_jpg_small, fx_jpg_small_len, &px, &w, &h, err, sizeof err) == 0 && w == 32 && h == 16);
        CHECK(px[(4 * 32 + 4) * 4] > 180 && px[(4 * 32 + 4) * 4 + 2] < 90 && px[(4 * 32 + 28) * 4] < 90 && px[(4 * 32 + 28) * 4 + 2] > 180 && px[(4 * 32 + 4) * 4 + 3] == 255);
        ni_free_rgba(px, fx_jpg_small);
        CHECK(ni_make(fx_jpg_small, fx_jpg_small_len, NI_SHOT, &gx, &gl, err, sizeof err) == 0 && nc_be32(gx + 12) == 256 && nc_be32(gx + 16) == 144);
        free(gx);
        /* a 3000 x 2000 PNG was "PNG unreadable (too large)"; it is shrunk into the canvas now */
        CHECK(ni_make(fx_png_big, fx_png_big_len, NI_THUMB, &gx, &gl, err, sizeof err) == 0 && nc_be32(gx + 12) == 96 && nc_be32(gx + 16) == 72);
        free(gx);
        CHECK(ni_make(fx_jpg_small, 40, NI_THUMB, &gx, &gl, err, sizeof err) < 0);               /* a cut-off JPEG is refused, not drawn */
        CHECK(ni_make((const uint8_t *) "RIFF\x20\0\0\0WEBPVP8 not decoded here at all", 40, NI_THUMB, &gx, &gl, err, sizeof err) < 0);
    }

    /* ---- 4. the GD mod zip ---- */
    {
        const char *root = "nb_mods";
        nst_entry e[16];
        nc_loose_env env = {0};
        nc_bundle_env benv;
        nc_bundle_result res;
        nc_post post = {0};
        size_t zn;
        int ne = 0;
        static const char *const modjson_a = "{\"id\":\"syn-fighter\",\"name\":\"Syn\",\"kind\":\"script\",\"gameplay\":true}";
        static const char *const modjson_b = "{\"id\":\"geno-lab\",\"name\":\"Lab\",\"kind\":\"script\",\"engine\":{\"future_feature\":9}}";
        static const char *const modjson_c = "{\"id\":\"own-mod\",\"name\":\"Own\"}";
        nc_mkdirs(root);
        nc_mkdirs("nb_mods/own-mod");
        nc_write_file("nb_mods/own-mod/mod.json", "{\"id\":\"own-mod\",\"name\":\"mine\"}", 30);
        nc_write_file("nb_mods/enabled.txt", "other\n", 6);
        memset(big, 7, sizeof big);
        bt_exe(exe, 1790000000u + 5000u);                            /* the zip's exe: built 5000 s after the player's */
        e[ne].name = "melee-pc.exe"; e[ne].data = exe; e[ne++].len = 256;
        e[ne].name = "mods/syn-fighter/mod.json"; e[ne].data = (const uint8_t *) modjson_a; e[ne++].len = strlen(modjson_a);
        e[ne].name = "mods/syn-fighter/files/a.dat"; e[ne].data = big; e[ne++].len = 4096;
        e[ne].name = "mods/syn-fighter/lua/x.lua"; e[ne].data = (const uint8_t *) "print(1)"; e[ne++].len = 8;
        e[ne].name = "mods/syn-fighter/tools/x.py"; e[ne].data = (const uint8_t *) "print(1)"; e[ne++].len = 8;
        e[ne].name = "mods/geno-lab/mod.json"; e[ne].data = (const uint8_t *) modjson_b; e[ne++].len = strlen(modjson_b);
        e[ne].name = "mods/geno-lab/scripts/lab.lua"; e[ne].data = (const uint8_t *) "return 1"; e[ne++].len = 8;
        e[ne].name = "mods/own-mod/mod.json"; e[ne].data = (const uint8_t *) modjson_c; e[ne++].len = strlen(modjson_c);
        e[ne].name = "GOKU-README.txt"; e[ne].data = (const uint8_t *) "hi"; e[ne++].len = 2;
        e[ne].name = "MANIFEST.sha256"; e[ne].data = (const uint8_t *) "00"; e[ne++].len = 2;
        e[ne].name = "nucleus_media/x/screenshot_0.png"; e[ne].data = (const uint8_t *) "png"; e[ne++].len = 3;
        zn = nst_zip(e, ne, z);
        CHECK(zn > 0 && nc_bundle_detect(z, zn));
        post.id = 17621; post.title = "Goku"; post.author = "peacerino"; post.page = "https://ssbmnucleus.net/post/17621"; post.updated = "2026-10-07"; post.installed_at = "2026-10-07T00:00:00Z"; post.stage = "";
        benv.host_stamp = 1790000000u; benv.engine_miss = bt_engine_miss;

        /* the player went online while it ran: nothing is written */
        env.can_commit = nst_deny_commit;
        rc = nc_bundle_install(root, &post, z, zn, &env, &benv, &res, msg, sizeof msg, nst_log);
        CHECK(rc < 0 && res.installed == 0 && strstr(msg, "online") && !nc_is_dir("nb_mods/syn-fighter") && !nc_is_dir("nb_mods/.staging-syn-fighter"));
        env.can_commit = NULL;

        rc = nc_bundle_install(root, &post, z, zn, &env, &benv, &res, msg, sizeof msg, nst_log);
        CHECK(rc == 0 && res.installed == 2 && res.kept == 1);
        CHECK_STR(res.ids[0], "syn-fighter");
        CHECK_STR(res.ids[1], "geno-lab");
        CHECK_STR(res.kept_ids[0], "own-mod");
        CHECK(strstr(msg, "Installed 2 mods (syn-fighter, geno-lab)") && strstr(msg, "Restart"));
        text = nc_read_all("nb_mods/syn-fighter/mod.json", NULL);
        CHECK(text && strstr(text, "syn-fighter"));
        free(text);
        text = nc_read_all("nb_mods/syn-fighter/files/a.dat", NULL);
        CHECK(text != NULL);
        free(text);
        text = nc_read_all("nb_mods/geno-lab/scripts/lab.lua", NULL);
        CHECK(text && !strcmp(text, "return 1"));
        free(text);
        text = nc_read_all("nb_mods/syn-fighter/.nucleus-bundle", NULL);
        CHECK(text && !strcmp(text, "post:17621"));
        free(text);
        /* the exe is never written anywhere; neither are programs or checksum lists; the player's own mod is untouched */
        CHECK(!nc_is_dir("nb_mods/melee-pc.exe"));
        for (i = 0; i < 3; ++i) {
            static const char *const gone[] = { "nb_mods/melee-pc.exe", "nb_mods/syn-fighter/tools/x.py", "nb_mods/MANIFEST.sha256" };
            FILE *fp = fopen(gone[i], "rb");
            CHECK(fp == NULL);
            if (fp) fclose(fp);
        }
        text = nc_read_all("nb_mods/own-mod/mod.json", NULL);
        CHECK(text && strstr(text, "mine"));
        free(text);
        /* the zip's exe is newer than the running one; one mod needs a newer engine */
        CHECK(res.has_exe && res.exe_newer && res.zip_stamp == 1790005000u);
        CHECK(strstr(res.notes, "NEWER game build") && strstr(res.notes, "Your game was not changed") && strstr(res.notes, "geno-lab needs a newer game") && strstr(res.notes, "Kept your own own-mod"));
        CHECK(res.needs_engine == 1 && strstr(msg, "See details."));
        text = nc_read_all("nb_mods/enabled.txt", NULL);
        CHECK(text && strstr(text, "syn-fighter") && strstr(text, "geno-lab") && !strstr(text, "own-mod"));
        free(text);

        /* installing it again from the same post replaces its own folders; no newer-exe note when the exe is not newer */
        bt_exe(exe, 1790000000u - 5u);
        zn = nst_zip(e, ne, z);
        rc = nc_bundle_install(root, &post, z, zn, &env, &benv, &res, msg, sizeof msg, nst_log);
        CHECK(rc == 0 && res.installed == 2 && !res.exe_newer && !strstr(res.notes, "NEWER"));
        /* with no known host build the exe is just ignored */
        rc = nc_bundle_install(root, &post, z, zn, &env, NULL, &res, msg, sizeof msg, nst_log);
        CHECK(rc == 0 && !res.exe_newer);

        /* a local copy of the same zip goes through the local-zip door and installs the same folders (its marker is "local:", so the post's folders are the player's to it) */
        nc_rmtree("nb_mods2"); nc_mkdirs("nb_mods2");
        nlz_bundle_env = &benv;
        rc = nlz_install_data("nb_mods2", "goku.zip", z, zn, &env, &(nc_loose_result){0}, msg, sizeof msg, nst_log);
        nlz_bundle_env = NULL;
        CHECK(rc == 0 && nc_is_dir("nb_mods2/syn-fighter") && nc_is_dir("nb_mods2/geno-lab") && strstr(msg, "Installed 3 mods"));

        /* hostile names are refused before anything is written */
        {
            nst_entry h[2];
            static const char *const mj = "{\"id\":\"x\"}";
            h[0].name = "mods/evil/mod.json"; h[0].data = (const uint8_t *) mj; h[0].len = strlen(mj);
            h[1].name = "mods/evil/../../evil.json"; h[1].data = (const uint8_t *) mj; h[1].len = strlen(mj);
            zn = nst_zip(h, 2, z);
            rc = nc_bundle_install(root, &post, z, zn, &env, &benv, &res, msg, sizeof msg, nst_log);
            CHECK(rc < 0 && res.installed == 0 && !nc_is_dir("nb_mods/evil") && !nc_is_dir("nb_mods/.staging-evil"));
            h[1].name = "mods/evil/note.txt";                          /* a zip with no mod.json folder at all */
            h[0].name = "readme.txt";
            zn = nst_zip(h, 2, z);
            CHECK(!nc_bundle_detect(z, zn));
            rc = nc_bundle_install(root, &post, z, zn, &env, &benv, &res, msg, sizeof msg, nst_log);
            CHECK(rc < 0 && strstr(msg, "no mods/<name>/mod.json"));
        }
        /* a plain Nucleus-style zip that holds a .lua is still refused by the old path (the wider list belongs to bundles only) */
        {
            nst_entry l[1];
            l[0].name = "script.lua"; l[0].data = (const uint8_t *) "x"; l[0].len = 1;
            zn = nst_zip(l, 1, z);
            rc = nlz_install_data("nb_mods2", "lua.zip", z, zn, &env, &(nc_loose_result){0}, msg, sizeof msg, nst_log);
            CHECK(rc < 0 && strstr(msg, "file type"));
        }
        nc_rmtree("nb_mods"); nc_rmtree("nb_mods2");
    }
    (void) path;
}

#endif
