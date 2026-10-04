/* Original mapping logic; independent of SDL, Windows and guest byte order.
 * Destinations are GC inputs, not fighter actions. Never consult profiles from
 * simulation: only the local physical sample is transformed. */
#ifndef GW_CONTROLS_MODEL_H
#define GW_CONTROLS_MODEL_H
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#define GC_PHYSICAL 36
#define GC_TARGETS 22
#define GC_DEST_MASK ((1u << GC_TARGETS) - 1u)
static const unsigned short gc_bits[12] = {
    0x100,0x200,0x400,0x800,0x10,0x1000,0x40,0x20,8,4,1,2
};
static const char *const gc_labels[GC_TARGETS] = {
    "A", "B", "X", "Y", "Z", "START", "L click", "R click",
    "D-pad Up", "D-pad Down", "D-pad Left", "D-pad Right",
    "Stick Right", "Stick Left", "Stick Up", "Stick Down",
    "C-stick Right", "C-stick Left", "C-stick Up", "C-stick Down",
    "Analog L", "Analog R"
};
typedef struct GcMap {
    unsigned dest[GC_PHYSICAL];
    int swap, analog_off, shield, dz[2], rumble;
} GcMap;
typedef struct GcIntent {
    unsigned short buttons;
    signed char sx, sy, cx, cy;
    unsigned char l, r;
} GcIntent;
static void gc_device_identity(char *out, int cap, int adapter, int port,
                               const char *guid, const char *name, unsigned vid, unsigned pid) {
    int i, has_guid = 0;
    if (adapter) snprintf(out, (size_t)cap, "gc-adapter-port-%d", port+1);
    else {
        if (guid) for (i = 0; guid[i]; ++i) if (guid[i] != '0') has_guid = 1;
        if (has_guid) snprintf(out, (size_t)cap, "sdl:%s:%.128s", guid, name && name[0] ? name : "Unnamed");
        else snprintf(out, (size_t)cap, "sdl:%04x:%04x:%.128s", vid, pid, name && name[0] ? name : "Unnamed");
    }
    for (i = 0; out[i]; ++i) if (out[i] == '\r' || out[i] == '\n') out[i] = ' ';
}
static void gc_map_default(GcMap *m) {
    int i;
    memset(m, 0, sizeof *m);
    for (i = 0; i < 12; ++i) m->dest[i] = 1u << i;
    for (i = 0; i < 10; ++i) m->dest[26+i] = 1u << (12+i);
    m->dz[0] = m->dz[1] = -1;
    m->rumble = 1;
}
/* Swap the old sources of this destination onto the displaced destinations.
 * Also preserves the sources already driving the selected GC input. */
