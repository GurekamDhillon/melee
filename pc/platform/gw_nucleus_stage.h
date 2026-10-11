/* gw_nucleus_stage.h - installing a Nucleus CUSTOM STAGE post (a MexManager stage package) as a mod that adds a NEW stage.
 * Pure C, header-only (static), after gw_nucleus_loose.h. No game types.
 *
 * The package is a zip: `stage.json` (the fields of a MexManager project's data/stages/NNN.json), the stage file `GrXx.dat` and optionally the icon
 * the json names (assets.icon, else icon.png). The installer checks the stage file's CONTENT (an HSD archive with map_head and coll_data), refuses
 * a name that is a retail stage file (that would REPLACE a stage; a stage skin is a different post kind), and writes
 *
 *   mods/nucleus-<post>-cstage/mod.json       kind "stage", source "nucleus", the post link, the compatibility note
 *   mods/nucleus-<post>-cstage/mexstage.txt   what the runtime reads (pc/platform/gw_mex_modstage.inc)
 *   mods/nucleus-<post>-cstage/files/GrXx.dat the stage file
 *   mods/nucleus-<post>-cstage/icon.gxtex     the stage-select icon (64 x 56, RGB5A3), when the package has a PNG
 *
 * WHAT THE JSON GIVES (mapped to the StageData row words, verified against ACE's rows): mapDescPointer = word 1 (StageCallbacks*), onStageInit 3,
 * onGo 4, onStageLoad 5, onStageGo 6, onUnknown2 7, onTouchLine 8, onUnknown4 9, unknownValue 10. They are absolute PPC addresses into the retail
 * code (the clone base) and run through the existing bridge; the stage's own code blob travels inside GrXx.dat (`grFunction`).
 * WHAT IT CANNOT GIVE on a vanilla disc, said in the compat note: a sound bank above the retail ones (silent stage effects), music ids above the
 * retail ones (dropped), line types (unless the package carries the GD extension `gdLineType`, the stage falls back to one retail stage's).
 * A package whose callbacks point outside the retail dol is refused. */
#ifndef GW_NUCLEUS_STAGE_H
#define GW_NUCLEUS_STAGE_H

#include "gw_nucleus_loose.h"

#define NS_ICON_W 64
#define NS_ICON_H 56

static int ns_word_ok(double v) {
    return v == 0.0 || v == 4294967295.0 || v == -1.0 || (v >= 2147483648.0 && v < 2172649472.0); /* 0, -1, or 0x80000000 .. 0x81800000 */
}
static uint32_t ns_word(double v) { return v < 0 ? 0xFFFFFFFFu : (uint32_t) v; }

/* RGBA8 -> NS_ICON_W x NS_ICON_H, nearest neighbour, aspect kept, centred on transparent. Caller frees. */
static uint8_t *ns_fit_icon(const uint8_t *px, int w, int h) {
    uint8_t *o = (uint8_t *) calloc((size_t) NS_ICON_W * NS_ICON_H, 4);
    int dw, dh, x0, y0, x, y;
    if (!o || w < 1 || h < 1) { free(o); return NULL; }
    if ((long) w * NS_ICON_H >= (long) h * NS_ICON_W) { dw = NS_ICON_W; dh = (int) ((long) h * NS_ICON_W / w); }
    else { dh = NS_ICON_H; dw = (int) ((long) w * NS_ICON_H / h); }
    if (dw < 1) dw = 1;
    if (dh < 1) dh = 1;
    x0 = (NS_ICON_W - dw) / 2; y0 = (NS_ICON_H - dh) / 2;
    for (y = 0; y < dh; ++y) for (x = 0; x < dw; ++x) {
        const uint8_t *s = px + ((size_t) (y * h / dh) * (size_t) w + (size_t) (x * w / dw)) * 4;
        memcpy(o + ((size_t) (y0 + y) * NS_ICON_W + (size_t) (x0 + x)) * 4, s, 4);
    }
    return o;
}

