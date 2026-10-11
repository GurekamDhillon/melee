/* Headless tests for the LAB input recorder's file format (.gdlp, gw_script_rec.inc): the container,
 * the round trip, a torn tail, the Slippi-style size-table skip, and the playback refusals. The
 * in-match exactness (record, play back, positions identical) is pc/tests/lab_rec_acceptance.lua. */
static int test_script_rec(void) {
    char path[MAX_PATH], why[200], out[512];
    GrFile g;
    long a, b, size;
    FILE *f;
    unsigned char fe[GR_FRAME_PAYLOAD], mk[36], seg[48], hdr[15], *bytes;
    int vp[2][9], i, rc = 0;
    memset(vp, 0, sizeof vp);
    vp[1][3] = 77;
    gr_path(path, sizeof path, "gdlp_selftest", "gdlp");
    DeleteFileA(path);
    {
        int state_size = gw_FighterControls_StateSize();
        void *saved = malloc((size_t)state_size);
        if (!saved || !gw_FighterControls_StateSave(saved, state_size)) { free(saved); return 1; }
        gw_FighterControls_MatchClear();
        gw_FighterControls_MatchSet(0, 2, 1);
        gw_FighterControls_MatchSet(1, 9, 0);
        f = gr_begin(path, 100, "gdlp_selftest.gdst", &a, &b);
        gw_FighterControls_StateLoad(saved, state_size);
        free(saved);
    }
    if (f == NULL) { gw_test_fail("rec: cannot create %s", path); return 1; }
    memset(seg, 0, sizeof seg); gr_put32(seg, 5); gr_put32(seg + 4, 9); snprintf((char *) seg + 8, 40, "source");
    gr_ev(f, GR_EV_SEGMENT, seg);
    for (i = 0; i < 5; ++i) {
        unsigned char pad[GR_PAD], c = GR_EV_FRAME;
        memset(pad, 0x10 + i, sizeof pad);
        vp[0][0] = i;
        gr_frame_bytes(fe, 100 + i, i != 3, pad, vp);
        fwrite(&c, 1, 1, f); fwrite(fe, 1, sizeof fe, f);
        if (i == 2) { c = GR_EV_MARK; memset(mk, 0, sizeof mk); gr_put32(mk, 3); snprintf((char *) mk + 4, 32, "good"); fwrite(&c, 1, 1, f); fwrite(mk, 1, sizeof mk, f); }
    }
    if (gr_finish(f, a, b) != 0) { gw_test_fail("rec: finish failed"); return 1; }
    /* the container is Slippi's: `{U\3raw[$U#l`, a length, then the 0x35 size table */
    f = fopen(path, "rb"); fseek(f, 0, SEEK_END); size = ftell(f); fseek(f, 0, SEEK_SET);
    bytes = (unsigned char *) malloc((size_t) size);
    if (bytes == NULL || fread(bytes, 1, (size_t) size, f) != (size_t) size) { fclose(f); free(bytes); gw_test_fail("rec: reread"); return 1; }
    fclose(f);
    memcpy(hdr, bytes, 15);
    if (memcmp(hdr, "{U\x03raw[$U#l", 11) != 0 || bytes[15] != 0x35) rc |= 1;
    if ((long) gr_get32(hdr + 11) != size - 15 - (long) (sizeof "U\x08metadata{U\x08playedOnSU\x08melee-pc}}" - 1)) rc |= 2;
    free(bytes);
    if (gr_read(path, &g, why, sizeof why) != 0) { gw_test_fail("rec: read back: %s", why); return 1; }
    if (!g.has_controls || g.controls_ck[0] != 2 || g.controls_ck[1] != 9 ||
        g.controls[0] != 1 || g.controls[1] != 0) rc |= 8192;
    if (g.n != 5 || g.fr[0].tag != 100 || g.fr[4].tag != 104) rc |= 4;
    if (g.fr[2].pad[0] != 0x12 || g.fr[3].have != 0 || g.fr[1].have != 1 || g.fr[4].vp[0][0] != 4 || g.fr[4].vp[1][3] != 77) rc |= 8;
    if (g.nmarks != 1 || g.marks[0].frame != 3 || strcmp(g.marks[0].label, "good") != 0) rc |= 16;
    if (g.seg_from != 5 || g.seg_to != 9 || strcmp(g.seg_src, "source") != 0) rc |= 32;
    if (!g.complete || g.ver != GR_VER || g.base != 100 || strcmp(g.state, "gdlp_selftest.gdst") != 0) rc |= 64;
    /* refusals: a changed build / mods / Geno data each say so; with no match running, say that */
    {
        GrFile h = g;
        h.fr = NULL; h.n = 0;
        gs_identity(h.id);
        h.geno_version = GENO_VERSION;
        snprintf(h.mods, sizeof h.mods, "some-mod");
        h.id[0] ^= 1;
        if (gr_check(&h, why, sizeof why) == 0 || !strstr(why, "another build")) rc |= 128;
        h.id[0] ^= 1; h.id[1] ^= 1;
        if (gr_check(&h, why, sizeof why) == 0 || !strstr(why, "another disc")) rc |= 128;
        h.id[1] ^= 1; h.id[2] ^= 1;
        if (gr_check(&h, why, sizeof why) == 0 || !strstr(why, "other mods") || !strstr(why, "some-mod")) rc |= 256;
        h.id[2] ^= 1; h.id[3] ^= 1;
        if (gr_check(&h, why, sizeof why) == 0 || !strstr(why, "Geno")) rc |= 512;
        h.id[3] ^= 1;
        gs.match_active = 0;
        if (gr_check(&h, why, sizeof why) == 0 || !strstr(why, "match first")) rc |= 1024;
    }
    gr_free_file(&g);
    /* A tagged controls record must be validated before any playback starts. */
    {
        unsigned short sizes[256] = {0};
        unsigned char *raw;
        size_t pos, at = 0;
        GrFile bad;
        f = fopen(path, "rb"); fseek(f, 0, SEEK_END); size = ftell(f); rewind(f);
        raw = (unsigned char *)malloc((size_t)size);
        if (!raw || fread(raw, 1, (size_t)size, f) != (size_t)size) { fclose(f); free(raw); return 1; }
        fclose(f);
        for (i = 0; i < (raw[16] - 1) / 3; ++i)
            sizes[raw[17 + 3 * i]] = (unsigned short)((raw[18 + 3 * i] << 8) | raw[19 + 3 * i]);
        for (pos = 16 + raw[16]; pos + 1 < (size_t)size && sizes[raw[pos]]; pos += 1 + sizes[raw[pos]])
            if (raw[pos] == GR_EV_CONTROLS) { at = pos + 1 + 11; break; }
        if (!at) rc |= 16384;
        else {
            unsigned char original = raw[at];
            raw[at] = 3; /* reserved jump mode */
            f = fopen(path, "wb"); fwrite(raw, 1, (size_t)size, f); fclose(f);
            if (gr_read(path, &bad, why, sizeof why) == 0) { rc |= 16384; gr_free_file(&bad); }
            else if (!strstr(why, "fighter controls")) rc |= 16384;
            raw[at] = original;
            f = fopen(path, "wb"); fwrite(raw, 1, (size_t)size, f); fclose(f);
        }
        free(raw);
    }
    /* a killed game: the tail torn mid-event still reads, up to the last whole frame */
    f = fopen(path, "r+b"); fseek(f, 0, SEEK_END); size = ftell(f);
    _chsize_s(_fileno(f), size - 200); fclose(f);
    if (gr_read(path, &g, why, sizeof why) != 0 || g.n != 3 || g.complete) rc |= 2048;
    gr_free_file(&g);
    /* the Lua surface exists and refuses cleanly outside a match */
    if (t_exec("local ok,why=gd.rec_start('selftest_x'); assert(ok==false or ok==nil, 'start outside a match'); "
               "assert(type(gd.rec_list())=='table' and type(gd.rec_status().mode)=='string')", out, sizeof out) != 0) {
        gw_test_fail("rec: Lua surface: %s", out); rc |= 4096;
    }
    DeleteFileA(path);
    if (rc) { gw_test_fail("script rec (.gdlp) format/refusal mask %d", rc); return 1; }
    return 0;
}

