/* Local zip input for the Nucleus installers. No network and no archive-path
 * extraction: dispatch to the same MexManager/override/content validators and
 * staged writers. Included by the file worker and the disc-free native tests. */
#ifndef GW_MODS_ZIP_H
#define GW_MODS_ZIP_H
#include "gw_nucleus_fighter.h"
#include "gw_nucleus_stage.h"

/* Costumes use the same content typing, manifest, staging and next-boot enable
 * rules as ne_install_impl. Local archives have no API-provided art/slot hints. */
static int nlz_skin_install(const char *mods, const nc_post *post, const nc_in_file *in, const nc_dat_info *info, int nin,
                            const nc_loose_env *env, nc_loose_result *res, char *msg, size_t mcap) {
    int i, j;
    for (i = 0; i < nin; ++i) {
        nc_skin_mod *s;
        char stage[620], final[620], files[720], path[800];
        nj_buf b = {0};
        int bad = 0;
        for (j = 0; j < i; ++j) if (info[j].fighter == info[i].fighter) break;
        if (j != i) continue;
        s = (nc_skin_mod *) calloc(1, sizeof *s);
        if (!s) { snprintf(msg, mcap, "Out of memory."); return -1; }
        s->post = post;
        nc_post_mod_id(post, nc_fighters[info[i].fighter].target, s->mod_id, sizeof s->mod_id);
        nc_copy(s->target, sizeof s->target, nc_fighters[info[i].fighter].target);
        nc_copy(s->title, sizeof s->title, post->title); nc_copy(s->updated, sizeof s->updated, post->updated);
        snprintf(stage, sizeof stage, "%s/.staging-%s", mods, s->mod_id);
        snprintf(final, sizeof final, "%s/%s", mods, s->mod_id);
        snprintf(files, sizeof files, "%s/files/skins/%s", stage, s->mod_id);
        nc_fix_slashes(stage); nc_fix_slashes(final); nc_fix_slashes(files);
        nc_rmtree(stage);
        if (nc_mkdirs(files) != 0) bad = 1;
        for (j = i; !bad && j < nin; ++j) if (info[j].fighter == info[i].fighter) {
            nc_skin_costume *c = &s->c[s->n];
            nc_costume_name(post->title, info[j].colour, nin > 1, c->name, sizeof c->name);
            snprintf(c->dat, sizeof c->dat, "skins/%s/%d.dat", s->mod_id, s->n + 1);
            nc_copy(c->joint, sizeof c->joint, info[j].joint); nc_copy(c->matanim, sizeof c->matanim, info[j].matanim);
            c->like = info[j].colour_idx > 0 ? info[j].colour_idx : 0;
            snprintf(path, sizeof path, "%s/%d.dat", files, s->n + 1);
            bad = nc_write_file(path, in[j].data, in[j].len) != 0;
            ++s->n;
        }
        nc_skin_json(s, &b);
        snprintf(path, sizeof path, "%s/mod.json", stage);
        if (b.bad || !b.s || nc_write_file(path, b.s, b.n) != 0) bad = 1;
        free(b.s);
        if (!nc_install_allowed(env, msg, mcap)) { nc_rmtree(stage); free(s); return -1; }
        if (!bad && nc_swap_dir(stage, final) != 0) bad = 1;
        if (bad) { nc_rmtree(stage); free(s); snprintf(msg, mcap, "Cannot write the costume in the mods folder."); return -1; }
        /* Publishing already changed the next-boot registry, even if enabling
         * or a later group fails. Keep restart protection in that case too. */
        if (res->nmods < 8) nc_copy(res->mods[res->nmods++], 64, s->mod_id);
        ++res->installed; res->files += s->n;
        if (nc_enabled_edit(mods, s->mod_id, 1) != 0) { free(s); snprintf(msg, mcap, "Could not enable the costume in enabled.txt."); return -1; }
        free(s);
    }
    snprintf(msg, mcap, "Installed %d costume mod(s) from local file. Restart the game to use them.", res->installed);
    return 0;
}

