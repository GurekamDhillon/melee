/* gw_mex_modstage.inc - custom stages that arrive as MODS on a disc with NO MxDt.dat (a vanilla disc).
 * Included at the end of gw_mex_grfunction.c (one TU; no new link entry). Native platform code.
 *
 * WHY. m-ex adds a stage by rows in MxDt.dat (Arch_Map + Arch_grFunction). A vanilla disc has none, and the
 * "filtered base" model (docs/mods-packaging.md section 3) only combines stages within one pack's baked MxDt.
 * A Nucleus custom-stage post carries the stage's own data (name, file, clone-base function addresses, sound
 * bank, playlist), so the runtime BUILDS the few m-ex stage tables from the enabled stage mods at boot:
 *
 *   mods/<id>/mexstage.txt   key=value lines (written by the Nucleus installer, see gw_nucleus_stage.h)
 *   mods/<id>/files/GrXx.dat the stage file (overlaid on the disc as /GrXx.dat like any mod file)
 *
 * WHAT IT BUILDS. A guest-memory image shaped like the part of mexData the stage readers use: root +0x00 metadata
 * (internal / external stage counts), +0x28 Arch_Map (StageIDs, Audio, LineTypeData, Playlists), +0x2C the
 * Arch_grFunction row pointers. Retail internal ids 0..70 keep the port's compiled-in rows (nothing below reads
 * them); mod stages are internal 71.. and external 288.. - the numbers ACE and Akaneia use for added stages, so every
 * downstream reader (ground.c rows, stage.c ids, the audio table, the playlist draw) works unchanged. Only the stage
 * readers see this image: gw_Mex_MexData() stays 0, so fighters, items, CSS, SSS and the rest still behave exactly
 * as on a vanilla disc. A disc that has a real MxDt.dat never uses it (merging into ACE's tables is not supported).
 *
 * The image lives in the m-ex persistent region, which the test harness rewinds; gw_Mex_GrInvalidate() therefore
 * drops it and the next reader rebuilds it from the (host-side) descriptors. */

#include "gw_mods.h"

#define MS_MAX 24
#define MS_EXT_FIRST 288
#define MS_INT_FIRST GW_MEX_GR_FIRST_NEW
#define MS_BGM_MAX 8
#define MS_NO_BANK 55 /* retail's empty bank */

typedef struct ms_stage {
    char mod[64], name[48], file[40], icon[64], post[96];
    uint32_t w[GW_MEX_GR_SLOT_COUNT]; /* the StageData row, guest words */
    int ssm, reverb, reverb2, linetype, nbgm;
    int bgm[MS_BGM_MAX], chance[MS_BGM_MAX];
    char dir[260];
} ms_stage;

static ms_stage ms_list[MS_MAX];
static int ms_n, ms_scanned;
static uint32_t ms_root, ms_base, ms_size; /* the built image; 0 = not built */

static uint32_t ms_hex(const char *s) { return (uint32_t) strtoul(s, NULL, 16); }

