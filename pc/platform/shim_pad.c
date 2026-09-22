/* PAD shims.
 *
 * Aurora implements the pad layer on SDL3, so most of these forward. PADStatus is 16 bytes in
 * both trees under TARGET_PC, so the arrays the game passes cross unchanged. */
#include "gw.h"
#include "gw_overlay.h"

#include <dolphin/pad.h>

#include <stdio.h>
#include <stdlib.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

/* Raw WinUSB GameCube adapter (gc_adapter.c). It takes precedence over Aurora's SDL pad path
 * when present, because SDL's gamepad layer remaps this adapter's buttons incorrectly and
 * rescales the analog axes away from the ranges Melee is calibrated against. */
extern int gw_gc_adapter_init(void);
extern int gw_gc_adapter_present(void);
extern int gw_gc_adapter_read(void *status);
extern void gw_gc_adapter_rumble(int chan, int on);
extern void gw_gc_adapter_diag(void);
extern void gw_gc_adapter_recalibrate(void);

/* MELEE_INPUT pins this window to one input device - two copies of the game on one machine
 * (netplay testing) must not both read every controller:
 *   gc        the GameCube controller only (the raw adapter, else SDL's pads); keys do nothing
 *   keyboard  the keyboard only, on channel 0: the adapter is never opened and SDL's pads are
 *             dropped; the full play mapping below applies
 *   (unset)   everything, as before. */
enum { GW_INPUT_ANY, GW_INPUT_GC, GW_INPUT_KEYBOARD };
static int gw_input_mode(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_INPUT");
    cached = GW_INPUT_ANY;
    if (v != NULL && (v[0] == 'g' || v[0] == 'G')) {
      cached = GW_INPUT_GC;
    } else if (v != NULL && (v[0] == 'k' || v[0] == 'K')) {
      cached = GW_INPUT_KEYBOARD;
    }
    if (cached != GW_INPUT_ANY) {
      gw_log("gw: input pinned to the %s", cached == GW_INPUT_GC ? "GameCube controller" : "keyboard");
    }
  }
  return cached;
}

int gw_PADInit(void) {
  int ret;

  /* Claim the adapter before Aurora brings SDL's joystick subsystem up. SDL's HIDAPI GameCube
   * driver opens the same device, and whichever side gets there first locks the other out. */
  if (gw_input_mode() != GW_INPUT_KEYBOARD) {
    gw_gc_adapter_init();
  }
  ret = (int)PADInit();
  return ret;
}

/* TEMP DIAG (remove once the input path is settled): report what SDL actually enumerated, with
 * vid/pid, so a GameCube adapter in Wii U/Switch mode (Nintendo WUP-028, vid 057e pid 0337) can
 * be told apart from a generic HID gamepad. Also sample the raw analog values the pad layer is
 * handing the game, which is what decides whether SDL's own scaling is good enough for Melee. */
static int gw_pad_diag_on(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_PAD_DIAG");
    cached = (v != NULL && v[0] == '1') ? 1 : 0;
  }
  return cached;
}

/* MELEE_PAD_IGNORE_ADAPTER=1 drops the raw GC adapter's contribution, so only the
 * script/live/keyboard input drives the pad. Useful when a plugged-in controller drifts or
 * holds a button and would otherwise fight the scripted input. */
static int gw_input_mode(void);
static int gw_pad_ignore_adapter(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_PAD_IGNORE_ADAPTER");
    cached = (v != NULL && v[0] == '1') ? 1 : 0;
    if (gw_input_mode() == 2 /* keyboard */) {
      cached = 1;
    }
  }
  return cached;
}

