/* gw_kit.c - the frontend kit for scripts: fonts, palette, textures and the frame's draw list
 * (gw_kit.h says what and why; docs/scripting.md "Kit drawing" is the script-facing reference).
 *
 * The rules are the game-side kit's (src/melee/gm/gmfrontend_kit.inc, gmfrontend_player.inc),
 * restated here for the host: the files are read once, textures are decoded from GX's tiled
 * formats into RGBA8 for ImGui, and every draw call becomes quads in the 640x480 screen that
 * gw_console.cpp renders in the overlay pass. */
#include "gw_kit.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <windows.h>

#include "gw_test.h"

extern void gw_log(const char *fmt, ...);
extern int gw_UiFile_Read(const char *name, void *dst, int cap);

/* ============================================================================================
 * a small JSON DOM (the kit's files are plain JSON; the font manifest is 350 KB, read once)
 * ============================================================================================ */
enum { KJ_NULL, KJ_FALSE, KJ_TRUE, KJ_NUM, KJ_STR, KJ_ARR, KJ_OBJ };

typedef struct {
    unsigned char type;
    const char *key;
    const char *s;
    double n;
    int child, next;
} KjNode;

typedef struct {
    char *p, *end;
    KjNode *pool;
    int n, cap;
    int bad;
} KjDoc;

static void kj_ws(KjDoc *d) {
    while (d->p < d->end && (*d->p == ' ' || *d->p == '\t' || *d->p == '\n' || *d->p == '\r')) d->p++;
}

static int kj_new(KjDoc *d, unsigned char type) {
    KjNode *nd;
    if (d->n >= d->cap) {
        int cap = d->cap ? d->cap * 2 : 1024;
        KjNode *np = (KjNode *)realloc(d->pool, (size_t)cap * sizeof *np);
        if (np == NULL) {
            d->bad = 1;
            return -1;
        }
        d->pool = np;
        d->cap = cap;
    }
    nd = &d->pool[d->n];
    memset(nd, 0, sizeof *nd);
    nd->type = type;
    nd->child = nd->next = -1;
    return d->n++;
}

/* A string, unescaped in place (\uXXXX becomes UTF-8, which is never longer than the escape). */
static const char *kj_string(KjDoc *d) {
    char *out, *start;
    if (d->p >= d->end || *d->p != '"') {
        d->bad = 1;
        return "";
    }
    start = out = ++d->p;
    while (d->p < d->end && *d->p != '"') {
        char c = *d->p++;
        if (c == '\\' && d->p < d->end) {
            c = *d->p++;
            if (c == 'u') {
                unsigned v = 0;
                int i;
                for (i = 0; i < 4 && d->p < d->end; i++, d->p++) {
                    char h = *d->p;
                    v = v * 16 + (unsigned)(h >= 'a' ? h - 'a' + 10 : h >= 'A' ? h - 'A' + 10 : h - '0');
                }
                if (v < 0x80) {
                    *out++ = (char)v;
                } else if (v < 0x800) {
                    *out++ = (char)(0xC0 | (v >> 6));
                    *out++ = (char)(0x80 | (v & 0x3F));
                } else {
                    *out++ = (char)(0xE0 | (v >> 12));
                    *out++ = (char)(0x80 | ((v >> 6) & 0x3F));
                    *out++ = (char)(0x80 | (v & 0x3F));
                }
                continue;
            }
            c = c == 'n' ? '\n' : c == 't' ? '\t' : c == 'r' ? '\r' : c == 'b' ? '\b' : c == 'f' ? '\f' : c;
        }
        *out++ = c;
    }
    if (d->p < d->end) d->p++;
    *out = '\0';
    return start;
}

static int kj_value(KjDoc *d);

static int kj_container(KjDoc *d, int obj) {
    int me = kj_new(d, obj ? KJ_OBJ : KJ_ARR), last = -1;
    d->p++;
    kj_ws(d);
    if (d->p < d->end && *d->p == (obj ? '}' : ']')) {
        d->p++;
        return me;
    }
    while (d->p < d->end && !d->bad) {
        const char *key = NULL;
        int v;
        kj_ws(d);
        if (obj) {
            key = kj_string(d);
            kj_ws(d);
            if (d->p >= d->end || *d->p != ':') {
                d->bad = 1;
                break;
            }
            d->p++;
        }
        v = kj_value(d);
        if (v < 0 || me < 0) break;
        d->pool[v].key = key;
        if (last < 0) d->pool[me].child = v;
        else d->pool[last].next = v;
        last = v;
        kj_ws(d);
        if (d->p < d->end && *d->p == ',') {
            d->p++;
            continue;
        }
        if (d->p < d->end && *d->p == (obj ? '}' : ']')) {
            d->p++;
            break;
        }
        d->bad = 1;
    }
    return me;
}

static int kj_value(KjDoc *d) {
    int v;
    kj_ws(d);
    if (d->p >= d->end) {
        d->bad = 1;
        return -1;
    }
    switch (*d->p) {
    case '{': return kj_container(d, 1);
    case '[': return kj_container(d, 0);
    case '"': {
        const char *s = kj_string(d);
        v = kj_new(d, KJ_STR);
        if (v >= 0) d->pool[v].s = s;
        return v;
    }
    case 't': d->p += 4; return kj_new(d, KJ_TRUE);
    case 'f': d->p += 5; return kj_new(d, KJ_FALSE);
    case 'n': d->p += 4; return kj_new(d, KJ_NULL);
    default: {
        char *e;
        double n = strtod(d->p, &e);
        if (e == d->p) {
            d->bad = 1;
            return -1;
        }
        d->p = e;
        v = kj_new(d, KJ_NUM);
        if (v >= 0) d->pool[v].n = n;
        return v;
    }
    }
}

/* Parses `text` (modified in place, must stay alive with the doc). Returns the root or NULL. */
static const KjNode *kj_parse(KjDoc *d, char *text, size_t len) {
    int root;
    memset(d, 0, sizeof *d);
    d->p = text;
    d->end = text + len;
    root = kj_value(d);
    if (d->bad || root < 0) return NULL;
    return &d->pool[root];
}

static void kj_free(KjDoc *d) {
    free(d->pool);
    d->pool = NULL;
}

/* Node helpers take the doc so indices resolve (the pool may move while parsing, not after). */
static const KjNode *kj_at(const KjDoc *d, int i) { return (i >= 0 && i < d->n) ? &d->pool[i] : NULL; }
static const KjNode *kj_first(const KjDoc *d, const KjNode *n) {
    return (n != NULL && (n->type == KJ_OBJ || n->type == KJ_ARR)) ? kj_at(d, n->child) : NULL;
}
static const KjNode *kj_next(const KjDoc *d, const KjNode *n) { return n != NULL ? kj_at(d, n->next) : NULL; }
static const KjNode *kj_get(const KjDoc *d, const KjNode *n, const char *key) {
    const KjNode *c;
    if (n == NULL || n->type != KJ_OBJ) return NULL;
    for (c = kj_first(d, n); c != NULL; c = kj_next(d, c)) {
        if (c->key != NULL && strcmp(c->key, key) == 0) return c;
    }
    return NULL;
}
static const KjNode *kj_idx(const KjDoc *d, const KjNode *n, int i) {
    const KjNode *c;
    for (c = kj_first(d, n); c != NULL && i > 0; c = kj_next(d, c), i--) {
    }
    return c;
}
static double kj_num(const KjNode *n, double def) { return (n != NULL && n->type == KJ_NUM) ? n->n : def; }
static const char *kj_str(const KjNode *n) { return (n != NULL && n->type == KJ_STR) ? n->s : NULL; }

/* ============================================================================================
 * files: the UI directories (the frontend's own search) and a mod's ui/ folder
 * ============================================================================================ */
static char *kit_read_path(const char *path, size_t *len) {
    FILE *f = fopen(path, "rb");
    long n;
    char *buf;
    if (f == NULL) return NULL;
    fseek(f, 0, SEEK_END);
    n = ftell(f);
    fseek(f, 0, SEEK_SET);
    buf = n >= 0 ? (char *)malloc((size_t)n + 1) : NULL;
    if (buf == NULL || fread(buf, 1, (size_t)n, f) != (size_t)n) {
        free(buf);
        fclose(f);
        return NULL;
    }
    fclose(f);
    buf[n] = '\0';
    *len = (size_t)n;
    return buf;
}

