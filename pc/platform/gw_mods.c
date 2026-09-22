/* gw_mods.c - the mods folder registry: scan, mod.json, enabled.txt, resolution, mount order,
 * the toggle API for the in-game menu, and the netplay mod-set fingerprint. See gw_mods.h for
 * the layout and the rules; docs/mods-packaging.md (root repo) for the why.
 *
 * The file overlay itself stays in shim_dvd.c: it asks this module which mods mount and in what
 * order, and reports back every disc path it mounted so the fingerprint covers real files.
 *
 * Every piece of logic works on a gw_mods_set so the in-engine tests can build their own sets in
 * a scratch folder without touching the one the game booted with.
 *
 * NATIVE platform code (i686 clang), not gwtool. */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "gw_mods.h"
#include "gw_test.h"

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define GW_MODS_ID_MAX 64
#define GW_MODS_TEXT_MAX 128
#define GW_MODS_LIST_MAX 8 /* requires / conflicts entries per mod */

typedef struct gw_mod {
    char id[GW_MODS_ID_MAX];
    char name[GW_MODS_TEXT_MAX];
    char version[32];
    char kind[16];
    char pack[32];
    char desc[GW_MODS_TEXT_MAX];
    char hash[80];
    char requires_text[GW_MODS_TEXT_MAX];
    char req[GW_MODS_LIST_MAX][GW_MODS_ID_MAX];
    int nreq;
    char con[GW_MODS_LIST_MAX][GW_MODS_ID_MAX];
    int ncon;
    char payload[MAX_PATH];
    int payload_is_mod_dir;
    int enabled_boot; /* in enabled.txt at boot (or no enabled.txt) */
    int enabled;      /* pending, for the next boot */
    int status;       /* GW_MOD_* */
    char status_text[GW_MODS_TEXT_MAX];
    uint64_t file_sum; /* order-independent sum of per-file digests the overlay reported */
    int nfiles;
} gw_mod;

typedef struct gw_mods_set {
    char dir[MAX_PATH];
    int have_dir;
    int explicit_set; /* enabled.txt existed */
    int mods_off;     /* MELEE_MODS=0 */
    int n;
    gw_mod mod[GW_MODS_MAX];
    int order[GW_MODS_MAX]; /* mount order: indices of ACTIVE mods */
    int norder;
    char describe[2048];
} gw_mods_set;

static gw_mods_set gw_mods_boot;
static int gw_mods_boot_loaded;

/* ---- small helpers -------------------------------------------------------------------------- */

static uint64_t fnv64(uint64_t h, const void *data, size_t len) {
    const unsigned char *p = (const unsigned char *) data;
    size_t i;
    for (i = 0; i < len; ++i) {
        h ^= p[i];
        h *= 1099511628211ull;
    }
    return h;
}
#define FNV64_INIT 1469598103934665603ull

static uint64_t fnv64_str_lower(uint64_t h, const char *s) {
    for (; *s; ++s) {
        char c = *s == '\\' ? '/' : *s;
        if (c >= 'A' && c <= 'Z') c = (char) (c - 'A' + 'a');
        h = fnv64(h, &c, 1);
    }
    return fnv64(h, "", 1);
}

static void copy_str(char *dst, size_t cap, const char *src) {
    size_t n = strlen(src);
    if (n >= cap) n = cap - 1;
    memcpy(dst, src, n);
    dst[n] = '\0';
}

static int is_dir(const char *path) {
    DWORD a = GetFileAttributesA(path);
    return a != INVALID_FILE_ATTRIBUTES && (a & FILE_ATTRIBUTE_DIRECTORY) != 0;
}

static char *read_text_file(const char *path) {
    FILE *f = fopen(path, "rb");
    long len;
    char *buf;
    if (f == NULL) return NULL;
    if (fseek(f, 0, SEEK_END) != 0 || (len = ftell(f)) < 0 || len > (1 << 20) ||
        fseek(f, 0, SEEK_SET) != 0) {
        fclose(f);
        return NULL;
    }
    buf = (char *) malloc((size_t) len + 1);
    if (buf == NULL) {
        fclose(f);
        return NULL;
    }
    len = (long) fread(buf, 1, (size_t) len, f);
    buf[len] = '\0';
    fclose(f);
    return buf;
}

/* ---- mod.json: a flat object of strings and string arrays ----------------------------------- */

static const char *json_skip_ws(const char *p) {
    while (*p == ' ' || *p == '\t' || *p == '\r' || *p == '\n') ++p;
    return p;
}

/* Parse a JSON string at p (which points at the opening quote) into out. Returns the position
 * after the closing quote, or NULL. \uXXXX escapes are kept only for ASCII. */
static const char *json_string(const char *p, char *out, size_t cap) {
    size_t n = 0;
    if (*p != '"') return NULL;
    ++p;
    while (*p && *p != '"') {
        char c = *p++;
        if (c == '\\') {
            c = *p++;
            switch (c) {
            case 'n': c = '\n'; break;
            case 't': c = '\t'; break;
            case 'r': c = '\r'; break;
            case 'b': c = '\b'; break;
            case 'f': c = '\f'; break;
            case 'u': {
                unsigned v = 0;
                int k;
                for (k = 0; k < 4 && p[k]; ++k) {
                    char h = p[k];
                    v = v * 16u + (unsigned) (h >= 'a' ? h - 'a' + 10 : h >= 'A' ? h - 'A' + 10 : h - '0');
                }
                p += k;
                c = v < 0x80u ? (char) v : '?';
                break;
            }
            case '\0': return NULL;
            default: break; /* \" \\ \/ */
            }
        }
        if (n + 1 < cap) out[n++] = c;
    }
    if (*p != '"') return NULL;
    out[n] = '\0';
    return p + 1;
}

