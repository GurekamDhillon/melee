/* Standalone native test; no game, bridge, Windows SDK or disc required. */
#include "../platform/gw_roster_registry.h"
#include <assert.h>
#include <stdio.h>

int main(void)
{
    GwRosterRegistry r = {0};
    GwRosterMatch m;
    uint16_t ids[6] = {0, 127, 128, 200, 254, 255};
    unsigned char wire[2];
    int i;
    for (i = 0; i < 256; ++i) {
        assert(gw_roster_add(&r, (uint64_t) i + 1, 31) == i);
    }
    assert(gw_roster_add(&r, 999, 31) == GW_ROSTER_NONE);
    assert(gw_roster_add(&r, 1000, 128) == GW_ROSTER_NONE);
    for (i = 0; i < 6; ++i) {
        gw_roster_id_write(wire, ids[i]);
        assert(gw_roster_id_read(wire) == ids[i]);
    }
    gw_roster_id_write(wire, GW_ROSTER_NONE);
    assert(gw_roster_id_read(wire) == GW_ROSTER_NONE);
    assert(gw_roster_bind(&r, &m, ids, 6));
    for (i = 0; i < 6; ++i) {
        assert(gw_roster_match_slot(&m, ids[i]) == i);
        assert(gw_roster_match_id(&m, i) == ids[i]);
    }
    assert(gw_roster_match_slot(&m, GW_ROSTER_NONE) == -1);
    assert(gw_roster_match_id(&m, 6) == GW_ROSTER_NONE);
    assert(gw_roster_match_id(&m, -1) == GW_ROSTER_NONE);
    assert(!gw_roster_bind(&r, &m, ids, 7));
    assert(m.count == 6); /* A refused bind leaves the old match untouched. */
    ids[5] = GW_ROSTER_NONE;
    assert(!gw_roster_bind(&r, &m, ids, 6));
    assert(m.ids[5] == 255);
    ids[5] = 0;
    assert(gw_roster_bind(&r, &m, ids, 6));
    assert(m.count == 5); /* Same selected fighter reuses one resident slot. */
    assert(gw_roster_pages(256, 30) == 9);
    assert(gw_roster_page_count(256, 30, 8) == 16);
    assert(gw_roster_page_count(256, 30, 9) == 0);
    assert(gw_roster_pages(0, 30) == 0);
    assert(gw_roster_pages(256, 0) == 0);
    assert(gw_roster_page_count(256, 30, -1) == 0);
    assert(gw_roster_pages(257, 30) == 0);
    puts("roster registry: PASS (wide ids, none, transactional bounds, pages)");
    return 0;
}
