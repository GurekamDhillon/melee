/* gw_nucleus_engine.inc - the Nucleus browser's engine half: two worker threads, the catalog and its cache, installs, and the source the NUCLEUS screen
 * reads (AtNcSrc, gw_ui_nucleus.inc). Included by gw_script.c at the top of gw_script_ui_mods.inc (after gw_ui_nucleus.inc); the pure logic is
 * gw_nucleus_catalog.h / gw_nucleus_install.h and the transport gw_nucleus_http.inc.
 *
 * NOTHING HERE RUNS ON THE GAME THREAD OR DURING A NETPLAY SESSION. The catalog thread syncs (first a full keyset page-through, then updated_since and
 * the removed feed, at most once per 10 minutes, with back-off); the files thread installs, uninstalls and fetches the preview art. The screen only
 * reads copies under one lock, and starts nothing while a netplay session is on. The threads start the first time the NUCLEUS screen opens.
 *
 * Where things live. mods dir = gw_Mods_Dir() (MELEE_MODS_DIR). The cache is <mods dir>/.nucleus (a dot folder: the mods scanner skips it), or
 * MELEE_NUCLEUS_DIR: catalog.jsonl, state.json, art/<file id>_csp.png and _stock.png. An install writes <mods dir>/nucleus-<post>-<fighter>/ (a skin mod,
 * gw_nucleus_install.h) through a .staging folder; it is read at the next boot (the skin registry is built before the arena is carved), so the screen says
 * "Restart the game to use new skins". A skin the other side lacks is shown as the default costume by the registry (docs/mods-packaging.md 4b).
 *
 * The thread-safety rule for logging: a worker never calls gw_log; it queues a line and the screen's tick (ne_pump) writes it. */
#ifndef GW_NUCLEUS_ENGINE_INC
#define GW_NUCLEUS_ENGINE_INC

#include "gw_nucleus_install.h"
#include "gw_nucleus_http.inc"

#define NE_INST_MAX 256
#define NE_QUEUE 16
#define NE_ART_FAIL 128

typedef struct { int kind, id, file, art_kind; } ne_job;     /* kind: 1 install, 2 uninstall, 3 art */

static struct {
    int inited, quit, loaded;
    char dir[MAX_PATH], mods[MAX_PATH], art[MAX_PATH], catpath[MAX_PATH], statepath[MAX_PATH];
    CRITICAL_SECTION lock;
    nc_catalog cat;
    nc_sync sync;
    int sync_want, syncing, sync_done, sync_total, sync_failed, sync_ever_ok;
    char sync_what[64], sync_msg[140];
    ne_job q[NE_QUEUE]; int qh, qt;
    int job_state, job_kind, job_id, job_done, job_total;
    int64_t job_at;
    char job_msg[160];
    nc_inst inst[NE_INST_MAX]; int ninst;
    int restart;
    int art_fail[NE_ART_FAIL]; int art_nfail;
    int art_busy[8]; int art_nbusy;
    char logq[24][220]; int nlog;
    unsigned ver;
    HANDLE th_sync, th_files;
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

/* the game thread, once per tick: write what the workers queued */
static void ne_pump(void) {
    int i;
    char lines[24][220];
    int n;
    if (!NE.inited) return;
    ne_lock();
    n = NE.nlog;
    for (i = 0; i < n; ++i) memcpy(lines[i], NE.logq[i], sizeof lines[i]);
    NE.nlog = 0;
    ne_unlock();
    for (i = 0; i < n; ++i) gw_log("nucleus: %s", lines[i]);
}

static int64_t ne_now(void) { return (int64_t) time(NULL); }

/* ---- paths and persistence ---- */

static void ne_paths(void) {
    const char *env = getenv("MELEE_NUCLEUS_DIR");
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
    snprintf(NE.catpath, sizeof NE.catpath, "%s/catalog.jsonl", NE.dir);
    snprintf(NE.statepath, sizeof NE.statepath, "%s/state.json", NE.dir);
    nc_fix_slashes(NE.art); nc_fix_slashes(NE.catpath); nc_fix_slashes(NE.statepath);
}

/* the catalog and the sync state to disk; the caller does NOT hold the lock */
static void ne_save(void) {
    nj_buf b;
    memset(&b, 0, sizeof b);
    ne_lock();
    nc_cat_save(&NE.cat, NE.catpath);
    nc_sync_to_json(&NE.sync, &b);
    ne_unlock();
    if (!b.bad && b.s) nc_write_file(NE.statepath, b.s, b.n);
    free(b.s);
}

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
    /* load what the last session saved */
    {
        char *st = nc_read_all(NE.statepath, NULL);
        nc_catalog loaded;
        int n;
        memset(&loaded, 0, sizeof loaded);
        n = nc_cat_load(&loaded, NE.catpath);
        ne_lock();
        if (st) { nc_sync_from_json(&NE.sync, st); }
        if (n >= 0) { nc_cat_free(&NE.cat); NE.cat = loaded; ++NE.ver; }
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
            ++NE.ver;
            ne_unlock();
            ne_log("sync %s: %d mods%s%s", rc == 0 ? "done" : rc > 0 ? "skipped" : "failed", NE.cat.n, st.msg[0] ? " - " : "", st.msg);
            if (rc >= 0) ne_save();
        }
        Sleep(150);
    }
    return 0;
}

