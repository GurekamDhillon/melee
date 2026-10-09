/* gw_nucleus_queue.h - the Nucleus browser's download queue, its saved form, the thumbnail disk cache and the remembered filters. Pure C over stdio and the
 * OS directory calls: no network, no game types, no threads (the engine, gw_nucleus_engine.inc, owns the lock and the worker). Header-only (static), like
 * gw_nucleus_core.h; the native test (pc/tests/nucleus_core_test.c) and the engine use one copy.
 *
 * THE QUEUE. Pressing A on a mod puts it in the queue (nq_toggle); Start begins processing; the worker takes the first QUEUED entry, installs it, and marks it
 * DONE or FAILED with a plain-words reason. The queue is saved after EVERY change with an atomic write (queue.json.tmp flushed to disk, then renamed over
 * queue.json), so a crash or a power cut leaves the last complete queue. On load an entry that was RUNNING is QUEUED again (the install it was in the middle
 * of left only a .staging-* folder, which nq_clean_staging removes: a mod folder appears in the mods folder only by one rename of a finished staging
 * folder), a DONE entry is dropped, and a FAILED one stays so its reason can be read and retried. A file that does not parse is renamed to queue.json.bad
 * and the queue starts empty; nothing here ever clears a queue except nq_clear_pending / nq_clear_finished, which the screen asks for explicitly.
 *
 * THE THUMBNAIL CACHE. Pictures are cached under <nucleus dir>/thumbs/<key>.img, key = the 64-bit FNV-1a of the media URL in hex. nq_cache_trim keeps the
 * folder under a byte cap by deleting the least recently used first (a file's modification time; the engine nq_touch-es a file it reads). */
#ifndef GW_NUCLEUS_QUEUE_H
#define GW_NUCLEUS_QUEUE_H

#include "gw_nucleus_install.h"
#ifndef _WIN32
#include <fcntl.h>
#endif

#define NQ_MAX 400
#define NQ_SCHEMA 1
#define NQ_CACHE_CAP_DEFAULT (200u * 1024u * 1024u)       /* the thumbnail cache's size cap */

enum { NQ_QUEUED, NQ_RUNNING, NQ_DONE, NQ_FAILED };
static const char *const nq_state_name[4] = { "queued", "running", "done", "failed" };

typedef struct {
    int id;                       /* the Nucleus post */
    int state;                    /* NQ_* */
    int pct;                      /* 0..100 while RUNNING */
    int attempts;                 /* how many times an install of it ended in failure */
    int64_t added;                /* epoch seconds */
    char title[80], author[40];   /* a snapshot, so a restored queue reads before the catalog has loaded */
    char reason[120];             /* FAILED: why, in plain words */
} nq_item;

typedef struct {
    nq_item it[NQ_MAX];
    int n;
    int processing;               /* Start was pressed: the worker takes entries (saved, so a restart picks up where it was) */
    unsigned ver;                 /* bumped by every change */
} nq_queue;

static void nq_init(nq_queue *q) { memset(q, 0, sizeof *q); }

static int nq_find(const nq_queue *q, int id) {
    int i;
    for (i = 0; i < q->n; ++i) if (q->it[i].id == id) return i;
    return -1;
}

static int nq_count(const nq_queue *q, unsigned state_mask) {
    int i, n = 0;
    for (i = 0; i < q->n; ++i) if (state_mask & (1u << q->it[i].state)) ++n;
    return n;
}
/* the entries still to do or being done */
static int nq_pending(const nq_queue *q) { return nq_count(q, (1u << NQ_QUEUED) | (1u << NQ_RUNNING)); }

/* 1-based place in line among the entries to do (a RUNNING entry is 1), 0 when the mod is not waiting */
static int nq_place(const nq_queue *q, int id) {
    int i, p = 0;
    for (i = 0; i < q->n; ++i) {
        if (q->it[i].state != NQ_QUEUED && q->it[i].state != NQ_RUNNING) continue;
        ++p;
        if (q->it[i].id == id) return p;
    }
    return 0;
}

static void nq_set_item(nq_item *t, int id, const char *title, const char *author, int64_t now) {
    memset(t, 0, sizeof *t);
    t->id = id; t->state = NQ_QUEUED; t->added = now;
    nc_ascii(t->title, sizeof t->title, title ? title : "");
    nc_ascii(t->author, sizeof t->author, author ? author : "");
}

