/* Exercise the real replay parser on synthetic, non-disc-derived .slp bytes. */
#define _CRT_SECURE_NO_WARNINGS 1
#include <assert.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "../platform/gw_replay.c"

void gw_log(const char *fmt, ...) { (void) fmt; }
/* Unused game callbacks retained by the Windows linker for this source-inclusion test. */
uint32_t gw_Netplay_Handshake(uint8_t *p, int n, int off, int len) {
    (void) p; (void) n; (void) off; (void) len; return 0;
}
int gw_rb_active(void) { return 0; }
const GwRbInput *gw_RB_InputFor(int port, int follower, int frame) {
    (void) port; (void) follower; (void) frame; return NULL;
}
const GwRbInput *gw_RB_InputAny(int port, int frame) {
    (void) port; (void) frame; return NULL;
}
int gw_Snap_Resimulating(void) { return 0; }

static void be32(uint8_t *p, uint32_t n) {
    p[0] = (uint8_t) (n >> 24); p[1] = (uint8_t) (n >> 16);
    p[2] = (uint8_t) (n >> 8); p[3] = (uint8_t) n;
}
static void bef(uint8_t *p, float f) { uint32_t u; memcpy(&u, &f, 4); be32(p, u); }

static void clear_replay(void) {
    free(rp.in); free(rp.fs_seed); free(rp.fs_has); free(rp.post);
    memset(&rp, 0, sizeof rp);
    rp.tried = 1;
}

static size_t fixture(uint8_t *d, int minor, int pre_size, int omit_p2, int truncate) {
    uint8_t *p = d + 12;
    int port;
    memcpy(d, "raw[$U#l", 8);
    *p++ = 0x35; *p++ = 7;
    *p++ = 0x36; *p++ = 1; *p++ = 0xA4;
    *p++ = 0x37; *p++ = 0; *p++ = (uint8_t) pre_size;
    memset(p, 0, 1 + 0x1A4);
    p[0] = 0x36; p[1] = 3; p[2] = (uint8_t) minor;
    p[0x1A4] = 8;
    be32(p + 0x13D, 0x12345678);
    for (port = 0; port < 4; ++port) p[5 + 0x60 + 0x24 * port + 1] = port < 2 ? 0 : 3;
    p += 1 + 0x1A4;
    for (port = 0; port < (omit_p2 ? 1 : 2); ++port) {
        memset(p, 0, 1 + pre_size);
        p[0] = 0x37; be32(p + 1, (uint32_t) -123); p[5] = (uint8_t) port;
        if (pre_size >= 0x42) {
            p[0x31] = 1; p[0x32] = 0;
            bef(p + 0x33, 0.0f); bef(p + 0x37, 1.0f);
            p[0x3B] = 0x7f; p[0x40] = 0x80; p[0x41] = 1; p[0x42] = 2;
        }
        p += 1 + pre_size;
    }
    be32(d + 8, (uint32_t) (p - (d + 12)));
    return (size_t) (p - d) - (size_t) truncate;
}

int main(void) {
    uint8_t d[2048] = {0};
    GwSlippiFixtureInfo info;
    GwSlippiPad pad;
    size_t n;

    n = fixture(d, 17, 0x42, 0, 0); clear_replay();
    assert(rp_parse(d, n) == 0); rp.active = 1;
    assert(gw_Replay_SlippiFixtureInfo(&info));
    assert(info.online && info.first_frame == -123 && info.last_frame == -123);
    assert(info.seed == 0x12345678 && info.human_ports[0] == 0 && info.human_ports[1] == 1);
    assert(gw_Replay_SlippiPad(0, -123, &pad));
    assert(pad.bytes[0] == 1 && pad.bytes[1] == 0 && pad.bytes[2] == 0x7f);
    assert(pad.bytes[3] == 0x80 && pad.bytes[4] == 1 && pad.bytes[5] == 2);
    assert(pad.bytes[6] == 0 && pad.bytes[7] == 140);
    assert(!gw_Replay_SlippiPad(2, -123, &pad));
    assert(!gw_Replay_SlippiPad(-1, -123, &pad));

    n = fixture(d, 16, 0x42, 0, 0); clear_replay();
    assert(rp_parse(d, n) == 0); rp.active = 1;
    assert(!gw_Replay_SlippiFixtureInfo(&info));
    assert(strcmp(gw_Replay_SlippiFixtureReason(), "Slippi version before 3.17") == 0);

    n = fixture(d, 17, 0x41, 0, 0); clear_replay();
    assert(rp_parse(d, n) == 0); rp.active = 1;
    assert(!gw_Replay_SlippiFixtureInfo(&info));
    assert(strcmp(gw_Replay_SlippiFixtureReason(), "missing physical input fields") == 0);

    n = fixture(d, 17, 0x42, 1, 0); clear_replay();
    assert(rp_parse(d, n) == 0); rp.active = 1;
    assert(!gw_Replay_SlippiFixtureInfo(&info));
    assert(strcmp(gw_Replay_SlippiFixtureReason(), "missing player input") == 0);

    n = fixture(d, 17, 0x42, 0, 1); clear_replay();
    assert(rp_parse(d, n) == 0); rp.active = 1;
    assert(!gw_Replay_SlippiFixtureInfo(&info));
    assert(strcmp(gw_Replay_SlippiFixtureReason(), "truncated replay") == 0);

    clear_replay();
    return 0;
}