/* A zero-offset trim copies its source state without loading it. Its header
 * must keep source controls even while the live match has opposite choices. */
static int test_script_rec_trim_controls(void) {
    char src[MAX_PATH], dst[MAX_PATH], name[40], why[200];
    int size = gw_FighterControls_StateSize(), saved_version = gr_write_ver;
    int paused = gs.paused, version, rc = 0;
    void *saved_controls = malloc((size_t)size), *saved_rec = malloc(sizeof gr);
    FILE *f = NULL;
    GrFile input, output;
    long a, b;
    unsigned char pad[GR_PAD] = {0}, fe[GR_FRAME_PAYLOAD];
    int vp[2][9] = {{0}};
    memset(&input, 0, sizeof input); memset(&output, 0, sizeof output);
    if (!saved_controls || !saved_rec || !gw_FighterControls_StateSave(saved_controls, size)) {
        free(saved_controls); free(saved_rec); gw_test_fail("rec trim controls: cannot preserve test state"); return 1;
    }
    memcpy(saved_rec, &gr, sizeof gr); memset(&gr, 0, sizeof gr);
    for (version = 2; version <= 3; ++version) {
        snprintf(name, sizeof name, "gdlp_trim_controls_v%d", version);
        gr_path(dst, sizeof dst, name, "gdlp");
        gr_path(src, sizeof src, "gdlp_trim_controls_source", "gdlp");
        DeleteFileA(src); DeleteFileA(dst);
        gw_FighterControls_MatchClear();
        gw_FighterControls_MatchSet(0, 2, 1); gw_FighterControls_MatchSet(1, 9, 0);
        gr_write_ver = version;
        f = gr_begin(src, 100, NULL, &a, &b);
        gr_write_ver = GR_VER;
        if (!f) { gw_test_fail("rec trim controls: cannot write source"); rc = 1; break; }
        { unsigned char cmd = GR_EV_FRAME;
          gr_frame_bytes(fe, 100, 1, pad, vp); fwrite(&cmd, 1, 1, f); fwrite(fe, 1, sizeof fe, f); }
        gr_finish(f, a, b); f = NULL;
        if (gr_read(src, &input, why, sizeof why) != 0) {
            gw_test_fail("rec trim controls: cannot read source: %s", why); rc = 1; break;
        }
        gw_FighterControls_MatchSet(0, 2, 0); gw_FighterControls_MatchSet(1, 9, 1);
        gr.pf = input; memset(&input, 0, sizeof input); /* job owns source allocations */
        gr.jfrom = 0; gr.jto = 1; gr.mode = GR_JOB; gr.jk = 1; gr.jpaused = paused;
        snprintf(gr.jdst, sizeof gr.jdst, "%s", name);
        snprintf(gr.name, sizeof gr.name, "source");
        gr_write_trim(); /* actual writer; the source state was never loaded */
        if (gw_FighterControls_MatchGet(0) != 0 || gw_FighterControls_MatchGet(1) != 1 ||
            gr_read(dst, &output, why, sizeof why) != 0) {
            gw_test_fail("rec trim controls: trim touched live mapping or unreadable output"); rc = 1; break;
        }
        if (!output.has_controls || output.controls[0] != (version == 3 ? 1u : 0u) ||
            output.controls[1] != 0u ||
            output.controls_ck[0] != (version == 3 ? 2 : -1) ||
            output.controls_ck[1] != (version == 3 ? 9 : -1)) {
            gw_test_fail("rec trim controls: v%d source mapping replaced by live choices", version); rc = 1; break;
        }
        gr.pf = output; memset(&output, 0, sizeof output);
        gr.mode = GR_LOAD; gr.map[0] = 0; gr.map[1] = 1; gr.map[2] = 2; gr.map[3] = 3;
        gr_after_load(); /* metadata must preserve source X/Y or legacy Both */
        if (gw_FighterControls_Apply(0, 0xc00) != 0xc00 ||
            gw_FighterControls_TapJumpOff(0) != (version == 3 ? 1 : 0) ||
            gw_FighterControls_TapJumpOff(1) != 0) {
            gw_test_fail("rec trim controls: v%d trimmed playback changed the recorded tap jump setting", version); rc = 1;
        }
        gr_free_file(&gr.pf); DeleteFileA(src); DeleteFileA(dst);
        if (rc) break;
    }
    if (f) fclose(f);
    gr_free_file(&input); gr_free_file(&output); gr_free_file(&gr.pf);
    DeleteFileA(src); DeleteFileA(dst);
    memcpy(&gr, saved_rec, sizeof gr); gs.paused = paused; gr_write_ver = saved_version;
    gw_FighterControls_StateLoad(saved_controls, size);
    free(saved_rec); free(saved_controls);
    return rc;
}

