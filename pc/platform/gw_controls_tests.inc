#define CTL_CHECK(...) do { if (!(__VA_ARGS__)) return __LINE__; } while (0)
#include <stdio.h>
#include <string.h>
#include "gw_controls_model.h"
#include "gw_slippi_pad.h"

static int controls_model_check(void) {
    GcMap m, n;
    GcIntent out;
    unsigned char raw[GC_PHYSICAL] = {0}, wire[8];
    char text[512];
    unsigned held = 0;
    gc_map_default(&m);
    raw[0] = 255;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.buttons == 0x100);
    m.dest[0] |= 1u << 2;
    m.dest[1] = 1u << 2;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.buttons == (0x100 | 0x400));
    raw[0] = 0; raw[1] = 255;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.buttons == 0x400);
    gc_map_default(&m);
    CTL_CHECK(gc_map_bind(&m, 1, 0, 0) == 1);
    CTL_CHECK(m.dest[1] == 1u && m.dest[0] == 2u);
    CTL_CHECK(gc_map_bind(&m, 0, 0, 1) == 1);
    CTL_CHECK(m.dest[0] == 3u && m.dest[1] == 1u);
    gc_map_encode(&m, text, sizeof text);
    CTL_CHECK(gc_map_decode(&n, text));
    CTL_CHECK(memcmp(&m, &n, sizeof m) == 0);
    CTL_CHECK(!gc_map_decode(&n, ""));
    CTL_CHECK(!gc_map_decode(&n, "1 broken"));
    memset(raw, 0, sizeof raw);
    gc_map_default(&m);
    raw[26] = 80; raw[34] = 90;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.sx == 80 && out.l == 90);
    m.swap = 1;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.cx == 80 && out.sx == 0);
    m.analog_off = 1;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.l == 0);
    m.swap = 0; m.analog_off = 0; m.dz[0] = 25;
    raw[26] = 19;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.sx == 0);
    raw[26] = 80; m.shield = 80; raw[6] = 255;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.l == 80 && !(out.buttons & 0x40));
    raw[34] = 0;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.l == 80);
    gc_intent_wire(&out, wire);
    CTL_CHECK(memcmp(wire, (unsigned char[]){0,0,80,0,0,0,80,0}, 8) == 0);
    {
        GwSlippiPad gc;
        CTL_CHECK(gw_SlippiPad_FromPhysical(0,80,0,0,0,80.0f/140.0f,0,&gc));
        CTL_CHECK(memcmp(wire, gc.bytes, 8) == 0);
    }
    CTL_CHECK(!gc_map_restore_hold(1, 100, &held));
    CTL_CHECK(!gc_map_restore_hold(1, 2099, &held));
    CTL_CHECK(gc_map_restore_hold(1, 2100, &held));
    CTL_CHECK(!gc_map_restore_hold(1, 2200, &held));
    CTL_CHECK(!gc_map_restore_hold(0, 2300, &held));
    CTL_CHECK(!gc_map_restore_hold(1, 2400, &held));

    /* Every destination: button, direction, and analog, including many-to-one
     * magnitude selection and cancellation of opposite directions. */
    for (int target = 0; target < GC_TARGETS; ++target) {
        memset(raw, 0, sizeof raw); gc_map_default(&m);
        memset(m.dest, 0, sizeof m.dest); m.dest[0] = 1u << target; raw[0] = 255;
        gc_map_apply(&m, raw, &out);
        if (target < 12) CTL_CHECK(out.buttons == gc_bits[target]);
        else if (target < 20) {
            int xy[4] = {out.sx,out.sy,out.cx,out.cy};
            CTL_CHECK(xy[(target-12)/2] == ((target & 1) ? -80 : 80));
        } else CTL_CHECK((target == 20 ? out.l : out.r) == 140);
    }
    memset(raw, 0, sizeof raw); gc_map_default(&m);
    raw[26] = 30; raw[27] = 30; raw[30] = 60; raw[32] = 20;
    m.dz[0] = 25; m.dz[1] = 30;
    gc_map_apply(&m, raw, &out);
    CTL_CHECK(out.sx == 0 && out.cx == 60 && out.cy == 0);
    m.dest[30] = 1u << 12; raw[26] = 40; raw[27] = 0;
    gc_map_apply(&m, raw, &out); CTL_CHECK(out.sx == 60);
    raw[26] = 127; raw[30] = 0;
    gc_map_apply_range(&m, raw, &out, 127); CTL_CHECK(out.sx == 127);
    m.shield = 140; raw[7] = 255;
    gc_map_apply(&m, raw, &out); CTL_CHECK(out.r == 140 && (out.buttons & 0x20));
    m.analog_off = 1; m.shield = 0; raw[35] = 180;
    gc_map_apply(&m, raw, &out); CTL_CHECK(out.r == 0 && (out.buttons & 0x20));
    m.rumble = 0; gc_map_encode(&m, text, sizeof text);
    CTL_CHECK(gc_map_decode(&n, text) && !n.rumble);
    CTL_CHECK(!gc_map_encode(&m, text, 8));
    n = m; CTL_CHECK(!gc_map_decode(&n, "2 0 0 0 -1 -1 1"));
    CTL_CHECK(!memcmp(&m, &n, sizeof m));
    CTL_CHECK(gc_map_bind(&m, -1, 0, 0) == -1);
    CTL_CHECK(gc_map_bind(&m, 0, 22, 0) == -1);
    {
        char a[192], b[192];
        gc_device_identity(a, sizeof a, 1, 0, NULL, NULL, 0, 0);
        gc_device_identity(b, sizeof b, 1, 1, NULL, NULL, 0, 0);
        CTL_CHECK(strcmp(a,b));
        gc_device_identity(a, sizeof a, 0, 0, "1234", "Pad", 1, 2);
        gc_device_identity(b, sizeof b, 0, 3, "1234", "Pad", 1, 2);
        CTL_CHECK(!strcmp(a,b)); /* SDL identity is independent of game port. */
        gc_device_identity(a, sizeof a, 0, 0, "0000", "Pad\nName", 1, 2);
        gc_device_identity(b, sizeof b, 0, 1, NULL, "Pad\nName", 1, 2);
        CTL_CHECK(!strcmp(a,b) && !strchr(a,'\n'));
        gc_device_identity(b, sizeof b, 0, 1, NULL, "Pad Name", 1, 3);
        CTL_CHECK(strcmp(a,b));
    }
    held = 0;
    CTL_CHECK(!gc_map_restore_hold(1, UINT32_MAX-1000u, &held));
    CTL_CHECK(gc_map_restore_hold(1, 1000u, &held));

    return 0;
}

#undef CTL_CHECK

