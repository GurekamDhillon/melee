/* nucleus_stage_tests.h - tests of the custom-stage installer (gw_nucleus_stage.h): a synthetic MexManager-style stage package (stage.json + a stage-shaped
 * HSD archive, stored zip), what it writes, what it refuses, and a small fuzz of damaged zips (no crash; every outcome is a plain refusal or an install).
 * Included by nucleus_core_test.c after nucleus_loose_tests.h. SYNTHETIC data only. */
#ifndef NUCLEUS_STAGE_TESTS_H
#define NUCLEUS_STAGE_TESTS_H

#include "../platform/gw_nucleus_stage.h"

static uint32_t nst_crc(const uint8_t *p, size_t n) {
    uint32_t c = 0xFFFFFFFFu;
    size_t i;
    int k;
    for (i = 0; i < n; ++i) { c ^= p[i]; for (k = 0; k < 8; ++k) c = (c >> 1) ^ (0xEDB88320u & (0u - (c & 1u))); }
    return ~c;
}
static void nst_le16(uint8_t *p, unsigned v) { p[0] = (uint8_t) v; p[1] = (uint8_t) (v >> 8); }
static void nst_le32(uint8_t *p, uint32_t v) { nst_le16(p, v & 0xFFFF); nst_le16(p + 2, v >> 16); }

typedef struct { const char *name; const uint8_t *data; size_t len; } nst_entry;

/* a stored zip of the entries; returns its length (out must hold everything) */
static size_t nst_zip(const nst_entry *e, int n, uint8_t *out) {
    size_t o = 0, cd, i;
    uint32_t offs[8];
    int k;
    for (k = 0; k < n; ++k) {
        size_t nl = strlen(e[k].name);
        offs[k] = (uint32_t) o;
        nst_le32(out + o, 0x04034B50u); nst_le16(out + o + 4, 20); nst_le16(out + o + 6, 0); nst_le16(out + o + 8, 0); nst_le16(out + o + 10, 0); nst_le16(out + o + 12, 0);
        nst_le32(out + o + 14, nst_crc(e[k].data, e[k].len)); nst_le32(out + o + 18, (uint32_t) e[k].len); nst_le32(out + o + 22, (uint32_t) e[k].len);
        nst_le16(out + o + 26, (unsigned) nl); nst_le16(out + o + 28, 0);
        memcpy(out + o + 30, e[k].name, nl); memcpy(out + o + 30 + nl, e[k].data, e[k].len);
        o += 30 + nl + e[k].len;
    }
    cd = o;
    for (k = 0; k < n; ++k) {
        size_t nl = strlen(e[k].name);
        nst_le32(out + o, 0x02014B50u); nst_le16(out + o + 4, 20); nst_le16(out + o + 6, 20); nst_le16(out + o + 8, 0); nst_le16(out + o + 10, 0);
        nst_le16(out + o + 12, 0); nst_le16(out + o + 14, 0);
        nst_le32(out + o + 16, nst_crc(e[k].data, e[k].len)); nst_le32(out + o + 20, (uint32_t) e[k].len); nst_le32(out + o + 24, (uint32_t) e[k].len);
        nst_le16(out + o + 28, (unsigned) nl); nst_le16(out + o + 30, 0); nst_le16(out + o + 32, 0); nst_le16(out + o + 34, 0); nst_le16(out + o + 36, 0);
        nst_le32(out + o + 38, 0); nst_le32(out + o + 42, offs[k]);
        memcpy(out + o + 46, e[k].name, nl);
        o += 46 + nl;
    }
    i = o - cd;
    nst_le32(out + o, 0x06054B50u); nst_le16(out + o + 4, 0); nst_le16(out + o + 6, 0); nst_le16(out + o + 8, (unsigned) n); nst_le16(out + o + 10, (unsigned) n);
    nst_le32(out + o + 12, (uint32_t) i); nst_le32(out + o + 16, (uint32_t) cd); nst_le16(out + o + 20, 0);
    return o + 22;
}

static void nst_log(const char *line) { (void) line; }

/* replace the first occurrence of `from` in `s` with `to` (a new malloc'd string) */
static char *nst_sub(const char *s, const char *from, const char *to) {
    const char *p = strstr(s, from);
    char *o = (char *) malloc(strlen(s) + strlen(to) + 1);
    size_t a = p ? (size_t) (p - s) : 0;
    if (!p) { strcpy(o, s); return o; }
    memcpy(o, s, a); strcpy(o + a, to); strcpy(o + a + strlen(to), p + strlen(from));
    return o;
}

