/* gw_nucleus_install.h - what installing a Nucleus costume needs besides the network: reading a costume DAT's public symbols, PNG -> .gxtex, the
 * skin mod's mod.json, folder helpers, the installed-mod scan and mods/enabled.txt. Pure C over stdio / the OS directory calls; no game types.
 * Header-only (static), like gw_nucleus_core.h; one TU includes it (it carries the stb_image implementation). */
#ifndef GW_NUCLEUS_INSTALL_H
#define GW_NUCLEUS_INSTALL_H

#include "gw_nucleus_catalog.h"

#define STB_IMAGE_IMPLEMENTATION
#define STBI_ONLY_PNG
#define STBI_NO_STDIO
#define STBI_MAX_DIMENSIONS 2048
#include "../third_party/stb/stb_image.h"

/* ---- DAT public symbols ----------------------------------------------------------------------------------- */

typedef struct { char token[24], colour[8], joint[96], matanim[112]; int fighter, colour_idx; } nc_dat_info;

static uint32_t nc_be32(const uint8_t *p) { return ((uint32_t) p[0] << 24) | ((uint32_t) p[1] << 16) | ((uint32_t) p[2] << 8) | p[3]; }

/* The costume symbol Ply<Token>5K<Colour>_Share_joint of an HSD archive (a costume DAT is typed by this, not by its file name). 0 ok, -1 with err. */
static int nc_dat_costume(const uint8_t *raw, size_t n, nc_dat_info *out, char *err, size_t ecap) {
    uint32_t fsize, dsize, nreloc, npub, next;
    uint64_t o_pub, o_sym;
    uint32_t i;
    memset(out, 0, sizeof *out);
    out->fighter = out->colour_idx = -1;
    if (n < 0x20) { snprintf(err, ecap, "too small to be a DAT"); return -1; }
    fsize = nc_be32(raw); dsize = nc_be32(raw + 4); nreloc = nc_be32(raw + 8); npub = nc_be32(raw + 12); next = nc_be32(raw + 16);
    if (fsize < 0x20 || fsize > n || npub > 8192 || next > 8192 || nreloc > 1000000) { snprintf(err, ecap, "not an HSD archive"); return -1; }
    o_pub = 0x20 + (uint64_t) dsize + (uint64_t) nreloc * 4;
    o_sym = o_pub + (uint64_t) npub * 8 + (uint64_t) next * 8;
    if (o_sym > fsize) { snprintf(err, ecap, "archive tables run past the end"); return -1; }
    for (i = 0; i < npub; ++i) {
        uint32_t so = nc_be32(raw + o_pub + (uint64_t) i * 8 + 4);
        uint64_t at = o_sym + so, e;
        char sym[160];
        size_t len;
        const char *mid, *k5;
        if (at >= fsize) continue;
        for (e = at; e < fsize && e - at < sizeof sym - 1 && raw[e]; ++e) sym[e - at] = (char) raw[e];
        if (e >= fsize || raw[e]) continue;
        sym[e - at] = '\0';
        len = strlen(sym);
        if (len < 20 || strncmp(sym, "Ply", 3) != 0 || strcmp(sym + len - 12, "_Share_joint") != 0) continue;
        mid = sym + 3;
        k5 = strstr(mid + 1, "5K");
        if (!k5) continue;
        {
            size_t tl = (size_t) (k5 - mid), cl = (size_t) ((sym + len - 12) - (k5 + 2));
            uint32_t j;
            if (tl >= sizeof out->token || cl >= sizeof out->colour) continue;
            memcpy(out->token, mid, tl); out->token[tl] = '\0';
            memcpy(out->colour, k5 + 2, cl); out->colour[cl] = '\0';
            nc_copy(out->joint, sizeof out->joint, sym);
            /* the matching matanim, if the archive has it */
            for (j = 0; j < npub; ++j) {
                uint32_t so2 = nc_be32(raw + o_pub + (uint64_t) j * 8 + 4);
                uint64_t at2 = o_sym + so2;
                char want[160];
                size_t wl;
                snprintf(want, sizeof want, "Ply%s5K%s_Share_matanim_joint", out->token, out->colour);
                wl = strlen(want);
                if (at2 + wl < fsize && memcmp(raw + at2, want, wl + 1) == 0) { nc_copy(out->matanim, sizeof out->matanim, want); break; }
            }
            out->fighter = nc_fighter_by_token(out->token);
            out->colour_idx = out->fighter >= 0 ? nc_colour_index(out->fighter, out->colour) : -1;
            return 0;
        }
    }
    snprintf(err, ecap, "no costume symbol (Ply<Fighter>5K<Colour>_Share_joint) in it");
    return -1;
}