/* Skip any JSON value (used for fields we do not read, including nested objects). Returns the
 * position just after it. */
static const char *json_skip_value(const char *p) {
    int depth = 0;
    p = json_skip_ws(p);
    for (;;) {
        char c = *p;
        if (c == '\0') return p;
        if (c == '"') {
            char tmp[8];
            const char *q = json_string(p, tmp, sizeof tmp);
            if (q == NULL) return p + strlen(p);
            p = q;
            if (depth == 0) return p;
            continue;
        }
        if (c == '{' || c == '[') {
            ++depth;
        } else if (c == '}' || c == ']') {
            if (depth == 0) return p;
            if (--depth == 0) return p + 1;
        } else if (depth == 0 && c == ',') {
            return p;
        }
        ++p;
    }
}

static void mod_set_field(gw_mod *m, const char *key, const char *val) {
    if (strcmp(key, "name") == 0) copy_str(m->name, sizeof m->name, val);
    else if (strcmp(key, "version") == 0) copy_str(m->version, sizeof m->version, val);
    else if (strcmp(key, "kind") == 0) copy_str(m->kind, sizeof m->kind, val);
    else if (strcmp(key, "pack") == 0) copy_str(m->pack, sizeof m->pack, val);
    else if (strcmp(key, "description") == 0) copy_str(m->desc, sizeof m->desc, val);
    else if (strcmp(key, "hash") == 0) copy_str(m->hash, sizeof m->hash, val);
    else if (strcmp(key, "id") == 0 && _stricmp(val, m->id) != 0) {
        gw_log("gw: mods: %s/mod.json says id \"%s\" - the folder name wins", m->id, val);
    }
}

static void mod_add_list(gw_mod *m, const char *key, const char *val) {
    if (strcmp(key, "requires") == 0 && m->nreq < GW_MODS_LIST_MAX) {
        copy_str(m->req[m->nreq++], GW_MODS_ID_MAX, val);
    } else if (strcmp(key, "conflicts") == 0 && m->ncon < GW_MODS_LIST_MAX) {
        copy_str(m->con[m->ncon++], GW_MODS_ID_MAX, val);
    }
}

/* Returns 0 on success, -1 on a syntax error (fields read before it are kept). */
static int mod_parse_json(gw_mod *m, const char *text) {
    const char *p = json_skip_ws(text);
    if (p[0] == (char) 0xEF && p[1] == (char) 0xBB && p[2] == (char) 0xBF) p = json_skip_ws(p + 3);
    if (*p != '{') return -1;
    p = json_skip_ws(p + 1);
    while (*p && *p != '}') {
        char key[32], val[GW_MODS_TEXT_MAX];
        p = json_string(p, key, sizeof key);
        if (p == NULL) return -1;
        p = json_skip_ws(p);
        if (*p != ':') return -1;
        p = json_skip_ws(p + 1);
        if (*p == '"') {
            p = json_string(p, val, sizeof val);
            if (p == NULL) return -1;
            mod_set_field(m, key, val);
        } else if (*p == '[') {
            p = json_skip_ws(p + 1);
            while (*p && *p != ']') {
                if (*p == '"') {
                    p = json_string(p, val, sizeof val);
                    if (p == NULL) return -1;
                    mod_add_list(m, key, val);
                } else {
                    p = json_skip_value(p);
                }
                p = json_skip_ws(p);
                if (*p == ',') p = json_skip_ws(p + 1);
            }
            if (*p != ']') return -1;
            ++p;
        } else {
            p = json_skip_value(p);
        }
        p = json_skip_ws(p);
        if (*p == ',') p = json_skip_ws(p + 1);
    }
    return *p == '}' ? 0 : -1;
}

/* ---- scanning -------------------------------------------------------------------------------- */

static int mod_cmp_id(const void *a, const void *b) {
    return _stricmp(((const gw_mod *) a)->id, ((const gw_mod *) b)->id);
}

static int set_find(const gw_mods_set *s, const char *id) {
    int i;
    for (i = 0; i < s->n; ++i) {
        if (_stricmp(s->mod[i].id, id) == 0) return i;
    }
    return -1;
}

static void mod_load_meta(gw_mod *m, const char *moddir) {
    char path[MAX_PATH];
    char *text;
    int i;
    snprintf(path, sizeof path, "%s\\files", moddir);
    if (is_dir(path)) {
        copy_str(m->payload, sizeof m->payload, path);
        m->payload_is_mod_dir = 0;
    } else {
        copy_str(m->payload, sizeof m->payload, moddir);
        m->payload_is_mod_dir = 1;
    }
    snprintf(path, sizeof path, "%s\\mod.json", moddir);
    text = read_text_file(path);
    if (text != NULL) {
        if (mod_parse_json(m, text) != 0) {
            gw_log("gw: mods: %s/mod.json is not valid JSON - read what parsed, ignored the rest", m->id);
        }
        free(text);
    }
    if (m->name[0] == '\0') copy_str(m->name, sizeof m->name, m->id);
    if (m->kind[0] == '\0' || (strcmp(m->kind, "base") != 0 && strcmp(m->kind, "fighter") != 0 &&
                               strcmp(m->kind, "stage") != 0 && strcmp(m->kind, "misc") != 0 &&
                               strcmp(m->kind, "script") != 0)) {
        if (m->kind[0] != '\0') gw_log("gw: mods: %s: unknown kind \"%s\" - treated as misc", m->id, m->kind);
        copy_str(m->kind, sizeof m->kind, "misc");
    }
    m->requires_text[0] = '\0';
    for (i = 0; i < m->nreq; ++i) {
        size_t l = strlen(m->requires_text);
        snprintf(m->requires_text + l, sizeof m->requires_text - l, "%s%s", i ? "," : "", m->req[i]);
    }
}

