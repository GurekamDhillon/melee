/* Packed per-physical-port match choices. Older processed-input replays use
 * default Both; raw .gdlp inputs are transformed with the recorded choices. */
#ifndef GW_REPLAY_CONTROLS_H
#define GW_REPLAY_CONTROLS_H
#include "gw_replay_matchrules.h"
#define GW_RP_CONTROLS_START (GW_RP_RULES_START + 20)

static int gw_controls_word_valid(unsigned word) {
    return !(word & ~0xff3du);
}
static void gw_rp_controls_write(unsigned char *event, const unsigned words[4]) {
    unsigned char *p = event + 1 + GW_RP_RULES_START;
    int i;
    p[0] = 'G'; p[1] = 'D'; p[2] = 'C'; p[3] = '1';
    for (i = 0; i < 4; ++i) {
        unsigned w = words[i];
        p[4 + i * 4] = (unsigned char)(w >> 24);
        p[5 + i * 4] = (unsigned char)(w >> 16);
        p[6 + i * 4] = (unsigned char)(w >> 8);
        p[7 + i * 4] = (unsigned char)w;
    }
}
static int gw_rp_controls_read(const unsigned char *event, size_t size, unsigned words[4]) {
    const unsigned char *p;
    unsigned tmp[4];
    int i;
    for (i = 0; i < 4; ++i) words[i] = 0;
    if (size <= GW_RP_RULES_START) return 0;
    p = event + 1 + GW_RP_RULES_START;
    /* An unrelated upstream extension is harmless; a torn GD tag is refused. */
    if (p[0] != 'G') return 0;
    if (size < GW_RP_RULES_START + 4) return -1;
    if (p[1] != 'D' || p[2] != 'C') return 0;
    if (p[3] != '1' || size < GW_RP_CONTROLS_START) return -1;
    for (i = 0; i < 4; ++i) {
        tmp[i] = ((unsigned)p[4 + i * 4] << 24) | ((unsigned)p[5 + i * 4] << 16) |
                 ((unsigned)p[6 + i * 4] << 8) | p[7 + i * 4];
        if (!gw_controls_word_valid(tmp[i])) return -1;
    }
    for (i = 0; i < 4; ++i) words[i] = tmp[i];
    return 0;
}
#endif