/* ---- .gdlp version 2 ------------------------------------------------------------------------- */
/* a whole-match file: GD head events, then Game Start and n frames of Frame Start / Pre / Post / Bookend x2 ports */
static int t_whole(const char *path, const unsigned char *game_info, int n, int with_seg, int *rc_out) {
    long a, b;
    FILE *f = gr_open(path, &a, &b);
    unsigned char head[GR_HEAD_MAX], ev[1 + GR_SZ_START];
    size_t hn;
    int i, port;
    (void) rc_out;
    if (f == NULL) return -1;
    hn = gw_Gdlp_MatchHead(head, game_info);
    fwrite(head, 1, hn, f);
    if (with_seg) {
        unsigned char seg[48];
        memset(seg, 0, sizeof seg); gr_put32(seg, 5); gr_put32(seg + 4, 9);
        gr_ev(f, GR_EV_SEGMENT, seg);
    }
    memset(ev, 0, sizeof ev);
    ev[0] = 0x36; ev[1] = 3; ev[2] = 19; ev[3] = 1;
    memcpy(ev + 5, game_info, 0x138);
    fwrite(ev, 1, 1 + GR_SZ_START, f);
    for (i = 0; i < n; ++i) {
        int frame = -123 + i;
        unsigned char fs[1 + GR_SZ_FSTART], pre[1 + GR_SZ_PRE], post[1 + GR_SZ_POST], bk[9];
        memset(fs, 0, sizeof fs); fs[0] = 0x3A; gr_put32(fs + 1, (unsigned) frame); gr_put32(fs + 5, 0x1234u + (unsigned) i);
        fwrite(fs, 1, sizeof fs, f);
        for (port = 0; port < 2; ++port) {
            memset(pre, 0, sizeof pre); pre[0] = 0x37; gr_put32(pre + 1, (unsigned) frame); pre[5] = (unsigned char) port;
            fwrite(pre, 1, sizeof pre, f);
        }
        for (port = 0; port < 2; ++port) {
            memset(post, 0, sizeof post); post[0] = 0x38; gr_put32(post + 1, (unsigned) frame); post[5] = (unsigned char) port;
            post[7] = (unsigned char) (port ? 10 : 2); post[0x21] = 4;
            fwrite(post, 1, sizeof post, f);
        }
        bk[0] = 0x3C; gr_put32(bk + 1, (unsigned) frame); gr_put32(bk + 5, (unsigned) frame);
        fwrite(bk, 1, sizeof bk, f);
    }
    return gr_finish(f, a, b);
}