/* Read enabled.txt into the set's enabled flags. Absent file = everything enabled. */
static void set_read_enabled(gw_mods_set *s) {
    char path[MAX_PATH];
    char *text, *line, *next;
    int i;
    snprintf(path, sizeof path, "%s\\enabled.txt", s->dir);
    text = read_text_file(path);
    s->explicit_set = text != NULL;
    for (i = 0; i < s->n; ++i) s->mod[i].enabled_boot = text == NULL;
    if (text == NULL) return;
    for (line = text; line != NULL && *line; line = next) {
        char *end, *hash;
        next = strchr(line, '\n');
        if (next != NULL) *next++ = '\0';
        hash = strchr(line, '#');
        if (hash != NULL) *hash = '\0';
        while (*line == ' ' || *line == '\t' || *line == '\r' ||
               (line[0] == (char) 0xEF && line[1] == (char) 0xBB && line[2] == (char) 0xBF)) {
            line += (*line == (char) 0xEF) ? 3 : 1;
        }
        end = line + strlen(line);
        while (end > line && (end[-1] == ' ' || end[-1] == '\t' || end[-1] == '\r')) *--end = '\0';
        if (*line == '\0') continue;
        i = set_find(s, line);
        if (i < 0) {
            gw_log("gw: mods: enabled.txt names \"%s\", which is not in %s - ignored", line, s->dir);
            continue;
        }
        s->mod[i].enabled_boot = 1;
    }
    free(text);
}

static int kind_rank(const gw_mod *m) {
    if (strcmp(m->kind, "base") == 0) return 0;
    if (strcmp(m->kind, "misc") == 0 || strcmp(m->kind, "script") == 0) return 1;
    return 2;
}

static int mods_conflict(const gw_mod *a, const gw_mod *b) {
    int i;
    for (i = 0; i < a->ncon; ++i) {
        if (_stricmp(a->con[i], b->id) == 0) return 1;
    }
    for (i = 0; i < b->ncon; ++i) {
        if (_stricmp(b->con[i], a->id) == 0) return 1;
    }
    return kind_rank(a) == 0 && kind_rank(b) == 0;
}

/* Resolve `flags` (1 = wanted) into the mounting set, writing each mod's status. Pure: works on
 * the set's metadata only. */
static void set_resolve(gw_mods_set *s) {
    int keep[GW_MODS_MAX];
    int changed, i, j, k;
    int by_rank[GW_MODS_MAX], nr = 0;

    for (i = 0; i < s->n; ++i) {
        gw_mod *m = &s->mod[i];
        keep[i] = m->enabled_boot && !s->mods_off;
        m->status = keep[i] ? GW_MOD_ACTIVE : GW_MOD_OFF;
        copy_str(m->status_text, sizeof m->status_text,
                 s->mods_off ? "all mods off (MELEE_MODS=0)" : keep[i] ? "on" : "off");
    }
    /* Conflict pass order: base first, then misc, then content, then id (the array is id-sorted). */
    for (k = 0; k < 3; ++k) {
        for (i = 0; i < s->n; ++i) {
            if (kind_rank(&s->mod[i]) == k) by_rank[nr++] = i;
        }
    }
    do {
        changed = 0;
        /* requirements */
        for (i = 0; i < s->n; ++i) {
            gw_mod *m = &s->mod[i];
            if (!keep[i]) continue;
            for (j = 0; j < m->nreq; ++j) {
                int r = set_find(s, m->req[j]);
                if (r < 0 || !keep[r]) {
                    keep[i] = 0;
                    m->status = GW_MOD_MISSING_DEP;
                    snprintf(m->status_text, sizeof m->status_text, "needs %s%s", m->req[j],
                             r < 0 ? " (not installed)" : "");
                    changed = 1;
                    break;
                }
            }
        }
        /* conflicts: a mod loses to any conflicting mod earlier in pass order */
        for (i = 0; i < nr; ++i) {
            int a = by_rank[i];
            if (!keep[a]) continue;
            for (j = 0; j < i; ++j) {
                int b = by_rank[j];
                if (keep[b] && mods_conflict(&s->mod[a], &s->mod[b])) {
                    keep[a] = 0;
                    s->mod[a].status = GW_MOD_CONFLICT;
                    snprintf(s->mod[a].status_text, sizeof s->mod[a].status_text, "conflicts with %s",
                             s->mod[b].id);
                    changed = 1;
                    break;
                }
            }
        }
    } while (changed);

    /* mount order: repeatedly take, in pass order, the first kept mod whose requirements are all
     * already placed. Requirement cycles cannot all be placed; they are dropped as MISSING_DEP. */
    s->norder = 0;
    {
        int placed[GW_MODS_MAX];
        int progress = 1;
        memset(placed, 0, sizeof placed);
        while (progress) {
            progress = 0;
            for (i = 0; i < nr; ++i) {
                int a = by_rank[i], ok = 1;
                if (!keep[a] || placed[a]) continue;
                for (j = 0; j < s->mod[a].nreq; ++j) {
                    int r = set_find(s, s->mod[a].req[j]);
                    if (r < 0 || !placed[r]) {
                        ok = 0;
                        break;
                    }
                }
                if (ok) {
                    placed[a] = 1;
                    s->order[s->norder++] = a;
                    progress = 1;
                    break; /* restart from the top so pass order is respected */
                }
            }
        }
        for (i = 0; i < s->n; ++i) {
            if (keep[i] && !placed[i]) {
                s->mod[i].status = GW_MOD_MISSING_DEP;
                copy_str(s->mod[i].status_text, sizeof s->mod[i].status_text, "requirement cycle");
            }
        }
    }
    for (i = 0; i < s->n; ++i) s->mod[i].enabled = s->mod[i].enabled_boot;
}

