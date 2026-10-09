/* Port-only Game Start suffix, after the standard Slippi 3.19.1 payload.
 * Event-size framing lets readers skip it. Old recordings have no suffix:
 * always off, independent of local settings. No disc-derived data here. */
#ifndef GW_REPLAY_MATCHRULES_H
#define GW_REPLAY_MATCHRULES_H
#include "gw_matchrules.h"
#include <stddef.h>

#define GW_RP_STANDARD_START 0x2F8
#define GW_RP_RULES_START (GW_RP_STANDARD_START + 8)

static void gw_rp_rules_write(unsigned char* event, unsigned rules)
{
    unsigned char* p = event + 1 + GW_RP_STANDARD_START;
    p[0] = 'G'; p[1] = 'D'; p[2] = 'T'; p[3] = '1';
    p[4] = (unsigned char) (rules >> 24); p[5] = (unsigned char) (rules >> 16);
    p[6] = (unsigned char) (rules >> 8); p[7] = (unsigned char) rules;
}

/* Size is payload bytes, excluding command byte. -1 means an unsupported
 * tagged rule (refuse playback); untagged upstream recordings mean off. */
static int gw_rp_rules_read(const unsigned char* event, size_t size, unsigned* rules)
{
    const unsigned char* p;
    *rules = 0;
    if (size < GW_RP_STANDARD_START + 4) return 0;
    p = event + 1 + GW_RP_STANDARD_START;
    if (p[0] != 'G' || p[1] != 'D' || p[2] != 'T') return 0;
    if (p[3] != '1' || size < GW_RP_RULES_START) return -1;
    *rules = ((unsigned) p[4] << 24) | ((unsigned) p[5] << 16) |
             ((unsigned) p[6] << 8) | p[7];
    return gw_turbo_valid(*rules) ? 0 : -1;
}
#endif
