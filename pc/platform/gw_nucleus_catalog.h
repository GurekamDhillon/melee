/* gw_nucleus_catalog.h - the Nucleus catalog (mods and files), its on-disk cache, search / filter / sort, and the sync algorithm. Pure C over an
 * injected fetch function: gw_nucleus_engine.inc supplies the HTTP, the clock and the sleep; the native test supplies recorded JSON.
 * See gw_nucleus_core.h for the rules this follows (the API owner's etiquette). */
#ifndef GW_NUCLEUS_CATALOG_H
#define GW_NUCLEUS_CATALOG_H

#include "gw_nucleus_core.h"

typedef struct {
    int id;
    char filename[100], character[24], color[16];
    char csp[240], stock[240];          /* media URLs, with NC_MEDIA_BASE stripped when they start with it */
    int fighter;                        /* nc_fighters index, or -1 */
    int colour_idx;                     /* the original costume this colour is, or -1 */
    int costume;                        /* 1: a costume .dat / .usd we can try to install */
    int mismatch;                       /* the API's character and the filename's slot code name different fighters */
    int zip;                            /* 1: the post is a zip (the API lists no per-file download_url / file_url): the file is an entry of the mod's own download */
} nc_file;

typedef struct {
    int id, type;
    char title[80], author[40], tags[96], desc[320], page[120], created[24], updated[24];
    int downloads, likes;
    int f0, nf, nf_total;               /* the files in the pool: [f0, f0 + nf); nf_total: what the API listed (may exceed NC_MAX_FILES_PER_MOD) */
    unsigned fmask;                     /* bit i: a costume file for nc_fighters[i] */
} nc_mod;

typedef struct {
    nc_mod *m; int n, cap;
    nc_file *f; int nf, fcap;
    int *gone; int ngone;               /* ids the removed feed reported (kept so an installed copy can be marked) */
    int *ht; int htcap;
    int version;                        /* bumped by every change: a cached filter is stale when it differs */
} nc_catalog;

static void nc_cat_free(nc_catalog *c) {
    free(c->m); free(c->f); free(c->gone); free(c->ht);
    memset(c, 0, sizeof *c);
}

/* ---- id lookup ---------------------------------------------------------------------------------------- */

static void nc_ht_rebuild(nc_catalog *c) {
    int i, cap = 64;
    while (cap < c->n * 2 + 2) cap *= 2;
    free(c->ht);
    c->ht = (int *) malloc((size_t) cap * sizeof(int));
    c->htcap = c->ht ? cap : 0;
    if (!c->ht) return;
    for (i = 0; i < cap; ++i) c->ht[i] = -1;
    for (i = 0; i < c->n; ++i) {
        unsigned h = ((unsigned) c->m[i].id * 2654435761u) & (unsigned) (cap - 1);
        while (c->ht[h] >= 0) h = (h + 1) & (unsigned) (cap - 1);
        c->ht[h] = i;
    }
}

static int nc_cat_find(nc_catalog *c, int id) {
    unsigned h;
    if (!c->ht || c->htcap < c->n * 2) nc_ht_rebuild(c);
    if (!c->ht) return -1;
    h = ((unsigned) id * 2654435761u) & (unsigned) (c->htcap - 1);
    while (c->ht[h] >= 0) {
        if (c->m[c->ht[h]].id == id) return c->ht[h];
        h = (h + 1) & (unsigned) (c->htcap - 1);
    }
    return -1;
}

/* ---- slots ------------------------------------------------------------------------------------------- */

/* The slot code in a messy filename ("Crash Bandicoot - Fox- PlFxLa (No Jacket).dat" -> pl "Fx", col "La"). Returns 1 when a Pl<Xx> is there. */
static int nc_slot_from_filename(const char *fn, char pl[3], char col[3]) {
    const char *p;
    pl[0] = pl[1] = col[0] = col[1] = '\0';
    pl[2] = col[2] = '\0';
    for (p = fn; *p; ++p) {
        if (p[0] != 'P' || p[1] != 'l') continue;
        if (p != fn && isalpha((unsigned char) p[-1])) continue;
        if (!isupper((unsigned char) p[2]) || !islower((unsigned char) p[3])) continue;
        pl[0] = p[2]; pl[1] = p[3];
        if (isupper((unsigned char) p[4]) && islower((unsigned char) p[5]) && !isalpha((unsigned char) p[6])) { col[0] = p[4]; col[1] = p[5]; }
        return 1;
    }
    return 0;
}

static const char *nc_colour_code(const char *name) {
    static const struct { const char *name, *code; } t[] = {
        { "default", "Nr" }, { "red", "Re" }, { "blue", "Bu" }, { "green", "Gr" }, { "yellow", "Ye" }, { "white", "Wh" }, { "black", "Bk" },
        { "pink", "Pi" }, { "aqua", "Aq" }, { "lavender", "La" }, { "lavendar", "La" }, { "orange", "Or" }, { "gray", "Gy" }, { "grey", "Gy" },
    };
    size_t i;
    if (!name) return NULL;
    for (i = 0; i < sizeof t / sizeof t[0]; ++i) if (nc_ieq(name, t[i].name)) return t[i].code;
    return NULL;
}