/* Scan `dir` into `s`. Returns the number of mods found. */
static int set_load(gw_mods_set *s, const char *dir, int mods_off) {
    char pattern[MAX_PATH];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    memset(s, 0, sizeof *s);
    s->mods_off = mods_off;
    if (dir == NULL || dir[0] == '\0') return 0;
    copy_str(s->dir, sizeof s->dir, dir);
    s->have_dir = 1;
    snprintf(pattern, sizeof pattern, "%s\\*", dir);
    h = FindFirstFileA(pattern, &fd);
    if (h == INVALID_HANDLE_VALUE) return 0;
    do {
        if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) || fd.cFileName[0] == '.') continue;
        if (_stricmp(fd.cFileName, "targettest") == 0) continue; /* Target Test layouts, not a mod */
        if (strlen(fd.cFileName) >= GW_MODS_ID_MAX) {
            gw_log("gw: mods: folder name %s is too long for a mod id - skipped", fd.cFileName);
            continue;
        }
        if (s->n >= GW_MODS_MAX) {
            gw_log("gw: mods: more than %d mods - %s and later skipped", GW_MODS_MAX, fd.cFileName);
            break;
        }
        copy_str(s->mod[s->n].id, GW_MODS_ID_MAX, fd.cFileName);
        s->n++;
    } while (FindNextFileA(h, &fd));
    FindClose(h);
    qsort(s->mod, (size_t) s->n, sizeof s->mod[0], mod_cmp_id);
    {
        int i;
        for (i = 0; i < s->n; ++i) {
            char moddir[MAX_PATH];
            snprintf(moddir, sizeof moddir, "%s\\%s", dir, s->mod[i].id);
            mod_load_meta(&s->mod[i], moddir);
        }
    }
    set_read_enabled(s);
    set_resolve(s);
    return s->n;
}

static void set_log(const gw_mods_set *s) {
    int i;
    gw_log("gw: mods: %s - %d mod(s), enabled set from %s", s->dir, s->n,
           s->explicit_set ? "enabled.txt" : "none (all enabled)");
    for (i = 0; i < s->n; ++i) {
        const gw_mod *m = &s->mod[i];
        gw_log("gw: mods:   %-24s %-7s %-8s %s", m->id, m->kind, m->version[0] ? m->version : "-",
               m->status_text);
    }
}

static const char *default_dir(char *buf, size_t cap) {
    const char *dir_env = getenv("MELEE_MODS_DIR");
    if (dir_env != NULL && dir_env[0] != '\0') {
        copy_str(buf, cap, dir_env);
        return buf;
    }
    {
        DWORD len = GetModuleFileNameA(NULL, buf, (DWORD) cap);
        char *slash = (len > 0 && len < cap) ? strrchr(buf, '\\') : NULL;
        if (slash == NULL) {
            copy_str(buf, cap, "mods");
            return buf;
        }
        slash[1] = '\0';
        strncat(buf, "mods", cap - strlen(buf) - 1);
    }
    return buf;
}

static gw_mods_set *boot(void) {
    if (!gw_mods_boot_loaded) {
        char dir[MAX_PATH];
        const char *en = getenv("MELEE_MODS");
        gw_mods_boot_loaded = 1;
        set_load(&gw_mods_boot, default_dir(dir, sizeof dir), en != NULL && en[0] == '0');
        if (gw_mods_boot.n > 0) set_log(&gw_mods_boot);
    }
    return &gw_mods_boot;
}

static gw_mod *mod_at(gw_mods_set *s, int i) { return (i >= 0 && i < s->n) ? &s->mod[i] : NULL; }

/* ---- toggling (on a set) ---------------------------------------------------------------------- */

static int set_enable(gw_mods_set *s, int i, int on, int depth) {
    gw_mod *m = mod_at(s, i);
    int changed = 0, j, k;
    if (m == NULL || depth > GW_MODS_MAX) return 0;
    if (!on) {
        if (!m->enabled) return 0;
        m->enabled = 0;
        changed = 1;
        for (j = 0; j < s->n; ++j) { /* dependents follow it off */
            for (k = 0; k < s->mod[j].nreq; ++k) {
                if (_stricmp(s->mod[j].req[k], m->id) == 0) {
                    changed += set_enable(s, j, 0, depth + 1);
                    break;
                }
            }
        }
        return changed;
    }
    if (!m->enabled) {
        m->enabled = 1;
        changed = 1;
    }
    for (k = 0; k < m->nreq; ++k) { /* requirements follow it on */
        int r = set_find(s, m->req[k]);
        if (r >= 0 && !s->mod[r].enabled) changed += set_enable(s, r, 1, depth + 1);
    }
    for (j = 0; j < s->n; ++j) { /* what conflicts with it goes off */
        if (j != i && s->mod[j].enabled && mods_conflict(m, &s->mod[j])) {
            changed += set_enable(s, j, 0, depth + 1);
        }
    }
    return changed;
}

