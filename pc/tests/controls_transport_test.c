/* Wire metadata fixtures: no disc, game memory, settings or sockets required. */
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include "../platform/gw_replay_controls.h"

int main(void) {
    unsigned char event[1 + GW_RP_CONTROLS_START] = {0};
    unsigned want[4] = {1, 2 | (3u << 2) | (5u << 8), 0, 0xff00}, got[4];
    assert(!gw_rp_controls_read(event, GW_RP_STANDARD_START, got));
    assert(!got[0] && !got[1] && !got[2] && !got[3]);
    gw_rp_controls_write(event, want);
    assert(!gw_rp_controls_read(event, GW_RP_CONTROLS_START, got));
    assert(!memcmp(want, got, sizeof got));
    assert(gw_rp_controls_read(event, GW_RP_CONTROLS_START - 1, got) < 0);
    event[1 + GW_RP_RULES_START + 3] = '2';
    assert(gw_rp_controls_read(event, GW_RP_CONTROLS_START, got) < 0);
    gw_rp_controls_write(event, want);
    event[1 + GW_RP_RULES_START + 7] = 3; /* reserved jump value */
    assert(gw_rp_controls_read(event, GW_RP_CONTROLS_START, got) < 0);
    want[0] = 0x40; /* reserved choice bit */
    gw_rp_controls_write(event, want);
    assert(gw_rp_controls_read(event, GW_RP_CONTROLS_START, got) < 0);
    puts("controls transport: player roundtrip, old defaults, truncated/version/reserved refusal PASS");
    return 0;
}