/* Put a mod at the end of the line. 1 added, 0 it was already waiting (nothing changes), -1 the queue is full. A DONE or FAILED entry for it is replaced. */
static int nq_add(nq_queue *q, int id, const char *title, const char *author, int64_t now) {
    int at = nq_find(q, id);
    if (at >= 0) {
        if (q->it[at].state == NQ_QUEUED || q->it[at].state == NQ_RUNNING) return 0;
        memmove(&q->it[at], &q->it[at + 1], (size_t) (q->n - at - 1) * sizeof q->it[0]);
        --q->n;
    }
    if (q->n >= NQ_MAX) return -1;
    nq_set_item(&q->it[q->n++], id, title, author, now);
    ++q->ver;
    return 1;
}

/* Take a mod out of the queue. 1 removed; 0 it is not there or is being installed right now (an install is never torn out from under the worker). */
static int nq_remove(nq_queue *q, int id) {
    int at = nq_find(q, id);
    if (at < 0 || q->it[at].state == NQ_RUNNING) return 0;
    memmove(&q->it[at], &q->it[at + 1], (size_t) (q->n - at - 1) * sizeof q->it[0]);
    --q->n;
    ++q->ver;
    return 1;
}

/* A on a mod: queue it, or take it out again when it is waiting. +1 queued, -1 removed, 0 refused (it is being installed, or the queue is full). */
static int nq_toggle(nq_queue *q, int id, const char *title, const char *author, int64_t now) {
    int at = nq_find(q, id);
    if (at >= 0 && q->it[at].state == NQ_QUEUED) return nq_remove(q, id) ? -1 : 0;
    if (at >= 0 && q->it[at].state == NQ_RUNNING) return 0;
    return nq_add(q, id, title, author, now) == 1 ? 1 : 0;
}

/* Move a waiting entry one place up (dir < 0) or down past the neighbouring waiting entry. 1 moved. */
static int nq_move(nq_queue *q, int id, int dir) {
    int at = nq_find(q, id), j;
    nq_item tmp;
    if (at < 0 || q->it[at].state != NQ_QUEUED) return 0;
    j = at + (dir < 0 ? -1 : 1);
    while (j >= 0 && j < q->n && q->it[j].state != NQ_QUEUED && q->it[j].state != NQ_RUNNING) j += dir < 0 ? -1 : 1;   /* hop over finished entries */
    if (j < 0 || j >= q->n || q->it[j].state != NQ_QUEUED) return 0;                                                   /* never above the one being installed */
    tmp = q->it[at]; q->it[at] = q->it[j]; q->it[j] = tmp;
    ++q->ver;
    return 1;
}

static int nq_next(const nq_queue *q) {
    int i;
    for (i = 0; i < q->n; ++i) if (q->it[i].state == NQ_QUEUED) return i;
    return -1;
}

static void nq_mark_running(nq_queue *q, int id) {
    int at = nq_find(q, id);
    if (at < 0) return;
    q->it[at].state = NQ_RUNNING; q->it[at].pct = 0; q->it[at].reason[0] = '\0';
    ++q->ver;
}
/* progress is a number the screen shows: it changes the version only when the shown percent does */
static void nq_progress(nq_queue *q, int id, int pct) {
    int at = nq_find(q, id);
    if (pct < 0) pct = 0;
    if (pct > 100) pct = 100;
    if (at < 0 || q->it[at].state != NQ_RUNNING || q->it[at].pct == pct) return;
    q->it[at].pct = pct;
    ++q->ver;
}
static void nq_finish(nq_queue *q, int id, int ok, const char *reason) {
    int at = nq_find(q, id);
    if (at < 0) return;
    q->it[at].state = ok ? NQ_DONE : NQ_FAILED;
    q->it[at].pct = ok ? 100 : 0;
    if (!ok) ++q->it[at].attempts;
    nc_ascii(q->it[at].reason, sizeof q->it[at].reason, reason ? reason : "");
    ++q->ver;
}
/* a failed entry goes back in line where it stands */
static int nq_retry(nq_queue *q, int id) {
    int at = nq_find(q, id);
    if (at < 0 || q->it[at].state != NQ_FAILED) return 0;
    q->it[at].state = NQ_QUEUED; q->it[at].pct = 0; q->it[at].reason[0] = '\0';
    ++q->ver;
    return 1;
}
/* Explicit clears (the screen confirms first). pending: every QUEUED and FAILED entry, never the one being installed. finished: DONE entries. Returns the count removed. */
static int nq_clear_state(nq_queue *q, unsigned state_mask) {
    int i, w = 0;
    for (i = 0; i < q->n; ++i) if (!(state_mask & (1u << q->it[i].state))) q->it[w++] = q->it[i];
    i = q->n - w;
    q->n = w;
    if (i) ++q->ver;
    return i;
}
static int nq_clear_pending(nq_queue *q) { return nq_clear_state(q, (1u << NQ_QUEUED) | (1u << NQ_FAILED)); }
static int nq_clear_finished(nq_queue *q) { return nq_clear_state(q, 1u << NQ_DONE); }

