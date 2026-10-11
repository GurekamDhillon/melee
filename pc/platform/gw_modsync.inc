/* gw_modsync.inc - "match the host's mods" (the contract is gw_modsync.h). Included at the end of gw_mods.c, so it uses that
 * file's statics (the mod table, the parser, the test scratch folder). It is not a translation unit of its own.
 *
 * WIRE. The stream is one header record "G <global hex16> <count>" followed by one "E id|name|kind|source|post|digest hex16|ident"
 * record per MOUNTED mod, records joined by '\n'. The stream is cut into slices of at most `cap` bytes (greedy); chunk k is
 * "MSM <k> <N> " followed by slice k. A slice can end inside a record (a 60-byte cap cannot hold a whole record), so the
 * receiver keeps each slice by its k: arrival order does not matter, and when all N are here the slices are joined in k order
 * and parsed as one stream. A chunk whose k is already held is a repeat and is ignored. A chunk 0 that arrives while chunk 0
 * is held, or a chunk whose N differs, starts a new stream. */

#define MS_CHUNK_MAX 200 /* GW_NET_LOBBY_MAX: the largest chunk the lobby carries */
#define MS_MAX_CHUNKS 256
#define MS_STREAM_MAX 16384 /* GW_MS_MAX records of at most 167 bytes, plus the header */
#define MS_JDIR ".modsync"
#define MS_LAUNCH_WINDOW 180 /* seconds: a relaunch older than this is abandoned (restore) */
#define MS_ENABLED_HEADER "# Mods enabled at the next boot, one folder name per line. Written by the game's\n" \
                          "# mods menu; hand edits are fine. Delete this file to enable every mod.\n"

/* ---- state ------------------------------------------------------------------------------------------------------ */

static gw_ms_manifest ms_mine;   /* our snapshot at peer_reset: every installed mod (the wire sends the mounted ones) */
static uint64_t ms_global;       /* the global hash gw_ms_set_global gave us */
static int ms_send_next;         /* the next chunk to send */
static gw_ms_manifest ms_peer;   /* the peer's list, valid while ms_rx_ready */
static gw_ms_manifest ms_scratch; /* decode scratch */
static char ms_stream_buf[MS_STREAM_MAX];       /* encode scratch */
static char ms_rx_stream[MS_MAX_CHUNKS * MS_CHUNK_MAX];
static unsigned char ms_rx_chunk[MS_MAX_CHUNKS][MS_CHUNK_MAX];
static int ms_rx_len[MS_MAX_CHUNKS];
static unsigned char ms_rx_have[MS_MAX_CHUNKS];
static int ms_rx_n, ms_rx_got, ms_rx_bad, ms_rx_ready;
static gw_ms_row ms_rows[GW_MS_MAX * 2];
static char ms_idbuf[GW_MS_MAX * 3][GW_MS_ID];

/* ---- small helpers ----------------------------------------------------------------------------------------------- */

/* Copy src into dst, dropping control characters and '|' (the wire separator), truncating to cap-1. */
static void ms_copy_clean(char *dst, size_t cap, const char *src) {
    size_t n = 0;
    if (cap == 0) return;
    for (; src != NULL && *src != '\0' && n + 1 < cap; ++src) {
        unsigned char c = (unsigned char) *src;
        if (c < 32 || c == 127 || c == '|') continue;
        dst[n++] = (char) c;
    }
    dst[n] = '\0';
}

static int ms_clamp_n(int n) { return n < 0 ? 0 : n > GW_MS_MAX ? GW_MS_MAX : n; }

/* Unsigned decimal at *pp (at most maxd digits, before end). */
static int ms_dec(const char **pp, const char *end, long long *out, int maxd) {
    const char *p = *pp;
    long long v = 0;
    int nd = 0;
    while (p < end && *p >= '0' && *p <= '9') {
        if (++nd > maxd) return 0;
        v = v * 10 + (*p - '0');
        ++p;
    }
    if (nd == 0) return 0;
    *pp = p;
    *out = v;
    return 1;
}

/* A whole NUL-terminated string that is a decimal of 1..maxd digits. */
static int ms_dec_str(const char *s, int maxd, long long *out) {
    const char *end = s + strlen(s);
    return ms_dec(&s, end, out, maxd) && s == end;
}

/* Exactly 16 hex digits at s (before end). */
static int ms_hex16(const char *s, const char *end, uint64_t *out) {
    int i;
    uint64_t v = 0;
    if (end - s < 16) return 0;
    for (i = 0; i < 16; ++i) {
        char c = s[i];
        int d;
        if (c >= '0' && c <= '9') d = c - '0';
        else if (c >= 'a' && c <= 'f') d = c - 'a' + 10;
        else if (c >= 'A' && c <= 'F') d = c - 'A' + 10;
        else return 0;
        v = (v << 4) | (uint64_t) d;
    }
    *out = v;
    return 1;
}

static int64_t ms_atoi64(const char *s) {
    int64_t v = 0;
    int neg = 0;
    if (*s == '-') {
        neg = 1;
        ++s;
    }
    while (*s >= '0' && *s <= '9') {
        v = v * 10 + (*s - '0');
        ++s;
    }
    return neg ? -v : v;
}

/* ---- the manifest of this install ------------------------------------------------------------------------------- */

static void ms_fill(gw_ms_entry *e, int i, int mounted) {
    memset(e, 0, sizeof *e);
    ms_copy_clean(e->id, sizeof e->id, gw_Mods_Id(i));
    ms_copy_clean(e->name, sizeof e->name, gw_Mods_Name(i));
    ms_copy_clean(e->kind, sizeof e->kind, gw_Mods_Kind(i));
    ms_copy_clean(e->source, sizeof e->source, gw_Mods_Source(i));
    e->post = gw_Mods_NucleusPost(i);
    e->digest = mounted ? gw_Mods_Digest(i) : 0;
    e->ident = strcmp(e->kind, "skin") != 0;
    e->enabled = gw_Mods_IsEnabled(i) != 0;
    e->mounted = mounted != 0;
}

void gw_ms_build_local(gw_ms_manifest *out, uint64_t global) {
    int pass, i, cnt;
    if (out == NULL) return;
    memset(out, 0, sizeof *out);
    out->global = global;
    cnt = gw_Mods_Count();
    /* mounted mods first, so the GW_MS_MAX cap drops installed-but-off mods, never the ones that play */
    for (pass = 0; pass < 2; ++pass) {
        for (i = 0; i < cnt && out->n < GW_MS_MAX; ++i) {
            int mounted = gw_Mods_IsActive(i) != 0;
            if ((pass == 0) != mounted) continue;
            ms_fill(&out->e[out->n++], i, mounted);
        }
    }
}

/* ---- the wire: encode ------------------------------------------------------------------------------------------- */

/* The stream for `m` (mounted entries only) into buf. Returns its length, -1 when it does not fit. */
static int ms_stream(const gw_ms_manifest *m, char *buf, int cap) {
    int i, n = ms_clamp_n(m->n), len, cnt = 0;
    for (i = 0; i < n; ++i) cnt += m->e[i].mounted != 0;
    len = snprintf(buf, (size_t) cap, "G %016llx %d", (unsigned long long) m->global, cnt);
    if (len < 0 || len >= cap) return -1;
    for (i = 0; i < n; ++i) {
        const gw_ms_entry *e = &m->e[i];
        char id[GW_MS_ID], name[GW_MS_NAME], kind[12], src[12];
        int w;
        if (!e->mounted) continue;
        ms_copy_clean(id, sizeof id, e->id);
        ms_copy_clean(name, sizeof name, e->name);
        ms_copy_clean(kind, sizeof kind, e->kind);
        ms_copy_clean(src, sizeof src, e->source);
        w = snprintf(buf + len, (size_t) (cap - len), "\nE %s|%s|%s|%s|%d|%016llx|%d", id, name, kind, src,
                     e->post < 0 ? 0 : e->post, (unsigned long long) e->digest, e->ident ? 1 : 0);
        if (w < 0 || w >= cap - len) return -1;
        len += w;
    }
    return len;
}

static int ms_prefix_len(int k, int N) {
    char t[32];
    return snprintf(t, sizeof t, "MSM %d %d ", k, N);
}

/* How many chunks a stream of `len` bytes takes when every chunk is announced as one of N. -1 when a chunk has no room. */
static int ms_pack_count(int len, int cap, int N) {
    int pos = 0, k = 0;
    while (pos < len) {
        int room = cap - ms_prefix_len(k, N), left = len - pos;
        if (room <= 0) return -1;
        pos += room < left ? room : left;
        ++k;
    }
    return k;
}