/* ---- PNG -> .gxtex ------------------------------------------------------------------------------------------ */

static uint16_t nc_rgb5a3(int r, int g, int b, int a) {
    int a3 = (a * 7 + 127) / 255;
    if (a3 == 7) return (uint16_t) (0x8000 | (((r * 31 + 127) / 255) << 10) | (((g * 31 + 127) / 255) << 5) | ((b * 31 + 127) / 255));
    return (uint16_t) ((a3 << 12) | (((r * 15 + 127) / 255) << 8) | (((g * 15 + 127) / 255) << 4) | ((b * 15 + 127) / 255));
}

static void nc_put32(uint8_t *p, uint32_t v) { p[0] = (uint8_t) (v >> 24); p[1] = (uint8_t) (v >> 16); p[2] = (uint8_t) (v >> 8); p[3] = (uint8_t) v; }

/* RGBA8 pixels -> a .gxtex v1 container (RGB5A3, 4x4 tiles; the size is padded up to a multiple of 4 with transparent texels), exactly what
 * pc/tools/png2gx.py --format rgb5a3 --allow-odd-size writes. The caller frees *out. */
static int nc_rgba_to_gxtex(const uint8_t *px, int w, int h, uint8_t **out, size_t *outlen) {
    int pw = (w + 3) & ~3, ph = (h + 3) & ~3, tx, ty, x, y;
    size_t isz, total;
    uint8_t *buf, *o;
    if (w < 1 || h < 1 || pw > 1024 || ph > 1024) return -1;
    isz = (size_t) pw * (size_t) ph * 2;
    total = 64 + isz;
    buf = (uint8_t *) calloc(1, total);
    if (!buf) return -1;
    memcpy(buf, "GXTX", 4);
    nc_put32(buf + 4, 1); nc_put32(buf + 8, 5); nc_put32(buf + 12, (uint32_t) pw); nc_put32(buf + 16, (uint32_t) ph);
    nc_put32(buf + 20, 0xFFFFFFFFu); nc_put32(buf + 24, 0); nc_put32(buf + 28, (uint32_t) isz); nc_put32(buf + 32, 0);
    nc_put32(buf + 36, 64); nc_put32(buf + 40, (uint32_t) (64 + isz));
    o = buf + 64;
    for (ty = 0; ty < ph; ty += 4) for (tx = 0; tx < pw; tx += 4) for (y = ty; y < ty + 4; ++y) for (x = tx; x < tx + 4; ++x) {
        uint16_t v = 0;
        if (x < w && y < h) { const uint8_t *p = px + ((size_t) y * (size_t) w + (size_t) x) * 4; v = nc_rgb5a3(p[0], p[1], p[2], p[3]); }
        *o++ = (uint8_t) (v >> 8); *o++ = (uint8_t) v;
    }
    *out = buf; *outlen = total;
    return 0;
}