static int set_save(gw_mods_set *s) {
    char path[MAX_PATH], tmp[MAX_PATH];
    FILE *f;
    int i;
    if (!s->have_dir) return -1;
    CreateDirectoryA(s->dir, NULL);
    snprintf(path, sizeof path, "%s\\enabled.txt", s->dir);
    snprintf(tmp, sizeof tmp, "%s\\enabled.txt.tmp", s->dir);
    f = fopen(tmp, "wb");
    if (f == NULL) {
        gw_log("gw: mods: cannot write %s", tmp);
        return -1;
    }
    fprintf(f, "# Mods enabled at the next boot, one folder name per line. Written by the game's\n"
               "# mods menu; hand edits are fine. Delete this file to enable every mod.\n");
    for (i = 0; i < s->n; ++i) {
        if (s->mod[i].enabled) fprintf(f, "%s\n", s->mod[i].id);
    }
    if (fclose(f) != 0 || !MoveFileExA(tmp, path, MOVEFILE_REPLACE_EXISTING)) {
        gw_log("gw: mods: cannot replace %s", path);
        DeleteFileA(tmp);
        return -1;
    }
    s->explicit_set = 1;
    gw_log("gw: mods: wrote %s", path);
    return 0;
}

/* ---- fingerprint (on a set) -------------------------------------------------------------------- */

static uint64_t mod_digest(const gw_mod *m) {
    uint64_t h = FNV64_INIT;
    h = fnv64_str_lower(h, m->id);
    h = fnv64(h, m->version, strlen(m->version) + 1);
    h = fnv64(h, m->hash, strlen(m->hash) + 1);
    h = fnv64(h, &m->file_sum, sizeof m->file_sum);
    h = fnv64(h, &m->nfiles, sizeof m->nfiles);
    return h;
}

static int active_sorted(const gw_mods_set *s, int *out) {
    /* the array is id-sorted, so walking it in index order is sorting by id */
    int i, n = 0;
    for (i = 0; i < s->n; ++i) {
        if (s->mod[i].status == GW_MOD_ACTIVE) out[n++] = i;
    }
    return n;
}

static uint64_t set_fingerprint(const gw_mods_set *s) {
    int idx[GW_MODS_MAX], n = active_sorted(s, idx), i;
    uint64_t h = FNV64_INIT;
    if (n == 0) return 0;
    for (i = 0; i < n; ++i) {
        uint64_t d = mod_digest(&s->mod[idx[i]]);
        h = fnv64_str_lower(h, s->mod[idx[i]].id);
        h = fnv64(h, &d, sizeof d);
    }
    return h != 0 ? h : 1;
}

static const char *set_describe(gw_mods_set *s) {
    int idx[GW_MODS_MAX], n = active_sorted(s, idx), i;
    size_t len = 0;
    s->describe[0] = '\0';
    for (i = 0; i < n; ++i) {
        uint64_t d = mod_digest(&s->mod[idx[i]]);
        unsigned short16 = (unsigned) ((d ^ (d >> 16) ^ (d >> 32) ^ (d >> 48)) & 0xFFFFu);
        int w = snprintf(s->describe + len, sizeof s->describe - len, "%s%s#%04x", i ? "," : "",
                         s->mod[idx[i]].id, short16);
        if (w < 0 || (size_t) w >= sizeof s->describe - len) break;
        len += (size_t) w;
    }
    return s->describe;
}

/* Parse "id#hhhh,..." into parallel arrays. Returns the count; *truncated set when it ends "...". */
static int parse_describe(const char *d, char ids[][GW_MODS_ID_MAX], unsigned *hashes, int cap,
                          int *truncated) {
    int n = 0;
    *truncated = 0;
    while (d != NULL && *d && n < cap) {
        const char *comma = strchr(d, ',');
        size_t len = comma != NULL ? (size_t) (comma - d) : strlen(d);
        const char *hashp;
        size_t idlen;
        if (len >= 3 && strncmp(d + len - 3, "...", 3) == 0) {
            *truncated = 1;
            break;
        }
        hashp = memchr(d, '#', len);
        idlen = hashp != NULL ? (size_t) (hashp - d) : len;
        if (idlen > 0 && idlen < GW_MODS_ID_MAX) {
            memcpy(ids[n], d, idlen);
            ids[n][idlen] = '\0';
            hashes[n] = hashp != NULL ? (unsigned) strtoul(hashp + 1, NULL, 16) : 0u;
            ++n;
        }
        if (comma == NULL) break;
        d = comma + 1;
    }
    return n;
}

static void append(char *out, int cap, const char *text) {
    size_t l = strlen(out);
    if ((int) l < cap - 1) snprintf(out + l, (size_t) cap - l, "%s", text);
}

static int diff_describe(const char *mine, const char *peer, char *out, int cap) {
    static char mid[GW_MODS_MAX][GW_MODS_ID_MAX], pid[GW_MODS_MAX][GW_MODS_ID_MAX];
    unsigned mh[GW_MODS_MAX], ph[GW_MODS_MAX];
    int mt, pt, nm, np, i, j, ndiff = 0;
    char lack[512] = "", extra[512] = "", files[512] = "";
    nm = parse_describe(mine, mid, mh, GW_MODS_MAX, &mt);
    np = parse_describe(peer, pid, ph, GW_MODS_MAX, &pt);
    for (i = 0; i < np; ++i) { /* peer has, I lack / differ */
        int found = -1;
        for (j = 0; j < nm; ++j) {
            if (_stricmp(pid[i], mid[j]) == 0) found = j;
        }
        if (found < 0) {
            append(lack, sizeof lack, lack[0] ? ", " : "");
            append(lack, sizeof lack, pid[i]);
            ++ndiff;
        } else if (mh[found] != ph[i]) {
            append(files, sizeof files, files[0] ? ", " : "");
            append(files, sizeof files, pid[i]);
            ++ndiff;
        }
    }
    if (!pt) {
        for (j = 0; j < nm; ++j) { /* I have, peer lacks */
            int found = 0;
            for (i = 0; i < np; ++i) {
                if (_stricmp(pid[i], mid[j]) == 0) found = 1;
            }
            if (!found) {
                append(extra, sizeof extra, extra[0] ? ", " : "");
                append(extra, sizeof extra, mid[j]);
                ++ndiff;
            }
        }
    }
    if (out != NULL && cap > 0) {
        out[0] = '\0';
        if (lack[0]) {
            append(out, cap, "you lack ");
            append(out, cap, lack);
        }
        if (extra[0]) {
            append(out, cap, out[0] ? "; peer lacks " : "peer lacks ");
            append(out, cap, extra);
        }
        if (files[0]) {
            append(out, cap, out[0] ? "; different files in " : "different files in ");
            append(out, cap, files);
        }
        if (pt) append(out, cap, out[0] ? " (peer list truncated)" : "peer list truncated");
        if (!out[0]) append(out, cap, "same mods");
    }
    return ndiff;
}