/* A kit file from the UI directories (gw_UiFile_Read's search), malloc'd and NUL-terminated. */
static char *kit_read_ui(const char *name, size_t *len) {
    int n = gw_UiFile_Read(name, NULL, 0);
    char *buf;
    if (n < 0) return NULL;
    buf = (char *)malloc((size_t)n + 1);
    if (buf == NULL) return NULL;
    if (gw_UiFile_Read(name, buf, n) != n) {
        free(buf);
        return NULL;
    }
    buf[n] = '\0';
    *len = (size_t)n;
    return buf;
}

/* The UI directories in search order, after a mod's own: MELEE_MENUTEX_DIR, then <exe>/ui,
 * <exe>/../../ui (_build/ui from a run sandbox), <exe>/../../../../ui (an agent's sandbox). */
static int kit_ui_dirs(char dirs[4][MAX_PATH]) {
    char exe[MAX_PATH];
    const char *env = getenv("MELEE_MENUTEX_DIR");
    static const char *const rel[] = {"ui", "..\\..\\ui", "..\\..\\..\\..\\ui"};
    DWORD n;
    char *slash;
    int k = 0, i;
    if (env != NULL && env[0] != '\0') {
        snprintf(dirs[k++], MAX_PATH, "%s", env);
    }
    n = GetModuleFileNameA(NULL, exe, (DWORD)sizeof exe);
    slash = (n > 0 && n < sizeof exe) ? strrchr(exe, '\\') : NULL;
    for (i = 0; slash != NULL && i < 3 && k < 4; i++) {
        *(slash + 1) = '\0';
        snprintf(dirs[k++], MAX_PATH, "%s%s", exe, rel[i]);
    }
    return k;
}

/* ============================================================================================
 * the font (font_manifest.json), the palette (kit.json) and the row template (list_layout.json)
 * ============================================================================================ */
#define KF_ROLES 12
#define KF_NCH 100 /* 0x20..0x7E, the ellipsis, the four arrows (as gmfrontend_kit.inc) */
#define KF_ELLIPSIS 95
#define KF_ARROW 96
#define KF_MAX_KERN 16000
#define KIT_SECTIONS 5

typedef struct {
    float adv, off[2], size[2], uv[4];
    signed char page; /* -1: no quad */
    unsigned char ok;
} KfGlyph;

typedef struct {
    char name[16];
    char face[8];
    float size, ascent, descent, cap, line;
    int caps;
    int npages;
    char page[2][48];
    int tex[2]; /* gw_Kit_Tex indices, -2 = not looked up yet */
    KfGlyph g[KF_NCH];
    int kern_first[KF_NCH], kern_count[KF_NCH];
} KfRole;

typedef struct {
    unsigned char a, b;
    short k; /* 1/100 px */
} KfKern;

static const char *const kit_section_names[KIT_SECTIONS] = {"versus", "solo", "collection",
                                                            "options", "data"};

static struct {
    int state; /* 0 not tried, 1 loaded, -1 unavailable */
    char why[128];
    KfRole r[KF_ROLES];
    int nr;
    KfKern kern[KF_MAX_KERN];
    int nk;
    char pal_name[24][16];
    uint32_t pal[24];
    int npal;
    uint32_t sec[KIT_SECTIONS][4]; /* face, bg, band, face_hi */
    uint32_t port[5];               /* p1..p4, cpu */
    /* list_layout.json */
    float row_h, pitch, label_x, label_base, lift[2], shear;
    char row_role[16];
    char st_face[3][16], st_label[3][16], st_plate[3][16], st_value[3][16];
} kf;

static int kf_char(const char *key) {
    const unsigned char *k = (const unsigned char *)key;
    if (k == NULL || k[0] == 0) return -1;
    if (k[1] == 0 && k[0] >= 0x20 && k[0] < 0x7F) return k[0] - 0x20;
    if (k[0] == 0xE2 && k[1] == 0x80 && k[2] == 0xA6 && k[3] == 0) return KF_ELLIPSIS;
    if (k[0] == 0xE2 && k[1] == 0x86 && k[2] >= 0x90 && k[2] <= 0x93 && k[3] == 0) return KF_ARROW + (k[2] - 0x90);
    return -1;
}

static int kit_hex(const char *s, uint32_t *rgba) {
    unsigned v = 0;
    size_t n, i;
    if (s == NULL || s[0] != '#') return 0;
    n = strlen(s + 1);
    if (n != 6 && n != 8) return 0;
    for (i = 1; i <= n; i++) {
        char h = s[i];
        int d = h >= '0' && h <= '9' ? h - '0' : h >= 'a' && h <= 'f' ? h - 'a' + 10 : h >= 'A' && h <= 'F' ? h - 'A' + 10 : -1;
        if (d < 0) return 0;
        v = v * 16 + (unsigned)d;
    }
    *rgba = n == 6 ? (v << 8) | 0xFFu : v;
    return 1;
}

static void kit_copy(char *dst, const char *src, size_t cap) {
    snprintf(dst, cap, "%s", src != NULL ? src : "");
}

static void kf_load_font(void) {
    size_t len = 0;
    char *text = kit_read_ui("font_manifest.json", &len);
    KjDoc d;
    const KjNode *M, *rn;
    if (text == NULL) {
        snprintf(kf.why, sizeof kf.why, "font_manifest.json not found in the UI directories");
        return;
    }
    M = kj_parse(&d, text, len);
    if (M == NULL) {
        snprintf(kf.why, sizeof kf.why, "font_manifest.json does not parse");
        kj_free(&d);
        free(text);
        return;
    }
    for (rn = kj_first(&d, kj_get(&d, M, "roles")); rn != NULL && kf.nr < KF_ROLES; rn = kj_next(&d, rn)) {
        KfRole *r = &kf.r[kf.nr];
        const KjNode *m = kj_get(&d, rn, "metrics"), *g, *p, *k;
        const char *cs = kj_str(kj_get(&d, rn, "charset"));
        int c, i;
        memset(r, 0, sizeof *r);
        kit_copy(r->name, rn->key, sizeof r->name);
        kit_copy(r->face, kj_str(kj_get(&d, rn, "face")), sizeof r->face);
        r->size = (float)kj_num(kj_get(&d, rn, "size"), 16);
        r->ascent = (float)kj_num(kj_get(&d, m, "ascent"), r->size);
        r->descent = (float)kj_num(kj_get(&d, m, "descent"), r->size * 0.25);
        r->cap = (float)kj_num(kj_get(&d, m, "cap_height"), r->size * 0.66);
        r->line = (float)kj_num(kj_get(&d, m, "line_height"), r->size * 1.25);
        r->caps = cs != NULL && strcmp(cs, "caps") == 0;
        r->tex[0] = r->tex[1] = -2;
        for (p = kj_first(&d, kj_get(&d, kj_get(&d, rn, "pages"), "latin")); p != NULL && r->npages < 2; p = kj_next(&d, p)) {
            kit_copy(r->page[r->npages++], kj_str(p), sizeof r->page[0]);
        }
        for (g = kj_first(&d, kj_get(&d, rn, "glyphs")); g != NULL; g = kj_next(&d, g)) {
            KfGlyph *o;
            c = kf_char(g->key);
            if (c < 0) continue;
            o = &r->g[c];
            o->ok = 1;
            o->adv = (float)kj_num(kj_get(&d, g, "advance"), 0);
            o->page = kj_get(&d, g, "uv") != NULL ? (signed char)kj_num(kj_get(&d, g, "page"), 0) : -1;
            for (i = 0; i < 4; i++) o->uv[i] = (float)kj_num(kj_idx(&d, kj_get(&d, g, "uv"), i), 0);
            for (i = 0; i < 2; i++) {
                o->size[i] = (float)kj_num(kj_idx(&d, kj_get(&d, g, "size"), i), 0);
                o->off[i] = (float)kj_num(kj_idx(&d, kj_get(&d, g, "offset"), i), 0);
            }
        }
        /* kerning pairs, bucketed by first character */
        for (c = 0; c < KF_NCH; c++) {
            int first = kf.nk;
            for (k = kj_first(&d, kj_get(&d, rn, "kerning")); k != NULL && kf.nk < KF_MAX_KERN; k = kj_next(&d, k)) {
                const unsigned char *key = (const unsigned char *)k->key;
                if (key == NULL || key[0] != c + 0x20 || key[1] < 0x20 || key[1] >= 0x7F || key[2] != 0) continue;
                kf.kern[kf.nk].a = key[0];
                kf.kern[kf.nk].b = key[1];
                kf.kern[kf.nk].k = (short)lround(kj_num(k, 0) * 100.0);
                kf.nk++;
            }
            r->kern_first[c] = first;
            r->kern_count[c] = kf.nk - first;
        }
        kf.nr++;
    }
    kj_free(&d);
    free(text);
    if (kf.nr == 0) snprintf(kf.why, sizeof kf.why, "font_manifest.json has no roles");
}