/* the costume index a colour code is for a fighter (Nr = 0), or -1 */
static int nc_colour_index(int fighter, const char *code) {
    const char *p;
    int idx = 0;
    if (fighter < 0 || fighter >= NC_NFIGHTERS) return -1;
    if (!code || !*code) return 0;
    for (p = nc_fighters[fighter].colours; *p; ++idx) {
        const char *e = strchr(p, ',');
        size_t n = e ? (size_t) (e - p) : strlen(p);
        if (n == strlen(code) && !strncmp(p, code, n)) return idx;
        if (!e) break;
        p = e + 1;
    }
    return -1;
}

static int nc_has_ext(const char *fn, const char *ext) {
    size_t a = strlen(fn), b = strlen(ext);
    return a >= b && nc_ieq(fn + a - b, ext);
}

static const char *nc_media_strip(const char *url) {
    size_t n = strlen(NC_MEDIA_BASE);
    return strncmp(url, NC_MEDIA_BASE, n) == 0 ? url + n : url;
}
static void nc_media_full(const char *stored, char *out, size_t cap) {
    if (!stored[0]) { out[0] = '\0'; return; }
    if (!strncmp(stored, "http", 4)) snprintf(out, cap, "%s", stored);
    else snprintf(out, cap, "%s%s", NC_MEDIA_BASE, stored);
}
static void nc_download_url(int mod, int file, char *out, size_t cap) {
    if (file > 0) snprintf(out, cap, "%s/mods/%d/download?file=%d", nc_api_base, mod, file);
    else snprintf(out, cap, "%s/mods/%d/download", nc_api_base, mod);
}

/* ---- parsing one mod ----------------------------------------------------------------------------------- */

static void nc_classify_file(nc_file *f, const char *file_type) {
    char pl[3], col[3];
    int slot = nc_slot_from_filename(f->filename, pl, col), nameF = nc_fighter_by_name(f->character), slotF = slot ? nc_fighter_by_pl(pl) : -1;
    const char *want = nc_colour_code(f->color);
    int ext_ok = nc_has_ext(f->filename, ".dat") || nc_has_ext(f->filename, ".usd");
    f->fighter = nameF >= 0 ? nameF : slotF;
    f->mismatch = nameF >= 0 && slotF >= 0 && nameF != slotF;
    f->costume = ext_ok && (!strcmp(file_type, "character_dat") || (!strcmp(file_type, "other") && slot));
    f->colour_idx = f->fighter >= 0 ? nc_colour_index(f->fighter, slot && col[0] ? col : want) : -1;
    if (slot && col[0] && want && strcmp(col, want) != 0 && f->colour_idx >= 0 && nc_colour_index(f->fighter, want) >= 0) f->mismatch = 1;
}

static void nc_cat_remove_at(nc_catalog *c, int at) {
    memmove(&c->m[at], &c->m[at + 1], (size_t) (c->n - at - 1) * sizeof c->m[0]);
    --c->n;
    free(c->ht); c->ht = NULL; c->htcap = 0;
    ++c->version;
}

