/* gw_nucleus_engine.inc - the Nucleus browser's engine half: worker threads, the catalog and its cache, the download queue, pictures, installs, and the source
 * the NUCLEUS screen reads (AtNcSrc, gw_ui_nucleus.inc). Included by gw_script.c at the top of gw_script_ui_mods.inc (after gw_ui_nucleus.inc); the pure logic is
 * gw_nucleus_catalog.h / gw_nucleus_install.h / gw_nucleus_queue.h / gw_nucleus_image.h and the transport gw_nucleus_http.inc.
 *
 * NO NETWORK OR DISK WORK RUNS ON THE GAME THREAD, AND NONE STARTS DURING A NETPLAY SESSION. Four kinds of thread, started the first time the NUCLEUS screen
 * opens and kept for the process:
 *   catalog  : syncs (first a full keyset page-through, then updated_since and the removed feed, at most once per 10 minutes, with back-off);
 *   files    : the DOWNLOAD QUEUE (installs one mod after another, never online), uninstalls, and the saves of the queue and the remembered filters;
 *   pictures : two threads that fetch, cache and decode the pictures of the rows in view (see below).
 * The screen only reads copies under one lock and starts nothing while a netplay session is on.
 *
 * Where things live. mods dir = gw_Mods_Dir() (MELEE_MODS_DIR). The cache is <mods dir>/.nucleus (a dot folder: the mods scanner skips it), or MELEE_NUCLEUS_DIR:
 *   catalog.jsonl, state.json  the catalog and where the sync is;
 *   queue.json                 the download queue, written atomically after every change (gw_nucleus_queue.h);
 *   ui.json                    the filters and the tab, kept across sessions;
 *   thumbs/<hash>.img          the pictures as served (PNG), a disk cache trimmed to MELEE_NUCLEUS_CACHE_MB (default 200) by last use;
 *   dl/<hash>.bin              downloaded costume files and zips, kept so a retry or a re-install does not count another download (trimmed to 400 MB);
 *   art/<file id>_csp.png      portraits and stock icons that installs convert into a skin's art.
 * An install writes <mods dir>/nucleus-<post>-<fighter>/ (a skin mod, gw_nucleus_install.h) through a .staging-* folder that is renamed into place only when
 * complete; one a crash left behind is removed at the next start. The skin is read at the next boot (the skin registry is built before the arena is carved), so
 * the screen says "Restart the game to use new skins". A skin the other side lacks is shown as the default costume by the registry (docs/mods-packaging.md 4b).
 *
 * PICTURES. A row asks (ne_pic_get) for its picture every frame it is on screen; the first ask records a want and returns "not yet" (the row draws a placeholder).
 * A picture thread takes the most recently wanted entry (a want not renewed for 3 seconds is dropped: the row scrolled away), reads the disk cache or downloads
 * the file (a media address, 6 MB at most), decodes the PNG OFF the game thread (stb_image), fits it into a small fixed canvas and keeps the .gxtex blob.
 * The game thread uploads at most NE_UPLOADS_PER_FRAME blobs per frame with gw_Kit_TexAddGxtex; the kit's texture pool evicts by use, and an evicted picture is
 * uploaded again from its blob (no decode, no disk).
 *
 * The thread-safety rule for logging: a worker never calls gw_log; it queues a line and the screen's tick (ne_pump) writes it. */
#ifndef GW_NUCLEUS_ENGINE_INC
#define GW_NUCLEUS_ENGINE_INC

#include "gw_nucleus_queue.h"
#include "gw_nucleus_image.h"
#include "gw_nucleus_http.inc"

#define NE_INST_MAX 256
#define NE_QUEUE 16
#define NE_PICS 640
#define NE_UPLOADS_PER_FRAME 2
#define NE_WANT_TICKS 180            /* a picture want that was not renewed for this many ticks (3 s) is dropped */
#define NE_DL_CAP (400ll * 1024 * 1024)

typedef struct { int kind, id; } ne_job;     /* kind: 2 uninstall (installs go through the download queue) */

enum { NE_PIC_FREE, NE_PIC_WANT, NE_PIC_BUSY, NE_PIC_READY, NE_PIC_FAILED };
typedef struct {
    uint64_t key;
    int state, kind;
    unsigned last;                   /* the tick of the last ask (LRU) */
    uint8_t *blob; size_t blen;      /* READY: the .gxtex */
    int64_t retry_at;                /* FAILED: when it may be asked again */
    char url[260];
} ne_pic;

static struct {
    int inited, quit, loaded;
    char dir[MAX_PATH], mods[MAX_PATH], art[MAX_PATH], thumbs[MAX_PATH], dl[MAX_PATH], catpath[MAX_PATH], statepath[MAX_PATH], qpath[MAX_PATH], prefpath[MAX_PATH];
    CRITICAL_SECTION lock;
    nc_catalog cat;
    nc_sync sync;
    int sync_want, syncing, sync_done, sync_total, sync_failed, sync_ever_ok;
    int cache_stale;                 /* the catalog on disk was from an older schema and was not loaded: nothing is installed from it, a record is refetched first */
    char sync_what[64], sync_msg[140];
    ne_job q[NE_QUEUE]; int qh, qt;  /* uninstalls */
    int job_state;
    int64_t job_at;
    char job_msg[160];
    nc_inst inst[NE_INST_MAX]; int ninst;
    int restart;
    nq_queue dq;                     /* the download queue */
    int dq_dirty, pref_dirty, interrupt;
    nq_prefs pref;
    int cur_id, cur_zip;
    ne_pic pics[NE_PICS];
    unsigned tick;
    int uploads;
    int img_writes;
    int64_t cache_cap;
    char logq[24][220]; int nlog;
    unsigned ver;
    HANDLE th_sync, th_files, th_img[2];
} NE;

static void ne_lock(void) { EnterCriticalSection(&NE.lock); }
static void ne_unlock(void) { LeaveCriticalSection(&NE.lock); }

static void ne_log(const char *fmt, ...) {
    char tmp[220];
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(tmp, sizeof tmp, fmt, ap);
    va_end(ap);
    ne_lock();
    if (NE.nlog < 24) snprintf(NE.logq[NE.nlog++], sizeof NE.logq[0], "%s", tmp);
    ne_unlock();
}

/* the game thread, once per tick: write what the workers queued, and renew the per-frame budgets */
static void ne_pump(void) {
    int i;
    char lines[24][220];
    int n;
    if (!NE.inited) return;
    ne_lock();
    n = NE.nlog;
    for (i = 0; i < n; ++i) memcpy(lines[i], NE.logq[i], sizeof lines[i]);
    NE.nlog = 0;
    ++NE.tick;
    NE.uploads = 0;
    ne_unlock();
    for (i = 0; i < n; ++i) gw_log("nucleus: %s", lines[i]);
}

static int64_t ne_now(void) { return (int64_t) time(NULL); }
static int ne_locked(void) { return gs_ui_netplay(); }

/* ---- paths and persistence ---- */

static void ne_paths(void) {
    const char *env = getenv("MELEE_NUCLEUS_DIR"), *mb = getenv("MELEE_NUCLEUS_CACHE_MB");
    snprintf(NE.mods, sizeof NE.mods, "%s", gw_Mods_Dir());
    nc_fix_slashes(NE.mods);
    {   /* no trailing separator */
        size_t n = strlen(NE.mods);
        while (n > 1 && (NE.mods[n - 1] == '\\' || NE.mods[n - 1] == '/')) NE.mods[--n] = '\0';
    }
    if (env && env[0]) snprintf(NE.dir, sizeof NE.dir, "%s", env);
    else snprintf(NE.dir, sizeof NE.dir, "%s/.nucleus", NE.mods);
    nc_fix_slashes(NE.dir);
    snprintf(NE.art, sizeof NE.art, "%s/art", NE.dir);
    snprintf(NE.thumbs, sizeof NE.thumbs, "%s/thumbs", NE.dir);
    snprintf(NE.dl, sizeof NE.dl, "%s/dl", NE.dir);
    snprintf(NE.catpath, sizeof NE.catpath, "%s/catalog.jsonl", NE.dir);
    snprintf(NE.statepath, sizeof NE.statepath, "%s/state.json", NE.dir);
    snprintf(NE.qpath, sizeof NE.qpath, "%s/queue.json", NE.dir);
    snprintf(NE.prefpath, sizeof NE.prefpath, "%s/ui.json", NE.dir);
    nc_fix_slashes(NE.art); nc_fix_slashes(NE.thumbs); nc_fix_slashes(NE.dl); nc_fix_slashes(NE.catpath); nc_fix_slashes(NE.statepath);
    nc_fix_slashes(NE.qpath); nc_fix_slashes(NE.prefpath);
    NE.cache_cap = (mb && atoi(mb) > 0) ? (int64_t) atoi(mb) * 1024 * 1024 : (int64_t) NQ_CACHE_CAP_DEFAULT;
}

