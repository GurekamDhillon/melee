/* gw_script_pad.c - scripted controller input, applied once per PADRead (shim_pad.c calls
 * gw_Script_PadApply after the adapter, the keyboard overlays and gw_inprof_poll).
 *
 * Three sources, strongest last:
 *   1. MELEE_PAD_SCRIPT=<file>.txt  the original fixed-frame format, unchanged (moved here from
 *      shim_pad.c): one segment per line, "<frames> <buttons_hex> [stickX stickY [trigL trigR]]",
 *      '#' comments, each PADRead consumes one frame, the last segment holds, broadcast to the
 *      channels MELEE_PAD_CHANNELS names (default "0"). Button bits: A 0x100, B 0x200, X 0x400,
 *      Y 0x800, Start 0x1000, L 0x40, R 0x20, Z 0x10, Up 0x8, Down 0x4, Left 0x1, Right 0x2.
 *      MELEE_PAD_SCRIPT=<file>.lua instead loads a Lua input script (gw_script.c) that can wait on
 *      game state: gd.run(function() gd.wait_until(...) gd.press(1, "A", 2) end).
 *   2. MELEE_PAD_LIVE=<path>  re-read every PADRead: "<buttons_hex> [sx sy [tl tr]]" on channel 0.
 *   3. gd.input / the console "input" command: per-port claims from the first input until
 *      explicit release, task completion, script unload or console-client disconnect. A hold of
 *      N frames covers exactly N LOGIC frames (gw_Script_PadFrameConsumed, at the end of each one),
 *      then the claimed port reports connected neutral. It used to count PADReads, and the pad
 *      alarm samples every field whether a logic frame consumes the sample or not: a paused or
 *      stepped game (the Lab's export) let the hold run out between frames, so realtime read a
 *      stick+B hold as neutral B while turbo (its reads closer to the frames) did not.
 * Whatever the game finally sees is recorded for gd.pad().
 */
#include "gw.h"
#include "gw_script.h"

#include <dolphin/pad.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifndef _WIN32
#include <strings.h> /* strcasecmp; MSVC declares _stricmp in <string.h> */
#define _stricmp strcasecmp
#endif

extern void gw_pad_log(const char *fmt, ...); /* rate-limited pad transition log (shim_pad.c) */

/* ---- 1. the fixed-frame txt script ---------------------------------------------------------- */
#define GW_SCRIPT_MAX 8192
static struct {
  int frames;
  uint16_t buttons;
  int8_t sx, sy;
  uint8_t tl, tr;
} gw_script[GW_SCRIPT_MAX];
static int gw_script_count;
static int gw_script_cursor;
static int gw_script_left;
static int gw_script_loaded;
static unsigned gw_script_chan_mask = 1u; /* MELEE_PAD_CHANNELS, e.g. "0123" */
static char gw_script_lua[260];           /* MELEE_PAD_SCRIPT when it names a .lua */

static void gw_pad_script_load(void) {
  const char *path;
  FILE *f;
  char line[256];
  size_t n;

  gw_script_loaded = 1;
  {
    const char *chans = getenv("MELEE_PAD_CHANNELS");
    if (chans != NULL && chans[0] != '\0') {
      const char *c;
      gw_script_chan_mask = 0;
      for (c = chans; *c != '\0'; ++c) {
        if (*c >= '0' && *c <= '3') {
          gw_script_chan_mask |= 1u << (*c - '0');
        }
      }
      if (gw_script_chan_mask == 0) {
        gw_script_chan_mask = 1u;
      }
    }
  }
  path = getenv("MELEE_PAD_SCRIPT");
  if (path == NULL || path[0] == '\0') {
    return;
  }
  n = strlen(path);
  if (n > 4 && _stricmp(path + n - 4, ".lua") == 0) {
    snprintf(gw_script_lua, sizeof gw_script_lua, "%s", path); /* gw_script.c loads it */
    return;
  }
  f = fopen(path, "r");
  if (f == NULL) {
    gw_log("gw: pad script: cannot open %s", path);
    return;
  }
  while (fgets(line, sizeof line, f) != NULL && gw_script_count < GW_SCRIPT_MAX) {
    char *p = line;
    int frames = 0;
    unsigned int buttons = 0;
    int sx = 0, sy = 0, tl = 0, tr = 0;
    int k;

    while (*p == ' ' || *p == '\t') {
      ++p;
    }
    if (*p == '#' || *p == '\n' || *p == '\r' || *p == '\0') {
      continue;
    }
    k = sscanf(p, "%d %x %d %d %d %d", &frames, &buttons, &sx, &sy, &tl, &tr);
    if (k < 2 || frames <= 0) {
      continue;
    }
    gw_script[gw_script_count].frames = frames;
    gw_script[gw_script_count].buttons = (uint16_t)buttons;
    gw_script[gw_script_count].sx = (int8_t)sx;
    gw_script[gw_script_count].sy = (int8_t)sy;
    gw_script[gw_script_count].tl = (uint8_t)tl;
    gw_script[gw_script_count].tr = (uint8_t)tr;
    ++gw_script_count;
  }
  fclose(f);
  gw_log("gw: pad script: %d segments from %s", gw_script_count, path);
}

