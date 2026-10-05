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

/* ---- 1b. MELEE_PAD_BOT: a TEST-ONLY reactive pad program for the Turbo soak -------------------------------------
 * MELEE_PAD_BOT="<slot>,<channel>,<seed>[;<slot>,<channel>,<seed>]" drives pad `channel` as the fighter in `slot`
 * (0 = P1): it reads the fighters (the same read-only accessors the Lua API uses) and writes ordinary pad values into
 * the local input path, so online it is sent and rolled back exactly like a human's input; it never touches the
 * simulation. It plays Turbo: it walks into range, throws out the first move of a plan, and the first free frame after
 * the Turbo window opens it presses the NEXT move of the plan (a different move: jab -> tilt -> jab ..., aerial ->
 * other aerial, special -> jab -> special, smash -> other smash, jab -> dash, jab -> crouch -> jab), restarting when a
 * move misses. The opponent is the other of slots 0/1. It stays on the stage and recovers (crudely). Not for play.
 * Its counters are logged every 1800 pad reads ("pad bot slot N: ..."). Its own state is native (not rolled back). */
extern float gw_ScriptGame_FighterF(int slot, int field);
extern int gw_ScriptGame_FighterI(int slot, int field);
extern int gw_ScriptGame_IntrWinRead(int entity, int field);
enum { PB_BTN_A = 0x100, PB_BTN_B = 0x200, PB_BTN_X = 0x400 };
/* sx is a multiple of the facing; kind 1 = jump (then wait to be airborne), 2 = dash (stick toward the opponent) */
typedef struct { unsigned btn; int sx, sy; int hold; int kind; int chain; } PbStep; /* chain 1: the next step follows at once, no window needed (crouch -> jab) */
static const PbStep pb_jab = { PB_BTN_A, 0, 0, 2, 0 }, pb_ftilt = { PB_BTN_A, 45, 0, 2, 0 },
    pb_utilt = { PB_BTN_A, 0, 55, 2, 0 }, pb_dtilt = { PB_BTN_A, 0, -55, 2, 0 },
    pb_fsmash = { PB_BTN_A, 100, 0, 2, 0 }, pb_usmash = { PB_BTN_A, 0, 100, 2, 0 },
    pb_dsmash = { PB_BTN_A, 0, -100, 2, 0 }, pb_special = { PB_BTN_B, 0, -80, 3, 0 },
    pb_dash = { 0, 100, 0, 4, 2, 1 }, pb_crouch = { 0, 0, -80, 6, 0, 1 }, pb_jump = { PB_BTN_X, 30, 0, 2, 1 },
    pb_airjump = { PB_BTN_X, 0, 0, 2, 0 }, pb_nair = { PB_BTN_A, 0, 0, 2, 0 }, pb_fair = { PB_BTN_A, 80, 0, 2, 0 },
    pb_bair = { PB_BTN_A, -80, 0, 2, 0 }, pb_uair = { PB_BTN_A, 0, 80, 2, 0 }, pb_dair = { PB_BTN_A, 0, -80, 2, 0 };