/* N such that packing announces exactly N chunks. The count never falls as N grows, so this settles upward. */
static int ms_total_chunks(int len, int cap) {
    int N = 1, c;
    for (;;) {
        c = ms_pack_count(len, cap, N);
        if (c <= 0 || c > MS_MAX_CHUNKS) return -1;
        if (c == N) return N;
        N = c;
    }
}

int gw_ms_wire_encode(const gw_ms_manifest *m, int index, uint8_t *out, int cap) {
    int len, N, k, pos = 0, off = 0, take = 0, pl;
    if (m == NULL || out == NULL || index < 0 || cap <= 0 || cap > MS_CHUNK_MAX) return 0;
    len = ms_stream(m, ms_stream_buf, (int) sizeof ms_stream_buf);
    if (len <= 0) return 0;
    N = ms_total_chunks(len, cap);
    if (N <= 0 || index >= N) return 0;
    for (k = 0; k <= index; ++k) {
        int room = cap - ms_prefix_len(k, N);
        take = room < len - pos ? room : len - pos;
        if (k == index) off = pos;
        pos += take;
    }
    pl = ms_prefix_len(index, N);
    snprintf((char *) out, (size_t) cap, "MSM %d %d ", index, N);
    memcpy(out + pl, ms_stream_buf + off, (size_t) take);
    return pl + take;
}

/* ---- the wire: decode ------------------------------------------------------------------------------------------- */

/* "MSM k N " at msg: k < N <= MS_MAX_CHUNKS, 1..MS_CHUNK_MAX payload bytes. Sets *pl to the payload's offset. */
static int ms_prefix(const uint8_t *msg, int len, int *k, int *N, int *pl) {
    const char *s = (const char *) msg, *end = s + len, *p = s + 4;
    long long kv = 0, nv = 0;
    if (!ms_dec(&p, end, &kv, 3) || p >= end || *p != ' ') return 0;
    ++p;
    if (!ms_dec(&p, end, &nv, 3) || p >= end || *p != ' ') return 0;
    ++p;
    if (nv < 1 || nv > MS_MAX_CHUNKS || kv >= nv) return 0;
    if (end - p < 1 || end - p > MS_CHUNK_MAX) return 0;
    *k = (int) kv;
    *N = (int) nv;
    *pl = (int) (p - s);
    return 1;
}

/* One "E ..." record (NUL-terminated) into e. */
static int ms_entry(const char *rec, gw_ms_entry *e) {
    char buf[256];
    char *f[7];
    size_t len = strlen(rec), i, n = 0;
    long long post = 0;
    uint64_t dg = 0;
    if (len < 3 || len - 2 >= sizeof buf || rec[0] != 'E' || rec[1] != ' ') return 0;
    len -= 2;
    memcpy(buf, rec + 2, len);
    buf[len] = '\0';
    f[n++] = buf;
    for (i = 0; i < len; ++i) {
        if (buf[i] != '|') continue;
        buf[i] = '\0';
        if (n == 7) return 0;
        f[n++] = buf + i + 1;
    }
    if (n != 7) return 0;
    if (f[0][0] == '\0' || strlen(f[0]) >= GW_MS_ID) return 0;
    if (f[1][0] == '\0' || strlen(f[1]) >= GW_MS_NAME) return 0;
    if (f[2][0] == '\0' || strlen(f[2]) >= sizeof e->kind) return 0;
    if (strlen(f[3]) >= sizeof e->source) return 0;
    if (!ms_dec_str(f[4], 9, &post)) return 0;
    if (strlen(f[5]) != 16 || !ms_hex16(f[5], f[5] + 16, &dg)) return 0;
    if (strcmp(f[6], "0") != 0 && strcmp(f[6], "1") != 0) return 0;
    memset(e, 0, sizeof *e);
    ms_copy_clean(e->id, sizeof e->id, f[0]);
    ms_copy_clean(e->name, sizeof e->name, f[1]);
    ms_copy_clean(e->kind, sizeof e->kind, f[2]);
    ms_copy_clean(e->source, sizeof e->source, f[3]);
    e->post = (int) post;
    e->digest = dg;
    e->ident = f[6][0] == '1';
    e->enabled = 1;
    e->mounted = 1;
    return 1;
}

/* A whole stream into out. 0 = malformed (out is then unusable). */
static int ms_decode(const char *s, int len, gw_ms_manifest *out) {
    const char *p = s, *end = s + len;
    long long cnt = 0;
    uint64_t g = 0;
    char rec[256];
    int n;
    memset(out, 0, sizeof *out);
    if (len < 20 || s[0] != 'G' || s[1] != ' ') return 0;
    if (!ms_hex16(s + 2, end, &g) || s[18] != ' ') return 0;
    p = s + 19;
    if (!ms_dec(&p, end, &cnt, 3) || cnt > GW_MS_MAX) return 0;
    for (n = 0; n < cnt; ++n) {
        const char *nl;
        size_t rl;
        if (p >= end || *p != '\n') return 0;
        ++p;
        nl = memchr(p, '\n', (size_t) (end - p));
        if (nl == NULL) nl = end;
        rl = (size_t) (nl - p);
        if (rl == 0 || rl >= sizeof rec) return 0;
        memcpy(rec, p, rl);
        rec[rl] = '\0';
        if (!ms_entry(rec, &out->e[n])) return 0;
        p = nl;
    }
    if (p != end) return 0;
    {
        int i, j;
        for (i = 0; i < n; ++i) {
            for (j = 0; j < i; ++j) {
                if (_stricmp(out->e[i].id, out->e[j].id) == 0) return 0;
            }
        }
    }
    out->global = g;
    out->n = n;
    return 1;
}

/* ---- the wire: the receiver (process-wide) ---------------------------------------------------------------------- */

static void ms_rx_clear(void) {
    memset(ms_rx_have, 0, sizeof ms_rx_have);
    ms_rx_n = ms_rx_got = ms_rx_bad = ms_rx_ready = 0;
}

static void ms_rx_finish(void) {
    int k, pos = 0;
    for (k = 0; k < ms_rx_n; ++k) {
        memcpy(ms_rx_stream + pos, ms_rx_chunk[k], (size_t) ms_rx_len[k]);
        pos += ms_rx_len[k];
    }
    if (!ms_decode(ms_rx_stream, pos, &ms_scratch)) {
        ms_rx_bad = 1;
        gw_log("modsync: the peer's mod list is malformed - ignored");
        return;
    }
    ms_peer = ms_scratch;
    ms_rx_ready = 1;
    gw_log("modsync: the peer's %d mod(s) are here", ms_peer.n);
}

void gw_ms_wire_reset(gw_ms_manifest *peer_state) {
    if (peer_state != NULL && peer_state != &ms_peer) memset(peer_state, 0, sizeof *peer_state);
    memset(&ms_peer, 0, sizeof ms_peer);
    ms_rx_clear();
}

int gw_ms_wire_feed(const uint8_t *msg, int len) {
    int k, N, pl;
    if (msg == NULL || len < 4 || memcmp(msg, "MSM ", 4) != 0) return 0;
    if (!ms_prefix(msg, len, &k, &N, &pl)) {
        gw_log("modsync: malformed chunk ignored");
        return 1;
    }
    if (ms_rx_n != 0 && (N != ms_rx_n || (k == 0 && ms_rx_have[0]))) ms_rx_clear(); /* a new stream */
    if (ms_rx_n == 0) ms_rx_n = N;
    if (ms_rx_bad || ms_rx_have[k]) return 1;
    ms_rx_len[k] = len - pl;
    memcpy(ms_rx_chunk[k], msg + pl, (size_t) (len - pl));
    ms_rx_have[k] = 1;
    if (++ms_rx_got == ms_rx_n) ms_rx_finish();
    return 1;
}

int gw_ms_peer_ready(void) { return ms_rx_ready; }
const gw_ms_manifest *gw_ms_peer(void) { return &ms_peer; }
const gw_ms_manifest *gw_ms_mine(void) { return &ms_mine; }

void gw_ms_set_global(uint64_t global) {
    ms_global = global;
    ms_mine.global = global;
}

/* ---- the wire: process-wide sending ----------------------------------------------------------------------------- */

void gw_ms_peer_reset(void) {
    gw_ms_build_local(&ms_mine, ms_global);
    ms_send_next = 0;
    gw_ms_wire_reset(NULL);
}