/* the catalog and the sync state to disk; the caller does NOT hold the lock */
static void ne_save(void) {
    nj_buf b;
    memset(&b, 0, sizeof b);
    ne_lock();
    nc_cat_save(&NE.cat, NE.catpath);
    nc_sync_to_json(&NE.sync, &b);
    ne_unlock();
    if (!b.bad && b.s) nc_write_file_atomic(NE.statepath, b.s, b.n);
    free(b.s);
}

/* the queue and the filters to disk (the files thread; a copy is taken under the lock so the lock is not held across the flush to disk) */
static void ne_save_dirty(void) {
    nq_queue *copy = NULL;
    nq_prefs pf;
    int dq = 0, pd = 0;
    memset(&pf, 0, sizeof pf);
    ne_lock();
    if (NE.dq_dirty) { dq = 1; NE.dq_dirty = 0; copy = (nq_queue *) malloc(sizeof *copy); if (copy) memcpy(copy, &NE.dq, sizeof *copy); else NE.dq_dirty = 1; }
    if (NE.pref_dirty) { pd = 1; NE.pref_dirty = 0; pf = NE.pref; }
    ne_unlock();
    if (dq && copy) {
        if (nq_save(copy, NE.qpath) != 0) { ne_log("queue: could not write %s", NE.qpath); ne_lock(); NE.dq_dirty = 1; ne_unlock(); }
        free(copy);
    }
    if (pd) nq_prefs_save(&pf, NE.prefpath);
}

/* a change to the queue: the screen is rebuilt, and the files thread writes queue.json within a moment. The caller holds the lock. */
static void ne_dq_changed(void) { NE.dq_dirty = 1; ++NE.ver; }

static void ne_scan_installed(void) {
    nc_inst tmp[NE_INST_MAX];
    int n = nc_inst_scan(NE.mods, tmp, NE_INST_MAX);
    ne_lock();
    memcpy(NE.inst, tmp, (size_t) n * sizeof tmp[0]);
    NE.ninst = n;
    ++NE.ver;
    ne_unlock();
}

/* ---- the catalog thread ---- */

static int ne_fetch(void *u, const char *url, nc_resp *out) {
    nch_result r;
    (void) u;
    nch_get(url, NC_MAX_BODY, &r);
    out->status = r.status; out->body = (char *) r.data; out->len = r.len; out->retry_after = r.retry_after;
    if (!r.status) ne_log("net: %s", r.err[0] ? r.err : "no answer");
    return 0;
}
static int64_t ne_clock(void *u) { (void) u; return ne_now(); }
static void ne_sleep(void *u, int ms) { int waited = 0; (void) u; while (waited < ms && !NE.quit) { Sleep(100); waited += 100; } }
static int ne_stop(void *u) { (void) u; return NE.quit; }
static void ne_progress(void *u, const char *what, int done, int total) {
    (void) u;
    ne_lock();
    snprintf(NE.sync_what, sizeof NE.sync_what, "%s", what);
    NE.sync_done = done; NE.sync_total = total;
    ++NE.ver;
    ne_unlock();
}
static void ne_checkpoint(void *u) { (void) u; ne_save(); }
static void ne_io_lock(void *u) { (void) u; ne_lock(); }
static void ne_io_unlock(void *u) { (void) u; ++NE.ver; ne_unlock(); }

static DWORD WINAPI ne_sync_thread(LPVOID arg) {
    nc_sync_io io;
    (void) arg;
    memset(&io, 0, sizeof io);
    io.fetch = ne_fetch; io.now = ne_clock; io.sleep_ms = ne_sleep; io.stop = ne_stop; io.progress = ne_progress; io.checkpoint = ne_checkpoint;
    io.lock = ne_io_lock; io.unlock = ne_io_unlock;
    /* load what the last session saved. A cache from an older schema (or one with no state beside it) is NOT loaded and is deleted: it lacks fields the browser
     * and the installer rely on, and a fresh full pass replaces it. Until that pass ends, an install refetches its mod's record first (ne_refetch_mod). */
    {
        char *st = nc_read_all(NE.statepath, NULL);
        nc_catalog loaded;
        nc_sync loaded_sync;
        int n = -1, stale;
        memset(&loaded, 0, sizeof loaded);
        memset(&loaded_sync, 0, sizeof loaded_sync);
        if (st) nc_sync_from_json(&loaded_sync, st);
        stale = !st || loaded_sync.stale;
        if (stale) {
            FILE *probe = fopen(NE.catpath, "rb");
            if (probe) { fclose(probe); remove(NE.catpath); ne_log("catalog: the cache is from an older version and was discarded; syncing again"); }
        } else n = nc_cat_load(&loaded, NE.catpath);
        ne_lock();
        if (st) NE.sync = loaded_sync;
        if (n >= 0) { nc_cat_free(&NE.cat); NE.cat = loaded; ++NE.ver; }
        NE.cache_stale = stale || !loaded_sync.full;
        NE.loaded = 1;
        ne_unlock();
        free(st);
        if (n >= 0) ne_log("catalog: %d mods from the cache", n);
    }
    while (!NE.quit) {
        int want;
        ne_lock();
        want = NE.sync_want;
        NE.sync_want = 0;
        if (want) { NE.syncing = 1; NE.sync_failed = 0; NE.sync_msg[0] = '\0'; snprintf(NE.sync_what, sizeof NE.sync_what, "Contacting SSBM Nucleus"); NE.sync_done = NE.sync_total = 0; }
        ne_unlock();
        if (want) {
            nc_sync st;
            int rc;
            ne_lock(); st = NE.sync; ne_unlock();
            rc = nc_sync_run(&st, &NE.cat, &io, want == 2);
            ne_lock();
            NE.sync = st;
            NE.syncing = 0;
            if (rc < 0) { NE.sync_failed = 1; snprintf(NE.sync_msg, sizeof NE.sync_msg, "%s", st.msg); }
            else if (rc == 0) { NE.sync_ever_ok = 1; NE.sync_msg[0] = '\0'; }
            else snprintf(NE.sync_msg, sizeof NE.sync_msg, "%s", st.msg);
            if (rc >= 0 && st.full) NE.cache_stale = 0;
            ++NE.ver;
            ne_unlock();
            ne_log("sync %s: %d mods%s%s", rc == 0 ? "done" : rc > 0 ? "skipped" : "failed", NE.cat.n, st.msg[0] ? " - " : "", st.msg);
            if (rc >= 0) ne_save();
        }
        Sleep(150);
    }
    return 0;
}

/* ---- one message for the screen's corner, and the install's progress ---- */

static void ne_set_job(int state, const char *msg, int done, int total) {
    (void) done; (void) total;
    ne_lock();
    NE.job_state = state; NE.job_at = ne_now();
    snprintf(NE.job_msg, sizeof NE.job_msg, "%s", msg);
    ++NE.ver;
    ne_unlock();
}

/* the queue item being installed: a percent (not saved), from file steps done / total, or from the bytes of a download */
static void ne_item_pct(int pct) {
    ne_lock();
    if (NE.cur_id) nq_progress(&NE.dq, NE.cur_id, pct);
    ne_unlock();
}
static void ne_item_steps(int done, int total) { ne_item_pct(total > 0 ? done * 100 / (total + 1) : 0); }

