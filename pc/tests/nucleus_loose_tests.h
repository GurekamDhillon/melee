/* nucleus_loose_tests.h - tests of the loose-override categories of the Nucleus browser (stage skins, effect files, menu files, audio): the catalog's
 * classification of every kind of post (tabs, reasons), identifying a stage by the CONTENT of its HSD archive, install / enable / remove / conflicts, and the
 * crash-safe install. Included by nucleus_core_test.c (uses its CHECK macros). SYNTHETIC archives only: nothing here is disc data or a downloaded mod. */
#ifndef NUCLEUS_LOOSE_TESTS_H
#define NUCLEUS_LOOSE_TESTS_H

#include "../platform/gw_nucleus_loose.h"

/* a synthetic HSD archive with the given public symbols (names only; the data section is a few zero bytes) */
static size_t nl_build_hsd(const char *const *syms, int n, uint8_t *out) {
    uint32_t dsize = 16, strsz = 0, o_pub = 0x20 + dsize, o_sym, i, fsize;
    uint32_t off[64];
    for (i = 0; i < (uint32_t) n; ++i) { off[i] = strsz; strsz += (uint32_t) strlen(syms[i]) + 1; }
    o_sym = o_pub + (uint32_t) n * 8;
    fsize = o_sym + strsz;
    memset(out, 0, fsize);
    nc_put32(out, fsize); nc_put32(out + 4, dsize); nc_put32(out + 8, 0); nc_put32(out + 12, (uint32_t) n); nc_put32(out + 16, 0);
    for (i = 0; i < (uint32_t) n; ++i) {
        nc_put32(out + o_pub + i * 8, 0); nc_put32(out + o_pub + i * 8 + 4, off[i]);
        memcpy(out + o_sym + off[i], syms[i], strlen(syms[i]) + 1);
    }
    return fsize;
}

/* the game's "own" files for the test: a few stages (same structure, different textures), an effect file, a menu file */
typedef struct { const char *path; const char *const *syms; int n; } nl_ref;
static const char *const nl_gr_struct[] = { "map_head", "coll_data", "grGroundParam", "yakumono_param", "itemdata" };
#define NL_STRUCT 5
static char nl_names[8][12][40];
static const char *nl_symv[8][20];
static int nl_nsyms[8];
static nl_ref nl_refs[8];
static int nl_nrefs;
static int nl_audio_ok;

static void nl_add_ref(const char *path, const char *tag, int ntex, int extra_struct) {
    int k, n = 0, i = nl_nrefs++;
    for (k = 0; k < NL_STRUCT; ++k) nl_symv[i][n++] = nl_gr_struct[k];
    for (k = 0; k < ntex; ++k) { snprintf(nl_names[i][k], sizeof nl_names[i][k], "Grd%s%d_CMPR_image", tag, k); nl_symv[i][n++] = nl_names[i][k]; }
    if (extra_struct) nl_symv[i][n++] = "extra_table";
    nl_nsyms[i] = n;
    nl_refs[i].path = path; nl_refs[i].syms = nl_symv[i]; nl_refs[i].n = n;
}
static int nl_env_ref(void *u, const char *path, uint8_t **buf, size_t *len) {
    int i;
    (void) u;
    for (i = 0; i < nl_nrefs; ++i) if (!strcmp(nl_refs[i].path, path)) {
        *buf = (uint8_t *) malloc(4096);
        *len = nl_build_hsd(nl_refs[i].syms, nl_refs[i].n, *buf);
        return 0;
    }
    return -1;
}
static int nl_env_exists(void *u, const char *path, size_t *len) { (void) u; *len = 0; return nl_audio_ok && !strcmp(path, "audio/jungle.hps"); }

/* a skin of ref `i`: its names with textures `swap` replaced by new names (the structure kept) */
static size_t nl_skin_of(int i, int swap, uint8_t *out) {
    const char *v[24];
    char fresh[4][40];
    int k, n = 0, s = 0;
    for (k = 0; k < nl_nrefs && k < 0; ++k) {}
    for (k = 0; k < nl_refs[i].n; ++k) {
        if (k >= NL_STRUCT && s < swap && strstr(nl_refs[i].syms[k], "_CMPR_image")) { snprintf(fresh[s], sizeof fresh[s], "Custom%d_RGBA8_image", s); v[n++] = fresh[s++]; }
        else v[n++] = nl_refs[i].syms[k];
    }
    return nl_build_hsd(v, n, out);
}