/* walks a container as a Slippi reader does: counts per command, the GD events skipped by size, returns 0 if the
 * declared raw length, the size table and the end event are all consistent */
static int t_slp_walk(const char *path, int counts[256], int *gd_events) {
    FILE *f = fopen(path, "rb");
    unsigned char *d;
    long n;
    size_t pos, end, tabend;
    int i, ended = 0;
    unsigned short sz[256];
    if (f == NULL) return -1;
    fseek(f, 0, SEEK_END); n = ftell(f); fseek(f, 0, SEEK_SET);
    d = (unsigned char *) malloc((size_t) n);
    if (d == NULL || fread(d, 1, (size_t) n, f) != (size_t) n) { fclose(f); free(d); return -1; }
    fclose(f);
    memset(counts, 0, 256 * sizeof(int));
    memset(sz, 0, sizeof sz);
    *gd_events = 0;
    if (n < 20 || memcmp(d, "{U\x03raw[$U#l", 11) != 0 || d[15] != 0x35) { free(d); return -2; }
    end = 15 + (size_t) gr_get32(d + 11);
    tabend = 16 + (size_t) d[16];
    for (i = 0; i < (d[16] - 1) / 3; ++i) sz[d[17 + 3 * i]] = (unsigned short) ((d[18 + 3 * i] << 8) | d[19 + 3 * i]);
    for (pos = tabend; pos < end;) {
        int cmd = d[pos];
        if (sz[cmd] == 0 || pos + 1 + sz[cmd] > end) { free(d); return -3; }
        counts[cmd]++;
        if (cmd >= 0x90) ++*gd_events;
        pos += 1 + sz[cmd];
        if (cmd == 0x39) { ended = 1; break; }
    }
    i = (!ended || pos != end || memcmp(d + end, "U\x08metadata", 10) != 0) ? -4 : 0;
    free(d);
    return i;
}

