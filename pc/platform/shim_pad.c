/* PAD shims.
 *
 * Aurora implements the pad layer on SDL3, so most of these forward. PADStatus is 16 bytes in
 * both trees under TARGET_PC, so the arrays the game passes cross unchanged. */
#include "gw.h"

#include <dolphin/pad.h>

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

int gw_PADInit(void) {
  int ret;

  /* Claim the adapter before Aurora brings SDL's joystick subsystem up. SDL's HIDAPI GameCube
   * driver opens the same device, and whichever side gets there first locks the other out. */
  gw_gc_adapter_init();
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

int gw_PADRead(void *status) {
  PADStatus *st = (PADStatus *)status;
  int ret = (int)PADRead(st);

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
  gw_gc_adapter_read(st);
  gw_gc_adapter_diag();

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
  {
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
    if (pid == GetCurrentProcessId() && !gw_gc_adapter_present()) {
      if (GetAsyncKeyState('W') & 0x8000) { sy += 80; stick = 1; }
      if (GetAsyncKeyState('S') & 0x8000) { sy -= 80; stick = 1; }
      if (GetAsyncKeyState('A') & 0x8000) { sx -= 80; stick = 1; }
      if (GetAsyncKeyState('D') & 0x8000) { sx += 80; stick = 1; }
      if (GetAsyncKeyState('J') & 0x8000) { btn |= PAD_BUTTON_A; held = 1; }
      if (GetAsyncKeyState('K') & 0x8000) { btn |= PAD_BUTTON_B; held = 1; }
      if (GetAsyncKeyState(VK_RETURN) & 0x8000) { btn |= PAD_BUTTON_START; held = 1; }

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
    if (!gw_gc_adapter_present()) {
      st[PAD_CHAN0].err = 0;
    }
  }

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