/* a retail versus stage's file stem? (the stage must not REPLACE one) */
static int ns_is_retail_stem(const char *base) {
    char stem[32];
    int i;
    size_t l = strlen(base);
    if (l < 5 || l >= sizeof stem) return 0;
    snprintf(stem, sizeof stem, "%.*s", (int) (l - 4), base);
    for (i = 0; i < NC_N_STAGE_STEMS; ++i) if (nc_ieq(stem, nc_stage_stems[i])) return 1;
    return 0;
}

typedef struct {
    char name[48], file[40], icon_ref[64];
    uint32_t w[13];
    int ssm, reverb, reverb2, linetype, nbgm, bgm[8], chance[8], bgm_dropped;
    int have_line;
    char compat[400];       /* the plain-words "works / partial" note */
} ns_stage;

/* Parse and check stage.json. 0 ok, -1 with err. */
static int ns_parse_json(const char *text, ns_stage *o, char *err, size_t ecap) {
    nj_doc d;
    int root, k, pl, en, e;
    double v;
    static const char *const keys[] = { "mapDescPointer", "", "", "onStageInit", "onGo", "onStageLoad", "onStageGo", "onUnknown2", "onTouchLine", "onUnknown4", "unknownValue" };
    const char *fn;
    size_t l;
    memset(o, 0, sizeof *o);
    root = nj_parse(&d, text);
    if (root < 0 || d.n[root].type != NJ_OBJ) { snprintf(err, ecap, "stage.json is not a JSON object"); nj_free(&d); return -1; }
    nc_copy(o->name, sizeof o->name, nj_gstr(&d, root, "name", ""));
    fn = nj_gstr(&d, root, "fileName", "");
    l = strlen(nc_basename(fn));
    if (!o->name[0]) { snprintf(err, ecap, "stage.json has no name"); nj_free(&d); return -1; }
    if (l < 6 || l > 24 || !nc_has_ext(fn, ".dat") || strncmp(nc_basename(fn), "Gr", 2) != 0) { snprintf(err, ecap, "the stage file name \"%.40s\" is not a GrXx.dat", fn); nj_free(&d); return -1; }
    snprintf(o->file, sizeof o->file, "/%s", nc_basename(fn));
    for (k = 0; k < 11; ++k) {
        if (!keys[k][0]) continue;
        v = nj_gnum(&d, nj_get(&d, root, keys[k]) >= 0 ? root : -1, keys[k], 0.0);
        if (k == 10) { o->w[k] = (uint32_t) v; continue; }                     /* unknownValue is flags2, not an address */
        if (!ns_word_ok(v)) { snprintf(err, ecap, "%s (%.0f) points outside the retail code: the stage was built for another clone base", keys[k], v); nj_free(&d); return -1; }
        o->w[k] = ns_word(v);
    }
    o->ssm = (int) nj_gnum(&d, root, "soundBank", 55);
    o->reverb = (int) nj_gnum(&d, root, "reverbValue1", 1);
    o->reverb2 = (int) nj_gnum(&d, root, "reverbValue2", 1);
    o->linetype = (int) nj_gnum(&d, root, "gdLineType", -1);
    o->have_line = o->linetype >= 0 && o->linetype < 71;
    if (!o->have_line) o->linetype = -1;
    en = nj_get(&d, root, "items");
    if (en >= 0 && d.n[en].type == NJ_ARR && d.n[en].first >= 0) { snprintf(err, ecap, "this stage owns custom items, which need the item tables of an m-ex disc (not supported on a vanilla disc yet)"); nj_free(&d); return -1; }
    en = nj_get(&d, root, "assets");
    if (en >= 0) nc_copy(o->icon_ref, sizeof o->icon_ref, nj_gstr(&d, en, "icon", ""));
    pl = nj_get(&d, root, "playlist");
    en = pl >= 0 ? nj_get(&d, pl, "entries") : -1;
    if (en >= 0 && d.n[en].type == NJ_ARR) {
        for (e = d.n[en].first; e >= 0; e = d.n[e].next) {
            static const char *const idk[] = { "musicID", "musicId", "bgm", "id", "music" }, *const chk[] = { "chance", "playChance", "weight" };
            int id = -1, ch = 100, i;
            for (i = 0; i < 5; ++i) if (nj_get(&d, e, idk[i]) >= 0) { id = (int) nj_gnum(&d, e, idk[i], -1); break; }
            for (i = 0; i < 3; ++i) if (nj_get(&d, e, chk[i]) >= 0) { ch = (int) nj_gnum(&d, e, chk[i], 100); break; }
            if (id > 0 && id <= 98 && o->nbgm < 8) { o->bgm[o->nbgm] = id; o->chance[o->nbgm] = ch > 0 ? ch : 100; ++o->nbgm; }
            else ++o->bgm_dropped;
        }
    }
    nj_free(&d);
    return 0;
}

