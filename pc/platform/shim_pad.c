/* PAD shims.
 *
 * Aurora implements the pad layer on SDL3, so most of these forward. PADStatus is 16 bytes in
 * both trees under TARGET_PC, so the arrays the game passes cross unchanged. */
#include "gw.h"
#include "gw_overlay.h"

#include <dolphin/pad.h>

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
extern int gw_gc_adapter_suspend(void);
extern void gw_gc_adapter_resume(void);
extern int gw_gc_adapter_suspended(void);
extern unsigned gw_gc_adapter_report_count(void);
extern int gw_Settings_Str(const char *key, char *out, int cap, const char *dflt);
extern int gw_Netplay_SessionActive(void);

/* A setting read the way MELEE_INPUT is: the environment variable wins, else the settings.cfg key
 * (the launcher and the SETTINGS screens write that file), else nothing. Returns where it came
 * from, for the log. */
static const char *gw_pad_setting(const char *env, const char *key, char *out, int cap) {
  const char *v = getenv(env);
  if (v != NULL && v[0] != '\0') {
    snprintf(out, (size_t)cap, "%s", v);
    return env;
  }
  if (gw_Settings_Str(key, out, cap, "") && out[0] != '\0') {
    return "settings.cfg";
  }
  out[0] = '\0';
  return NULL;
}

/* ---- pad diagnostics log --------------------------------------------------------------------
 * MELEE_PAD_DIAG (env) / pad_diag (settings.cfg):
 *   0        off
 *   1        (default) event log, cheap enough for every player's log: per-port source changes
 *            (adapter / sdl / keyboard / script / none = disconnected), keyboard<->controller
 *            hand-overs, adapter open/close/lost/suspend/resume and scanner retries, SDL's device
 *            list when it changes, and a 10 s summary while anyone is pressing things. Rate-capped:
 *            at most 20 lines per 10 s, then one "N suppressed" line.
 *   2        level 1 plus the old verbose dumps (every channel's values and the adapter's raw
 *            report bytes, twice a second). */
static int gw_pad_level = -1;
int gw_pad_diag_level(void) {
  if (gw_pad_level < 0) {
    char buf[16];
    const char *from = gw_pad_setting("MELEE_PAD_DIAG", "pad_diag", buf, sizeof buf);
    int lv = from != NULL ? atoi(buf) : 1;
    if (lv < 0) lv = 0;
    if (lv > 2) lv = 2;
    gw_pad_level = lv;
    gw_log("gw: pad diag level %d (%s; MELEE_PAD_DIAG / settings pad_diag = 0 off, 1 events, "
           "2 verbose)", lv, from != NULL ? from : "default");
  }
  return gw_pad_level;
}

#define GW_PADLOG_WINDOW_MS 10000u
#define GW_PADLOG_LINES 20
static SRWLOCK gw_padlog_lock = SRWLOCK_INIT;
static DWORD gw_padlog_win;
static int gw_padlog_used, gw_padlog_dropped, gw_padlog_started;

/* Emit the "suppressed" line once the window that dropped lines has closed. */
static void gw_pad_log_roll(DWORD now) {
  int dropped = 0;
  AcquireSRWLockExclusive(&gw_padlog_lock);
  if (!gw_padlog_started || now - gw_padlog_win >= GW_PADLOG_WINDOW_MS) {
    dropped = gw_padlog_dropped;
    gw_padlog_win = now;
    gw_padlog_used = 0;
    gw_padlog_dropped = 0;
    gw_padlog_started = 1;
  }
  ReleaseSRWLockExclusive(&gw_padlog_lock);
  if (dropped > 0) {
    gw_log("gw: pad: %d pad log line(s) suppressed in the last 10 s (rate cap)", dropped);
  }
}

/* Level-1 pad event line (any thread). */
void gw_pad_log(const char *fmt, ...) {
  va_list ap;
  int ok;
  if (gw_pad_diag_level() < 1) {
    return;
  }
  gw_pad_log_roll(GetTickCount());
  AcquireSRWLockExclusive(&gw_padlog_lock);
  ok = gw_padlog_used < GW_PADLOG_LINES;
  if (ok) {
    ++gw_padlog_used;
  } else {
    ++gw_padlog_dropped;
  }
  ReleaseSRWLockExclusive(&gw_padlog_lock);
  if (!ok) {
    return;
  }
  va_start(ap, fmt);
  gw_logv(fmt, ap);
  va_end(ap);
}

