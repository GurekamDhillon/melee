/* gw_script_rec.inc - the LAB input recorder (.gdlp, "GD live playback"). Included by gw_script.c
 * right after gs_rw_request, so it sees the rewind log, the state files and the pending-operation
 * machinery. docs/geno.md 14.20 is the format reference.
 *
 * A recording is TWO files in <script-data>/geno-lab_lab/recordings/:
 *   <name>.gdst   the LAB state at the first recorded frame (the state library's own file format,
 *                 so it carries and checks the exe / disc / mods / Geno hashes)
 *   <name>.gdlp   a Slippi-framed event stream: the same container as .slp (UBJSON `raw` array, the
 *                 0x35 Payloads size table, 0x39 Game End, metadata), plus GD events 0x90..0x97 that
 *                 a Slippi reader skips by their sizes: header, fighters (with Geno define names),
 *                 identity hashes, state reference, one raw-pad frame per logic frame, marks,
 *                 segment provenance (trim) and text (the mods in words).
 * Exact replay feeds the PADStatus[4] each frame (and the CPU slots' virtual pads) back at
 * HSD_PadRenewMasterStatus, the point the rewind log uses (controller.c), so it needs nothing the
 * rewind does not. The file is flushed every second: a killed game loses at most one second.
 */
#define GR_PAD 64 /* 4 x sizeof(PADStatus): the PC struct is 16 bytes (extButton); the rewind log keeps only 48 */
#define GR_VP 72
#define GR_VER 1
#define GR_FRAME_PAYLOAD (4 + 1 + GR_PAD + GR_VP)
#define GR_EV_HEAD 0x90
#define GR_EV_FIGHTER 0x91
#define GR_EV_IDENT 0x92
#define GR_EV_STATE 0x93
#define GR_EV_FRAME 0x94
#define GR_EV_MARK 0x95
#define GR_EV_SEGMENT 0x96
#define GR_EV_TEXT 0x97
#define GR_FL_RETAIL 1  /* every fighter is in the retail roster (a candidate for .slp export) */
#define GR_FL_STATE 2   /* starts from the .gdst beside it */
enum { GR_IDLE, GR_ARM, GR_REC, GR_LOAD, GR_PLAY, GR_JOB };

typedef struct { int tag; unsigned char have; unsigned char pad[GR_PAD]; int vp[2][9]; } GrFrame;
typedef struct {
    int present, chr, costume, cpu;
    char name[25];
} GrFighter;
typedef struct {
    unsigned ver, flags, base, unix_time;
    GrFighter fi[6];
    uint64_t id[4];
    int stage;
    unsigned geno_version;
    char state[41];
    unsigned state_size;
    char mods[256];
    GrFrame *fr;
    int n, cap, complete;
    struct { int frame; char label[33]; } marks[64];
    int nmarks;
    char seg_src[41];
    int seg_from, seg_to;
} GrFile;

static struct {
    int mode;
    char name[48], last[48];
    char gdlp[MAX_PATH], gdst[MAX_PATH];
    FILE *f;
    long raw_len_at, raw_start;
    int base, count;
    long *off;
    int offcap;
    GrFile pf;      /* playback / trim source */
    int tag0, loop, map[4];
    int jk, jphase, jfrom, jto, jguard, jnow0, jF, jpaused;
    char jdst[48];
} gr;

static void gr_put32(unsigned char *p, unsigned v) {
    p[0] = (unsigned char) (v >> 24); p[1] = (unsigned char) (v >> 16);
    p[2] = (unsigned char) (v >> 8); p[3] = (unsigned char) v;
}
static unsigned gr_get32(const unsigned char *p) {
    return ((unsigned) p[0] << 24) | ((unsigned) p[1] << 16) | ((unsigned) p[2] << 8) | p[3];
}
static void gr_put64(unsigned char *p, uint64_t v) { gr_put32(p, (unsigned) (v >> 32)); gr_put32(p + 4, (unsigned) v); }
static uint64_t gr_get64(const unsigned char *p) { return ((uint64_t) gr_get32(p) << 32) | gr_get32(p + 4); }

/* payload sizes (bytes after the command byte) */
static int gr_size_of(int cmd) {
    switch (cmd) {
    case 0x39: return 6;
    case GR_EV_HEAD: return 16;
    case GR_EV_FIGHTER: return 30;
    case GR_EV_IDENT: return 40;
    case GR_EV_STATE: return 44;
    case GR_EV_FRAME: return GR_FRAME_PAYLOAD;
    case GR_EV_MARK: return 36;
    case GR_EV_SEGMENT: return 48;
    case GR_EV_TEXT: return 256;
    }
    return -1;
}
static const unsigned char gr_cmds[] = { 0x39, GR_EV_HEAD, GR_EV_FIGHTER, GR_EV_IDENT, GR_EV_STATE,
                                         GR_EV_FRAME, GR_EV_MARK, GR_EV_SEGMENT, GR_EV_TEXT };

static void gr_dir(char *out, size_t cap) {
    char *p;
    snprintf(out, cap, "%s\\geno-lab_lab", gs.data_dir);
    for (p = out + strlen(gs.data_dir); *p != '\0'; ++p) if (*p == '/') *p = '_';
    CreateDirectoryA(gs.data_dir, NULL);
    CreateDirectoryA(out, NULL);
    strncat(out, "\\recordings", cap - strlen(out) - 1);
    CreateDirectoryA(out, NULL);
}
static int gr_name_ok(const char *n) {
    size_t i, len = n != NULL ? strlen(n) : 0;
    if (len < 1 || len > 40) return 0;
    for (i = 0; i < len; ++i) {
        char c = n[i];
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') || c == '_' || c == '-')) return 0;
    }
    return 1;
}
static void gr_path(char *out, size_t cap, const char *name, const char *ext) {
    char dir[MAX_PATH];
    gr_dir(dir, sizeof dir);
    snprintf(out, cap, "%s\\%s.%s", dir, name, ext);
}
static int gr_exists(const char *path) { return GetFileAttributesA(path) != INVALID_FILE_ATTRIBUTES; }

static void gr_free_file(GrFile *g) { free(g->fr); memset(g, 0, sizeof *g); }