static void kf_load_palette(void) {
    size_t len = 0;
    char *text = kit_read_ui("kit.json", &len);
    KjDoc d;
    const KjNode *K, *n;
    int i;
    if (text == NULL) {
        if (kf.why[0] == '\0') snprintf(kf.why, sizeof kf.why, "kit.json not found in the UI directories");
        return;
    }
    K = kj_parse(&d, text, len);
    if (K != NULL) {
        for (n = kj_first(&d, kj_get(&d, K, "palette")); n != NULL && kf.npal < 24; n = kj_next(&d, n)) {
            if (kit_hex(kj_str(n), &kf.pal[kf.npal])) {
                kit_copy(kf.pal_name[kf.npal], n->key, sizeof kf.pal_name[0]);
                kf.npal++;
            }
        }
        for (i = 0; i < KIT_SECTIONS; i++) {
            const KjNode *s = kj_get(&d, kj_get(&d, K, "sections"), kit_section_names[i]);
            kit_hex(kj_str(kj_get(&d, s, "face")), &kf.sec[i][0]);
            kit_hex(kj_str(kj_get(&d, s, "bg")), &kf.sec[i][1]);
            kit_hex(kj_str(kj_get(&d, s, "band")), &kf.sec[i][2]);
            kit_hex(kj_str(kj_get(&d, s, "face_hi")), &kf.sec[i][3]);
        }
        {
            static const char *const pk[5] = {"p1", "p2", "p3", "p4", "cpu"};
            const KjNode *pc = kj_get(&d, kj_get(&d, K, "ports"), "colours");
            for (i = 0; i < 5; i++) kit_hex(kj_str(kj_get(&d, pc, pk[i])), &kf.port[i]);
        }
    } else if (kf.why[0] == '\0') {
        snprintf(kf.why, sizeof kf.why, "kit.json does not parse");
    }
    kj_free(&d);
    free(text);
}

static void kf_load_rows(void) {
    size_t len = 0;
    char *text;
    KjDoc d;
    const KjNode *L, *row, *s;
    static const char *const st[3] = {"ng", "sel", "disabled"};
    int i;
    /* the delivery's values, so a missing file still draws the template */
    kf.row_h = 30;
    kf.pitch = 34;
    kf.label_x = 16;
    kf.label_base = 20.28f;
    kf.lift[0] = kf.lift[1] = -3;
    kf.shear = 0.25f;
    kit_copy(kf.row_role, "row", sizeof kf.row_role);
    for (i = 0; i < 3; i++) {
        kit_copy(kf.st_face[i], i == 1 ? "gold" : "@face", 16);
        kit_copy(kf.st_label[i], i == 0 ? "bone" : i == 1 ? "ink" : "disabled", 16);
        kit_copy(kf.st_plate[i], i == 1 ? "gold_dk" : "", 16);
        kit_copy(kf.st_value[i], i == 0 ? "muted" : i == 1 ? "ink" : "disabled", 16); /* widgets' "aux" */
    }
    text = kit_read_ui("list_layout.json", &len);
    if (text == NULL) return;
    L = kj_parse(&d, text, len);
    if (L != NULL) {
        const KjNode *pn = kj_get(&d, L, "panel");
        kf.shear = (float)kj_num(kj_get(&d, kj_get(&d, L, "shear"), "S"), kf.shear);
        kf.row_h = (float)kj_num(kj_get(&d, pn, "row_h"), kf.row_h);
        kf.pitch = (float)kj_num(kj_get(&d, pn, "pitch"), kf.pitch);
        row = kj_idx(&d, kj_get(&d, L, "templates"), 0);
        for (s = kj_first(&d, kj_get(&d, row, "slots")); s != NULL; s = kj_next(&d, s)) {
            const char *id = kj_str(kj_get(&d, s, "id"));
            if (id != NULL && strcmp(id, "label") == 0) {
                kf.label_x = (float)kj_num(kj_idx(&d, kj_get(&d, s, "anchor"), 0), kf.label_x);
                kf.label_base = (float)kj_num(kj_idx(&d, kj_get(&d, s, "anchor"), 1), kf.label_base);
                if (kj_str(kj_get(&d, s, "role")) != NULL) kit_copy(kf.row_role, kj_str(kj_get(&d, s, "role")), 16);
            }
        }
        for (i = 0; i < 3; i++) {
            const KjNode *sn = kj_get(&d, kj_get(&d, row, "states"), st[i]);
            if (sn == NULL) continue;
            if (kj_str(kj_get(&d, sn, "face")) != NULL) kit_copy(kf.st_face[i], kj_str(kj_get(&d, sn, "face")), 16);
            if (kj_str(kj_get(&d, sn, "label")) != NULL) kit_copy(kf.st_label[i], kj_str(kj_get(&d, sn, "label")), 16);
            kit_copy(kf.st_plate[i], kj_str(kj_get(&d, sn, "plate")), 16); /* null: no plate */
            if (i == 1) {
                kf.lift[0] = (float)kj_num(kj_idx(&d, kj_get(&d, sn, "offset"), 0), kf.lift[0]);
                kf.lift[1] = (float)kj_num(kj_idx(&d, kj_get(&d, sn, "offset"), 1), kf.lift[1]);
            }
        }
    }
    kj_free(&d);
    free(text);
}

static void kf_load(void) {
    if (kf.state != 0) return;
    kf.state = -1;
    kf_load_font();
    kf_load_palette();
    kf_load_rows();
    if (kf.nr > 0 && kf.npal > 0) {
        kf.state = 1;
        gw_log("kit: scripts' kit - %d text roles, %d kerning pairs, %d palette colours", kf.nr, kf.nk, kf.npal);
    } else {
        gw_log("kit: scripts' kit unavailable - %s", kf.why);
    }
}

int gw_Kit_Available(void) {
    kf_load();
    return kf.state == 1;
}

const char *gw_Kit_Why(void) {
    kf_load();
    return kf.state == 1 ? "" : kf.why;
}

int gw_Kit_RoleCount(void) {
    kf_load();
    return kf.nr;
}

const char *gw_Kit_RoleName(int role) {
    kf_load();
    return (role >= 0 && role < kf.nr) ? kf.r[role].name : NULL;
}

int gw_Kit_Role(const char *name) {
    int i;
    kf_load();
    for (i = 0; name != NULL && i < kf.nr; i++) {
        if (strcmp(kf.r[i].name, name) == 0) return i;
    }
    return -1;
}

int gw_Kit_RoleMetrics(int role, float *size, float *ascent, float *descent, float *cap, float *line) {
    const KfRole *r;
    kf_load();
    if (role < 0 || role >= kf.nr) return 0;
    r = &kf.r[role];
    if (size) *size = r->size;
    if (ascent) *ascent = r->ascent;
    if (descent) *descent = r->descent;
    if (cap) *cap = r->cap;
    if (line) *line = r->line;
    return 1;
}

static float kf_kern(const KfRole *r, int prev, int ch) {
    int i;
    if (prev < 0 || prev >= 95 || ch >= 95) return 0.0f;
    for (i = r->kern_first[prev]; i < r->kern_first[prev] + r->kern_count[prev]; i++) {
        if (kf.kern[i].b == ch + 0x20) return (float)kf.kern[i].k / 100.0f;
    }
    return 0.0f;
}

/* The glyph slot a byte sets in role r: caps roles uppercase, 0x01 the ellipsis, 0x02..0x05 the
 * arrows, a missing character '?'. */
static int kf_slot(const KfRole *r, unsigned char c) {
    int s;
    if (r->caps && c >= 'a' && c <= 'z') c = (unsigned char)(c - 'a' + 'A');
    if (c == 0x01) s = KF_ELLIPSIS;
    else if (c >= 0x02 && c <= 0x05) s = KF_ARROW + (c - 0x02);
    else s = (c >= 0x20 && c < 0x7F) ? c - 0x20 : -1;
    if (s < 0 || !r->g[s].ok) s = '?' - 0x20;
    return s;
}

/* Script text is UTF-8: the ellipsis and the arrows become the kit's internal bytes, any other
 * non-ASCII character one '?'. */