int gw_ms_wire_next(uint8_t *out, int cap) { return gw_ms_wire_encode(&ms_mine, ms_send_next, out, cap); }
void gw_ms_wire_sent(void) { ms_send_next++; }

/* ---- diff ------------------------------------------------------------------------------------------------------- */

static const gw_ms_entry *ms_find(const gw_ms_manifest *m, const char *id, int mounted_only) {
    int i, n = ms_clamp_n(m->n);
    for (i = 0; i < n; ++i) {
        const gw_ms_entry *e = &m->e[i];
        if (mounted_only && !e->mounted) continue;
        if (_stricmp(e->id, id) == 0) return e;
    }
    return NULL;
}

static int ms_row_cmp(const void *a, const void *b) {
    return _stricmp(((const gw_ms_row *) a)->id, ((const gw_ms_row *) b)->id);
}

static void ms_row_put(int *n, int state, const char *id, const char *name, int ident, const char *source, int post) {
    gw_ms_row *r;
    if (*n >= GW_MS_MAX * 2) return;
    r = &ms_rows[*n];
    memset(r, 0, sizeof *r);
    ms_copy_clean(r->id, sizeof r->id, id);
    ms_copy_clean(r->name, sizeof r->name, name);
    r->state = state;
    r->ident = ident != 0;
    ms_copy_clean(r->source, sizeof r->source, source);
    r->post = post;
    ++*n;
}

int gw_ms_diff(const gw_ms_manifest *mine, const gw_ms_manifest *theirs, gw_ms_row *rows, int cap) {
    int n = 0, i, out, mn, tn;
    if (mine == NULL || theirs == NULL) return 0;
    mn = ms_clamp_n(mine->n);
    tn = ms_clamp_n(theirs->n);
    for (i = 0; i < mn; ++i) {
        const gw_ms_entry *m = &mine->e[i], *t;
        if (!m->mounted) continue;
        t = ms_find(theirs, m->id, 0);
        if (t == NULL) {
            ms_row_put(&n, GW_MS_MINE_ONLY, m->id, m->name, m->ident, m->source, m->post);
        } else if (t->digest != m->digest) {
            ms_row_put(&n, GW_MS_DIFFERS, m->id, m->name, m->ident || t->ident, t->source, t->post);
        }
    }
    for (i = 0; i < tn; ++i) {
        const gw_ms_entry *t = &theirs->e[i];
        if (ms_find(mine, t->id, 1) == NULL) ms_row_put(&n, GW_MS_THEIRS_ONLY, t->id, t->name, t->ident, t->source, t->post);
    }
    qsort(ms_rows, (size_t) n, sizeof ms_rows[0], ms_row_cmp);
    for (out = 0; out < n && out < cap && rows != NULL; ++out) rows[out] = ms_rows[out];
    return n;
}

int gw_ms_ident_diffs(const gw_ms_row *rows, int n) {
    int i, c = 0;
    if (rows == NULL) return 0;
    for (i = 0; i < n; ++i) c += rows[i].ident != 0;
    return c;
}

/* ---- plan ------------------------------------------------------------------------------------------------------- */

static void ms_act(gw_ms_plan *p, int kind, const char *id, const char *name, int post, const char *why) {
    gw_ms_action *a;
    if (p->n >= GW_MS_MAX * 2) return;
    a = &p->a[p->n++];
    memset(a, 0, sizeof *a);
    a->kind = kind;
    copy_str(a->id, sizeof a->id, id);
    copy_str(a->name, sizeof a->name, name);
    a->post = post;
    copy_str(a->why, sizeof a->why, why);
    switch (kind) {
    case GW_MS_ACT_ENABLE: p->nenable++; break;
    case GW_MS_ACT_DISABLE: p->ndisable++; break;
    case GW_MS_ACT_DOWNLOAD: p->ndownload++; break;
    default: p->nmissing++; break;
    }
}

/* The peer mod `t` must be installed here and is not: fetch it from Nucleus when it can be, else it cannot be had. */
static void ms_want(gw_ms_plan *p, const gw_ms_entry *t) {
    char why[112];
    int nuc = strcmp(t->source, "nucleus") == 0;
    if (nuc && t->post > 0) {
        snprintf(why, sizeof why, "download from SSBM Nucleus: %s", t->name);
        ms_act(p, GW_MS_ACT_DOWNLOAD, t->id, t->name, t->post, why);
    } else {
        snprintf(why, sizeof why, "you need this mod: %s (%s)", t->name, nuc ? "from SSBM Nucleus" : !strcmp(t->source, "local file") ? "local file" : "source unknown");
        ms_act(p, GW_MS_ACT_MISSING, t->id, t->name, t->post, why);
    }
}

void gw_ms_plan_build(const gw_ms_manifest *mine, const gw_ms_manifest *theirs, gw_ms_plan *out) {
    int i, tn, mn;
    if (out == NULL) return;
    memset(out, 0, sizeof *out);
    if (mine == NULL || theirs == NULL) {
        out->can_complete = 0;
        snprintf(out->summary, sizeof out->summary, "No mod list from the peer");
        return;
    }
    tn = ms_clamp_n(theirs->n);
    mn = ms_clamp_n(mine->n);
    for (i = 0; i < tn; ++i) {
        const gw_ms_entry *t = &theirs->e[i], *l;
        char why[112];
        if (!t->ident) continue; /* costume skins are left as they are */
        l = ms_find(mine, t->id, 0);
        if (l == NULL) {
            ms_want(out, t);
            continue;
        }
        if (!l->enabled) {
            snprintf(why, sizeof why, "turn on: %s", l->name);
            ms_act(out, GW_MS_ACT_ENABLE, l->id, l->name, 0, why);
            continue;
        }
        if (!l->mounted) continue; /* enabled, restart pending: checked again after the relaunch */
        if (l->digest == t->digest) continue;
        ms_want(out, t);
    }
    for (i = 0; i < mn; ++i) {
        const gw_ms_entry *m = &mine->e[i];
        char why[112];
        if (!m->mounted || !m->ident) continue;
        if (ms_find(theirs, m->id, 0) != NULL) continue;
        snprintf(why, sizeof why, "turn off: %s (the peer does not have it)", m->name);
        ms_act(out, GW_MS_ACT_DISABLE, m->id, m->name, 0, why);
    }
    out->can_complete = out->nmissing == 0;
    if (out->nmissing > 0) {
        snprintf(out->summary, sizeof out->summary, "Cannot match: %d mod%s %s not on Nucleus", out->nmissing,
                 out->nmissing == 1 ? "" : "s", out->nmissing == 1 ? "is" : "are");
    } else {
        snprintf(out->summary, sizeof out->summary, "Download %d, turn on %d, turn off %d", out->ndownload,
                 out->nenable, out->ndisable);
    }
}

static void ms_id_add(int *n, const char *id) {
    int i;
    for (i = 0; i < *n; ++i) {
        if (_stricmp(ms_idbuf[i], id) == 0) return;
    }
    if (*n >= GW_MS_MAX * 3) return;
    ms_copy_clean(ms_idbuf[*n], GW_MS_ID, id);
    ++*n;
}

static int ms_id_cmp(const void *a, const void *b) { return _stricmp((const char *) a, (const char *) b); }

int gw_ms_target_ids(const gw_ms_manifest *mine, const gw_ms_plan *plan, char ids[][GW_MS_ID], int cap) {
    int n = 0, w = 0, i, j, mn;
    if (mine == NULL || plan == NULL) return 0;
    mn = ms_clamp_n(mine->n);
    for (i = 0; i < mn; ++i) {
        if (mine->e[i].enabled) ms_id_add(&n, mine->e[i].id);
    }
    for (i = 0; i < plan->n; ++i) {
        if (plan->a[i].kind == GW_MS_ACT_ENABLE || plan->a[i].kind == GW_MS_ACT_DOWNLOAD) ms_id_add(&n, plan->a[i].id);
    }
    for (i = 0; i < n; ++i) { /* drop what the plan turns off */
        int off = 0;
        for (j = 0; j < plan->n; ++j) {
            if (plan->a[j].kind == GW_MS_ACT_DISABLE && _stricmp(plan->a[j].id, ms_idbuf[i]) == 0) off = 1;
        }
        if (!off) {
            if (w != i) memcpy(ms_idbuf[w], ms_idbuf[i], sizeof ms_idbuf[0]);
            ++w;
        }
    }
    qsort(ms_idbuf, (size_t) w, sizeof ms_idbuf[0], ms_id_cmp);
    for (i = 0; i < w && i < cap && ids != NULL; ++i) memcpy(ids[i], ms_idbuf[i], GW_MS_ID);
    return w;
}

