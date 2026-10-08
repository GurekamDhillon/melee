/* nucleus_core_test.c - native test of the Nucleus browser's pure logic (gw_nucleus_*.h): the JSON reader, slot codes, the catalog, the sync against
 * recorded (synthetic) API JSON, the cache, search / sort, reading a costume DAT's symbols, PNG -> .gxtex, the skin mod.json, the installed scan.
 * Run: tools/port/build.sh --native-test nucleus-core. No network, no game, no disc. */
#define _CRT_SECURE_NO_WARNINGS
#define _CRT_NONSTDC_NO_WARNINGS
#include "../platform/gw_nucleus_install.h"
#include "nucleus_fixture.h"

static int g_fail, g_checks;
#define CHECK(c) do { ++g_checks; if (!(c)) { ++g_fail; printf("FAIL %s:%d: %s\n", __FILE__, __LINE__, #c); } } while (0)
#define CHECK_STR(a, b) do { ++g_checks; if (strcmp((a), (b)) != 0) { ++g_fail; printf("FAIL %s:%d: \"%s\" != \"%s\"\n", __FILE__, __LINE__, (a), (b)); } } while (0)

/* ---- the fake network ---- */
static char g_urls[32][700];
static int g_nurls;
static int64_t g_clock;
static int g_slept[16], g_nslept;
static int g_mode;          /* 0 normal, 1 always 503, 2 first 429 with Retry-After 7 then normal, 3 400 */
static int g_calls;

static int fake_fetch(void *u, const char *url, nc_resp *r) {
    const char *body = NULL;
    (void) u;
    if (g_nurls < 32) snprintf(g_urls[g_nurls++], sizeof g_urls[0], "%s", url);
    ++g_calls;
    if (g_mode == 1) { r->status = 503; return 0; }
    if (g_mode == 3) { r->status = 400; r->body = strdup("{\"error\":{\"code\":\"bad\",\"message\":\"x\"}}"); return 0; }
    if (g_mode == 2 && g_calls == 1) { r->status = 429; r->retry_after = 7; r->body = strdup(fx_error429); return 0; }
    if (strstr(url, "/mods/removed")) body = fx_removed;
    else if (strstr(url, "updated_since=")) body = fx_delta;
    else if (strstr(url, "cursor=")) body = fx_page2;
    else body = fx_page1;
    r->status = 200;
    r->body = strdup(body);
    r->len = strlen(body);
    return 0;
}
static int64_t fake_now(void *u) { (void) u; return g_clock; }
static void fake_sleep(void *u, int ms) { (void) u; if (g_nslept < 16) g_slept[g_nslept++] = ms; }
static int fake_stop(void *u) { (void) u; return 0; }
static int g_checkpoints;
static void fake_checkpoint(void *u) { (void) u; ++g_checkpoints; }

static void reset_net(int mode) { g_nurls = 0; g_nslept = 0; g_mode = mode; g_calls = 0; }