static int nlz_install_data(const char *mods, const char *filename, const uint8_t *z, size_t zn, const nc_loose_env *env,
                            nc_loose_result *res, char *msg, size_t mcap, void (*logf)(const char *)) {
    char (*names)[260] = (char (*)[260]) calloc(NZ_MAX_ENTRIES, 260);
    nc_post p = {0};
    nc_in_file in[NC_MAX_FILES_PER_MOD] = {0};
    nc_dat_info info[NC_MAX_FILES_PER_MOD];
    char stems[NC_MAX_FILES_PER_MOD][64], key[17], title[80], when[24], err[200];
    int n, i, nf = 0, ns = 0, nin = 0, nskin = 0, rc = -1;
    memset(res, 0, sizeof *res);
    msg[0] = 0;
    if (!names) { snprintf(msg, mcap, "Out of memory."); return -1; }
    n = nc_zip_list(z, zn, names, NZ_MAX_ENTRIES, msg, mcap);
    if (n < 0) goto done;
    for (i = 0; i < n; ++i) {
        nf += nc_ieq(nc_basename(names[i]), "fighter.json");
        ns += nc_ieq(nc_basename(names[i]), "stage.json");
    }
    if (nf + ns > 1) { snprintf(msg, mcap, "The zip contains several fighter/stage packages. Install one package at a time."); goto done; }
    snprintf(key, sizeof key, "%016llx", (unsigned long long) nc_fnv64(z, zn));
    nc_ascii(title, sizeof title, nc_basename(filename));
    nc_iso((int64_t) time(NULL), when, sizeof when);
    p.title = title; p.author = ""; p.page = ""; p.updated = key; p.installed_at = when; p.stage = "";
    p.local_key = key; p.local_file = nc_basename(filename);
    if (nf || ns) {
        nc_in_file pack = {0};
        pack.cat = NC_FC_ARCHIVE; pack.filename = filename; pack.stem = ""; pack.data = z; pack.len = zn;
        rc = nf ? nc_fighter_install(mods, &p, &pack, 1, env, res, msg, mcap, logf)
                : nc_stage_install(mods, &p, &pack, 1, env, res, msg, mcap, logf);
        goto done;
    }
    /* Inspect everything before any writes. Art and readmes are not mounted. */
    for (i = 0; i < n; ++i) {
        uint8_t *data = NULL;
        size_t len = 0;
        int cat, supported = 0;
        if (nc_has_ext(names[i], ".txt") || nc_has_ext(names[i], ".md")) continue;
        if (nin >= NC_MAX_FILES_PER_MOD) { snprintf(msg, mcap, "Too many mod files (maximum %d).", NC_MAX_FILES_PER_MOD); goto done; }
        cat = nc_file_category("", names[i], stems[nin], sizeof stems[nin], &supported);
        if (cat == NC_FC_IMAGE || nc_has_ext(names[i], ".jpeg") || nc_has_ext(names[i], ".gxtex")) continue;
        if (cat == NC_FC_ARCHIVE || nc_has_ext(names[i], ".json")) { snprintf(msg, mcap, "No fighter.json or stage.json in this zip. Use a MexManager package or supported costume/stage/effect/menu/audio files."); goto done; }
        if (nc_zip_extract(z, zn, names[i], &data, &len, cat == NC_FC_AUDIO ? NZ_MAX_FILE : 24u * 1024 * 1024, err, sizeof err) < 0) {
            snprintf(msg, mcap, "%s", err); goto done;
        }
        if (cat == NC_FC_OTHER && nc_dat_costume(data, len, &info[nin], err, sizeof err) == 0 && info[nin].fighter >= 0 && nc_fighters[info[nin].fighter].installable) {
            cat = NC_FC_COSTUME; ++nskin;
        } else if (!supported) {
            free(data); snprintf(msg, mcap, "Unsupported mod file: %.100s. Whole fighters need a MexManager fighter.json package.", names[i]); goto done;
        }
        in[nin].cat = cat; in[nin].filename = names[i]; in[nin].stem = stems[nin]; in[nin].data = data; in[nin].len = len;
        ++nin;
    }
    if (!nin) { snprintf(msg, mcap, "No supported mod files in this zip."); goto done; }
    if (nskin && nskin != nin) { snprintf(msg, mcap, "This zip mixes costumes and disc overrides. Install them as separate packages."); goto done; }
    if (!nskin) {
        for (i = 0; i < nin; ++i) {
            nc_plan plan;
            if (nc_loose_plan_file(&in[i], env, &plan, msg, mcap) < 0) goto done;
        }
    }
    rc = nskin ? nlz_skin_install(mods, &p, in, info, nin, env, res, msg, mcap)
               : nc_loose_install(mods, &p, in, nin, env, res, msg, mcap, logf);
done:
    if (rc < 0 && res->installed > 0) {
        char why[300]; nc_copy(why, sizeof why, msg);
        snprintf(msg, mcap, "Installed %d mod(s), then stopped: %s Restart before changing mods.", res->installed, why);
    }
    for (i = 0; i < nin; ++i) free((void *) in[i].data);
    free(names);
    if (logf) { char line[500]; snprintf(line, sizeof line, "local file %.120s: %s: %.300s", nc_basename(filename), rc == 0 ? "installed" : "refused", msg); logf(line); }
    return rc;
}