/* ---- writing ----------------------------------------------------------------------------- */
static void gr_ev(FILE *f, int cmd, const unsigned char *payload) {
    int n = gr_size_of(cmd);
    unsigned char c = (unsigned char) cmd;
    fwrite(&c, 1, 1, f);
    fwrite(payload, 1, (size_t) n, f);
}

/* the container, the size table and the header events for the match as it is now */
static FILE *gr_begin(const char *path, int base, const char *statefile, long *raw_len_at, long *raw_start) {
    static const unsigned char head[] = { '{', 'U', 3, 'r', 'a', 'w', '[', '$', 'U', '#', 'l' };
    unsigned char zero[4] = { 0, 0, 0, 0 }, tab[2 + 3 * 9], ev[300];
    FILE *f = fopen(path, "wb");
    uint64_t id[4];
    int s, i, retail = 1, n = (int) sizeof gr_cmds;
    const char *m;
    if (f == NULL) return NULL;
    fwrite(head, 1, sizeof head, f);
    *raw_len_at = ftell(f);
    fwrite(zero, 1, 4, f);
    *raw_start = ftell(f);
    tab[0] = 0x35;
    tab[1] = (unsigned char) (1 + 3 * n);
    for (i = 0; i < n; ++i) {
        int sz = gr_size_of(gr_cmds[i]);
        tab[2 + 3 * i] = gr_cmds[i];
        tab[3 + 3 * i] = (unsigned char) (sz >> 8);
        tab[4 + 3 * i] = (unsigned char) sz;
    }
    fwrite(tab, 1, 2 + 3 * (size_t) n, f);
    for (s = 0; s < 6; ++s)
        if (gs_players_present(s) && (gw_ScriptGame_FighterI(s, SI_CHAR) < 0 || gw_ScriptGame_FighterI(s, SI_CHAR) > 25)) retail = 0;
    memset(ev, 0, sizeof ev);
    ev[0] = 0; ev[1] = GR_VER;
    ev[2] = (unsigned char) ((retail ? GR_FL_RETAIL : 0) | (statefile != NULL ? GR_FL_STATE : 0));
    ev[3] = 6;
    gr_put32(ev + 4, (unsigned) base);
    ev[8] = 0; ev[9] = GR_PAD; ev[10] = 0; ev[11] = GR_VP;
    gr_put32(ev + 12, (unsigned) time(NULL));
    gr_ev(f, GR_EV_HEAD, ev);
    for (s = 0; s < 6; ++s) {
        int present = gs_players_present(s), chr = present ? gw_ScriptGame_FighterI(s, SI_CHAR) : -1;
        memset(ev, 0, sizeof ev);
        ev[0] = (unsigned char) s; ev[1] = (unsigned char) present;
        ev[2] = (unsigned char) (chr >> 8); ev[3] = (unsigned char) chr;
        ev[4] = (unsigned char) (present ? gw_ScriptGame_FighterI(s, SI_COSTUME) : 0xFF);
        ev[5] = (unsigned char) (present ? gw_ScriptGame_FighterI(s, SI_SLOT_TYPE) : 0xFF);
        if (present) snprintf((char *) ev + 6, 24, "%s", gs_char_name(chr));
        gr_ev(f, GR_EV_FIGHTER, ev);
    }
    gs_identity(id);
    memset(ev, 0, sizeof ev);
    for (i = 0; i < 4; ++i) gr_put64(ev + 8 * i, id[i]);
    gr_put32(ev + 32, (unsigned) gw_ScriptGame_StageKind());
    gr_put32(ev + 36, GENO_VERSION);
    gr_ev(f, GR_EV_IDENT, ev);
    memset(ev, 0, sizeof ev);
    if (statefile != NULL) snprintf((char *) ev, 40, "%s", statefile);
    gr_ev(f, GR_EV_STATE, ev);
    m = gw_Mods_Describe();
    memset(ev, 0, sizeof ev);
    ev[0] = 1;
    snprintf((char *) ev + 1, 255, "%s", m != NULL ? m : "");
    gr_ev(f, GR_EV_TEXT, ev);
    return f;
}

static int gr_finish(FILE *f, long raw_len_at, long raw_start) {
    static const char meta[] = "U\x08metadata{U\x08playedOnSU\x08melee-pc}}";
    unsigned char ev[6], len[4];
    long end;
    int bad;
    memset(ev, 0, sizeof ev);
    ev[0] = 7;
    gr_ev(f, 0x39, ev);
    end = ftell(f);
    fwrite(meta, 1, sizeof meta - 1, f);
    gr_put32(len, (unsigned) (end - raw_start));
    bad = fseek(f, raw_len_at, SEEK_SET) != 0;
    fwrite(len, 1, 4, f);
    bad |= ferror(f) != 0;
    bad |= fclose(f) != 0;
    return bad ? -1 : 0;
}

static void gr_frame_bytes(unsigned char *ev, int tag, int have, const unsigned char *pad, const int vp[2][9]) {
    int i, k;
    memset(ev, 0, GR_FRAME_PAYLOAD);
    gr_put32(ev, (unsigned) tag);
    ev[4] = (unsigned char) have;
    if (have && pad != NULL) memcpy(ev + 5, pad, GR_PAD);
    for (i = 0; i < 2; ++i) for (k = 0; k < 9; ++k) gr_put32(ev + 5 + GR_PAD + (i * 9 + k) * 4, (unsigned) vp[i][k]);
}