static void kf_from_utf8(const char *in, char *out, int cap) {
    const unsigned char *s = (const unsigned char *)in;
    int n = 0;
    while (*s != 0 && n < cap - 1) {
        if (*s < 0x80) {
            out[n++] = (char)*s++;
        } else if (s[0] == 0xE2 && s[1] == 0x80 && s[2] == 0xA6) {
            out[n++] = 0x01;
            s += 3;
        } else if (s[0] == 0xE2 && s[1] == 0x86 && s[2] >= 0x90 && s[2] <= 0x93) {
            out[n++] = (char)(0x02 + (s[2] - 0x90));
            s += 3;
        } else {
            out[n++] = '?';
            s++;
            while ((*s & 0xC0) == 0x80) s++;
        }
    }
    out[n] = '\0';
}

static float kf_width_raw(int role, const char *s) {
    const KfRole *r = &kf.r[role];
    float x = 0.0f;
    int prev = -1;
    for (; *s != '\0'; s++) {
        int c = kf_slot(r, (unsigned char)*s);
        x += kf_kern(r, prev, c) + r->g[c].adv;
        prev = c;
    }
    return x;
}

float gw_Kit_TextWidth(int role, const char *s) {
    char buf[512];
    kf_load();
    if (role < 0 || role >= kf.nr || s == NULL) return 0.0f;
    kf_from_utf8(s, buf, sizeof buf);
    return kf_width_raw(role, buf);
}

static int kf_fit_raw(int role, const char *s, float max_w, char *out, int cap) {
    int i, n;
    kit_copy(out, s, (size_t)cap);
    if (max_w <= 0.0f) return role;
    while (kf_width_raw(role, out) > max_w) {
        int best = -1;
        for (i = 0; i < kf.nr; i++) {
            if (strcmp(kf.r[i].face, kf.r[role].face) == 0 && kf.r[i].size < kf.r[role].size &&
                (best < 0 || kf.r[i].size > kf.r[best].size)) {
                best = i;
            }
        }
        if (best < 0) break;
        role = best;
    }
    n = (int)strlen(out);
    while (n > 1 && kf_width_raw(role, out) > max_w) {
        out[n - 2] = 0x01;
        out[n - 1] = '\0';
        n--;
    }
    return role;
}

int gw_Kit_Fit(int role, const char *s, float max_w, char *out, int cap) {
    char buf[512];
    kf_load();
    if (role < 0 || role >= kf.nr || s == NULL || cap <= 0) return role;
    kf_from_utf8(s, buf, sizeof buf);
    return kf_fit_raw(role, buf, max_w, out, cap);
}

float gw_Kit_Shear(void) {
    kf_load();
    return kf.shear;
}

void gw_Kit_RowMetrics(float *h, float *pitch, float *label_x, float *label_base, float *lift) {
    kf_load();
    if (h) *h = kf.row_h;
    if (pitch) *pitch = kf.pitch;
    if (label_x) *label_x = kf.label_x;
    if (label_base) *label_base = kf.label_base;
    if (lift) *lift = kf.lift[1];
}

/* ============================================================================================
 * a mod's ui/ manifests (*_ui.json): texture sizes and tints, palette names
 * ============================================================================================ */
#define KM_DIRS 16
#define KM_TEX 256
#define KM_PAL 128

typedef struct {
    char dir[MAX_PATH];
    int ntex, npal;
    char tex_name[KM_TEX][48];
    float tex_w1x[KM_TEX], tex_h1x[KM_TEX];
    char tex_tint[KM_TEX][24];
    char pal_name[KM_PAL][24];
    uint32_t pal[KM_PAL];
} KitMod;

static KitMod *km[KM_DIRS];
static int nkm;

static void km_palette_entry(KitMod *m, const char *name, const KjDoc *d, const KjNode *v) {
    uint32_t c;
    const char *hex = kj_str(v);
    if (m->npal >= KM_PAL || name == NULL) return;
    if (hex == NULL && v != NULL && v->type == KJ_OBJ) {
        const KjNode *u = kj_get(d, v, "u32");
        const char *us = kj_str(u);
        if (us != NULL && (us[0] == '0') && (us[1] == 'x' || us[1] == 'X')) {
            m->pal[m->npal] = (uint32_t)strtoul(us + 2, NULL, 16);
            kit_copy(m->pal_name[m->npal++], name, 24);
            return;
        }
        hex = kj_str(kj_get(d, v, "hex"));
    }
    if (kit_hex(hex, &c)) {
        m->pal[m->npal] = c;
        kit_copy(m->pal_name[m->npal++], name, 24);
    }
}

static void km_read(KitMod *m, const char *path) {
    size_t len = 0;
    char *text = kit_read_path(path, &len);
    KjDoc d;
    const KjNode *R, *t, *g, *e;
    if (text == NULL) return;
    R = kj_parse(&d, text, len);
    if (R != NULL) {
        for (t = kj_first(&d, kj_get(&d, R, "textures")); t != NULL && m->ntex < KM_TEX; t = kj_next(&d, t)) {
            const char *name = kj_str(kj_get(&d, t, "name"));
            const KjNode *sz = kj_get(&d, t, "size_1x");
            if (name == NULL) continue;
            kit_copy(m->tex_name[m->ntex], name, 48);
            m->tex_w1x[m->ntex] = (float)kj_num(kj_idx(&d, sz, 0), 0);
            m->tex_h1x[m->ntex] = (float)kj_num(kj_idx(&d, sz, 1), 0);
            kit_copy(m->tex_tint[m->ntex], kj_str(kj_get(&d, t, "tint")), 24);
            m->ntex++;
        }
        /* "palette": {name: "#hex" | {hex, u32}} or groups of those one level down */
        for (g = kj_first(&d, kj_get(&d, R, "palette")); g != NULL; g = kj_next(&d, g)) {
            if (g->type == KJ_STR || kj_get(&d, g, "hex") != NULL || kj_get(&d, g, "u32") != NULL) {
                km_palette_entry(m, g->key, &d, g);
            } else if (g->type == KJ_OBJ) {
                for (e = kj_first(&d, g); e != NULL; e = kj_next(&d, e)) km_palette_entry(m, e->key, &d, e);
            }
        }
    }
    kj_free(&d);
    free(text);
}

static const KitMod *km_get(const char *mod_ui) {
    int i;
    KitMod *m;
    char pat[MAX_PATH + 16];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    if (mod_ui == NULL || mod_ui[0] == '\0') return NULL;
    for (i = 0; i < nkm; i++) {
        if (_stricmp(km[i]->dir, mod_ui) == 0) return km[i];
    }
    if (nkm >= KM_DIRS) return NULL;
    m = (KitMod *)calloc(1, sizeof *m);
    if (m == NULL) return NULL;
    kit_copy(m->dir, mod_ui, sizeof m->dir);
    snprintf(pat, sizeof pat, "%s\\*_ui.json", mod_ui);
    h = FindFirstFileA(pat, &fd);
    if (h != INVALID_HANDLE_VALUE) {
        do {
            char path[MAX_PATH * 2];
            snprintf(path, sizeof path, "%s\\%s", mod_ui, fd.cFileName);
            km_read(m, path);
        } while (FindNextFileA(h, &fd));
        FindClose(h);
    }
    km[nkm++] = m;
    if (m->ntex > 0 || m->npal > 0) {
        gw_log("kit: %s - %d texture entries, %d palette colours from its *_ui.json", mod_ui, m->ntex, m->npal);
    }
    return m;
}

/* ============================================================================================
 * colours
 * ============================================================================================ */
int gw_Kit_Colour(const char *tok, const char *mod_ui, uint32_t *rgba) {
    int i;
    const KitMod *m;
    kf_load();
    if (tok == NULL || tok[0] == '\0') return 0;
    if (tok[0] == '#') return kit_hex(tok, rgba);
    if (tok[0] == '@') {
        int w = strcmp(tok, "@face") == 0 ? 0 : strcmp(tok, "@bg") == 0 ? 1 : strcmp(tok, "@band") == 0 ? 2 : strcmp(tok, "@face_hi") == 0 ? 3 : -1;
        if (w < 0) return 0;
        *rgba = kf.sec[0][w];
        return 1;
    }
    if (strncmp(tok, "port:", 5) == 0) tok += 5;
    if ((tok[0] == 'p' && tok[1] >= '1' && tok[1] <= '4' && tok[2] == '\0') || strcmp(tok, "cpu") == 0) {
        *rgba = kf.port[tok[0] == 'p' ? tok[1] - '1' : 4];
        return 1;
    }
    {
        const char *dot = strchr(tok, '.');
        if (dot != NULL) {
            for (i = 0; i < KIT_SECTIONS; i++) {
                size_t n = strlen(kit_section_names[i]);
                if ((size_t)(dot - tok) == n && strncmp(tok, kit_section_names[i], n) == 0) {
                    const char *w = dot + 1;
                    int k = strcmp(w, "face") == 0 ? 0 : strcmp(w, "bg") == 0 ? 1 : strcmp(w, "band") == 0 ? 2 : strcmp(w, "face_hi") == 0 ? 3 : -1;
                    if (k < 0) return 0;
                    *rgba = kf.sec[i][k];
                    return 1;
                }
            }
        }
    }
    m = km_get(mod_ui);
    for (i = 0; m != NULL && i < m->npal; i++) { /* the mod's own names win over the kit's */
        if (strcmp(m->pal_name[i], tok) == 0) {
            *rgba = m->pal[i];
            return 1;
        }
    }
    for (i = 0; i < kf.npal; i++) {
        if (strcmp(kf.pal_name[i], tok) == 0) {
            *rgba = kf.pal[i];
            return 1;
        }
    }
    return 0;
}