/* a PNG file's bytes -> a .gxtex. Refuses anything but a PNG within 2048 x 2048. */
static int nc_png_to_gxtex(const uint8_t *png, size_t n, uint8_t **out, size_t *outlen, char *err, size_t ecap) {
    int w = 0, h = 0, comp = 0, rc;
    unsigned char *px;
    if (n < 8 || n > (size_t) 8 * 1024 * 1024 || memcmp(png, "\x89PNG", 4) != 0) { snprintf(err, ecap, "not a PNG"); return -1; }
    px = stbi_load_from_memory(png, (int) n, &w, &h, &comp, 4);
    if (!px) { snprintf(err, ecap, "PNG unreadable (%s)", stbi_failure_reason() ? stbi_failure_reason() : "?"); return -1; }
    if (w > 1024 || h > 1024) { stbi_image_free(px); snprintf(err, ecap, "PNG %dx%d is larger than 1024", w, h); return -1; }
    rc = nc_rgba_to_gxtex(px, w, h, out, outlen);
    stbi_image_free(px);
    if (rc < 0) snprintf(err, ecap, "texture conversion failed");
    return rc;
}

/* ---- zip (read only) -------------------------------------------------------------------------------------------
 * Some posts are only a zip ("Luffy Falco/Animelee/PlFcBu.dat" inside). Reads the central directory and pulls one entry out: stored or
 * deflated (inflate is stb_image's own zlib decoder, already vendored for the PNGs). No zip64, no encryption, no multi-disk. */

static uint32_t nz_u16(const uint8_t *p) { return (uint32_t) p[0] | ((uint32_t) p[1] << 8); }
static uint32_t nz_u32(const uint8_t *p) { return nz_u16(p) | (nz_u16(p + 2) << 16); }

static int nz_ieq(const char *a, size_t an, const char *b, size_t bn) {
    size_t i;
    if (an != bn) return 0;
    for (i = 0; i < an; ++i) if (tolower((unsigned char) a[i]) != tolower((unsigned char) b[i])) return 0;
    return 1;
}
static const char *nz_base(const char *p, size_t n, size_t *bn) {
    size_t i = n;
    while (i > 0 && p[i - 1] != '/' && p[i - 1] != '\\') --i;
    *bn = n - i;
    return p + i;
}