/* ---- public API: the boot set ------------------------------------------------------------------ */

#define MOD_STR(field)                                                                             \
    gw_mod *m = mod_at(boot(), i);                                                                 \
    return m != NULL ? m->field : ""

int gw_Mods_Count(void) { return boot()->n; }
const char *gw_Mods_Id(int i) { MOD_STR(id); }
const char *gw_Mods_Name(int i) { MOD_STR(name); }
const char *gw_Mods_Version(int i) { MOD_STR(version); }
const char *gw_Mods_Kind(int i) { MOD_STR(kind); }
const char *gw_Mods_Pack(int i) { MOD_STR(pack); }
const char *gw_Mods_Description(int i) { MOD_STR(desc); }
const char *gw_Mods_Requires(int i) { MOD_STR(requires_text); }
const char *gw_Mods_StatusText(int i) { MOD_STR(status_text); }
const char *gw_Mods_PayloadDir(int i) { MOD_STR(payload); }
int gw_Mods_Find(const char *id) { return id != NULL ? set_find(boot(), id) : -1; }

int gw_Mods_IsActive(int i) {
    gw_mod *m = mod_at(boot(), i);
    return m != NULL && m->status == GW_MOD_ACTIVE;
}
int gw_Mods_Status(int i) {
    gw_mod *m = mod_at(boot(), i);
    return m != NULL ? m->status : GW_MOD_OFF;
}
int gw_Mods_IsEnabled(int i) {
    gw_mod *m = mod_at(boot(), i);
    return m != NULL && m->enabled;
}
int gw_Mods_PayloadIsModDir(int i) {
    gw_mod *m = mod_at(boot(), i);
    return m != NULL && m->payload_is_mod_dir;
}
int gw_Mods_SetEnabled(int i, int on) {
    int n = set_enable(boot(), i, on != 0, 0);
    if (n > 0) gw_log("gw: mods: %s %s for the next boot (%d change(s))", gw_Mods_Id(i), on ? "on" : "off", n);
    return n;
}
int gw_Mods_Save(void) { return set_save(boot()); }
int gw_Mods_RestartNeeded(void) {
    gw_mods_set *s = boot();
    int i;
    for (i = 0; i < s->n; ++i) {
        if (s->mod[i].enabled != s->mod[i].enabled_boot) return 1;
    }
    return 0;
}
const char *gw_Mods_Dir(void) { return boot()->dir; }
int gw_Mods_ActiveCount(void) { return boot()->norder; }
int gw_Mods_ActiveAt(int n) {
    gw_mods_set *s = boot();
    return (n >= 0 && n < s->norder) ? s->order[n] : -1;
}

void gw_Mods_NoteFile(int i, const char *disc_path, uint32_t size) {
    gw_mod *m = mod_at(boot(), i);
    uint64_t h;
    if (m == NULL || disc_path == NULL) return;
    while (*disc_path == '/' || *disc_path == '\\') ++disc_path;
    h = fnv64_str_lower(FNV64_INIT, disc_path);
    h = fnv64(h, &size, sizeof size);
    m->file_sum += h;
    m->nfiles++;
}

uint64_t gw_Mods_Fingerprint(void) {
    extern void gw_DVDInit(void);
    gw_DVDInit(); /* idempotent: makes sure the overlay has reported its files */
    return set_fingerprint(boot());
}

const char *gw_Mods_Describe(void) {
    extern void gw_DVDInit(void);
    gw_DVDInit();
    return set_describe(boot());
}

int gw_Mods_DiffDescribe(const char *peer_desc, char *out, int cap) {
    return diff_describe(gw_Mods_Describe(), peer_desc, out, cap);
}

/* ---- tests -------------------------------------------------------------------------------------- */

static char gw_mods_test_root[MAX_PATH];

static int t_write(const char *rel, const char *text) {
    char path[MAX_PATH], dir[MAX_PATH];
    char *slash;
    FILE *f;
    snprintf(path, sizeof path, "%s\\%s", gw_mods_test_root, rel);
    copy_str(dir, sizeof dir, path);
    for (slash = dir + strlen(gw_mods_test_root) + 1; (slash = strchr(slash, '\\')) != NULL; ++slash) {
        *slash = '\0';
        CreateDirectoryA(dir, NULL);
        *slash = '\\';
    }
    f = fopen(path, "wb");
    if (f == NULL) return -1;
    fputs(text, f);
    fclose(f);
    return 0;
}

