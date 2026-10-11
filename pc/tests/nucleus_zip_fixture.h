/* Synthetic stored-zip fixtures. No downloaded or disc-derived bytes. */
#ifndef NUCLEUS_ZIP_FIXTURE_H
#define NUCLEUS_ZIP_FIXTURE_H
static uint32_t nst_crc(const uint8_t *p, size_t n) {
    uint32_t c = 0xFFFFFFFFu;
    size_t i;
    int k;
    for (i = 0; i < n; ++i) { c ^= p[i]; for (k = 0; k < 8; ++k) c = (c >> 1) ^ (0xEDB88320u & (0u - (c & 1u))); }
    return ~c;
}
static void nst_le16(uint8_t *p, unsigned v) { p[0] = (uint8_t) v; p[1] = (uint8_t) (v >> 8); }
static void nst_le32(uint8_t *p, uint32_t v) { nst_le16(p, v & 0xFFFF); nst_le16(p + 2, v >> 16); }

typedef struct { const char *name; const uint8_t *data; size_t len; } nst_entry;

/* a stored zip of the entries; returns its length (out must hold everything) */
static size_t nst_zip(const nst_entry *e, int n, uint8_t *out) {
    size_t o = 0, cd, i;
    uint32_t offs[8];
    int k;
    for (k = 0; k < n; ++k) {
        size_t nl = strlen(e[k].name);
        offs[k] = (uint32_t) o;
        nst_le32(out + o, 0x04034B50u); nst_le16(out + o + 4, 20); nst_le16(out + o + 6, 0); nst_le16(out + o + 8, 0); nst_le16(out + o + 10, 0); nst_le16(out + o + 12, 0);
        nst_le32(out + o + 14, nst_crc(e[k].data, e[k].len)); nst_le32(out + o + 18, (uint32_t) e[k].len); nst_le32(out + o + 22, (uint32_t) e[k].len);
        nst_le16(out + o + 26, (unsigned) nl); nst_le16(out + o + 28, 0);
        memcpy(out + o + 30, e[k].name, nl); memcpy(out + o + 30 + nl, e[k].data, e[k].len);
        o += 30 + nl + e[k].len;
    }
    cd = o;
    for (k = 0; k < n; ++k) {
        size_t nl = strlen(e[k].name);
        nst_le32(out + o, 0x02014B50u); nst_le16(out + o + 4, 20); nst_le16(out + o + 6, 20); nst_le16(out + o + 8, 0); nst_le16(out + o + 10, 0);
        nst_le16(out + o + 12, 0); nst_le16(out + o + 14, 0);
        nst_le32(out + o + 16, nst_crc(e[k].data, e[k].len)); nst_le32(out + o + 20, (uint32_t) e[k].len); nst_le32(out + o + 24, (uint32_t) e[k].len);
        nst_le16(out + o + 28, (unsigned) nl); nst_le16(out + o + 30, 0); nst_le16(out + o + 32, 0); nst_le16(out + o + 34, 0); nst_le16(out + o + 36, 0);
        nst_le32(out + o + 38, 0); nst_le32(out + o + 42, offs[k]);
        memcpy(out + o + 46, e[k].name, nl);
        o += 46 + nl;
    }
    i = o - cd;
    nst_le32(out + o, 0x06054B50u); nst_le16(out + o + 4, 0); nst_le16(out + o + 6, 0); nst_le16(out + o + 8, (unsigned) n); nst_le16(out + o + 10, (unsigned) n);
    nst_le32(out + o + 12, (uint32_t) i); nst_le32(out + o + 16, (uint32_t) cd); nst_le16(out + o + 20, 0);
    return o + 22;
}

static void nst_log(const char *line) { (void) line; }
static int nst_deny_commit(void *user) { (void) user; return 0; }

/* replace the first occurrence of `from` in `s` with `to` (a new malloc'd string) */
static char *nst_sub(const char *s, const char *from, const char *to) {
    const char *p = strstr(s, from);
    char *o = (char *) malloc(strlen(s) + strlen(to) + 1);
    size_t a = p ? (size_t) (p - s) : 0;
    if (!p) { strcpy(o, s); return o; }
    memcpy(o, s, a); strcpy(o + a, to); strcpy(o + a + strlen(to), p + strlen(from));
    return o;
}


#endif