/* Parse one mexstage.txt body. Returns 1 when it names a stage (a name and a /Gr file). */
static int ms_parse(const char *text, ms_stage *o) {
    const char *p = text;
    memset(o, 0, sizeof *o);
    o->ssm = MS_NO_BANK;
    o->reverb = o->reverb2 = 1;
    o->linetype = -1;
    o->w[GW_MEX_GR_SLOT_FLAGS2] = 0;
    while (*p) {
        char line[256], *eq;
        size_t n = 0;
        while (p[n] && p[n] != '\n' && p[n] != '\r') ++n;
        if (n >= sizeof line) n = sizeof line - 1;
        memcpy(line, p, n);
        line[n] = '\0';
        p += n;
        while (*p == '\n' || *p == '\r') ++p;
        eq = strchr(line, '=');
        if (eq == NULL || line[0] == '#') continue;
        *eq++ = '\0';
        if (!strcmp(line, "name")) snprintf(o->name, sizeof o->name, "%s", eq);
        else if (!strcmp(line, "file")) snprintf(o->file, sizeof o->file, "%s", eq);
        else if (!strcmp(line, "icon")) snprintf(o->icon, sizeof o->icon, "%s", eq);
        else if (!strcmp(line, "post")) snprintf(o->post, sizeof o->post, "%s", eq);
        else if (!strcmp(line, "ssm")) o->ssm = atoi(eq);
        else if (!strcmp(line, "reverb")) o->reverb = atoi(eq);
        else if (!strcmp(line, "reverb2")) o->reverb2 = atoi(eq);
        else if (!strcmp(line, "linetype")) o->linetype = atoi(eq);
        else if (!strcmp(line, "flags2")) o->w[GW_MEX_GR_SLOT_FLAGS2] = ms_hex(eq);
        else if (!strcmp(line, "callbacks")) o->w[GW_MEX_GR_SLOT_CALLBACKS] = ms_hex(eq);
        else if (line[0] == 'w' && line[1] >= '3' && line[1] <= '9' && line[2] == '\0') o->w[line[1] - '0'] = ms_hex(eq);
        else if (!strcmp(line, "bgm")) {
            const char *q = eq;
            while (*q && o->nbgm < MS_BGM_MAX) {
                int id = (int) strtol(q, (char **) &q, 10), ch = 100;
                if (*q == ':') ch = (int) strtol(q + 1, (char **) &q, 10);
                if (id > 0 && id <= 98) { o->bgm[o->nbgm] = id; o->chance[o->nbgm] = ch > 0 ? ch : 100; ++o->nbgm; }
                while (*q && *q != ',') ++q;
                if (*q == ',') ++q;
            }
        }
    }
    if (o->name[0] == '\0' || o->file[0] != '/' || strlen(o->file) < 6 || strncmp(o->file, "/Gr", 3) != 0) return 0;
    return 1;
}

static void ms_scan(void) {
    int n, k;
    if (ms_scanned) return;
    ms_scanned = 1;
    ms_n = 0;
    for (n = 0; n < gw_Mods_ActiveCount() && ms_n < MS_MAX; ++n) {
        int i = gw_Mods_ActiveAt(n);
        char dir[260], path[300];
        FILE *f;
        char text[2048];
        size_t got, len;
        if (strcmp(gw_Mods_Kind(i), "stage") != 0) continue;
        snprintf(dir, sizeof dir, "%s", gw_Mods_PayloadDir(i));
        if (!gw_Mods_PayloadIsModDir(i)) {
            len = strlen(dir);
            while (len > 0 && (dir[len - 1] == '/' || dir[len - 1] == '\\')) dir[--len] = '\0';
            while (len > 0 && dir[len - 1] != '/' && dir[len - 1] != '\\') dir[--len] = '\0';
        }
        snprintf(path, sizeof path, "%smexstage.txt", dir);
        f = fopen(path, "rb");
        if (f == NULL) continue;
        got = fread(text, 1, sizeof text - 1, f);
        fclose(f);
        text[got] = '\0';
        if (!ms_parse(text, &ms_list[ms_n])) {
            gw_log("modstage: %s: mexstage.txt does not name a stage (needs name= and file=/GrXx.dat) - skipped", gw_Mods_Id(i));
            continue;
        }
        snprintf(ms_list[ms_n].mod, sizeof ms_list[ms_n].mod, "%s", gw_Mods_Id(i));
        snprintf(ms_list[ms_n].dir, sizeof ms_list[ms_n].dir, "%s", dir);
        ++ms_n;
    }
    for (k = 0; k < ms_n; ++k) {
        gw_log("modstage: stage %d (ext %d) \"%s\" %s from %s", MS_INT_FIRST + k, MS_EXT_FIRST + k, ms_list[k].name, ms_list[k].file, ms_list[k].mod);
    }
}