#define PB_MAXSTEP 16
typedef struct { int n; const PbStep *s[PB_MAXSTEP]; int air; } PbPlan;
static const PbPlan pb_plans[] = {
    { 7, { &pb_jab, &pb_ftilt, &pb_jab, &pb_utilt, &pb_jab, &pb_dtilt, &pb_jab }, 0 },
    { 4, { &pb_fsmash, &pb_usmash, &pb_dsmash, &pb_fsmash }, 0 },
    { 6, { &pb_special, &pb_jab, &pb_special, &pb_jab, &pb_special, &pb_jab }, 0 },
    { 3, { &pb_jab, &pb_dash, &pb_jab }, 0 },
    { 15, { &pb_jab, &pb_crouch, &pb_jab, &pb_crouch, &pb_jab, &pb_crouch, &pb_jab, &pb_crouch, &pb_jab, &pb_crouch, &pb_jab, &pb_crouch, &pb_jab, &pb_crouch, &pb_jab }, 0 },
    { 3, { &pb_jab, &pb_dash, &pb_jab }, 0 },
    { 8, { &pb_jump, &pb_nair, &pb_fair, &pb_airjump, &pb_bair, &pb_uair, &pb_dair, &pb_nair }, 1 },
    { 5, { &pb_jab, &pb_special, &pb_ftilt, &pb_fsmash, &pb_jab }, 0 },
};
#define PB_NPLANS ((int) (sizeof pb_plans / sizeof pb_plans[0]))
typedef struct {
  int slot, chan;
  unsigned rng;
  int plan, step, phase, t, hold; /* phase 0 approach, 1 press, 2 wait */
  long reads, plans, steps, wins, cancels, aborts, last_wf, pause;
} PbBot;
static PbBot pb_bots[4];
static int pb_nbots = -1;
static float pb_edge = 62.0f; /* MELEE_PAD_BOT_EDGE: the stage half-width to stay inside (Battlefield 62, Final Destination 70) */
static unsigned pb_rand(PbBot *b) {
  b->rng = b->rng * 1664525u + 1013904223u;
  return b->rng >> 8;
}
static void pb_load(void) {
  const char *e = getenv("MELEE_PAD_BOT");
  pb_nbots = 0;
  if (getenv("MELEE_PAD_BOT_EDGE") != NULL) {
    pb_edge = (float) atof(getenv("MELEE_PAD_BOT_EDGE"));
  }
  while (e != NULL && *e != '\0' && pb_nbots < 4) {
    int slot, chan;
    unsigned seed;
    if (sscanf(e, "%d,%d,%u", &slot, &chan, &seed) == 3 && slot >= 0 && slot < 6 && chan >= 0 && chan < 4) {
      PbBot *b = &pb_bots[pb_nbots++];
      memset(b, 0, sizeof *b);
      b->slot = slot;
      b->chan = chan;
      b->rng = seed * 2654435761u + 12345u;
      b->plan = -1;
      gw_log("pad bot: slot %d on channel %d, seed %u (TEST-ONLY Turbo soak driver)", slot, chan, seed);
    }
    e = strchr(e, ';');
    if (e != NULL) {
      ++e;
    }
  }
}
static void pb_out(PADStatus *st, int ch, unsigned btn, int sx, int sy) {
  if (sx > 100) {
    sx = 100;
  }
  if (sx < -100) {
    sx = -100;
  }
  st[ch].stickX = (s8)sx;
  st[ch].stickY = (s8)sy;
  st[ch].substickX = 0;
  st[ch].substickY = 0;
  st[ch].triggerLeft = 0;
  st[ch].triggerRight = 0;
  gw_w16(&st[ch].button, (uint16_t)btn);
  st[ch].err = 0;
}
static void pb_press(PbBot *b, PADStatus *st, const PbStep *sp, int dir, int fdir) {
  pb_out(st, b->chan, sp->btn, sp->kind == 2 ? dir * 100 : sp->sx * fdir, sp->sy);
  if (++b->hold >= sp->hold) {
    b->phase = 2;
    b->t = 0;
    if (sp->chain && b->plan >= 0 && b->step + 1 < pb_plans[b->plan].n) { /* crouch -> jab: no window to wait for */
      b->step++;
      b->phase = 1;
      b->hold = 0;
      b->steps++;
    }
  }
}
static unsigned pb_one(PbBot *b, PADStatus *st) {
  int me = b->slot, opp = b->slot == 0 ? 1 : 0, ent = me + 1;
  float x, y, ox, face, hitlag, dist;
  int air, wf, dir, fdir;
  unsigned ret = 1u << b->chan;
  const PbPlan *pl;
  const PbStep *sp;

  b->reads++;
  if (gw_ScriptGame_FighterI(me, 0) != 1 || gw_ScriptGame_FighterI(opp, 0) != 1) {
    pb_out(st, b->chan, 0, 0, 0);
    b->phase = 0;
    b->plan = -1;
    return ret;
  }
  x = gw_ScriptGame_FighterF(me, 0);
  y = gw_ScriptGame_FighterF(me, 1);
  ox = gw_ScriptGame_FighterF(opp, 0);
  face = gw_ScriptGame_FighterF(me, 5);
  hitlag = gw_ScriptGame_FighterF(me, 7);
  air = gw_ScriptGame_FighterI(me, 4);
  wf = gw_ScriptGame_IntrWinRead(ent, 0);
  dir = ox >= x ? 1 : -1;
  fdir = face >= 0 ? 1 : -1;
  dist = ox >= x ? ox - x : x - ox;
  if (wf > 0 && b->last_wf == 0) {
    b->wins++;
  }
  b->last_wf = wf;
  if ((b->reads % 1800) == 0) {
    gw_log("pad bot slot %d: reads %ld plans %ld steps %ld windows seen %ld cancels pressed %ld aborts %ld", me,
           b->reads, b->plans, b->steps, b->wins, b->cancels, b->aborts);
  }
  /* stay on the stage: Battlefield's main platform is about +-68 wide */
  if (x > pb_edge || x < -pb_edge || y < -8.0f) {
    int toward = x > 0 ? -1 : 1;
    unsigned btn = 0;
    int sy = 0;
    b->phase = 0;
    b->plan = -1;
    if (air && (x > pb_edge + 4.0f || x < -pb_edge - 4.0f || y < -8.0f)) {
      if ((b->reads % 24) == 0) {
        btn = PB_BTN_X;
      }
      if (y < -25.0f && (b->reads % 6) < 3) {
        btn |= PB_BTN_B;
        sy = 90;
      }
    }
    pb_out(st, b->chan, btn, toward * 100, sy);
    return ret;
  }
  if (b->pause > 0) {
    b->pause--;
    pb_out(st, b->chan, 0, 0, 0);
    return ret;
  }
  if (b->phase == 0) {
    if (dist > 15.0f || (fdir != dir && !air)) {
      pb_out(st, b->chan, 0, dist > 45.0f || fdir != dir ? dir * 100 : dir * 55, 0); /* dash from afar, walk the last steps */
      return ret;
    }
    b->plan = (int)(pb_rand(b) % PB_NPLANS);
    if ((pb_rand(b) & 3) < 2) {
      b->plan = 4; /* the jab / crouch loop is the fastest window source: half of all plans */
    }
    if (air) {
      pb_out(st, b->chan, 0, 0, 0); /* landing first: every plan starts on the ground */
      b->plan = -1;
      return ret;
    }
    b->step = 0;
    b->phase = 1;
    b->hold = 0;
    b->t = 0;
    b->plans++;
  }
  pl = &pb_plans[b->plan];
  sp = pl->s[b->step];
  if (b->phase == 1) {
    pb_press(b, st, sp, dir, fdir);
    return ret;
  }
  /* phase 2: neutral; the first free frame after a window opens, press the NEXT move */
  pb_out(st, b->chan, 0, 0, 0);
  b->t++;
  if (sp->kind == 1) { /* a jump: wait to be in the air, then the next step */
    if (air && b->t >= 5) {
      b->step++;
      b->phase = 1;
      b->hold = 0;
      b->steps++;
    } else if (b->t > 30) {
      b->phase = 0;
      b->aborts++;
    }
    return ret;
  }
  if (wf > 0 && hitlag <= 0.0f) {
    if (b->step + 1 >= pl->n) {
      b->phase = 0;
      b->pause = 4;
      return ret;
    }
    b->step++;
    b->phase = 1;
    b->hold = 0;
    b->steps++;
    b->cancels++;
    pb_press(b, st, pl->s[b->step], dir, fdir); /* this very frame: the first allowed one */
    return ret;
  }
  if (b->t > 60 || (b->t > 4 && gw_ScriptGame_FighterI(me, 3) == 14 && wf == 0)) { /* timed out, or the move is over */
    b->phase = 0;
    b->aborts++;
    b->pause = 1;
  }
  return ret;
}
static unsigned gw_pad_bot_apply(PADStatus *st) {
  int i;
  unsigned driven = 0;
  if (pb_nbots < 0) {
    pb_load();
  }
  for (i = 0; i < pb_nbots; ++i) {
    driven |= pb_one(&pb_bots[i], st);
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
  unsigned driven = gw_pad_script_apply(st) | gw_pad_live_apply(st) | gw_pad_bot_apply(st);
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