void gw_diag_pads(void) {
  static int reported;

  if (!gw_pad_diag_on()) {
    return;
  }
  static int frames;
  u32 count;
  u32 i;

  if (!reported) {
    reported = 1;
    count = PADCount();
    gw_log("gw: DIAG pads: PADCount=%u", count);
    for (i = 0; i < count; ++i) {
      const char *name = PADGetNameForControllerIndex(i);
      s32 port = -1;
      u32 vid = 0;
      u32 pid = 0;
      if (i < 4u) {
        PADGetVidPid(i, &vid, &pid);
        port = PADGetIndexForPort(i);
      }
      gw_log("gw: DIAG pads: [%u] name=[%s] vid=%04X pid=%04X portIndex=%d", i,
             name != NULL ? name : "(null)", vid, pid, port);
    }
    for (i = 0; i < 4u; ++i) {
      const char *pn = PADGetName(i);
      gw_log("gw: DIAG pads: port%u name=[%s]", i, pn != NULL ? pn : "(none)");
    }
  }

  (void)frames;
}

/* Dump each channel's analog values as the game will see them, roughly twice a second. The
 * status array lives in game memory, so the u16 button field must be read through gw_r16; the
 * stick, substick and trigger fields are single bytes and cross unchanged. GX pads centre the
 * main stick near 0 here (PADClamp output) with roughly +/-80 of usable travel. */
void gw_diag_pad_values(const void *status) {
  static int frames;
  const PADStatus *st = (const PADStatus *)status;
  int c;

  if (!gw_pad_diag_on() || st == NULL || (++frames % 30) != 0) {
    return;
  }
  for (c = 0; c < 4; ++c) {
    if (st[c].err != 0 && c != 0) {
      continue;
    }
    gw_log("gw: DIAG pads: ch%d err=%d stick=(%d,%d) cstick=(%d,%d) trig=(%u,%u) btn=%04X", c,
           (int)st[c].err, (int)st[c].stickX, (int)st[c].stickY, (int)st[c].substickX,
           (int)st[c].substickY, (unsigned)st[c].triggerLeft, (unsigned)st[c].triggerRight,
           (unsigned)gw_r16(&st[c].button));
  }
}

/* MELEE_PAD_SCRIPT (.txt or .lua), MELEE_PAD_LIVE and gd.input: gw_script_pad.c. */
#include "gw_script.h"

/* L + R + Y + X + Start recalibrates every controller - ON RELEASE, not on press.
 *
 * Recalibrating samples the sticks' CURRENT position and calls that neutral, so doing it while
 * the combo is still held would latch in whatever the thumbs happen to be resting on. Waiting
 * for every button in the combo to come up means the sticks are back at rest by the time the
 * origin is taken, which is the whole point of the gesture. */
#define GW_PAD_RESET_COMBO (0x0040u | 0x0020u | 0x0800u | 0x0400u | 0x1000u) /* L R Y X Start */

static int gw_pad_reset_armed;

static void gw_pad_reset_combo(PADStatus *st) {
  int ch;
  int held = 0;
  for (ch = 0; ch < 4; ++ch) {
    if (st[ch].err == 0 &&
        (gw_r16(&st[ch].button) & GW_PAD_RESET_COMBO) == GW_PAD_RESET_COMBO) {
      held = 1;
      break;
    }
  }
  if (held) {
    gw_pad_reset_armed = 1;
    return;
  }
  if (gw_pad_reset_armed) {
    gw_pad_reset_armed = 0;
    gw_log("gw: pad: L+R+Y+X+Start released - recalibrating all four channels");
    PADRecalibrate(0xF0000000u);
    gw_Overlay_Toast("controllers recalibrated");
  }
}
/* MELEE_PROFILE_FRAMES: where gw_PADRead spends its time, summed per frame (read and reset by
 * shim_vi.c). aurora = Aurora's PADRead (SDL), adapter = the raw GameCube adapter read, rest = the
 * overlays, keyboard and script. */
static long long gw_pad_prof_t0;
double gw_pad_prof_aurora_ms, gw_pad_prof_adapter_ms, gw_pad_prof_rest_ms;
uint32_t gw_pad_prof_calls;
static double gw_pad_prof_ms(long long a, long long b) {
  static double freq;
  if (freq == 0.0) {
    LARGE_INTEGER f;
    QueryPerformanceFrequency(&f);
    freq = (double)f.QuadPart;
  }
  return (double)(b - a) * 1000.0 / freq;
}
static long long gw_pad_prof_now(void) {
  LARGE_INTEGER t;
  QueryPerformanceCounter(&t);
  return t.QuadPart;
}