/* ---- reading ----------------------------------------------------------------------------- */
static int gr_read(const char *path, GrFile *g, char *why, size_t cap) {
    FILE *f = fopen(path, "rb");
    unsigned char *d, szt[256][2];
    long n;
    size_t pos, tabend;
    int i;
    memset(g, 0, sizeof *g);
    if (f == NULL) { snprintf(why, cap, "no such recording"); return -1; }
    fseek(f, 0, SEEK_END); n = ftell(f); fseek(f, 0, SEEK_SET);
    d = (unsigned char *) malloc((size_t) n + 1);
    if (d == NULL || fread(d, 1, (size_t) n, f) != (size_t) n) { fclose(f); free(d); snprintf(why, cap, "unreadable"); return -1; }
    fclose(f);
    if (n < 15 + 2 || memcmp(d, "{U\x03raw[$U#l", 11) != 0) { free(d); snprintf(why, cap, "not a .gdlp (or .slp) file"); return -1; }
    pos = 15;
    memset(szt, 0xFF, sizeof szt);
    if (d[pos] != 0x35) { free(d); snprintf(why, cap, "no Payloads event"); return -1; }
    tabend = pos + 1 + (size_t) d[pos + 1];
    for (i = 0; i < (d[pos + 1] - 1) / 3 && pos + 2 + (size_t) i * 3 + 2 < (size_t) n; ++i) {
        unsigned char c = d[pos + 2 + i * 3];
        szt[c][0] = d[pos + 3 + i * 3]; szt[c][1] = d[pos + 4 + i * 3];
    }
    pos = tabend;
    for (;;) {
        int cmd, sz;
        const unsigned char *p;
        if (pos >= (size_t) n) break;
        cmd = d[pos];
        if (cmd == 'U') break; /* the metadata */
        if (szt[cmd][0] == 0xFF && szt[cmd][1] == 0xFF) break;
        sz = (szt[cmd][0] << 8) | szt[cmd][1];
        if (pos + 1 + (size_t) sz > (size_t) n) break; /* a torn last event */
        p = d + pos + 1;
        switch (cmd) {
        case GR_EV_HEAD:
            g->ver = ((unsigned) p[0] << 8) | p[1]; g->flags = p[2]; g->base = gr_get32(p + 4);
            g->unix_time = gr_get32(p + 12);
            break;
        case GR_EV_FIGHTER:
            if (p[0] < 6) {
                GrFighter *x = &g->fi[p[0]];
                x->present = p[1]; x->chr = (short) ((p[2] << 8) | p[3]);
                x->costume = p[4] == 0xFF ? -1 : p[4]; x->cpu = p[5] == 0xFF ? -1 : p[5];
                memcpy(x->name, p + 6, 24); x->name[24] = '\0';
            }
            break;
        case GR_EV_IDENT:
            for (i = 0; i < 4; ++i) g->id[i] = gr_get64(p + 8 * i);
            g->stage = (int) gr_get32(p + 32); g->geno_version = gr_get32(p + 36);
            break;
        case GR_EV_STATE: memcpy(g->state, p, 40); g->state[40] = '\0'; break;
        case GR_EV_TEXT: if (p[0] == 1) { memcpy(g->mods, p + 1, 255); g->mods[255] = '\0'; } break;
        case GR_EV_MARK:
            if (g->nmarks < 64) {
                g->marks[g->nmarks].frame = (int) gr_get32(p);
                memcpy(g->marks[g->nmarks].label, p + 4, 32); g->marks[g->nmarks].label[32] = '\0';
                g->nmarks++;
            }
            break;
        case GR_EV_SEGMENT:
            g->seg_from = (int) gr_get32(p); g->seg_to = (int) gr_get32(p + 4);
            memcpy(g->seg_src, p + 8, 40); g->seg_src[40] = '\0';
            break;
        case GR_EV_FRAME:
            if (g->n == g->cap) {
                int nc = g->cap ? g->cap * 2 : 1024;
                GrFrame *nf = (GrFrame *) realloc(g->fr, (size_t) nc * sizeof *nf);
                if (nf == NULL) { free(d); snprintf(why, cap, "out of memory"); gr_free_file(g); return -1; }
                g->fr = nf; g->cap = nc;
            }
            {
                GrFrame *fr = &g->fr[g->n++];
                int k;
                fr->tag = (int) gr_get32(p); fr->have = p[4];
                memcpy(fr->pad, p + 5, GR_PAD);
                for (i = 0; i < 2; ++i) for (k = 0; k < 9; ++k) fr->vp[i][k] = (int) gr_get32(p + 5 + GR_PAD + (i * 9 + k) * 4);
            }
            break;
        case 0x39: g->complete = 1; break;
        default: break; /* any other event: skipped by its size, as a Slippi reader does */
        }
        pos += 1 + (size_t) sz;
        if (cmd == 0x39) break;
    }
    free(d);
    if (g->ver == 0) { gr_free_file(g); snprintf(why, cap, "no GD header (an .slp is not a LAB recording)"); return -1; }
    if (g->ver > GR_VER) { snprintf(why, cap, "recorded by a newer game (format %u, this reads %d)", g->ver, GR_VER); gr_free_file(g); return -1; }
    return 0;
}

/* a recording is refused, with the reason, unless this build / mods / Geno data / match is the one it was made in */
static int gr_check(const GrFile *g, char *why, size_t cap) {
    uint64_t id[4];
    int s;
    gs_identity(id);
    if (g->id[0] != id[0]) { snprintf(why, cap, "recorded by another build of the game"); return -1; }
    if (g->id[1] != id[1]) { snprintf(why, cap, "recorded with another disc"); return -1; }
    if (g->id[2] != id[2]) { snprintf(why, cap, "recorded with other mods (it used: %s)", g->mods[0] ? g->mods : "none"); return -1; }
    if (g->id[3] != id[3] || g->geno_version != GENO_VERSION) { snprintf(why, cap, "recorded with other Geno fighter data (a define changed)"); return -1; }
    if (!gs.match_active) { snprintf(why, cap, "start the LAB match first"); return -1; }
    for (s = 0; s < 6; ++s) {
        int present = gs_players_present(s);
        if (g->fi[s].present != present ||
            (present && (gw_ScriptGame_FighterI(s, SI_CHAR) != g->fi[s].chr || gw_ScriptGame_FighterI(s, SI_COSTUME) != g->fi[s].costume))) {
            snprintf(why, cap, "this recording is P%d %s (costume %d): start that match first", s + 1,
                     g->fi[s].present ? g->fi[s].name : "empty", g->fi[s].costume);
            return -1;
        }
    }
    if (gw_ScriptGame_StageKind() != g->stage) { snprintf(why, cap, "this recording is on another stage: start that match first"); return -1; }
    why[0] = '\0';
    return 0;
}

/* ---- recording --------------------------------------------------------------------------- */
static void gr_say(int col, const char *fmt, ...) {
    char b[300];
    va_list ap;
    va_start(ap, fmt); vsnprintf(b, sizeof b, fmt, ap); va_end(ap);
    gw_Console_Print(col, "%s", b);
    gw_log("rec: %s", b);
}