int gw_Kit_PaletteCount(void) {
    kf_load();
    return kf.npal;
}
const char *gw_Kit_PaletteName(int i) { return (i >= 0 && i < kf.npal) ? kf.pal_name[i] : NULL; }
uint32_t gw_Kit_PaletteRGBA(int i) { return (i >= 0 && i < kf.npal) ? kf.pal[i] : 0; }
int gw_Kit_SectionCount(void) { return KIT_SECTIONS; }
const char *gw_Kit_SectionName(int i) { return (i >= 0 && i < KIT_SECTIONS) ? kit_section_names[i] : NULL; }
uint32_t gw_Kit_SectionRGBA(int section, int which) {
    kf_load();
    if (section < 0 || section >= KIT_SECTIONS || which < 0 || which > 3) return 0;
    return kf.sec[section][which];
}

/* ============================================================================================
 * textures: .gxtex (pc/tools/png2gx.py) decoded from GX's tiled formats into RGBA8
 * ============================================================================================ */
#define KT_MAX 256

typedef struct {
    char path[MAX_PATH]; /* the file it came from (the cache key) */
    char name[48];
    int w, h, fmt, mask;
    float w1x, h1x;
    char tint[24];
    uint8_t *rgba;
} KitTex;

static KitTex *kt[KT_MAX];
static int nkt;

static uint32_t kt_be32(const uint8_t *p) {
    return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
}
static unsigned kt_be16(const uint8_t *p) { return ((unsigned)p[0] << 8) | p[1]; }

static void kt_put(uint8_t *out, int w, int h, int x, int y, int r, int g, int b, int a) {
    uint8_t *p;
    if (x >= w || y >= h) return; /* the tile padding past the image */
    p = out + ((size_t)y * w + x) * 4;
    p[0] = (uint8_t)r;
    p[1] = (uint8_t)g;
    p[2] = (uint8_t)b;
    p[3] = (uint8_t)a;
}

static void kt_rgb5a3(unsigned v, int *r, int *g, int *b, int *a) {
    if (v & 0x8000) {
        *r = (int)(((v >> 10) & 31) * 255 / 31);
        *g = (int)(((v >> 5) & 31) * 255 / 31);
        *b = (int)((v & 31) * 255 / 31);
        *a = 255;
    } else {
        *a = (int)(((v >> 12) & 7) * 255 / 7);
        *r = (int)(((v >> 8) & 15) * 17);
        *g = (int)(((v >> 4) & 15) * 17);
        *b = (int)((v & 15) * 17);
    }
}

static void kt_tlut(unsigned v, int fmt, int *r, int *g, int *b, int *a) {
    if (fmt == 0) { /* IA8 */
        *a = (int)(v >> 8);
        *r = *g = *b = (int)(v & 0xFF);
    } else if (fmt == 1) { /* RGB565 */
        *r = (int)(((v >> 11) & 31) * 255 / 31);
        *g = (int)(((v >> 5) & 63) * 255 / 63);
        *b = (int)((v & 31) * 255 / 31);
        *a = 255;
    } else {
        kt_rgb5a3(v, r, g, b, a);
    }
}

/* Decodes one GX image. Masks (I4/I8/IA4/IA8) come out white with the texture's alpha - the
 * kit's mask TEV takes RGB from the tint and alpha from the texture. Returns NULL for a format
 * this does not read (CMPR) or a short image. */
static uint8_t *kt_decode(const uint8_t *img, size_t img_size, int fmt, int w, int h,
                          const uint8_t *tlut, int tlut_fmt, int tlut_n) {
    static const int bw[] = {8, 8, 8, 4, 4, 4, 4, 0, 8, 8, 4};
    static const int bh[] = {8, 4, 4, 4, 4, 4, 4, 0, 8, 4, 4};
    uint8_t *out;
    int bx, by, x, y, tw, th;
    size_t off = 0, need;
    if (fmt < 0 || fmt > 10 || bw[fmt] == 0 || w <= 0 || h <= 0) return NULL;
    tw = (w + bw[fmt] - 1) / bw[fmt];
    th = (h + bh[fmt] - 1) / bh[fmt];
    need = (size_t)tw * th * (fmt == 6 ? 64 : 32);
    if (img_size < need) return NULL;
    out = (uint8_t *)calloc((size_t)w * h, 4);
    if (out == NULL) return NULL;
    for (by = 0; by < th; by++) {
        for (bx = 0; bx < tw; bx++) {
            int x0 = bx * bw[fmt], y0 = by * bh[fmt];
            switch (fmt) {
            case 0: /* I4 */
                for (y = 0; y < 8; y++)
                    for (x = 0; x < 8; x += 2, off++) {
                        int hi = (img[off] >> 4) * 17, lo = (img[off] & 15) * 17;
                        kt_put(out, w, h, x0 + x, y0 + y, 255, 255, 255, hi);
                        kt_put(out, w, h, x0 + x + 1, y0 + y, 255, 255, 255, lo);
                    }
                break;
            case 1: /* I8 */
                for (y = 0; y < 4; y++)
                    for (x = 0; x < 8; x++, off++) kt_put(out, w, h, x0 + x, y0 + y, 255, 255, 255, img[off]);
                break;
            case 2: /* IA4: alpha in the high nibble */
                for (y = 0; y < 4; y++)
                    for (x = 0; x < 8; x++, off++) kt_put(out, w, h, x0 + x, y0 + y, 255, 255, 255, (img[off] >> 4) * 17);
                break;
            case 3: /* IA8: alpha byte first */
                for (y = 0; y < 4; y++)
                    for (x = 0; x < 4; x++, off += 2) kt_put(out, w, h, x0 + x, y0 + y, 255, 255, 255, img[off]);
                break;
            case 4: /* RGB565 */
                for (y = 0; y < 4; y++)
                    for (x = 0; x < 4; x++, off += 2) {
                        int r, g, b, a;
                        kt_tlut(kt_be16(img + off), 1, &r, &g, &b, &a);
                        kt_put(out, w, h, x0 + x, y0 + y, r, g, b, a);
                    }
                break;
            case 5: /* RGB5A3 */
                for (y = 0; y < 4; y++)
                    for (x = 0; x < 4; x++, off += 2) {
                        int r, g, b, a;
                        kt_rgb5a3(kt_be16(img + off), &r, &g, &b, &a);
                        kt_put(out, w, h, x0 + x, y0 + y, r, g, b, a);
                    }
                break;
            case 6: /* RGBA8: 32 bytes of AR pairs, then 32 of GB */
                for (y = 0; y < 4; y++)
                    for (x = 0; x < 4; x++) {
                        int i = y * 4 + x;
                        kt_put(out, w, h, x0 + x, y0 + y, img[off + i * 2 + 1], img[off + 32 + i * 2],
                               img[off + 32 + i * 2 + 1], img[off + i * 2]);
                    }
                off += 64;
                break;
            case 8:  /* CI4 */
            case 9:  /* CI8 */
            case 10: /* CI14X2 */
                for (y = 0; y < bh[fmt]; y++)
                    for (x = 0; x < bw[fmt]; x++) {
                        int idx, r = 0, g = 0, b = 0, a = 0;
                        if (fmt == 8) {
                            idx = (x & 1) ? (img[off] & 15) : (img[off] >> 4);
                            if (x & 1) off++;
                        } else if (fmt == 9) {
                            idx = img[off++];
                        } else {
                            idx = (int)(kt_be16(img + off) & 0x3FFF);
                            off += 2;
                        }
                        if (tlut != NULL && idx < tlut_n) kt_tlut(kt_be16(tlut + idx * 2), tlut_fmt, &r, &g, &b, &a);
                        kt_put(out, w, h, x0 + x, y0 + y, r, g, b, a);
                    }
                break;
            }
        }
    }
    return out;
}