/* ---- the files thread: art, installs, uninstalls ---- */

static void ne_set_job(int state, const char *msg, int done, int total) {
    ne_lock();
    NE.job_state = state; NE.job_done = done; NE.job_total = total; NE.job_at = ne_now();
    snprintf(NE.job_msg, sizeof NE.job_msg, "%s", msg);
    ++NE.ver;
    ne_unlock();
}

static void ne_art_path(int file_id, int kind, char *out, size_t cap) { snprintf(out, cap, "%s/%d_%s.png", NE.art, file_id, kind == 1 ? "csp" : "stock"); nc_fix_slashes(out); }

/* the PNG for a file's csp (kind 1) or stock (kind 2): the cached copy or a download into the cache; a malloc'd buffer, or NULL */
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

static int ne_file_is_failed(int id) { int i; for (i = 0; i < NE.art_nfail; ++i) if (NE.art_fail[i] == id) return 1; return 0; }

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

static int ne_install_impl(int id, char *msg, size_t mcap, uint8_t **zipout);
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
    int zip_tried = 0, zip_status = 0;
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
            ne_set_job(1, "Downloading a costume", done, total);
            if (files[i].zip) {
                /* no per-file download: the file is an entry of the mod's zip (GET /mods/{id}/download), fetched once */
                memset(&r, 0, sizeof r);
                if (!zip_tried) {
                    nch_result zr;
                    zip_tried = 1;
                    nc_download_url(m.id, 0, url, sizeof url);
                    if (nch_get(url, 64u * 1024 * 1024, &zr) < 0 || zr.status != 200 || !zr.data) {
                        zip_status = zr.status ? zr.status : -1;
                        ne_log("install %d: zip download: %s (HTTP %d)", m.id, zr.err[0] ? zr.err : "failed", zr.status);
                        free(zr.data);
                    } else { zipbuf = zr.data; ziplen = zr.len; *zipout = zipbuf; }
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
            } else {
            nc_download_url(m.id, files[i].id, url, sizeof url);
            if (nch_get(url, 24u * 1024 * 1024, &r) < 0 || r.status != 200 || !r.data) {
                if (!first_err[0]) snprintf(first_err, sizeof first_err, "%s", nc_http_reason(r.status));
                ne_log("install %d: file %d: %s (HTTP %d)", m.id, files[i].id, r.err[0] ? r.err : "download failed", r.status);
                free(r.data); done += 3; continue;
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
            if (nc_write_file(tmp, r.data, r.len) != 0) { free(r.data); snprintf(msg, mcap, "Cannot write the costume file."); free(sm); return -1; }
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

static DWORD WINAPI ne_files_thread(LPVOID arg) {
    (void) arg;
    nc_mkdirs(NE.art);
    while (!NE.quit) {
        ne_job j;
        int have = 0;
        ne_lock();
        if (NE.qh != NE.qt) { j = NE.q[NE.qh]; NE.qh = (NE.qh + 1) % NE_QUEUE; have = 1; }
        ne_unlock();
        if (!have) { Sleep(60); continue; }
        if (j.kind == 3) {
            nc_mod m;
            nc_file files[NC_MAX_FILES_PER_MOD];
            int k;
            if (ne_copy_mod(j.id, &m, files)) {
                for (k = 0; k < m.nf; ++k) if (files[k].id == j.file) {
                    size_t len;
                    uint8_t *p = ne_art_png(&files[k], j.art_kind, &len);
                    if (!p) { ne_lock(); if (NE.art_nfail < NE_ART_FAIL) NE.art_fail[NE.art_nfail++] = j.file; ne_unlock(); }
                    free(p);
                }
            }
            ne_lock(); { int b; for (b = 0; b < NE.art_nbusy; ++b) if (NE.art_busy[b] == j.file) { NE.art_busy[b] = NE.art_busy[--NE.art_nbusy]; break; } ++NE.ver; } ne_unlock();
        } else {
            char msg[200];
            int rc;
            ne_lock(); NE.job_kind = j.kind; NE.job_id = j.id; ne_unlock();
            ne_set_job(1, j.kind == 1 ? "Installing" : "Removing", 0, 1);
            rc = j.kind == 1 ? ne_install(j.id, msg, sizeof msg) : ne_uninstall(j.id, msg, sizeof msg);
            ne_scan_installed();
            if (rc == 0) { ne_lock(); NE.restart = 1; ne_unlock(); }
            ne_set_job(rc == 0 ? 2 : 3, msg, 1, 1);
            ne_log("%s %d: %s", j.kind == 1 ? "install" : "uninstall", j.id, msg);
        }
    }
    return 0;
}

static int ne_enqueue(ne_job j) {
    int ok = 0;
    ne_lock();
    if ((NE.qt + 1) % NE_QUEUE != NE.qh) { NE.q[NE.qt] = j; NE.qt = (NE.qt + 1) % NE_QUEUE; ok = 1; }
    ne_unlock();
    return ok;
}

/* ---- the screen's source (the game thread) ---- */

/* the game thread: start the threads once, and ask for a sync (the sync itself keeps the 10-minute rule) */
static void ne_open(void) {
    if (!NE.inited) {
        const char *api = getenv("MELEE_NUCLEUS_API");
        DWORD tid;
        memset(&NE, 0, sizeof NE);
        InitializeCriticalSection(&NE.lock);
        if (api && api[0] && strlen(api) < sizeof nc_api_base - 1) { snprintf(nc_api_base, sizeof nc_api_base, "%s", api); nch_set_override(api); }
        ne_paths();
        nc_mkdirs(NE.dir);
        nc_mkdirs(NE.art);
        NE.inited = 1;
        ne_scan_installed();
        NE.th_sync = CreateThread(NULL, 0, ne_sync_thread, NULL, 0, &tid);
        NE.th_files = CreateThread(NULL, 0, ne_files_thread, NULL, 0, &tid);
        gw_log("nucleus: browser started; cache %s, api %s", NE.dir, nc_api_base);
    }
    if (!gs_ui_netplay()) { ne_lock(); if (!NE.sync_want && !NE.syncing) NE.sync_want = 1; ne_unlock(); }
}

static int ne_locked(void) { return gs_ui_netplay(); }

static void ne_fill_row_from_mod(const nc_mod *m, const nc_file *files, AtNcRow *r) {
    int i, fighter = -1;
    memset(r, 0, sizeof *r);
    r->tex = -1;
    r->id = m->id;
    snprintf(r->title, sizeof r->title, "%s", m->title);
    snprintf(r->author, sizeof r->author, "%s", m->author);
    snprintf(r->kind, sizeof r->kind, "%s", nc_type_label[m->type]);
    snprintf(r->updated, sizeof r->updated, "%.10s", m->updated);
    snprintf(r->page, sizeof r->page, "%s", m->page);
    snprintf(r->desc, sizeof r->desc, "%s", m->desc);
    snprintf(r->tags, sizeof r->tags, "%s", m->tags);
    r->downloads = m->downloads; r->likes = m->likes;
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
    int found, i, inst = -1;
    const char *why = "";
    (void) u;
    found = ne_copy_mod(id, &m, files);
    ne_lock();
    for (i = 0; i < NE.ninst; ++i) if (NE.inst[i].id == id) { inst = i; break; }
    if (found) {
        ne_fill_row_from_mod(&m, files, r);
        why = nc_why_not_files(&m, files);
        r->installable = why[0] == '\0';
        snprintf(r->reason, sizeof r->reason, "%s", why);
    } else if (inst >= 0) {
        memset(r, 0, sizeof *r);
        r->tex = -1; r->id = id;
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
    } else { ne_unlock(); return 0; }
    if (inst >= 0) {
        int gone = nc_cat_is_gone(&NE.cat, id) || (!found && NE.sync.full);
        r->state = gone ? AT_NC_ST_REMOVED : (found && strcmp(NE.inst[inst].updated, m.updated) != 0) ? AT_NC_ST_UPDATE : AT_NC_ST_INSTALLED;
    } else r->state = r->installable ? AT_NC_ST_NONE : AT_NC_ST_UNSUPPORTED;
    if (NE.job_state == 1 && NE.job_id == id) r->state = AT_NC_ST_BUSY;
    ne_unlock();
    /* the preview: the first costume file's portrait, from the cache; a miss asks the files thread */
    if (found && r->ncostumes > 0) {
        for (i = 0; i < m.nf; ++i) if (files[i].costume && files[i].csp[0]) {
            char key[48], path[MAX_PATH];
            int tex;
            snprintf(key, sizeof key, "nc:csp:%d", files[i].id);
            tex = gw_Kit_TexHsdFind(key);
            if (tex < 0) {
                uint8_t *png, *gx;
                size_t pl, gl;
                char err[80];
                ne_art_path(files[i].id, 1, path, sizeof path);
                png = (uint8_t *) nc_read_all(path, &pl);
                if (png) {
                    if (nc_png_to_gxtex(png, pl, &gx, &gl, err, sizeof err) == 0) { tex = gw_Kit_TexAddGxtex(key, gx, gl); free(gx); }
                    free(png);
                } else {
                    int b, busy = 0, fail;
                    ne_lock();
                    for (b = 0; b < NE.art_nbusy; ++b) if (NE.art_busy[b] == files[i].id) busy = 1;
                    fail = ne_file_is_failed(files[i].id);
                    if (!busy && !fail && NE.art_nbusy < 8) {
                        ne_job j;
                        j.kind = 3; j.id = id; j.file = files[i].id; j.art_kind = 1;
                        if (ne_enqueue(j)) NE.art_busy[NE.art_nbusy++] = files[i].id;
                    }
                    ne_unlock();
                }
            }
            r->tex = tex;
            break;
        }
    }
    return 1;
}

typedef struct { int id; char sort_title[80]; } ne_inst_key;
static int ne_cmp_inst(const void *a, const void *b) { return strcmp(((const ne_inst_key *) a)->sort_title, ((const ne_inst_key *) b)->sort_title); }

static int ne_src_filter(void *u, const AtNcFilter *f, int *ids, int cap, unsigned *version) {
    int n = 0, i;
    (void) u;
    ne_lock();
    *version = NE.ver ? NE.ver : 1;
    if (f->view == AT_NC_VIEW_INSTALLED) {
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
            for (i = 0; i < m && n < cap; ++i) ids[n++] = NE.cat.m[idx[i]].id;
            free(idx);
        }
    }
    ne_unlock();
    return n;
}

static unsigned ne_src_version(void *u) { unsigned v; (void) u; ne_lock(); v = NE.ver ? NE.ver : 1; ne_unlock(); return v; }

static void ne_src_status(void *u, AtNcStatus *s) {
    (void) u;
    memset(s, 0, sizeof *s);
    ne_lock();
    s->syncing = NE.syncing; s->done = NE.sync_done; s->total = NE.sync_total;
    s->catalog_n = NE.cat.n; s->restart = NE.restart; s->locked = ne_locked();
    s->job_state = (NE.job_state == 1 || ne_now() - NE.job_at < 7) ? NE.job_state : 0;     /* a finished job's message shows for a few seconds */
    snprintf(s->job_msg, sizeof s->job_msg, "%s", NE.job_msg);
    snprintf(s->line, sizeof s->line, "%s", NE.syncing ? NE.sync_what : NE.sync_msg);
    s->offline = NE.sync_failed;
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
    if (ne_locked()) { ne_set_job(3, "Mods cannot be changed while online.", 0, 0); return 0; }
    if (what == AT_NC_ACT_REFRESH) { ne_lock(); if (!NE.syncing) NE.sync_want = 2; ne_unlock(); return 1; }
    ne_lock();
    if (NE.job_state == 1) { ne_unlock(); return 0; }
    ne_unlock();
    j.kind = what == AT_NC_ACT_INSTALL ? 1 : 2; j.id = id; j.file = 0; j.art_kind = 0;
    ne_set_job(1, j.kind == 1 ? "Installing" : "Removing", 0, 1);
    if (!ne_enqueue(j)) { ne_set_job(3, "Busy. Try again in a moment.", 0, 0); return 0; }
    return 1;
}

static int ne_src_fighters(void *u) { (void) u; return NC_NFIGHTERS; }
static const char *ne_src_fighter_label(void *u, int i) { (void) u; return i >= 0 && i < NC_NFIGHTERS ? nc_fighters[i].label : "?"; }

static void ne_make_src(AtNcSrc *s) {
    memset(s, 0, sizeof *s);
    s->filter = ne_src_filter; s->row = ne_src_row; s->status = ne_src_status; s->act = ne_src_act;
    s->fighters = ne_src_fighters; s->fighter_label = ne_src_fighter_label; s->version = ne_src_version;
}

/* the process is ending: let the threads leave (they check NE.quit between requests) */
static void ne_shutdown(void) {
    if (!NE.inited) return;
    NE.quit = 1;
    if (NE.th_sync) WaitForSingleObject(NE.th_sync, 3000);
    if (NE.th_files) WaitForSingleObject(NE.th_files, 3000);
}

#endif