/* MELEE_INPUT pins this window to one input device - two copies of the game on one machine
 * (netplay testing) must not both read every controller:
 *   gc        the GameCube controller only (the raw adapter, else SDL's pads); keys do nothing
 *   keyboard  the keyboard only, on port 1: the adapter is never opened and SDL's pads are
 *             dropped; the full play mapping below applies. For tools that run a keyboard window
 *             next to a controller window (netplay_local.ps1, crash_sweep.py) - it must not grab
 *             the adapter.
 *   keyboard+ (and unset: the default for players) the keyboard and every controller together.
 *             The keyboard plays on its home port (port 1, or MELEE_KEYBOARD_PORT / settings
 *             keyboard_port = 1-4), shared with whatever controller is in that port: the device
 *             used last drives the port - press a key and the keyboard has it, move the stick or
 *             press a button and the controller has it back. Netplay plays channel 0 (port 1), so
 *             a player can switch mid-match. Other ports are controllers only. */
enum { GW_INPUT_GC = 1, GW_INPUT_KEYBOARD, GW_INPUT_KEYBOARD_PLUS };
extern int gw_gc_adapter_device_plugged(void);
extern void gw_gc_adapter_start_hotplug(void);
static int gw_input_mode(void) {
  static int cached = -1;
  if (cached < 0) {
    char v[16];
    const char *from = gw_pad_setting("MELEE_INPUT", "input", v, sizeof v);
    cached = GW_INPUT_KEYBOARD_PLUS;
    if (v[0] == 'g' || v[0] == 'G') {
      cached = GW_INPUT_GC;
    } else if ((v[0] == 'k' || v[0] == 'K') && from != NULL && strcmp(from, "MELEE_INPUT") == 0 &&
               strchr(v, '+') == NULL) {
      /* a player's setting never switches controllers off; only the env var's plain
       * "keyboard" (tools) does */
      cached = GW_INPUT_KEYBOARD;
    }
    if (cached == GW_INPUT_KEYBOARD_PLUS) {
      gw_log("gw: input: keyboard and controllers together, last used wins the port (%s)",
             from != NULL ? from : "default");
    } else {
      gw_log("gw: input pinned to the %s (%s)",
             cached == GW_INPUT_GC ? "GameCube controller" : "keyboard", from);
    }
    if (cached == GW_INPUT_KEYBOARD && gw_gc_adapter_device_plugged()) {
      gw_log("gw: input: NOTE a GameCube adapter is plugged in but this window is keyboard-only "
             "(MELEE_INPUT=keyboard), so the adapter is ignored. Use MELEE_INPUT=keyboard+ or "
             "leave it unset to play with it.");
    }
  }
  return cached;
}

/* The keyboard's home port, chosen once: MELEE_KEYBOARD_PORT / settings keyboard_port (1-4),
 * else port 1. Keyboard-only windows always use port 1. */
static int gw_keyboard_home(void) {
  static int cached = -1;
  if (cached < 0) {
    char v[8];
    const char *from = gw_pad_setting("MELEE_KEYBOARD_PORT", "keyboard_port", v, sizeof v);
    int p = from != NULL ? atoi(v) : 1;
    if (p < 1 || p > 4 || gw_input_mode() == GW_INPUT_KEYBOARD) {
      p = 1;
    }
    cached = p - 1;
    if (gw_input_mode() != GW_INPUT_GC) {
      gw_log("gw: input: keyboard plays port %d (%s)", p, from != NULL ? from : "default");
    }
  }
  return cached;
}

/* MELEE_PAD_RELEASE_ON_BLUR (env) / pad_release_on_blur (settings.cfg): 1 (default) = when the
 * window goes to the background, let go of the GameCube adapter so another program (Dolphin, a
 * second copy of the game) can use it, and take it back on focus. 0 = keep it claimed, as Dolphin
 * itself does. Never released during a netplay session. */