/* Reads and decodes `dir`\`name`.gxtex into a new cache entry; -1 when the file is missing. */
static int kt_load(const char *dir, const char *name) {
    char path[MAX_PATH * 2];
    size_t len = 0;
    uint8_t *blob;
    KitTex *t;
    uint32_t fmt, w, h, tfmt, tn, isz, tsz, ioff, toff;
    int i;
    snprintf(path, sizeof path, "%s\\%s.gxtex", dir, name);
    for (i = 0; i < nkt; i++) {
        if (_stricmp(kt[i]->path, path) == 0) return kt[i]->rgba != NULL ? i : -1;
    }
    blob = (uint8_t *)kit_read_path(path, &len);
    if (blob == NULL) return -1;
    if (nkt >= KT_MAX || len < 64 || kt_be32(blob) != 0x47585458u || kt_be32(blob + 4) != 1) {
        gw_log("kit: %s is not a v1 .gxtex (or the cache is full)", path);
        free(blob);
        return -1;
    }
    fmt = kt_be32(blob + 8);
    w = kt_be32(blob + 12);
    h = kt_be32(blob + 16);
    tfmt = kt_be32(blob + 20);
    tn = kt_be32(blob + 24);
    isz = kt_be32(blob + 28);
    tsz = kt_be32(blob + 32);
    ioff = kt_be32(blob + 36);
    toff = kt_be32(blob + 40);
    if ((uint64_t)ioff + isz > len || (tsz != 0 && (uint64_t)toff + tsz > len) || w > 4096 || h > 4096) {
        gw_log("kit: %s: the image does not fit in the file", path);
        free(blob);
        return -1;
    }
    t = (KitTex *)calloc(1, sizeof *t);
    if (t == NULL) {
        free(blob);
        return -1;
    }
    kit_copy(t->path, path, sizeof t->path);
    kit_copy(t->name, name, sizeof t->name);
    t->w = (int)w;
    t->h = (int)h;
    t->fmt = (int)fmt;
    t->mask = fmt <= 3;
    t->w1x = (float)w * 0.5f;
    t->h1x = (float)h * 0.5f;
    t->rgba = kt_decode(blob + ioff, isz, (int)fmt, (int)w, (int)h, tsz != 0 ? blob + toff : NULL,
                        (int)tfmt, (int)(tsz / 2 < tn ? tsz / 2 : tn));
    free(blob);
    kt[nkt] = t;
    if (t->rgba == NULL) {
        gw_log("kit: %s: format %u is not decoded here", path, (unsigned)fmt);
        return nkt++, -1;
    }
    return nkt++;
}

int gw_Kit_Tex(const char *name, const char *mod_ui) {
    char dirs[4][MAX_PATH];
    int n, i, t = -1;
    if (name == NULL || name[0] == '\0' || strchr(name, '/') != NULL || strchr(name, '\\') != NULL ||
        strstr(name, "..") != NULL) {
        return -1;
    }
    if (mod_ui != NULL && mod_ui[0] != '\0') {
        t = kt_load(mod_ui, name);
        if (t >= 0) {
            const KitMod *m = km_get(mod_ui);
            for (i = 0; m != NULL && i < m->ntex; i++) {
                if (strcmp(m->tex_name[i], name) == 0) {
                    if (m->tex_w1x[i] > 0) kt[t]->w1x = m->tex_w1x[i];
                    if (m->tex_h1x[i] > 0) kt[t]->h1x = m->tex_h1x[i];
                    kit_copy(kt[t]->tint, m->tex_tint[i], sizeof kt[t]->tint);
                }
            }
            return t;
        }
    }
    n = kit_ui_dirs(dirs);
    for (i = 0; i < n && t < 0; i++) t = kt_load(dirs[i], name);
    return t;
}

int gw_Kit_TexInfo(int tex, int *w, int *h, float *w1x, float *h1x, int *mask, const char **tint) {
    const KitTex *t;
    if (tex < 0 || tex >= nkt || kt[tex]->rgba == NULL) return 0;
    t = kt[tex];
    if (w) *w = t->w;
    if (h) *h = t->h;
    if (w1x) *w1x = t->w1x;
    if (h1x) *h1x = t->h1x;
    if (mask) *mask = t->mask;
    if (tint) *tint = t->tint;
    return 1;
}

const uint8_t *gw_Kit_TexPixels(int tex) { return (tex >= 0 && tex < nkt) ? kt[tex]->rgba : NULL; }
int gw_Kit_TexCount(void) { return nkt; }

/* ============================================================================================
 * the frame's draw list
 * ============================================================================================ */
#define KQ_MAX 16384
static GwKitQuad kq[KQ_MAX];
static int nkq;

void gw_Kit_BeginFrame(void) { nkq = 0; }
int gw_Kit_QuadCount(void) { return nkq; }
void gw_Kit_TruncateQuads(int n) { if (n >= 0 && n < nkq) nkq = n; }
const GwKitQuad *gw_Kit_QuadAt(int i) { return (i >= 0 && i < nkq) ? &kq[i] : NULL; }

/* A quad from its unsheared corners, sheared about y0: x' = x + (y0 - y) * shear. */
static int kq_add(float x0, float y0, float x1, float y1, float u0, float v0, float u1, float v1,
                  uint32_t rgba, int tex, float shear, float sy) {
    GwKitQuad *q;
    if (nkq >= KQ_MAX || (rgba & 0xFF) == 0) return 0;
    q = &kq[nkq++];
    q->x[0] = x0; q->y[0] = y0; q->u[0] = u0; q->v[0] = v0;
    q->x[1] = x1; q->y[1] = y0; q->u[1] = u1; q->v[1] = v0;
    q->x[2] = x1; q->y[2] = y1; q->u[2] = u1; q->v[2] = v1;
    q->x[3] = x0; q->y[3] = y1; q->u[3] = u0; q->v[3] = v1;
    if (shear != 0.0f) {
        int k;
        for (k = 0; k < 4; k++) q->x[k] += (sy - q->y[k]) * shear;
    }
    q->rgba = rgba;
    q->tex = tex;
    return 1;
}

int gw_Kit_DrawText(float x, float y, const char *s, int role, uint32_t rgba, int align,
                    float max_w, float shear, float *out_w) {
    char buf[512];
    const KfRole *r;
    float pen, w;
    int prev = -1, added = 0, page;
    const char *c;
    kf_load();
    if (out_w) *out_w = 0.0f;
    if (kf.state != 1 || role < 0 || role >= kf.nr || s == NULL) return 0;
    {
        char raw[512];
        kf_from_utf8(s, raw, sizeof raw);
        role = kf_fit_raw(role, raw, max_w, buf, sizeof buf);
    }
    r = &kf.r[role];
    w = kf_width_raw(role, buf);
    if (out_w) *out_w = w;
    pen = align == GW_KIT_ALIGN_CENTER ? x - w * 0.5f : align == GW_KIT_ALIGN_RIGHT ? x - w : x;
    for (page = 0; page < r->npages; page++) {
        KfRole *rw = &kf.r[role];
        if (rw->tex[page] == -2) rw->tex[page] = gw_Kit_Tex(rw->page[page], NULL);
    }
    for (c = buf; *c != '\0'; c++) {
        int ch = kf_slot(r, (unsigned char)*c);
        const KfGlyph *g = &r->g[ch];
        pen += kf_kern(r, prev, ch);
        prev = ch;
        if (g->page >= 0 && g->page < r->npages && r->tex[g->page] >= 0 && g->size[0] > 0.0f) {
            float gx = pen + g->off[0], gy = y + g->off[1];
            added += kq_add(gx, gy, gx + g->size[0], gy + g->size[1], g->uv[0], g->uv[1], g->uv[2], g->uv[3],
                            rgba, r->tex[g->page], shear, y);
        }
        pen += g->adv;
    }
    return added;
}

int gw_Kit_DrawImage(int tex, float x, float y, float w, float h, uint32_t rgba, int flip, float shear) {
    float u0 = 0, v0 = 0, u1 = 1, v1 = 1;
    if (tex < 0 || tex >= nkt || kt[tex]->rgba == NULL) return 0;
    if (flip & GW_KIT_FLIP_X) { u0 = 1; u1 = 0; }
    if (flip & GW_KIT_FLIP_Y) { v0 = 1; v1 = 0; }
    return kq_add(x, y, x + w, y + h, u0, v0, u1, v1, rgba, tex, shear, y + h * 0.5f);
}