/* ---- journal ---------------------------------------------------------------------------------------------------- */

static int ms_exists(const char *path) { return GetFileAttributesA(path) != INVALID_FILE_ATTRIBUTES; }

static void ms_jpath(char *out, size_t cap, const char *mods_dir, const char *leaf) {
    snprintf(out, cap, "%s\\" MS_JDIR "\\%s", mods_dir, leaf);
}

static void ms_jdir(char *out, size_t cap, const char *mods_dir) { snprintf(out, cap, "%s\\" MS_JDIR, mods_dir); }

/* Write text to path atomically: a tmp file, then a replace. */
static int ms_write_file(const char *path, const char *text) {
    char tmp[MAX_PATH];
    FILE *f;
    snprintf(tmp, sizeof tmp, "%s.tmp", path);
    f = fopen(tmp, "wb");
    if (f == NULL) {
        gw_log("gw: modsync: cannot write %s", tmp);
        return -1;
    }
    if (fputs(text, f) < 0 || fclose(f) != 0 || !MoveFileExA(tmp, path, MOVEFILE_REPLACE_EXISTING)) {
        gw_log("gw: modsync: cannot replace %s", path);
        DeleteFileA(tmp);
        return -1;
    }
    return 0;
}

/* header (may be "") then one id per line, written atomically. */
static int ms_write_lines(const char *path, const char *header, const char *const *ids, int n) {
    size_t cap = strlen(header) + 2, len;
    int i, rc;
    char *out;
    for (i = 0; i < n; ++i) cap += strlen(ids[i]) + 1;
    out = (char *) malloc(cap);
    if (out == NULL) return -1;
    strcpy(out, header);
    len = strlen(out);
    for (i = 0; i < n; ++i) {
        size_t l = strlen(ids[i]);
        if (l == 0) continue;
        memcpy(out + len, ids[i], l);
        len += l;
        out[len++] = '\n';
    }
    out[len] = '\0';
    rc = ms_write_file(path, out);
    free(out);
    return rc;
}

/* enabled.txt from the orig journal text: the same file set_save writes (header, then the ids). */
static int ms_restore_enabled(const char *enabled_path, const char *orig_text) {
    size_t cap = strlen(orig_text) + sizeof MS_ENABLED_HEADER + 2, len;
    const char *line = orig_text;
    char *out = (char *) malloc(cap);
    int rc;
    if (out == NULL) return -1;
    strcpy(out, MS_ENABLED_HEADER);
    len = strlen(out);
    while (*line != '\0') {
        const char *nl = strchr(line, '\n');
        const char *eol = nl != NULL ? nl : line + strlen(line);
        const char *a = line, *b = eol;
        while (a < b && (*a == ' ' || *a == '\t' || *a == '\r')) ++a;
        while (b > a && (b[-1] == ' ' || b[-1] == '\t' || b[-1] == '\r')) --b;
        if (b > a) {
            memcpy(out + len, a, (size_t) (b - a));
            len += (size_t) (b - a);
            out[len++] = '\n';
        }
        line = nl != NULL ? nl + 1 : eol;
    }
    out[len] = '\0';
    rc = ms_write_file(enabled_path, out);
    free(out);
    return rc;
}

static int ms_write_state(const char *mods_dir, int state, const char *room, uint32_t pid, int64_t t) {
    char path[MAX_PATH], text[256];
    ms_jpath(path, sizeof path, mods_dir, "state.txt");
    snprintf(text, sizeof text, "state=%d\nroom=%s\nt=%lld\npid=%lu\n", state, room != NULL ? room : "", (long long) t,
             (unsigned long) pid);
    return ms_write_file(path, text);
}

/* Remove the journal's files and its folder. 0 = nothing is left behind. */
static int ms_remove_journal(const char *mods_dir) {
    char p[MAX_PATH], d[MAX_PATH];
    ms_jpath(p, sizeof p, mods_dir, "state.txt");
    DeleteFileA(p);
    ms_jpath(p, sizeof p, mods_dir, "state.txt.tmp");
    DeleteFileA(p);
    ms_jpath(p, sizeof p, mods_dir, "enabled.orig");
    DeleteFileA(p);
    ms_jpath(p, sizeof p, mods_dir, "enabled.orig.tmp");
    DeleteFileA(p);
    ms_jdir(d, sizeof d, mods_dir);
    RemoveDirectoryA(d);
    return ms_exists(d) ? -1 : 0;
}

int gw_ms_j_read(const char *mods_dir, gw_ms_journal *j) {
    char path[MAX_PATH], *text, *line, *next = NULL;
    int have_state = 0;
    if (j == NULL) return -1;
    memset(j, 0, sizeof *j);
    j->state = GW_MS_J_NONE;
    if (mods_dir == NULL) return 0;
    ms_jpath(path, sizeof path, mods_dir, "state.txt");
    if (!ms_exists(path)) return 0;
    text = read_text_file(path);
    if (text == NULL) return -1;
    for (line = text; line != NULL && *line != '\0'; line = next) {
        size_t l;
        next = strchr(line, '\n');
        if (next != NULL) *next++ = '\0';
        l = strlen(line);
        if (l > 0 && line[l - 1] == '\r') line[l - 1] = '\0';
        if (strncmp(line, "state=", 6) == 0) {
            j->state = (int) ms_atoi64(line + 6);
            have_state = 1;
        } else if (strncmp(line, "room=", 5) == 0) {
            copy_str(j->room, sizeof j->room, line + 5);
        } else if (strncmp(line, "t=", 2) == 0) {
            j->t = ms_atoi64(line + 2);
        } else if (strncmp(line, "pid=", 4) == 0) {
            j->pid = (uint32_t) ms_atoi64(line + 4);
        }
    }
    free(text);
    if (!have_state || j->state < GW_MS_J_NONE || j->state > GW_MS_J_ACTIVE) {
        memset(j, 0, sizeof *j);
        j->state = GW_MS_J_NONE;
        return -1;
    }
    return 0;
}

int gw_ms_j_begin(const char *mods_dir, const char *room, const char *const *orig_ids, int n, int64_t now) {
    char path[MAX_PATH], orig[MAX_PATH];
    if (mods_dir == NULL || n < 0 || (n > 0 && orig_ids == NULL)) return -1;
    ms_jpath(path, sizeof path, mods_dir, "state.txt");
    if (ms_exists(path)) return -1; /* a session already exists */
    CreateDirectoryA(mods_dir, NULL);
    {
        char d[MAX_PATH];
        ms_jdir(d, sizeof d, mods_dir);
        CreateDirectoryA(d, NULL);
    }
    ms_jpath(orig, sizeof orig, mods_dir, "enabled.orig");
    if (ms_write_lines(orig, "", orig_ids, n) != 0) return -1;
    if (ms_write_state(mods_dir, GW_MS_J_DOWNLOADING, room, 0, now) != 0) {
        DeleteFileA(orig);
        return -1;
    }
    gw_log("gw: modsync: session begun (room %s, %d mod(s) saved)", room != NULL ? room : "", n);
    return 0;
}

int gw_ms_j_set(const char *mods_dir, int state, uint32_t pid, int64_t now) {
    gw_ms_journal j;
    char path[MAX_PATH];
    if (mods_dir == NULL || state < GW_MS_J_NONE || state > GW_MS_J_ACTIVE) return -1;
    ms_jpath(path, sizeof path, mods_dir, "state.txt");
    if (!ms_exists(path)) return -1;
    gw_ms_j_read(mods_dir, &j);
    return ms_write_state(mods_dir, state, j.room, pid, now);
}

int gw_ms_write_enabled(const char *mods_dir, const char *const *ids, int n) {
    char path[MAX_PATH];
    int rc;
    if (mods_dir == NULL || n < 0 || (n > 0 && ids == NULL)) return -1;
    CreateDirectoryA(mods_dir, NULL);
    snprintf(path, sizeof path, "%s\\enabled.txt", mods_dir);
    rc = ms_write_lines(path, MS_ENABLED_HEADER, ids, n);
    if (rc == 0) gw_log("gw: modsync: wrote %s (%d mod(s))", path, n);
    return rc;
}

