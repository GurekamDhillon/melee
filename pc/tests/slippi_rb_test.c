/* Standalone rollback seam test with synthetic fixture pads; no game or private .slp. */
#define _CRT_SECURE_NO_WARNINGS 1
#define typeof __typeof__ /* production uses GNU shorthand under the Windows shim build */
#include <assert.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "../platform/gw_slippi_pad.h"
#include "../platform/gw_rollback.c"

static int replay_frame = -124, fixture_reads[2], ticks, fixture_active = 1, flushed = -999;
static int startup_processed_bad_port = -1;
static void tick(int online_frame) { assert(online_frame == 1); ++ticks; }
void gw_log(const char *fmt, ...) { (void) fmt; }
int gw_Replay_Active(void) { return fixture_active; }
int gw_Replay_Frame(void) { return replay_frame; }
int gw_Replay_LastFrame(void) { return -120; }
int gw_Replay_Live(void) { return 0; }
int gw_Replay_PortHuman(int port) { return port < 2; }
int gw_Netplay_Enabled(void) { return 0; }
int gw_Netplay_LocalPort(void) { return 0; }
int gw_Netplay_RemotePort(void) { return 1; }
int gw_Netplay_Delay(void) { return 2; }
void gw_Netplay_Tick(void) {}
void gw_Netplay_MatchOver(void) {}
void gw_snap_save(int frame) { (void) frame; }
int gw_snap_load(int frame) { replay_frame = frame - 1; return 1; }
int gw_Snap_OpenSession(int count) { (void) count; return 1; }
void gw_Snap_SessionResim(int on, int frame) { (void) on; (void) frame; }
int gw_Snap_HasFrame(int frame) { (void) frame; return 1; }
uint32_t gw_Snap_Checksum(int frame) { (void) frame; return 1; }
int gw_Snap_SfxFrameEnd(int frame) { (void) frame; return 0; }
uint32_t gw_RB_GameHash(void) { return 0x1234; }
void gw_RbViz_Push(int a, int b, int c, int d, double e, int f) {
    (void) a; (void) b; (void) c; (void) d; (void) e; (void) f;
}
void gw_RbViz_Desync(int f, uint32_t a, uint32_t b) { (void) f; (void) a; (void) b; }
void gw_Replay_TraceBeginIter(int frame) { (void) frame; }
void gw_Replay_TraceFlushUpTo(int frame) { flushed = frame; }
int gw_Replay_PeekInput(int slot, int frame, GwRbInput *out) {
    assert((slot == 0 || slot == 2) && frame >= -123 && frame <= -122);
    memset(out, 0, sizeof *out);
    out->present = 1;
    if (slot / 2 == startup_processed_bad_port) out->trigger = 0.5f;
    return 1;
}
int gw_Replay_SlippiFixtureInfo(GwSlippiFixtureInfo *out) {
    if (!fixture_active) return 0;
    memset(out, 0, sizeof *out);
    out->first_frame = -123; out->last_frame = -120;
    out->online = 1; out->human_ports[0] = 0; out->human_ports[1] = 1;
    return 1;
}
const char *gw_Replay_SlippiFixtureReason(void) { return "test fixture"; }
void gw_Replay_SlippiLocalMismatchReset(void) {}
int gw_Replay_SlippiLocalMismatches(void) { return 0; }
int gw_Replay_SlippiPad(int port, int frame, GwSlippiPad *out) {
    if (port < 0 || port > 1 || frame < -123 || frame > -120) return 0;
    fixture_reads[port]++;
    memset(out, 0, sizeof *out);
    if (port == 0 && frame == -123) {
        out->bytes[5] = 0xFF; /* physical C-stick noise despite neutral processed input */
        out->bytes[7] = 3;
    }
    if (frame >= -121) {
        out->bytes[1] = 0x01; /* A, changing non-neutral input after delay window */
        out->bytes[2] = (uint8_t) (frame + 130);
    }
    return 1;
}

int main(void) {
    GwSlippiPad pad, bad;
    int epoch;
    assert(!gw_rb_slippi_receive(1, 1, 1, &pad)); /* off mode */
    startup_processed_bad_port = 1;
    assert(!gw_rb_slippi_configure(0, 2, tick)); /* other port's applied input must be neutral */
    startup_processed_bad_port = 0;
    assert(!gw_rb_slippi_configure(0, 2, tick)); /* local applied input must be neutral too */
    startup_processed_bad_port = -1;
    assert(gw_rb_slippi_configure(0, 2, tick));
    assert(gw_RB_Enabled());
    gw_RB_SceneBegin(2);
    epoch = gw_rb_epoch();
    assert(gw_RB_Iterations(1) >= 1);
    assert(ticks == 1);
    assert(gw_rb_slippi_local_pad(1, &pad) && !memcmp(pad.bytes, "\0\0\0\0\0\0\0\0", 8));
    assert(gw_rb_slippi_local_pad(2, &pad));
    assert(!gw_rb_slippi_local_pad(3, &pad));
    assert(gw_rb_local_fixture_reads(0) == 0 && gw_rb_local_fixture_reads(1) == 0);
    assert(gw_rb_confirmed_frame() == -124); /* remote dummy pads still require peer delivery */
    assert(!gw_rb_slippi_receive(epoch, 0, 1, &pad)); /* local port cannot become remote */
    assert(!gw_rb_slippi_receive(epoch - 1, 1, 1, &pad));
    bad = pad; bad.bytes[1] = 1;
    assert(!gw_rb_slippi_receive(epoch, 1, 1, &bad)); /* startup must be neutral */
    assert(gw_rb_slippi_receive(epoch, 1, 1, &pad));
    assert(!gw_rb_slippi_receive(epoch, 1, 1, &bad)); /* conflicting duplicate cannot be ACKed */
    assert(gw_rb_slippi_receive(epoch, 1, 2, &pad));
    assert(gw_rb_confirmed_frame() == -122);
    gw_RB_IterStart(); /* sample only local applied frame -121 for online PAD 3 */
    assert(gw_rb_slippi_local_pad(3, &pad) && pad.bytes[1] == 1 && pad.bytes[2] == 9);
    assert(gw_rb_local_fixture_reads(0) == 1 && gw_rb_local_fixture_reads(1) == 0);
    bad = pad;
    assert(!gw_rb_slippi_receive(epoch, 0, 3, &bad));
    assert(gw_rb_slippi_receive(epoch, 1, 3, &bad));
    assert(gw_rb_confirmed_frame() == -121);
    assert(!gw_rb_slippi_receive(epoch, 1, 10000, &bad)); /* no ACK for window refusal */
    assert(gw_rb_slippi_receive(epoch, 1, 4, &bad));
    replay_frame = -120;
    assert(!gw_rb_slippi_finalized()); /* local last pad was not sampled yet */
    rb_slippi_sample(-122);
    assert(gw_rb_slippi_finalized() && flushed == -120);
    gw_rb_slippi_disable();
    assert(!gw_rb_slippi_receive(epoch, 1, 3, &bad));

    memset(&rb, 0, sizeof rb); memset(fixture_reads, 0, sizeof fixture_reads);
    replay_frame = -124; ticks = 0;
    assert(gw_rb_slippi_configure(1, 2, tick));
    gw_RB_SceneBegin(2); assert(gw_RB_Iterations(1) >= 1); gw_RB_IterStart();
    assert(gw_rb_local_fixture_reads(0) == 0 && gw_rb_local_fixture_reads(1) == 1);
    assert(!gw_rb_slippi_receive(gw_rb_epoch(), 1, 3, &bad));
    gw_rb_slippi_disable();
    return 0;
}