static void nl_log(const char *line) { (void) line; }
static int nl_read_has(const char *path, const char *needle) {
    char *t = nc_read_all(path, NULL);
    int r = t && strstr(t, needle) != NULL;
    free(t);
    return r;
}

static const char *const nl_kinds_json =
"{\"data\":["
"{\"id\":201,\"title\":\"Stage A\",\"type\":\"stage_skin\",\"stage\":\"Dreamland\",\"files\":[{\"id\":1,\"filename\":\"Pumpkin Hill - GrOp.dat\",\"file_type\":\"stage_dat\"}]},"
"{\"id\":202,\"title\":\"Stage PS\",\"type\":\"stage_skin\",\"stage\":\"Pokemon Stadium\",\"files\":[{\"id\":2,\"filename\":\"GrPs.dat\",\"file_type\":\"stage_dat\"},{\"id\":3,\"filename\":\"GrPs1.dat\",\"file_type\":\"stage_dat\"}]},"
"{\"id\":203,\"title\":\"Custom stage\",\"type\":\"custom_stage\",\"files\":[{\"id\":4,\"filename\":\"GrCustom.dat\",\"file_type\":\"stage_dat\"}]},"
"{\"id\":204,\"title\":\"Fx\",\"type\":\"effects\",\"files\":[{\"id\":5,\"filename\":\"EfFxData.dat\",\"file_type\":\"effects_dat\"}]},"
"{\"id\":205,\"title\":\"Fighter fx\",\"type\":\"effects\",\"files\":[{\"id\":6,\"filename\":\"PlMs White Streak Color.dat\",\"file_type\":\"effects_dat\",\"character\":\"Marth\"}]},"
"{\"id\":206,\"title\":\"Menu\",\"type\":\"other\",\"files\":[{\"id\":7,\"filename\":\"Stock Icons for ISO/MnSlChr.usd\",\"file_type\":\"other\"},{\"id\":8,\"filename\":\"Stock Icons for ISO/IfAll.dat\",\"file_type\":\"other\"}]},"
"{\"id\":207,\"title\":\"Other menu\",\"type\":\"other\",\"files\":[{\"id\":9,\"filename\":\"MnExtAll.usd\",\"file_type\":\"other\"}]},"
"{\"id\":208,\"title\":\"Music\",\"type\":\"other\",\"files\":[{\"id\":10,\"filename\":\"jungle.hps\",\"file_type\":\"other\"}]},"
"{\"id\":209,\"title\":\"Delta\",\"type\":\"patch\",\"files\":[{\"id\":11,\"filename\":\"mod.xdelta\",\"file_type\":\"other\"}]},"
"{\"id\":210,\"title\":\"Pics\",\"type\":\"other\",\"files\":[{\"id\":12,\"filename\":\"doors.png\",\"file_type\":\"other\"}]},"
"{\"id\":211,\"title\":\"Code\",\"type\":\"gameplay_mod\",\"files\":[{\"id\":13,\"filename\":\"codes.gct\",\"file_type\":\"other\"}]},"
"{\"id\":212,\"title\":\"Fox costume\",\"type\":\"costume\",\"files\":[{\"id\":14,\"filename\":\"PlFxLa.dat\",\"file_type\":\"character_dat\",\"character\":\"Fox\",\"color\":\"Lavender\"}]},"
"{\"id\":213,\"title\":\"Tails\",\"type\":\"gameplay_mod\",\"files\":[{\"id\":15,\"filename\":\"Download from google drive.txt\",\"file_type\":\"other\"}]}"
"],\"next_cursor\":null,\"total\":13}";

static const nc_mod *nl_mod(const nc_catalog *c, int id) { int at = nc_cat_find((nc_catalog *) c, id); return at >= 0 ? &c->m[at] : NULL; }