typedef struct { int steps_done, steps_total; } ne_dl_ctx;
static int ne_dl_progress(void *u, size_t got, size_t expect) {
    ne_dl_ctx *c = (ne_dl_ctx *) u;
    if (NE.quit || ne_locked()) { NE.interrupt = 1; return 1; }       /* the player went online (or the game is closing): stop the download, the entry stays queued */
    if (expect > 0 && c) {
        double frac = (double) got / (double) expect;
        if (NE.cur_zip) ne_item_pct((int) (5.0 + 75.0 * frac));
        else if (c->steps_total > 0) ne_item_pct((int) (100.0 * ((double) c->steps_done + frac) / (double) (c->steps_total + 1)));
    }
    return 0;
}

/* GET for an install: reports progress, stops when the player goes online, and keeps the answer in the dl cache under `tag` (so a retry or a re-install of the
 * same version does not download, and so count, again). tag may be NULL. 0 when a 200 came back. */
static int ne_dl(const char *url, size_t max_bytes, nch_result *r, const char *tag, int steps_done, int steps_total) {
    ne_dl_ctx ctx;
    char path[MAX_PATH];
    ctx.steps_done = steps_done; ctx.steps_total = steps_total;
    memset(r, 0, sizeof *r);
    path[0] = '\0';
    if (tag) {
        size_t len = 0;
        char name[40];
        snprintf(name, sizeof name, "%016llx.bin", (unsigned long long) nq_hash(tag));
        snprintf(path, sizeof path, "%s/%s", NE.dl, name);
        nc_fix_slashes(path);
        r->data = (uint8_t *) nc_read_all(path, &len);
        if (r->data) { r->len = len; r->status = 200; nq_touch(path, 0); return 0; }
    }
    if (nch_get_p(url, max_bytes, r, ne_dl_progress, &ctx) < 0 || r->status != 200 || !r->data) return -1;
    if (tag && r->len <= 48u * 1024 * 1024) { nc_mkdirs(NE.dl); nc_write_file_atomic(path, r->data, r->len); }
    return 0;
}

static void ne_art_path(int file_id, int kind, char *out, size_t cap) { snprintf(out, cap, "%s/%d_%s.png", NE.art, file_id, kind == 1 ? "csp" : "stock"); nc_fix_slashes(out); }

/* the PNG for a file's csp (kind 1) or stock (kind 2) that an install turns into a skin's art: the cached copy or a download into the cache; a malloc'd buffer, or NULL */
static uint8_t *ne_art_png(const nc_file *f, int kind, size_t *len) {
    char path[MAX_PATH], url[600];
    uint8_t *data;
    nch_result r;
    const char *stored = kind == 1 ? f->csp : f->stock;
    if (!stored[0]) return NULL;
    ne_art_path(f->id, kind, path, sizeof path);
    data = (uint8_t *) nc_read_all(path, len);
    if (data) return data;
    nc_media_full(stored, url, sizeof url);
    if (nch_get(url, 2u * 1024 * 1024, &r) < 0 || r.status != 200 || !r.data || r.len < 16) { free(r.data); return NULL; }
    nc_mkdirs(NE.art);
    nc_write_file(path, r.data, r.len);
    *len = r.len;
    return r.data;
}

/* copy mod id (and its files) out under the lock; 0 if it is not in the catalog */
static int ne_copy_mod(int id, nc_mod *m, nc_file *files) {
    int at, ok = 0;
    ne_lock();
    at = nc_cat_find(&NE.cat, id);
    if (at >= 0) { *m = NE.cat.m[at]; memcpy(files, NE.cat.f + m->f0, (size_t) m->nf * sizeof files[0]); ok = 1; }
    ne_unlock();
    return ok;
}

static void ne_now_iso(char *out, size_t cap) { nc_iso(ne_now(), out, cap); }

/* One mod's record straight from the API (GET /mods/{id}), merged into the catalog: for an install whose mod is not in the catalog yet (the cache was discarded
 * and the first pass is still running, or the post is newer than the last poll). 1: it is in the catalog now. *status: the HTTP status (0: no answer). */
static int ne_refetch_mod(int id, int *status) {
    char url[300];
    nch_result r;
    int ok = 0;
    snprintf(url, sizeof url, "%s/mods/%d", nc_api_base, id);
    nch_get(url, NC_MAX_BODY, &r);
    *status = r.status;
    if (r.status == 200 && r.data) {
        nj_doc d;
        int root = nj_parse(&d, (const char *) r.data);
        if (root >= 0) {
            int data = nj_get(&d, root, "data");
            if (data >= 0) {
                ne_lock();
                ok = nc_cat_apply(&NE.cat, &d, data) != 0;
                ++NE.ver;
                ne_unlock();
            }
        }
        nj_free(&d);
    }
    free(r.data);
    return ok;
}

/* ---- the install (the queue's worker calls it) ---- */

static int ne_install_impl(int id, char *msg, size_t mcap, uint8_t **zipout);
/* 0 installed, -1 failed (msg says why in plain words), -2 stopped because the player went online (the entry stays queued) */
static int ne_install(int id, char *msg, size_t mcap) {
    uint8_t *z = NULL;
    int rc = ne_install_impl(id, msg, mcap, &z);
    free(z);
    return rc;
}