static int nlz_path_ok(const char *path) {
    return path && path[0] && strlen(path) < 500 && nc_has_ext(path, ".zip") && !strstr(path, "://") &&
           !((path[0] == '/' || path[0] == '\\') && (path[1] == '/' || path[1] == '\\'));
}

#ifdef _WIN32
#define NLZ_INCOMING_MAX 128
typedef struct { char name[260]; uint64_t size, stamp; } nlz_file;
/* Directory-only scan on the worker. Reparse points and folders are never offered. */
static int nlz_scan_incoming(const char *mods, nlz_file *out, int cap) {
    char path[760];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    int n = 0;
    snprintf(path, sizeof path, "%s/_incoming", mods); nc_mkdirs(path);
    snprintf(path, sizeof path, "%s/_incoming/*", mods); nc_fix_slashes(path);
    h = FindFirstFileA(path, &fd);
    if (h == INVALID_HANDLE_VALUE) return 0;
    do {
        if (fd.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT)) continue;
        if (!nc_has_ext(fd.cFileName, ".zip") || n >= cap) continue;
        nc_copy(out[n].name, sizeof out[n].name, fd.cFileName);
        out[n].size = ((uint64_t) fd.nFileSizeHigh << 32) | fd.nFileSizeLow;
        out[n].stamp = ((uint64_t) fd.ftLastWriteTime.dwHighDateTime << 32) | fd.ftLastWriteTime.dwLowDateTime;
        ++n;
    } while (FindNextFileA(h, &fd));
    FindClose(h);
    return n;
}
#endif

static int nlz_install_file(const char *mods, const char *path, const nc_loose_env *env, nc_loose_result *res,
                            char *msg, size_t mcap, void (*logf)(const char *)) {
    FILE *fp;
    long len;
    uint8_t *z;
    int rc;
    memset(res, 0, sizeof *res);
    if (!nlz_path_ok(path)) { snprintf(msg, mcap, "Choose a local .zip file (no URLs or network paths, maximum 499 path characters)."); return -1; }
    fp = fopen(path, "rb");
    if (!fp) { snprintf(msg, mcap, "Cannot open local zip: %.160s", path); return -1; }
    if (fseek(fp, 0, SEEK_END) || (len = ftell(fp)) < 0 || len > NZ_MAX_ARCHIVE || fseek(fp, 0, SEEK_SET)) {
        fclose(fp); snprintf(msg, mcap, "Cannot read zip or file too large (maximum 160 MiB)."); return -1;
    }
    z = (uint8_t *) malloc(len ? (size_t) len : 1);
    if (!z || fread(z, 1, (size_t) len, fp) != (size_t) len) { free(z); fclose(fp); snprintf(msg, mcap, "Cannot read local zip."); return -1; }
    fclose(fp);
    rc = nlz_install_data(mods, path, z, (size_t) len, env, res, msg, mcap, logf);
    free(z);
    return rc;
}
#endif