/* Insert or replace the mod in JSON node `node`. Returns 1 added, 2 replaced, 0 skipped (no id). */
static int nc_cat_apply(nc_catalog *c, const nj_doc *d, int node) {
    nc_mod m;
    int id = (int) nj_gnum(d, node, "id", -1), at, files, e, nfiles = 0, listed = 0;
    const char *s;
    if (id <= 0 || d->n[node].type != NJ_OBJ) return 0;
    memset(&m, 0, sizeof m);
    m.id = id;
    m.type = nc_type_of(nj_gstr(d, node, "type", "other"));
    nc_ascii(m.title, sizeof m.title, nj_gstr(d, node, "title", ""));
    nc_ascii(m.author, sizeof m.author, nj_gstr(d, node, "author", ""));
    nc_ascii(m.desc, sizeof m.desc, nj_gstr(d, node, "description", ""));
    nc_copy(m.page, sizeof m.page, nj_gstr(d, node, "page_url", ""));
    nc_copy(m.created, sizeof m.created, nj_gstr(d, node, "created_at", ""));
    nc_copy(m.updated, sizeof m.updated, nj_gstr(d, node, "updated_at", ""));
    m.downloads = (int) nj_gnum(d, node, "download_count", 0);
    m.likes = (int) nj_gnum(d, node, "like_count", 0);
    s = NULL;
    e = nj_get(d, node, "tags");
    if (e >= 0 && d->n[e].type == NJ_ARR) {
        int t;
        char tmp[200];
        tmp[0] = '\0';
        for (t = d->n[e].first; t >= 0; t = d->n[t].next) {
            size_t l = strlen(tmp);
            if (d->n[t].type != NJ_STR || l > 150) continue;
            snprintf(tmp + l, sizeof tmp - l, "%s%s", l ? ", " : "", d->pool + d->n[t].str);
        }
        nc_ascii(m.tags, sizeof m.tags, tmp);
    }
    (void) s;
    files = nj_get(d, node, "files");
    m.f0 = c->nf;
    if (files >= 0 && d->n[files].type == NJ_ARR) {
        for (e = d->n[files].first; e >= 0; e = d->n[e].next) {
            nc_file f;
            char tmp[260];
            ++listed;
            if (nfiles >= NC_MAX_FILES_PER_MOD || d->n[e].type != NJ_OBJ) continue;
            if (m.type != NC_T_COSTUME) continue;      /* other kinds are shown, not installed: their file lists are not kept */
            memset(&f, 0, sizeof f);
            f.id = (int) nj_gnum(d, e, "id", 0);
            nc_ascii(f.filename, sizeof f.filename, nj_gstr(d, e, "filename", ""));
            nc_ascii(f.character, sizeof f.character, nj_gstr(d, e, "character", ""));
            nc_ascii(f.color, sizeof f.color, nj_gstr(d, e, "color", ""));
            snprintf(tmp, sizeof tmp, "%s", nc_media_strip(nj_gstr(d, e, "csp_url", "")));
            if (strlen(tmp) < sizeof f.csp) nc_copy(f.csp, sizeof f.csp, tmp);
            snprintf(tmp, sizeof tmp, "%s", nc_media_strip(nj_gstr(d, e, "stock_url", "")));
            if (strlen(tmp) < sizeof f.stock) nc_copy(f.stock, sizeof f.stock, tmp);
            nc_classify_file(&f, nj_gstr(d, e, "file_type", ""));
            {   /* explicit nulls (not absent keys) mean the file is only reachable inside the mod's zip */
                int du = nj_get(d, e, "download_url"), fu = nj_get(d, e, "file_url");
                f.zip = (du >= 0 && d->n[du].type != NJ_STR && fu >= 0 && d->n[fu].type != NJ_STR) || (int) nj_gnum(d, e, "zip", 0) == 1;
            }
            if (f.id <= 0) continue;
            if (c->nf == c->fcap) {
                int nc = c->fcap ? c->fcap * 2 : 1024;
                nc_file *p = (nc_file *) realloc(c->f, (size_t) nc * sizeof *p);
                if (!p) return 0;
                c->f = p; c->fcap = nc;
            }
            c->f[c->nf++] = f;
            ++nfiles;
            if (f.costume && f.fighter >= 0) m.fmask |= 1u << f.fighter;
        }
    }
    m.nf = nfiles;
    m.nf_total = (int) nj_gnum(d, node, "files_total", listed);
    at = nc_cat_find(c, id);
    if (at >= 0) { c->m[at] = m; ++c->version; return 2; }
    if (c->n == c->cap) {
        int nc = c->cap ? c->cap * 2 : 512;
        nc_mod *p = (nc_mod *) realloc(c->m, (size_t) nc * sizeof *p);
        if (!p) return 0;
        c->m = p; c->cap = nc;
    }
    c->m[c->n++] = m;
    free(c->ht); c->ht = NULL; c->htcap = 0;
    ++c->version;
    return 1;
}

/* drop mods nothing points at any more: the file pool only grows while syncing, so it is compacted before a save */
static void nc_cat_compact(nc_catalog *c) {
    nc_file *nf;
    int i, w = 0;
    nf = (nc_file *) malloc((size_t) (c->nf > 0 ? c->nf : 1) * sizeof *nf);
    if (!nf) return;
    for (i = 0; i < c->n; ++i) {
        memcpy(nf + w, c->f + c->m[i].f0, (size_t) c->m[i].nf * sizeof *nf);
        c->m[i].f0 = w;
        w += c->m[i].nf;
    }
    free(c->f);
    c->f = nf; c->nf = w; c->fcap = c->nf > 0 ? c->nf : 1;
}

static void nc_cat_add_gone(nc_catalog *c, int id) {
    int i;
    int *p;
    for (i = 0; i < c->ngone; ++i) if (c->gone[i] == id) return;
    if (c->ngone >= 4000) memmove(c->gone, c->gone + 1, (size_t) (--c->ngone) * sizeof(int));
    p = (int *) realloc(c->gone, (size_t) (c->ngone + 1) * sizeof(int));
    if (!p) return;
    c->gone = p;
    c->gone[c->ngone++] = id;
}
static int nc_cat_is_gone(const nc_catalog *c, int id) {
    int i;
    for (i = 0; i < c->ngone; ++i) if (c->gone[i] == id) return 1;
    return 0;
}