static int ne_install_impl(int id, char *msg, size_t mcap, uint8_t **zipout) {
    nc_mod m;
    nc_file files[NC_MAX_FILES_PER_MOD];
    int fighters[NC_NFIGHTERS], nfg = 0, g, i, installed = 0, total = 0, done = 0;
    char first_err[160];
    uint8_t *zipbuf = NULL;                 /* the mod's own zip, fetched once for all its zip-only files and fighter groups; handed to the caller to free */
    size_t ziplen = 0;
    int zip_tried = 0, zip_status = 0, use_zip;
    first_err[0] = '\0';
    if (!ne_copy_mod(id, &m, files)) { snprintf(msg, mcap, "That mod is not in the catalog."); return -1; }
    if (nc_why_not_files(&m, files)[0]) { snprintf(msg, mcap, "%s", nc_why_not_files(&m, files)); return -1; }
    for (i = 0; i < m.nf; ++i) {
        int k, seen = 0;
        if (!files[i].costume || files[i].fighter < 0 || !nc_fighters[files[i].fighter].installable) continue;
        if (nc_variant_skip(files, m.nf, i)) continue;
        for (k = 0; k < nfg; ++k) if (fighters[k] == files[i].fighter) seen = 1;
        if (!seen) fighters[nfg++] = files[i].fighter;
        total += 3;
    }
    for (g = 0; g < nfg; ++g) {
        nc_skin_mod *sm = (nc_skin_mod *) calloc(1, sizeof *sm);
        char stage[MAX_PATH], files_dir[MAX_PATH], final_dir[MAX_PATH], tmp[MAX_PATH];
        int in_group = 0, n = 0;
        nj_buf b;
        if (!sm) { snprintf(msg, mcap, "Out of memory."); return -1; }
        for (i = 0; i < m.nf; ++i) if (files[i].costume && files[i].fighter == fighters[g] && !nc_variant_skip(files, m.nf, i)) ++in_group;
        nc_mod_id(m.id, fighters[g], sm->mod_id, sizeof sm->mod_id);
        sm->nucleus_id = m.id;
        snprintf(sm->title, sizeof sm->title, "%s", m.title);
        snprintf(sm->author, sizeof sm->author, "%s", m.author);
        snprintf(sm->page, sizeof sm->page, "%s", m.page);
        snprintf(sm->updated, sizeof sm->updated, "%s", m.updated);
        snprintf(sm->target, sizeof sm->target, "%s", nc_fighters[fighters[g]].target);
        ne_now_iso(sm->installed_at, sizeof sm->installed_at);
        snprintf(stage, sizeof stage, "%s/.staging-%s", NE.mods, sm->mod_id);
        snprintf(final_dir, sizeof final_dir, "%s/%s", NE.mods, sm->mod_id);
        snprintf(files_dir, sizeof files_dir, "%s/files/skins/%s", stage, sm->mod_id);
        nc_fix_slashes(stage); nc_fix_slashes(files_dir); nc_fix_slashes(final_dir);
        nc_rmtree(stage);
        if (nc_mkdirs(files_dir) != 0) { snprintf(msg, mcap, "Cannot write to the mods folder."); free(sm); return -1; }
        for (i = 0; i < m.nf && n < NC_MAX_FILES_PER_MOD; ++i) {
            nch_result r;
            nc_dat_info info;
            char url[400], err[120];
            nc_skin_costume *c;
            uint8_t *png, *gx;
            size_t pl, gl;
            if (!files[i].costume || files[i].fighter != fighters[g]) continue;
            if (nc_variant_skip(files, m.nf, i)) { ne_log("install %d: file %d (%s): the Animelee variant is skipped, the vanilla one is installed", m.id, files[i].id, files[i].filename); continue; }
            ne_item_steps(done, total);
            use_zip = files[i].zip;
            if (!use_zip) {
                char tag[96];
                nc_download_url(m.id, files[i].id, url, sizeof url);
                snprintf(tag, sizeof tag, "f%d_%d_%s", m.id, files[i].id, m.updated);
                if (ne_dl(url, 24u * 1024 * 1024, &r, tag, done, total) < 0) {
                    if (NE.interrupt) { free(r.data); nc_rmtree(stage); free(sm); snprintf(msg, mcap, "Paused."); return -2; }
                    ne_log("install %d: file %d: %s (HTTP %d)", m.id, files[i].id, r.err[0] ? r.err : "download failed", r.status);
                    free(r.data); memset(&r, 0, sizeof r);
                    use_zip = 1;   /* no per-file download (a stale catalog, or the post lists it wrong): the file may still be in the post's zip */
                }
            }
            if (use_zip) {
                /* no per-file download: the file is an entry of the mod's zip (GET /mods/{id}/download), fetched once */
                memset(&r, 0, sizeof r);
                if (!zip_tried) {
                    nch_result zr;
                    char ztag[96];
                    zip_tried = 1;
                    nc_download_url(m.id, 0, url, sizeof url);
                    snprintf(ztag, sizeof ztag, "z%d_%s", m.id, m.updated);
                    NE.cur_zip = 1;
                    if (ne_dl(url, 64u * 1024 * 1024, &zr, ztag, 0, total) < 0) {
                        NE.cur_zip = 0;
                        if (NE.interrupt) { free(zr.data); nc_rmtree(stage); free(sm); snprintf(msg, mcap, "Paused."); return -2; }
                        zip_status = zr.status ? zr.status : -1;
                        ne_log("install %d: zip download: %s (HTTP %d)", m.id, zr.err[0] ? zr.err : "failed", zr.status);
                        free(zr.data);
                    } else { NE.cur_zip = 0; zipbuf = zr.data; ziplen = zr.len; *zipout = zipbuf; ne_item_pct(80); }
                }
                if (!zipbuf) {
                    if (!first_err[0]) snprintf(first_err, sizeof first_err, "%s", nc_http_reason(zip_status < 0 ? 0 : zip_status));
                    done += 3; continue;
                }
                if (nc_zip_extract(zipbuf, ziplen, files[i].filename, &r.data, &r.len, 24u * 1024 * 1024, err, sizeof err) < 0) {
                    if (!first_err[0]) snprintf(first_err, sizeof first_err, "Could not unpack the post's zip: %s.", err);
                    ne_log("install %d: file %d (%s): %s", m.id, files[i].id, files[i].filename, err);
                    done += 3; continue;
                }
            }
            ++done;
            if (nc_dat_costume(r.data, r.len, &info, err, sizeof err) < 0) {
                if (!first_err[0]) snprintf(first_err, sizeof first_err, "Not a plain costume file: %s.", err);
                ne_log("install %d: file %d (%s): %s", m.id, files[i].id, files[i].filename, err);
                free(r.data); done += 2; continue;
            }
            if (info.fighter != fighters[g]) {
                if (!first_err[0]) snprintf(first_err, sizeof first_err, "A file is for %s, not %s.", info.token, nc_fighters[fighters[g]].label);
                ne_log("install %d: file %d is a %s costume but the post lists it for %s - skipped", m.id, files[i].id, info.token, nc_fighters[fighters[g]].label);
                free(r.data); done += 2; continue;
            }
            if (files[i].mismatch) ne_log("install %d: file %d: the API's character / colour disagree with the file name (%s) - using the file's own symbols", m.id, files[i].id, files[i].filename);
            c = &sm->c[n];
            nc_costume_name(m.title, files[i].color, in_group > 1, c->name, sizeof c->name);
            snprintf(c->dat, sizeof c->dat, "skins/%s/%d.dat", sm->mod_id, n + 1);
            snprintf(tmp, sizeof tmp, "%s/%d.dat", files_dir, n + 1); nc_fix_slashes(tmp);
            if (nc_write_file(tmp, r.data, r.len) != 0) { free(r.data); nc_rmtree(stage); snprintf(msg, mcap, "Cannot write the costume file."); free(sm); return -1; }
            free(r.data);
            snprintf(c->joint, sizeof c->joint, "%s", info.joint);
            snprintf(c->matanim, sizeof c->matanim, "%s", info.matanim);
            c->like = info.colour_idx > 0 ? info.colour_idx : 0;
            c->file_id = files[i].id;
            {   /* the portrait, then the stock icon: art that cannot be had or converted just leaves the default */
            int kind;
            for (kind = 1; kind <= 2; ++kind) {
                png = ne_art_png(&files[i], kind, &pl);
                ++done;
                if (!png) continue;
                if (nc_png_to_gxtex(png, pl, &gx, &gl, err, sizeof err) == 0) {
                    char *dst = kind == 1 ? c->csp : c->stock;
                    snprintf(dst, 160, "skins/%s/%d_%s.gxtex", sm->mod_id, n + 1, kind == 1 ? "csp" : "stock");
                    snprintf(tmp, sizeof tmp, "%s/%d_%s.gxtex", files_dir, n + 1, kind == 1 ? "csp" : "stock"); nc_fix_slashes(tmp);
                    if (nc_write_file(tmp, gx, gl) != 0) dst[0] = '\0';
                    free(gx);
                } else ne_log("install %d: file %d: %s art: %s - default art", m.id, files[i].id, kind == 1 ? "portrait" : "stock icon", err);
                free(png);
            }
            }
            ++n;
        }
        if (n == 0) {
            nc_rmtree(stage);
            snprintf(msg, mcap, "%s", first_err[0] ? first_err : "Nothing in this post could be installed.");
            free(sm);
            return -1;
        }
        sm->n = n;
        memset(&b, 0, sizeof b);
        nc_skin_json(sm, &b);
        snprintf(tmp, sizeof tmp, "%s/mod.json", stage); nc_fix_slashes(tmp);
        if (b.bad || nc_write_file(tmp, b.s, b.n) != 0) { free(b.s); nc_rmtree(stage); snprintf(msg, mcap, "Cannot write mod.json."); free(sm); return -1; }
        free(b.s);
        if (nc_swap_dir(stage, final_dir) != 0) {
            nc_rmtree(stage);
            snprintf(msg, mcap, "Could not replace %s (the game may be using it). Restart and try again.", sm->mod_id);
            free(sm);
            return -1;
        }
        nc_enabled_edit(NE.mods, sm->mod_id, 1);
        ne_log("install %d: %s: %d costume(s) for %s", m.id, sm->mod_id, n, nc_fighters[fighters[g]].label);
        installed += n;
        free(sm);
    }
    snprintf(msg, mcap, "Installed %d costume%s. Restart the game to use %s.", installed, installed == 1 ? "" : "s", installed == 1 ? "it" : "them");
    return installed > 0 ? 0 : -1;
}