static void run_stage_tests(void) {
    static uint8_t zipbuf[8192], bad[8192], hsd[2048], hsd_bad[2048];
    static const char *const good_syms[] = { "map_head", "coll_data", "grGroundParam", "grFunction" };
    static const char *const bad_syms[] = { "grGroundParam", "grFunction" };
    const char *json_ok =
        "{\"name\":\"Test Hill\",\"fileName\":\"/GrTest.dat\",\"soundBank\":57,\"reverbValue1\":3,\"reverbValue2\":4,\"mapDescPointer\":2149000000,"
        "\"onStageInit\":2149639432,\"onGo\":2149639428,\"onStageLoad\":2149639544,\"onStageGo\":2149639548,\"onUnknown2\":2149639584,\"onTouchLine\":2149640072,"
        "\"onUnknown4\":2149640080,\"unknownValue\":1,\"playlist\":{\"entries\":[{\"musicID\":3,\"chance\":50},{\"musicID\":200,\"chance\":50}]},\"gdLineType\":31}";
    char *json_retail, *json_far;
    size_t hn = nl_build_hsd(good_syms, 4, hsd), hbn = nl_build_hsd(bad_syms, 2, hsd_bad), zn, i;
    nst_entry ents[3];
    nc_post post;
    nc_in_file in;
    nc_loose_env env;
    nc_loose_result res;
    char msg[300], path[400], *text;
    const char *root_dir = "ns_mods";
    int rc, fuzz_ok = 0, fuzz_n = 0;
    unsigned seed = 12345u;

    memset(&post, 0, sizeof post); memset(&in, 0, sizeof in); memset(&env, 0, sizeof env);
    post.id = 801; post.title = "Test Hill"; post.author = "Tester"; post.page = "https://ssbmnucleus.net/post/801"; post.updated = "2026-10-10T00:00:00Z"; post.installed_at = "2026-10-10T00:00:00Z";
    in.cat = NC_FC_ARCHIVE; in.filename = "pack/test-stage.zip"; in.stem = "";
    nc_mkdirs(root_dir);

    ents[0].name = "stage.json"; ents[0].data = (const uint8_t *) json_ok; ents[0].len = strlen(json_ok);
    ents[1].name = "folder/GrTest.dat"; ents[1].data = hsd; ents[1].len = hn;
    zn = nst_zip(ents, 2, zipbuf);
    in.data = zipbuf; in.len = zn;
    rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    if (rc != 0) printf("stage install msg: %s | %s", msg, res.first_err);
    CHECK(rc == 0 && res.installed == 1 && !strcmp(res.mods[0], "nucleus-801-cstage"));
    snprintf(path, sizeof path, "%s/nucleus-801-cstage/mexstage.txt", root_dir); nc_fix_slashes(path);
    text = nc_read_all(path, NULL);
    CHECK(text != NULL);
    if (text) {
        CHECK(strstr(text, "name=Test Hill") != NULL && strstr(text, "file=/GrTest.dat") != NULL);
        CHECK(strstr(text, "ssm=55") != NULL);                                    /* bank 57 is not on a vanilla disc */
        CHECK(strstr(text, "bgm=3:50") != NULL && strstr(text, "200") == NULL);   /* music 200 dropped */
        CHECK(strstr(text, "linetype=31") != NULL && strstr(text, "w3=") != NULL && strstr(text, "w9=") != NULL);
        CHECK(strstr(text, "post=https://ssbmnucleus.net/post/801") != NULL);
        free(text);
    }
    snprintf(path, sizeof path, "%s/nucleus-801-cstage/files/GrTest.dat", root_dir); nc_fix_slashes(path);
    CHECK(nc_file_exists(path));
    snprintf(path, sizeof path, "%s/nucleus-801-cstage/mod.json", root_dir); nc_fix_slashes(path);
    text = nc_read_all(path, NULL);
    CHECK(text && strstr(text, "\"kind\": \"stage\"") && strstr(text, "post/801") && strstr(text, "Works, partial"));
    if (text) { nc_inst inst; CHECK(nc_inst_parse("nucleus-801-cstage", text, &inst) && !strcmp(inst.cat, "cstage") && inst.compat[0]); }
    free(text);
    rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);   /* reinstall / update */
    CHECK(rc == 0);

    /* refusals */
    json_retail = nst_sub(json_ok, "GrTest", "GrNBa");
    ents[0].data = (const uint8_t *) json_retail; ents[0].len = strlen(json_retail); ents[1].name = "GrNBa.dat";
    in.len = nst_zip(ents, 2, bad); in.data = bad; post.id = 802;
    rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "retail stage") != NULL && !nc_is_dir("ns_mods/nucleus-802-cstage"));
    json_far = nst_sub(json_ok, "2149639432", "3000000000");
    ents[0].data = (const uint8_t *) json_far; ents[0].len = strlen(json_far); ents[1].name = "GrTest.dat";
    in.len = nst_zip(ents, 2, bad); post.id = 803;
    rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "outside the retail code") != NULL);
    ents[0].data = (const uint8_t *) json_ok; ents[0].len = strlen(json_ok); ents[1].data = hsd_bad; ents[1].len = hbn;
    in.len = nst_zip(ents, 2, bad); post.id = 804;
    rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && strstr(msg, "not a stage file") != NULL);
    ents[1].data = (const uint8_t *) "just text"; ents[1].len = 9;
    in.len = nst_zip(ents, 2, bad);
    rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0);
    in.data = (const uint8_t *) "PK not really"; in.len = 13;
    rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
    CHECK(rc != 0 && msg[0]);
    free(json_retail); free(json_far);

    /* fuzz: single-byte flips and truncations of the good package never crash; each is a refusal or an install */
    ents[0].data = (const uint8_t *) json_ok; ents[0].len = strlen(json_ok); ents[1].data = hsd; ents[1].len = hn; ents[1].name = "GrTest.dat";
    zn = nst_zip(ents, 2, zipbuf);
    post.id = 805;
    for (i = 0; i < 400; ++i) {
        size_t cut = zn;
        memcpy(bad, zipbuf, zn);
        seed = seed * 1103515245u + 12345u;
        if (i % 4 == 3) cut = (seed >> 8) % zn; else { bad[(seed >> 8) % zn] ^= (uint8_t) (1u + (seed >> 24)); }
        in.data = bad; in.len = cut; ++fuzz_n;
        rc = nc_stage_install(root_dir, &post, &in, 1, &env, &res, msg, sizeof msg, nst_log);
        if ((rc == 0 || rc == -1) && msg[0]) ++fuzz_ok;
    }
    CHECK(fuzz_n == 400 && fuzz_ok == 400);

    CHECK(nc_loose_remove(root_dir, 801) == 1 && !nc_is_dir("ns_mods/nucleus-801-cstage"));
    nc_rmtree(root_dir);
}

#endif
