/* Shared report decoding for Windows and Linux. Included after calibration state. */
#define GC_PAYLOAD_SIZE 37
#define GC_PORTS 4

/* Adapter report bits. This is the adapter's own encoding, not the console's PADStatus bits. */
#define GC_BTN_A 0x01
#define GC_BTN_B 0x02
#define GC_BTN_X 0x04
#define GC_BTN_Y 0x08
#define GC_BTN_DLEFT 0x10
#define GC_BTN_DRIGHT 0x20
#define GC_BTN_DDOWN 0x40
#define GC_BTN_DUP 0x80

#define GC_BTN2_START 0x01
#define GC_BTN2_Z 0x02
#define GC_BTN2_R 0x04
#define GC_BTN2_L 0x08

static u8 gw_gc_trigger(unsigned char raw, unsigned char rest) {
  int span;
  int v;

  if (rest >= GW_GC_TRIG_DEAD) {
    return 0;
  }
  span = 255 - (int)rest;
  v = ((int)raw - (int)rest) * 255 / span;
  if (v < 0) {
    v = 0;
  }
  if (v > 255) {
    v = 255;
  }
  return (u8)v;
}

/* Centre a raw 0..255 axis on its captured origin and clamp into the console's s8 range. */
static s8 gw_gc_axis(unsigned char raw, unsigned char origin) {
  int v = (int)raw - (int)origin;
  if (v > 127) {
    v = 127;
  }
  if (v < -128) {
    v = -128;
  }
  return (s8)v;
}

static int gw_gc_decode(const unsigned char snap[37], void *status, int recal) {
  PADStatus *st = status;
  int chan, any = 0;
  for (chan = 0; chan < GC_PORTS; ++chan) {
    const unsigned char *p = &snap[1 + chan * 9];
    const int type = p[0] >> 4;
    u16 btn = 0;
    unsigned char b1;
    unsigned char b2;

    if (type != 1 && type != 2) {
      /* Nothing plugged into this adapter port: leave the channel untouched so Aurora's own
       * devices and the keyboard overlay can still own it. */
      gw_gc_have_origin[chan] = 0;
      continue;
    }

    if (!gw_gc_have_origin[chan] || recal) {
      gw_gc_origin[chan][0] = p[3];
      gw_gc_origin[chan][1] = p[4];
      gw_gc_origin[chan][2] = p[5];
      gw_gc_origin[chan][3] = p[6];
      gw_gc_trig_rest[chan][0] = p[7];
      gw_gc_trig_rest[chan][1] = p[8];
      /* Anything held at the moment of calibration is treated as stuck and masked out. Hold
       * nothing while this runs; the hotkey exists so it can be repeated deliberately. */
      gw_gc_btn_stuck[chan][0] = p[1];
      gw_gc_btn_stuck[chan][1] = p[2];
      gw_gc_have_origin[chan] = 1;
      gw_log("gw: gc adapter: ch%d calibrated -- stick (%u,%u) c (%u,%u) triggers (%u,%u)%s%s",
             chan, p[3], p[4], p[5], p[6], p[7], p[8],
             p[7] >= GW_GC_TRIG_DEAD ? " [L has no travel, disabled]" : "",
             p[8] >= GW_GC_TRIG_DEAD ? " [R has no travel, disabled]" : "");
      if (p[1] != 0 || p[2] != 0) {
        gw_log("gw: gc adapter: ch%d has buttons held at rest (b1=%02X b2=%02X); masking them",
               chan, p[1], p[2]);
      }
    }

    b1 = (unsigned char)(p[1] & ~gw_gc_btn_stuck[chan][0]);
    b2 = (unsigned char)(p[2] & ~gw_gc_btn_stuck[chan][1]);

    if (b1 & GC_BTN_A) { btn |= PAD_BUTTON_A; }
    if (b1 & GC_BTN_B) { btn |= PAD_BUTTON_B; }
    if (b1 & GC_BTN_X) { btn |= PAD_BUTTON_X; }
    if (b1 & GC_BTN_Y) { btn |= PAD_BUTTON_Y; }
    if (b1 & GC_BTN_DLEFT) { btn |= PAD_BUTTON_LEFT; }
    if (b1 & GC_BTN_DRIGHT) { btn |= PAD_BUTTON_RIGHT; }
    if (b1 & GC_BTN_DDOWN) { btn |= PAD_BUTTON_DOWN; }
    if (b1 & GC_BTN_DUP) { btn |= PAD_BUTTON_UP; }
    if (b2 & GC_BTN2_START) { btn |= PAD_BUTTON_START; }
    if (b2 & GC_BTN2_Z) { btn |= PAD_TRIGGER_Z; }
    if (b2 & GC_BTN2_R) { btn |= PAD_TRIGGER_R; }
    if (b2 & GC_BTN2_L) { btn |= PAD_TRIGGER_L; }

    /* The status array lives in game memory, so the u16 button field is written big-endian;
     * every other field is a single byte and crosses unchanged. */
    gw_w16(&st[chan].button, btn);
    st[chan].stickX = gw_gc_axis(p[3], gw_gc_origin[chan][0]);
    st[chan].stickY = gw_gc_axis(p[4], gw_gc_origin[chan][1]);
    st[chan].substickX = gw_gc_axis(p[5], gw_gc_origin[chan][2]);
    st[chan].substickY = gw_gc_axis(p[6], gw_gc_origin[chan][3]);
    st[chan].triggerLeft = gw_gc_trigger(p[7], gw_gc_trig_rest[chan][0]);
    st[chan].triggerRight = gw_gc_trigger(p[8], gw_gc_trig_rest[chan][1]);
    st[chan].analogA = 0;
    st[chan].analogB = 0;
    st[chan].err = 0;
    any = 1;
  }

  return any;
}