static int ne_uninstall(int id, char *msg, size_t mcap) {
    int i, removed = 0;
    nc_inst list[NE_INST_MAX];
    int n;
    ne_lock(); n = NE.ninst; memcpy(list, NE.inst, (size_t) n * sizeof list[0]); ne_unlock();
    for (i = 0; i < n; ++i) {
        char dir[MAX_PATH];
        if (list[i].id != id) continue;
        snprintf(dir, sizeof dir, "%s/%s", NE.mods, list[i].folder);
        nc_fix_slashes(dir);
        nc_rmtree(dir);
        nc_enabled_edit(NE.mods, list[i].folder, 0);
        if (!nc_is_dir(dir)) ++removed;
        ne_log("uninstall %d: %s%s", id, list[i].folder, nc_is_dir(dir) ? " could not be fully removed" : "");
    }
    if (!removed) { snprintf(msg, mcap, "Could not remove it (the game may be using the files). Restart and try again."); return -1; }
    snprintf(msg, mcap, "Removed. The change shows after a restart.");
    return 0;
}

/* ---- the download queue's worker ---- */

/* Take the first queued mod and install it. Returns 1 when it did something (so the loop asks again at once), 0 when there was nothing to do now. */
static int ne_queue_step(void) {
    int idx, id, in_cat, rc, status = 0, refetched = 0;
    char title[80], msg[200];
    if (ne_locked()) return 0;                                  /* online: the downloads wait (the screen says so) */
    ne_lock();
    if (!NE.dq.processing) { ne_unlock(); return 0; }
    idx = nq_next(&NE.dq);
    if (idx < 0) {                                              /* nothing left to do: the run is over */
        if (nq_pending(&NE.dq) == 0) { NE.dq.processing = 0; ne_dq_changed(); }
        ne_unlock();
        return 0;
    }
    id = NE.dq.it[idx].id;
    snprintf(title, sizeof title, "%s", NE.dq.it[idx].title);
    in_cat = nc_cat_find(&NE.cat, id) >= 0;
    nq_mark_running(&NE.dq, id);
    NE.cur_id = id; NE.cur_zip = 0; NE.interrupt = 0;
    ne_dq_changed();
    ne_unlock();
    /* the mod's record: from the catalog, else (or when the catalog is an older cache not yet replaced) straight from the API */
    if (!in_cat || NE.cache_stale) {
        refetched = ne_refetch_mod(id, &status);
        if (!refetched && !in_cat) {
            snprintf(msg, sizeof msg, "%s", status == 404 ? "It is not on SSBM Nucleus any more." : status == 0 ? "SSBM Nucleus could not be reached. Check the connection and retry." : nc_http_reason(status));
            rc = -1;
            goto finished;
        }
    }
    rc = ne_install(id, msg, sizeof msg);
finished:
    ne_scan_installed();
    ne_lock();
    NE.cur_id = 0;
    if (rc == -2 || NE.interrupt) {                             /* the player went online or the game is closing: back in line, nothing failed */
        int at = nq_find(&NE.dq, id);
        if (at >= 0) { NE.dq.it[at].state = NQ_QUEUED; NE.dq.it[at].pct = 0; }
        ne_dq_changed();
        ne_unlock();
        ne_log("queue: %d (%s) paused: %s", id, title, NE.quit ? "closing" : "online");
        return 0;
    }
    nq_finish(&NE.dq, id, rc == 0, rc == 0 ? "" : msg);
    if (rc == 0) NE.restart = 1;
    if (nq_next(&NE.dq) < 0) NE.dq.processing = 0;              /* the last one: the run is over (failed entries stay for a retry) */
    ne_dq_changed();
    ne_unlock();
    if (rc == 0) { char m2[220]; snprintf(m2, sizeof m2, "%s: %s", title, msg); ne_set_job(2, m2, 1, 1); }
    else { char m2[220]; snprintf(m2, sizeof m2, "%s failed: %s", title, msg); ne_set_job(3, m2, 1, 1); }
    ne_log("queue: %d (%s) %s: %s", id, title, rc == 0 ? "installed" : "failed", msg);
    return 1;
}

static DWORD WINAPI ne_files_thread(LPVOID arg) {
    (void) arg;
    nc_mkdirs(NE.art);
    nc_mkdirs(NE.thumbs);
    while (!NE.quit) {
        ne_job j;
        int have = 0;
        ne_save_dirty();
        ne_lock();
        if (NE.qh != NE.qt) { j = NE.q[NE.qh]; NE.qh = (NE.qh + 1) % NE_QUEUE; have = 1; }
        ne_unlock();
        if (have) {
            char msg[200];
            int rc = ne_uninstall(j.id, msg, sizeof msg);
            ne_scan_installed();
            if (rc == 0) { ne_lock(); NE.restart = 1; ne_unlock(); }
            ne_set_job(rc == 0 ? 2 : 3, msg, 1, 1);
            ne_log("uninstall %d: %s", j.id, msg);
            continue;
        }
        if (ne_queue_step()) continue;
        Sleep(60);
    }
    ne_save_dirty();
    return 0;
}

static int ne_enqueue(ne_job j) {
    int ok = 0;
    ne_lock();
    if ((NE.qt + 1) % NE_QUEUE != NE.qh) { NE.q[NE.qt] = j; NE.qt = (NE.qt + 1) % NE_QUEUE; ok = 1; }
    ne_unlock();
    return ok;
}

/* ---- pictures ---- */

static uint64_t ne_pic_key(const char *url, int kind) { return nq_hash(url) ^ ((uint64_t) (kind + 1) * 0x9E3779B97F4A7C15ull); }

/* The game thread, for a row in view: the texture slot of the picture at `stored` (a media path or URL), -1 while it is not ready. Records a want the first time.
 * Returns 1 when there is a picture to wait for or show, 0 when the source is empty or failed for good. */
static int ne_pic_get(const char *stored, int kind, int *tex) {
    char url[300], texkey[40];
    uint64_t key;
    int i, free_at = -1, lru_at = -1, ok = 1;
    unsigned lru = 0xFFFFFFFFu;
    *tex = -1;
    if (!stored || !stored[0]) return 0;
    nc_media_full(stored, url, sizeof url);
    key = ne_pic_key(url, kind);
    ne_lock();
    for (i = 0; i < NE_PICS; ++i) {
        ne_pic *p = &NE.pics[i];
        if (p->state == NE_PIC_FREE) { if (free_at < 0) free_at = i; continue; }
        if (p->key == key) {
            p->last = NE.tick;
            if (p->state == NE_PIC_READY) {
                snprintf(texkey, sizeof texkey, "nc:p:%016llx", (unsigned long long) key);
                ne_unlock();
                *tex = gw_Kit_TexHsdFind(texkey);
                if (*tex < 0) {                                 /* never uploaded, or the kit evicted it: upload from the blob, within the frame's budget */
                    ne_lock();
                    if (p->state == NE_PIC_READY && p->key == key && NE.uploads < NE_UPLOADS_PER_FRAME) {
                        ++NE.uploads;
                        *tex = gw_Kit_TexAddGxtex(texkey, p->blob, p->blen);
                    }
                    ne_unlock();
                }
                return 1;
            }
            if (p->state == NE_PIC_FAILED) {
                if (ne_now() >= p->retry_at) { p->state = NE_PIC_WANT; p->last = NE.tick; }
                else ok = 0;
            }
            ne_unlock();
            return ok;
        }
        if (p->state != NE_PIC_BUSY && p->last < lru) { lru = p->last; lru_at = i; }
    }
    if (free_at < 0) {                                          /* full: the least recently asked entry makes room */
        if (lru_at < 0) { ne_unlock(); return 1; }
        free(NE.pics[lru_at].blob);
        memset(&NE.pics[lru_at], 0, sizeof NE.pics[lru_at]);
        free_at = lru_at;
    }
    {
        ne_pic *p = &NE.pics[free_at];
        memset(p, 0, sizeof *p);
        p->key = key; p->kind = kind; p->state = NE_PIC_WANT; p->last = NE.tick;
        snprintf(p->url, sizeof p->url, "%s", url);
    }
    ne_unlock();
    return 1;
}