/* what a crash leaves: the RUNNING entry goes back in line */
static void nq_recover(nq_queue *q) {
    int i;
    for (i = 0; i < q->n; ++i) if (q->it[i].state == NQ_RUNNING) { q->it[i].state = NQ_QUEUED; q->it[i].pct = 0; }
}

/* ---- saved form -------------------------------------------------------------------------------------------- */

static void nq_to_json(const nq_queue *q, nj_buf *b) {
    int i;
    nj_printf(b, "{\"schema\":%d,\"processing\":%d,\"items\":[", NQ_SCHEMA, q->processing ? 1 : 0);
    for (i = 0; i < q->n; ++i) {
        const nq_item *t = &q->it[i];
        nj_printf(b, "%s\n{\"id\":%d,\"state\":", i ? "," : "", t->id); nj_qstr(b, nq_state_name[t->state]);
        nj_printf(b, ",\"attempts\":%d,\"added\":%lld,\"title\":", t->attempts, (long long) t->added); nj_qstr(b, t->title);
        nj_puts(b, ",\"author\":"); nj_qstr(b, t->author);
        nj_puts(b, ",\"reason\":"); nj_qstr(b, t->reason);
        nj_puts(b, "}");
    }
    nj_puts(b, "\n]}\n");
}

/* Parse a saved queue into q (cleared first). 0 ok, -1 when the text is not a queue. Applies the recovery rules: RUNNING -> QUEUED, DONE dropped. */
static int nq_from_json(nq_queue *q, const char *text) {
    nj_doc d;
    int root, items, e;
    nq_init(q);
    root = nj_parse(&d, text);
    if (root < 0 || d.n[root].type != NJ_OBJ) { nj_free(&d); return -1; }
    items = nj_get(&d, root, "items");
    if (items < 0 || d.n[items].type != NJ_ARR) { nj_free(&d); return -1; }
    q->processing = (int) nj_gnum(&d, root, "processing", 0) != 0;
    for (e = d.n[items].first; e >= 0 && q->n < NQ_MAX; e = d.n[e].next) {
        nq_item t;
        const char *st;
        int id = (int) nj_gnum(&d, e, "id", 0), s = -1, i;
        if (id <= 0 || d.n[e].type != NJ_OBJ || nq_find(q, id) >= 0) continue;
        st = nj_gstr(&d, e, "state", "queued");
        for (i = 0; i < 4; ++i) if (!strcmp(st, nq_state_name[i])) s = i;
        if (s < 0 || s == NQ_DONE) continue;
        nq_set_item(&t, id, nj_gstr(&d, e, "title", ""), nj_gstr(&d, e, "author", ""), (int64_t) nj_gnum(&d, e, "added", 0));
        t.state = s == NQ_RUNNING ? NQ_QUEUED : s;
        t.attempts = (int) nj_gnum(&d, e, "attempts", 0);
        nc_ascii(t.reason, sizeof t.reason, nj_gstr(&d, e, "reason", ""));
        q->it[q->n++] = t;
    }
    nj_free(&d);
    ++q->ver;
    return 0;
}

/* The queue to disk, atomically. The caller holds whatever lock guards q. 0 ok. */
static int nq_save(const nq_queue *q, const char *path) {
    nj_buf b;
    int rc;
    memset(&b, 0, sizeof b);
    nq_to_json(q, &b);
    if (b.bad || !b.s) { free(b.s); return -1; }
    rc = nc_write_file_atomic(path, b.s, b.n);
    free(b.s);
    return rc;
}

/* Load the saved queue. Returns the number of entries, 0 for no file or an empty queue, -1 when the file was unreadable (it is renamed to <path>.bad and the
 * queue is empty). A leftover <path>.tmp (a crash mid-write) is never used: the target is complete or absent. */
static int nq_load(nq_queue *q, const char *path) {
    char *text = nc_read_all(path, NULL);
    nq_init(q);
    if (!text) return 0;
    if (nq_from_json(q, text) != 0) {
        char bad[620];
        free(text);
        snprintf(bad, sizeof bad, "%s.bad", path);
        remove(bad);
        rename(path, bad);
        nq_init(q);
        return -1;
    }
    free(text);
    return q->n;
}

/* ---- directories ---------------------------------------------------------------------------------------------- */