const char *gw_script_pad_lua_path(void) {
  if (!gw_script_loaded) {
    gw_pad_script_load();
  }
  return gw_script_lua[0] != '\0' ? gw_script_lua : NULL;
}

static unsigned gw_pad_script_apply(PADStatus *st) {
  int idx;
  int ch;
  unsigned driven = 0;

  if (!gw_script_loaded) {
    gw_pad_script_load();
  }
  if (gw_script_count == 0) {
    return 0;
  }
  if (gw_script_cursor >= gw_script_count) {
    gw_script_cursor = gw_script_count - 1;
  }
  if (gw_script_left == 0) {
    gw_script_left = gw_script[gw_script_cursor].frames;
  }
  idx = gw_script_cursor;
  for (ch = 0; ch < 4; ++ch) {
    if ((gw_script_chan_mask & (1u << ch)) == 0) {
      continue;
    }
    st[ch].stickX = gw_script[idx].sx;
    st[ch].stickY = gw_script[idx].sy;
    st[ch].substickX = 0;
    st[ch].substickY = 0;
    st[ch].triggerLeft = gw_script[idx].tl;
    st[ch].triggerRight = gw_script[idx].tr;
    gw_w16(&st[ch].button, gw_script[idx].buttons);
    st[ch].err = 0;
    driven |= 1u << ch;
  }
  if (--gw_script_left == 0) {
    ++gw_script_cursor;
  }
  return driven;
}

/* ---- 2. live file ------------------------------------------------------------------------------ */
static unsigned gw_pad_live_apply(PADStatus *st) {
  const char *path = getenv("MELEE_PAD_LIVE");
  char line[128];
  FILE *f;
  unsigned int buttons = 0;
  int sx = 0, sy = 0, tl = 0, tr = 0;

  if (path == NULL || path[0] == '\0') {
    return 0;
  }
  f = fopen(path, "r");
  if (f == NULL) {
    return 0;
  }
  if (fgets(line, sizeof line, f) != NULL &&
      sscanf(line, "%x %d %d %d %d", &buttons, &sx, &sy, &tl, &tr) >= 1) {
    st[PAD_CHAN0].stickX = (int8_t)sx;
    st[PAD_CHAN0].stickY = (int8_t)sy;
    st[PAD_CHAN0].triggerLeft = (uint8_t)tl;
    st[PAD_CHAN0].triggerRight = (uint8_t)tr;
    gw_w16(&st[PAD_CHAN0].button, (uint16_t)buttons);
    st[PAD_CHAN0].err = 0;
    fclose(f);
    return 1u << PAD_CHAN0;
  }
  fclose(f);
  return 0;
}

/* ---- 3. gd.input overrides, and what the game saw ---------------------------------------------- */
static struct {
  int samples; /* logic frames left; 0 = claimed but neutral */
  int owner;   /* 0 = unclaimed; script or console-client identity otherwise */
  unsigned buttons;
  int sx, sy, cx, cy, tl, tr;
} gw_ovr[4];
static struct {
  unsigned buttons;
  int sx, sy, cx, cy, tl, tr;
} gw_seen[4];
/* Menu interception preserves the controller's analog state and connection.
 * Raw buttons remain available to the owning script for edge detection. */
static unsigned gw_raw_buttons[4];
static struct { int owner; unsigned buttons; } gw_menu_mask[4];
/* A chord a script keeps from the game: while every button of `buttons` is held in one sample, the game sees none
 * of them. Decided on the same sample, so it works when the buttons arrive on the same frame (a mask set from the
 * previous frame's read could not). Raw buttons stay readable through gd.pad(port, true). */
