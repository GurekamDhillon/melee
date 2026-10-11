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
    f = gr_begin(path, 100, "gdlp_selftest.gdst", &a, &b);
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
