#include <assert.h>
#include <stdio.h>
#include "../platform/geno_define_core.h"
#include "../geno/geno_profile_storage.h"

int main(void)
{
    {
        unsigned char arena[96] = {0}, saved[96];
        unsigned int used = 0, mark;
        void* first = geno_storage_take(arena, sizeof arena, &used, 4);
        void* next;
        assert(first == arena && used == 8);
        memcpy(saved, arena, sizeof arena); mark = used;
        next = geno_storage_take(arena, sizeof arena, &used, 17);
        assert(next == arena+8 && used == 32);
        assert(!geno_storage_take(arena, sizeof arena, &used, 65));
        assert(used == 32); /* budget refusal is transactional */
        memcpy(arena, saved, sizeof arena); used = mark; /* snapshot model */
        assert(geno_storage_take(arena, sizeof arena, &used, 17) == next);
        assert(!geno_storage_take(arena, sizeof arena, &used, -1));
        assert(used == 32);
    }
    GdfCatalog c = {0};
    unsigned char occupied[128] = {0};
    int a, b;
    occupied[127] = 1; occupied[126] = 1;
    assert(gdf_add(&c, "hero", "hero", "Vanilla Hero", 1, 0));
    assert(gdf_add(&c, "other", "second", "Second", 2, 1));
    assert(!gdf_add(&c, "hero", "hero", "duplicate", 3, 2));
    assert(c.catalog.count == 2);
    assert(gdf_bind(&c, occupied));
    a = gdf_alias_for_profile(&c, 0); b = gdf_alias_for_profile(&c, 1);
    assert(a == 125 && b == 124);
    assert(gdf_profile_for_kind(&c, a-1) == 0);
    assert(gdf_profile_for_kind(&c, 0) == -1);
    assert(c.catalog.digest != 0);
    {
        GdfCatalog reverse = {0};
        assert(gdf_add(&reverse, "other", "second", "Second", 2, 1));
        assert(gdf_add(&reverse, "hero", "hero", "Vanilla Hero", 1, 0));
        assert(gdf_bind(&reverse, occupied));
        assert(reverse.catalog.digest == c.catalog.digest);
        assert(gdf_alias_for_profile(&reverse, 0) == a);
        assert(gdf_alias_for_profile(&reverse, 1) == b);
        gdf_free(&reverse);
    }
    occupied[125] = 1; occupied[124] = 1;
    assert(gdf_bind(&c, occupied));
    assert(gdf_alias_for_profile(&c, 0) == 123);
    for (a = 34; a < 128; ++a) occupied[a] = 1;
    assert(!gdf_bind(&c, occupied));
    assert(gdf_alias_for_profile(&c, 0) == 123); /* failed transaction preserves map */
    gdf_free(&c);
    assert(GENO_ART_KIND(31, 7) == 0x10ff);
    assert(GENO_ART_KIND(0, 8) == 0x1100);
    assert(GENO_ART_KIND(31, 15) == 0x11ff);
    assert(GENO_ART_KIND(32, 0) == 0x20000);
    assert(gdf_article_profile(GENO_ART_KIND(255, 15)) == 255);
    assert(gdf_article_index(GENO_ART_KIND(255, 15)) == 15);
    assert(gdf_article_profile(5000) == -1);
    puts("geno define core: PASS catalogue/alias refusal and legacy/extended article ids");
    return 0;
}