static struct { int owner; unsigned buttons; } gw_chord[4];
static int gw_paused_sample;

/* D-pad (bits 0-3) and START (0x1000, the PAD button bit): what a script may keep from the game. START lets an
 * overlay opened by a chord (Envoy's Z+START bag) stop the same press from also pausing the match. */
#define GW_PAD_MASKABLE (15u | 0x1000u)
int gw_script_pad_mask(int ch, int owner, unsigned buttons) {
  if (ch < 0 || ch > 3 || owner <= 0 || (buttons & ~GW_PAD_MASKABLE)) return 0;
  if (gw_menu_mask[ch].owner && gw_menu_mask[ch].owner != owner) return 0;
  gw_menu_mask[ch].owner = buttons ? owner : 0;
  gw_menu_mask[ch].buttons = buttons;
  return 1;
}
int gw_script_pad_chord(int ch, int owner, unsigned buttons) {
  if (ch < 0 || ch > 3 || owner <= 0 || (buttons & ~0x1F7Fu)) return 0;
  if (buttons && (buttons & (buttons - 1)) == 0) return 0; /* a chord is two or more buttons; 0 releases */
  if (gw_chord[ch].owner && gw_chord[ch].owner != owner) return 0;
  gw_chord[ch].owner = buttons ? owner : 0;
  gw_chord[ch].buttons = buttons;
  return 1;
}
void gw_script_pad_masks_clear(void) {
  memset(gw_menu_mask, 0, sizeof gw_menu_mask);
  memset(gw_chord, 0, sizeof gw_chord);
}
unsigned gw_script_pad_raw_buttons(int ch) { return ch >= 0 && ch < 4 ? gw_raw_buttons[ch] : 0; }

/* gd.pad() still sees the paused menu sample, but it is not one of the logic
 * PADReads promised by gd.input/console input. */
void gw_script_pad_paused_sample(int on) { gw_paused_sample = on != 0; }

void gw_script_pad_override(int ch, int owner, unsigned buttons, int sx, int sy, int cx, int cy,
                            int l, int r, int samples) {
  if (ch < 0 || ch > 3 || owner <= 0) {
    return;
  }
  if (gw_ovr[ch].owner != owner) {
    if (gw_ovr[ch].owner != 0) {
      gw_pad_log("gw: pad: P%d script release (owner %d)", ch + 1, gw_ovr[ch].owner);
    }
    gw_pad_log("gw: pad: P%d script claim (owner %d)", ch + 1, owner);
  }
  gw_ovr[ch].owner = owner;
  gw_ovr[ch].samples = samples;
  gw_ovr[ch].buttons = buttons;
  gw_ovr[ch].sx = sx;
  gw_ovr[ch].sy = sy;
  gw_ovr[ch].cx = cx;
  gw_ovr[ch].cy = cy;
  gw_ovr[ch].tl = l;
  gw_ovr[ch].tr = r;
}

/* The end of a logic frame that read the pads (gw_Script_FramePost; not a rollback or rewind
 * re-simulation): each claimed port's hold has covered one more frame. */
void gw_Script_PadFrameConsumed(void) {
  int ch;
  for (ch = 0; ch < 4; ++ch) {
    if (gw_ovr[ch].owner != 0 && gw_ovr[ch].samples > 0) {
      gw_ovr[ch].samples--;
    }
  }
}

extern void gw_ScriptGame_VirtualPadRelease(int slot,int owner);
void gw_script_pad_release(int ch, int owner) {
  if (ch>=4 && ch<6 && owner>0) { gw_ScriptGame_VirtualPadRelease(ch,owner); return; }
  if (ch >= 0 && ch < 4 && owner > 0 && gw_menu_mask[ch].owner == owner) {
    memset(&gw_menu_mask[ch], 0, sizeof gw_menu_mask[ch]);
  }
  if (ch >= 0 && ch < 4 && owner > 0 && gw_chord[ch].owner == owner) {
    memset(&gw_chord[ch], 0, sizeof gw_chord[ch]);
  }
  if (ch >= 0 && ch <= 3 && gw_ovr[ch].owner == owner && owner > 0) {
    gw_ovr[ch].samples = 0;
    gw_ovr[ch].owner = 0;
    gw_pad_log("gw: pad: P%d script release (owner %d)", ch + 1, owner);
  }
}