static void gr_stop_rec(const char *why) {
    if (gr.mode != GR_REC) return;
    if (gr.f != NULL) {
        fflush(gr.f);
        if (gr.count >= 0 && gr.count < gr.offcap) { /* drop a truncated tail */
            fseek(gr.f, gr.off[gr.count], SEEK_SET);
            _chsize_s(_fileno(gr.f), gr.off[gr.count]);
        }
        gr_finish(gr.f, gr.raw_len_at, gr.raw_start);
        gr.f = NULL;
    }
    snprintf(gr.last, sizeof gr.last, "%s", gr.name);
    gr_say(GS_GREEN, "recording \"%s\" stopped (%s): %d frames, %.1f s", gr.name, why, gr.count, gr.count / 60.0);
    gr.mode = GR_IDLE;
}

static void gr_capture(const void *st, int size) {
    unsigned char ev[GR_FRAME_PAYLOAD];
    int tag, idx, vp[2][9], i, k;
    if (gr.mode != GR_REC || gr.f == NULL || !gs.match_active) return;
    tag = gs_ring_now();
    idx = tag - gr.base;
    if (idx < 0 || idx > gr.count) { gr_stop_rec("the timeline left the recording"); return; }
    if (idx < gr.count) {
        if (gs_rw_replaying(tag)) return; /* the rewind log replays the very pads already written */
        gr.count = idx; /* a live frame at an earlier tag: the take was forked here */
        fseek(gr.f, gr.off[idx], SEEK_SET);
    }
    if (gr.count + 1 >= gr.offcap) {
        long *no = (long *) realloc(gr.off, (size_t) (gr.offcap ? gr.offcap * 2 : 4096) * sizeof(long));
        if (no == NULL) { gr_stop_rec("out of memory"); return; }
        gr.off = no; gr.offcap = gr.offcap ? gr.offcap * 2 : 4096;
    }
    for (i = 0; i < 2; ++i) for (k = 0; k < 9; ++k) vp[i][k] = gw_ScriptGame_VirtualPadField(i + 4, k, 0, 0);
    gr.off[idx] = ftell(gr.f);
    {
        unsigned char pad[GR_PAD];
        memset(pad, 0, sizeof pad);
        if (st != NULL) memcpy(pad, st, size < GR_PAD ? (size_t) size : GR_PAD);
        gr_frame_bytes(ev, tag, st != NULL, pad, vp);
    }
    {
        unsigned char c = GR_EV_FRAME;
        fwrite(&c, 1, 1, gr.f);
        fwrite(ev, 1, sizeof ev, gr.f);
    }
    gr.count = idx + 1;
    gr.off[gr.count] = ftell(gr.f);
    if (gr.count % 60 == 0) fflush(gr.f);
}

/* ---- playback ---------------------------------------------------------------------------- */
static void gr_stop_play(const char *why) {
    if (gr.mode != GR_PLAY && gr.mode != GR_LOAD) return;
    gr_say(GS_GREEN, "playback of \"%s\" ended (%s); the pads are yours again", gr.name, why);
    gr.mode = GR_IDLE;
    gr_free_file(&gr.pf);
}

/* 0 = not playing, 1 = out holds the pads, 2 = the frame renewed nothing */
static int gr_feed(int tag, void *out, int size) {
    int idx, d;
    const GrFrame *fr;
    if ((gr.mode != GR_PLAY && gr.mode != GR_JOB) || gr.pf.n == 0) return 0;
    idx = tag - gr.tag0;
    if (idx < 0) { if (gr.mode == GR_PLAY) gr_stop_play("rewound before the start"); return 0; }
    if (idx >= gr.pf.n) {
        if (gr.mode == GR_PLAY && !gr.loop) gr_stop_play("finished");
        return gr.mode == GR_PLAY && gr.loop ? 2 : 0;
    }
    fr = &gr.pf.fr[idx];
    {
        int i, k;
        for (i = 0; i < 2; ++i) for (k = 0; k < 9; ++k) gw_ScriptGame_VirtualPadField(i + 4, k, fr->vp[i][k], 1);
    }
    if (!fr->have) return 2;
    {
        int per = size / 4, perrec = GR_PAD / 4;
        int n = per < perrec ? per : perrec;
        for (d = 0; d < 4; ++d) {
            int s = gr.mode == GR_PLAY ? gr.map[d] : d;
            memcpy((unsigned char *) out + d * per, fr->pad + s * perrec, (size_t) n);
        }
    }
    return 1;
}

static void gr_parse_map(const char *spec) {
    int i;
    for (i = 0; i < 4; ++i) gr.map[i] = i;
    /* "1:2,2:1": the recorded port 1 plays on port 2, recorded port 2 on port 1 */
    while (spec != NULL && *spec) {
        int a, b, used = 0;
        if (sscanf(spec, "%d:%d%n", &a, &b, &used) >= 2 && used > 0 && a >= 1 && a <= 4 && b >= 1 && b <= 4) {
            gr.map[b - 1] = a - 1;
            spec += used;
            while (*spec == ',' || *spec == ' ') ++spec;
        } else break;
    }
}

/* ---- jobs: trim (replay to `from` and save a state there) and save-history ---------------- */
static int gr_busy_rw(void) { return gs.rw_pending_on || gs.rw_resim > 0 || gs.rw_resim_run > 0; }

static void gr_job_end(const char *msg, int ok) {
    gr_say(ok ? GS_GREEN : GS_RED, "%s", msg);
    gr.mode = GR_IDLE;
    gr.jk = 0;
    gr_free_file(&gr.pf);
}