/* the picture thread's work for one want: the bytes (disk cache or download), then the decode into a blob. 0 ok. *status: the HTTP status when a download failed. */
static int ne_pic_work(const char *url, int kind, uint8_t **blob, size_t *blen, int *status) {
    char path[MAX_PATH], name[40], err[100];
    size_t len = 0;
    uint8_t *data;
    int from_disk = 0, rc;
    nq_cache_name(url, name, sizeof name);
    snprintf(path, sizeof path, "%s/%s", NE.thumbs, name);
    nc_fix_slashes(path);
    data = (uint8_t *) nc_read_all(path, &len);
    if (data) { from_disk = 1; nq_touch(path, 0); }
    else {
        nch_result r;
        if (ne_locked()) return -2;                              /* online: no downloads (the want comes back after the cooldown) */
        nch_get(url, NI_MAX_BYTES, &r);
        *status = r.status;
        if (r.status != 200 || !r.data || r.len < 16) { free(r.data); return -1; }
        data = r.data; len = r.len;
    }
    rc = ni_make(data, len, kind, blob, blen, err, sizeof err);
    if (rc < 0) {
        ne_log("picture %s: %s", url, err);
        if (from_disk) remove(path);                             /* a cached file that does not decode is not kept */
        free(data);
        return -3;
    }
    if (!from_disk) {
        int n;
        nc_mkdirs(NE.thumbs);
        nc_write_file_atomic(path, data, len);
        ne_lock(); n = ++NE.img_writes; ne_unlock();
        if (n % 25 == 0) nq_cache_trim(NE.thumbs, NE.cache_cap);
    }
    free(data);
    return 0;
}

static DWORD WINAPI ne_img_thread(LPVOID arg) {
    (void) arg;
    nc_mkdirs(NE.thumbs);
    while (!NE.quit) {
        int pick = -1, i, status = 0, rc;
        unsigned best = 0;
        char url[260];
        int kind = 0;
        uint64_t key = 0;
        uint8_t *blob = NULL;
        size_t blen = 0;
        ne_lock();
        for (i = 0; i < NE_PICS; ++i) {
            ne_pic *p = &NE.pics[i];
            if (p->state != NE_PIC_WANT) continue;
            if (NE.tick - p->last > NE_WANT_TICKS) { memset(p, 0, sizeof *p); continue; }       /* the row scrolled away */
            if (pick < 0 || p->last >= best) { pick = i; best = p->last; }
        }
        if (pick >= 0) { NE.pics[pick].state = NE_PIC_BUSY; snprintf(url, sizeof url, "%s", NE.pics[pick].url); kind = NE.pics[pick].kind; key = NE.pics[pick].key; }
        ne_unlock();
        if (pick < 0) { Sleep(40); continue; }
        rc = ne_pic_work(url, kind, &blob, &blen, &status);
        ne_lock();
        {
            ne_pic *p = &NE.pics[pick];
            if (p->key == key && p->state == NE_PIC_BUSY) {
                if (rc == 0) { p->blob = blob; p->blen = blen; p->state = NE_PIC_READY; blob = NULL; }
                else {
                    p->state = NE_PIC_FAILED;
                    p->retry_at = ne_now() + (rc == -2 ? 20 : status == 404 ? 3600 : 120);
                }
            }
        }
        ne_unlock();
        free(blob);
    }
    return 0;
}

/* ---- the screen's source (the game thread) ---- */

/* the game thread: start the threads once, and ask for a sync (the sync itself keeps the 10-minute rule) */
static void ne_open(void) {
    if (!NE.inited) {
        const char *api = getenv("MELEE_NUCLEUS_API");
        DWORD tid;
        int nloaded, cleaned;
        memset(&NE, 0, sizeof NE);
        InitializeCriticalSection(&NE.lock);
        if (api && api[0] && strlen(api) < sizeof nc_api_base - 1) { snprintf(nc_api_base, sizeof nc_api_base, "%s", api); nch_set_override(api); }
        ne_paths();
        nc_mkdirs(NE.dir);
        nc_mkdirs(NE.art);
        nc_mkdirs(NE.thumbs);
        NE.inited = 1;
        cleaned = nq_clean_staging(NE.mods);                    /* an install a crash cut short left only a .staging-* folder */
        nloaded = nq_load(&NE.dq, NE.qpath);                    /* the queue of the last session; a RUNNING entry is QUEUED again */
        nq_prefs_load(&NE.pref, NE.prefpath);
        nq_cache_trim(NE.thumbs, NE.cache_cap);
        nq_cache_trim(NE.dl, NE_DL_CAP);
        ne_scan_installed();
        NE.th_sync = CreateThread(NULL, 0, ne_sync_thread, NULL, 0, &tid);
        NE.th_files = CreateThread(NULL, 0, ne_files_thread, NULL, 0, &tid);
        NE.th_img[0] = CreateThread(NULL, 0, ne_img_thread, NULL, 0, &tid);
        NE.th_img[1] = CreateThread(NULL, 0, ne_img_thread, NULL, 0, &tid);
        gw_log("nucleus: browser started; cache %s, api %s", NE.dir, nc_api_base);
        gw_log("nucleus: queue: %d entr%s restored%s%s%s", nloaded < 0 ? 0 : nloaded, nloaded == 1 ? "y" : "ies", nloaded < 0 ? " (queue.json was unreadable and was set aside as queue.json.bad)" : "",
               NE.dq.processing && nq_pending(&NE.dq) > 0 ? "; downloads were running and resume now" : "", cleaned ? "; removed an unfinished install folder" : "");
    }
    if (!gs_ui_netplay()) { ne_lock(); if (!NE.sync_want && !NE.syncing) NE.sync_want = 1; ne_unlock(); }
}

static void ne_fill_row_from_mod(const nc_mod *m, const nc_file *files, AtNcRow *r) {
    int i, fighter = -1;
    memset(r, 0, sizeof *r);
    r->tex = -1; r->thumb = -1; r->qstate = AT_NC_Q_NONE;
    r->id = m->id;
    snprintf(r->title, sizeof r->title, "%s", m->title);
    snprintf(r->author, sizeof r->author, "%s", m->author);
    snprintf(r->kind, sizeof r->kind, "%s", nc_type_label[m->type]);
    snprintf(r->updated, sizeof r->updated, "%.10s", m->updated);
    snprintf(r->page, sizeof r->page, "%s", m->page);
    snprintf(r->desc, sizeof r->desc, "%s", m->desc);
    snprintf(r->tags, sizeof r->tags, "%s", m->tags);
    r->downloads = m->downloads; r->likes = m->likes;
    r->nshots = m->nshots;
    for (i = 0; i < m->nf; ++i) {
        if (files[i].costume) ++r->ncostumes;
        if (fighter < 0 && files[i].fighter >= 0) fighter = files[i].fighter;
    }
    r->files = m->nf_total;
    if (fighter >= 0) snprintf(r->fighter, sizeof r->fighter, "%s", nc_fighters[fighter].label);
}