static int test_script_rec_v2(void) {
    char path[MAX_PATH], sidecar[MAX_PATH], ext[MAX_PATH], why[240];
    GrFile g;
    long a, b;
    FILE *f;
    unsigned char fe[GR_FRAME_PAYLOAD], pad[GR_PAD], *state, *got;
    int vp[2][9], i, rc = 0;
    size_t total = 2 * GR_STCHUNK + 12345, nread;
    memset(vp, 0, sizeof vp);
    memset(pad, 0x33, sizeof pad);
    gr_path(path, sizeof path, "gdlp_selftest_v2_legacy", "gdlp");
    gr_write_ver = 2;
    f = gr_begin(path, 7, NULL, &a, &b);
    gr_write_ver = GR_VER;
    if (f == NULL) return 1;
    gr_finish(f, a, b);
    if (gr_read(path, &g, why, sizeof why) != 0 || g.ver != 2 || g.has_controls ||
        g.controls[0] || g.controls[1]) rc |= 8192;
    gr_free_file(&g);
    DeleteFileA(path);
    /* version 1 still reads: the old header, the old 9-event size table, no Slippi or state events */
    gr_path(path, sizeof path, "gdlp_selftest_v1", "gdlp");
    DeleteFileA(path);
    gr_write_ver = 1;
    f = gr_begin(path, 7, "gdlp_selftest_v1.gdst", &a, &b);
    gr_write_ver = GR_VER;
    if (f == NULL) { gw_test_fail("rec v2: cannot create the v1 file"); return 1; }
    for (i = 0; i < 3; ++i) { unsigned char c = GR_EV_FRAME; gr_frame_bytes(fe, 7 + i, 1, pad, vp); fwrite(&c, 1, 1, f); fwrite(fe, 1, sizeof fe, f); }
    gr_finish(f, a, b);
    if (gr_read(path, &g, why, sizeof why) != 0 || g.ver != 1 || g.n != 3 || g.base != 7 || g.emb_size != 0 || (g.flags & (GR_FL_WHOLE | GR_FL_SLIPPI)) ||
        !g.complete) rc |= 1;
    else {
        int counts[256], gd;
        gr_free_file(&g);
        if (t_slp_walk(path, counts, &gd) != 0 || counts[GR_EV_FRAME] != 3 || counts[GR_EV_STCHUNK] != 0) rc |= 2;
    }
    DeleteFileA(path);

    /* version 2: the start state embedded in chunks, read back byte for byte; the sidecar wins while it exists */
    gr_path(path, sizeof path, "gdlp_selftest_v2", "gdlp");
    gr_path(sidecar, sizeof sidecar, "gdlp_selftest_v2", "gdst");
    gr_path(ext, sizeof ext, "_emb_gdlp_selftest_v2", "gdst");
    DeleteFileA(path); DeleteFileA(sidecar); DeleteFileA(ext);
    state = (unsigned char *) malloc(total);
    got = (unsigned char *) malloc(total);
    if (state == NULL || got == NULL) { free(state); free(got); gw_test_fail("rec v2: out of memory"); return 1; }
    for (i = 0; i < (int) total; ++i) state[i] = (unsigned char) (i * 31 + (i >> 8) * 7);
    f = fopen(sidecar, "wb");
    if (f == NULL) { free(state); free(got); gw_test_fail("rec v2: cannot write %s", sidecar); return 1; }
    fwrite(state, 1, total, f); fclose(f);
    gr_write_ver = 2; /* exercise the real legacy embedded-state reader */
    f = gr_begin(path, 100, "gdlp_selftest_v2.gdst", &a, &b);
    gr_write_ver = GR_VER;
    if (f == NULL) { free(state); free(got); gw_test_fail("rec v2: cannot create"); return 1; }
    { unsigned char c = GR_EV_FRAME; gr_frame_bytes(fe, 100, 1, pad, vp); fwrite(&c, 1, 1, f); fwrite(fe, 1, sizeof fe, f); }
    gr_finish(f, a, b);
    if (gr_read(path, &g, why, sizeof why) != 0 || g.ver != 2 || g.emb_size != total || g.emb_got != total || g.n != 1) {
        rc |= 4;
        gw_test_fail("rec v2: read: %s ver=%u emb=%u/%u (want %u) n=%d", why, g.ver, g.emb_got, g.emb_size, (unsigned) total, g.n);
    }
    else {
        char sp[MAX_PATH];
        if (gr_state_path(&g, path, "gdlp_selftest_v2", sp, sizeof sp, why, sizeof why) != 0 || strcmp(sp, sidecar) != 0) rc |= 8; /* sidecar first */
        DeleteFileA(sidecar);
        if (gr_state_path(&g, path, "gdlp_selftest_v2", sp, sizeof sp, why, sizeof why) != 0 || strcmp(sp, ext) != 0) rc |= 16; /* then the embedded one */
        else {
            f = fopen(sp, "rb");
            nread = f != NULL ? fread(got, 1, total + 1, f) : 0;
            if (f) fclose(f);
            if (nread != total || memcmp(got, state, total) != 0) rc |= 32;
        }
    }
    gr_free_file(&g);
    free(state); free(got);
    DeleteFileA(path); DeleteFileA(sidecar); DeleteFileA(ext);
    if (rc) { gw_test_fail("rec v2: v1 read / embedded state mask %d", rc); return 1; }
    return 0;
}