static void run_loose_tests(void) {
    nc_catalog cat;
    nj_doc d;
    int root, data, e;
    const char *why;
    memset(&cat, 0, sizeof cat);

    /* ---- the classification of every kind of post ---- */
    root = nj_parse(&d, nl_kinds_json);
    CHECK(root >= 0);
    data = nj_get(&d, root, "data");
    for (e = d.n[data].first; e >= 0; e = d.n[e].next) nc_cat_apply(&cat, &d, e);
    nj_free(&d);
    CHECK(cat.n == 13);
    CHECK(nl_mod(&cat, 201)->tab == NC_TAB_STAGES && nc_installable_mod(&cat, nl_mod(&cat, 201)));
    CHECK(cat.f[nl_mod(&cat, 201)->f0].cat == NC_FC_STAGE && !strcmp(cat.f[nl_mod(&cat, 201)->f0].stem, "GrOp.dat"));
    CHECK(nl_mod(&cat, 202)->nf == 2 && !strcmp(cat.f[nl_mod(&cat, 202)->f0 + 1].stem, "GrPs1.dat"));
    CHECK(nl_mod(&cat, 203)->tab == NC_TAB_STAGES && !nc_installable_mod(&cat, nl_mod(&cat, 203)));
    CHECK(strstr(nc_why_not(&cat, nl_mod(&cat, 203)), "no .zip stage package") != NULL);
    CHECK(nl_mod(&cat, 204)->tab == NC_TAB_EFFECTS && nc_installable_mod(&cat, nl_mod(&cat, 204)));
    CHECK(cat.f[nl_mod(&cat, 204)->f0].cat == NC_FC_EFFECT && !strcmp(cat.f[nl_mod(&cat, 204)->f0].stem, "EfFxData.dat"));
    CHECK(nl_mod(&cat, 205)->tab == NC_TAB_EFFECTS && !nc_installable_mod(&cat, nl_mod(&cat, 205)) && cat.f[nl_mod(&cat, 205)->f0].cat == NC_FC_FIGHTER);
    CHECK(strstr(nc_why_not(&cat, nl_mod(&cat, 205)), "whole fighter file") != NULL);
    CHECK(nl_mod(&cat, 206)->tab == NC_TAB_MENUS && nc_installable_mod(&cat, nl_mod(&cat, 206)));
    CHECK(nl_mod(&cat, 207)->tab == NC_TAB_MENUS && !nc_installable_mod(&cat, nl_mod(&cat, 207)) && strstr(nc_why_not(&cat, nl_mod(&cat, 207)), "not supported yet") != NULL);
    CHECK(nl_mod(&cat, 208)->tab == NC_TAB_AUDIO && nc_installable_mod(&cat, nl_mod(&cat, 208)));
    CHECK(nl_mod(&cat, 209)->tab == NC_TAB_OTHER && !nc_installable_mod(&cat, nl_mod(&cat, 209)) && strstr(nc_why_not(&cat, nl_mod(&cat, 209)), "xdelta patch: not a mod") != NULL);
    CHECK(nl_mod(&cat, 210)->tab == NC_TAB_OTHER && strstr(nc_why_not(&cat, nl_mod(&cat, 210)), "Image") != NULL);
    CHECK(nl_mod(&cat, 211)->tab == NC_TAB_OTHER && strstr(nc_why_not(&cat, nl_mod(&cat, 211)), "Gameplay mod: needs conversion") != NULL);
    CHECK(nl_mod(&cat, 213) != NULL && !nc_installable_mod(&cat, nl_mod(&cat, 213)) && strstr(nc_why_not(&cat, nl_mod(&cat, 213)), "external link") != NULL);
    CHECK(nl_mod(&cat, 212)->tab == NC_TAB_COSTUMES && nc_installable_mod(&cat, nl_mod(&cat, 212)));
    why = nc_why_not(&cat, nl_mod(&cat, 212));
    CHECK(why[0] == '\0' && strstr("extra costume slots", "needs conversion") == NULL);
    {   /* the tab filter and the search by stage name */
        nc_query q;
        int ids[16], n;
        nc_query_init(&q);
        q.tab_mask = 1u << NC_TAB_STAGES;
        n = nc_cat_filter(&cat, &q, ids, 16);
        CHECK(n == 3);
        q.tab_mask = 1u << NC_TAB_MENUS;
        CHECK(nc_cat_filter(&cat, &q, ids, 16) == 2);
        q.tab_mask = 0; snprintf(q.text, sizeof q.text, "dreamland");
        CHECK(nc_cat_filter(&cat, &q, ids, 16) == 1 && cat.m[ids[0]].id == 201);
        nc_query_init(&q); q.installable_only = 1;
        CHECK(nc_cat_filter(&cat, &q, ids, 16) == 6);          /* 201 202 204 206 208 212 */
    }
    {   /* the cache keeps the file types, the stage and the tabs */
        nc_catalog c2;
        memset(&c2, 0, sizeof c2);
        CHECK(nc_cat_save(&cat, "nl_cache.jsonl") == 0 && nc_cat_load(&c2, "nl_cache.jsonl") == 13);
        CHECK(nl_mod(&c2, 201)->tab == NC_TAB_STAGES && !strcmp(nl_mod(&c2, 201)->stage, "Dreamland") && !strcmp(c2.f[nl_mod(&c2, 201)->f0].ftype, "stage_dat"));
        CHECK(nl_mod(&c2, 206)->tab == NC_TAB_MENUS && nl_mod(&c2, 205)->tab == NC_TAB_EFFECTS);
        nc_cat_free(&c2);
        remove("nl_cache.jsonl");
    }
    nc_cat_free(&cat);

    /* ---- the installer ---- */
    {
        static uint8_t op_skin[4096], op_skin2[4096], nba_as_op[4096], junk[200], ps_skin[4096], fx_skin[4096], menu_bad[4096], menu_ok[4096];
        size_t op_n, op2_n, ps_n, fx_n, mb_n, mo_n;
        const char *root_dir = "nl_mods";
        char path[300], msg[300], names[8][64];
        nc_loose_env env;
        nc_loose_result res;
        nc_post post = { 301, "Pumpkin Hill", "Tester", "https://ssbmnucleus.net/post/301/pumpkin-hill", "2026-02-01T00:00:00Z", "2026-02-02T00:00:00Z", "Dreamland" };
        nc_in_file in[4];
        char cf[16][64], tg[1][64];
        int rc;

        nl_nrefs = 0;
        nl_add_ref("GrOp.dat", "Op", 10, 0);
        nl_add_ref("GrNBa.dat", "Ba", 10, 0);
        nl_add_ref("GrPs.dat", "Ps", 10, 0);
        nl_add_ref("GrPs.usd", "Ps", 10, 0);
        nl_add_ref("EfFxData.dat", "Fx", 6, 0);
        nl_add_ref("MnSlChr.usd", "Mn", 6, 1);
        nl_audio_ok = 1;
        memset(&env, 0, sizeof env);
        env.ref = nl_env_ref; env.exists = nl_env_exists;
        nc_rmtree(root_dir);
        nc_mkdirs(root_dir);

        /* (a) a Dream Land skin: identified by content, installed as a stage mod with the post link */
        op_n = nl_skin_of(0, 2, op_skin);
        in[0].file_id = 7001; in[0].cat = NC_FC_STAGE; in[0].filename = "Pumpkin Hill - GrOp.dat"; in[0].stem = "GrOp.dat"; in[0].data = op_skin; in[0].len = op_n;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc == 0 && res.installed == 1 && res.nmods == 1 && !strcmp(res.mods[0], "nucleus-301-stage-grop"));
        snprintf(path, sizeof path, "%s/nucleus-301-stage-grop/files/GrOp.dat", root_dir); nc_fix_slashes(path);
        { size_t n2 = 0; char *b = nc_read_all(path, &n2); CHECK(b && n2 == op_n && !memcmp(b, op_skin, op_n)); free(b); }
        snprintf(path, sizeof path, "%s/nucleus-301-stage-grop/mod.json", root_dir); nc_fix_slashes(path);
        CHECK(nl_read_has(path, "\"kind\": \"stage\"") && nl_read_has(path, "\"source\": \"nucleus\"") && nl_read_has(path, "ssbmnucleus.net/post/301/pumpkin-hill"));
        CHECK(nl_read_has(path, "\"author\": \"Tester\"") && nl_read_has(path, "\"targets\": [\"GrOp.dat\"]") && nl_read_has(path, "both players need the same skin"));
        CHECK(strstr(msg, "Installed 1 mod") != NULL);
        {
            nc_inst inst[8];
            int n = nc_inst_scan(root_dir, inst, 8);
            CHECK(n == 1 && inst[0].id == 301 && !strcmp(inst[0].cat, "stage") && !strcmp(inst[0].stage, "Dream Land"));
        }

        /* (b) a second Dream Land skin: the newest wins, the first goes off (enabled.txt did not exist: it is written), conflicts are recorded */
        op2_n = nl_skin_of(0, 3, op_skin2);
        post.id = 302; post.title = "Other Dream Land";
        in[0].filename = "GrOp.dat"; in[0].data = op_skin2; in[0].len = op2_n;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc == 0 && res.nturned_off == 1 && !strcmp(res.turned_off[0], "nucleus-301-stage-grop") && strstr(msg, "Turned off nucleus-301-stage-grop") != NULL);
        CHECK(nc_enabled_query(root_dir, "nucleus-302-stage-grop") == 1 && nc_enabled_query(root_dir, "nucleus-301-stage-grop") == 0);
        snprintf(path, sizeof path, "%s/nucleus-302-stage-grop/mod.json", root_dir); nc_fix_slashes(path);
        CHECK(nl_read_has(path, "\"conflicts\": [\"nucleus-301-stage-grop\"]"));
        /* pick the first again from the browser: the second goes off */
        rc = nc_loose_set_enabled(root_dir, "nucleus-301-stage-grop", 1, names, 8);
        CHECK(rc == 1 && !strcmp(names[0], "nucleus-302-stage-grop") && nc_enabled_query(root_dir, "nucleus-301-stage-grop") == 1 && nc_enabled_query(root_dir, "nucleus-302-stage-grop") == 0);
        CHECK(nc_loose_set_enabled(root_dir, "nucleus-301-stage-grop", 0, names, 8) == 0 && nc_enabled_query(root_dir, "nucleus-301-stage-grop") == 0);
        tg[0][0] = '\0'; snprintf(tg[0], 64, "GrOp.dat");
        CHECK(nc_find_conflicts(root_dir, "nucleus-999-x", tg, 1, cf, 16) == 2);          /* both replace GrOp.dat */

        /* (c) the wrong name: a Dream Land file named GrNBa is installed as Dream Land (the content wins); junk and non-stages are refused */
        nba_as_op[0] = 0;
        post.id = 303; post.title = "Mislabelled";
        in[0].filename = "GrNBa.dat"; in[0].stem = "GrNBa.dat"; in[0].data = op_skin; in[0].len = op_n;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc == 0 && !strcmp(res.mods[0], "nucleus-303-stage-grop"));
        memset(junk, 7, sizeof junk);
        in[0].data = junk; in[0].len = sizeof junk; post.id = 304;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc != 0 && res.installed == 0 && res.refused == 1 && strstr(msg, "not an HSD archive") != NULL);
        {   /* an archive with no stage structure (a fighter file, say) */
            const char *fs[] = { "ftData", "something_image" };
            uint8_t fb[512];
            size_t fn = nl_build_hsd(fs, 2, fb);
            in[0].data = fb; in[0].len = fn;
            rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
            CHECK(rc != 0 && strstr(msg, "not a stage file") != NULL);
        }
        {   /* a stage that matches none of the game's files */
            const char *fs[] = { "map_head", "coll_data", "grGroundParam", "alien_a_image", "alien_b_image", "alien_c_image" };
            uint8_t fb[1024];
            size_t fn = nl_build_hsd(fs, 6, fb);
            in[0].data = fb; in[0].len = fn;
            rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
            CHECK(rc != 0 && strstr(msg, "does not look like") != NULL);
        }
        CHECK(!nc_is_dir("nl_mods/nucleus-304-stage-grop") && !nc_is_dir("nl_mods/.staging-nucleus-304-stage-grop"));

        /* (d) Pokemon Stadium: the .dat is mirrored to .usd (the game reads the .usd on an American disc) */
        ps_n = nl_skin_of(2, 1, ps_skin);
        post.id = 305; post.title = "PS skin";
        in[0].filename = "GrPs.dat"; in[0].stem = "GrPs.dat"; in[0].data = ps_skin; in[0].len = ps_n; in[0].cat = NC_FC_STAGE;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        snprintf(path, sizeof path, "%s/nucleus-305-stage-grps/files/GrPs.usd", root_dir); nc_fix_slashes(path);
        CHECK(rc == 0 && nc_file_exists(path));

        /* (e) an effect file and a menu file: same structure as the game's own copy, else refused; menus are global (one active per file) */
        fx_n = nl_skin_of(4, 1, fx_skin);
        post.id = 306; post.title = "Fx";
        in[0].file_id = 1; in[0].cat = NC_FC_EFFECT; in[0].filename = "EfFxData.dat"; in[0].stem = "EfFxData.dat"; in[0].data = fx_skin; in[0].len = fx_n;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc == 0 && !strcmp(res.mods[0], "nucleus-306-fx"));
        mo_n = nl_skin_of(5, 1, menu_ok);
        { const char *bad[] = { "map_head", "Grd_x_image" }; mb_n = nl_build_hsd(bad, 2, menu_bad); }
        post.id = 307; post.title = "Menu";
        in[0].cat = NC_FC_MENU; in[0].filename = "Stock Icons/MnSlChr.usd"; in[0].stem = "MnSlChr.usd"; in[0].data = menu_bad; in[0].len = mb_n;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc != 0 && strstr(msg, "lacks") != NULL);                                   /* missing parts the game's own file has */
        in[0].data = menu_ok; in[0].len = mo_n;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc == 0 && !strcmp(res.mods[0], "nucleus-307-menu"));
        post.id = 308;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc == 0 && res.nturned_off == 1 && !strcmp(res.turned_off[0], "nucleus-307-menu") && nc_enabled_query(root_dir, "nucleus-307-menu") == 0);
        in[0].cat = NC_FC_MENU; in[0].stem = "MnExtAll.usd"; post.id = 309;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        CHECK(rc != 0 && strstr(msg, "no MnExtAll.usd") != NULL);                         /* the game has no such file */

        /* (f) audio: the name has to be one of the game's own audio files, an HPS has its magic */
        {
            uint8_t hps[64];
            memset(hps, 0, sizeof hps); memcpy(hps, " HALPST", 8);
            post.id = 310; post.title = "Music";
            in[0].cat = NC_FC_AUDIO; in[0].filename = "some folder/jungle.hps"; in[0].stem = ""; in[0].data = hps; in[0].len = sizeof hps;
            rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
            snprintf(path, sizeof path, "%s/nucleus-310-audio/files/audio/jungle.hps", root_dir); nc_fix_slashes(path);
            CHECK(rc == 0 && nc_file_exists(path));
            in[0].filename = "other.hps"; post.id = 311;
            rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
            CHECK(rc != 0 && strstr(msg, "not one of the game's audio files") != NULL);
            memset(hps, 1, sizeof hps); in[0].filename = "jungle.hps";
            rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
            CHECK(rc != 0 && strstr(msg, "not an HPS") != NULL);
        }

        /* (g) crash safety: a half-written staging folder from a crash is replaced; a refused update leaves the installed mod untouched */
        snprintf(path, sizeof path, "%s/.staging-nucleus-301-stage-grop/files", root_dir); nc_fix_slashes(path);
        nc_mkdirs(path);
        snprintf(path, sizeof path, "%s/.staging-nucleus-301-stage-grop/files/GrOp.dat", root_dir); nc_fix_slashes(path);
        nc_write_file(path, "half", 4);
        post.id = 301; post.title = "Pumpkin Hill v2";
        in[0].cat = NC_FC_STAGE; in[0].filename = "GrOp.dat"; in[0].stem = "GrOp.dat"; in[0].data = op_skin2; in[0].len = op2_n;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        snprintf(path, sizeof path, "%s/nucleus-301-stage-grop/files/GrOp.dat", root_dir); nc_fix_slashes(path);
        { size_t n2 = 0; char *b = nc_read_all(path, &n2); CHECK(rc == 0 && b && n2 == op2_n && !memcmp(b, op_skin2, op2_n)); free(b); }
        CHECK(!nc_is_dir("nl_mods/.staging-nucleus-301-stage-grop") && !nc_is_dir("nl_mods/.staging-old-nucleus-301-stage-grop"));
        in[0].data = junk; in[0].len = sizeof junk;
        rc = nc_loose_install(root_dir, &post, in, 1, &env, &res, msg, sizeof msg, nl_log);
        { size_t n2 = 0; char *b = nc_read_all(path, &n2); CHECK(rc != 0 && b && n2 == op2_n); free(b); }       /* the earlier install is still whole */

        /* (h) remove: the folders and the enabled.txt lines go */
        CHECK(nc_loose_remove(root_dir, 305) == 1 && !nc_is_dir("nl_mods/nucleus-305-stage-grps"));
        CHECK(nc_enabled_query(root_dir, "nucleus-305-stage-grps") == 0);
        nc_rmtree(root_dir);
    }
}

#endif