/* ---- a PNG made by hand (stored deflate), so the test has no image file ---- */
static uint32_t crc_tab[256];
static uint32_t crc32b(const uint8_t *p, size_t n, uint32_t c) {
    size_t i;
    if (!crc_tab[1]) { uint32_t k, j; for (k = 0; k < 256; ++k) { uint32_t v = k; for (j = 0; j < 8; ++j) v = v & 1 ? 0xEDB88320u ^ (v >> 1) : v >> 1; crc_tab[k] = v; } }
    c = ~c;
    for (i = 0; i < n; ++i) c = crc_tab[(c ^ p[i]) & 255] ^ (c >> 8);
    return ~c;
}
static void put_chunk(uint8_t **o, const char *type, const uint8_t *d, size_t n) {
    uint8_t *p = *o;
    uint32_t c;
    nc_put32(p, (uint32_t) n); memcpy(p + 4, type, 4); if (n) memcpy(p + 8, d, n);
    c = crc32b(p + 4, n + 4, 0);
    nc_put32(p + 8 + n, c);
    *o = p + 12 + n;
}
static size_t make_png(const uint8_t *rgba, int w, int h, uint8_t *out) {
    uint8_t ihdr[13], *raw, *z, *o = out;
    size_t rawn = (size_t) h * (1 + (size_t) w * 4), i, zn, a = 1, b = 0;
    int y;
    memcpy(o, "\x89PNG\r\n\x1a\n", 8); o += 8;
    nc_put32(ihdr, (uint32_t) w); nc_put32(ihdr + 4, (uint32_t) h); ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = ihdr[11] = ihdr[12] = 0;
    put_chunk(&o, "IHDR", ihdr, 13);
    raw = (uint8_t *) malloc(rawn);
    for (y = 0; y < h; ++y) { raw[(size_t) y * (1 + (size_t) w * 4)] = 0; memcpy(raw + (size_t) y * (1 + (size_t) w * 4) + 1, rgba + (size_t) y * w * 4, (size_t) w * 4); }
    z = (uint8_t *) malloc(rawn + 64);
    zn = 0; z[zn++] = 0x78; z[zn++] = 0x01;
    z[zn++] = 1; z[zn++] = (uint8_t) (rawn & 255); z[zn++] = (uint8_t) (rawn >> 8); z[zn++] = (uint8_t) (~rawn & 255); z[zn++] = (uint8_t) ((~rawn >> 8) & 255);
    memcpy(z + zn, raw, rawn); zn += rawn;
    for (i = 0; i < rawn; ++i) { a = (a + raw[i]) % 65521; b = (b + a) % 65521; }
    nc_put32(z + zn, (uint32_t) ((b << 16) | a)); zn += 4;
    put_chunk(&o, "IDAT", z, zn);
    put_chunk(&o, "IEND", NULL, 0);
    free(raw); free(z);
    return (size_t) (o - out);
}

/* ---- a costume DAT made by hand: header, empty data, one public per symbol ---- */
static size_t make_dat(const char *const *syms, int ns, uint8_t *out) {
    uint32_t data = 0x40, o_pub = 0x20 + data, o_sym = o_pub + (uint32_t) ns * 8, so = 0;
    int i;
    memset(out, 0, 0x400);
    nc_put32(out + 4, data); nc_put32(out + 12, (uint32_t) ns);
    for (i = 0; i < ns; ++i) {
        nc_put32(out + o_pub + i * 8, 0); nc_put32(out + o_pub + i * 8 + 4, so);
        strcpy((char *) out + o_sym + so, syms[i]);
        so += (uint32_t) strlen(syms[i]) + 1;
    }
    nc_put32(out, o_sym + so);
    return o_sym + so;
}

static int has_url(const char *needle) { int i; for (i = 0; i < g_nurls; ++i) if (strstr(g_urls[i], needle)) return 1; return 0; }