/* a mod the removed feed names: out of the catalog, remembered as gone */
static void nc_cat_remove_id(nc_catalog *c, int id) {
    int at = nc_cat_find(c, id);
    if (at >= 0) nc_cat_remove_at(c, at);
    nc_cat_add_gone(c, id);
    ++c->version;
}

/* a mod that is back after having been removed */
static void nc_cat_unmark_gone(nc_catalog *c, int id) {
    int i;
    for (i = 0; i < c->ngone; ++i) if (c->gone[i] == id) { memmove(c->gone + i, c->gone + i + 1, (size_t) (c->ngone - i - 1) * sizeof(int)); --c->ngone; return; }
}

/* ---- cache file (JSON lines) ------------------------------------------------------------------------------ */

static void nc_write_mod(nj_buf *b, const nc_catalog *c, const nc_mod *m) {
    int i;
    char full[320];
    nj_printf(b, "{\"id\":%d,\"type\":", m->id); nj_qstr(b, nc_type_api[m->type]);
    nj_puts(b, ",\"title\":"); nj_qstr(b, m->title);
    nj_puts(b, ",\"author\":"); nj_qstr(b, m->author);
    nj_puts(b, ",\"description\":"); nj_qstr(b, m->desc);
    nj_puts(b, ",\"tags\":["); if (m->tags[0]) nj_qstr(b, m->tags); nj_puts(b, "]");
    nj_puts(b, ",\"page_url\":"); nj_qstr(b, m->page);
    nj_puts(b, ",\"created_at\":"); nj_qstr(b, m->created);
    nj_puts(b, ",\"updated_at\":"); nj_qstr(b, m->updated);
    nj_printf(b, ",\"download_count\":%d,\"like_count\":%d,\"files_total\":%d,\"files\":[", m->downloads, m->likes, m->nf_total);
    for (i = 0; i < m->nf; ++i) {
        const nc_file *f = &c->f[m->f0 + i];
        const char *ft = f->costume ? "character_dat" : "other";
        nj_printf(b, "%s{\"id\":%d,\"file_type\":\"%s\",\"filename\":", i ? "," : "", f->id, ft); nj_qstr(b, f->filename);
        nj_puts(b, ",\"character\":"); nj_qstr(b, f->character);
        nj_puts(b, ",\"color\":"); nj_qstr(b, f->color);
        nc_media_full(f->csp, full, sizeof full); nj_puts(b, ",\"csp_url\":"); nj_qstr(b, full);
        nc_media_full(f->stock, full, sizeof full); nj_puts(b, ",\"stock_url\":"); nj_qstr(b, full);
        if (f->zip) nj_puts(b, ",\"zip\":1");
        nj_puts(b, "}");
    }
    nj_puts(b, "]}\n");
}

/* Write the catalog to path (tmp + rename). Returns 0 or -1. */
static int nc_cat_save(nc_catalog *c, const char *path) {
    nj_buf b;
    char tmp[512];
    FILE *fp;
    int i;
    memset(&b, 0, sizeof b);
    nc_cat_compact(c);
    nj_puts(&b, "{\"gone\":[");
    for (i = 0; i < c->ngone; ++i) nj_printf(&b, "%s%d", i ? "," : "", c->gone[i]);
    nj_puts(&b, "]}\n");
    for (i = 0; i < c->n; ++i) nc_write_mod(&b, c, &c->m[i]);
    if (b.bad) { free(b.s); return -1; }
    snprintf(tmp, sizeof tmp, "%s.tmp", path);
    fp = fopen(tmp, "wb");
    if (!fp) { free(b.s); return -1; }
    if (fwrite(b.s, 1, b.n, fp) != b.n) { fclose(fp); free(b.s); remove(tmp); return -1; }
    fclose(fp);
    free(b.s);
    remove(path);
    return rename(tmp, path) == 0 ? 0 : -1;
}

static char *nc_read_all(const char *path, size_t *len) {
    FILE *fp = fopen(path, "rb");
    long n;
    char *s;
    if (!fp) return NULL;
    fseek(fp, 0, SEEK_END);
    n = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    if (n < 0 || (unsigned long) n > 64ul * 1024 * 1024) { fclose(fp); return NULL; }
    s = (char *) malloc((size_t) n + 1);
    if (!s) { fclose(fp); return NULL; }
    if (fread(s, 1, (size_t) n, fp) != (size_t) n) { fclose(fp); free(s); return NULL; }
    fclose(fp);
    s[n] = '\0';
    if (len) *len = (size_t) n;
    return s;
}