static int ne_src_row(void *u, int id, AtNcRow *r) {
    nc_mod m;
    nc_file files[NC_MAX_FILES_PER_MOD];
    int found, i, inst = -1, qi;
    const char *why = "";
    (void) u;
    found = ne_copy_mod(id, &m, files);
    ne_lock();
    for (i = 0; i < NE.ninst; ++i) if (NE.inst[i].id == id) { inst = i; break; }
    qi = nq_find(&NE.dq, id);
    if (found) {
        ne_fill_row_from_mod(&m, files, r);
        why = nc_why_not_files(&m, files);
        r->installable = why[0] == '\0';
        snprintf(r->reason, sizeof r->reason, "%s", why);
    } else if (inst >= 0) {
        memset(r, 0, sizeof *r);
        r->tex = -1; r->thumb = -1; r->qstate = AT_NC_Q_NONE; r->id = id;
        snprintf(r->title, sizeof r->title, "%s", NE.inst[inst].name);
        snprintf(r->author, sizeof r->author, "%s", NE.inst[inst].author);
        snprintf(r->kind, sizeof r->kind, "%s", "Costume");
        snprintf(r->updated, sizeof r->updated, "%.10s", NE.inst[inst].updated);
        {
            int f;
            for (f = 0; f < NC_NFIGHTERS; ++f) if (!strcmp(nc_fighters[f].target, NE.inst[inst].target)) snprintf(r->fighter, sizeof r->fighter, "%s", nc_fighters[f].label);
        }
        r->ncostumes = NE.inst[inst].ncost;
        snprintf(r->page, sizeof r->page, "https://%s/post/%d", NC_SITE, id);
    } else if (qi >= 0) {                                       /* a queued mod the catalog has not loaded yet: what the queue remembers of it */
        memset(r, 0, sizeof *r);
        r->tex = -1; r->thumb = -1; r->id = id;
        snprintf(r->title, sizeof r->title, "%s", NE.dq.it[qi].title);
        snprintf(r->author, sizeof r->author, "%s", NE.dq.it[qi].author);
        snprintf(r->kind, sizeof r->kind, "%s", "Mod");
        snprintf(r->page, sizeof r->page, "https://%s/post/%d", NC_SITE, id);
        r->installable = 1;
    } else { ne_unlock(); return 0; }
    if (inst >= 0) {
        int gone = nc_cat_is_gone(&NE.cat, id) || (!found && NE.sync.full);
        r->state = gone ? AT_NC_ST_REMOVED : (found && strcmp(NE.inst[inst].updated, m.updated) != 0) ? AT_NC_ST_UPDATE : AT_NC_ST_INSTALLED;
    } else r->state = r->installable ? AT_NC_ST_NONE : AT_NC_ST_UNSUPPORTED;
    r->qstate = AT_NC_Q_NONE;
    if (qi >= 0) {
        r->qstate = NE.dq.it[qi].state; r->qpct = NE.dq.it[qi].pct; r->qplace = nq_place(&NE.dq, id);
        snprintf(r->qreason, sizeof r->qreason, "%s", NE.dq.it[qi].reason);
    }
    ne_unlock();
    return 1;
}

/* the list picture, a screenshot (1..3) or a costume's stock icon (10) of a mod: asks the picture cache (ne_pic_get) */
static int ne_src_pic(void *u, int id, int which, int *tex) {
    nc_mod m;
    nc_file files[NC_MAX_FILES_PER_MOD];
    const char *stored = "";
    int i, kind = NI_THUMB;
    (void) u;
    *tex = -1;
    if (!ne_copy_mod(id, &m, files)) return 0;
    if (which == AT_NC_PIC_STOCK) {
        kind = NI_ICON;
        for (i = 0; i < m.nf; ++i) if (files[i].costume && files[i].stock[0]) { stored = files[i].stock; break; }
    } else if (which >= AT_NC_PIC_SHOT1) {
        kind = NI_SHOT;
        if (which - AT_NC_PIC_SHOT1 < m.nshots) stored = m.shot[which - AT_NC_PIC_SHOT1];
    } else {
        for (i = 0; i < m.nf; ++i) if (files[i].costume && files[i].csp[0]) { stored = files[i].csp; break; }       /* a costume: its portrait */
        if (!stored[0] && m.nshots > 0) stored = m.shot[0];       /* any other kind: its first screenshot (the API's own thumbnails are WebP and are not used) */
    }
    return ne_pic_get(stored, kind, tex);
}

typedef struct { int id; char sort_title[80]; } ne_inst_key;
static int ne_cmp_inst(const void *a, const void *b) { return strcmp(((const ne_inst_key *) a)->sort_title, ((const ne_inst_key *) b)->sort_title); }

/* does the "Show" filter keep this mod? The caller holds the lock. */
static int ne_show_keeps(int show, const nc_mod *m) {
    int i, inst = 0, update = 0, queued = 0, q;
    if (show == AT_NC_SHOW_ALL) return 1;
    for (i = 0; i < NE.ninst; ++i) if (NE.inst[i].id == m->id) { inst = 1; if (strcmp(NE.inst[i].updated, m->updated) != 0) update = 1; break; }
    q = nq_find(&NE.dq, m->id);
    queued = q >= 0 && NE.dq.it[q].state != NQ_DONE;
    switch (show) {
    case AT_NC_SHOW_NOT_INSTALLED: return !inst;
    case AT_NC_SHOW_INSTALLED: return inst;
    case AT_NC_SHOW_UPDATE: return update;
    case AT_NC_SHOW_QUEUED: return queued;
    default: return 1;
    }
}

static int ne_src_filter(void *u, const AtNcFilter *f, int *ids, int cap, unsigned *version) {
    int n = 0, i;
    (void) u;
    ne_lock();
    *version = NE.ver ? NE.ver : 1;
    if (f->view == AT_NC_VIEW_QUEUE) {
        for (i = 0; i < NE.dq.n && n < cap; ++i) ids[n++] = NE.dq.it[i].id;
    } else if (f->view == AT_NC_VIEW_INSTALLED) {
        ne_inst_key keys[NE_INST_MAX];
        int nk = 0, k;
        for (i = 0; i < NE.ninst && nk < NE_INST_MAX; ++i) {
            int dup = 0;
            for (k = 0; k < nk; ++k) if (keys[k].id == NE.inst[i].id) dup = 1;
            if (dup) continue;
            if (f->text[0] && !nc_icontains(NE.inst[i].name, f->text) && !nc_icontains(NE.inst[i].author, f->text)) continue;
            if (f->fighter >= 0 && strcmp(nc_fighters[f->fighter].target, NE.inst[i].target) != 0) continue;
            keys[nk].id = NE.inst[i].id; snprintf(keys[nk].sort_title, sizeof keys[nk].sort_title, "%s", NE.inst[i].name);
            ++nk;
        }
        qsort(keys, (size_t) nk, sizeof keys[0], ne_cmp_inst);
        for (k = 0; k < nk && n < cap; ++k) ids[n++] = keys[k].id;
    } else {
        nc_query q;
        int *idx = (int *) malloc((size_t) (NE.cat.n > 0 ? NE.cat.n : 1) * sizeof(int));
        nc_query_init(&q);
        snprintf(q.text, sizeof q.text, "%s", f->text);
        q.type_mask = f->type_mask; q.fighter = f->fighter; q.sort = f->sort;
        if (idx) {
            int m = nc_cat_filter(&NE.cat, &q, idx, NE.cat.n);
            for (i = 0; i < m && n < cap; ++i) if (ne_show_keeps(f->show, &NE.cat.m[idx[i]])) ids[n++] = NE.cat.m[idx[i]].id;
            free(idx);
        }
    }
    ne_unlock();
    return n;
}

static unsigned ne_src_version(void *u) { unsigned v; (void) u; ne_lock(); v = NE.ver ? NE.ver : 1; ne_unlock(); return v; }

static void ne_src_status(void *u, AtNcStatus *s) {
    int at;
    (void) u;
    memset(s, 0, sizeof *s);
    ne_lock();
    s->syncing = NE.syncing; s->done = NE.sync_done; s->total = NE.sync_total;
    s->catalog_n = NE.cat.n; s->restart = NE.restart; s->locked = ne_locked();
    s->job_state = (NE.job_state == 1 || ne_now() - NE.job_at < 7) ? NE.job_state : 0;     /* a finished job's message shows for a few seconds */
    snprintf(s->job_msg, sizeof s->job_msg, "%s", NE.job_msg);
    snprintf(s->line, sizeof s->line, "%s", NE.syncing ? NE.sync_what : NE.sync_msg);
    s->offline = NE.sync_failed;
    s->q_pending = nq_pending(&NE.dq);
    s->q_failed = nq_count(&NE.dq, 1u << NQ_FAILED);
    s->q_processing = NE.dq.processing;
    s->q_running_pct = -1;
    at = NE.cur_id ? nq_find(&NE.dq, NE.cur_id) : -1;
    if (at >= 0 && NE.dq.it[at].state == NQ_RUNNING) { s->q_running_pct = NE.dq.it[at].pct; snprintf(s->q_running_title, sizeof s->q_running_title, "%s", NE.dq.it[at].title); }
    ne_unlock();
}

