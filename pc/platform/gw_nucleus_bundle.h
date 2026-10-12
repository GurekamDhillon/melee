/* gw_nucleus_bundle.h - installing a GD mod zip: a Nucleus post (or a local zip) whose archive is a game folder, "mods/<id>/mod.json + files" for each mod it carries,
 * usually with the game's melee-pc.exe at the root (the Goku post: melee-pc.exe, mods/goku-xv2/, mods/geno-lab/). Pure C over stdio / the OS directory calls, header-only,
 * no game types, like gw_nucleus_loose.h; the engine feeds it the downloaded zip.
 *
 * What it does:
 *   - installs every mods/<id>/ folder that has a mod.json, through a staging folder swapped in whole (a failure leaves the old mod or none). <id> is the folder name and is
 *     validated (1..40 of [A-Za-z0-9_-]); archive paths are never used as output paths beyond that: each file lands at mods/<id>/<its path inside the folder>, and the zip
 *     directory was validated first (no traversal, absolute or stream names, symlinks, encryption, duplicates);
 *   - never writes melee-pc.exe or any file at the zip's root, and never writes .exe/.py/.sha256 anywhere. The running game is never replaced;
 *   - leaves alone a mods/<id>/ that is already there and was not installed from this post (it may be the player's own, or the geno-lab that ships with the game): it is
 *     kept and the message says so. A folder this installer wrote earlier (a `.nucleus-bundle` marker holding "post:<id>", or "local:<hash>" for a local zip, which any later local zip may replace) is replaced by the update;
 *   - tells the player, in the message, when the zip's melee-pc.exe is NEWER than the running one (the PE timestamps), and when a mod's mod.json needs an engine feature this
 *     build lacks (the same "engine" rule the mod scanner applies at load). Both are notes on a successful install: the mods are in place, they may just not load yet.
 * Enabled for the next boot through enabled.txt like every other Nucleus install. */
#ifndef GW_NUCLEUS_BUNDLE_H
#define GW_NUCLEUS_BUNDLE_H

#include "gw_nucleus_loose.h"

#define NB_MAX_MODS 8

typedef struct {
    uint32_t host_stamp;                                              /* the running exe's PE TimeDateStamp, 0 when unknown */
    int (*engine_miss)(const char *mod_json, char *why, size_t cap);  /* NULL, or 1 and a phrase when the mod.json needs an engine feature this build lacks */
} nc_bundle_env;

typedef struct {
    int installed, files, kept;
    char ids[NB_MAX_MODS][48], kept_ids[NB_MAX_MODS][48];
    int has_exe;                                                      /* the zip carries melee-pc.exe at its root */
    uint32_t zip_stamp;                                               /* its PE timestamp, 0 when unreadable */
    int exe_newer;                                                    /* zip_stamp > host_stamp (both known) */
    int needs_engine;                                                 /* mods installed that need a newer engine */
    char notes[500];
} nc_bundle_result;

/* PE TimeDateStamp of an exe image (the first bytes are enough), 0 when it is not one. */
static uint32_t nc_pe_stamp(const uint8_t *p, size_t n) {
    uint32_t pe;
    if (n < 0x40 || p[0] != 'M' || p[1] != 'Z') return 0;
    pe = nz_u32(p + 0x3C);
    if ((size_t) pe + 12 > n || memcmp(p + pe, "PE\0\0", 4) != 0) return 0;
    return nz_u32(p + pe + 8);
}

/* does the name sit in mods/<id>/...? returns the id length (0: no); *rel is the path inside the folder */
static size_t nc_bundle_split(const char *name, const char **rel) {
    const char *sl;
    if (strncmp(name, "mods/", 5) != 0) return 0;
    sl = strchr(name + 5, '/');
    if (!sl || !sl[1] || !nc_bundle_id_ok(name + 5, (size_t) (sl - name - 5))) return 0;
    *rel = sl + 1;
    return (size_t) (sl - name - 5);
}

/* a file inside a mod folder that is written: not a program or a checksum list */
static int nc_bundle_writable(const char *rel) {
    return !nc_has_ext(rel, ".exe") && !nc_has_ext(rel, ".py") && !nc_has_ext(rel, ".sha256") && !nc_has_ext(rel, ".zip");
}

/* 1 when the directory lists at least one mods/<id>/mod.json (so the zip is a GD mod zip), else 0. Does not extract. Uses the bundle's wider file-type allow-list. */
static int nc_bundle_detect(const uint8_t *z, size_t zn) {
    char (*names)[260] = (char (*)[260]) calloc(NZ_MAX_ENTRIES, 260);
    char err[160];
    int n, i, found = 0;
    if (!names) return 0;
    nz_allow_fn = nz_allowed_bundle;
    n = nc_zip_list(z, zn, names, NZ_MAX_ENTRIES, err, sizeof err);
    nz_allow_fn = NULL;
    for (i = 0; i < n && !found; ++i) {
        const char *rel;
        if (nc_bundle_split(names[i], &rel) && !strcmp(rel, "mod.json")) found = 1;
    }
    free(names);
    return found;
}