/* ---- the public view (host data only) ---- */
int gw_ModStage_Count(void) { uint32_t b, s; if (gw_Mex_MexData(&b, &s) != 0u) return 0; ms_scan(); return ms_n; }
int gw_ModStage_ExtAt(int i) { return i >= 0 && i < gw_ModStage_Count() ? MS_EXT_FIRST + i : -1; }
int gw_ModStage_IndexForExt(int ext) { int i = ext - MS_EXT_FIRST; return i >= 0 && i < gw_ModStage_Count() ? i : -1; }
const char *gw_ModStage_Name(int i) { return i >= 0 && i < gw_ModStage_Count() ? ms_list[i].name : ""; }
const char *gw_ModStage_File(int i) { return i >= 0 && i < gw_ModStage_Count() ? ms_list[i].file : ""; }
const char *gw_ModStage_Post(int i) { return i >= 0 && i < gw_ModStage_Count() ? ms_list[i].post : ""; }
const char *gw_ModStage_Mod(int i) { return i >= 0 && i < gw_ModStage_Count() ? ms_list[i].mod : ""; }
/* The icon's host path ("" when the mod has none). */
const char *gw_ModStage_Icon(int i) {
    static char path[340];
    if (i < 0 || i >= gw_ModStage_Count() || ms_list[i].icon[0] == '\0') return "";
    snprintf(path, sizeof path, "%s%s", ms_list[i].dir, ms_list[i].icon);
    return path;
}
/* The icon (.gxtex v1, RGB5A3, written by the installer): (w << 16 | h), or 0. */
static int ms_icon_read(int i, uint8_t *dst, int cap, int *w, int *h) {
    const char *path = gw_ModStage_Icon(i);
    uint8_t hdr[64];
    uint32_t iw, ih, fmt, isz;
    FILE *f;
    int ok = 0;
    *w = *h = 0;
    if (path[0] == '\0' || (f = fopen(path, "rb")) == NULL) return 0;
    if (fread(hdr, 1, 64, f) == 64 && !memcmp(hdr, "GXTX", 4)) {
        fmt = ((uint32_t) hdr[8] << 24) | ((uint32_t) hdr[9] << 16) | ((uint32_t) hdr[10] << 8) | hdr[11];
        iw = ((uint32_t) hdr[12] << 24) | ((uint32_t) hdr[13] << 16) | ((uint32_t) hdr[14] << 8) | hdr[15];
        ih = ((uint32_t) hdr[16] << 24) | ((uint32_t) hdr[17] << 16) | ((uint32_t) hdr[18] << 8) | hdr[19];
        isz = ((uint32_t) hdr[28] << 24) | ((uint32_t) hdr[29] << 16) | ((uint32_t) hdr[30] << 8) | hdr[31];
        if (fmt == 5 && iw >= 4 && ih >= 4 && iw <= 256 && ih <= 256 && isz == iw * ih * 2u) {
            *w = (int) iw; *h = (int) ih; ok = 1;
            if (dst != NULL) ok = cap >= (int) isz && fread(dst, 1, isz, f) == isz;
        }
    }
    fclose(f);
    return ok;
}
int gw_ModStage_IconInfo(int i) { int w, h; return ms_icon_read(i, NULL, 0, &w, &h) ? (w << 16) | h : 0; }
/* Copy the icon's pixels to guest memory (texture bytes are order-neutral). Returns the byte count, or 0. */
int gw_ModStage_IconFill(int i, void *dst, int cap) { int w, h; return dst != NULL && ms_icon_read(i, (uint8_t *) dst, cap, &w, &h) ? w * h * 2 : 0; }

/* Whether the audio / line-type tables are the virtual ones (graudio then leaves retail rows 0..70 alone). */
int gw_ModStage_AudioFirst(void) { uint32_t b, s; return gw_Mex_MexData(&b, &s) == 0u && gw_ModStage_Count() > 0 ? MS_INT_FIRST : 0; }

/* ---- the image ---- */
static void ms_w32(uint32_t at, uint32_t v) { gw_w32((void *) (uintptr_t) at, v); }