int gw_ms_j_restore(const char *mods_dir) {
    char orig[MAX_PATH], enabled[MAX_PATH];
    if (mods_dir == NULL) return -1;
    ms_jpath(orig, sizeof orig, mods_dir, "enabled.orig");
    snprintf(enabled, sizeof enabled, "%s\\enabled.txt", mods_dir);
    if (ms_exists(orig)) {
        char *text = read_text_file(orig);
        int rc;
        if (text == NULL) return -1;
        rc = ms_restore_enabled(enabled, text); /* first: the player's set is back before the journal goes */
        free(text);
        if (rc != 0) return -1;
        gw_log("gw: modsync: restored the enabled set");
    }
    return ms_remove_journal(mods_dir);
}

int gw_ms_j_keep(const char *mods_dir) {
    if (mods_dir == NULL) return -1;
    gw_log("gw: modsync: kept the synced mod set");
    return ms_remove_journal(mods_dir);
}

int gw_ms_boot(const char *mods_dir, int64_t now, uint32_t my_pid, int (*pid_alive)(uint32_t), char *room_out, int room_cap) {
    gw_ms_journal j;
    if (mods_dir == NULL) return 0;
    gw_ms_j_read(mods_dir, &j);
    switch (j.state) {
    case GW_MS_J_DOWNLOADING:
        gw_log("gw: modsync: a download did not finish - dropped (nothing was applied)");
        gw_ms_j_keep(mods_dir);
        return 0;
    case GW_MS_J_APPLIED:
        gw_log("gw: modsync: the game ended before it relaunched - restoring the enabled set");
        gw_ms_j_restore(mods_dir);
        return 2;
    case GW_MS_J_LAUNCHING:
        if (now - j.t <= MS_LAUNCH_WINDOW) {
            if (gw_ms_j_set(mods_dir, GW_MS_J_ACTIVE, my_pid, now) != 0) {
                gw_ms_j_restore(mods_dir);
                return 2;
            }
            if (room_out != NULL && room_cap > 0) copy_str(room_out, (size_t) room_cap, j.room);
            gw_log("gw: modsync: continuing the sync session (room %s)", j.room);
            return 1;
        }
        gw_log("gw: modsync: the relaunch took too long - restoring the enabled set");
        gw_ms_j_restore(mods_dir);
        return 2;
    case GW_MS_J_ACTIVE: {
        int alive = pid_alive != NULL && pid_alive(j.pid) != 0;
        if (alive) return 0; /* another running game owns the session */
        gw_log("gw: modsync: the game that synced is gone - restoring the enabled set");
        gw_ms_j_restore(mods_dir);
        return 2;
    }
    default:
        return 0;
    }
}

/* ---- tests ------------------------------------------------------------------------------------------------------ */

#define MS_NEED(cond, msg) do { if (!(cond)) { gw_test_fail("%s", msg); return 1; } } while (0)
#define MS_CHECK(cond, msg) do { if (!(cond)) { gw_test_fail("%s", msg); goto done; } } while (0)

static void t_ms_put(gw_ms_manifest *m, const char *id, const char *name, const char *kind, const char *source, int post,
                     uint64_t digest, int enabled, int mounted) {
    gw_ms_entry *e;
    if (m->n >= GW_MS_MAX) return;
    e = &m->e[m->n++];
    memset(e, 0, sizeof *e);
    copy_str(e->id, sizeof e->id, id);
    copy_str(e->name, sizeof e->name, name);
    copy_str(e->kind, sizeof e->kind, kind);
    copy_str(e->source, sizeof e->source, source);
    e->post = post;
    e->digest = digest;
    e->ident = strcmp(kind, "skin") != 0;
    e->enabled = enabled;
    e->mounted = mounted;
}

/* ids listed in rel (comments and blank lines skipped). Returns the count, -1 when the file is missing. */
static int t_read_ids(const char *rel, char ids[][GW_MS_ID], int cap) {
    char path[MAX_PATH];
    char *text, *line, *next = NULL;
    int n = 0;
    snprintf(path, sizeof path, "%s\\%s", gw_mods_test_root, rel);
    text = read_text_file(path);
    if (text == NULL) return -1;
    for (line = text; line != NULL && *line != '\0'; line = next) {
        size_t l;
        next = strchr(line, '\n');
        if (next != NULL) *next++ = '\0';
        while (*line == ' ' || *line == '\t' || *line == '\r') ++line;
        l = strlen(line);
        while (l > 0 && (line[l - 1] == ' ' || line[l - 1] == '\t' || line[l - 1] == '\r')) line[--l] = '\0';
        if (*line == '\0' || *line == '#') continue;
        if (n < cap) copy_str(ids[n], GW_MS_ID, line);
        ++n;
    }
    free(text);
    return n;
}

static int t_ids_eq(const char *rel, const char *const *want, int n) {
    char got[16][GW_MS_ID];
    int g = t_read_ids(rel, got, 16), i;
    if (g != n || g > 16) return 0;
    for (i = 0; i < n; ++i) {
        if (strcmp(got[i], want[i]) != 0) return 0;
    }
    return 1;
}

static int test_ms_roundtrip(void) {
    static gw_ms_manifest m;
    static uint8_t chunks[MS_MAX_CHUNKS][MS_CHUNK_MAX];
    static int lens[MS_MAX_CHUNKS];
    static const int caps[2] = { 200, 60 };
    static gw_mod gm;
    int ci, N, k, i;
    const gw_ms_manifest *peer;
    memset(&m, 0, sizeof m);
    m.global = 0x0123456789abcdefull;
    t_ms_put(&m, "ace-base", "ACE base", "base", "nucleus", 1234, 0x1111u, 1, 1);
    t_ms_put(&m, "ace-wolf", "Wolf with a long display name", "fighter", "nucleus", 4321, 0x2222u, 1, 1);
    t_ms_put(&m, "not-mounted", "Off", "misc", "", 0, 0, 0, 0);
    t_ms_put(&m, "legacy-folder-mod", "legacy", "misc", "", 0, 0x3333u, 1, 1);
    t_ms_put(&m, "skin-costume", "Costume", "skin", "", 0, 0x4444u, 1, 1);
    for (ci = 0; ci < 2; ++ci) {
        int cap = caps[ci], mounted = 0, j;
        gw_ms_wire_reset(NULL);
        for (N = 0; N < MS_MAX_CHUNKS; ++N) {
            lens[N] = gw_ms_wire_encode(&m, N, chunks[N], cap);
            if (lens[N] <= 0) break;
            if (lens[N] > cap) {
                gw_test_fail("chunk %d is %d bytes at cap %d", N, lens[N], cap);
                return 1;
            }
        }
        if (N >= MS_MAX_CHUNKS || N < (cap == 60 ? 3 : 1)) {
            gw_test_fail("cap %d gave %d chunks", cap, N);
            return 1;
        }
        if (gw_ms_wire_encode(&m, N, chunks[0], cap) != 0) {
            gw_test_fail("encode did not stop at the last chunk (cap %d)", cap);
            return 1;
        }
        for (k = N - 1; k >= 0; --k) { /* reverse order: chunk 0 arrives last */
            if (!gw_ms_wire_feed(chunks[k], lens[k])) {
                gw_test_fail("chunk %d of %d not taken as ours (cap %d)", k, N, cap);
                return 1;
            }
            if (k > 0 && gw_ms_peer_ready()) {
                gw_test_fail("ready before chunk 0 (cap %d)", cap);
                return 1;
            }
        }
        if (!gw_ms_peer_ready()) {
            gw_test_fail("not ready after all %d chunks (cap %d)", N, cap);
            return 1;
        }
        if (N > 1) { /* a repeat of a held chunk changes nothing */
            if (!gw_ms_wire_feed(chunks[N - 1], lens[N - 1]) || !gw_ms_peer_ready()) {
                gw_test_fail("a repeated chunk unset the peer (cap %d)", cap);
                return 1;
            }
        }
        peer = gw_ms_peer();
        for (i = 0; i < m.n; ++i) mounted += m.e[i].mounted != 0;
        if (peer->n != mounted || peer->global != m.global) {
            gw_test_fail("peer has %d entries, expected %d (cap %d)", peer->n, mounted, cap);
            return 1;
        }
        j = 0;
        for (i = 0; i < m.n; ++i) {
            const gw_ms_entry *a = &m.e[i], *b;
            if (!a->mounted) continue;
            b = &peer->e[j++];
            if (strcmp(a->id, b->id) != 0 || strcmp(a->name, b->name) != 0 || strcmp(a->kind, b->kind) != 0 ||
                strcmp(a->source, b->source) != 0 || a->post != b->post || a->digest != b->digest ||
                a->ident != b->ident || !b->mounted || !b->enabled) {
                gw_test_fail("peer entry %s differs from ours (cap %d)", a->id, cap);
                return 1;
            }
        }
    }
    /* mod.json: "source" and the nested Nucleus post id */
    memset(&gm, 0, sizeof gm);
    if (mod_parse_json(&gm, "{\"name\":\"Wolf\",\"source\":\"nucleus\",\"nucleus\":{\"id\":4321,\"url\":\"x\"}}") != 0 ||
        strcmp(gm.source, "nucleus") != 0 || gm.nucleus_post != 4321) {
        gw_test_fail("source / nucleus id not read from mod.json");
        return 1;
    }
    memset(&gm, 0, sizeof gm);
    if (mod_parse_json(&gm, "{\"name\":\"Local fighter\",\"source\":\"local file\",\"local\":{\"hash\":\"1234\"}}") != 0 ||
        strcmp(gm.source, "local file") != 0 || gm.nucleus_post != 0) {
        gw_test_fail("local source must survive parsing without a Nucleus post");
        return 1;
    }
    memset(&gm, 0, sizeof gm);
    if (mod_parse_json(&gm, "{\"nucleus\":{\"id\":\"4321\"}}") != 0 || gm.nucleus_post != 0) {
        gw_test_fail("a non-number nucleus id must read as 0");
        return 1;
    }
    return 0;
}

