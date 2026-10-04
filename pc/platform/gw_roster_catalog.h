/* Dynamic native catalogue and versioned match-map codec. No guest struct widens.
 * IDs 0..127 retain legacy CK meaning; 128..65534 are native; 65535 is none.
 * Host allocations are boot metadata. Mutable match maps MUST be snapshotted by
 * the caller before gameplay integration; this module never silently activates one.
 */
#ifndef GW_ROSTER_CATALOG_H
#define GW_ROSTER_CATALOG_H
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#define GWR_NATIVE_FIRST 128u
#define GWR_NONE 65535u
#define GWR_NATIVE_MAX (GWR_NONE - GWR_NATIVE_FIRST)
#define GWR_FORMAT_VERSION 1u
#define GWR_MATCH_MAX 6u
#define GWR_HEADER_BYTES 36u
#define GWR_ROW_BYTES 96u

typedef struct GwrRow {
    uint64_t identity, table_hash, fighter_hash, animation_hash;
    uint16_t source_internal, source_external;
    char mod[64], key[40], name[48];
} GwrRow;

typedef struct GwrCatalog {
    GwrRow* rows;
    uint32_t count;
    uint64_t digest;
} GwrCatalog;

typedef struct GwrMatch {
    uint16_t ids[GWR_MATCH_MAX], aliases[GWR_MATCH_MAX], count;
} GwrMatch;

static inline uint16_t gwr_r16(const unsigned char* p)
{ return (uint16_t) (((uint16_t) p[0] << 8) | p[1]); }
static inline uint32_t gwr_r32(const unsigned char* p)
{ return ((uint32_t) gwr_r16(p) << 16) | gwr_r16(p + 2); }
static inline uint64_t gwr_r64(const unsigned char* p)
{ return ((uint64_t) gwr_r32(p) << 32) | gwr_r32(p + 4); }
static inline void gwr_w16(unsigned char* p, uint16_t v)
{ p[0] = (unsigned char) (v >> 8); p[1] = (unsigned char) v; }
static inline void gwr_w64(unsigned char* p, uint64_t v)
{
    int i;
    for (i = 7; i >= 0; --i) { p[i] = (unsigned char) v; v >>= 8; }
}
static inline uint64_t gwr_hash(uint64_t h, const void* data, size_t n)
{
    const unsigned char* p = (const unsigned char*) data;
    while (n--) { h ^= *p++; h *= UINT64_C(1099511628211); }
    return h;
}
static inline uint64_t gwr_bytes_hash(const void* p, size_t n)
{ return gwr_hash(UINT64_C(14695981039346656037), p, n); }

static inline int gwr_text(const unsigned char* p, size_t cap, char* out)
{
    size_t i;
    for (i = 0; i < cap && p[i]; ++i) {
        if (p[i] < 32 || p[i] > 126) return 0;
    }
    if (!i || i == cap) return 0;
    memcpy(out, p, i); out[i] = 0;
    /* Canonical encoding: nonzero hidden tails must not change peer identity. */
    while (++i < cap) if (p[i]) return 0;
    return 1;
}

static inline int gwr_row_order(const void* a, const void* b)
{
    const GwrRow* x = (const GwrRow*) a;
    const GwrRow* y = (const GwrRow*) b;
    int n = strcmp(x->mod, y->mod);
    return n ? n : strcmp(x->key, y->key);
}

static inline uint64_t gwr_row_hash(const GwrRow* r)
{
    unsigned char wire[28];
    uint64_t h = gwr_bytes_hash("GDRSTR01", 8);
    gwr_w64(wire, r->table_hash); gwr_w64(wire + 8, r->fighter_hash);
    gwr_w64(wire + 16, r->animation_hash);
    gwr_w16(wire + 24, r->source_internal); gwr_w16(wire + 26, r->source_external);
    h = gwr_hash(h, r->mod, strlen(r->mod) + 1);
    h = gwr_hash(h, r->key, strlen(r->key) + 1);
    h = gwr_hash(h, r->name, strlen(r->name) + 1);
    return gwr_hash(h, wire, sizeof wire);
}

static inline void gwr_catalog_free(GwrCatalog* c)
{ if (c) { free(c->rows); memset(c, 0, sizeof *c); } }

/* Whole-manifest transaction. A malformed/conflicting/full input leaves c unchanged. */
static inline int gwr_catalog_import(GwrCatalog* c, const char* mod,
                                      const unsigned char* bytes, size_t length)
{
    uint32_t n, i;
    GwrRow* next;
    uint64_t digest;
    if (!c || (c->count && !c->rows) || !mod || !mod[0] || strlen(mod) >= 64 || !bytes || length < GWR_HEADER_BYTES ||
        memcmp(bytes, "GDRSTR01", 8)) return 0;
    n = gwr_r32(bytes + 8);
    if (!n || n > GWR_NATIVE_MAX || c->count > GWR_NATIVE_MAX - n ||
        length != GWR_HEADER_BYTES + (size_t) n * GWR_ROW_BYTES) return 0;
    next = (GwrRow*) calloc((size_t) c->count + n, sizeof *next);
    if (!next) return 0;
    if (c->count) memcpy(next, c->rows, c->count * sizeof *next);
    for (i = 0; i < n; ++i) {
        const unsigned char* p = bytes + GWR_HEADER_BYTES + i * GWR_ROW_BYTES;
        GwrRow* r = &next[c->count + i];
        r->source_internal = gwr_r16(p); r->source_external = gwr_r16(p + 2);
        if (r->source_internal > 127 || r->source_external > 127 || gwr_r32(p + 4) ||
            !gwr_text(p + 8, 40, r->key) || !gwr_text(p + 48, 48, r->name)) {
            free(next); return 0;
        }
        memcpy(r->mod, mod, strlen(mod) + 1);
        r->table_hash = gwr_r64(bytes + 12); r->fighter_hash = gwr_r64(bytes + 20);
        r->animation_hash = gwr_r64(bytes + 28);
        r->identity = gwr_row_hash(r);
    }
    qsort(next, c->count + n, sizeof *next, gwr_row_order);
    digest = gwr_bytes_hash("GWR-CATALOG-1", 13);
    for (i = 0; i < c->count + n; ++i) {
        unsigned char encoded[8];
        if (i && !gwr_row_order(&next[i-1], &next[i])) { free(next); return 0; }
        gwr_w64(encoded, next[i].identity);
        digest = gwr_hash(digest, encoded, sizeof encoded);
    }
    free(c->rows); c->rows = next; c->count += n; c->digest = digest;
    return 1;
}