static int gc_map_bind(GcMap *m, int physical, int target, int also) {
    unsigned bit, displaced;
    int i, conflict = 0;
    if (physical < 0 || physical >= GC_PHYSICAL || target < 0 || target >= GC_TARGETS) return -1;
    bit = 1u << target;
    displaced = m->dest[physical] & ~bit;
    for (i = 0; i < GC_PHYSICAL; ++i) {
        if (i != physical && (m->dest[i] & bit)) {
            conflict = 1;
            if (!also) m->dest[i] = (m->dest[i] & ~bit) | displaced;
        }
    }
    m->dest[physical] = also ? m->dest[physical] | bit : bit;
    return conflict;
}
static void gc_map_apply_range(const GcMap *m, const unsigned char raw[GC_PHYSICAL], GcIntent *out, int range) {
    int values[GC_TARGETS] = {0}, i, t, x[4];
    memset(out, 0, sizeof *out);
    for (i = 0; i < GC_PHYSICAL; ++i) {
        for (t = 0; t < GC_TARGETS; ++t) {
            int v = raw[i];
            if (!(m->dest[i] & (1u << t))) continue;
            if (t < 12) {
                /* Axis-to-button bindings activate at half travel. */
                if (v >= (i < 26 ? 1 : i < 34 ? range/2 : 140)) values[t] = 1;
            } else {
                if (i < 26) v = v ? (t < 20 ? range : 140) : 0;
                else if (i < 34 && t >= 20) v = v * 140 / range;
                else if (i >= 34 && t < 20) v = v * range / 255;
                if (v > values[t]) values[t] = v;
            }
        }
    }
    for (i = 0; i < 12; ++i) if (values[i]) out->buttons |= gc_bits[i];
    for (i = 0; i < 4; ++i) {
        x[i] = values[12+i*2] - values[13+i*2];
        if (x[i] > 127) x[i] = 127;
        if (x[i] < -127) x[i] = -127;
        t = m->dz[i/2];
        if (t >= 0 && abs(x[i])*100 < t*range) x[i] = 0;
    }
    out->sx = (signed char)x[m->swap ? 2 : 0];
    out->sy = (signed char)x[m->swap ? 3 : 1];
    out->cx = (signed char)x[m->swap ? 0 : 2];
    out->cy = (signed char)x[m->swap ? 1 : 3];
    out->l = (unsigned char)(m->analog_off ? 0 : values[20] > 255 ? 255 : values[20]);
    out->r = (unsigned char)(m->analog_off ? 0 : values[21] > 255 ? 255 : values[21]);
    if (m->shield) {
        if (out->buttons & 0x40) {
            out->l = (unsigned char)m->shield;
            if (m->shield < 140) out->buttons &= ~0x40u;
        }
        if (out->buttons & 0x20) {
            out->r = (unsigned char)m->shield;
            if (m->shield < 140) out->buttons &= ~0x20u;
        }
    }
}
static void gc_map_apply(const GcMap *m, const unsigned char raw[GC_PHYSICAL], GcIntent *out) {
    gc_map_apply_range(m, raw, out, 80);
}
static void gc_intent_wire(const GcIntent *p, unsigned char bytes[8]) {
    bytes[0] = (unsigned char)(p->buttons >> 8); bytes[1] = (unsigned char)p->buttons;
    bytes[2] = (unsigned char)p->sx; bytes[3] = (unsigned char)p->sy;
    bytes[4] = (unsigned char)p->cx; bytes[5] = (unsigned char)p->cy;
    bytes[6] = p->l; bytes[7] = p->r;
}
static int gc_map_encode(const GcMap *m, char *out, int cap) {
    int i, n = snprintf(out, (size_t)cap, "1 %d %d %d %d %d %d", m->swap,
        m->analog_off, m->shield, m->dz[0], m->dz[1], m->rumble);
    for (i = 0; i < GC_PHYSICAL && n >= 0 && n < cap; ++i) {
        int k = snprintf(out+n, (size_t)(cap-n), " %x", m->dest[i]);
        if (k < 0 || k >= cap-n) return 0;
        n += k;
    }
    return n > 0 && n < cap;
}
static int gc_map_decode(GcMap *m, const char *text) {
    GcMap v;
    int version, n = 0, i;
    const char *p;
    memset(&v, 0, sizeof v);
    if (sscanf(text, "%d %d %d %d %d %d %d%n", &version, &v.swap,
        &v.analog_off, &v.shield, &v.dz[0], &v.dz[1], &v.rumble, &n) != 7 || version != 1) return 0;
    if ((v.swap != 0 && v.swap != 1) || (v.analog_off != 0 && v.analog_off != 1) ||
        v.shield < 0 || v.shield > 140 || v.dz[0] < -1 || v.dz[0] > 50 ||
        v.dz[1] < -1 || v.dz[1] > 50 || (v.rumble != 0 && v.rumble != 1)) return 0;
    p = text+n;
    for (i = 0; i < GC_PHYSICAL; ++i) {
        char *end;
        unsigned long bits = strtoul(p, &end, 16);
        if (end == p || bits > GC_DEST_MASK) return 0;
        v.dest[i] = (unsigned)bits;
        p = end;
    }
    while (*p == ' ') ++p;
    if (*p) return 0;
    *m = v;
    return 1;
}
/* Real elapsed milliseconds, not PADRead count or turbo logic frames. UINT32
 * wrap is well-defined; UINT32_MAX latches until release after one restore. */
static int gc_map_restore_hold(int down, unsigned now, unsigned *since) {
    if (!down) { *since = 0; return 0; }
    if (*since == UINT32_MAX) return 0;
    if (!*since) { *since = now ? now : 1; return 0; }
    if (now - *since < 2000u) return 0;
    *since = UINT32_MAX;
    return 1;
}
#endif
