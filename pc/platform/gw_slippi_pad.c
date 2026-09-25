#include "gw_slippi_pad.h"
#include <math.h>
#include <string.h>

uint8_t gw_SlippiPad_TriggerByte(float value) {
    if (!(value >= 0.0f)) return 0;
    if (value >= 1.0f) return 140;
    return (uint8_t) floorf(value * 140.0f + 0.5f);
}

int gw_SlippiPad_FromPhysical(uint16_t buttons, int8_t sx, int8_t sy, int8_t cx, int8_t cy,
                              float l, float r, GwSlippiPad *out) {
    if (!out || !isfinite(l) || !isfinite(r) || l < 0.0f || l > 1.0f ||
        r < 0.0f || r > 1.0f) return 0;
    out->bytes[0] = (uint8_t) (buttons >> 8);
    out->bytes[1] = (uint8_t) buttons;
    out->bytes[2] = (uint8_t) sx;
    out->bytes[3] = (uint8_t) sy;
    out->bytes[4] = (uint8_t) cx;
    out->bytes[5] = (uint8_t) cy;
    out->bytes[6] = gw_SlippiPad_TriggerByte(l);
    out->bytes[7] = gw_SlippiPad_TriggerByte(r);
    return 1;
}

int gw_SlippiPad_ToRb(const GwSlippiPad *pad, GwRbInput *out) {
    if (!pad || !out) return 0;
    memset(out, 0, sizeof *out);
    out->buttons = ((uint16_t) pad->bytes[0] << 8) | pad->bytes[1];
    out->raw[0] = (int8_t) pad->bytes[2];
    out->raw[1] = (int8_t) pad->bytes[3];
    out->raw[2] = (int8_t) pad->bytes[4];
    out->raw[3] = (int8_t) pad->bytes[5];
    out->pad_l = pad->bytes[6];
    out->pad_r = pad->bytes[7];
    out->is_raw = out->present = out->confirmed = 1;
    return 1;
}

int gw_SlippiPad_OnlineFrame(int slp_frame) {
    return slp_frame >= -123 ? slp_frame + 124 : -1;
}

int gw_SlippiPad_SlpFrame(int online_frame) {
    return online_frame >= 1 ? online_frame - 124 : -1;
}

int gw_SlippiPad_SampleFrame(int slp_frame, int delay) {
    int applied = gw_SlippiPad_OnlineFrame(slp_frame);
    return delay >= 0 && applied > delay ? applied - delay : -1;
}
