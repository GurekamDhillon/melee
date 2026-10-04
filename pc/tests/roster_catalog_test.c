/* Standalone C11 test. Input is an artificial manifest, no game assets. */
#include "../platform/gw_roster_catalog.h"
#include <assert.h>
#include <stdio.h>

int main(int argc, char** argv)
{
    GwrCatalog c = {0}, reordered = {0};
    GwrMatch match, restored;
    uint16_t ids[6] = {0, 127, 128, 200, 227, 128};
    unsigned char wire[64], *bytes;
    FILE* file;
    long length;
    uint64_t digest;
    int n;
    assert(argc == 2);
    file = fopen(argv[1], "rb"); assert(file);
    assert(!fseek(file, 0, SEEK_END)); length = ftell(file); assert(length > 0);
    assert(!fseek(file, 0, SEEK_SET)); bytes = (unsigned char*) malloc((size_t) length); assert(bytes);
    assert(fread(bytes, 1, (size_t) length, file) == (size_t) length); fclose(file);
    assert(gwr_catalog_import(&c, "sonic-test", bytes, (size_t) length));
    assert(c.count == 100);
    assert(!strcmp(gwr_catalog_at(&c, 128)->name, "Sonic 001"));
    assert(!strcmp(gwr_catalog_at(&c, 227)->name, "Sonic 100"));
    assert(gwr_catalog_at(&c, 128)->identity != gwr_catalog_at(&c, 129)->identity);
    assert(!gwr_catalog_at(&c, 127)); assert(!gwr_catalog_at(&c, 228));
    digest = c.digest;
    assert(!gwr_catalog_import(&c, "sonic-test", bytes, (size_t) length));
    assert(c.count == 100 && c.digest == digest);
    assert(!gwr_catalog_import(&c, "truncated", bytes, (size_t) length - 1));
    assert(c.count == 100 && c.digest == digest);
    bytes[0] ^= 1;
    assert(!gwr_catalog_import(&c, "wrong-version", bytes, (size_t) length)); bytes[0] ^= 1;
    assert(gwr_match_plan(&c, &match, ids, 6, 31));
    assert(match.count == 5);
    assert(match.aliases[0] == 0 && match.aliases[1] == 127);
    assert(match.aliases[2] == 126 && match.aliases[3] == 125 && match.aliases[4] == 124);
    n = gwr_match_write(&c, &match, wire, sizeof wire); assert(n == 36);
    assert(gwr_match_read(&c, &restored, wire, (size_t) n, 31));
    assert(!memcmp(&match, &restored, sizeof match));
    {
        GwrMatch bad = match;
        unsigned char untouched[64];
        memset(untouched, 0xCC, sizeof untouched); bad.aliases[0] = 128;
        assert(!gwr_match_write(&c, &bad, untouched, sizeof untouched));
        assert(untouched[0] == 0xCC);
    }
    assert(!gwr_match_read(&c, &restored, wire, (size_t) n - 1, 31));
    wire[8] ^= 1; assert(!gwr_match_read(&c, &restored, wire, (size_t) n, 31)); wire[8] ^= 1;
    wire[18] ^= 1; assert(!gwr_match_read(&c, &restored, wire, (size_t) n, 31)); wire[18] ^= 1;
    ids[0] = 65535; assert(!gwr_match_plan(&c, &restored, ids, 6, 31));
    assert(!memcmp(&match, &restored, sizeof match));
    ids[0] = 0; assert(!gwr_match_plan(&c, &restored, ids, 6, 94));
    assert(!gwr_match_plan(&c, &restored, ids, 7, 31));
    assert(gwr_page_count(165, 44, 3) == 33);
    assert(gwr_page_count(165, 44, 4) == 0);
    assert(gwr_page_count(165, 0, 0) == 0);
    assert(gwr_catalog_import(&c, "a-test", bytes, (size_t) length));
    assert(gwr_catalog_import(&reordered, "a-test", bytes, (size_t) length));
    assert(gwr_catalog_import(&reordered, "sonic-test", bytes, (size_t) length));
    assert(c.digest == reordered.digest);
    assert(c.count == 200 && gwr_catalog_at(&c, 327)); /* Beyond the old 256-row foundation. */
    assert(!gwr_match_read(&c, &restored, wire, (size_t) n, 31));
    gwr_catalog_free(&c); gwr_catalog_free(&reordered); free(bytes);
    {
        unsigned i;
        size_t size = GWR_HEADER_BYTES + (size_t) GWR_NATIVE_MAX * GWR_ROW_BYTES;
        unsigned char* large = (unsigned char*) calloc(size, 1);
        uint16_t last[1] = {65534};
        assert(large);
        memcpy(large, "GDRSTR01", 8);
        gwr_w16(large + 8, (uint16_t) (GWR_NATIVE_MAX >> 16));
        gwr_w16(large + 10, (uint16_t) GWR_NATIVE_MAX);
        for (i = 0; i < GWR_NATIVE_MAX; ++i) {
            unsigned char* row = large + GWR_HEADER_BYTES + i*GWR_ROW_BYTES;
            gwr_w16(row, 31); gwr_w16(row + 2, 30);
            snprintf((char*) row + 8, 40, "clone-%05u", i);
            snprintf((char*) row + 48, 48, "Sonic %05u", i);
        }
        assert(gwr_catalog_import(&c, "maximum", large, size));
        assert(c.count == GWR_NATIVE_MAX && gwr_catalog_at(&c, 65534));
        assert(!gwr_catalog_at(&c, 65535));
        digest = c.digest;
        assert(!gwr_catalog_import(&c, "overflow", large, size));
        assert(c.digest == digest && c.count == GWR_NATIVE_MAX);
        assert(gwr_match_plan(&c, &match, last, 1, 31));
        n = gwr_match_write(&c, &match, wire, sizeof wire);
        assert(gwr_match_read(&c, &restored, wire, (size_t) n, 31));
        assert(restored.ids[0] == 65534);
        assert(gwr_page_count(165, UINT32_MAX, 0) == 165);
        gwr_catalog_free(&c); free(large);
    }
    puts("roster catalogue: PASS (100 identities, canonical order, aliases, versioned restore, bounds)");
    return 0;
}