static void t_rmtree(const char *dir) {
    char pattern[MAX_PATH];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    snprintf(pattern, sizeof pattern, "%s\\*", dir);
    h = FindFirstFileA(pattern, &fd);
    if (h != INVALID_HANDLE_VALUE) {
        do {
            char p[MAX_PATH];
            if (strcmp(fd.cFileName, ".") == 0 || strcmp(fd.cFileName, "..") == 0) continue;
            snprintf(p, sizeof p, "%s\\%s", dir, fd.cFileName);
            if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) t_rmtree(p);
            else DeleteFileA(p);
        } while (FindNextFileA(h, &fd));
        FindClose(h);
    }
    RemoveDirectoryA(dir);
}

static int t_setup(void) {
    char tmp[MAX_PATH];
    DWORD n = GetTempPathA(sizeof tmp, tmp);
    if (n == 0 || n >= sizeof tmp) return -1;
    snprintf(gw_mods_test_root, sizeof gw_mods_test_root, "%sgw_mods_test_%lu", tmp,
             (unsigned long) GetCurrentProcessId());
    t_rmtree(gw_mods_test_root);
    CreateDirectoryA(gw_mods_test_root, NULL);
    /* an ACE-like split: a base, two fighters, a stage; an Akaneia base; a legacy mod */
    t_write("ace-base\\mod.json", "{\"id\":\"ace-base\",\"name\":\"ACE base\",\"version\":\"2.0.0\","
                                  "\"kind\":\"base\",\"pack\":\"ace\",\"extra\":{\"nested\":[1,2,{\"a\":\"b\"}]},"
                                  "\"conflicts\":[]}");
    t_write("ace-base\\files\\MxDt.dat", "x");
    t_write("ace-wolf\\mod.json", "{ \"name\": \"Wolf\", \"version\": \"2.0.0\", \"kind\": \"fighter\",\n"
                                  "  \"requires\": [ \"ace-base\" ], \"hash\": \"abc\" }");
    t_write("ace-wolf\\files\\PlWf.dat", "wolf");
    t_write("ace-sonic\\mod.json", "{\"kind\":\"fighter\",\"requires\":[\"ace-base\"],\"version\":\"2.0.0\"}");
    t_write("ace-sonic\\files\\PlSn.dat", "sonic");
    t_write("ace-stage-kcs\\mod.json", "{\"kind\":\"stage\",\"requires\":[\"ace-base\"]}");
    t_write("akaneia-base\\mod.json", "{\"kind\":\"base\",\"pack\":\"akaneia\"}");
    t_write("akaneia-sonic\\mod.json", "{\"kind\":\"fighter\",\"requires\":[\"akaneia-base\"]}");
    t_write("legacy\\PlXx.dat", "legacy");
    t_write("targettest\\mario-sample.tt", "not a mod");
    return 0;
}

static gw_mods_set *t_set(void) {
    static gw_mods_set s;
    return &s;
}

static int t_status(gw_mods_set *s, const char *id) {
    int i = set_find(s, id);
    return i >= 0 ? s->mod[i].status : -1;
}

static int test_mods_scan_and_json(void) {
    gw_mods_set *s = t_set();
    int i;
    if (t_setup() != 0) {
        gw_test_fail("cannot create a scratch folder");
        return 1;
    }
    set_load(s, gw_mods_test_root, 0);
    if (s->n != 7) {
        gw_test_fail("found %d mods, expected 7 (targettest must be skipped)", s->n);
        return 1;
    }
    i = set_find(s, "ace-wolf");
    if (i < 0 || strcmp(s->mod[i].name, "Wolf") != 0 || strcmp(s->mod[i].kind, "fighter") != 0 ||
        s->mod[i].nreq != 1 || strcmp(s->mod[i].req[0], "ace-base") != 0 ||
        strcmp(s->mod[i].hash, "abc") != 0 || s->mod[i].payload_is_mod_dir) {
        gw_test_fail("ace-wolf mod.json parsed wrong");
        return 1;
    }
    i = set_find(s, "ace-base");
    if (i < 0 || strcmp(s->mod[i].pack, "ace") != 0 || strcmp(s->mod[i].version, "2.0.0") != 0) {
        gw_test_fail("ace-base: nested value derailed the parser");
        return 1;
    }
    i = set_find(s, "legacy");
    if (i < 0 || !s->mod[i].payload_is_mod_dir || strcmp(s->mod[i].kind, "misc") != 0 ||
        strcmp(s->mod[i].name, "legacy") != 0) {
        gw_test_fail("legacy mod (no mod.json, no files/) not handled");
        return 1;
    }
    return 0;
}

static int test_mods_resolve(void) {
    gw_mods_set *s = t_set();
    int i, pos_base = -1, pos_wolf = -1;
    /* no enabled.txt: everything wanted; two bases -> akaneia-base loses (id order), its sonic too */
    set_load(s, gw_mods_test_root, 0);
    if (t_status(s, "ace-base") != GW_MOD_ACTIVE || t_status(s, "akaneia-base") != GW_MOD_CONFLICT ||
        t_status(s, "akaneia-sonic") != GW_MOD_MISSING_DEP || t_status(s, "ace-wolf") != GW_MOD_ACTIVE ||
        t_status(s, "legacy") != GW_MOD_ACTIVE) {
        gw_test_fail("all-enabled resolution wrong");
        return 1;
    }
    for (i = 0; i < s->norder; ++i) {
        if (strcmp(s->mod[s->order[i]].id, "ace-base") == 0) pos_base = i;
        if (strcmp(s->mod[s->order[i]].id, "ace-wolf") == 0) pos_wolf = i;
    }
    if (pos_base != 0 || pos_wolf <= pos_base) {
        gw_test_fail("mount order: base must mount first and before its dependents (%d, %d)", pos_base,
                     pos_wolf);
        return 1;
    }
    /* explicit set: a fighter without its base does not mount */
    t_write("enabled.txt", "# test\nace-wolf\r\n  akaneia-base  \nnot-installed\n");
    set_load(s, gw_mods_test_root, 0);
    if (!s->explicit_set || t_status(s, "ace-wolf") != GW_MOD_MISSING_DEP ||
        t_status(s, "akaneia-base") != GW_MOD_ACTIVE || t_status(s, "ace-sonic") != GW_MOD_OFF ||
        s->norder != 1) {
        gw_test_fail("explicit enabled set resolved wrong");
        return 1;
    }
    /* MELEE_MODS=0: nothing mounts, the list is still there */
    set_load(s, gw_mods_test_root, 1);
    if (s->n != 7 || s->norder != 0 || t_status(s, "akaneia-base") != GW_MOD_OFF) {
        gw_test_fail("MELEE_MODS=0 still mounted something");
        return 1;
    }
    return 0;
}