static int test_ms_malformed(void) {
    static gw_ms_manifest m;
    static uint8_t c[MS_MAX_CHUNKS][MS_CHUNK_MAX];
    static int l[MS_MAX_CHUNKS];
    static uint8_t junk[MS_CHUNK_MAX];
    static const char *const bad[] = { "MSM x y z", "MSM 0 0 abc", "MSM 5 2 abc", "MSM 0 1 ", "MSM 0 1 G zzzz",
                                       "MSM 9999 1 a", "MSM 0 300 a" };
    static const char *const raw_bad[] = { "MSM 0 1 G 0000000000000001 2\nE a|b|misc|nucleus|5|0000000000000001|1",
                                           "MSM 0 1 G 0000000000000001 1\nE a|b|misc|nucleus|5|zzzzzzzzzzzzzzzz|1",
                                           "MSM 0 1 G 0000000000000001 1\nE a|b|misc||0000000000000001|1",
                                           "MSM 0 1 G 0000000000000001 1\nE a|b|misc|nucleus|5|0000000000000001|1\n"
                                           "E c|d|misc|nucleus|6|0000000000000002|1" };
    static const char *const raw_good = "MSM 0 1 G 0000000000000001 1\nE a|b|misc|nucleus|5|0000000000000001|1";
    unsigned rng = 12345u;
    int N = 0, k, i, j;
    memset(&m, 0, sizeof m);
    m.global = 7;
    t_ms_put(&m, "ace-base", "ACE base", "base", "nucleus", 1, 0x10, 1, 1);
    t_ms_put(&m, "ace-wolf", "Wolf", "fighter", "nucleus", 2, 0x20, 1, 1);
    t_ms_put(&m, "legacy", "legacy", "misc", "", 0, 0x30, 1, 1);
    gw_ms_wire_reset(NULL);
    MS_NEED(gw_ms_wire_feed(NULL, 0) == 0, "a NULL message must not be ours");
    MS_NEED(gw_ms_wire_feed((const uint8_t *) "hello", 5) == 0, "a non-MSM message must not be ours");
    for (i = 0; i < (int) (sizeof bad / sizeof bad[0]); ++i) {
        MS_NEED(gw_ms_wire_feed((const uint8_t *) bad[i], (int) strlen(bad[i])) == 1, "a malformed MSM chunk must be consumed");
    }
    MS_NEED(!gw_ms_peer_ready(), "garbage made the peer ready");
    for (N = 0; N < MS_MAX_CHUNKS; ++N) {
        l[N] = gw_ms_wire_encode(&m, N, c[N], 60);
        if (l[N] <= 0) break;
    }
    MS_NEED(N >= 3, "the valid stream should take at least 3 chunks at cap 60");

    gw_ms_wire_reset(NULL); /* all but the last: not ready */
    for (k = 0; k < N - 1; ++k) gw_ms_wire_feed(c[k], l[k]);
    MS_NEED(!gw_ms_peer_ready(), "ready with a chunk missing");
    gw_ms_wire_feed(c[1], l[1]); /* a repeat does not count twice */
    MS_NEED(!gw_ms_peer_ready(), "a repeated chunk completed the stream");
    gw_ms_wire_feed(c[N - 1], l[N - 1]);
    MS_NEED(gw_ms_peer_ready(), "the complete stream did not become ready");

    gw_ms_wire_reset(NULL); /* a chunk 0 restarts: the partial stream is dropped */
    for (k = 0; k < N - 1; ++k) gw_ms_wire_feed(c[k], l[k]);
    gw_ms_wire_feed(c[0], l[0]);
    MS_NEED(!gw_ms_peer_ready(), "restart did not drop the partial stream");
    for (k = 1; k < N; ++k) gw_ms_wire_feed(c[k], l[k]);
    MS_NEED(gw_ms_peer_ready(), "restart plus a full resend did not become ready");

    gw_ms_wire_reset(NULL); /* a corrupted header: malformed, then ignored until a new stream */
    memcpy(junk, c[0], (size_t) l[0]);
    {
        uint8_t *g = (uint8_t *) memchr(junk, 'G', (size_t) l[0]);
        if (g != NULL) *g = 'X';
    }
    gw_ms_wire_feed(junk, l[0]);
    for (k = 1; k < N; ++k) gw_ms_wire_feed(c[k], l[k]);
    MS_NEED(!gw_ms_peer_ready(), "a corrupted header became ready");

    for (i = 0; i < (int) (sizeof raw_bad / sizeof raw_bad[0]); ++i) { /* hand-made streams, each malformed */
        gw_ms_wire_reset(NULL);
        gw_ms_wire_feed((const uint8_t *) raw_bad[i], (int) strlen(raw_bad[i]));
        MS_NEED(!gw_ms_peer_ready(), "a malformed record stream became ready");
    }
    gw_ms_wire_reset(NULL); /* the same format, correct: it is ready (so the cases above fail for the right reason) */
    gw_ms_wire_feed((const uint8_t *) raw_good, (int) strlen(raw_good));
    MS_NEED(gw_ms_peer_ready() && gw_ms_peer()->n == 1 && gw_ms_peer()->e[0].post == 5, "the valid hand-made stream was refused");

    for (i = 0; i < 300; ++i) { /* random byte flips in a valid set: no crash */
        gw_ms_wire_reset(NULL);
        for (k = 0; k < N; ++k) {
            memcpy(junk, c[k], (size_t) l[k]);
            rng = rng * 1103515245u + 12345u;
            junk[(rng >> 4) % (unsigned) l[k]] = (uint8_t) (rng >> 16);
            gw_ms_wire_feed(junk, l[k]);
        }
    }
    for (i = 0; i < 300; ++i) { /* random payloads behind a valid prefix: no crash */
        memcpy(junk, "MSM 0 1 ", 8);
        for (j = 8; j < 40; ++j) {
            rng = rng * 1103515245u + 12345u;
            junk[j] = (uint8_t) (rng >> 16);
        }
        gw_ms_wire_feed(junk, 40);
    }
    gw_ms_wire_reset(NULL);
    MS_NEED(!gw_ms_peer_ready(), "reset did not clear the ready mark");
    return 0;
}