static inline const GwrRow* gwr_catalog_at(const GwrCatalog* c, uint16_t id)
{
    return c && id >= GWR_NATIVE_FIRST && (uint32_t) id - GWR_NATIVE_FIRST < c->count
               ? &c->rows[id - GWR_NATIVE_FIRST] : NULL;
}

/* Plan unused resident aliases, not a mutation of the live engine's tables.
 * Legacy m-ex slots stay intact. More than six distinct selected kinds or no
 * spare alias space refuses the entire request. Transformation partners must
 * be included by the game-side caller in the requested list.
 */
static inline int gwr_match_plan(const GwrCatalog* c, GwrMatch* out,
                                 const uint16_t* ids, unsigned count, unsigned legacy_slots)
{
    GwrMatch next = {{0}, {0}, 0};
    unsigned i, j;
    int alias = 127;
    if (!c || !out || !ids || count > GWR_MATCH_MAX || legacy_slots > 94) return 0;
    for (i = 0; i < count; ++i) {
        uint16_t id = ids[i];
        if (id == GWR_NONE || id == 33 || (id >= 128 && !gwr_catalog_at(c, id))) return 0;
        for (j = 0; j < next.count && next.ids[j] != id; ++j) {}
        if (j < next.count) continue;
        next.ids[next.count] = id;
        if (id < 128) next.aliases[next.count] = id;
        else {
            int conflict;
            do {
                conflict = 0;
                for (j = 0; j < count; ++j) if (ids[j] == alias) conflict = 1;
                if (conflict) --alias;
            } while (conflict && alias >= (int) (34 + legacy_slots));
            if (alias < (int) (34 + legacy_slots)) return 0;
            next.aliases[next.count] = (uint16_t) alias--;
        }
        ++next.count;
    }
    *out = next;
    return 1;
}

static inline int gwr_match_write(const GwrCatalog* c, const GwrMatch* m,
                                  unsigned char* out, size_t cap)
{
    unsigned i;
    size_t n;
    if (!c || !m || !out || m->count > GWR_MATCH_MAX) return 0;
    n = 16 + (size_t) m->count * 4;
    if (cap < n) return 0;
    for (i = 0; i < m->count; ++i) {
        unsigned j;
        uint16_t id = m->ids[i], alias = m->aliases[i];
        if (id == GWR_NONE || id == 33 || alias > 127 || alias == 33 ||
            (id < 128 && alias != id) ||
            (id >= 128 && (alias < 34 || !gwr_catalog_at(c, id)))) return 0;
        for (j = 0; j < i; ++j) if (m->ids[j] == id || m->aliases[j] == alias) return 0;
    }
    memcpy(out, "GWRM", 4); gwr_w16(out + 4, GWR_FORMAT_VERSION);
    gwr_w16(out + 6, m->count); gwr_w64(out + 8, c->digest);
    for (i = 0; i < m->count; ++i) {
        gwr_w16(out + 16 + 4*i, m->ids[i]); gwr_w16(out + 18 + 4*i, m->aliases[i]);
    }
    return (int) n;
}

static inline int gwr_match_read(const GwrCatalog* c, GwrMatch* out,
                                 const unsigned char* bytes, size_t length, unsigned legacy_slots)
{
    GwrMatch expected;
    uint16_t ids[GWR_MATCH_MAX];
    unsigned i, n;
    if (!c || !out || !bytes || length < 16 || memcmp(bytes, "GWRM", 4) ||
        gwr_r16(bytes + 4) != GWR_FORMAT_VERSION || gwr_r64(bytes + 8) != c->digest) return 0;
    n = gwr_r16(bytes + 6);
    if (n > GWR_MATCH_MAX || length != 16 + (size_t) n * 4) return 0;
    for (i = 0; i < n; ++i) ids[i] = gwr_r16(bytes + 16 + i*4);
    if (!gwr_match_plan(c, &expected, ids, n, legacy_slots) || expected.count != n) return 0;
    for (i = 0; i < n; ++i) if (expected.aliases[i] != gwr_r16(bytes + 18 + i*4)) return 0;
    *out = expected;
    return 1;
}

static inline unsigned gwr_page_count(unsigned total, unsigned per, unsigned page)
{
    unsigned first;
    if (!per || !total || total >= GWR_NONE || page >= 1 + (total - 1) / per) return 0;
    first = page * per;
    return total - first < per ? total - first : per;
}
#endif