/* Load a cache written by nc_cat_save into an empty catalog. Returns the number of mods, or -1 when there is no file. */
static int nc_cat_load(nc_catalog *c, const char *path) {
    char *text = nc_read_all(path, NULL), *line;
    int count = 0;
    if (!text) return -1;
    for (line = text; *line; ) {
        char *eol = strchr(line, '\n');
        nj_doc d;
        int root;
        if (eol) *eol = '\0';
        if (*line) {
            root = nj_parse(&d, line);
            if (root >= 0) {
                int g = nj_get(&d, root, "gone");
                if (g >= 0 && d.n[g].type == NJ_ARR) {
                    int e;
                    for (e = d.n[g].first; e >= 0; e = d.n[e].next) nc_cat_add_gone(c, (int) nj_num(&d, e, 0));
                } else if (nc_cat_apply(c, &d, root)) ++count;
            }
            nj_free(&d);
        }
        if (!eol) break;
        line = eol + 1;
    }
    free(text);
    return count;
}

/* ---- search / filter / sort --------------------------------------------------------------------------------- */

enum { NC_SORT_UPDATED, NC_SORT_NEWEST, NC_SORT_DOWNLOADS, NC_SORT_LIKES, NC_SORT_TITLE, NC_SORT_N };
static const char *const nc_sort_label[NC_SORT_N] = { "Recently updated", "Newest", "Most downloaded", "Most liked", "Title" };

typedef struct {
    char text[48];          /* words, all of which must appear in the title, author, tags or a file name */
    unsigned type_mask;     /* 0 any, else bit (1 << NC_T_*) per accepted type */
    int fighter;            /* -1 any, else nc_fighters index (costume mods with a file for it) */
    int sort;               /* NC_SORT_* */
    int installable_only;   /* only mods the browser can install */
} nc_query;

static void nc_query_init(nc_query *q) { memset(q, 0, sizeof *q); q->fighter = -1; }

static const nc_catalog *nc_sort_cat;
static int nc_sort_mode;
static int nc_cmp_idx(const void *a, const void *b) {
    const nc_mod *x = &nc_sort_cat->m[*(const int *) a], *y = &nc_sort_cat->m[*(const int *) b];
    int r = 0;
    switch (nc_sort_mode) {
    case NC_SORT_NEWEST: r = strcmp(y->created, x->created); break;
    case NC_SORT_DOWNLOADS: r = y->downloads - x->downloads; break;
    case NC_SORT_LIKES: r = y->likes - x->likes; break;
    case NC_SORT_TITLE: r = nc_ieq(x->title, y->title) ? 0 : (tolower((unsigned char) x->title[0]) - tolower((unsigned char) y->title[0])); if (!r) { size_t i = 0; while (x->title[i] && tolower((unsigned char) x->title[i]) == tolower((unsigned char) y->title[i])) ++i; r = tolower((unsigned char) x->title[i]) - tolower((unsigned char) y->title[i]); } break;
    default: r = strcmp(y->updated, x->updated); break;
    }
    if (r) return r;
    return x->id - y->id;                  /* a total order: the list never reshuffles between two builds */
}

/* "Luffy Falco/Animelee/PlFcBu.dat": a post that ships an Animelee set beside a vanilla one installs the vanilla (non-Animelee) files only.
 * A file is skipped when its path names Animelee and another costume file of the same fighter does not. */
static int nc_is_animelee(const nc_file *f) {
    const char *p = f->filename;
    size_t n = strlen("animelee");
    for (; *p; ++p) if (!strncmp(p, "Animelee", n) || !strncmp(p, "animelee", n) || !strncmp(p, "ANIMELEE", n)) return 1;
    return 0;
}
static int nc_variant_skip(const nc_file *files, int nf, int i) {
    int k;
    if (!files[i].costume || !nc_is_animelee(&files[i])) return 0;
    for (k = 0; k < nf; ++k) if (k != i && files[k].costume && files[k].fighter == files[i].fighter && !nc_is_animelee(&files[k])) return 1;
    return 0;
}

/* a download that did not work, in words a player can use */
static const char *nc_http_reason(int status) {
    if (status == 404 || status == 410) return "Nucleus offers no download for this file (the author may have removed it).";
    if (status == 401 || status == 403) return "Nucleus does not allow this download.";
    if (status == 429) return "Nucleus asked us to slow down. Try again in a minute.";
    if (status >= 500) return "Nucleus is not answering right now. Try again later.";
    if (status == 0) return "Could not reach Nucleus.";
    return "The download did not work.";
}

static int nc_installable_files(const nc_mod *m, const nc_file *files) {
    int i;
    if (m->type != NC_T_COSTUME) return 0;
    for (i = 0; i < m->nf; ++i) {
        const nc_file *f = &files[i];
        if (f->costume && f->fighter >= 0 && nc_fighters[f->fighter].installable) return 1;
    }
    return 0;
}
static int nc_installable_mod(const nc_catalog *c, const nc_mod *m) { return nc_installable_files(m, c->f + m->f0); }