static int gw_pad_release_on_blur(void) {
  static int cached = -1;
  if (cached < 0) {
    char v[8];
    const char *from = gw_pad_setting("MELEE_PAD_RELEASE_ON_BLUR", "pad_release_on_blur", v,
                                      sizeof v);
    cached = from != NULL ? (v[0] != '0') : 1;
    gw_log("gw: pad: release the GC adapter when unfocused: %s (%s)", cached ? "on" : "off",
           from != NULL ? from : "default");
  }
  return cached;
}

int gw_PADInit(void) {
  int ret;

  gw_pad_diag_level();
  gw_keyboard_home();
  /* Claim the adapter before Aurora brings SDL's joystick subsystem up. SDL's HIDAPI GameCube
   * driver opens the same device, and whichever side gets there first locks the other out. */
  if (gw_input_mode() != GW_INPUT_KEYBOARD) {
    gw_pad_release_on_blur();
    gw_gc_adapter_init();
    gw_gc_adapter_start_hotplug(); /* plugged in later, or moved to another port: picked up */
  }
  ret = (int)PADInit();
  return ret;
}

/* ---- focus: release the adapter in the background ---------------------------------------- */
static int gw_focus_have = 1;
static DWORD gw_focus_lost_at;
static int gw_focus_released;
static int gw_focus_kept_logged;

/* shim_vi.c, on SDL_EVENT_WINDOW_FOCUS_LOST / _GAINED. */
void gw_pad_focus_event(int focused) {
  focused = focused != 0;
  if (focused == gw_focus_have) {
    return;
  }
  gw_focus_have = focused;
  if (!focused) {
    gw_focus_lost_at = GetTickCount();
    gw_focus_kept_logged = 0;
  }
}

/* shim_vi.c, once per frame after the events. A focus loss acts after 250 ms (an alt-tab flicker
 * does nothing); a gain acts at once. */
void gw_pad_focus_tick(void) {
  int want;
  if (gw_input_mode() == GW_INPUT_KEYBOARD || !gw_pad_release_on_blur()) {
    return;
  }
  want = 0;
  if (!gw_focus_have && GetTickCount() - gw_focus_lost_at >= 250u) {
    if (gw_Netplay_SessionActive()) {
      if (!gw_focus_kept_logged) {
        gw_focus_kept_logged = 1;
        gw_pad_log("gw: pad: window unfocused during a netplay session - GC adapter kept "
                   "(netplay needs input in the background)");
      }
    } else {
      want = 1;
    }
  }
  if (want && !gw_focus_released) {
    gw_focus_released = 1;
    gw_gc_adapter_suspend();
  } else if (!want && gw_focus_released) {
    gw_focus_released = 0;
    gw_gc_adapter_resume();
  }
}

/* MELEE_PAD_IGNORE_ADAPTER=1 drops the raw GC adapter's contribution, so only the
 * script/live/keyboard input drives the pad. Useful when a plugged-in controller drifts or
 * holds a button and would otherwise fight the scripted input. */
static int gw_pad_ignore_adapter(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_PAD_IGNORE_ADAPTER");
    cached = (v != NULL && v[0] == '1') ? 1 : 0;
    if (gw_input_mode() == GW_INPUT_KEYBOARD) {
      cached = 1;
    }
  }
  return cached;
}

/* SDL's device list: logged at level 1 whenever the count changes (plugging an Xbox pad in, the
 * adapter showing up as a generic HID gamepad), with vid/pid so a GameCube adapter in Wii U/Switch
 * mode (Nintendo WUP-028, vid 057e pid 0337) can be told apart from a generic HID gamepad. */
void gw_diag_pads(void) {
  static u32 last_count = 0xFFFFFFFFu;
  u32 count;
  u32 i;

  if (gw_pad_diag_level() < 1) {
    return;
  }
  count = PADCount();
  if (count == last_count) {
    return;
  }
  last_count = count;
  gw_pad_log("gw: pad: SDL sees %u controller(s)", count);
  for (i = 0; i < count && i < 8u; ++i) {
    const char *name = PADGetNameForControllerIndex(i);
    s32 port = -1;
    u32 vid = 0;
    u32 pid = 0;
    if (i < 4u) {
      PADGetVidPid(i, &vid, &pid);
      port = PADGetIndexForPort(i);
    }
    gw_pad_log("gw: pad:   SDL [%u] %s vid=%04X pid=%04X portIndex=%d", i,
               name != NULL ? name : "(null)", vid, pid, port);
  }
}