/* Extracts the entry named `want` (the API's filename; the full path first, then a unique match on the file name alone) into a malloc'd buffer. */
static int nc_zip_extract(const uint8_t *z, size_t zn, const char *want, uint8_t **out, size_t *outlen, size_t max_out, char *err, size_t ecap) {
    size_t eocd, i, cd, wl = strlen(want), wbn, found_hdr = 0;
    uint32_t count, k;
    int hits = 0;
    const char *wb = nz_base(want, wl, &wbn);
    *out = NULL; *outlen = 0;
    if (zn < 22 || zn > 0x7FFFFFFFu) { snprintf(err, ecap, "not a zip file"); return -1; }
    for (i = zn - 22, eocd = (size_t) -1; ; --i) {
        if (nz_u32(z + i) == 0x06054B50u) { eocd = i; break; }
        if (i == 0 || zn - i > 22 + 65535) break;
    }
    if (eocd == (size_t) -1) { snprintf(err, ecap, "not a zip file"); return -1; }
    count = nz_u16(z + eocd + 10);
    cd = nz_u32(z + eocd + 16);
    if (count == 0xFFFF || cd >= zn) { snprintf(err, ecap, "unsupported zip (zip64 or damaged)"); return -1; }
    for (k = 0, i = cd; k < count; ++k) {
        size_t nl, xl, cl;
        if (i + 46 > zn || nz_u32(z + i) != 0x02014B50u) { snprintf(err, ecap, "damaged zip directory"); return -1; }
        nl = nz_u16(z + i + 28); xl = nz_u16(z + i + 30); cl = nz_u16(z + i + 32);
        if (i + 46 + nl > zn) { snprintf(err, ecap, "damaged zip directory"); return -1; }
        {
            const char *nm = (const char *) z + i + 46;
            size_t bn;
            const char *bs = nz_base(nm, nl, &bn);
            if (nz_ieq(nm, nl, want, wl)) { found_hdr = i; hits = 1; break; }
            if (bn && nz_ieq(bs, bn, wb, wbn)) { if (!hits) found_hdr = i; ++hits; }
        }
        i += 46 + nl + xl + cl;
    }
    if (!hits) { snprintf(err, ecap, "%s is not in the zip", want); return -1; }
    if (hits > 1) { snprintf(err, ecap, "%s matches several files in the zip", want); return -1; }
    {
        size_t h = found_hdr, nl = nz_u16(z + h + 28), xl;
        uint32_t method = nz_u16(z + h + 10), csz = nz_u32(z + h + 20), usz = nz_u32(z + h + 24), lho = nz_u32(z + h + 42), flags = nz_u16(z + h + 8);
        size_t data;
        (void) nl;
        if (flags & 1u) { snprintf(err, ecap, "the zip is encrypted"); return -1; }
        if (usz == 0xFFFFFFFFu || csz == 0xFFFFFFFFu) { snprintf(err, ecap, "unsupported zip (zip64)"); return -1; }
        if (usz > max_out) { snprintf(err, ecap, "file too large (%u bytes)", (unsigned) usz); return -1; }
        if ((size_t) lho + 30 > zn || nz_u32(z + lho) != 0x04034B50u) { snprintf(err, ecap, "damaged zip entry"); return -1; }
        nl = nz_u16(z + lho + 26); xl = nz_u16(z + lho + 28);
        data = (size_t) lho + 30 + nl + xl;
        if (data > zn || csz > zn - data) { snprintf(err, ecap, "damaged zip entry"); return -1; }
        if (method == 0) {
            uint8_t *b = (uint8_t *) malloc(usz ? usz : 1);
            if (!b || csz != usz) { free(b); snprintf(err, ecap, "damaged zip entry"); return -1; }
            memcpy(b, z + data, usz);
            *out = b; *outlen = usz;
            return 0;
        } else if (method == 8) {
            int ol = 0;
            char *b = stbi_zlib_decode_noheader_malloc((const char *) z + data, (int) csz, &ol);
            if (!b || (uint32_t) ol != usz) { if (b) stbi_image_free(b); snprintf(err, ecap, "could not unpack the file"); return -1; }
            *out = (uint8_t *) b; *outlen = (size_t) ol;
            return 0;
        }
        snprintf(err, ecap, "unsupported zip compression (%u)", (unsigned) method);
        return -1;
    }
}

/* ---- files and folders ---------------------------------------------------------------------------------------- */

static void nc_fix_slashes(char *p) {
#ifdef _WIN32
    for (; *p; ++p) if (*p == '/') *p = '\\';
#else
    for (; *p; ++p) if (*p == '\\') *p = '/';
#endif
}

static int nc_is_dir(const char *path) {
#ifdef _WIN32
    DWORD a = GetFileAttributesA(path);
    return a != INVALID_FILE_ATTRIBUTES && (a & FILE_ATTRIBUTE_DIRECTORY);
#else
    struct stat st;
    return stat(path, &st) == 0 && S_ISDIR(st.st_mode);
#endif
}

static int nc_mkdir1(const char *path) {
#ifdef _WIN32
    return CreateDirectoryA(path, NULL) ? 0 : (GetLastError() == ERROR_ALREADY_EXISTS ? 0 : -1);
#else
    return mkdir(path, 0777) == 0 || nc_is_dir(path) ? 0 : -1;
#endif
}

/* mkdir -p */
static int nc_mkdirs(const char *path) {
    char buf[600], *p;
    snprintf(buf, sizeof buf, "%s", path);
    nc_fix_slashes(buf);
    for (p = buf + 1; *p; ++p) {
        if (*p == '\\' || *p == '/') {
            char c = *p;
            *p = '\0';
            if (buf[0] && !(p - buf == 2 && buf[1] == ':')) nc_mkdir1(buf);
            *p = c;
        }
    }
    return nc_mkdir1(buf);
}