/* Why a mod cannot be installed here, or "" when it can. */
static const char *nc_why_not_files(const nc_mod *m, const nc_file *files) {
    int i, any_costume = 0, any_known = 0;
    if (m->type == NC_T_STAGE_SKIN) return "Stage skins cannot be installed yet.";
    if (m->type == NC_T_EFFECTS) return "Effect mods cannot be installed yet.";
    if (m->type == NC_T_CUSTOM_STAGE) return "Custom stages cannot be installed yet.";
    if (m->type == NC_T_GAMEPLAY) return "Gameplay mods cannot be installed yet.";
    if (m->type == NC_T_PATCH) return "Patches are for the whole disc and cannot be installed.";
    if (m->type != NC_T_COSTUME) return "This kind of mod cannot be installed yet.";
    for (i = 0; i < m->nf; ++i) {
        const nc_file *f = &files[i];
        if (!f->costume) continue;
        ++any_costume;
        if (f->fighter >= 0) { ++any_known; if (nc_fighters[f->fighter].installable) return ""; }
    }
    if (!any_costume) return "No costume file in this post.";
    if (!any_known) return "Not a retail fighter's costume.";
    return "Sheik and Nana costumes come with Zelda and Popo (not yet).";
}
static const char *nc_why_not(const nc_catalog *c, const nc_mod *m) { return nc_why_not_files(m, c->f + m->f0); }

/* Fills out[] (at most cap) with catalog indices in the query's order; returns the count. */
static int nc_cat_filter(const nc_catalog *c, const nc_query *q, int *out, int cap) {
    int i, n = 0;
    char words[8][48];
    int nw = 0;
    const char *p = q->text;
    while (*p && nw < 8) {
        size_t l = 0;
        while (*p == ' ') ++p;
        while (*p && *p != ' ' && l < 47) words[nw][l++] = *p++;
        if (l) words[nw++][l] = '\0';
        while (*p && *p != ' ') ++p;
    }
    for (i = 0; i < c->n && n < cap; ++i) {
        const nc_mod *m = &c->m[i];
        int w, ok = 1;
        if (q->type_mask && !(q->type_mask & (1u << m->type))) continue;
        if (q->fighter >= 0 && !(m->fmask & (1u << q->fighter))) continue;
        if (q->installable_only && !nc_installable_mod(c, m)) continue;
        for (w = 0; w < nw && ok; ++w) {
            int f, hit = nc_icontains(m->title, words[w]) || nc_icontains(m->author, words[w]) || nc_icontains(m->tags, words[w]);
            for (f = 0; f < m->nf && !hit; ++f) hit = nc_icontains(c->f[m->f0 + f].filename, words[w]);
            ok = hit;
        }
        if (ok) out[n++] = i;
    }
    nc_sort_cat = c; nc_sort_mode = q->sort;
    qsort(out, (size_t) n, sizeof(int), nc_cmp_idx);
    return n;
}

/* ---- the sync ---------------------------------------------------------------------------------------- */

typedef struct { int status; char *body; size_t len; int retry_after; } nc_resp;      /* status 0: no answer (network); body is malloc'd, the caller frees it */
typedef struct {
    void *user;
    int (*fetch)(void *user, const char *url, nc_resp *out);                  /* 0 when a response (any status) came back */
    int64_t (*now)(void *user);
    void (*sleep_ms)(void *user, int ms);
    int (*stop)(void *user);                                                  /* nonzero: leave at the next page */
    void (*progress)(void *user, const char *what, int done, int total);
    void (*checkpoint)(void *user);                                           /* save the catalog and the state: a long first sync resumes */
    void (*lock)(void *user);                                                 /* optional: held around every change to the catalog */
    void (*unlock)(void *user);
} nc_sync_io;

typedef struct {
    int64_t t0;              /* when the first full pass started (noted before the first page) */
    int64_t since;           /* W: when the last completed pass started */
    int64_t last_poll;       /* when the last poll started: nothing polls again within NC_POLL_SECONDS */
    int64_t next_ok;         /* after a failed run: not before this */
    int full;                /* 1: the first full pass finished */
    char cursor[300];        /* resume point of an unfinished full pass */
    int pages, total;
    char msg[120];
} nc_sync;

static void nc_sync_to_json(const nc_sync *s, nj_buf *b) {
    nj_printf(b, "{\"t0\":%lld,\"since\":%lld,\"last_poll\":%lld,\"next_ok\":%lld,\"full\":%d,\"cursor\":", (long long) s->t0, (long long) s->since,
              (long long) s->last_poll, (long long) s->next_ok, s->full);
    nj_qstr(b, s->cursor);
    nj_puts(b, "}\n");
}
static int nc_sync_from_json(nc_sync *s, const char *text) {
    nj_doc d;
    int root = nj_parse(&d, text);
    memset(s, 0, sizeof *s);
    if (root >= 0) {
        s->t0 = (int64_t) nj_gnum(&d, root, "t0", 0);
        s->since = (int64_t) nj_gnum(&d, root, "since", 0);
        s->last_poll = (int64_t) nj_gnum(&d, root, "last_poll", 0);
        s->next_ok = (int64_t) nj_gnum(&d, root, "next_ok", 0);
        s->full = (int) nj_gnum(&d, root, "full", 0);
        nc_copy(s->cursor, sizeof s->cursor, nj_gstr(&d, root, "cursor", ""));
    }
    nj_free(&d);
    return root >= 0 ? 0 : -1;
}