/* writes <jdst>.gdlp from gr.pf frames [from, to) (a trim) after the .gdst exists */
static void gr_write_trim(void) {
    long a, b;
    FILE *f;
    char path[MAX_PATH], sname[48];
    unsigned char ev[64];
    int i;
    gr_path(path, sizeof path, gr.jdst, "gdlp");
    snprintf(sname, sizeof sname, "%s.gdst", gr.jdst);
    f = gr_begin(path, gr.pf.fr[gr.jfrom].tag, sname, &a, &b);
    if (f == NULL) { gr_job_end("trim: cannot write the recording", 0); return; }
    memset(ev, 0, sizeof ev);
    gr_put32(ev, (unsigned) gr.jfrom); gr_put32(ev + 4, (unsigned) gr.jto);
    snprintf((char *) ev + 8, 40, "%s", gr.name);
    gr_ev(f, GR_EV_SEGMENT, ev);
    for (i = gr.jfrom; i < gr.jto; ++i) {
        unsigned char fe[GR_FRAME_PAYLOAD];
        unsigned char c = GR_EV_FRAME;
        gr_frame_bytes(fe, gr.pf.fr[i].tag, gr.pf.fr[i].have, gr.pf.fr[i].pad, gr.pf.fr[i].vp);
        fwrite(&c, 1, 1, f);
        fwrite(fe, 1, sizeof fe, f);
    }
    for (i = 0; i < gr.pf.nmarks; ++i) {
        if (gr.pf.marks[i].frame >= gr.jfrom && gr.pf.marks[i].frame < gr.jto) {
            memset(ev, 0, sizeof ev);
            gr_put32(ev, (unsigned) (gr.pf.marks[i].frame - gr.jfrom));
            snprintf((char *) ev + 4, 32, "%s", gr.pf.marks[i].label);
            gr_ev(f, GR_EV_MARK, ev);
        }
    }
    gr_finish(f, a, b);
    snprintf(gr.last, sizeof gr.last, "%s", gr.jdst);
    gs.paused = gr.jpaused; /* the cut leaves the game as it found it */
    {
        char m[160];
        snprintf(m, sizeof m, "recording \"%s\" saved: %d frames (%.1f s) cut from \"%s\" [%d,%d)", gr.jdst,
                 gr.jto - gr.jfrom, (gr.jto - gr.jfrom) / 60.0, gr.name, gr.jfrom, gr.jto);
        gr_job_end(m, 1);
    }
}

static void gr_write_history(void) {
    long a, b;
    FILE *f;
    char path[MAX_PATH], sname[48];
    int tag, bad = 0;
    gr_path(path, sizeof path, gr.jdst, "gdlp");
    snprintf(sname, sizeof sname, "%s.gdst", gr.jdst);
    f = gr_begin(path, gr.jF, sname, &a, &b);
    if (f == NULL) { gr_job_end("save-history: cannot write the recording", 0); return; }
    for (tag = gr.jF; tag < gr.jto; ++tag) {
        const GsLogEntry *e = &gs_log[(unsigned) tag % GS_LOG_N];
        unsigned char fe[GR_FRAME_PAYLOAD], c = GR_EV_FRAME;
        if (e->tag != tag) { bad = 1; break; }
        gr_frame_bytes(fe, tag, e->have, e->pad, e->virtual_pad);
        fwrite(&c, 1, 1, f);
        fwrite(fe, 1, sizeof fe, f);
    }
    gr_finish(f, a, b);
    if (bad) { gr_say(GS_RED, "save-history: the input log had a gap; \"%s\" is short", gr.jdst); }
    snprintf(gr.last, sizeof gr.last, "%s", gr.jdst);
    gr.jpaused = gr.jpaused ? 1 : 0;
    if (gr.jnow0 == gs_ring_now()) {
        gs.paused = gr.jpaused;
        gr_say(GS_GREEN, "recording \"%s\" saved from the history: %d frames (%.1f s)", gr.jdst, gr.jto - gr.jF, (gr.jto - gr.jF) / 60.0);
        gr.mode = GR_IDLE; gr.jk = 0;
    } else {
        const char *why = NULL;
        gr.jphase = 4;
        if (gs_rw_request(gr.jnow0, &why) != 0) {
            gs.paused = gr.jpaused;
            gr_say(GS_RED, "save-history: saved, but could not return to frame %d (%s)", gr.jnow0, why ? why : "?");
            gr.mode = GR_IDLE; gr.jk = 0;
        }
    }
}

/* at the top of gs_apply_pending (every boundary where pending operations may run) */
static void gr_tick(int at_tick) {
    if (gr.mode == GR_PLAY && gr.loop && gr.pf.n > 0 && gs.match_active && gs_ring_now() - gr.tag0 >= gr.pf.n &&
        !gs.st_pending_load[0]) {
        gr_path(gr.gdst, sizeof gr.gdst, gr.name, "gdst");
        snprintf(gs.st_pending_load, sizeof gs.st_pending_load, "%s", gr.gdst);
        gr.mode = GR_LOAD;
        return;
    }
    if (gr.mode != GR_JOB) return;
    if (++gr.jguard > 200000) { gs.paused = gr.jpaused; gr_job_end("the recorder job timed out", 0); return; }
    if (gr.jk == 1) { /* trim */
        if (gr.jphase == 2 && at_tick && gs.step == 0 && gs_ring_now() - gr.tag0 == gr.jfrom) {
            gr_path(gr.gdst, sizeof gr.gdst, gr.jdst, "gdst");
            snprintf(gs.st_pending_save, sizeof gs.st_pending_save, "%s", gr.gdst);
            snprintf(gs.st_save_name, sizeof gs.st_save_name, "%s", gr.jdst);
            snprintf(gs.st_save_desc, sizeof gs.st_save_desc, "trim of %s", gr.name);
            gr.jphase = 3;
        }
    } else if (gr.jk == 2 && at_tick) { /* save-history */
        if (gr.jphase == 1 && !gr_busy_rw() && gs_ring_now() == gr.jF) {
            gr_path(gr.gdst, sizeof gr.gdst, gr.jdst, "gdst");
            snprintf(gs.st_pending_save, sizeof gs.st_pending_save, "%s", gr.gdst);
            snprintf(gs.st_save_name, sizeof gs.st_save_name, "%s", gr.jdst);
            snprintf(gs.st_save_desc, sizeof gs.st_save_desc, "history");
            gr.jphase = 3;
        } else if (gr.jphase == 4 && !gr_busy_rw() && gs_ring_now() == gr.jnow0) {
            gs.paused = gr.jpaused;
            gr_say(GS_GREEN, "recording \"%s\" saved from the history: %d frames (%.1f s)", gr.jdst, gr.jto - gr.jF, (gr.jto - gr.jF) / 60.0);
            gr.mode = GR_IDLE; gr.jk = 0;
        }
    }
}

