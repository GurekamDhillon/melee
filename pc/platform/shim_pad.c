/* PAD shims.
 *
 * Aurora implements the pad layer on SDL3, so most of these forward. PADStatus is 16 bytes in
 * both trees under TARGET_PC, so the arrays the game passes cross unchanged. */
#include "gw.h"

#include <dolphin/pad.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

int gw_PADInit(void) { return (int)PADInit(); }

int gw_PADRead(void *status) {
  PADStatus *st = (PADStatus *)status;
  int ret = (int)PADRead(st);

  /* TARGET_PC test controls: keyboard overlay on channel 0 (easy to remove).
   * W/A/S/D drive the analog stick (up/left/down/right), J = A, K = B, Enter = Start.
   * The game reads this array big-endian (see gw.h), so the u16 button field is
   * byte-swapped here; stick and err are single bytes and cross unchanged. Only the
   * fields the keys actually provide are touched, so a real controller on channel 0
   * keeps its own values; while any key is held the channel is marked connected
   * (err = 0) so HSD_PadRenewMasterStatus accepts the synthesized status. */
  {
    u16 btn = gw_r16(&st[PAD_CHAN0].button);
    int held = 0;
    int stick = 0;
    s8 sx = 0;
    s8 sy = 0;

    if (GetAsyncKeyState('W') & 0x8000) { sy += 80; stick = 1; }
    if (GetAsyncKeyState('S') & 0x8000) { sy -= 80; stick = 1; }
    if (GetAsyncKeyState('A') & 0x8000) { sx -= 80; stick = 1; }
    if (GetAsyncKeyState('D') & 0x8000) { sx += 80; stick = 1; }
    if (GetAsyncKeyState('J') & 0x8000) { btn |= PAD_BUTTON_A; held = 1; }
    if (GetAsyncKeyState('K') & 0x8000) { btn |= PAD_BUTTON_B; held = 1; }
    if (GetAsyncKeyState(VK_RETURN) & 0x8000) { btn |= PAD_BUTTON_START; held = 1; }

    if (held || stick) {
      if (stick) {
        st[PAD_CHAN0].stickX = sx;
        st[PAD_CHAN0].stickY = sy;
      }
      gw_w16(&st[PAD_CHAN0].button, btn);
      st[PAD_CHAN0].err = 0;
      gw_log("gw_PADRead: kbd overlay held=%d btn=%04x stick=%d sx=%d sy=%d", held, btn, stick, sx, sy); /* TEMP probe */
    }
  }

  return ret;
}

int gw_PADReset(int mask) { return (int)PADReset((u32)mask); }

int gw_PADRecalibrate(int mask) { return (int)PADRecalibrate((u32)mask); }

void gw_PADSetSpec(int spec) { PADSetSpec((u32)spec); }

void gw_PADControlMotor(int chan, int cmd) { PADControlMotor((u32)chan, (u32)cmd); }

void gw_PADClamp(void *status) { PADClamp((PADStatus *)status); }

/* Aurora does not model the GameCube's serial sampling rate; SDL3 pads are polled by Aurora. */
void gw_PADSetSamplingRate(int rate) {
  (void)rate;
  GW_STUB();
}