static void nc_urlenc(const char *s, char *out, size_t cap) {
    size_t o = 0;
    for (; *s && o + 4 < cap; ++s) {
        unsigned char c = (unsigned char) *s;
        if (isalnum(c) || c == '-' || c == '_' || c == '.' || c == '~') out[o++] = (char) c;
        else { snprintf(out + o, cap - o, "%%%02X", c); o += 3; }
    }
    out[o] = '\0';
}

/* GET with the API's back-off: 429 / 5xx / no answer wait and retry (Retry-After when given), other 4xx stop. Returns 0 with *r filled (status 200),
 * -1 on a stop / give-up with msg set. */
static int nc_get(const nc_sync_io *io, const char *url, nc_resp *r, char *msg, size_t mcap) {
    int attempt, wait = 5;
    for (attempt = 0; attempt < 5; ++attempt) {
        memset(r, 0, sizeof *r);
        if (io->stop && io->stop(io->user)) { snprintf(msg, mcap, "Stopped."); return -1; }
        if (io->fetch(io->user, url, r) == 0 && r->status == 200 && r->body) return 0;
        if (r->status >= 400 && r->status < 500 && r->status != 429) {
            snprintf(msg, mcap, "SSBM Nucleus refused the request (%d).", r->status);
            free(r->body); r->body = NULL;
            return -1;
        }
        {
            int ra = r->retry_after > 0 ? r->retry_after : wait;
            if (ra > 120) ra = 120;
            free(r->body); r->body = NULL;
            if (attempt == 4) break;
            io->sleep_ms(io->user, ra * 1000);
            wait *= 3;
        }
    }
    snprintf(msg, mcap, "SSBM Nucleus is not answering. Showing what is saved.");
    return -1;
}

/* one page of /mods: applies every mod; returns the number applied, -1 on bad JSON; *next set to the next cursor ("" at the end) */
static int nc_apply_page_(nc_catalog *c, const char *body, char *next, size_t ncap, int *total) {
    nj_doc d;
    int root = nj_parse(&d, body), data, e, n = 0, nx;
    next[0] = '\0';
    if (root < 0) { nj_free(&d); return -1; }
    data = nj_get(&d, root, "data");
    if (data < 0 || d.n[data].type != NJ_ARR) { nj_free(&d); return -1; }
    for (e = d.n[data].first; e >= 0; e = d.n[e].next) {
        int id = (int) nj_gnum(&d, e, "id", -1);
        if (nc_cat_apply(c, &d, e)) { ++n; if (id > 0) nc_cat_unmark_gone(c, id); }
    }
    nx = nj_get(&d, root, "next_cursor");
    if (nx >= 0 && d.n[nx].type == NJ_STR) nc_copy(next, ncap, d.pool + d.n[nx].str);
    if (total && nj_get(&d, root, "total") >= 0) *total = (int) nj_gnum(&d, root, "total", 0);
    nj_free(&d);
    return n;
}

static int nc_apply_removed_(nc_catalog *c, const char *body, char *next, size_t ncap) {
    nj_doc d;
    int root = nj_parse(&d, body), data, e, n = 0, nx;
    next[0] = '\0';
    if (root < 0) { nj_free(&d); return -1; }
    data = nj_get(&d, root, "data");
    if (data < 0 || d.n[data].type != NJ_ARR) { nj_free(&d); return -1; }
    for (e = d.n[data].first; e >= 0; e = d.n[e].next) {
        int id = (int) nj_gnum(&d, e, "id", -1);
        if (id > 0) { nc_cat_remove_id(c, id); ++n; }
    }
    nx = nj_get(&d, root, "next_cursor");
    if (nx >= 0 && d.n[nx].type == NJ_STR) nc_copy(next, ncap, d.pool + d.n[nx].str);
    nj_free(&d);
    return n;
}

static int nc_apply_page(const nc_sync_io *io, nc_catalog *c, const char *body, char *next, size_t ncap, int *total) {
    int n;
    if (io->lock) io->lock(io->user);
    n = nc_apply_page_(c, body, next, ncap, total);
    if (io->unlock) io->unlock(io->user);
    return n;
}
static int nc_apply_removed(const nc_sync_io *io, nc_catalog *c, const char *body, char *next, size_t ncap) {
    int n;
    if (io->lock) io->lock(io->user);
    n = nc_apply_removed_(c, body, next, ncap);
    if (io->unlock) io->unlock(io->user);
    return n;
}