static int test_ms_diff(void) {
    static gw_ms_manifest mine, theirs;
    static gw_ms_row rows[16], two[2];
    int n, n2, n3;
    memset(&mine, 0, sizeof mine);
    memset(&theirs, 0, sizeof theirs);
    t_ms_put(&mine, "ace-base", "ACE base", "base", "nucleus", 1, 100, 1, 1);
    t_ms_put(&theirs, "ace-base", "ACE base", "base", "nucleus", 1, 100, 1, 1);
    t_ms_put(&mine, "ace-wolf", "Wolf", "fighter", "nucleus", 77, 200, 1, 1);
    t_ms_put(&theirs, "ACE-WOLF", "Wolf", "fighter", "nucleus", 77, 201, 1, 1);
    t_ms_put(&mine, "mine-only", "Mine", "misc", "", 0, 5, 1, 1);
    t_ms_put(&mine, "skin-x", "Skin X", "skin", "", 0, 6, 1, 1);
    t_ms_put(&theirs, "skin-y", "Skin Y", "skin", "", 0, 7, 1, 1);
    t_ms_put(&theirs, "theirs-only", "Theirs", "stage", "nucleus", 9, 8, 1, 1);
    t_ms_put(&mine, "unmounted", "Not mounted", "misc", "", 0, 0, 0, 0);
    n = gw_ms_diff(&mine, &theirs, rows, 16);
    MS_NEED(n == 5, "diff should find 5 rows (same ace-base left out, unmounted left out)");
    MS_NEED(strcmp(rows[0].id, "ace-wolf") == 0 && rows[0].state == GW_MS_DIFFERS && rows[0].ident == 1, "row 0 (ace-wolf DIFFERS)");
    MS_NEED(strcmp(rows[1].id, "mine-only") == 0 && rows[1].state == GW_MS_MINE_ONLY && rows[1].ident == 1, "row 1 (mine-only)");
    MS_NEED(strcmp(rows[2].id, "skin-x") == 0 && rows[2].state == GW_MS_MINE_ONLY && rows[2].ident == 0, "row 2 (skin-x is a cosmetic row)");
    MS_NEED(strcmp(rows[3].id, "skin-y") == 0 && rows[3].state == GW_MS_THEIRS_ONLY && rows[3].ident == 0, "row 3 (skin-y)");
    MS_NEED(strcmp(rows[4].id, "theirs-only") == 0 && rows[4].state == GW_MS_THEIRS_ONLY && rows[4].ident == 1, "row 4 (theirs-only)");
    MS_NEED(gw_ms_ident_diffs(rows, n) == 3, "ident_diffs should count the 3 identity rows");
    n2 = gw_ms_diff(&mine, &theirs, two, 2);
    MS_NEED(n2 == 5 && strcmp(two[1].id, "mine-only") == 0, "cap must limit the rows, not the count");
    {   /* the peer's list is the mounted list: a copy of ours without the unmounted entry must match exactly */
        static gw_ms_manifest peer_view;
        int i;
        memset(&peer_view, 0, sizeof peer_view);
        for (i = 0; i < mine.n; ++i) {
            if (mine.e[i].mounted) peer_view.e[peer_view.n++] = mine.e[i];
        }
        n3 = gw_ms_diff(&mine, &peer_view, rows, 16);
        MS_NEED(n3 == 0 && gw_ms_ident_diffs(rows, 0) == 0, "identical sets must differ in nothing");
    }
    return 0;
}

static const gw_ms_action *ms_find_action(const gw_ms_plan *p, const char *id) {
    int i;
    for (i = 0; i < p->n; ++i) {
        if (_stricmp(p->a[i].id, id) == 0) return &p->a[i];
    }
    return NULL;
}

static int test_ms_plan(void) {
    static gw_ms_manifest mine, theirs, same_mine, same_theirs;
    static gw_ms_plan plan, empty;
    static char ids[64][GW_MS_ID];
    const gw_ms_action *a;
    int n;
    memset(&mine, 0, sizeof mine);
    memset(&theirs, 0, sizeof theirs);
    t_ms_put(&mine, "ace-base", "ACE base", "base", "nucleus", 1, 100, 1, 1);
    t_ms_put(&mine, "ace-wolf", "Wolf", "fighter", "nucleus", 70, 200, 1, 1);
    t_ms_put(&mine, "akaneia-sonic", "Sonic", "fighter", "nucleus", 4, 0, 0, 0);
    t_ms_put(&mine, "ace-half", "Half", "fighter", "nucleus", 3, 0, 1, 0);
    t_ms_put(&mine, "extra-misc", "Extra", "misc", "", 0, 5, 1, 1);
    t_ms_put(&mine, "skin-mine", "Skin", "skin", "", 0, 6, 1, 1);
    t_ms_put(&mine, "ace-old", "Old", "stage", "", 0, 3, 1, 1);
    t_ms_put(&theirs, "ace-base", "ACE base", "base", "nucleus", 1, 100, 1, 1);
    t_ms_put(&theirs, "ace-wolf", "Wolf", "fighter", "nucleus", 77, 201, 1, 1);
    t_ms_put(&theirs, "akaneia-sonic", "Sonic", "fighter", "nucleus", 5, 9, 1, 1);
    t_ms_put(&theirs, "ace-half", "Half", "fighter", "nucleus", 6, 9, 1, 1);
    t_ms_put(&theirs, "ace-sonic", "Ace Sonic", "fighter", "nucleus", 88, 55, 1, 1);
    t_ms_put(&theirs, "ace-stage", "Ace stage", "stage", "", 0, 66, 1, 1);
    t_ms_put(&theirs, "ace-old", "Old", "stage", "", 0, 4, 1, 1);
    t_ms_put(&theirs, "skin-theirs", "Their skin", "skin", "nucleus", 9, 7, 1, 1);
    gw_ms_plan_build(&mine, &theirs, &plan);
    MS_NEED(plan.n == 6, "plan should have 6 actions");
    MS_NEED(plan.ndownload == 2 && plan.nenable == 1 && plan.ndisable == 1 && plan.nmissing == 2, "plan counts wrong");
    MS_NEED(!plan.can_complete, "a MISSING mod must stop can_complete");
    MS_NEED(strcmp(plan.summary, "Cannot match: 2 mods are not on Nucleus") == 0, "summary wording");
    a = ms_find_action(&plan, "ace-wolf");
    MS_NEED(a && a->kind == GW_MS_ACT_DOWNLOAD && a->post == 77, "ace-wolf: a changed Nucleus mod is downloaded");
    a = ms_find_action(&plan, "akaneia-sonic");
    MS_NEED(a && a->kind == GW_MS_ACT_ENABLE, "akaneia-sonic: an installed, off mod is turned on");
    a = ms_find_action(&plan, "ace-sonic");
    MS_NEED(a && a->kind == GW_MS_ACT_DOWNLOAD && a->post == 88, "ace-sonic: not installed, Nucleus: download");
    a = ms_find_action(&plan, "extra-misc");
    MS_NEED(a && a->kind == GW_MS_ACT_DISABLE, "extra-misc: an extra identity mod is turned off");
    a = ms_find_action(&plan, "ace-stage");
    MS_NEED(a && a->kind == GW_MS_ACT_MISSING && strstr(a->why, "you need this mod: ") != NULL &&
                strstr(a->why, "(source unknown)") != NULL, "ace-stage: non-Nucleus missing wording");
    a = ms_find_action(&plan, "ace-old");
    MS_NEED(a && a->kind == GW_MS_ACT_MISSING && strstr(a->why, "(source unknown)") != NULL, "ace-old: changed, non-Nucleus: missing");
    MS_NEED(ms_find_action(&plan, "ace-base") == NULL && ms_find_action(&plan, "ace-half") == NULL, "in-sync mods need no action");
    MS_NEED(ms_find_action(&plan, "skin-mine") == NULL && ms_find_action(&plan, "skin-theirs") == NULL, "skins must stay untouched");
    {
        gw_ms_manifest empty = {0}, local = {0};
        gw_ms_plan lp;
        t_ms_put(&local, "local-fixture-fighter", "Local fighter", "fighter", "local file", 0, 123, 1, 1);
        gw_ms_plan_build(&empty, &local, &lp);
        MS_NEED(lp.ndownload == 0 && lp.nmissing == 1 && !lp.can_complete && lp.a[0].post == 0 &&
                strstr(lp.a[0].why, "you need this mod: Local fighter (local file)"), "local files must never queue a Nucleus download");
    }

    n = gw_ms_target_ids(&mine, &plan, ids, 64);
    MS_NEED(n == 7, "target set should have 7 ids");
    MS_NEED(strcmp(ids[0], "ace-base") == 0 && strcmp(ids[6], "skin-mine") == 0, "target ids sorted");
    MS_NEED(strcmp(ids[5], "akaneia-sonic") == 0 && strcmp(ids[4], "ace-wolf") == 0, "target ids: enables and downloads in");
    {
        int i;
        for (i = 0; i < n; ++i) MS_NEED(_stricmp(ids[i], "extra-misc") != 0, "target ids keep a disabled mod");
    }

    /* the clean case: same mods, a skin differs, nothing to do */
    memset(&same_mine, 0, sizeof same_mine);
    memset(&same_theirs, 0, sizeof same_theirs);
    t_ms_put(&same_mine, "ace-base", "ACE base", "base", "nucleus", 1, 100, 1, 1);
    t_ms_put(&same_theirs, "ace-base", "ACE base", "base", "nucleus", 1, 100, 1, 1);
    t_ms_put(&same_theirs, "their-skin", "Skin", "skin", "", 0, 3, 1, 1);
    gw_ms_plan_build(&same_mine, &same_theirs, &empty);
    MS_NEED(empty.n == 0 && empty.can_complete, "a matching set needs no actions");
    MS_NEED(strcmp(empty.summary, "Download 0, turn on 0, turn off 0") == 0, "clean summary wording");
    return 0;
}