/* opens a page of ssbmnucleus.net in the player's browser. Only that site's own address is ever opened. */
static void ne_open_page(const char *url) {
#ifdef _WIN32
    typedef HINSTANCE (WINAPI *PFN_Shell)(HWND, LPCSTR, LPCSTR, LPCSTR, LPCSTR, INT);
    HMODULE sh;
    PFN_Shell fn;
    if (strncmp(url, "https://ssbmnucleus.net/", 24) != 0 && strncmp(url, "https://www.ssbmnucleus.net/", 28) != 0) { gw_log("nucleus: not opening %s", url); return; }
    sh = LoadLibraryA("shell32.dll");
    fn = sh ? (PFN_Shell) GetProcAddress(sh, "ShellExecuteA") : NULL;
    if (fn) fn(NULL, "open", url, NULL, NULL, 1);
#else
    char cmd[400];
    if (strncmp(url, "https://ssbmnucleus.net/", 24) != 0 || strchr(url, '\'') || strlen(url) > 300) return;
    snprintf(cmd, sizeof cmd, "xdg-open '%s' >/dev/null 2>&1 &", url);
    (void) !system(cmd);
#endif
}

static void ne_say(int kind, const char *text) { ne_set_job(kind, text, 0, 0); }

/* the queue actions (AtNcSrc.act). They only change the queue under the lock and mark it dirty: the files thread writes it and does the work. */
static int ne_queue_act(int id, int what) {
    char msg[200];
    int rc = 1;
    ne_lock();
    switch (what) {
    case AT_NC_ACT_QUEUE: {
        nc_mod m;
        int at = nc_cat_find(&NE.cat, id), qi = nq_find(&NE.dq, id), r;
        const char *title = "", *author = "";
        if (at >= 0) {
            m = NE.cat.m[at];
            if (nc_why_not_files(&m, NE.cat.f + m.f0)[0] && (qi < 0)) { snprintf(msg, sizeof msg, "%s", nc_why_not_files(&m, NE.cat.f + m.f0)); ne_unlock(); ne_say(3, msg); return 0; }
            title = NE.cat.m[at].title; author = NE.cat.m[at].author;
        } else if (qi >= 0) { title = NE.dq.it[qi].title; author = NE.dq.it[qi].author; }
        r = nq_toggle(&NE.dq, id, title, author, ne_now());
        if (r > 0) snprintf(msg, sizeof msg, "Queued (%d waiting). Press Start to download.", nq_pending(&NE.dq));
        else if (r < 0) snprintf(msg, sizeof msg, "Taken out of the queue (%d waiting).", nq_pending(&NE.dq));
        else { snprintf(msg, sizeof msg, "%s", nq_find(&NE.dq, id) >= 0 ? "It is being installed right now." : "The queue is full."); rc = 0; }
        if (r != 0) ne_dq_changed();
        break;
    }
    case AT_NC_ACT_START:
        if (NE.dq.processing) { NE.dq.processing = 0; snprintf(msg, sizeof msg, "Paused. The mod being installed finishes first."); }
        else if (nq_pending(&NE.dq) == 0) { snprintf(msg, sizeof msg, "The queue is empty. Press A on a mod to add it."); rc = 0; }
        else {
            NE.dq.processing = 1;
            snprintf(msg, sizeof msg, ne_locked() ? "Downloads start when you are offline." : "Downloading %d mod%s.", nq_pending(&NE.dq), nq_pending(&NE.dq) == 1 ? "" : "s");
        }
        if (rc) ne_dq_changed();
        break;
    case AT_NC_ACT_Q_UP: case AT_NC_ACT_Q_DOWN:
        rc = nq_move(&NE.dq, id, what == AT_NC_ACT_Q_UP ? -1 : 1);
        msg[0] = '\0';
        if (rc) ne_dq_changed();
        break;
    case AT_NC_ACT_Q_REMOVE:
        rc = nq_remove(&NE.dq, id);
        snprintf(msg, sizeof msg, rc ? "Taken out of the queue." : "It is being installed right now.");
        if (rc) ne_dq_changed();
        break;
    case AT_NC_ACT_Q_RETRY:
        rc = nq_retry(&NE.dq, id);
        snprintf(msg, sizeof msg, "%s", rc ? (NE.dq.processing ? "Trying again." : "Back in the queue. Press Start to download.") : "Nothing to retry.");
        if (rc) { if (nq_pending(&NE.dq) > 0 && !NE.dq.processing) NE.dq.processing = 1; ne_dq_changed(); }
        break;
    case AT_NC_ACT_Q_CLEAR: {
        int n = nq_clear_pending(&NE.dq) + nq_clear_finished(&NE.dq);
        snprintf(msg, sizeof msg, "Queue cleared (%d removed).", n);
        if (nq_pending(&NE.dq) == 0) NE.dq.processing = 0;
        ne_dq_changed();
        break;
    }
    case AT_NC_ACT_Q_TIDY:
        nq_clear_finished(&NE.dq);
        msg[0] = '\0';
        ne_dq_changed();
        break;
    default: msg[0] = '\0'; rc = 0; break;
    }
    ne_unlock();
    if (msg[0]) ne_say(rc ? 2 : 3, msg);
    return rc;
}

static int ne_src_act(void *u, int id, int what) {
    ne_job j;
    (void) u;
    if (what == AT_NC_ACT_PAGE) {
        AtNcRow r;
        char url[200];
        if (ne_src_row(NULL, id, &r) && r.page[0]) snprintf(url, sizeof url, "%s", r.page); else snprintf(url, sizeof url, "https://%s/post/%d", NC_SITE, id);
        ne_open_page(url);
        return 1;
    }
    if (what == AT_NC_ACT_REFRESH) {
        if (ne_locked()) { ne_say(3, "The catalog cannot refresh while you are online."); return 0; }
        ne_lock(); if (!NE.syncing) NE.sync_want = 2; ne_unlock();
        return 1;
    }
    if (what != AT_NC_ACT_UNINSTALL) return ne_queue_act(id, what);
    if (ne_locked()) { ne_say(3, "Mods cannot be changed while you are online."); return 0; }
    j.kind = 2; j.id = id;
    ne_say(1, "Removing");
    if (!ne_enqueue(j)) { ne_say(3, "Busy. Try again in a moment."); return 0; }
    return 1;
}

static void ne_src_pref_get(void *u, AtNcFilter *f, int *tab) {
    (void) u;
    ne_lock();
    f->fighter = NE.pref.fighter; f->show = NE.pref.show; f->sort = NE.pref.sort;
    snprintf(f->text, sizeof f->text, "%s", NE.pref.text);
    *tab = NE.pref.tab;
    ne_unlock();
}
static void ne_src_pref_set(void *u, const AtNcFilter *f, int tab) {
    (void) u;
    ne_lock();
    NE.pref.fighter = f->fighter; NE.pref.show = f->show; NE.pref.sort = f->sort; NE.pref.tab = tab;
    snprintf(NE.pref.text, sizeof NE.pref.text, "%s", f->text);
    NE.pref_dirty = 1;
    ne_unlock();
}

static int ne_src_fighters(void *u) { (void) u; return NC_NFIGHTERS; }
static const char *ne_src_fighter_label(void *u, int i) { (void) u; return i >= 0 && i < NC_NFIGHTERS ? nc_fighters[i].label : "?"; }

static void ne_make_src(AtNcSrc *s) {
    memset(s, 0, sizeof *s);
    s->filter = ne_src_filter; s->row = ne_src_row; s->status = ne_src_status; s->act = ne_src_act;
    s->fighters = ne_src_fighters; s->fighter_label = ne_src_fighter_label; s->version = ne_src_version;
    s->pic = ne_src_pic; s->pref_get = ne_src_pref_get; s->pref_set = ne_src_pref_set;
}

/* the process is ending: let the threads leave (they check NE.quit between requests), then write the queue one last time */
static void ne_shutdown(void) {
    int i;
    if (!NE.inited) return;
    NE.quit = 1;
    if (NE.th_sync) WaitForSingleObject(NE.th_sync, 3000);
    if (NE.th_files) WaitForSingleObject(NE.th_files, 3000);
    for (i = 0; i < 2; ++i) if (NE.th_img[i]) WaitForSingleObject(NE.th_img[i], 1500);
    ne_lock(); NE.dq_dirty = 1; ne_unlock();
    ne_save_dirty();
}

#endif