/* delete a folder and everything under it (only ever a folder this browser made) */
static void nc_rmtree(const char *path) {
#ifdef _WIN32
    char pat[600];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    snprintf(pat, sizeof pat, "%s\\*", path);
    h = FindFirstFileA(pat, &fd);
    if (h != INVALID_HANDLE_VALUE) {
        do {
            char sub[600];
            if (!strcmp(fd.cFileName, ".") || !strcmp(fd.cFileName, "..")) continue;
            snprintf(sub, sizeof sub, "%s\\%s", path, fd.cFileName);
            if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) nc_rmtree(sub);
            else { SetFileAttributesA(sub, FILE_ATTRIBUTE_NORMAL); DeleteFileA(sub); }
        } while (FindNextFileA(h, &fd));
        FindClose(h);
    }
    RemoveDirectoryA(path);
#else
    DIR *d = opendir(path);
    struct dirent *e;
    if (d) {
        while ((e = readdir(d)) != NULL) {
            char sub[600];
            if (!strcmp(e->d_name, ".") || !strcmp(e->d_name, "..")) continue;
            snprintf(sub, sizeof sub, "%s/%s", path, e->d_name);
            if (nc_is_dir(sub)) nc_rmtree(sub); else remove(sub);
        }
        closedir(d);
    }
    rmdir(path);
#endif
}

static int nc_write_file(const char *path, const void *data, size_t n) {
    FILE *fp = fopen(path, "wb");
    if (!fp) return -1;
    if (n && fwrite(data, 1, n, fp) != n) { fclose(fp); return -1; }
    return fclose(fp) == 0 ? 0 : -1;
}

/* Move a finished folder into place. An existing target is first renamed aside (to .staging-old-<name> beside it: a dot folder the mods scanner skips and nq_clean_staging removes) and deleted only
 * after the new folder is in place; if the final rename fails the old one is put back, so a crash or a failure never leaves the mod missing. 0 ok. */
static int nc_swap_dir(const char *from, const char *to) {
    char old[620];
    int had = nc_is_dir(to);
    const char *base = to + strlen(to);
    while (base > to && base[-1] != '/' && base[-1] != '\\') --base;
    snprintf(old, sizeof old, "%.*s.staging-old-%s", (int) (base - to), to, base);
    if (had) {
        if (nc_is_dir(old)) nc_rmtree(old);
        if (rename(to, old) != 0) return -1;
    }
    if (rename(from, to) != 0) {
        if (had) rename(old, to);
        return -1;
    }
    if (had) nc_rmtree(old);
    return 0;
}

/* ---- the skin mod ---------------------------------------------------------------------------------------------- */

typedef struct {
    char name[24];                        /* printable ASCII, 1..23 */
    char dat[160], csp[160], stock[160];  /* paths inside files/ ("skins/<id>/1.dat"); csp / stock "" when there is no art */
    char joint[96], matanim[112];
    int like, file_id;
} nc_skin_costume;

typedef struct {
    int nucleus_id;
    char mod_id[64], title[80], author[40], page[120], updated[24], installed_at[24], target[24];
    int n;
    nc_skin_costume c[NC_MAX_FILES_PER_MOD];
} nc_skin_mod;

/* the id of the skin mod for (post, fighter): lower case, [a-z0-9-] */
static void nc_mod_id(int post, int fighter, char *out, size_t cap) { snprintf(out, cap, "nucleus-%d-%s", post, nc_fighters[fighter].target); }

/* A costume's display name: the post's title, plus its colour when the post has several, as printable ASCII of at most 23 characters. */
static void nc_costume_name(const char *title, const char *colour, int several, char *out, size_t cap) {
    char tmp[128];
    size_t i, o = 0;
    snprintf(tmp, sizeof tmp, "%s%s%s", title, several && colour && *colour ? " " : "", several && colour ? colour : "");
    for (i = 0; tmp[i] && o + 1 < cap && o < 23; ++i) if ((unsigned char) tmp[i] >= 32 && (unsigned char) tmp[i] < 127) out[o++] = tmp[i];
    out[o] = '\0';
    while (o && out[o - 1] == ' ') out[--o] = '\0';
    if (!o) snprintf(out, cap, "Nucleus");
}