static int test_ms_journal_flow(void) {
    static const char *const orig[] = { "ace-base", "ace-wolf" };
    static const char *const target[] = { "akaneia-base" };
    char dir[MAX_PATH];
    gw_ms_journal j;
    int rc = 1;
    if (t_setup() != 0) {
        gw_test_fail("cannot create a scratch folder");
        return 1;
    }
    snprintf(dir, sizeof dir, "%s\\" MS_JDIR, gw_mods_test_root);
    t_write("enabled.txt", "# the player's own set\nace-base\nace-wolf\n");
    MS_CHECK(gw_ms_j_begin(gw_mods_test_root, "ROOM1", orig, 2, 1000) == 0, "begin failed");
    MS_CHECK(gw_ms_j_begin(gw_mods_test_root, "ROOM1", orig, 2, 1001) != 0, "a second begin must fail");
    MS_CHECK(gw_ms_j_read(gw_mods_test_root, &j) == 0 && j.state == GW_MS_J_DOWNLOADING && strcmp(j.room, "ROOM1") == 0 &&
                 j.t == 1000, "the journal did not read back");
    MS_CHECK(gw_ms_write_enabled(gw_mods_test_root, target, 1) == 0, "writing the target set failed");
    MS_CHECK(gw_ms_j_set(gw_mods_test_root, GW_MS_J_APPLIED, 0, 1002) == 0, "state change failed");
    MS_CHECK(gw_ms_j_restore(gw_mods_test_root) == 0, "restore failed");
    MS_CHECK(t_ids_eq("enabled.txt", orig, 2), "restore did not give back the original list");
    MS_CHECK(!ms_exists(dir), "the journal folder was left behind");
    MS_CHECK(gw_ms_j_read(gw_mods_test_root, &j) == 0 && j.state == GW_MS_J_NONE, "a journal is still readable after restore");
    MS_CHECK(gw_ms_j_restore(gw_mods_test_root) == 0 && t_ids_eq("enabled.txt", orig, 2), "a second restore is not idempotent");
    MS_CHECK(gw_ms_j_begin(gw_mods_test_root, "ROOM2", orig, 2, 2000) == 0, "a new session failed to begin");
    MS_CHECK(gw_ms_write_enabled(gw_mods_test_root, target, 1) == 0, "writing the target set failed (keep)");
    MS_CHECK(gw_ms_j_keep(gw_mods_test_root) == 0, "keep failed");
    MS_CHECK(t_ids_eq("enabled.txt", target, 1), "keep did not leave the synced set in enabled.txt");
    MS_CHECK(!ms_exists(dir), "keep left the journal folder behind");
    rc = 0;
done:
    t_rmtree(gw_mods_test_root);
    return rc;
}

static int ms_alive_yes(uint32_t pid) {
    (void) pid;
    return 1;
}

static int ms_alive_no(uint32_t pid) {
    (void) pid;
    return 0;
}

/* a journal in `state` (begun as a download, then moved on) with the player's set `ace-base` saved */
static void t_ms_jstart(const char *room, int state, uint32_t pid, int64_t t) {
    static const char *const orig[] = { "ace-base" };
    char dir[MAX_PATH];
    snprintf(dir, sizeof dir, "%s\\" MS_JDIR, gw_mods_test_root);
    t_rmtree(dir);
    gw_ms_j_begin(gw_mods_test_root, room, orig, 1, t);
    if (state != GW_MS_J_DOWNLOADING) gw_ms_j_set(gw_mods_test_root, state, pid, t);
}

static int test_ms_boot(void) {
    static const char *const akaneia_only[] = { "akaneia-base" };
    static const char *const ace_only[] = { "ace-base" };
    const int64_t now = 1000000;
    const uint32_t me = 77, owner = 555;
    char room[8], dir[MAX_PATH];
    gw_ms_journal j;
    int rc = 1;
    if (t_setup() != 0) {
        gw_test_fail("cannot create a scratch folder");
        return 1;
    }
    snprintf(dir, sizeof dir, "%s\\" MS_JDIR, gw_mods_test_root);
    room[0] = '\0';

    t_rmtree(dir); /* no journal: nothing happens */
    t_write("enabled.txt", "akaneia-base\n");
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_no, room, sizeof room) == 0, "no journal: boot did something");
    MS_CHECK(t_ids_eq("enabled.txt", akaneia_only, 1), "no journal: enabled.txt changed");

    t_write("enabled.txt", "akaneia-base\n"); /* DOWNLOADING: dropped, nothing applied */
    t_ms_jstart("R1", GW_MS_J_DOWNLOADING, 0, now);
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_no, room, sizeof room) == 0, "downloading: expected 0");
    MS_CHECK(!ms_exists(dir), "downloading: the journal was not dropped");
    MS_CHECK(t_ids_eq("enabled.txt", akaneia_only, 1), "downloading: enabled.txt must be left as it is");

    t_write("enabled.txt", "akaneia-base\n"); /* APPLIED: restored */
    t_ms_jstart("R1", GW_MS_J_APPLIED, 0, now);
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_no, room, sizeof room) == 2, "applied: expected restore (2)");
    MS_CHECK(t_ids_eq("enabled.txt", ace_only, 1) && !ms_exists(dir), "applied: the player's set was not put back");

    t_write("enabled.txt", "akaneia-base\n"); /* LAUNCHING, fresh: continues as ACTIVE */
    t_ms_jstart("R1", GW_MS_J_LAUNCHING, 0, now - 10);
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_no, room, sizeof room) == 1, "launching fresh: expected continue (1)");
    MS_CHECK(strcmp(room, "R1") == 0, "launching fresh: room not returned");
    MS_CHECK(gw_ms_j_read(gw_mods_test_root, &j) == 0 && j.state == GW_MS_J_ACTIVE && j.pid == me,
             "launching fresh: the journal is not ACTIVE under this process");
    MS_CHECK(t_ids_eq("enabled.txt", akaneia_only, 1), "launching fresh: the synced set must stay");

    t_write("enabled.txt", "akaneia-base\n"); /* LAUNCHING, stale: restored */
    t_ms_jstart("R1", GW_MS_J_LAUNCHING, 0, now - 200);
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_no, room, sizeof room) == 2, "launching stale: expected restore (2)");
    MS_CHECK(t_ids_eq("enabled.txt", ace_only, 1) && !ms_exists(dir), "launching stale: not restored");

    t_write("enabled.txt", "akaneia-base\n"); /* ACTIVE, owner dead: restored (crash) */
    t_ms_jstart("R2", GW_MS_J_ACTIVE, owner, now);
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_no, room, sizeof room) == 2, "active, dead owner: expected restore (2)");
    MS_CHECK(t_ids_eq("enabled.txt", ace_only, 1) && !ms_exists(dir), "active, dead owner: not restored");
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_no, room, sizeof room) == 0, "a second boot after a restore must do nothing");
    MS_CHECK(t_ids_eq("enabled.txt", ace_only, 1), "a second boot changed enabled.txt");

    t_write("enabled.txt", "akaneia-base\n"); /* ACTIVE, owner alive: another game's session, left alone */
    t_ms_jstart("R2", GW_MS_J_ACTIVE, owner, now);
    MS_CHECK(gw_ms_boot(gw_mods_test_root, now, me, ms_alive_yes, room, sizeof room) == 0, "active, live owner: expected 0");
    MS_CHECK(gw_ms_j_read(gw_mods_test_root, &j) == 0 && j.state == GW_MS_J_ACTIVE && j.pid == owner,
             "active, live owner: the journal was touched");
    MS_CHECK(t_ids_eq("enabled.txt", akaneia_only, 1), "active, live owner: enabled.txt was touched");
    rc = 0;
done:
    t_rmtree(gw_mods_test_root);
    return rc;
}

void gw_modsync_tests_register(void) {
    gw_test_register("modsync_manifest_roundtrip", test_ms_roundtrip);
    gw_test_register("modsync_wire_malformed", test_ms_malformed);
    gw_test_register("modsync_diff", test_ms_diff);
    gw_test_register("modsync_plan", test_ms_plan);
    gw_test_register("modsync_journal_flow", test_ms_journal_flow);
    gw_test_register("modsync_boot", test_ms_boot);
}