void gw_script_pad_release_owner(int owner) {
  int ch;
  if (owner>0) gw_ScriptGame_VirtualPadRelease(-1,owner);
  for (ch = 0; ch < 4; ++ch) {
    gw_script_pad_release(ch, owner);
  }
}

void gw_script_pad_state(int ch, unsigned *buttons, int *sx, int *sy, int *cx, int *cy, int *l,
                         int *r) {
  if (ch < 0 || ch > 3) {
    return;
  }
  *buttons = gw_seen[ch].buttons;
  *sx = gw_seen[ch].sx;
  *sy = gw_seen[ch].sy;
  *cx = gw_seen[ch].cx;
  *cy = gw_seen[ch].cy;
  *l = gw_seen[ch].tl;
  *r = gw_seen[ch].tr;
}

/* gd.mirror_pad (the Geno Lab): port `to` receives exactly what port `from` sends, after every
 * other source - two fighters under the same inputs. Off during any netplay/rollback session. */
static int gw_mirror_from = -1, gw_mirror_to = -1, gw_mirror_take = 0;
void gw_script_pad_mirror(int from, int to) {
  gw_mirror_from = from;
  gw_mirror_to = to;
  gw_mirror_take = 0;
}
/* Stage D (the dummy's recording): "take" = port `from` is left neutral while `to` gets its pad,
 * so the player drives the dummy and their own fighter stands still. Set after the mirror. */
void gw_script_pad_take(int on) { gw_mirror_take = on; }

extern int gw_RB_Enabled(void);
extern int gw_Netplay_Enabled(void);

unsigned gw_Script_PadApply(void *pad_status_array) {
  PADStatus *st = (PADStatus *)pad_status_array;
  int ch;
  unsigned driven = gw_pad_script_apply(st) | gw_pad_live_apply(st);
  for (ch = 0; ch < 4; ++ch) {
    if (gw_ovr[ch].owner != 0) {
      /* A claim stays connected and neutral after its queued samples run out. Clear every
       * physical field so an adapter or SDL pad cannot leak through during that gap. */
      memset(&st[ch], 0, sizeof st[ch]);
      st[ch].err = 0;
      if (gw_ovr[ch].samples > 0) {
        st[ch].stickX = (s8)gw_ovr[ch].sx;
        st[ch].stickY = (s8)gw_ovr[ch].sy;
        st[ch].substickX = (s8)gw_ovr[ch].cx;
        st[ch].substickY = (s8)gw_ovr[ch].cy;
        st[ch].triggerLeft = (u8)gw_ovr[ch].tl;
        st[ch].triggerRight = (u8)gw_ovr[ch].tr;
        gw_w16(&st[ch].button, (uint16_t)gw_ovr[ch].buttons);
      }
      driven |= 1u << ch;
    }
  }
  if (gw_mirror_from >= 0 && gw_mirror_from < 4 && gw_mirror_to >= 0 && gw_mirror_to < 4 &&
      !gw_RB_Enabled() && !gw_Netplay_Enabled()) {
    st[gw_mirror_to] = st[gw_mirror_from];
    driven |= 1u << gw_mirror_to;
    if (gw_mirror_take) {
      memset(&st[gw_mirror_from], 0, sizeof st[gw_mirror_from]);
      driven |= 1u << gw_mirror_from;
    }
  }
  for (ch = 0; ch < 4; ++ch) {
    gw_raw_buttons[ch] = gw_r16(&st[ch].button);
    if (!gw_RB_Enabled() && !gw_Netplay_Enabled()) {
      unsigned hide = gw_menu_mask[ch].buttons;
      if (gw_chord[ch].buttons && (gw_raw_buttons[ch] & gw_chord[ch].buttons) == gw_chord[ch].buttons) {
        hide |= gw_chord[ch].buttons;
      }
      gw_w16(&st[ch].button, (uint16_t)(gw_raw_buttons[ch] & ~hide));
    }
    gw_seen[ch].buttons = gw_r16(&st[ch].button);
    gw_seen[ch].sx = st[ch].stickX;
    gw_seen[ch].sy = st[ch].stickY;
    gw_seen[ch].cx = st[ch].substickX;
    gw_seen[ch].cy = st[ch].substickY;
    gw_seen[ch].tl = st[ch].triggerLeft;
    gw_seen[ch].tr = st[ch].triggerRight;
  }
  return driven;
}