static int test_script_rec_export(void) {
    char path[MAX_PATH], out[MAX_PATH], why[300];
    unsigned char gi[0x138];
    int counts[256], gd, rc = 0, frames = 0, s;
    GrFile g;
    gr_path(path, sizeof path, "gdlp_selftest_whole", "gdlp");
    gr_path(out, sizeof out, "gdlp_selftest_whole", "slp");
    DeleteFileA(path); DeleteFileA(out);
    memset(gi, 0, sizeof gi);
    gi[0x0E] = 0; gi[0x0F] = 31;
    for (s = 0; s < 6; ++s) gi[0x60 + 0x24 * s + 1] = 3;      /* every slot empty... */
    gi[0x60] = 2;  gi[0x61] = 0;  gi[0x63] = 1;               /* ...P1 ck 2, human, costume 1 */
    gi[0x60 + 0x24] = 10; gi[0x61 + 0x24] = 1; gi[0x63 + 0x24] = 0; /* P2 ck 10, cpu */
    gr_mods_override = "";
    /* a retail-only whole match: valid .gdlp, then a plain .slp with only the standard events */
    if (t_whole(path, gi, 6, 0, NULL) != 0) { gr_mods_override = NULL; gw_test_fail("rec export: cannot write"); return 1; }
    if (gr_read(path, &g, why, sizeof why) != 0) { rc |= 1; gw_test_fail("rec export: read: %s", why); }
    else {
        if (g.ver != GR_VER || !g.has_controls || !(g.flags & GR_FL_WHOLE) || !(g.flags & GR_FL_SLIPPI) || !(g.flags & GR_FL_RETAIL) || (g.flags & GR_FL_STATE)) rc |= 2;
        if (!g.has_start || g.n_pre != 12 || g.n_post != 12 || g.first_frame != -123 || g.last_frame != -118 || !g.complete) {
            rc |= 4;
            gw_test_fail("rec export: start=%d pre=%d post=%d frames %d..%d complete=%d", g.has_start, g.n_pre, g.n_post, g.first_frame, g.last_frame, g.complete);
        }
        if (!g.fi[0].present || g.fi[0].chr != 2 || g.fi[0].costume != 1 || g.fi[1].chr != 10 || g.fi[2].present || g.stage != 31) rc |= 8;
        gr_free_file(&g);
    }
    if (t_slp_walk(path, counts, &gd) != 0 || gd < 8 || counts[0x36] != 1 || counts[0x3C] != 6) rc |= 16; /* GD events are skipped by size, the rest is consistent */
    if (gr_export_slp(path, out, why, sizeof why, &frames) != 0 || frames != 12) { rc |= 32; gw_test_fail("rec export: %s (frames %d)", why, frames); }
    else if (t_slp_walk(out, counts, &gd) != 0 || gd != 0 || counts[0x36] != 1 || counts[0x37] != 12 || counts[0x38] != 12 ||
             counts[0x3A] != 6 || counts[0x3C] != 6 || counts[0x39] != 1) rc |= 64;
    /* refusals, each with its reason */
    {
        unsigned char gk[0x138];
        memcpy(gk, gi, sizeof gk);
        gk[0x60 + 0x24] = 40; /* a Geno / m-ex character */
        DeleteFileA(path); DeleteFileA(out);
        t_whole(path, gk, 2, 0, NULL);
        if (gr_export_slp(path, out, why, sizeof why, NULL) == 0 || !strstr(why, "Geno") || gr_exists(out)) rc |= 128;
        DeleteFileA(path);
        t_whole(path, gi, 2, 1, NULL); /* a trim */
        if (gr_export_slp(path, out, why, sizeof why, NULL) == 0 || !strstr(why, "trimmed") || gr_exists(out)) rc |= 256;
        DeleteFileA(path);
        gr_mods_override = "some-mod";
        t_whole(path, gi, 2, 0, NULL);
        if (gr_export_slp(path, out, why, sizeof why, NULL) == 0 || !strstr(why, "some-mod") || gr_exists(out)) rc |= 512;
        gr_mods_override = "";
        DeleteFileA(path);
    }
    {   /* a mid-match LAB clip (starts from a state) and a version 1 file */
        long a, b;
        FILE *f;
        gr_mods_override = "";
        f = gr_begin(path, 50, "gdlp_selftest_whole.gdst", &a, &b);
        if (f != NULL) gr_finish(f, a, b);
        if (gr_export_slp(path, out, why, sizeof why, NULL) == 0 || !strstr(why, "saved state") || gr_exists(out)) rc |= 1024;
        DeleteFileA(path);
        gr_write_ver = 1;
        f = gr_begin(path, 50, NULL, &a, &b);
        gr_write_ver = GR_VER;
        if (f != NULL) gr_finish(f, a, b);
        if (gr_export_slp(path, out, why, sizeof why, NULL) == 0 || !strstr(why, "format-1") || gr_exists(out)) rc |= 2048;
    }
    /* a whole-match file is not a LAB clip: lab rec play refuses it, and says how to play it */
    {
        GrFile h;
        memset(&h, 0, sizeof h);
        h.flags = GR_FL_WHOLE | GR_FL_SLIPPI;
        if (gr_check(&h, why, sizeof why) == 0 || !strstr(why, "MELEE_GDLP")) rc |= 4096;
    }
    gr_mods_override = NULL;
    DeleteFileA(path); DeleteFileA(out);
    if (rc) { gw_test_fail("rec export: mask %d", rc); return 1; }
    return 0;
}