int gw_Kit_DrawFlat(float x, float y, float w, float h, uint32_t rgba, float shear) {
    return kq_add(x, y, x + w, y + h, 0, 0, 0, 0, rgba, -1, shear, y + h * 0.5f);
}

int gw_Kit_DrawPanel(float x, float y, float w, float h, const char *prefix, const char *mod_ui,
                     float piece, uint32_t tint, uint32_t fill_rgba, float shear) {
    char nm[80];
    int tl, tr, bl, br, eh, ev, fill, added = 0;
    float c, eth = 0, etv = 0, sy = y + h * 0.5f;
    if (prefix == NULL || prefix[0] == '\0') prefix = "frame";
#define KP_TEX(var, suffix) (snprintf(nm, sizeof nm, "%s_%s", prefix, suffix), var = gw_Kit_Tex(nm, mod_ui))
    KP_TEX(tl, "corner_tl");
    KP_TEX(tr, "corner_tr");
    KP_TEX(bl, "corner_bl");
    KP_TEX(br, "corner_br");
    KP_TEX(eh, "edge_h");
    KP_TEX(ev, "edge_v");
    KP_TEX(fill, "fill");
#undef KP_TEX
    {
        /* the corner's 1x size; `piece` or a small panel scales the whole frame by c / corner */
        float w1 = 16, h1 = 16, corner, k;
        if (tl >= 0) gw_Kit_TexInfo(tl, NULL, NULL, &w1, &h1, NULL, NULL);
        corner = w1 > h1 ? w1 : h1;
        c = piece > 0 ? piece : corner;
        if (c > w * 0.5f) c = w * 0.5f;
        if (c > h * 0.5f) c = h * 0.5f;
        k = corner > 0 ? c / corner : 1.0f;
        /* an edge's thickness: its short side at 1x, scaled with the corners */
        if (eh >= 0) {
            float a, b;
            gw_Kit_TexInfo(eh, NULL, NULL, &a, &b, NULL, NULL);
            eth = b * k;
        }
        if (ev >= 0) {
            float a, b;
            gw_Kit_TexInfo(ev, NULL, NULL, &a, &b, NULL, NULL);
            etv = a * k;
        }
    }
    if (eth > c) eth = c;
    if (etv > c) etv = c;
    /* fill: under the frame */
    if ((fill_rgba & 0xFF) != 0) {
        if (fill >= 0) added += kq_add(x, y, x + w, y + h, 0, 0, 1, 1, fill_rgba, fill, shear, sy);
        else added += kq_add(x, y, x + w, y + h, 0, 0, 0, 0, fill_rgba, -1, shear, sy);
    }
    if (eh >= 0 && w > 2 * c) {
        added += kq_add(x + c, y, x + w - c, y + eth, 0, 0, 1, 1, tint, eh, shear, sy);
        added += kq_add(x + c, y + h - eth, x + w - c, y + h, 0, 1, 1, 0, tint, eh, shear, sy); /* flipped V */
    }
    if (ev >= 0 && h > 2 * c) {
        added += kq_add(x, y + c, x + etv, y + h - c, 0, 0, 1, 1, tint, ev, shear, sy);
        added += kq_add(x + w - etv, y + c, x + w, y + h - c, 1, 0, 0, 1, tint, ev, shear, sy); /* flipped H */
    }
    if (tl >= 0) added += kq_add(x, y, x + c, y + c, 0, 0, 1, 1, tint, tl, shear, sy);
    if (tr >= 0) added += kq_add(x + w - c, y, x + w, y + c, 0, 0, 1, 1, tint, tr, shear, sy);
    if (bl >= 0) added += kq_add(x, y + h - c, x + c, y + h, 0, 0, 1, 1, tint, bl, shear, sy);
    if (br >= 0) added += kq_add(x + w - c, y + h - c, x + w, y + h, 0, 0, 1, 1, tint, br, shear, sy);
    return added;
}

static uint32_t kit_tok(const char *tok, int section, uint32_t def) {
    uint32_t c;
    if (tok == NULL || tok[0] == '\0') return def;
    if (tok[0] == '@' && section > 0 && section < KIT_SECTIONS) {
        int w = strcmp(tok, "@face") == 0 ? 0 : strcmp(tok, "@bg") == 0 ? 1 : strcmp(tok, "@band") == 0 ? 2 : 3;
        return kf.sec[section][w];
    }
    return gw_Kit_Colour(tok, NULL, &c) ? c : def;
}

int gw_Kit_DrawRow(float x, float y, float w, float h, const char *label, const char *value,
                   int state, int section, float shear) {
    int added = 0, role;
    float lx, ly, sy;
    uint32_t face, lab, plate, val;
    kf_load();
    if (kf.state != 1) return 0;
    if (state < 0 || state > 2) state = 0;
    if (h <= 0) h = kf.row_h;
    face = kit_tok(kf.st_face[state], section, 0x1E3A8CFFu);
    lab = kit_tok(kf.st_label[state], section, 0xF2EFE4FFu);
    plate = kit_tok(kf.st_plate[state], section, 0);
    val = kit_tok(kf.st_value[state], section, lab);
    sy = y + h * 0.5f;
    if (state == GW_KIT_ROW_SEL && kf.st_plate[state][0] != '\0') {
        added += kq_add(x, y, x + w, y + h, 0, 0, 0, 0, plate, -1, shear, sy);
    }
    if (state == GW_KIT_ROW_SEL) {
        x += kf.lift[0];
        y += kf.lift[1];
        sy += kf.lift[1];
    }
    added += kq_add(x, y, x + w, y + h, 0, 0, 0, 0, face, -1, shear, sy);
    role = gw_Kit_Role(kf.row_role);
    if (role < 0) role = 0;
    lx = x + kf.label_x;
    ly = y + kf.label_base * (h / kf.row_h);
    if (value != NULL && value[0] != '\0') {
        float vw = 0;
        added += gw_Kit_DrawText(x + w - kf.label_x, ly, value, role, val, GW_KIT_ALIGN_RIGHT, w * 0.45f, shear, &vw);
        if (label != NULL) added += gw_Kit_DrawText(lx, ly, label, role, lab, GW_KIT_ALIGN_LEFT, w - 3 * kf.label_x - vw, shear, NULL);
    } else if (label != NULL) {
        added += gw_Kit_DrawText(lx, ly, label, role, lab, GW_KIT_ALIGN_LEFT, w - 2 * kf.label_x, shear, NULL);
    }
    return added;
}

/* ============================================================================================
 * tests
 * ============================================================================================ */
static int test_kit_decode_formats(void) {
    /* one 8x8 I4 tile: texel (x, y) = x + y (low bits) */
    uint8_t i4[32], rgb5a3[32], rgba8[64], ci8[32], tl[4];
    uint8_t *o;
    int x, y, bad = 0;
    for (y = 0; y < 8; y++)
        for (x = 0; x < 8; x += 2) i4[y * 4 + x / 2] = (uint8_t)((((x + y) & 15) << 4) | ((x + 1 + y) & 15));
    o = kt_decode(i4, sizeof i4, 0, 8, 8, NULL, 0, 0);
    for (y = 0; o != NULL && y < 8; y++)
        for (x = 0; x < 8; x++) {
            const uint8_t *p = o + (y * 8 + x) * 4;
            if (p[0] != 255 || p[3] != ((x + y) & 15) * 17) bad = 1;
        }
    free(o);
    if (bad || o == NULL) {
        gw_test_fail("I4 did not decode as a white mask with the texel as alpha");
        return 1;
    }
    /* RGB5A3: an opaque pure red texel, then a half-transparent (3-bit 4/7) blue one */
    memset(rgb5a3, 0, sizeof rgb5a3);
    rgb5a3[0] = 0xFC; rgb5a3[1] = 0x00;       /* 1 11111 00000 00000 */
    rgb5a3[2] = 0x40; rgb5a3[3] = 0x0F;       /* 0 100 0000 0000 1111 */
    o = kt_decode(rgb5a3, sizeof rgb5a3, 5, 4, 4, NULL, 0, 0);
    if (o == NULL || o[0] != 255 || o[1] != 0 || o[3] != 255 || o[4 + 2] != 255 || o[4 + 3] != 4 * 255 / 7) {
        gw_test_fail("RGB5A3 decode wrong");
        free(o);
        return 1;
    }
    free(o);
    /* RGBA8: texel 0 = (r 10, g 20, b 30, a 40) */
    memset(rgba8, 0, sizeof rgba8);
    rgba8[0] = 40; rgba8[1] = 10; rgba8[32] = 20; rgba8[33] = 30;
    o = kt_decode(rgba8, sizeof rgba8, 6, 4, 4, NULL, 0, 0);
    if (o == NULL || o[0] != 10 || o[1] != 20 || o[2] != 30 || o[3] != 40) {
        gw_test_fail("RGBA8 decode wrong");
        free(o);
        return 1;
    }
    free(o);
    /* CI8 with an RGB5A3 palette: index 1 = opaque white */
    memset(ci8, 1, sizeof ci8);
    tl[0] = 0; tl[1] = 0; tl[2] = 0xFF; tl[3] = 0xFF;
    o = kt_decode(ci8, sizeof ci8, 9, 8, 4, tl, 2, 2);
    if (o == NULL || o[0] != 255 || o[3] != 255) {
        gw_test_fail("CI8 decode wrong");
        free(o);
        return 1;
    }
    free(o);
    /* a short image is refused, not over-read */
    if (kt_decode(i4, 16, 0, 8, 8, NULL, 0, 0) != NULL) {
        gw_test_fail("a short I4 image was decoded");
        return 1;
    }
    return 0;
}