/* Run one sync. Returns 1 when it was too soon (nothing fetched), 0 when it finished, -1 when it gave up (s->msg says why; the catalog keeps what
 * arrived). force: ignore the 10-minute rule (a person pressed Refresh) but never the back-off after a failure. */
static int nc_sync_run(nc_sync *s, nc_catalog *c, const nc_sync_io *io, int force) {
    int64_t now = io->now(io->user), start;
    char url[700], enc[400], next[300], iso[32];
    nc_resp r;
    int n, total = 0;
    s->msg[0] = '\0';
    if (now < s->next_ok) { snprintf(s->msg, sizeof s->msg, "Waiting before the next try."); return 1; }
    if (s->full && !force && s->last_poll && now - s->last_poll < NC_POLL_SECONDS) return 1;
    if (s->full && force && s->last_poll && now - s->last_poll < 60) return 1;     /* even a press does not hammer the API */
    start = now;
    if (!s->full) {
        if (s->t0 == 0) { s->t0 = now; s->cursor[0] = '\0'; s->pages = 0; }
        for (;;) {
            if (s->cursor[0]) { nc_urlenc(s->cursor, enc, sizeof enc); snprintf(url, sizeof url, "%s/mods?limit=%d&cursor=%s", nc_api_base, NC_PAGE_LIMIT, enc); }
            else snprintf(url, sizeof url, "%s/mods?limit=%d", nc_api_base, NC_PAGE_LIMIT);
            if (nc_get(io, url, &r, s->msg, sizeof s->msg) < 0) { s->next_ok = io->now(io->user) + NC_POLL_SECONDS; if (io->checkpoint) io->checkpoint(io->user); return -1; }
            n = nc_apply_page(io, c, r.body, next, sizeof next, &total);
            free(r.body);
            if (n < 0) { snprintf(s->msg, sizeof s->msg, "SSBM Nucleus sent something unreadable."); s->next_ok = io->now(io->user) + NC_POLL_SECONDS; return -1; }
            ++s->pages;
            s->total = total;
            nc_copy(s->cursor, sizeof s->cursor, next);
            if (io->progress) io->progress(io->user, "Downloading the catalog", c->n, total);
            if (s->pages % 10 == 0 && io->checkpoint) io->checkpoint(io->user);
            if (!next[0]) break;
            io->sleep_ms(io->user, 700);
        }
        s->full = 1;
        s->since = s->t0;
        s->cursor[0] = '\0';
        if (io->checkpoint) io->checkpoint(io->user);
    }
    /* the delta: everything updated since W - 15 minutes, then the removals */
    nc_iso(s->since - NC_OVERLAP_SECONDS, iso, sizeof iso);
    nc_urlenc(iso, enc, sizeof enc);
    next[0] = '\0';
    for (;;) {
        char cur[400];
        cur[0] = '\0';
        if (next[0]) { char e2[400]; nc_urlenc(next, e2, sizeof e2); snprintf(cur, sizeof cur, "&cursor=%s", e2); }
        snprintf(url, sizeof url, "%s/mods?limit=%d&updated_since=%s%s", nc_api_base, NC_PAGE_LIMIT, enc, cur);
        if (nc_get(io, url, &r, s->msg, sizeof s->msg) < 0) { s->next_ok = io->now(io->user) + NC_POLL_SECONDS; return -1; }
        n = nc_apply_page(io, c, r.body, next, sizeof next, NULL);
        free(r.body);
        if (n < 0) { snprintf(s->msg, sizeof s->msg, "SSBM Nucleus sent something unreadable."); s->next_ok = io->now(io->user) + NC_POLL_SECONDS; return -1; }
        if (io->progress) io->progress(io->user, "Checking for changes", c->n, 0);
        if (!next[0]) break;
        io->sleep_ms(io->user, 700);
    }
    next[0] = '\0';
    for (;;) {
        char cur[400];
        cur[0] = '\0';
        if (next[0]) { char e2[400]; nc_urlenc(next, e2, sizeof e2); snprintf(cur, sizeof cur, "&cursor=%s", e2); }
        snprintf(url, sizeof url, "%s/mods/removed?limit=1000&since=%s%s", nc_api_base, enc, cur);
        io->sleep_ms(io->user, 700);
        if (nc_get(io, url, &r, s->msg, sizeof s->msg) < 0) { s->next_ok = io->now(io->user) + NC_POLL_SECONDS; return -1; }
        n = nc_apply_removed(io, c, r.body, next, sizeof next);
        free(r.body);
        if (n < 0) { snprintf(s->msg, sizeof s->msg, "SSBM Nucleus sent something unreadable."); s->next_ok = io->now(io->user) + NC_POLL_SECONDS; return -1; }
        if (!next[0]) break;
    }
    s->since = start;
    s->last_poll = start;
    s->next_ok = 0;
    if (io->checkpoint) io->checkpoint(io->user);
    return 0;
}

#endif
