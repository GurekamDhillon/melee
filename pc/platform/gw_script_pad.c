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
 *   3. gd.input / the console "input" command: per-port overrides for N samples.
 * Whatever the game finally sees is recorded for gd.pad().
 */
#include "gw.h"
#include "gw_script.h"

#include <dolphin/pad.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

static void gw_pad_script_apply(PADStatus *st) {
  int idx;
  int ch;

  if (!gw_script_loaded) {
    gw_pad_script_load();
  }
  if (gw_script_count == 0) {
    return;
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
  }
  if (--gw_script_left == 0) {
    ++gw_script_cursor;
  }
}

/* ---- 2. live file ------------------------------------------------------------------------------ */
static void gw_pad_live_apply(PADStatus *st) {
  const char *path = getenv("MELEE_PAD_LIVE");
  char line[128];
  FILE *f;
  unsigned int buttons = 0;
  int sx = 0, sy = 0, tl = 0, tr = 0;

  if (path == NULL || path[0] == '\0') {
    return;
  }
  f = fopen(path, "r");
  if (f == NULL) {
    return;
  }
  if (fgets(line, sizeof line, f) != NULL &&
      sscanf(line, "%x %d %d %d %d", &buttons, &sx, &sy, &tl, &tr) >= 1) {
    st[PAD_CHAN0].stickX = (int8_t)sx;
    st[PAD_CHAN0].stickY = (int8_t)sy;
    st[PAD_CHAN0].triggerLeft = (uint8_t)tl;
    st[PAD_CHAN0].triggerRight = (uint8_t)tr;
    gw_w16(&st[PAD_CHAN0].button, (uint16_t)buttons);
    st[PAD_CHAN0].err = 0;
  }
  fclose(f);
}

/* ---- 3. gd.input overrides, and what the game saw ---------------------------------------------- */
static struct {
  int samples; /* PADReads left; 0 = off */
  unsigned buttons;
  int sx, sy, cx, cy, tl, tr;
} gw_ovr[4];
static struct {
  unsigned buttons;
  int sx, sy, cx, cy, tl, tr;
} gw_seen[4];

void gw_script_pad_override(int ch, unsigned buttons, int sx, int sy, int cx, int cy, int l, int r,
                            int samples) {
  if (ch < 0 || ch > 3) {
    return;
  }
  gw_ovr[ch].samples = samples;
  gw_ovr[ch].buttons = buttons;
  gw_ovr[ch].sx = sx;
  gw_ovr[ch].sy = sy;
  gw_ovr[ch].cx = cx;
  gw_ovr[ch].cy = cy;
  gw_ovr[ch].tl = l;
  gw_ovr[ch].tr = r;
}

void gw_script_pad_release(int ch) {
  if (ch >= 0 && ch <= 3) {
    gw_ovr[ch].samples = 0;
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
static int gw_mirror_from = -1, gw_mirror_to = -1;
void gw_script_pad_mirror(int from, int to) {
  gw_mirror_from = from;
  gw_mirror_to = to;
}

extern int gw_RB_Enabled(void);
extern int gw_Netplay_Enabled(void);

void gw_Script_PadApply(void *pad_status_array) {
  PADStatus *st = (PADStatus *)pad_status_array;
  int ch;
  gw_pad_script_apply(st);
  gw_pad_live_apply(st);
  for (ch = 0; ch < 4; ++ch) {
    if (gw_ovr[ch].samples > 0) {
      st[ch].stickX = (s8)gw_ovr[ch].sx;
      st[ch].stickY = (s8)gw_ovr[ch].sy;
      st[ch].substickX = (s8)gw_ovr[ch].cx;
      st[ch].substickY = (s8)gw_ovr[ch].cy;
      st[ch].triggerLeft = (u8)gw_ovr[ch].tl;
      st[ch].triggerRight = (u8)gw_ovr[ch].tr;
      gw_w16(&st[ch].button, (uint16_t)gw_ovr[ch].buttons);
      st[ch].err = 0;
      gw_ovr[ch].samples--;
    }
  }
  if (gw_mirror_from >= 0 && gw_mirror_from < 4 && gw_mirror_to >= 0 && gw_mirror_to < 4 &&
      !gw_RB_Enabled() && !gw_Netplay_Enabled()) {
    st[gw_mirror_to] = st[gw_mirror_from];
  }
  for (ch = 0; ch < 4; ++ch) {
    gw_seen[ch].buttons = gw_r16(&st[ch].button);
    gw_seen[ch].sx = st[ch].stickX;
    gw_seen[ch].sy = st[ch].stickY;
    gw_seen[ch].cx = st[ch].substickX;
    gw_seen[ch].cy = st[ch].substickY;
    gw_seen[ch].tl = st[ch].triggerLeft;
    gw_seen[ch].tr = st[ch].triggerRight;
  }
}
