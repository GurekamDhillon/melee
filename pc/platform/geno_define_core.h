/* Disc-independent definition catalogue and immutable boot resident aliases.
 * Reuses the roster catalogue's stable identity primitives, not its MxDt loader.
 */
#ifndef GENO_DEFINE_CORE_H
#define GENO_DEFINE_CORE_H
#include "gw_roster_catalog.h"
#include "../geno/geno.h"

typedef struct GdfCatalog {
    GwrCatalog catalog;
    int* profiles;
    int* aliases;
} GdfCatalog;

static inline void gdf_free(GdfCatalog* c)
{
    gwr_catalog_free(&c->catalog); free(c->profiles); free(c->aliases);
    memset(c, 0, sizeof *c);
}

static inline int gdf_add(GdfCatalog* c, const char* mod, const char* key,
                          const char* name, uint64_t hash, int profile)
{
    GwrCatalog next = {0};
    unsigned char manifest[GWR_HEADER_BYTES + GWR_ROW_BYTES] = {0};
    unsigned i, n = c->catalog.count;
    int *profiles, *aliases;
    if (!key || !name || !key[0] || strlen(key) >= 40 || !name[0] || strlen(name) >= 48 || profile < 0) return 0;
    if (n) {
        next.rows = (GwrRow*) malloc(n * sizeof *next.rows);
        if (!next.rows) return 0;
        memcpy(next.rows, c->catalog.rows, n * sizeof *next.rows); next.count = n;
    }
    memcpy(manifest, "GDRSTR01", 8); manifest[11] = 1;
    gwr_w64(manifest + 12, UINT64_C(0x47454e4f44454631)); /* GENODEF1, not a disc/table hash */
    gwr_w64(manifest + 20, hash); gwr_w64(manifest + 28, 1); /* behavior ABI */
    gwr_w16(manifest + GWR_HEADER_BYTES, 0); gwr_w16(manifest + GWR_HEADER_BYTES + 2, 8);
    memcpy(manifest + GWR_HEADER_BYTES + 8, key, strlen(key));
    memcpy(manifest + GWR_HEADER_BYTES + 48, name, strlen(name));
    if (!gwr_catalog_import(&next, mod, manifest, sizeof manifest)) { gwr_catalog_free(&next); return 0; }
    profiles = (int*) malloc((n+1) * sizeof *profiles);
    aliases = (int*) malloc((n+1) * sizeof *aliases);
    if (!profiles || !aliases) { free(profiles); free(aliases); gwr_catalog_free(&next); return 0; }
    for (i = 0; i <= n; ++i) {
        unsigned j;
        profiles[i] = profile; aliases[i] = -1;
        for (j = 0; j < n; ++j) if (!gwr_row_order(&next.rows[i], &c->catalog.rows[j])) {
            profiles[i] = c->profiles[j]; aliases[i] = c->aliases[j]; break;
        }
    }
    gdf_free(c); c->catalog = next; c->profiles = profiles; c->aliases = aliases;
    return 1;
}

static inline int gdf_bind(GdfCatalog* c, const unsigned char occupied[128])
{
    unsigned i;
    int next[94], alias = 127;
    if (c->catalog.count > 94) return 0;
    for (i = 0; i < c->catalog.count; ++i) {
        while (alias >= 34 && occupied[alias]) --alias;
        if (alias < 34) return 0;
        next[i] = alias--;
    }
    for (i = 0; i < c->catalog.count; ++i) c->aliases[i] = next[i];
    return 1;
}

static inline int gdf_alias_for_profile(const GdfCatalog* c, int p)
{
    unsigned i;
    for (i = 0; i < c->catalog.count; ++i) if (c->profiles[i] == p) return c->aliases[i];
    return -1;
}
static inline int gdf_profile_for_kind(const GdfCatalog* c, int kind)
{
    unsigned i;
    for (i = 0; i < c->catalog.count; ++i) if (c->aliases[i] >= 34 && c->aliases[i]-1 == kind) return c->profiles[i];
    return -1;
}
static inline int gdf_article_profile(int kind)
{
    if (kind >= GENO_ART_KIND_BASE && kind < GENO_ART_KIND_BASE2) return (kind-GENO_ART_KIND_BASE)/8;
    if (kind >= GENO_ART_KIND_BASE2 && kind < GENO_ART_KIND_END) return (kind-GENO_ART_KIND_BASE2)/8;
    if (kind >= GENO_ART_EXTRA_BASE && kind < GENO_ART_EXTRA_END) return 32+(kind-GENO_ART_EXTRA_BASE)/16;
    return -1;
}
static inline int gdf_article_index(int kind)
{
    if (kind >= GENO_ART_KIND_BASE && kind < GENO_ART_KIND_BASE2) return (kind-GENO_ART_KIND_BASE)%8;
    if (kind >= GENO_ART_KIND_BASE2 && kind < GENO_ART_KIND_END) return 8+(kind-GENO_ART_KIND_BASE2)%8;
    if (kind >= GENO_ART_EXTRA_BASE && kind < GENO_ART_EXTRA_END) return (kind-GENO_ART_EXTRA_BASE)%16;
    return -1;
}
#endif