/* is the marker text in a mod folder one this install may replace? A Nucleus post replaces only its own ("post:<id>"); a local zip replaces any earlier local one. */
static int nc_bundle_marker_ours(const char *old, const char *tag) {
    size_t tl = strlen(tag);
    if (!strncmp(tag, "local:", 6)) return strncmp(old, "local:", 6) == 0;
    return strncmp(old, tag, tl) == 0 && (old[tl] == 0 || old[tl] == '\n' || old[tl] == '\r');
}

static void nc_bundle_note(nc_bundle_result *r, const char *fmt, const char *a, const char *b) {
    size_t l = strlen(r->notes);
    if (l + 8 >= sizeof r->notes) return;
    snprintf(r->notes + l, sizeof r->notes - l, fmt, a, b);
}

static void nc_bundle_when(uint32_t t, char *out, size_t cap) {
    time_t tt = (time_t) t;
    struct tm *g = gmtime(&tt);
    if (!g || !strftime(out, cap, "%Y-%m-%d", g)) snprintf(out, cap, "?");
}

/* 0 when at least one mod folder was written (res->notes holds what to tell the player), -1 with msg otherwise. */
static int nc_bundle_install(const char *mods_dir, const nc_post *post, const uint8_t *z, size_t zn, const nc_loose_env *env, const nc_bundle_env *benv,
                             nc_bundle_result *res, char *msg, size_t mcap, void (*logf)(const char *line)) {
    char (*names)[260] = (char (*)[260]) calloc(NZ_MAX_ENTRIES, 260);
    char err[200], line[400];
    int n, i, m, nmods = 0, rc = -1;
    char ids[NB_MAX_MODS][48];
    size_t total = 0;
    memset(res, 0, sizeof *res);
    msg[0] = '\0';
    if (!names) { snprintf(msg, mcap, "Out of memory."); return -1; }
    nz_allow_fn = nz_allowed_bundle;
    n = nc_zip_list(z, zn, names, NZ_MAX_ENTRIES, msg, mcap);
    if (n < 0) goto done;
    for (i = 0; i < n; ++i) {                                         /* which folders are mods: those with a mod.json */
        const char *rel;
        size_t il = nc_bundle_split(names[i], &rel);
        if (!strcmp(names[i], "melee-pc.exe")) res->has_exe = 1;
        if (il && !strcmp(rel, "mod.json")) {
            char id[48];
            int dup = 0;
            memcpy(id, names[i] + 5, il); id[il] = '\0';
            for (m = 0; m < nmods; ++m) if (nc_ieq(ids[m], id)) dup = 1;
            if (dup) continue;
            if (nmods >= NB_MAX_MODS) { snprintf(msg, mcap, "This zip carries more than %d mods. Install them as separate packages.", NB_MAX_MODS); goto done; }
            memcpy(ids[nmods++], id, il + 1);
        }
    }
    if (!nmods) { snprintf(msg, mcap, "This zip has no mods/<name>/mod.json folder, so there is nothing to install."); goto done; }
    if (res->has_exe) {                                               /* read its build time; the exe itself is dropped */
        uint8_t *x = NULL;
        size_t xl = 0;
        if (nc_zip_extract(z, zn, "melee-pc.exe", &x, &xl, NZ_MAX_FILE, err, sizeof err) == 0) {
            res->zip_stamp = nc_pe_stamp(x, xl < 4096 ? xl : 4096);
            free(x);
        }
        res->exe_newer = benv && benv->host_stamp && res->zip_stamp && res->zip_stamp > benv->host_stamp;
    }
    for (m = 0; m < nmods; ++m) {
        char stage[620], dest[620], marker[700], path[1100], tag[96];
        char *old;
        int bad = 0, written = 0;
        size_t il = strlen(ids[m]);
        snprintf(dest, sizeof dest, "%s/%s", mods_dir, ids[m]);
        snprintf(stage, sizeof stage, "%s/.staging-%s", mods_dir, ids[m]);
        nc_fix_slashes(dest); nc_fix_slashes(stage);
        if (post && post->local_key) snprintf(tag, sizeof tag, "local:%.40s", post->local_key);
        else snprintf(tag, sizeof tag, "post:%d", post ? post->id : 0);
        if (nc_is_dir(dest)) {                                        /* already there: ours from this post (replace), else the player's (keep) */
            snprintf(marker, sizeof marker, "%s/.nucleus-bundle", dest);
            nc_fix_slashes(marker);
            old = nc_read_all(marker, NULL);
            if (!old || !nc_bundle_marker_ours(old, tag)) {
                free(old);
                if (res->kept < NB_MAX_MODS) nc_copy(res->kept_ids[res->kept], sizeof res->kept_ids[0], ids[m]);
                ++res->kept;
                if (logf) { snprintf(line, sizeof line, "bundle: mods/%s is already installed and was not installed from this post; kept", ids[m]); logf(line); }
                continue;
            }
            free(old);
        }
        nc_rmtree(stage);
        if (nc_mkdirs(stage) != 0) { snprintf(msg, mcap, "Cannot write to the mods folder."); goto done; }
        for (i = 0; i < n && !bad; ++i) {
            const char *rel;
            size_t l2 = nc_bundle_split(names[i], &rel);
            uint8_t *data = NULL;
            size_t len = 0;
            char *sl;
            if (l2 != il || !nc_ieqn(names[i] + 5, ids[m], il) || !nc_bundle_writable(rel)) continue;
            if (nc_zip_extract(z, zn, names[i], &data, &len, NZ_MAX_FILE, err, sizeof err) < 0) { snprintf(msg, mcap, "Could not unpack %.100s: %s.", names[i], err); bad = 1; break; }
            total += len;
            if (total > NZ_MAX_TOTAL) { free(data); snprintf(msg, mcap, "This zip is too big to install at once."); bad = 1; break; }
            snprintf(path, sizeof path, "%s/%s", stage, rel);
            nc_fix_slashes(path);
            sl = strrchr(path, '\\');
            if (!sl) sl = strrchr(path, '/');
            if (sl) { char dir[1100]; snprintf(dir, sizeof dir, "%.*s", (int) (sl - path), path); if (nc_mkdirs(dir) != 0) bad = 1; }
            if (!bad && nc_write_file(path, data, len) != 0) { snprintf(msg, mcap, "Cannot write %.100s.", rel); bad = 1; }
            if (!bad && !strcmp(rel, "mod.json") && benv && benv->engine_miss) {
                char *js = (char *) malloc(len + 1), why[160];
                if (js) {
                    memcpy(js, data, len); js[len] = '\0';
                    if (benv->engine_miss(js, why, sizeof why)) { nc_bundle_note(res, " %s %s.", ids[m], why); ++res->needs_engine; }
                    free(js);
                }
            }
            free(data);
            if (!bad) ++written;
        }
        if (!bad) {
            snprintf(path, sizeof path, "%s/.nucleus-bundle", stage);
            nc_fix_slashes(path);
            if (nc_write_file(path, tag, strlen(tag)) != 0) bad = 1;
        }
        if (bad) { nc_rmtree(stage); if (!msg[0]) snprintf(msg, mcap, "Cannot write the mod in the mods folder."); goto done; }
        if (!nc_install_allowed(env, msg, mcap)) { nc_rmtree(stage); goto done; }
        if (nc_swap_dir(stage, dest) != 0) { nc_rmtree(stage); snprintf(msg, mcap, "Cannot put %s in the mods folder.", ids[m]); goto done; }
        if (res->installed < NB_MAX_MODS) nc_copy(res->ids[res->installed], sizeof res->ids[0], ids[m]);
        ++res->installed; res->files += written;
        if (nc_enabled_edit(mods_dir, ids[m], 1) != 0) { snprintf(msg, mcap, "Installed %s but could not enable it in enabled.txt.", ids[m]); goto done; }
        if (logf) { snprintf(line, sizeof line, "bundle: mods/%s: %d files", ids[m], written); logf(line); }
    }
    if (!res->installed) {
        snprintf(msg, mcap, "Nothing new to install: %s %s already in your mods folder from somewhere else.", res->kept_ids[0], res->kept > 1 ? "and the others are" : "is");
        goto done;
    }
    {
        char list[220] = "";
        size_t l = 0;
        for (i = 0; i < res->installed && i < NB_MAX_MODS; ++i) { l += (size_t) snprintf(list + l, sizeof list - l, "%s%s", i ? ", " : "", res->ids[i]); if (l >= sizeof list - 24) break; }
        snprintf(msg, mcap, "Installed %d mod%s (%s). Restart the game to use %s.", res->installed, res->installed == 1 ? "" : "s", list, res->installed == 1 ? "it" : "them");
    }
    if (res->kept) { for (i = 0; i < res->kept && i < NB_MAX_MODS; ++i) nc_bundle_note(res, " Kept your own %s.", res->kept_ids[i], ""); }
    if (res->has_exe && res->exe_newer) {
        char a[16], b[16];
        nc_bundle_when(res->zip_stamp, a, sizeof a); nc_bundle_when(benv->host_stamp, b, sizeof b);
        nc_bundle_note(res, " This zip also carries a NEWER game build (%s) than yours (%s). Your game was not changed; if a mod does not load, get that build from the post page.", a, b);
    }
    if (res->notes[0]) { size_t l = strlen(msg); if (l + 16 < mcap) snprintf(msg + l, mcap - l, " See details."); }   /* the notes themselves are res->notes (the browser shows them in the post's details) */
    rc = 0;
done:
    nz_allow_fn = NULL;
    free(names);
    if (logf) { snprintf(line, sizeof line, "bundle: %s: %.300s", rc == 0 ? "installed" : "refused", msg); logf(line); }
    return rc;
}

#endif