/* the compatibility note, in the words the browser shows */
static void ns_compat(ns_stage *s) {
    char part[300];
    part[0] = '\0';
    if (s->ssm > 54) { snprintf(part + strlen(part), sizeof part - strlen(part), "sound bank %d is not on a vanilla disc (stage sounds silent); ", s->ssm); s->ssm = 55; }
    if (s->bgm_dropped > 0) snprintf(part + strlen(part), sizeof part - strlen(part), "%d music entr%s not on a vanilla disc dropped; ", s->bgm_dropped, s->bgm_dropped == 1 ? "y is" : "ies are");
    if (s->nbgm == 0) snprintf(part + strlen(part), sizeof part - strlen(part), "no music; ");
    if (!s->have_line) snprintf(part + strlen(part), sizeof part - strlen(part), "collision line types default to a retail stage's; ");
    if (part[0]) snprintf(s->compat, sizeof s->compat, "Works, partial: %s", part);
    else snprintf(s->compat, sizeof s->compat, "Works: new stage with its own file, sound and music.");
    { size_t l = strlen(s->compat); while (l > 1 && (s->compat[l - 1] == ' ' || s->compat[l - 1] == ';')) s->compat[--l] = '\0'; }
}

/* Installs the post's custom stage(s). Same contract as nc_loose_install: 0 when at least one mod was written. */
static int nc_stage_install(const char *mods_dir, const nc_post *post, const nc_in_file *in, int nin, const nc_loose_env *env, nc_loose_result *res,
                            char *msg, size_t mcap, void (*logf)(const char *line)) {
    int i;
    char line[300];
    (void) env;
    memset(res, 0, sizeof *res);
    for (i = 0; i < nin; ++i) {
        uint8_t *js = NULL, *dat = NULL, *png = NULL;
        size_t jl = 0, dl = 0, pl = 0;
        char err[200], mod_id[96], stage_dir[620], final_dir[620], tmp[760], conf[16][64], target[64];
        ns_stage st;
        nc_symset cs;
        nj_buf b;
        int ncf, k;
        uint8_t *gx = NULL, *px = NULL, *fit = NULL;
        size_t gxl = 0;
        if (in[i].cat != NC_FC_ARCHIVE || !nc_has_ext(in[i].filename, ".zip")) continue;
        if (nc_zip_extract(in[i].data, in[i].len, "stage.json", &js, &jl, 1u << 20, err, sizeof err) < 0) {
            snprintf(res->first_err, sizeof res->first_err, "No stage.json in %s (%s).", nc_basename(in[i].filename), err); ++res->refused; continue;
        }
        js = (uint8_t *) realloc(js, jl + 1); js[jl] = '\0';
        if (ns_parse_json((const char *) js, &st, err, sizeof err) < 0) { snprintf(res->first_err, sizeof res->first_err, "%s.", err); ++res->refused; free(js); continue; }
        free(js);
        snprintf(target, sizeof target, "%s", st.file + 1);
        if (ns_is_retail_stem(target)) { snprintf(res->first_err, sizeof res->first_err, "%s is a retail stage's file name: this would replace that stage (use a stage skin post for that).", target); ++res->refused; continue; }
        if (nc_zip_extract(in[i].data, in[i].len, target, &dat, &dl, 24u * 1024 * 1024, err, sizeof err) < 0) {
            snprintf(res->first_err, sizeof res->first_err, "The package has no %s (%s).", target, err); ++res->refused; continue;
        }
        if (nc_dat_symbols(dat, dl, &cs, err, sizeof err) < 0 || !cs.map_head || !cs.coll_data) {
            snprintf(res->first_err, sizeof res->first_err, "%s is not a stage file (%s).", target, cs.map_head && cs.coll_data ? err : "no map_head / coll_data"); ++res->refused; free(dat);
            if (cs.h) nc_symset_free(&cs);
            continue;
        }
        nc_symset_free(&cs);
        ns_compat(&st);
        { /* the icon: the json's name, else icon.png; any failure just leaves the stage with a text tile */
            const char *want = st.icon_ref[0] ? nc_basename(st.icon_ref) : "icon.png";
            int w = 0, h = 0, comp = 0;
            if (nc_zip_extract(in[i].data, in[i].len, want, &png, &pl, 8u * 1024 * 1024, err, sizeof err) == 0 && pl > 8 && !memcmp(png, "\x89PNG", 4)) {
                px = stbi_load_from_memory(png, (int) pl, &w, &h, &comp, 4);
                if (px && w <= 2048 && h <= 2048 && (fit = ns_fit_icon(px, w, h)) != NULL) {
                    if (nc_rgba_to_gxtex(fit, NS_ICON_W, NS_ICON_H, &gx, &gxl) != 0) gx = NULL;
                }
                if (px) stbi_image_free(px);
                free(fit);
            }
            free(png);
        }
        nc_post_mod_id(post, "cstage", mod_id, sizeof mod_id);
        snprintf(stage_dir, sizeof stage_dir, "%s/.staging-%s", mods_dir, mod_id);
        snprintf(final_dir, sizeof final_dir, "%s/%s", mods_dir, mod_id);
        nc_fix_slashes(stage_dir); nc_fix_slashes(final_dir);
        nc_rmtree(stage_dir);
        snprintf(tmp, sizeof tmp, "%s/files/%s", stage_dir, target);
        nc_fix_slashes(tmp);
        { char d[760], *sl = strrchr(tmp, '\\'); if (!sl) sl = strrchr(tmp, '/'); if (sl) { snprintf(d, sizeof d, "%.*s", (int) (sl - tmp), tmp); nc_mkdirs(d); } }
        if (nc_write_file(tmp, dat, dl) != 0) { snprintf(res->first_err, sizeof res->first_err, "Cannot write to the mods folder."); nc_rmtree(stage_dir); free(dat); free(gx); continue; }
        if (gx) { snprintf(tmp, sizeof tmp, "%s/icon.gxtex", stage_dir); nc_fix_slashes(tmp); nc_write_file(tmp, gx, gxl); }
        {   /* mexstage.txt */
            nj_buf t;
            int j;
            memset(&t, 0, sizeof t);
            nj_printf(&t, "name=%s\nfile=%s\nssm=%d\nreverb=%d\nreverb2=%d\n", st.name, st.file, st.ssm, st.reverb, st.reverb2);
            if (!post->local_key) nj_printf(&t, "post=https://%s/post/%d\n", NC_SITE, post->id);
            if (gx) nj_puts(&t, "icon=icon.gxtex\n");
            nj_printf(&t, "callbacks=%08X\nflags2=%X\n", (unsigned) st.w[1], (unsigned) st.w[10]);
            for (j = 3; j <= 9; ++j) nj_printf(&t, "w%d=%08X\n", j, (unsigned) st.w[j]);
            if (st.have_line) nj_printf(&t, "linetype=%d\n", st.linetype);
            if (st.nbgm > 0) { nj_puts(&t, "bgm="); for (j = 0; j < st.nbgm; ++j) nj_printf(&t, "%s%d:%d", j ? "," : "", st.bgm[j], st.chance[j]); nj_puts(&t, "\n"); }
            snprintf(tmp, sizeof tmp, "%s/mexstage.txt", stage_dir); nc_fix_slashes(tmp);
            k = t.bad || nc_write_file(tmp, t.s, t.n) != 0;
            free(t.s);
            if (k) { snprintf(res->first_err, sizeof res->first_err, "Cannot write mexstage.txt."); nc_rmtree(stage_dir); free(dat); free(gx); continue; }
        }
        ncf = nc_find_conflicts(mods_dir, mod_id, (char (*)[64]) target, 1, conf, 16);
        memset(&b, 0, sizeof b);
        nj_puts(&b, "{\n  \"id\": "); nj_qstr(&b, mod_id);
        nj_puts(&b, ",\n  \"name\": "); nj_qstr(&b, st.name);
        nj_puts(&b, ",\n  \"version\": "); nj_qstr(&b, post->updated);
        nj_puts(&b, ",\n  \"kind\": \"stage\",\n  \"authors\": "); nj_qstr(&b, post->author);
        { char desc[800];
            if (post->local_key) snprintf(desc, sizeof desc, "New stage from local file: %s. Online, both players need it. %s", post->local_file, st.compat);
            else snprintf(desc, sizeof desc, "New stage from SSBM Nucleus: %s/post/%d. Online, both players need it. %s", NC_SITE, post->id, st.compat);
            nj_puts(&b, ",\n  \"description\": "); nj_qstr(&b, desc);
        }
        nj_puts(&b, ",\n  \"conflicts\": [");
        for (k = 0; k < ncf; ++k) { if (k) nj_puts(&b, ", "); nj_qstr(&b, conf[k]); }
        nj_puts(&b, "]"); nc_post_source(&b, post);
        nj_puts(&b, ",\n    \"category\": \"cstage\",\n    \"stage\": "); nj_qstr(&b, st.name);
        nj_puts(&b, ",\n    \"compat\": "); nj_qstr(&b, st.compat);
        nj_puts(&b, ",\n    \"targets\": ["); nj_qstr(&b, target);
        nj_printf(&b, "],\n    \"files\": [{\"id\": %d, \"size\": %u, \"fnv64\": \"%016llx\"}]\n  }\n}\n", in[i].file_id, (unsigned) in[i].len, (unsigned long long) nc_fnv64(in[i].data, in[i].len));
        snprintf(tmp, sizeof tmp, "%s/mod.json", stage_dir); nc_fix_slashes(tmp);
        k = b.bad || nc_write_file(tmp, b.s, b.n) != 0;
        free(b.s); free(dat); free(gx);
        if (k) { snprintf(res->first_err, sizeof res->first_err, "Cannot write mod.json."); nc_rmtree(stage_dir); continue; }
        if (!nc_install_allowed(env, res->first_err, sizeof res->first_err)) { nc_rmtree(stage_dir); ++res->refused; break; }
        if (nc_swap_dir(stage_dir, final_dir) != 0) {
            nc_rmtree(stage_dir);
            snprintf(res->first_err, sizeof res->first_err, "Could not replace %s (the game may be using it). Restart and try again.", mod_id);
            continue;
        }
        { char off[8][64]; nc_loose_set_enabled(mods_dir, mod_id, 1, off, 8); }
        for (k = 0; k < ncf && res->nturned_off < 4; ++k) snprintf(res->turned_off[res->nturned_off++], 64, "%s", conf[k]);
        if (res->nmods < 8) snprintf(res->mods[res->nmods++], 64, "%s", mod_id);
        ++res->installed; ++res->files;
        if (logf) { snprintf(line, sizeof line, "install %d: %s: custom stage \"%s\" (%s). %s", post->id, mod_id, st.name, st.file, st.compat); logf(line); }
        snprintf(res->first_err, sizeof res->first_err, "%s", st.compat);
    }
    if (res->installed > 0) {
        snprintf(msg, mcap, "Installed the stage. Restart the game to use it. %s", res->first_err);
        return 0;
    }
    snprintf(msg, mcap, "%s", res->first_err[0] ? res->first_err : "No stage package (.zip with a stage.json) in this post.");
    return -1;
}

#endif