/* mod.json for the skin mod: the kind "skin" format of docs/mods-packaging.md 4b, plus the source block ("source": "nucleus", the post, the author, the
 * link back, updated_at). Unknown top-level keys are ignored by the mods loader; the skin block is exactly what the skin reader accepts. */
static void nc_skin_json(const nc_skin_mod *s, nj_buf *b) {
    int i;
    char desc[200];
    snprintf(desc, sizeof desc, "By %s. From SSBM Nucleus: %s/post/%d", s->author[0] ? s->author : "unknown", NC_SITE, s->nucleus_id);
    nj_puts(b, "{\n  \"id\": "); nj_qstr(b, s->mod_id);
    nj_puts(b, ",\n  \"name\": "); nj_qstr(b, s->title);
    nj_puts(b, ",\n  \"version\": "); nj_qstr(b, s->updated);
    nj_puts(b, ",\n  \"kind\": \"skin\",\n  \"authors\": "); nj_qstr(b, s->author);
    nj_puts(b, ",\n  \"description\": "); nj_qstr(b, desc);
    nj_puts(b, ",\n  \"source\": \"nucleus\",\n  \"nucleus\": {\n    \"id\": ");
    nj_printf(b, "%d,\n    \"author\": ", s->nucleus_id); nj_qstr(b, s->author);
    nj_puts(b, ",\n    \"page_url\": "); nj_qstr(b, s->page);
    nj_puts(b, ",\n    \"updated_at\": "); nj_qstr(b, s->updated);
    nj_puts(b, ",\n    \"installed_at\": "); nj_qstr(b, s->installed_at);
    nj_puts(b, ",\n    \"fighter\": "); nj_qstr(b, s->target);
    nj_puts(b, ",\n    \"files\": [");
    for (i = 0; i < s->n; ++i) nj_printf(b, "%s%d", i ? ", " : "", s->c[i].file_id);
    nj_puts(b, "]\n  },\n  \"skin\": {\n    \"format\": 1,\n    \"target\": { \"retail\": "); nj_qstr(b, s->target);
    nj_puts(b, " },\n    \"costumes\": [");
    for (i = 0; i < s->n; ++i) {
        const nc_skin_costume *c = &s->c[i];
        nj_puts(b, i ? ",\n      { \"name\": " : "\n      { \"name\": "); nj_qstr(b, c->name);
        nj_puts(b, ", \"file\": "); nj_qstr(b, c->dat);
        nj_puts(b, ", \"joint\": "); nj_qstr(b, c->joint);
        if (c->matanim[0]) { nj_puts(b, ", \"matanim\": "); nj_qstr(b, c->matanim); }
        if (c->like > 0) nj_printf(b, ", \"like\": %d", c->like);
        if (c->csp[0]) { nj_puts(b, ", \"csp\": "); nj_qstr(b, c->csp); }
        if (c->stock[0]) { nj_puts(b, ", \"stock\": "); nj_qstr(b, c->stock); }
        nj_puts(b, " }");
    }
    nj_puts(b, "\n    ]\n  }\n}\n");
}

/* ---- installed mods ------------------------------------------------------------------------------------------- */

typedef struct { char folder[64]; int id; char name[80], updated[24], author[40], target[24]; int ncost; } nc_inst;