typedef struct { const char *name; int is_dir; int64_t size, mtime; } nq_entry;

/* call fn for every entry of a folder (not . or ..); fn returns nonzero to stop. The name is valid only during the call. */
static void nq_each(const char *dir, int (*fn)(void *u, const nq_entry *e), void *u) {
#ifdef _WIN32
    char pat[620];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    snprintf(pat, sizeof pat, "%s\\*", dir);
    h = FindFirstFileA(pat, &fd);
    if (h == INVALID_HANDLE_VALUE) return;
    do {
        nq_entry e;
        unsigned long long t;
        if (!strcmp(fd.cFileName, ".") || !strcmp(fd.cFileName, "..")) continue;
        t = ((unsigned long long) fd.ftLastWriteTime.dwHighDateTime << 32) | fd.ftLastWriteTime.dwLowDateTime;
        e.name = fd.cFileName;
        e.is_dir = (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0;
        e.size = ((int64_t) fd.nFileSizeHigh << 32) | fd.nFileSizeLow;
        e.mtime = (int64_t) (t / 10000000ull) - 11644473600ll;
        if (fn(u, &e)) break;
    } while (FindNextFileA(h, &fd));
    FindClose(h);
#else
    DIR *d = opendir(dir);
    struct dirent *de;
    if (!d) return;
    while ((de = readdir(d)) != NULL) {
        nq_entry e;
        struct stat st;
        char path[620];
        if (!strcmp(de->d_name, ".") || !strcmp(de->d_name, "..")) continue;
        snprintf(path, sizeof path, "%s/%s", dir, de->d_name);
        if (stat(path, &st) != 0) continue;
        e.name = de->d_name;
        e.is_dir = S_ISDIR(st.st_mode) != 0;
        e.size = (int64_t) st.st_size;
        e.mtime = (int64_t) st.st_mtime;
        if (fn(u, &e)) break;
    }
    closedir(d);
#endif
}

/* Set a file's modification time (t <= 0: now). The thumbnail cache's "recently used" mark. */
static void nq_touch(const char *path, int64_t t) {
#ifdef _WIN32
    HANDLE h = CreateFileA(path, FILE_WRITE_ATTRIBUTES, FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
    if (h != INVALID_HANDLE_VALUE) {
        FILETIME ft;
        if (t > 0) {
            unsigned long long v = ((unsigned long long) t + 11644473600ull) * 10000000ull;
            ft.dwLowDateTime = (DWORD) (v & 0xFFFFFFFFu); ft.dwHighDateTime = (DWORD) (v >> 32);
        } else GetSystemTimeAsFileTime(&ft);
        SetFileTime(h, NULL, NULL, &ft);
        CloseHandle(h);
    }
#else
    struct timespec ts[2];
    ts[0].tv_sec = ts[1].tv_sec = t > 0 ? (time_t) t : time(NULL);
    ts[0].tv_nsec = ts[1].tv_nsec = 0;
    utimensat(AT_FDCWD, path, ts, 0);
#endif
}

/* ---- crash leftovers ------------------------------------------------------------------------------------------ */

typedef struct { const char *dir; int removed; } nq_clean_ctx;
static int nq_clean_cb(void *u, const nq_entry *e) {
    nq_clean_ctx *c = (nq_clean_ctx *) u;
    char path[620];
    if (!e->is_dir || strncmp(e->name, ".staging-", 9) != 0) return 0;
    snprintf(path, sizeof path, "%s/%s", c->dir, e->name);
    nc_fix_slashes(path);
    nc_rmtree(path);
    ++c->removed;
    return 0;
}
/* Remove the .staging-* folders an interrupted install left in the mods folder (an install builds in .staging-<mod> and renames the finished folder into
 * place, so these are never a mod). Returns how many were removed. */
static int nq_clean_staging(const char *mods_dir) {
    nq_clean_ctx c;
    c.dir = mods_dir; c.removed = 0;
    nq_each(mods_dir, nq_clean_cb, &c);
    return c.removed;
}

/* ---- the thumbnail disk cache -------------------------------------------------------------------------------------- */

/* 64-bit FNV-1a of a URL as 16 hex digits: the cache file's name and the engine's in-memory key */
static uint64_t nq_hash(const char *s) {
    uint64_t h = 1469598103934665603ull;
    for (; *s; ++s) { h ^= (unsigned char) *s; h *= 1099511628211ull; }
    return h;
}
static void nq_cache_name(const char *url, char *out, size_t cap) { snprintf(out, cap, "%016llx.img", (unsigned long long) nq_hash(url)); }

typedef struct { char name[40]; int64_t size, mtime; } nq_cfile;
typedef struct { nq_cfile *f; int n, cap; int64_t total; const char *dir; } nq_cache_scan;
static int nq_cache_cb(void *u, const nq_entry *e) {
    nq_cache_scan *s = (nq_cache_scan *) u;
    size_t l = strlen(e->name);
    char path[620];
    if (e->is_dir || l >= sizeof s->f[0].name) return 0;
    if (l > 4 && !strcmp(e->name + l - 4, ".tmp")) {          /* a download or write that never finished */
        snprintf(path, sizeof path, "%s/%s", s->dir, e->name);
        nc_fix_slashes(path);
        remove(path);
        return 0;
    }
    if (l < 5 || strcmp(e->name + l - 4, ".img") != 0) return 0;
    if (s->n == s->cap) {
        int nc = s->cap ? s->cap * 2 : 256;
        nq_cfile *p = (nq_cfile *) realloc(s->f, (size_t) nc * sizeof *p);
        if (!p) return 1;
        s->f = p; s->cap = nc;
    }
    memcpy(s->f[s->n].name, e->name, l + 1);
    s->f[s->n].size = e->size; s->f[s->n].mtime = e->mtime;
    ++s->n;
    s->total += e->size;
    return 0;
}
static int nq_cache_cmp(const void *a, const void *b) {
    const nq_cfile *x = (const nq_cfile *) a, *y = (const nq_cfile *) b;
    if (x->mtime != y->mtime) return x->mtime < y->mtime ? -1 : 1;
    return strcmp(x->name, y->name);
}
/* Keep the cache folder at or under cap_bytes: the least recently used files (oldest modification time) go first, down to 90% of the cap so the next trim is
 * not immediate. Stray .tmp files are deleted. Returns the bytes left. */
static int64_t nq_cache_trim(const char *dir, int64_t cap_bytes) {
    nq_cache_scan s;
    int i;
    memset(&s, 0, sizeof s);
    s.dir = dir;
    nq_each(dir, nq_cache_cb, &s);
    if (s.total > cap_bytes && s.n > 0) {
        int64_t goal = cap_bytes - cap_bytes / 10;
        qsort(s.f, (size_t) s.n, sizeof s.f[0], nq_cache_cmp);
        for (i = 0; i < s.n && s.total > goal; ++i) {
            char path[620];
            snprintf(path, sizeof path, "%s/%s", dir, s.f[i].name);
            nc_fix_slashes(path);
            if (remove(path) == 0) s.total -= s.f[i].size;
        }
    }
    free(s.f);
    return s.total;
}

/* ---- remembered filters ---------------------------------------------------------------------------------------- */

/* The browser's list settings, kept across sessions in <nucleus dir>/ui.json. show: 0 all, 1 not installed, 2 installed, 3 update available, 4 queued. */
typedef struct { int tab, fighter, show, sort; char text[48]; } nq_prefs;

static void nq_prefs_default(nq_prefs *p) { memset(p, 0, sizeof *p); p->fighter = -1; }

static int nq_prefs_save(const nq_prefs *p, const char *path) {
    nj_buf b;
    int rc;
    memset(&b, 0, sizeof b);
    nj_printf(&b, "{\"schema\":1,\"tab\":%d,\"fighter\":%d,\"show\":%d,\"sort\":%d,\"text\":", p->tab, p->fighter, p->show, p->sort);
    nj_qstr(&b, p->text);
    nj_puts(&b, "}\n");
    if (b.bad || !b.s) { free(b.s); return -1; }
    rc = nc_write_file_atomic(path, b.s, b.n);
    free(b.s);
    return rc;
}

/* 1 loaded, 0 no usable file (p holds the defaults). Out-of-range numbers are clamped by the caller's own limits (tabs, fighters), not trusted here. */
static int nq_prefs_load(nq_prefs *p, const char *path) {
    char *text = nc_read_all(path, NULL);
    nj_doc d;
    int root;
    nq_prefs_default(p);
    if (!text) return 0;
    root = nj_parse(&d, text);
    free(text);
    if (root < 0 || d.n[root].type != NJ_OBJ) { nj_free(&d); return 0; }
    p->tab = (int) nj_gnum(&d, root, "tab", 0);
    p->fighter = (int) nj_gnum(&d, root, "fighter", -1);
    p->show = (int) nj_gnum(&d, root, "show", 0);
    p->sort = (int) nj_gnum(&d, root, "sort", 0);
    nc_ascii(p->text, sizeof p->text, nj_gstr(&d, root, "text", ""));
    nj_free(&d);
    return 1;
}

#endif