/* a state file was just written */
static void gr_after_save(void) {
    if (gr.mode == GR_ARM) {
        char sname[48];
        snprintf(sname, sizeof sname, "%s.gdst", gr.name);
        gr.f = gr_begin(gr.gdlp, gs_ring_now(), sname, &gr.raw_len_at, &gr.raw_start);
        if (gr.f == NULL) { gr.mode = GR_IDLE; gr_say(GS_RED, "recording: cannot write %s", gr.gdlp); return; }
        gr.base = gs_ring_now();
        gr.count = 0;
        if (gr.offcap == 0) {
            gr.off = (long *) malloc(4096 * sizeof(long));
            gr.offcap = gr.off ? 4096 : 0;
        }
        if (gr.off == NULL) { fclose(gr.f); gr.f = NULL; gr.mode = GR_IDLE; return; }
        gr.off[0] = ftell(gr.f);
        gr.mode = GR_REC;
        gr_say(GS_GREEN, "recording \"%s\" from frame %d", gr.name, gr.base);
    } else if (gr.mode == GR_JOB && gr.jphase == 3) {
        if (gr.jk == 1) gr_write_trim();
        else if (gr.jk == 2) gr_write_history();
    }
}
static void gr_save_failed(void) {
    if (gr.mode == GR_ARM) { gr.mode = GR_IDLE; }
    else if (gr.mode == GR_JOB) { gs.paused = gr.jpaused; gr_job_end("the state could not be saved", 0); }
}
/* a state file load that gr started finished */
static void gr_after_load(void) {
    if (gr.mode == GR_LOAD) {
        gr.tag0 = gs_ring_now();
        gr.mode = GR_PLAY;
        gr_say(GS_GREEN, "playing \"%s\" (%d frames, %.1f s)%s", gr.name, gr.pf.n, gr.pf.n / 60.0, gr.loop ? ", looping" : "");
    } else if (gr.mode == GR_JOB && gr.jk == 1 && gr.jphase == 1) {
        gr.tag0 = gs_ring_now();
        gr.jphase = 2;
        gs.paused = 1;
        gs.step = gr.jfrom;
    }
}
static void gr_load_failed(void) {
    if (gr.mode == GR_LOAD) { gr.mode = GR_IDLE; gr_free_file(&gr.pf); }
    else if (gr.mode == GR_JOB) { gs.paused = gr.jpaused; gr_job_end("the state could not be loaded", 0); }
}
/* some other state load / scene change: whatever we were doing no longer matches the timeline */
static void gr_external_load(void) {
    if (gr.mode == GR_REC) gr_stop_rec("a state was loaded");
    else if (gr.mode == GR_PLAY) gr_stop_play("a state was loaded");
}
static void gr_scene_end(void) {
    if (gr.mode == GR_REC) gr_stop_rec("the match ended");
    else if (gr.mode == GR_PLAY || gr.mode == GR_LOAD) { gr.mode = GR_IDLE; gr_free_file(&gr.pf); }
    else if (gr.mode == GR_JOB || gr.mode == GR_ARM) { gr.mode = GR_IDLE; gr.jk = 0; gr_free_file(&gr.pf); }
}

/* ---- gd.* ------------------------------------------------------------------------------- */
static int gr_fail(lua_State *L, const char *why) { return gs_push_fail(L, why); }
static int gr_gate(lua_State *L, const char *fn, const char **why) {
    gs_require_offline(L, fn);
    if (gs_1p_state_active()) { *why = "retail 1P state active"; return -1; }
    if (gw_ScriptGame_StageSlotsActive()) { *why = "stage slot session active"; return -1; }
    if (!gs.match_active) { *why = "not in a match"; return -1; }
    if (gs_rw_session()) { *why = "not during a netplay/rollback session"; return -1; }
    return 0;
}

/* gd.rec_start([name]) -> name | false, why: save the LAB state and write every frame's pads from here */
static int l_rec_start(lua_State *L) {
    const char *why = NULL;
    char name[48], path[MAX_PATH];
    if (gr_gate(L, "rec_start", &why) != 0) return gr_fail(L, why);
    if (gr.mode != GR_IDLE) return gr_fail(L, gr.mode == GR_REC ? "already recording" : "the recorder is busy");
    if (gs.st_pending_save[0]) return gr_fail(L, "a state save is waiting");
    if (lua_isnoneornil(L, 1)) snprintf(name, sizeof name, "rec_%lld", (long long) time(NULL));
    else snprintf(name, sizeof name, "%s", luaL_checkstring(L, 1));
    if (!gr_name_ok(name)) return gr_fail(L, "name: letters, digits, _ and - only (40 max)");
    gr_path(path, sizeof path, name, "gdlp");
    if (gr_exists(path)) return gr_fail(L, "a recording with that name exists (nothing is overwritten)");
    snprintf(gr.name, sizeof gr.name, "%s", name);
    snprintf(gr.gdlp, sizeof gr.gdlp, "%s", path);
    gr_path(gr.gdst, sizeof gr.gdst, name, "gdst");
    snprintf(gs.st_pending_save, sizeof gs.st_pending_save, "%s", gr.gdst);
    snprintf(gs.st_save_name, sizeof gs.st_save_name, "%s", name);
    snprintf(gs.st_save_desc, sizeof gs.st_save_desc, "recording");
    gr.mode = GR_ARM;
    lua_pushstring(L, name);
    return 1;
}

/* gd.rec_stop() -> true, frames | false, why */
static int l_rec_stop(lua_State *L) {
    int n;
    gs_require_offline(L, "rec_stop");
    if (gr.mode == GR_ARM) { gr.mode = GR_IDLE; gs.st_pending_save[0] = '\0'; lua_pushboolean(L, 1); lua_pushinteger(L, 0); return 2; }
    if (gr.mode != GR_REC) return gr_fail(L, "not recording");
    n = gr.count;
    gr_stop_rec("stopped");
    lua_pushboolean(L, 1);
    lua_pushinteger(L, n);
    return 2;
}

