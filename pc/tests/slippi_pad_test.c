/* Standalone focused Slippi PAD contract test. */
#include "../platform/gw_slippi_pad.h"
#include <assert.h>
#include <string.h>

int main(void) {
    GwSlippiPad pad;
    GwRbInput in;
    const uint8_t expected[8] = {0x01, 0x60, 0x80, 0x7f, 0xfe, 0x02, 0x00, 140};
    assert(gw_SlippiPad_FromPhysical(0x0160, -128, 127, -2, 2, 0.0f, 1.0f, &pad));
    assert(memcmp(pad.bytes, expected, 8) == 0);
    assert(gw_SlippiPad_ToRb(&pad, &in));
    assert(in.is_raw && in.present && in.confirmed && in.buttons == 0x0160);
    assert(in.raw[0] == -128 && in.raw[1] == 127 && in.raw[2] == -2 && in.raw[3] == 2);
    assert(in.pad_l == 0 && in.pad_r == 140 && in.pad_err == 0);
    assert(gw_SlippiPad_TriggerByte(0.5f / 140.0f) == 1);
    assert(gw_SlippiPad_TriggerByte(69.5f / 140.0f) == 70);
    assert(gw_SlippiPad_OnlineFrame(-123) == 1);
    assert(gw_SlippiPad_SlpFrame(1) == -123);
    assert(gw_SlippiPad_OnlineFrame(-122) == 2);
    assert(gw_SlippiPad_SampleFrame(-123, 2) == -1);
    assert(gw_SlippiPad_SampleFrame(-121, 2) == 1);
    assert(gw_SlippiPad_OnlineFrame(-124) == -1);
    /* Every original trigger byte must process identically after canonical replay encoding. */
    for (int original = 0; original <= 255; ++original) {
        int clamped = original > 140 ? 140 : original;
        float recorded = (float) clamped / 140.0f;
        int decoded = gw_SlippiPad_TriggerByte(recorded);
        assert(decoded == clamped);
    }
    assert(!gw_SlippiPad_FromPhysical(0, 0, 0, 0, 0, -0.1f, 0, &pad));
    assert(!gw_SlippiPad_FromPhysical(0, 0, 0, 0, 0, 0, 1.1f, &pad));
    return 0;
}