/* Build the image. Returns the root guest address, or 0 when there is nothing to build. */
uint32_t gw_ModStage_Root(uint32_t *base, uint32_t *size) {
    extern void *gw_Mex_PersistAlloc(uint32_t size);
    uint32_t b, s, off, root, meta, map, ids, audio, line, plt, ple, desc, rows, strs, total, at;
    int I, E, k, j, nent = 0;

    if (base) *base = 0u;
    if (size) *size = 0u;
    if (gw_Mex_MexData(&b, &s) != 0u) return 0u; /* a real MxDt.dat: never merge */
    ms_scan();
    if (ms_n <= 0) return 0u;
    if (ms_root != 0u) {
        if (base) *base = ms_base;
        if (size) *size = ms_size;
        return ms_root;
    }
    I = MS_INT_FIRST + ms_n;
    E = MS_EXT_FIRST + ms_n;
    for (k = 0; k < ms_n; ++k) nent += ms_list[k].nbgm;
    off = 0x40u;                         root = 0;
    meta = off;   off += 0x40u;
    map = off;    off += 0x20u;
    ids = off;    off += (uint32_t) E * 12u;
    audio = off;  off += ((uint32_t) I * 3u + 3u) & ~3u;
    line = off;   off += (uint32_t) I * 8u;
    plt = off;    off += (uint32_t) I * 8u;
    ple = off;    off += (uint32_t) (nent + 1) * 4u;
    desc = off;   off += (uint32_t) I * 4u;
    rows = off;   off += (uint32_t) ms_n * GW_MEX_GR_SLOT_COUNT * 4u;
    strs = off;   off += (uint32_t) ms_n * 40u;
    total = off;
    b = (uint32_t) (uintptr_t) gw_Mex_PersistAlloc(total);
    if (b == 0u) return 0u;
    (void) root;
    root = b;
    ms_w32(root + 0x00u, b + meta);
    ms_w32(root + 0x28u, b + map);
    ms_w32(root + 0x2Cu, b + desc);
    ms_w32(b + meta + 4u + 3u * 4u, (uint32_t) I);
    ms_w32(b + meta + 4u + 4u * 4u, (uint32_t) E);
    ms_w32(b + map + 0x00u, b + ids);
    ms_w32(b + map + 0x04u, b + audio);
    ms_w32(b + map + 0x08u, b + line);
    ms_w32(b + map + 0x14u, b + plt);
    for (j = 0; j < E; ++j) {
        int g = j >= MS_EXT_FIRST ? MS_INT_FIRST + (j - MS_EXT_FIRST) : -1;
        ms_w32(b + ids + (uint32_t) j * 12u, (uint32_t) g);
    }
    for (j = 0; j < I; ++j) {
        uint8_t *a = (uint8_t *) (uintptr_t) (b + audio + (uint32_t) j * 3u);
        a[0] = MS_NO_BANK; a[1] = 1; a[2] = 1;
        ms_w32(b + line + (uint32_t) j * 8u, (uint32_t) j);
    }
    at = ple;
    for (k = 0; k < ms_n; ++k) {
        ms_stage *m = &ms_list[k];
        uint32_t row = b + rows + (uint32_t) k * GW_MEX_GR_SLOT_COUNT * 4u, sp = b + strs + (uint32_t) k * 40u;
        uint8_t *a = (uint8_t *) (uintptr_t) (b + audio + (uint32_t) (MS_INT_FIRST + k) * 3u);
        a[0] = (uint8_t) m->ssm; a[1] = (uint8_t) m->reverb; a[2] = (uint8_t) m->reverb2;
        memcpy((void *) (uintptr_t) sp, m->file, strlen(m->file) + 1u);
        for (j = 0; j < GW_MEX_GR_SLOT_COUNT; ++j) ms_w32(row + (uint32_t) j * 4u, m->w[j]);
        ms_w32(row + GW_MEX_GR_SLOT_GRKIND * 4u, (uint32_t) (MS_INT_FIRST + k));
        ms_w32(row + GW_MEX_GR_SLOT_FILE * 4u, sp);
        ms_w32(b + desc + (uint32_t) (MS_INT_FIRST + k) * 4u, row);
        if (m->linetype >= 0 && m->linetype < MS_INT_FIRST) ms_w32(b + line + (uint32_t) (MS_INT_FIRST + k) * 8u, (uint32_t) m->linetype);
        if (m->nbgm > 0) {
            ms_w32(b + plt + (uint32_t) (MS_INT_FIRST + k) * 8u, (uint32_t) m->nbgm);
            ms_w32(b + plt + (uint32_t) (MS_INT_FIRST + k) * 8u + 4u, b + at);
            for (j = 0; j < m->nbgm; ++j, at += 4u) {
                uint8_t *e = (uint8_t *) (uintptr_t) (b + at);
                e[0] = (uint8_t) (m->bgm[j] >> 8); e[1] = (uint8_t) m->bgm[j];
                e[2] = (uint8_t) (m->chance[j] >> 8); e[3] = (uint8_t) m->chance[j];
            }
        }
    }
    ms_root = root;
    ms_base = b;
    ms_size = total;
    gw_log("modstage: built the stage tables for %d mod stage(s): %d internal / %d external, %u bytes @ 0x%08X", ms_n, I, E, total, b);
    if (base) *base = ms_base;
    if (size) *size = ms_size;
    return ms_root;
}