/* gd.rec_mark([label]) -> frame offset | false, why: a marker event in the file */
static int l_rec_mark(lua_State *L) {
    unsigned char ev[36];
    const char *label = luaL_optstring(L, 1, "");
    if (gr.mode != GR_REC || gr.f == NULL) return gr_fail(L, "not recording");
    memset(ev, 0, sizeof ev);
    gr_put32(ev, (unsigned) gr.count);
    snprintf((char *) ev + 4, 32, "%s", label);
    { unsigned char c = GR_EV_MARK; fseek(gr.f, gr.off[gr.count], SEEK_SET); fwrite(&c, 1, 1, gr.f); fwrite(ev, 1, sizeof ev, gr.f); gr.off[gr.count] = ftell(gr.f); }
    fflush(gr.f);
    gr_say(GS_GREEN, "mark at frame %d (%.1f s)%s%s", gr.count, gr.count / 60.0, label[0] ? ": " : "", label);
    lua_pushinteger(L, gr.count);
    return 1;
}

/* gd.rec_play(name [, map [, loop]]) -> true | false, why. map "1:2,2:1" sends recorded port a to port b. */
static int l_rec_play(lua_State *L) {
    const char *why = NULL, *name = luaL_checkstring(L, 1), *map = luaL_optstring(L, 2, "");
    int loop = lua_toboolean(L, 3);
    char path[MAX_PATH], w[200];
    GrFile g;
    GsStateHdr h;
    int len;
    if (gr_gate(L, "rec_play", &why) != 0) return gr_fail(L, why);
    if (gr.mode != GR_IDLE) return gr_fail(L, gr.mode == GR_REC ? "stop recording first" : "the recorder is busy");
    if (!gr_name_ok(name)) return gr_fail(L, "bad recording name");
    gr_path(path, sizeof path, name, "gdlp");
    if (gr_read(path, &g, w, sizeof w) != 0) return gr_fail(L, w);
    if (g.n == 0) { gr_free_file(&g); return gr_fail(L, "the recording has no frames"); }
    if (gr_check(&g, w, sizeof w) != 0) { gr_free_file(&g); return gr_fail(L, w); }
    {
        char dir[MAX_PATH];
        gr_dir(dir, sizeof dir);
        snprintf(gr.gdst, sizeof gr.gdst, "%s\\%s", dir, g.state[0] ? g.state : "none.gdst");
    }
    len = gw_snap_file_header(gr.gdst, &h, sizeof h);
    if (len < 0) { gr_free_file(&g); return gr_fail(L, "the recording's state file (.gdst) is missing"); }
    if (gs_state_check(&h, len, w, sizeof w) != 0) { gr_free_file(&g); return gr_fail(L, w); }
    gr_free_file(&gr.pf);
    gr.pf = g;
    snprintf(gr.name, sizeof gr.name, "%s", name);
    gr_parse_map(map);
    gr.loop = loop;
    snprintf(gs.st_pending_load, sizeof gs.st_pending_load, "%s", gr.gdst);
    gr.mode = GR_LOAD;
    lua_pushboolean(L, 1);
    return 1;
}

static int l_rec_play_stop(lua_State *L) {
    gs_require_offline(L, "rec_play_stop");
    if (gr.mode != GR_PLAY && gr.mode != GR_LOAD) return gr_fail(L, "not playing");
    gr_stop_play("stopped");
    lua_pushboolean(L, 1);
    return 1;
}

static void gr_job_begin(int kind, const char *dst) {
    gr.mode = GR_JOB; gr.jk = kind; gr.jphase = 1; gr.jguard = 0;
    gr.jpaused = gs.paused;
    snprintf(gr.jdst, sizeof gr.jdst, "%s", dst);
}

/* gd.rec_trim(name, from, to [, newname]) -> newname | false, why: a new recording of frames [from, to) */
static int l_rec_trim(lua_State *L) {
    const char *why = NULL, *name = luaL_checkstring(L, 1);
    int from = (int) luaL_checkinteger(L, 2), to = (int) luaL_checkinteger(L, 3);
    char dst[48], path[MAX_PATH], w[200];
    GrFile g;
    GsStateHdr h;
    int len;
    if (gr_gate(L, "rec_trim", &why) != 0) return gr_fail(L, why);
    if (gr.mode != GR_IDLE) return gr_fail(L, "the recorder is busy");
    if (!gr_name_ok(name)) return gr_fail(L, "bad recording name");
    if (lua_isnoneornil(L, 4)) snprintf(dst, sizeof dst, "%s-cut", name); else snprintf(dst, sizeof dst, "%s", luaL_checkstring(L, 4));
    if (!gr_name_ok(dst)) return gr_fail(L, "bad new name");
    gr_path(path, sizeof path, dst, "gdlp");
    if (gr_exists(path)) return gr_fail(L, "a recording with the new name exists");
    gr_path(path, sizeof path, name, "gdlp");
    if (gr_read(path, &g, w, sizeof w) != 0) return gr_fail(L, w);
    if (from < 0) from = 0;
    if (to > g.n) to = g.n;
    if (from >= to) { gr_free_file(&g); return gr_fail(L, "empty range (frames are 0 .. count)"); }
    if (gr_check(&g, w, sizeof w) != 0) { gr_free_file(&g); return gr_fail(L, w); }
    {
        char dir[MAX_PATH];
        gr_dir(dir, sizeof dir);
        snprintf(gr.gdst, sizeof gr.gdst, "%s\\%s", dir, g.state[0] ? g.state : "none.gdst");
    }
    len = gw_snap_file_header(gr.gdst, &h, sizeof h);
    if (len < 0) { gr_free_file(&g); return gr_fail(L, "the state file (.gdst) is missing"); }
    if (gs_state_check(&h, len, w, sizeof w) != 0) { gr_free_file(&g); return gr_fail(L, w); }
    gr_free_file(&gr.pf);
    gr.pf = g;
    snprintf(gr.name, sizeof gr.name, "%s", name);
    gr.jfrom = from; gr.jto = to;
    gr_job_begin(1, dst);
    gs.paused = 1; gs.step = 0;
    if (from == 0) { /* the start state is the source's own: copy it */
        char to_st[MAX_PATH];
        gr_path(to_st, sizeof to_st, dst, "gdst");
        if (!CopyFileA(gr.gdst, to_st, TRUE)) { gs.paused = gr.jpaused; gr_job_end("trim: cannot copy the state file", 0); return gr_fail(L, "cannot copy the state file"); }
        gr.tag0 = gr.pf.fr[0].tag;
        gr.jphase = 3;
        gr_write_trim();
        gs.paused = gr.jpaused;
    } else {
        snprintf(gs.st_pending_load, sizeof gs.st_pending_load, "%s", gr.gdst);
    }
    lua_pushstring(L, dst);
    return 1;
}