/* read one mod.json of ours; 1 when it is a Nucleus skin */
static int nc_inst_parse(const char *folder, const char *text, nc_inst *out) {
    nj_doc d;
    int root = nj_parse(&d, text), nu, sk, co, e, ok = 0;
    memset(out, 0, sizeof *out);
    if (root >= 0 && !strcmp(nj_gstr(&d, root, "source", ""), "nucleus")) {
        nu = nj_get(&d, root, "nucleus");
        sk = nj_get(&d, root, "skin");
        nc_copy(out->folder, sizeof out->folder, folder);
        out->id = (int) nj_gnum(&d, nu, "id", 0);
        nc_copy(out->name, sizeof out->name, nj_gstr(&d, root, "name", folder));
        nc_copy(out->updated, sizeof out->updated, nj_gstr(&d, nu, "updated_at", ""));
        nc_copy(out->author, sizeof out->author, nj_gstr(&d, nu, "author", ""));
        nc_copy(out->target, sizeof out->target, nj_gstr(&d, nu, "fighter", ""));
        co = nj_get(&d, sk, "costumes");
        if (co >= 0) for (e = d.n[co].first; e >= 0; e = d.n[e].next) ++out->ncost;
        ok = out->id > 0;
    }
    nj_free(&d);
    return ok;
}

/* The Nucleus skins in a mods folder (folders named nucleus-*), at most cap. */
static int nc_inst_scan(const char *mods_dir, nc_inst *out, int cap) {
    int n = 0;
#ifdef _WIN32
    char pat[600];
    WIN32_FIND_DATAA fd;
    HANDLE h;
    snprintf(pat, sizeof pat, "%s\\nucleus-*", mods_dir);
    h = FindFirstFileA(pat, &fd);
    if (h == INVALID_HANDLE_VALUE) return 0;
    do {
        char path[700];
        char *text;
        if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) || n >= cap) continue;
        snprintf(path, sizeof path, "%s\\%s\\mod.json", mods_dir, fd.cFileName);
        text = nc_read_all(path, NULL);
        if (text) { if (nc_inst_parse(fd.cFileName, text, &out[n])) ++n; free(text); }
    } while (FindNextFileA(h, &fd));
    FindClose(h);
#else
    DIR *d = opendir(mods_dir);
    struct dirent *e;
    if (!d) return 0;
    while ((e = readdir(d)) != NULL && n < cap) {
        char path[700];
        char *text;
        if (strncmp(e->d_name, "nucleus-", 8) != 0) continue;
        snprintf(path, sizeof path, "%s/%s/mod.json", mods_dir, e->d_name);
        text = nc_read_all(path, NULL);
        if (text) { if (nc_inst_parse(e->d_name, text, &out[n])) ++n; free(text); }
    }
    closedir(d);
#endif
    return n;
}

/* mods/enabled.txt: when the file exists, a new mod is enabled by listing it; when it does not, every mod is on and nothing is written. */
static int nc_enabled_edit(const char *mods_dir, const char *id, int add) {
    char path[600], *text, *out;
    size_t n = 0, idl = strlen(id), o = 0;
    const char *p;
    int found = 0, rc;
    snprintf(path, sizeof path, "%s/enabled.txt", mods_dir);
    nc_fix_slashes(path);
    text = nc_read_all(path, &n);
    if (!text) return 0;
    out = (char *) malloc(n + idl + 4);
    if (!out) { free(text); return -1; }
    for (p = text; *p; ) {
        const char *e = strchr(p, '\n');
        size_t ll = e ? (size_t) (e - p) : strlen(p), tl = ll;
        while (tl && (p[tl - 1] == '\r' || p[tl - 1] == ' ')) --tl;
        if (tl == idl && !nc_ieqn(p, id, idl)) { /* equal: handled below */ }
        if (tl == idl && nc_ieqn(p, id, idl)) { found = 1; if (!add) { p = e ? e + 1 : p + ll; continue; } }
        memcpy(out + o, p, ll); o += ll; out[o++] = '\n';
        p = e ? e + 1 : p + ll;
    }
    if (add && !found) { memcpy(out + o, id, idl); o += idl; out[o++] = '\n'; }
    rc = nc_write_file(path, out, o);
    free(out); free(text);
    return rc;
}

#endif