static int test_kit_json_and_colours(void) {
    char text[] = "{\"a\": [1, 2.5, {\"b\": \"x\\u2026\"}], \"c\": true}";
    KjDoc d;
    const KjNode *R = kj_parse(&d, text, strlen(text));
    const KjNode *b;
    uint32_t c = 0;
    int rc = 0;
    if (R == NULL || kj_num(kj_idx(&d, kj_get(&d, R, "a"), 1), 0) != 2.5 ||
        (b = kj_get(&d, kj_idx(&d, kj_get(&d, R, "a"), 2), "b")) == NULL ||
        strcmp(kj_str(b), "x\xE2\x80\xA6") != 0 || kj_get(&d, R, "c")->type != KJ_TRUE) {
        gw_test_fail("the kit's JSON reader misread a document");
        rc = 1;
    }
    kj_free(&d);
    if (!gw_Kit_Colour("#102030", NULL, &c) || c != 0x102030FFu || !gw_Kit_Colour("#10203040", NULL, &c) ||
        c != 0x10203040u || gw_Kit_Colour("#12", NULL, &c)) {
        gw_test_fail("hex colour tokens");
        rc = 1;
    }
    return rc;
}

/* With the kit's files present (a run sandbox finds _build/ui): text sets from the manifest. */
static int test_kit_text_layout(void) {
    int role, n, i;
    float w = 0, w2, size = 0;
    uint32_t bone = 0;
    char fit[64];
    if (!gw_Kit_Available()) {
        return 0; /* no UI directory beside this exe: nothing to check (the unit tests still ran) */
    }
    role = gw_Kit_Role("row");
    if (role < 0 || !gw_Kit_RoleMetrics(role, &size, NULL, NULL, NULL, NULL) || size <= 0) {
        gw_test_fail("the kit has no 'row' role");
        return 1;
    }
    if (!gw_Kit_Colour("bone", NULL, &bone) || !gw_Kit_Colour("@face", NULL, &bone) ||
        !gw_Kit_Colour("p1", NULL, &bone) || !gw_Kit_Colour("solo.bg", NULL, &bone)) {
        gw_test_fail("kit palette / section / port tokens did not resolve");
        return 1;
    }
    gw_Kit_BeginFrame();
    n = gw_Kit_DrawText(100, 200, "AV A", role, 0xFFFFFFFFu, GW_KIT_ALIGN_LEFT, 0, 0, &w);
    if (n != 3 || gw_Kit_QuadCount() != 3 || w <= 0) {
        gw_test_fail("\"AV A\" set %d quads (want 3: the space has none)", n);
        return 1;
    }
    for (i = 0; i < n; i++) {
        const GwKitQuad *q = gw_Kit_QuadAt(i);
        /* glyphs sit on the baseline: offsets reach a little left of the pen and up by the ascent */
        if (q->tex < 0 || q->x[0] < 100 - size * 0.5f || q->y[0] < 200 - size * 1.5f || q->y[2] > 200 + size) {
            gw_test_fail("glyph quad %d misplaced or untextured (tex %d, x %.1f, y %.1f..%.1f)", i, q->tex,
                         q->x[0], q->y[0], q->y[2]);
            return 1;
        }
    }
    /* the shear moves each corner by (baseline - y) * s: the top edge right, the baseline not */
    gw_Kit_BeginFrame();
    gw_Kit_DrawText(100, 200, "I", role, 0xFFFFFFFFu, GW_KIT_ALIGN_LEFT, 0, 0.25f, NULL);
    {
        const GwKitQuad *q = gw_Kit_QuadAt(0);
        float want = (200 - q->y[0]) * 0.25f;
        if (q == NULL || fabsf((q->x[1] - q->x[2]) - (want - (200 - q->y[2]) * 0.25f)) > 0.01f) {
            gw_test_fail("text shear is not about the baseline");
            return 1;
        }
    }
    /* right alignment ends at x; the fit rule truncates with the ellipsis */
    w2 = gw_Kit_TextWidth(role, "Stock Time Limit (min)");
    gw_Kit_Fit(role, "Stock Time Limit (min)", w2 * 0.5f, fit, sizeof fit);
    if (strlen(fit) >= strlen("Stock Time Limit (min)") || strchr(fit, 0x01) == NULL) {
        gw_test_fail("the fit rule did not truncate with an ellipsis");
        return 1;
    }
    /* caps roles uppercase: "abc" in hero is as wide as "ABC" */
    i = gw_Kit_Role("hero");
    if (i >= 0 && fabsf(gw_Kit_TextWidth(i, "abc") - gw_Kit_TextWidth(i, "ABC")) > 0.01f) {
        gw_test_fail("the hero role is not caps-only");
        return 1;
    }
    gw_Kit_BeginFrame();
    return 0;
}

static int test_kit_panel_and_row(void) {
    int n, tex, mask = 0;
    float w1 = 0, h1 = 0;
    if (!gw_Kit_Available()) {
        return 0;
    }
    /* the art pack's 9-slice: four corners, both edges twice (flipped), a flat fill */
    gw_Kit_BeginFrame();
    n = gw_Kit_DrawPanel(100, 100, 300, 200, "frame", NULL, 0, 0xFFFFFFFFu, 0x032568E0u, 0);
    if (gw_Kit_Tex("frame_corner_tl", NULL) >= 0) {
        const GwKitQuad *bottom;
        if (n != 9) {
            gw_test_fail("a frame panel drew %d quads (want 9)", n);
            return 1;
        }
        bottom = gw_Kit_QuadAt(2); /* fill, top edge, bottom edge */
        if (bottom->v[0] != 1.0f || bottom->v[2] != 0.0f) {
            gw_test_fail("the bottom edge is not flipped vertically");
            return 1;
        }
        if (gw_Kit_QuadAt(4)->u[0] != 1.0f) {
            gw_test_fail("the right edge is not flipped horizontally");
            return 1;
        }
    }
    /* icons: ico_<name>, a mask at half its texel size */
    tex = gw_Kit_Tex("ico_lock", NULL);
    if (tex >= 0 && (!gw_Kit_TexInfo(tex, NULL, NULL, &w1, &h1, &mask, NULL) || !mask || w1 <= 0)) {
        gw_test_fail("ico_lock is not an I4 mask with a 1x size");
        return 1;
    }
    /* a selected row: plate, lifted face, label */
    gw_Kit_BeginFrame();
    n = gw_Kit_DrawRow(96, 84, 444, 0, "Stocks", "4", GW_KIT_ROW_SEL, -1, 0);
    if (n < 4 || gw_Kit_QuadAt(0)->tex != -1 || gw_Kit_QuadAt(1)->x[0] != 96 - 3) {
        gw_test_fail("a selected row did not draw plate + lifted face (%d quads)", n);
        return 1;
    }
    gw_Kit_BeginFrame();
    return 0;
}

void gw_kit_tests_register(void) {
    gw_test_register("kit_decode_formats", test_kit_decode_formats);
    gw_test_register("kit_json_and_colours", test_kit_json_and_colours);
    gw_test_register("kit_text_layout", test_kit_text_layout);
    gw_test_register("kit_panel_and_row", test_kit_panel_and_row);
}