/* gd.rec_keep_last(seconds [, name]) -> newname | false, why: the final N seconds of a recording */
static int l_rec_keep_last(lua_State *L) {
    double sec = luaL_checknumber(L, 1);
    const char *name = luaL_optstring(L, 2, gr.last);
    char path[MAX_PATH], w[200];
    GrFile g;
    int n, from;
    if (name == NULL || !name[0]) return gr_fail(L, "no recording named (and none made this session)");
    gr_path(path, sizeof path, name, "gdlp");
    if (gr_read(path, &g, w, sizeof w) != 0) return gr_fail(L, w);
    n = g.n;
    gr_free_file(&g);
    from = n - (int) (sec * 60.0 + 0.5);
    if (from < 0) from = 0;
    lua_settop(L, 0);
    lua_pushstring(L, name);
    lua_pushinteger(L, from);
    lua_pushinteger(L, n);
    {
        char dst[48];
        snprintf(dst, sizeof dst, "%s-last%d", name, (int) sec);
        lua_pushstring(L, dst);
    }
    return l_rec_trim(L);
}

/* gd.rec_save_history(name [, seconds]) -> name | false, why: the rewind log (last N s) + its keyframe, saved as a recording */
static int l_rec_save_history(lua_State *L) {
    const char *why = NULL, *name = luaL_checkstring(L, 1);
    double sec = luaL_optnumber(L, 2, 3600.0);
    char path[MAX_PATH];
    int head, F, lo;
    if (gr_gate(L, "rec_save_history", &why) != 0) return gr_fail(L, why);
    if (gr.mode != GR_IDLE) return gr_fail(L, "the recorder is busy");
    if (!gr_name_ok(name)) return gr_fail(L, "bad recording name");
    gr_path(path, sizeof path, name, "gdlp");
    if (gr_exists(path)) return gr_fail(L, "a recording with that name exists");
    if (gs.rw_frames <= 0 || !gs.rw_began) return gr_fail(L, "history is off (gd.history(n) turns it on)");
    if (gs.st_pending_save[0] || gs.st_pending_load[0]) return gr_fail(L, "a state operation is waiting");
    head = gs.rw_head;
    lo = gw_rw_base_tag() > gs.rw_log_start ? gw_rw_base_tag() : gs.rw_log_start;
    F = head - (int) (sec * 60.0 + 0.5);
    if (F < lo) F = lo;
    if (head - F < 2) return gr_fail(L, "the history holds fewer than 2 frames");
    gr_job_begin(2, name);
    gr.jF = F; gr.jto = head; gr.jnow0 = gs_ring_now();
    if (gs_ring_now() != F) {
        if (gs_rw_request(F, &why) != 0) { gr.mode = GR_IDLE; gr.jk = 0; return gr_fail(L, why); }
    } else {
        gr.jpaused = gs.paused;
        gs.paused = 1; /* the save runs at the loop top */
    }
    lua_pushstring(L, name);
    return 1;
}

static int l_rec_status(lua_State *L) {
    static const char *const names[] = { "idle", "arming", "recording", "loading", "playing", "working" };
    lua_createtable(L, 0, 10);
    gs_setstr(L, "mode", names[gr.mode]);
    gs_setstr(L, "name", gr.name);
    gs_setstr(L, "last", gr.last);
    gs_setint(L, "frames", gr.mode == GR_REC ? gr.count : gr.pf.n);
    gs_setint(L, "frame", gr.mode == GR_PLAY ? gs_ring_now() - gr.tag0 : gr.mode == GR_REC ? gr.count : 0);
    gs_setbool(L, "loop", gr.loop);
    gs_setint(L, "base", gr.base);
    return 1;
}

/* gd.rec_list() -> {{name, frames, seconds, marks, complete, flags, retail, bytes, ok, why}, ...} */
static int l_rec_list(lua_State *L) {
    char dir[MAX_PATH], pat[MAX_PATH], path[MAX_PATH], w[200];
    WIN32_FIND_DATAA fd;
    HANDLE hf;
    int n = 0;
    gr_dir(dir, sizeof dir);
    snprintf(pat, sizeof pat, "%s\\*.gdlp", dir);
    lua_newtable(L);
    hf = FindFirstFileA(pat, &fd);
    if (hf == INVALID_HANDLE_VALUE) return 1;
    do {
        GrFile g;
        char nm[64];
        snprintf(nm, sizeof nm, "%s", fd.cFileName);
        nm[strlen(nm) - 5] = '\0';
        snprintf(path, sizeof path, "%s\\%s", dir, fd.cFileName);
        lua_createtable(L, 0, 10);
        gs_setstr(L, "name", nm);
        gs_setint(L, "bytes", (lua_Integer) fd.nFileSizeLow);
        if (gr_read(path, &g, w, sizeof w) == 0) {
            gs_setint(L, "frames", g.n);
            gs_setnum(L, "seconds", g.n / 60.0);
            gs_setint(L, "marks", g.nmarks);
            gs_setbool(L, "complete", g.complete);
            gs_setbool(L, "retail", (g.flags & GR_FL_RETAIL) != 0);
            gs_setstr(L, "fighters", g.fi[0].name);
            gs_setbool(L, "ok", gr_check(&g, w, sizeof w) == 0);
            gs_setstr(L, "why", w);
            gr_free_file(&g);
        } else {
            gs_setbool(L, "ok", 0);
            gs_setstr(L, "why", w);
        }
        lua_rawseti(L, -2, ++n);
    } while (FindNextFileA(hf, &fd));
    FindClose(hf);
    return 1;
}