static int test_mods_toggle_and_save(void) {
    gw_mods_set *s = t_set();
    int n;
    t_write("enabled.txt", "akaneia-base\nakaneia-sonic\n");
    set_load(s, gw_mods_test_root, 0);
    /* enabling ace-wolf pulls in ace-base, which pushes out akaneia-base and its sonic */
    n = set_enable(s, set_find(s, "ace-wolf"), 1, 0);
    if (n != 4 || !s->mod[set_find(s, "ace-base")].enabled || s->mod[set_find(s, "akaneia-base")].enabled ||
        s->mod[set_find(s, "akaneia-sonic")].enabled) {
        gw_test_fail("enable cascade wrong (%d changes)", n);
        return 1;
    }
    set_enable(s, set_find(s, "ace-sonic"), 1, 0);
    if (set_save(s) != 0) {
        gw_test_fail("save failed");
        return 1;
    }
    set_load(s, gw_mods_test_root, 0);
    if (t_status(s, "ace-wolf") != GW_MOD_ACTIVE || t_status(s, "ace-sonic") != GW_MOD_ACTIVE ||
        t_status(s, "ace-base") != GW_MOD_ACTIVE || t_status(s, "akaneia-base") != GW_MOD_OFF || s->norder != 3) {
        gw_test_fail("saved enabled.txt did not round-trip");
        return 1;
    }
    /* disabling the base takes its dependents with it */
    n = set_enable(s, set_find(s, "ace-base"), 0, 0);
    if (n != 3 || s->mod[set_find(s, "ace-wolf")].enabled) {
        gw_test_fail("disable cascade wrong (%d changes)", n);
        return 1;
    }
    return 0;
}

static int test_mods_fingerprint_and_diff(void) {
    gw_mods_set *s = t_set();
    static gw_mods_set other;
    char desc_a[512], why[256];
    uint64_t fa, fb;
    int w;
    t_write("enabled.txt", "ace-base\nace-wolf\nace-sonic\n");
    set_load(s, gw_mods_test_root, 0);
    set_load(&other, gw_mods_test_root, 0);
    s->mod[set_find(s, "ace-wolf")].file_sum = other.mod[set_find(&other, "ace-wolf")].file_sum = 123;
    fa = set_fingerprint(s);
    fb = set_fingerprint(&other);
    if (fa == 0 || fa != fb) {
        gw_test_fail("same set, different fingerprints");
        return 1;
    }
    copy_str(desc_a, sizeof desc_a, set_describe(s));
    if (diff_describe(desc_a, set_describe(&other), why, sizeof why) != 0) {
        gw_test_fail("same set reported as different: %s", why);
        return 1;
    }
    /* a file differs inside ace-wolf */
    other.mod[set_find(&other, "ace-wolf")].file_sum = 124;
    if (set_fingerprint(&other) == fa) {
        gw_test_fail("a changed file did not change the fingerprint");
        return 1;
    }
    w = diff_describe(desc_a, set_describe(&other), why, sizeof why);
    if (w != 1 || strstr(why, "different files in ace-wolf") == NULL) {
        gw_test_fail("file difference not named: %d \"%s\"", w, why);
        return 1;
    }
    /* the peer lacks ace-sonic */
    other.mod[set_find(&other, "ace-wolf")].file_sum = 123;
    other.mod[set_find(&other, "ace-sonic")].status = GW_MOD_OFF;
    w = diff_describe(desc_a, set_describe(&other), why, sizeof why);
    if (w != 1 || strstr(why, "peer lacks ace-sonic") == NULL) {
        gw_test_fail("extra mod not named: %d \"%s\"", w, why);
        return 1;
    }
    w = diff_describe(set_describe(&other), desc_a, why, sizeof why);
    if (w != 1 || strstr(why, "you lack ace-sonic") == NULL) {
        gw_test_fail("missing mod not named: %d \"%s\"", w, why);
        return 1;
    }
    /* no mods on either side = 0, so a vanilla pair matches whatever else differs */
    set_load(&other, gw_mods_test_root, 1);
    if (set_fingerprint(&other) != 0 || set_describe(&other)[0] != '\0') {
        gw_test_fail("an empty set must fingerprint to 0 and describe as \"\"");
        return 1;
    }
    t_rmtree(gw_mods_test_root);
    return 0;
}

void gw_mods_tests_register(void) {
    gw_test_register("mods_scan_and_json", test_mods_scan_and_json);
    gw_test_register("mods_resolve", test_mods_resolve);
    gw_test_register("mods_toggle_and_save", test_mods_toggle_and_save);
    gw_test_register("mods_fingerprint_and_diff", test_mods_fingerprint_and_diff);
}