/* Level 2: each channel's analog values as the game will see them, roughly twice a second. The
 * status array lives in game memory, so the u16 button field must be read through gw_r16; the
 * stick, substick and trigger fields are single bytes and cross unchanged. GX pads centre the
 * main stick near 0 here (PADClamp output) with roughly +/-80 of usable travel. */
void gw_diag_pad_values(const void *status) {
  static int frames;
  const PADStatus *st = (const PADStatus *)status;
  int c;

  if (gw_pad_diag_level() < 2 || st == NULL || (++frames % 30) != 0) {
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

static int gw_this_window_focused(void) {
  HWND fg = GetForegroundWindow();
  DWORD pid = 0;
  if (fg != NULL) GetWindowThreadProcessId(fg, &pid);
  return pid == GetCurrentProcessId();
}

/* The keyboard as a controller. Focus-gated: GetAsyncKeyState reports the GLOBAL key state, so
 * without the gate the game would react to typing in another window. Returns whether any play
 * key is down (the keyboard "was used" this sample).
 *   W A S D     control stick (Left Shift: half tilt, for walking and tilts)
 *   arrow keys  C-stick
 *   J = A   K = B   I or Space = X (jump)   O = Y (jump)
 *   L = R (shield)   U = L (shield)   ; = Z (grab)
 *   Enter = Start   T F G H = D-pad up/left/down/right */
static int gw_keyboard_sample(PADStatus *k, int focused) {
  u16 btn = 0;
  int sx = 0, sy = 0, cx = 0, cy = 0;
  int any;

  memset(k, 0, sizeof *k);
  if (focused) {
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
  any = btn != 0 || sx != 0 || sy != 0 || cx != 0 || cy != 0;
  gw_w16(&k->button, btn);
  k->err = 0; /* always connected: a keyboard does not unplug */
  return any;
}

/* A controller "really used" - the rule from melee-unlocked's keyboard_and_pad (Hero88go,
 * port/runtime/host/window.cpp, GPL-2.0-or-later): any button, a stick or C-stick past the game's
 * own deadzone (23 of ~80, so a worn stick resting off centre does not lock the keyboard out), or
 * a trigger past 20. */
static int gw_pad_moved(s8 v) { return v > 23 || v < -23; }
static int gw_pad_really_used(const PADStatus *p) {
  return p->err == 0 &&
         (gw_r16(&p->button) != 0 || gw_pad_moved(p->stickX) || gw_pad_moved(p->stickY) ||
          gw_pad_moved(p->substickX) || gw_pad_moved(p->substickY) || p->triggerLeft > 20 ||
          p->triggerRight > 20);
}

/* ---- per-port source tracking (level-1 log) ---------------------------------------------- */
enum { GW_SRC_NONE, GW_SRC_ADAPTER, GW_SRC_SDL, GW_SRC_KEYBOARD, GW_SRC_SCRIPT };
static const char *const gw_src_name[] = { "none", "adapter", "sdl", "keyboard", "script" };
static int gw_src_last[PAD_CHANMAX] = { -1, -1, -1, -1 };
static int gw_src_script_hold[PAD_CHANMAX]; /* frames a port keeps the "script" label */
static int gw_kb_owner = GW_SRC_KEYBOARD;   /* who drives the keyboard's home port */

/* Last-active-wins on the keyboard's home port. owner and pad_src are GW_SRC_*; pad_src is what
 * is in the port apart from the keyboard (GW_SRC_NONE = nothing). *why is set when the owner
 * changes for a reason worth logging. */
static int gw_pad_owner_next(int owner, int keys, int pad_src, int pad_used, const char **why) {
  if (keys) {
    if (owner != GW_SRC_KEYBOARD) *why = "key pressed";
    return GW_SRC_KEYBOARD;
  }
  if (pad_src != GW_SRC_NONE && pad_used) {
    if (owner != pad_src) *why = "controller used";
    return pad_src;
  }
  if (owner != GW_SRC_KEYBOARD && pad_src == GW_SRC_NONE) {
    *why = "controller gone";
    return GW_SRC_KEYBOARD;
  }
  if (owner != GW_SRC_KEYBOARD) {
    return pad_src; /* the same controller, reached through the other path (adapter<->sdl) */
  }
  return owner;
}

/* 10 s summary: which ports were live, what drove them, which buttons were seen. */
static DWORD gw_sum_at;
static unsigned gw_sum_reports;
static u16 gw_sum_btn[PAD_CHANMAX];
static int gw_sum_used[PAD_CHANMAX];

static void gw_pad_track(const PADStatus *st, const int *src) {
  DWORD now;
  int c;
  if (gw_pad_diag_level() < 1) {
    return;
  }
  for (c = 0; c < PAD_CHANMAX; ++c) {
    if (src[c] != gw_src_last[c]) {
      if (gw_src_last[c] >= 0) {
        gw_pad_log("gw: pad: P%d %s -> %s%s", c + 1, gw_src_name[gw_src_last[c]],
                   gw_src_name[src[c]],
                   src[c] == GW_SRC_NONE ? " (disconnected)"
                                         : (gw_src_last[c] == GW_SRC_NONE ? " (connected)" : ""));
      } else if (src[c] != GW_SRC_NONE) {
        gw_pad_log("gw: pad: P%d %s (connected)", c + 1, gw_src_name[src[c]]);
      }
      gw_src_last[c] = src[c];
    }
    if (st[c].err == 0) {
      gw_sum_btn[c] |= gw_r16(&st[c].button);
      gw_sum_used[c] |= gw_pad_really_used(&st[c]);
    }
  }
  now = GetTickCount();
  if (gw_sum_at == 0) {
    gw_sum_at = now;
    gw_sum_reports = gw_gc_adapter_report_count();
    return;
  }
  if (now - gw_sum_at >= GW_PADLOG_WINDOW_MS) {
    const unsigned reports = gw_gc_adapter_report_count();
    const double hz = (double)(reports - gw_sum_reports) * 1000.0 / (double)(now - gw_sum_at);
    char line[320];
    int n = 0, any = 0;
    line[0] = '\0';
    for (c = 0; c < PAD_CHANMAX; ++c) {
      if (src[c] == GW_SRC_NONE) continue;
      any |= gw_sum_used[c];
      n += snprintf(line + n, sizeof line - (size_t)n, " | P%d %s%s btn=%04X%s", c + 1,
                    gw_src_name[src[c]],
                    (gw_input_mode() == GW_INPUT_KEYBOARD_PLUS && c == gw_keyboard_home())
                        ? "(kb home)" : "",
                    (unsigned)gw_sum_btn[c], gw_sum_used[c] ? "" : " idle");
      if (n >= (int)sizeof line) n = (int)sizeof line - 1;
    }
    /* only while someone is actually playing: an idle menu writes nothing */
    if (any) {
      gw_log("gw: pad: 10 s summary%s | adapter %s %.0f Hz", line,
             gw_gc_adapter_present() ? "open" : (gw_gc_adapter_suspended() ? "released" : "closed"),
             hz);
    }
    gw_pad_log_roll(now);
    memset(gw_sum_btn, 0, sizeof gw_sum_btn);
    memset(gw_sum_used, 0, sizeof gw_sum_used);
    gw_sum_at = now;
    gw_sum_reports = reports;
  }
}

int gw_PADRead(void *status) {
  PADStatus *st = (PADStatus *)status;
  PADStatus adp[PAD_CHANMAX];
  PADStatus before[PAD_CHANMAX];
  int src[PAD_CHANMAX];
  const int mode = gw_input_mode();
  const int focused = gw_this_window_focused();
  long long tp0 = gw_pad_prof_now(), tp1, tp2;
  int ret = (int)PADRead(st);
  int i;
  tp1 = gw_pad_prof_now();

  /* Aurora fills every channel's PADStatus natively, but the game reads the array big-endian
   * (gw.h). Swap each channel's u16 button into big-endian order before the overlays below touch
   * it, so channels nothing drives (and 1-3) keep a value the game can read, the adapter's and
   * overlays' gw_w16 writes stay correct, and the keyboard overlay's gw_r16 read sees big-endian.
   * stick/substick/trigger/analog/err are single bytes and cross unchanged; extButton is not
   * swapped because the game never reads it (only Aurora's own pad.cpp touches it). */
  for (i = 0; i < PAD_CHANMAX; ++i) {
    st[i].button = gw_bswap16(st[i].button);
  }
  /* SDL's pads play no part in a keyboard-only window, nor while the window is in the background
   * with the adapter released (every port reads disconnected unless the keyboard drives it). */
  if (mode == GW_INPUT_KEYBOARD || gw_gc_adapter_suspended()) {
    for (i = 0; i < PAD_CHANMAX; ++i) {
      memset(&st[i], 0, sizeof st[i]);
      st[i].err = (s8)-1; /* PAD_ERR_NO_CONTROLLER */
    }
  }
  for (i = 0; i < PAD_CHANMAX; ++i) {
    src[i] = st[i].err == 0 ? GW_SRC_SDL : GW_SRC_NONE;
  }

  /* F9 re-runs controller calibration: the resting stick and trigger positions are re-sampled
   * and any button held at that moment is masked as stuck. Worn triggers and trigger plugs drift,
   * so this needs to be repeatable without restarting the game. Gated to the foreground window,
   * like the keyboard below, so it cannot fire while the game is not focused. */
  if (gw_gc_adapter_present()) {
    static int f9_was_down;
    const int f9_down = focused && (GetAsyncKeyState(VK_F9) & 0x8000) != 0;
    if (f9_down && !f9_was_down) {
      gw_gc_adapter_recalibrate();
    }
    f9_was_down = f9_down;
  }

  /* The raw adapter, read into its own array: a port with a controller in the adapter replaces
   * what SDL produced for it; empty adapter ports leave SDL's value. */
  tp2 = gw_pad_prof_now();
  if (!gw_pad_ignore_adapter()) {
    memset(adp, 0, sizeof adp);
    for (i = 0; i < PAD_CHANMAX; ++i) {
      adp[i].err = (s8)-1;
    }
    gw_gc_adapter_read(adp);
    gw_gc_adapter_diag();
    for (i = 0; i < PAD_CHANMAX; ++i) {
      if (adp[i].err == 0) {
        st[i] = adp[i];
        src[i] = GW_SRC_ADAPTER;
      }
    }
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

  /* The keyboard. Keyboard-only: it is port 1. Otherwise it shares its home port with whatever
   * controller is there, and the device used last owns the port: any play key down -> keyboard;
   * the controller really used -> controller; neither -> no change. The home port is fixed
   * (gw_keyboard_home), so the keyboard never hops between ports as controllers come and go.
   * The decision uses only this machine's live devices before anything is sent to a netplay peer,
   * so rollback sees one ordinary input stream either way. */
  if (mode == GW_INPUT_KEYBOARD || mode == GW_INPUT_KEYBOARD_PLUS) {
    /* typing a room code (gw_netplay.c): keys are text, not a controller, for the moment */
    extern int gw_TextEntryUntil;
    const int typing = (int)(GetTickCount() - (DWORD)gw_TextEntryUntil) < 0;
    const int h = gw_keyboard_home();
    PADStatus kb;
    const int keys = gw_keyboard_sample(&kb, focused && !typing);

    if (mode == GW_INPUT_KEYBOARD) {
      st[h] = kb;
      src[h] = GW_SRC_KEYBOARD;
    } else {
      const int prev = gw_kb_owner;
      const char *why = NULL;
      gw_kb_owner = gw_pad_owner_next(gw_kb_owner, keys, src[h], gw_pad_really_used(&st[h]), &why);
      if (gw_kb_owner != prev && why != NULL) {
        gw_pad_log("gw: pad: P%d now driven by the %s (%s)", h + 1, gw_src_name[gw_kb_owner], why);
      }
      if (gw_kb_owner == GW_SRC_KEYBOARD) {
        st[h] = kb;
        src[h] = GW_SRC_KEYBOARD;
      }
    }
    /* F1 is the in-match C-stick toggle hotkey. It has no GameCube button, so it rides the
     * reserved pad bit 0x0080 (HSD_PAD_7) into game code, where fighter.c treats that bit's press
     * edge as the F1 toggle. A hotkey, not play input: it does not take the port over. */
    if (focused && !typing && (GetAsyncKeyState(VK_F1) & 0x8000) != 0 && st[h].err == 0) {
      gw_w16(&st[h].button, (u16)(gw_r16(&st[h].button) | 0x0080u));
    }
  }

  /* Scripted and live input take precedence over the adapter and the keyboard. */
  memcpy(before, st, sizeof before);
  gw_Script_PadApply(st);
  for (i = 0; i < PAD_CHANMAX; ++i) {
    if (memcmp(&before[i], &st[i], sizeof st[i]) != 0) {
      gw_src_script_hold[i] = 60; /* a script writing what was already there is still the script */
    } else if (gw_src_script_hold[i] > 0) {
      --gw_src_script_hold[i];
    }
    if (gw_src_script_hold[i] > 0) {
      src[i] = st[i].err == 0 ? GW_SRC_SCRIPT : GW_SRC_NONE;
    }
  }

  gw_pad_reset_combo(st);
  gw_pad_track(st, src);

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

/* ---- tests (run.sh --test) ------------------------------------------------------------------ */
#include "gw_test.h"

static int test_pad_last_active_wins(void) {
  const char *why = NULL;
  int o = GW_SRC_KEYBOARD;
  o = gw_pad_owner_next(o, 0, GW_SRC_ADAPTER, 0, &why); /* pad idle: keyboard keeps it */
  if (o != GW_SRC_KEYBOARD || why != NULL) { gw_test_fail("idle pad took the port"); return 1; }
  o = gw_pad_owner_next(o, 0, GW_SRC_ADAPTER, 1, &why);
  if (o != GW_SRC_ADAPTER || why == NULL) { gw_test_fail("used pad did not take the port"); return 1; }
  why = NULL;
  o = gw_pad_owner_next(o, 0, GW_SRC_ADAPTER, 0, &why); /* pad let go: stays the pad's */
  if (o != GW_SRC_ADAPTER || why != NULL) { gw_test_fail("pad lost the port while idle"); return 1; }
  o = gw_pad_owner_next(o, 1, GW_SRC_ADAPTER, 1, &why); /* key wins even over a moving pad */
  if (o != GW_SRC_KEYBOARD || why == NULL) { gw_test_fail("key did not take the port"); return 1; }
  why = NULL;
  o = gw_pad_owner_next(o, 0, GW_SRC_SDL, 1, &why);
  if (o != GW_SRC_SDL) { gw_test_fail("SDL pad did not take the port"); return 1; }
  why = NULL;
  o = gw_pad_owner_next(o, 0, GW_SRC_NONE, 0, &why); /* unplugged: back to the keyboard */
  if (o != GW_SRC_KEYBOARD || why == NULL) { gw_test_fail("unplug did not return the port"); return 1; }
  return 0;
}

static int test_pad_really_used_thresholds(void) {
  PADStatus p;
  memset(&p, 0, sizeof p);
  p.stickX = 23; p.substickY = -23; p.triggerLeft = 20; p.triggerRight = 20;
  if (gw_pad_really_used(&p)) { gw_test_fail("inside the deadzone counted as used"); return 1; }
  p.stickX = 24;
  if (!gw_pad_really_used(&p)) { gw_test_fail("stick 24 not counted"); return 1; }
  p.stickX = 0; p.triggerRight = 21;
  if (!gw_pad_really_used(&p)) { gw_test_fail("trigger 21 not counted"); return 1; }
  p.triggerRight = 0;
  gw_w16(&p.button, PAD_BUTTON_A);
  if (!gw_pad_really_used(&p)) { gw_test_fail("button not counted"); return 1; }
  p.err = (s8)-1;
  if (gw_pad_really_used(&p)) { gw_test_fail("disconnected pad counted"); return 1; }
  return 0;
}

void gw_pad_tests_register(void) {
  gw_test_register("pad_last_active_wins", test_pad_last_active_wins);
  gw_test_register("pad_really_used_thresholds", test_pad_really_used_thresholds);
}