/* The vanilla line-type row a mod stage uses (its descriptor's `linetype=`), or -1 = the caller's fallback. */
int gw_ModStage_LineTypeRow(int grkind) {
    int i = grkind - MS_INT_FIRST;
    return i >= 0 && i < gw_ModStage_Count() ? ms_list[i].linetype : -1;
}

/* Drop the built image (the persistent region was rewound); descriptors stay. Called from gw_Mex_GrInvalidate. */
static void ms_invalidate(void) { ms_root = ms_base = ms_size = 0u; }

/* ---- tests ---- */
static int test_modstage_parse(void) {
    ms_stage s;
    const char *ok = "name=Test Stage\nfile=/GrTest.dat\nw3=80123456\nw9=80ABCDEF\nflags2=1\nssm=57\nbgm=3:50,4,200:9,98:10\nlinetype=31\n";
    if (!ms_parse(ok, &s)) { gw_test_fail("a complete mexstage.txt must parse"); return 1; }
    if (strcmp(s.name, "Test Stage") || strcmp(s.file, "/GrTest.dat")) { gw_test_fail("name/file wrong: %s %s", s.name, s.file); return 1; }
    if (s.w[3] != 0x80123456u || s.w[9] != 0x80ABCDEFu || s.w[10] != 1u) { gw_test_fail("words wrong: %08X %08X %u", s.w[3], s.w[9], s.w[10]); return 1; }
    if (s.ssm != 57 || s.linetype != 31) { gw_test_fail("ssm/linetype wrong: %d %d", s.ssm, s.linetype); return 1; }
    /* 200 is not a retail music id (> 98) and is dropped; 4 has the default chance */
    if (s.nbgm != 3 || s.bgm[0] != 3 || s.chance[0] != 50 || s.bgm[1] != 4 || s.chance[1] != 100 || s.bgm[2] != 98) { gw_test_fail("playlist wrong: n=%d", s.nbgm); return 1; }
    if (ms_parse("name=No file\n", &s) || ms_parse("file=/GrX.dat\n", &s) || ms_parse("name=x\nfile=/Pl.dat\n", &s) || ms_parse("", &s) || ms_parse("garbage\n=\n\xff\xfe", &s)) {
        gw_test_fail("a descriptor without a name, a /Gr file or with garbage must be refused");
        return 1;
    }
    return 0;
}

void gw_modstage_tests_register(void) { gw_test_register("modstage_parse", test_modstage_parse); }