/* R5: the rewind log keeps all four ports' pads (the PC PADStatus is 16 bytes: 4 x 16 = 64, not 48) */
typedef char gs_log_pad_is_four_ports[(sizeof(((GsLogEntry *) 0)->pad) == 4 * sizeof(PADStatus) && sizeof(((GsLogEntry *) 0)->pad) == GR_PAD) ? 1 : -1];
static int test_script_rec_port4(void) {
    GsLogEntry *e = &gs_log[0];
    unsigned char before[sizeof e->pad], ev[GR_FRAME_PAYLOAD], pads[GR_PAD];
    int vp[2][9], i, tag = e->tag, have = e->have;
    memcpy(before, e->pad, sizeof before);
    memset(vp, 0, sizeof vp);
    for (i = 0; i < GR_PAD; ++i) pads[i] = (unsigned char) (i + 1);
    /* the log's record path: gw_LabPad_Record copies min(size, sizeof e->pad) */
    memcpy(e->pad, pads, GR_PAD < (int) sizeof e->pad ? GR_PAD : sizeof e->pad);
    gr_frame_bytes(ev, 1, 1, e->pad, vp);                 /* the recorder reads the same bytes (save-history) */
    memcpy(e->pad, before, sizeof before);
    e->tag = tag; e->have = (unsigned char) have;
    /* port 4 is bytes 48..63: all of them reach the file */
    for (i = 48; i < 64; ++i) if (ev[5 + i] != (unsigned char) (i + 1)) { gw_test_fail("rec port 4: byte %d lost (%d)", i, ev[5 + i]); return 1; }
    return 0;
}
