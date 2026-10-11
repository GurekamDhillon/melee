/* Shared archive gate for Nucleus downloads and local files. Entries are read into
 * bounded buffers, never extracted to their archive paths. Validate the entire
 * directory before selecting an entry (including ignored README/art entries). */
#ifndef GW_NUCLEUS_ZIP_H
#define GW_NUCLEUS_ZIP_H

#define NZ_MAX_ARCHIVE (160u * 1024 * 1024)
#define NZ_MAX_FILE (64u * 1024 * 1024)
#define NZ_MAX_TOTAL (200u * 1024 * 1024)
#define NZ_MAX_ENTRIES 2048

static int nz_safe_name(const char *s) {
    const char *p = s, *b = s;
    if (!*s || *s == '/' || *s == '\\') return 0;
    for (;;) {
        unsigned char c = (unsigned char) *p;
        if (c && (c < 32 || strchr(":*?\"<>|", c))) return 0;
        if (!c || c == '/' || c == '\\') {
            size_t n = (size_t) (p - b);
            if (!n || b[n - 1] == '.' || b[n - 1] == ' ') return 0;
            if (!c || !p[1]) return 1;
            b = p + 1;
        }
        ++p;
    }
}

static int nz_allowed(const char *name) {
    static const char *const exts[] = { ".dat", ".usd", ".json", ".zip", ".hps", ".ssm", ".png", ".jpg", ".jpeg", ".webp", ".bmp", ".tpl", ".gxtex", ".txt", ".md" };
    size_t i;
    for (i = 0; i < sizeof exts / sizeof exts[0]; ++i) if (nc_has_ext(name, exts[i])) return 1;
    return 0;
}

/* Returns file count or -1. When names is supplied it receives every file name. */
static int nc_zip_list(const uint8_t *z, size_t zn, char names[][260], int cap, char *err, size_t ecap) {
    size_t eocd, i, cd, end, total = 0;
    unsigned count, k;
    int n = 0;
    if (zn < 22) { snprintf(err, ecap, "not a zip file"); return -1; }
    if (zn > NZ_MAX_ARCHIVE) { snprintf(err, ecap, "zip too large (maximum 160 MiB)"); return -1; }
    for (i = zn - 22, eocd = (size_t) -1; ; --i) {
        if (nz_u32(z + i) == 0x06054B50u && i + 22 + nz_u16(z + i + 20) == zn) { eocd = i; break; }
        if (!i || zn - i > 22 + 65535) break;
    }
    if (eocd == (size_t) -1) { snprintf(err, ecap, "not a zip file (missing directory)"); return -1; }
    count = nz_u16(z + eocd + 10); cd = nz_u32(z + eocd + 16);
    if (nz_u16(z + eocd + 4) || nz_u16(z + eocd + 6) || nz_u16(z + eocd + 8) != count || count > NZ_MAX_ENTRIES) {
        snprintf(err, ecap, "unsupported zip (multi-disk, zip64 or too many entries)"); return -1;
    }
    if (cd > eocd || nz_u32(z + eocd + 12) != eocd - cd) { snprintf(err, ecap, "damaged zip directory"); return -1; }
    end = eocd;
    for (k = 0, i = cd; k < count; ++k) {
        size_t nl, next, lh, data, j;
        uint32_t usz, csz, attr, flags, method;
        char name[260];
        if (i + 46 > end || nz_u32(z + i) != 0x02014B50u) goto damaged;
        nl = nz_u16(z + i + 28); next = i + 46 + nl + nz_u16(z + i + 30) + nz_u16(z + i + 32);
        if (!nl || nl >= sizeof name || next > end || memchr(z + i + 46, 0, nl)) goto damaged;
        memcpy(name, z + i + 46, nl); name[nl] = 0;
        if (!nz_safe_name(name)) { snprintf(err, ecap, "unsafe zip path: %.100s", name); return -1; }
        flags = nz_u16(z + i + 8); method = nz_u16(z + i + 10); attr = nz_u32(z + i + 38);
        if ((flags & 1) || (method != 0 && method != 8) || nz_u16(z + i + 34) || ((attr >> 16) & 0170000) == 0120000) {
            snprintf(err, ecap, "unsupported zip entry (encrypted, symlink or compression): %.80s", name); return -1;
        }
        usz = nz_u32(z + i + 24); csz = nz_u32(z + i + 20);
        total += usz;
        if (usz > NZ_MAX_FILE || total > NZ_MAX_TOTAL) { snprintf(err, ecap, "zip file too large (64 MiB per file, 200 MiB total)"); return -1; }
        lh = nz_u32(z + i + 42);
        if (lh > cd || cd - lh < 30 || nz_u32(z + lh) != 0x04034B50u) goto damaged;
        data = lh + 30 + nz_u16(z + lh + 26) + nz_u16(z + lh + 28);
        if (data > cd || csz > cd - data || nz_u16(z + lh + 26) != nl || memcmp(z + lh + 30, z + i + 46, nl) ||
            nz_u16(z + lh + 8) != method || nz_u16(z + lh + 6) != flags || (method == 0 && csz != usz)) goto damaged;
        /* Exact duplicates would let the first match conceal a different payload. */
        for (j = cd; j < i; j += 46 + nz_u16(z + j + 28) + nz_u16(z + j + 30) + nz_u16(z + j + 32)) {
            if (nz_ieq((const char *) z + j + 46, nz_u16(z + j + 28), name, nl)) {
                snprintf(err, ecap, "duplicate zip path: %.100s", name); return -1;
            }
        }
        if (name[nl - 1] != '/' && name[nl - 1] != '\\') {
            if (!nz_allowed(name)) { snprintf(err, ecap, "unsupported file type in zip: %.100s", name); return -1; }
            if (names) {
                if (n >= cap) { snprintf(err, ecap, "too many files in zip"); return -1; }
                nc_copy(names[n], 260, name);
            }
            ++n;
        }
        i = next;
    }
    if (i != end) goto damaged;
    return n;
damaged:
    snprintf(err, ecap, "damaged zip directory or entry"); return -1;
}

static uint32_t nz_crc(const uint8_t *p, size_t n) {
    uint32_t table[256], c = 0xFFFFFFFFu, v;
    unsigned i, k;
    for (i = 0; i < 256; ++i) { v = i; for (k = 0; k < 8; ++k) v = (v >> 1) ^ (0xEDB88320u & (0u - (v & 1))); table[i] = v; }
    while (n--) c = table[(c ^ *p++) & 255] ^ (c >> 8);
    return ~c;
}
#endif