int gw_PADRead(void *status) {
  PADStatus *st = (PADStatus *)status;
  long long tp0 = gw_pad_prof_now(), tp1, tp2;
  int ret = (int)PADRead(st);
  tp1 = gw_pad_prof_now();

  /* Aurora fills every channel's PADStatus natively, but the game reads the array big-endian
   * (gw.h). Swap each channel's u16 button into big-endian order before the overlays below touch
   * it, so channels nothing drives (and 1-3) keep a value the game can read, the adapter's and
   * overlays' gw_w16 writes stay correct, and the keyboard overlay's gw_r16 read sees big-endian.
   * stick/substick/trigger/analog/err are single bytes and cross unchanged; extButton is not
   * swapped because the game never reads it (only Aurora's own pad.cpp touches it). */
  for (int i = 0; i < PAD_CHANMAX; ++i) {
    st[i].button = gw_bswap16(st[i].button);
  }
  if (gw_input_mode() == GW_INPUT_KEYBOARD) {
    /* keyboard only: whatever SDL enumerated plays no part */
    for (int i = 0; i < PAD_CHANMAX; ++i) {
      memset(&st[i], 0, sizeof st[i]);
      st[i].err = (s8)-1; /* PAD_ERR_NO_CONTROLLER */
    }
  }

  /* F9 re-runs controller calibration: the resting stick and trigger positions are re-sampled
   * and any button held at that moment is masked as stuck. Worn triggers and trigger plugs drift,
   * so this needs to be repeatable without restarting the game. Gated to the foreground window,
   * like the keyboard overlay below, so it cannot fire while the game is not focused. */
  if (gw_gc_adapter_present()) {
    static int f9_was_down;
    int f9_down = 0;
    HWND fg = GetForegroundWindow();
    DWORD fgpid = 0;

    if (fg != NULL) {
      GetWindowThreadProcessId(fg, &fgpid);
    }
    if (fgpid == GetCurrentProcessId()) {
      f9_down = (GetAsyncKeyState(VK_F9) & 0x8000) != 0;
    }
    if (f9_down && !f9_was_down) {
      gw_gc_adapter_recalibrate();
    }
    f9_was_down = f9_down;
  }

  /* Overwrite whatever SDL produced for any channel the raw adapter is driving. Channels with
   * nothing plugged into the adapter are left as Aurora filled them. */
  tp2 = gw_pad_prof_now();
  if (!gw_pad_ignore_adapter()) {
    gw_gc_adapter_read(st);
    gw_gc_adapter_diag();
  }
  {
    extern void gw_inprof_poll(void); /* shim_vi.c, MELEE_INPUT_PROFILE */
    gw_inprof_poll();
  }
  gw_pad_prof_aurora_ms += gw_pad_prof_ms(tp0, tp1);
  gw_pad_prof_adapter_ms += gw_pad_prof_ms(tp2, gw_pad_prof_now());
  gw_pad_prof_t0 = gw_pad_prof_now();

  gw_diag_pads();
  gw_diag_pad_values(st);

  /* TARGET_PC test controls: keyboard overlay on channel 0 (easy to remove).
   * W/A/S/D drive the analog stick (up/left/down/right), J = A, K = B, Enter = Start.
   * The game reads this array big-endian (see gw.h), so the u16 button field is
   * byte-swapped here; stick and err are single bytes and cross unchanged. Only the
   * fields the keys actually provide are touched, so a real controller on channel 0
   * keeps its own values; the channel is always marked connected (err = 0) so
   * HSD_PadRenewMasterStatus accepts the synthesized status and the pad never appears
   * to disconnect while idle. */
  /* typing a room code (gw_netplay.c): keys are text, not a controller, for the moment */
  extern int gw_TextEntryUntil;
  if ((int) (GetTickCount() - (DWORD) gw_TextEntryUntil) < 0) {
    /* nothing: the keyboard mapping stands down */
  } else if (gw_input_mode() == GW_INPUT_KEYBOARD) {
    /* The full play mapping (keyboard-pinned windows). Focus-gated like the overlay below.
     *   W A S D     control stick (Left Shift: half tilt, for walking and tilts)
     *   arrow keys  C-stick
     *   J = A   K = B   I or Space = X (jump)   O = Y (jump)
     *   L = R (shield)   U = L (shield)   ; = Z (grab)
     *   Enter = Start   T F G H = D-pad up/left/down/right */
    HWND fg = GetForegroundWindow();
    DWORD pid = 0;
    PADStatus *k = &st[PAD_CHAN0];
    u16 btn = 0;
    int sx = 0, sy = 0, cx = 0, cy = 0;

    if (fg != NULL) GetWindowThreadProcessId(fg, &pid);
    memset(k, 0, sizeof *k);
    if (pid == GetCurrentProcessId()) {
#define GW_KEY(vk) ((GetAsyncKeyState(vk) & 0x8000) != 0)
      int tilt = GW_KEY(VK_LSHIFT) ? 40 : 80;
      if (GW_KEY('W')) sy += 1;
      if (GW_KEY('S')) sy -= 1;
      if (GW_KEY('A')) sx -= 1;
      if (GW_KEY('D')) sx += 1;
      if (GW_KEY(VK_UP)) cy += 1;
      if (GW_KEY(VK_DOWN)) cy -= 1;
      if (GW_KEY(VK_LEFT)) cx -= 1;
      if (GW_KEY(VK_RIGHT)) cx += 1;
      if (GW_KEY('J')) btn |= PAD_BUTTON_A;
      if (GW_KEY('K')) btn |= PAD_BUTTON_B;
      if (GW_KEY('I') || GW_KEY(VK_SPACE)) btn |= PAD_BUTTON_X;
      if (GW_KEY('O')) btn |= PAD_BUTTON_Y;
      if (GW_KEY('L')) { btn |= PAD_TRIGGER_R; k->triggerRight = 140; }
      if (GW_KEY('U')) { btn |= PAD_TRIGGER_L; k->triggerLeft = 140; }
      if (GW_KEY(VK_OEM_1)) btn |= PAD_TRIGGER_Z;
      if (GW_KEY(VK_RETURN)) btn |= PAD_BUTTON_START;
      if (GW_KEY('T')) btn |= PAD_BUTTON_UP;
      if (GW_KEY('G')) btn |= PAD_BUTTON_DOWN;
      if (GW_KEY('F')) btn |= PAD_BUTTON_LEFT;
      if (GW_KEY('H')) btn |= PAD_BUTTON_RIGHT;
#undef GW_KEY
      /* a diagonal is the same distance from centre as a cardinal (0.7 of each axis) */
      k->stickX = (s8)(sx * (sy != 0 ? tilt * 7 / 10 : tilt));
      k->stickY = (s8)(sy * (sx != 0 ? tilt * 7 / 10 : tilt));
      k->substickX = (s8)(cx * (cy != 0 ? 56 : 80));
      k->substickY = (s8)(cy * (cx != 0 ? 56 : 80));
    }
    gw_w16(&k->button, btn);
    k->err = 0; /* always connected: a keyboard does not unplug */
  } else if (gw_input_mode() == GW_INPUT_ANY) {
    u16 btn = gw_r16(&st[PAD_CHAN0].button);
    int held = 0;
    int stick = 0;
    s8 sx = 0;
    s8 sy = 0;
    HWND fg = GetForegroundWindow();
    DWORD pid = 0;

    if (fg != NULL) GetWindowThreadProcessId(fg, &pid);

    /* GetAsyncKeyState reports the GLOBAL key state, so without this gate the game
     * would react to W/A/S/D/J/K/Enter even while its window is not focused. Only
     * apply the overlay when the foreground window belongs to this exe. */
    if (pid == GetCurrentProcessId()) {
      if (GetAsyncKeyState('W') & 0x8000) { sy += 80; stick = 1; }
      if (GetAsyncKeyState('S') & 0x8000) { sy -= 80; stick = 1; }
      if (GetAsyncKeyState('A') & 0x8000) { sx -= 80; stick = 1; }
      if (GetAsyncKeyState('D') & 0x8000) { sx += 80; stick = 1; }
      if (GetAsyncKeyState('J') & 0x8000) { btn |= PAD_BUTTON_A; held = 1; }
      if (GetAsyncKeyState('K') & 0x8000) { btn |= PAD_BUTTON_B; held = 1; }
      if (GetAsyncKeyState(VK_RETURN) & 0x8000) { btn |= PAD_BUTTON_START; held = 1; }
      /* F1 is the in-match C-stick toggle hotkey. It has no GameCube button, so
       * it rides the reserved pad bit 0x0080 (HSD_PAD_7) into game code, where
       * fighter.c treats that bit's press edge as the F1 toggle. */
      if (GetAsyncKeyState(VK_F1) & 0x8000) { btn |= 0x0080; held = 1; }
      if (GetAsyncKeyState(VK_UP) & 0x8000) { btn |= PAD_BUTTON_UP; held = 1; }
      if (GetAsyncKeyState(VK_DOWN) & 0x8000) { btn |= PAD_BUTTON_DOWN; held = 1; }
      if (GetAsyncKeyState(VK_LEFT) & 0x8000) { btn |= PAD_BUTTON_LEFT; held = 1; }
      if (GetAsyncKeyState(VK_RIGHT) & 0x8000) { btn |= PAD_BUTTON_RIGHT; held = 1; }

      if (stick) {
        st[PAD_CHAN0].stickX = sx;
        st[PAD_CHAN0].stickY = sy;
      }
      if (held || stick) {
        gw_w16(&st[PAD_CHAN0].button, btn);
      }
    }
    /* The keyboard bridge stands in for a physical controller on channel 0, so mark it
     * connected unconditionally: otherwise Aurora reports err = -1 whenever no key is
     * held and HSD_PadRenewMasterStatus drops the pad between inputs. With the raw adapter
     * driving the channel this is unnecessary -- it sets err itself for ports that really
     * have a controller, and forcing it here would make an empty port look connected. */
    if (!gw_gc_adapter_present() || gw_pad_ignore_adapter()) {
      st[PAD_CHAN0].err = 0;
    }
  }

/* Scripted and live input take precedence over the adapter and the keyboard overlay. */
  gw_Script_PadApply(st);

  gw_pad_reset_combo(st);

  /* Mirror what the game will actually see into the F9 panel. Reading it back out of the
   * PADStatus array later would mean byte-swapping guest memory in the overlay; doing it here,
   * once, at the one place that already knows the layout, is the cheap and correct end. */
  {
    int c;
    for (c = 0; c < 4; ++c) {
      gw_Overlay_NotePad(c, (unsigned)gw_r16(&st[c].button), (int)st[c].stickX,
                         (int)st[c].stickY);
    }
  }

  gw_pad_prof_rest_ms += gw_pad_prof_ms(tp1, tp2) + gw_pad_prof_ms(gw_pad_prof_t0, gw_pad_prof_now());
  ++gw_pad_prof_calls;
  return ret;
}

int gw_PADReset(int mask) { return (int)PADReset((u32)mask); }

int gw_PADRecalibrate(int mask) { return (int)PADRecalibrate((u32)mask); }

void gw_PADSetSpec(int spec) { PADSetSpec((u32)spec); }

void gw_PADControlMotor(int chan, int cmd) {
  /* PAD_MOTOR_STOP = 0, PAD_MOTOR_RUMBLE = 1, PAD_MOTOR_STOP_HARD = 2. */
  if (gw_gc_adapter_present()) {
    gw_gc_adapter_rumble(chan, cmd == 1);
    return;
  }
  PADControlMotor((u32)chan, (u32)cmd);
}

void gw_PADClamp(void *status) { PADClamp((PADStatus *)status); }

/* Aurora does not model the GameCube's serial sampling rate; SDL3 pads are polled by Aurora. */
void gw_PADSetSamplingRate(int rate) {
  (void)rate;
  GW_STUB();
}