int main(void) {
    nj_doc d;
    int root;
    nc_catalog cat;
    nc_sync st;
    nc_sync_io io;
    int r;

    memset(&io, 0, sizeof io);
    io.fetch = fake_fetch; io.now = fake_now; io.sleep_ms = fake_sleep; io.stop = fake_stop; io.checkpoint = fake_checkpoint;

    /* JSON: escapes, surrogate pairs, nesting, errors */
    root = nj_parse(&d, "{\"a\":\"x\\ny\\u00e9\\ud83d\\ude00\",\"b\":[1,2.5,true,null],\"c\":{\"d\":-3}}");
    CHECK(root >= 0);
    CHECK(strlen(nj_gstr(&d, root, "a", "")) == 1 + 1 + 1 + 2 + 4);
    CHECK(nj_gnum(&d, nj_get(&d, root, "c"), "d", 0) == -3);
    nj_free(&d);
    CHECK(nj_parse(&d, "{\"a\":") < 0); nj_free(&d);
    CHECK(nj_parse(&d, "{\"a\":1} x") < 0); nj_free(&d);
    {
        char deep[200];
        memset(deep, '[', 150); deep[150] = '\0';
        CHECK(nj_parse(&d, deep) < 0); nj_free(&d);
    }

    /* times */
    {
        char iso[32];
        CHECK(nc_parse_iso("2026-10-08T22:37:39.000000Z") == 1791499059LL);
        nc_iso(1791499059LL, iso, sizeof iso);
        CHECK_STR(iso, "2026-10-08T22:37:39Z");
        nc_iso(0, iso, sizeof iso);
        CHECK_STR(iso, "1970-01-01T00:00:00Z");
    }

    /* slot codes from messy names */
    {
        char pl[3], col[3];
        CHECK(nc_slot_from_filename("Crash Bandicoot - Fox- PlFxLa (No Jacket).dat", pl, col) && !strcmp(pl, "Fx") && !strcmp(col, "La"));
        CHECK(nc_slot_from_filename("PlGw.dat", pl, col) && !strcmp(pl, "Gw") && col[0] == '\0');
        CHECK(nc_slot_from_filename("PlMs (blue).dat", pl, col) && !strcmp(pl, "Ms") && col[0] == '\0');
        CHECK(!nc_slot_from_filename("Plain costume.dat", pl, col));
        CHECK(nc_fighter_by_name("Captain Falcon") == 2 && nc_fighter_by_name("ice climbers") == 10 && nc_fighter_by_name("Nobody") < 0);
        CHECK(nc_colour_index(1, "La") == 2 && nc_colour_index(1, "Nr") == 0 && nc_colour_index(1, "Zz") == -1);
    }

    /* the sync: a full pass, then the delta and the removed feed in the same run */
    memset(&cat, 0, sizeof cat); memset(&st, 0, sizeof st);
    g_clock = 1791499059LL; reset_net(0);
    r = nc_sync_run(&st, &cat, &io, 0);
    CHECK(r == 0);
    CHECK(g_nurls == 4);
    CHECK(strstr(g_urls[0], "/mods?limit=100") && !strstr(g_urls[0], "cursor"));
    CHECK(strstr(g_urls[1], "cursor=AB%2B%2F%3D%3D"));                 /* the cursor is URL-encoded */
    CHECK(strstr(g_urls[2], "updated_since=2026-10-08T22%3A22%3A39Z"));   /* T0 minus 15 minutes */
    CHECK(strstr(g_urls[3], "/mods/removed?limit=1000&since=2026-10-08T22%3A22%3A39Z"));
    CHECK(cat.n == 6);                                   /* 101..106 + 107 - 102 */
    CHECK(nc_cat_find(&cat, 102) < 0 && nc_cat_is_gone(&cat, 102));
    CHECK(nc_cat_find(&cat, 107) >= 0);
    CHECK_STR(cat.m[nc_cat_find(&cat, 101)].title, "Neon Fox v2");
    CHECK(cat.m[nc_cat_find(&cat, 101)].nf == 1);
    CHECK(cat.m[nc_cat_find(&cat, 106)].title[3] == '?');   /* non-ASCII became '?' */
    CHECK(st.full == 1 && st.last_poll == g_clock && st.since == g_clock && g_checkpoints >= 1);
    CHECK(g_nslept >= 2 && g_slept[0] == 700);           /* paced between pages */
    {
        nc_mod *m105 = &cat.m[nc_cat_find(&cat, 105)];
        const nc_file *f = &cat.f[m105->f0];
        CHECK(f->costume && f->fighter == 24 && f->colour_idx == 0);   /* "other" + a Pl code: a Game & Watch costume by the file name */
        CHECK(nc_installable_mod(&cat, m105));
        CHECK(!nc_installable_mod(&cat, &cat.m[nc_cat_find(&cat, 103)]));
        CHECK_STR(nc_why_not(&cat, &cat.m[nc_cat_find(&cat, 103)]), "Sheik and Nana costumes come with Zelda and Popo (not yet).");
        CHECK(nc_installable_mod(&cat, &cat.m[nc_cat_find(&cat, 101)]));
    }
    /* a second run inside 10 minutes fetches nothing */
    reset_net(0); g_clock += 30;
    CHECK(nc_sync_run(&st, &cat, &io, 0) == 1 && g_nurls == 0);
    CHECK(nc_sync_run(&st, &cat, &io, 1) == 1 && g_nurls == 0);   /* even forced, not inside a minute */
    g_clock += 569;
    CHECK(nc_sync_run(&st, &cat, &io, 0) == 1 && g_nurls == 0);   /* 599 s after the last poll: still too soon */
    g_clock += 2;
    reset_net(0);
    CHECK(nc_sync_run(&st, &cat, &io, 0) == 0 && g_nurls == 2);   /* the delta and the removed feed only: no second full pass */
    CHECK(strstr(g_urls[0], "updated_since=") && strstr(g_urls[1], "/mods/removed"));

    /* back-off: 503s are retried with growing waits, then the run gives up and says so; the catalog keeps what it had */
    g_clock += 1000; reset_net(1);
    r = nc_sync_run(&st, &cat, &io, 0);
    CHECK(r == -1 && g_calls == 5 && g_nslept == 4);
    CHECK(g_slept[0] == 5000 && g_slept[1] == 15000 && g_slept[2] == 45000 && g_slept[3] == 120000);
    CHECK(st.msg[0] != '\0' && cat.n == 6 && st.next_ok > g_clock);
    reset_net(0);
    CHECK(nc_sync_run(&st, &cat, &io, 1) == 1 && g_nurls == 0);   /* waiting after a failure, even forced */
    g_clock = st.next_ok + 1;
    reset_net(2);                                                   /* a 429 with Retry-After: 7 s is honoured */
    CHECK(nc_sync_run(&st, &cat, &io, 0) == 0 && g_slept[0] == 7000);
    g_clock += 700; reset_net(3);                                   /* a 400 is not retried */
    st.last_poll = 0;
    CHECK(nc_sync_run(&st, &cat, &io, 0) == -1 && g_calls == 1 && strstr(st.msg, "400"));

    /* the state survives a restart, and an unfinished first pass resumes at its cursor */
    {
        nj_buf b;
        nc_sync back;
        memset(&b, 0, sizeof b);
        st.full = 0; snprintf(st.cursor, sizeof st.cursor, "AB+/==");
        nc_sync_to_json(&st, &b);
        CHECK(nc_sync_from_json(&back, b.s) == 0 && back.full == 0 && !strcmp(back.cursor, "AB+/==") && back.t0 == st.t0);
        free(b.s);
        st = back; st.next_ok = 0;
        reset_net(0);
        CHECK(nc_sync_run(&st, &cat, &io, 0) == 0 && strstr(g_urls[0], "cursor=AB%2B%2F%3D%3D"));   /* no restart from page 1 */
    }

    /* the cache round trip */
    {
        nc_catalog back;
        nc_query q;
        int a[64], b[64], na, nb;
        const char *path = "nc_cache_test.jsonl";
        memset(&back, 0, sizeof back);
        CHECK(nc_cat_save(&cat, path) == 0);
        CHECK(nc_cat_load(&back, path) == cat.n && back.ngone == cat.ngone);
        nc_query_init(&q); q.sort = NC_SORT_TITLE;
        na = nc_cat_filter(&cat, &q, a, 64); nb = nc_cat_filter(&back, &q, b, 64);
        CHECK(na == nb && na == 6);
        CHECK_STR(cat.m[a[0]].title, back.m[b[0]].title);
        CHECK(back.f[back.m[nc_cat_find(&back, 105)].f0].fighter == 24);
        nc_cat_free(&back);
        remove(path);
        CHECK(nc_cat_load(&back, "nc_no_such_file.jsonl") == -1);
    }

    /* search, filters, sorts */
    {
        nc_query q;
        int a[64], n;
        nc_query_init(&q); q.sort = NC_SORT_DOWNLOADS;
        n = nc_cat_filter(&cat, &q, a, 64);
        CHECK(n == 6 && cat.m[a[0]].id == 106 && cat.m[a[n - 1]].id == 107);       /* 100 downloads first, 0 (105, 107) last by id */
        nc_query_init(&q); snprintf(q.text, sizeof q.text, "neon FOX");
        n = nc_cat_filter(&cat, &q, a, 64);
        CHECK(n == 1 && cat.m[a[0]].id == 101);
        nc_query_init(&q); snprintf(q.text, sizeof q.text, "tester three");
        n = nc_cat_filter(&cat, &q, a, 64);
        CHECK(n == 1 && cat.m[a[0]].id == 104);                                     /* every word must hit (title, author, tags or a file name) */
        nc_query_init(&q); snprintf(q.text, sizeof q.text, "tester nomatch");
        CHECK(nc_cat_filter(&cat, &q, a, 64) == 0);
        nc_query_init(&q); snprintf(q.text, sizeof q.text, "tester");
        CHECK(nc_cat_filter(&cat, &q, a, 64) == 5);                                 /* author search */
        nc_query_init(&q); q.fighter = 1;
        CHECK(nc_cat_filter(&cat, &q, a, 64) == 1);
        nc_query_init(&q); q.type_mask = 1u << NC_T_STAGE_SKIN;
        CHECK(nc_cat_filter(&cat, &q, a, 64) == 0);                                 /* 102 was removed */
        nc_query_init(&q); q.installable_only = 1;
        CHECK(nc_cat_filter(&cat, &q, a, 64) == 5);                                 /* all but the Sheik post */
        nc_query_init(&q); q.sort = NC_SORT_UPDATED;
        n = nc_cat_filter(&cat, &q, a, 64);
        CHECK(cat.m[a[0]].id == 101);                                               /* updated 2026-03-03 */
    }

    /* a costume DAT is typed by its content */
    {
        static uint8_t dat[0x400];
        const char *syms[] = { "SomethingElse", "PlyFox5KLa_Share_joint", "PlyFox5KLa_Share_matanim_joint" };
        const char *bad[] = { "Foo", "Bar" };
        nc_dat_info info;
        char err[96];
        size_t n = make_dat(syms, 3, dat);
        CHECK(nc_dat_costume(dat, n, &info, err, sizeof err) == 0);
        CHECK_STR(info.token, "Fox"); CHECK_STR(info.colour, "La"); CHECK_STR(info.joint, "PlyFox5KLa_Share_joint");
        CHECK_STR(info.matanim, "PlyFox5KLa_Share_matanim_joint");
        CHECK(info.fighter == 1 && info.colour_idx == 2);
        n = make_dat(syms, 2, dat);
        CHECK(nc_dat_costume(dat, n, &info, err, sizeof err) == 0 && info.matanim[0] == '\0');
        n = make_dat(bad, 2, dat);
        CHECK(nc_dat_costume(dat, n, &info, err, sizeof err) < 0 && strstr(err, "no costume symbol"));
        CHECK(nc_dat_costume(dat, 10, &info, err, sizeof err) < 0);
        memset(dat, 0xFF, 0x40);
        CHECK(nc_dat_costume(dat, 0x40, &info, err, sizeof err) < 0);                /* a hostile header does not run off the buffer */
        {
            const char *def[] = { "PlyFox5K_Share_joint" };
            n = make_dat(def, 1, dat);
            CHECK(nc_dat_costume(dat, n, &info, err, sizeof err) == 0 && info.colour[0] == '\0' && info.colour_idx == 0);
        }
    }

    /* PNG -> .gxtex: the header, the padding and one texel */
    {
        uint8_t px[6 * 5 * 4], png[4096], *g;
        size_t pn, gn, x;
        char err[96];
        for (x = 0; x < sizeof px; x += 4) { px[x] = 255; px[x + 1] = 0; px[x + 2] = 0; px[x + 3] = 255; }       /* opaque red */
        px[(1 * 6 + 2) * 4 + 3] = 0;                                                                              /* one transparent texel */
        pn = make_png(px, 6, 5, png);
        CHECK(nc_png_to_gxtex(png, pn, &g, &gn, err, sizeof err) == 0);
        CHECK(memcmp(g, "GXTX", 4) == 0 && nc_be32(g + 8) == 5 && nc_be32(g + 12) == 8 && nc_be32(g + 16) == 8);   /* RGB5A3, padded to 8 x 8 */
        CHECK(nc_be32(g + 20) == 0xFFFFFFFFu && nc_be32(g + 28) == 8 * 8 * 2 && gn == 64 + 128);
        CHECK(((g[64] << 8) | g[65]) == 0xFC00);                                                                   /* texel (0,0): opaque 31,0,0 */
        /* texel (2,1) is tile 0, row 1, column 2 -> index 6 within the tile */
        CHECK(((g[64 + 12] << 8) | g[64 + 13]) == 0x0F00);
        CHECK((g[64 + 3 * 32 + 30] | g[64 + 3 * 32 + 31]) == 0);                                      /* padding is transparent */
        free(g);
        CHECK(nc_png_to_gxtex((const uint8_t *) "not a png at all", 16, &g, &gn, err, sizeof err) < 0);
    }

    /* the skin mod.json: the exact keys the skin reader accepts, the source block, the credit */
    {
        nc_skin_mod s;
        nj_buf b;
        nj_doc dd;
        int rt, sk, co, c0, k;
        static const char *const top_keys[] = { "format", "target", "order", "costumes" };
        static const char *const cos_keys[] = { "name", "file", "joint", "matanim", "team", "like", "kirby_hat", "csp", "stock", "icon", "partner" };
        memset(&s, 0, sizeof s); memset(&b, 0, sizeof b);
        s.nucleus_id = 101; nc_mod_id(101, 1, s.mod_id, sizeof s.mod_id);
        snprintf(s.title, sizeof s.title, "Neon \"Fox\""); snprintf(s.author, sizeof s.author, "Tester One");
        snprintf(s.page, sizeof s.page, "https://ssbmnucleus.net/post/101/neon-fox"); snprintf(s.updated, sizeof s.updated, "2026-03-03T04:05:06Z");
        snprintf(s.installed_at, sizeof s.installed_at, "2026-10-08T22:00:00Z"); snprintf(s.target, sizeof s.target, "fox");
        s.n = 2;
        nc_costume_name("Neon Fox with a very long title indeed", "Lavender", 1, s.c[0].name, sizeof s.c[0].name);
        CHECK(strlen(s.c[0].name) <= 23 && strlen(s.c[0].name) >= 1);
        snprintf(s.c[0].dat, sizeof s.c[0].dat, "skins/%s/1.dat", s.mod_id); snprintf(s.c[0].joint, sizeof s.c[0].joint, "PlyFox5KLa_Share_joint");
        snprintf(s.c[0].matanim, sizeof s.c[0].matanim, "PlyFox5KLa_Share_matanim_joint"); s.c[0].like = 2; s.c[0].file_id = 9001;
        snprintf(s.c[0].csp, sizeof s.c[0].csp, "skins/%s/1_csp.gxtex", s.mod_id); snprintf(s.c[0].stock, sizeof s.c[0].stock, "skins/%s/1_stock.gxtex", s.mod_id);
        snprintf(s.c[1].name, sizeof s.c[1].name, "Green"); snprintf(s.c[1].dat, sizeof s.c[1].dat, "skins/%s/2.dat", s.mod_id);
        snprintf(s.c[1].joint, sizeof s.c[1].joint, "PlyFox5KGr_Share_joint"); s.c[1].file_id = 9002;
        nc_skin_json(&s, &b);
        rt = nj_parse(&dd, b.s);
        CHECK(rt >= 0);
        CHECK_STR(nj_gstr(&dd, rt, "id", ""), "nucleus-101-fox");
        CHECK_STR(nj_gstr(&dd, rt, "kind", ""), "skin"); CHECK_STR(nj_gstr(&dd, rt, "source", ""), "nucleus");
        CHECK_STR(nj_gstr(&dd, rt, "name", ""), "Neon \"Fox\"");
        CHECK(strstr(nj_gstr(&dd, rt, "description", ""), "ssbmnucleus.net/post/101") && strstr(nj_gstr(&dd, rt, "description", ""), "Tester One"));
        CHECK_STR(nj_gstr(&dd, rt, "version", ""), "2026-03-03T04:05:06Z");
        sk = nj_get(&dd, rt, "skin");
        for (k = dd.n[sk].first; k >= 0; k = dd.n[k].next) {
            int j, ok = 0;
            for (j = 0; j < 4; ++j) if (!strcmp(dd.pool + dd.n[k].key, top_keys[j])) ok = 1;
            CHECK(ok);
        }
        CHECK_STR(nj_gstr(&dd, nj_get(&dd, sk, "target"), "retail", ""), "fox");
        co = nj_get(&dd, sk, "costumes");
        for (c0 = dd.n[co].first; c0 >= 0; c0 = dd.n[c0].next)
            for (k = dd.n[c0].first; k >= 0; k = dd.n[k].next) {
                int j, ok = 0;
                for (j = 0; j < 11; ++j) if (!strcmp(dd.pool + dd.n[k].key, cos_keys[j])) ok = 1;
                CHECK(ok);
            }
        c0 = dd.n[co].first;
        CHECK(nj_gnum(&dd, c0, "like", 0) == 2 && nj_gnum(&dd, dd.n[c0].next, "like", -1) == -1);
        {
            nc_inst in;
            CHECK(nc_inst_parse("nucleus-101-fox", b.s, &in) && in.id == 101 && in.ncost == 2 && !strcmp(in.target, "fox") && !strcmp(in.updated, "2026-03-03T04:05:06Z"));
        }
        nj_free(&dd);
        free(b.s);
    }

    /* the installed scan, enabled.txt and folder helpers, in a scratch folder */
    {
        const char *root_dir = "nc_scan_test";
        char path[300], *text;
        nc_inst inst[8];
        nc_rmtree(root_dir);
        CHECK(nc_mkdirs("nc_scan_test/nucleus-5-fox/files/skins") == 0 && nc_is_dir("nc_scan_test/nucleus-5-fox/files/skins"));
        nc_mkdirs("nc_scan_test/other-mod");
        snprintf(path, sizeof path, "%s/nucleus-5-fox/mod.json", root_dir);
        { const char *mj = "{\"id\":\"nucleus-5-fox\",\"name\":\"Five\",\"source\":\"nucleus\",\"nucleus\":{\"id\":5,\"updated_at\":\"u\",\"author\":\"a\",\"fighter\":\"fox\"},\"skin\":{\"costumes\":[{}]}}"; nc_write_file(path, mj, strlen(mj)); }
        snprintf(path, sizeof path, "%s/other-mod/mod.json", root_dir);
        nc_write_file(path, "{\"source\":\"nucleus\"}", strlen("{\"source\":\"nucleus\"}"));
        CHECK(nc_inst_scan(root_dir, inst, 8) == 1 && inst[0].id == 5 && inst[0].ncost == 1);
        /* enabled.txt: absent means everything is on, so nothing is written */
        CHECK(nc_enabled_edit(root_dir, "nucleus-5-fox", 1) == 0);
        snprintf(path, sizeof path, "%s/enabled.txt", root_dir);
        CHECK(nc_read_all(path, NULL) == NULL);
        nc_write_file(path, "ace-base\n", 9);
        CHECK(nc_enabled_edit(root_dir, "nucleus-5-fox", 1) == 0);
        CHECK(nc_enabled_edit(root_dir, "nucleus-5-fox", 1) == 0);               /* adding twice lists it once */
        text = nc_read_all(path, NULL);
        CHECK(text && !strcmp(text, "ace-base\nnucleus-5-fox\n"));
        free(text);
        CHECK(nc_enabled_edit(root_dir, "nucleus-5-fox", 0) == 0);
        text = nc_read_all(path, NULL);
        CHECK(text && !strcmp(text, "ace-base\n"));
        free(text);
        nc_mkdirs("nc_scan_test/stage");
        nc_write_file("nc_scan_test/stage/x.txt", "x", 1);
        CHECK(nc_swap_dir("nc_scan_test/stage", "nc_scan_test/nucleus-5-fox") == 0 && nc_read_all("nc_scan_test/nucleus-5-fox/x.txt", NULL) != NULL);
        nc_rmtree(root_dir);
        CHECK(!nc_is_dir(root_dir));
    }
    (void) has_url;
    nc_cat_free(&cat);
    printf("nucleus-core: %d checks, %d failed\n", g_checks, g_fail);
    return g_fail ? 1 : 0;
}
