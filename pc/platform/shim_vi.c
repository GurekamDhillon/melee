/* VI shims and the frame driver.
 *
 * On a GameCube the video interface raises an interrupt every field, and HSD hangs its whole
 * buffer-management state machine off that: HSD_VIWaitXFBDrawEnable, HSD_VIWaitXFBFlush and
 * HSD_VIGXSetDrawDone all spin until a callback moves an external framebuffer to the next state.
 * Aurora has no VI at all, so the port owns that clock. Every blocking wait in the game ends up
 * in VIWaitForRetrace, which is therefore where a frame is presented, window events are pumped,
 * the virtual clock advances, alarms fire and asynchronous transfers complete.
 */
#include "shim_vi.h"

#include "gw.h"
#include "shim_gx.h"
#include "shim_os.h"

#include <aurora/aurora.h>
#include <aurora/event.h>
#include <dolphin/gx.h>
#include <dolphin/vi.h>

#include <stdlib.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

typedef void (*gw_retrace_cb)(uint32_t retraceCount);
typedef void (*gw_drawdone_cb)(void);

static gw_retrace_cb gw_pre_retrace_cb;
static gw_retrace_cb gw_post_retrace_cb;
static gw_drawdone_cb gw_draw_done_cb;

static bool gw_frame_begun;
static bool gw_frame_has_content;
static uint32_t gw_retrace_count;
static void *gw_next_framebuffer;
static bool gw_exiting;

/* ---- virtual GameCube clock -------------------------------------------------------------- */

/* The GameCube's time base runs at the bus clock divided by four. */
#define GW_TIMER_CLOCK 40500000u
#define GW_TICKS_PER_FIELD (GW_TIMER_CLOCK / 60u)

static uint64_t gw_ticks;
static uint64_t gw_last_advance_ms;

uint64_t gw_time_ticks(void) { return gw_ticks; }
void gw_time_advance_field(void) { gw_ticks += GW_TICKS_PER_FIELD; }

/* ---- deferred work -----------------------------------------------------------------------
 * The DVD and ARQ shims complete their transfers immediately but must not call back into game
 * code from the middle of the call that started them: on hardware those callbacks arrive later,
 * from an interrupt, and game code is written around that. They queue here instead. */
#define GW_MAX_DEFERRED 256
static struct {
  gw_deferred_fn fn;
  void *a;
  void *b;
  uint32_t c;
} gw_deferred[GW_MAX_DEFERRED];
static int gw_deferred_count;

void gw_defer(gw_deferred_fn fn, void *a, void *b, uint32_t c) {
  if (gw_deferred_count >= GW_MAX_DEFERRED) {
    gw_log("gw: deferred queue overflow, running callback inline");
    fn(a, b, c);
    return;
  }
  gw_deferred[gw_deferred_count].fn = fn;
  gw_deferred[gw_deferred_count].a = a;
  gw_deferred[gw_deferred_count].b = b;
  gw_deferred[gw_deferred_count].c = c;
  ++gw_deferred_count;
}

void gw_run_deferred(void) {
  /* Callbacks may queue more work; run only what was pending on entry. */
  int n = gw_deferred_count;
  if (n == 0) {
    return;
  }
  static bool running;
  if (running) {
    return;
  }
  running = true;
  for (int i = 0; i < n; ++i) {
    gw_deferred[i].fn(gw_deferred[i].a, gw_deferred[i].b, gw_deferred[i].c);
  }
  int left = gw_deferred_count - n;
  for (int i = 0; i < left; ++i) {
    gw_deferred[i] = gw_deferred[n + i];
  }
  gw_deferred_count = left;
  running = false;
}

/* ---- frame driver ------------------------------------------------------------------------- */

static void gw_handle_events(void) {
  const AuroraEvent *event = aurora_update();
  while (event != NULL && event->type != AURORA_NONE) {
    switch (event->type) {
    case AURORA_EXIT:
      gw_exiting = true;
      break;
    default:
      break;
    }
    ++event;
  }
  if (gw_exiting) {
    gw_log("melee-pc: window closed, shutting down");
    gw_dump_stub_summary();
    if (gw_frame_begun) {
      aurora_end_frame();
      gw_frame_begun = false;
    }
    aurora_shutdown();
    exit(0);
  }
}

bool gw_frame_init(void) {
  gw_handle_events();
  gw_frame_begun = aurora_begin_frame();
  return true;
}

void gw_frame_mark_content(void) { gw_frame_has_content = true; }

void gw_frame_tick(void) {
  if (gw_frame_has_content && gw_frame_begun) {
    aurora_end_frame(); /* presents, and waits for vsync, which is what paces the game */
    gw_frame_begun = false;
    gw_frame_has_content = false;
  } else {
    /* A wait that produced no new frame: don't burn a core spinning. */
    Sleep(1);
  }

  gw_handle_events();

  if (!gw_frame_begun) {
    gw_frame_begun = aurora_begin_frame();
  }

  ++gw_retrace_count;
  gw_time_advance_field();
  gw_last_advance_ms = GetTickCount64();
  gw_os_run_alarms(gw_ticks);
  gw_run_deferred();

  if (gw_pre_retrace_cb != NULL) {
    gw_pre_retrace_cb(gw_retrace_count);
  }
  if (gw_post_retrace_cb != NULL) {
    gw_post_retrace_cb(gw_retrace_count);
  }
}

void gw_wait_idle(void) {
  const uint64_t now = GetTickCount64();
  if (gw_last_advance_ms == 0) {
    gw_last_advance_ms = now;
    return;
  }
  bool ticked = false;
  while (now - gw_last_advance_ms >= 16) {
    gw_last_advance_ms += 16;
    gw_time_advance_field();
    ticked = true;
  }
  if (ticked) {
    gw_os_run_alarms(gw_ticks);
    gw_run_deferred();
  }
}

/* ---- GX draw-done ------------------------------------------------------------------------
 * HSD_VIGXSetDrawDone sets a "waiting" flag and calls GXSetDrawDone; the flag is cleared by the
 * draw-done callback, and HSD_VIGXSetDrawDone spins on GXWaitDrawDone until that happens. The
 * port renders synchronously, so the callback can fire as soon as the FIFO is flushed. */

void gw_gx_set_draw_done(void) {
  GXSetDrawDone();
  if (gw_draw_done_cb != NULL) {
    gw_draw_done_cb();
  }
}

void gw_gx_wait_draw_done(void) {
  GXDrawDone();
  if (gw_draw_done_cb != NULL) {
    gw_draw_done_cb();
  }
}

void *gw_gx_set_draw_done_callback(void *cb) {
  void *old = (void *)gw_draw_done_cb;
  gw_draw_done_cb = (gw_drawdone_cb)cb;
  return old;
}

/* ---- VI entry points ---------------------------------------------------------------------- */

void gw_VIInit(void) { VIInit(); }

void gw_VIConfigure(const void *rm_be) {
  GXRenderModeObj rm;
  gw_read_render_mode(&rm, rm_be);
  VIConfigure(&rm);
}

void gw_VIFlush(void) { VIFlush(); }

void gw_VISetBlack(int black) { (void)black; }

void gw_VIWaitForRetrace(void) { gw_frame_tick(); }

uint32_t gw_VIGetRetraceCount(void) { return gw_retrace_count; }

void gw_VISetNextFrameBuffer(void *fb) { gw_next_framebuffer = fb; }

void *gw_VISetPreRetraceCallback(void *cb) {
  void *old = (void *)gw_pre_retrace_cb;
  gw_pre_retrace_cb = (gw_retrace_cb)cb;
  return old;
}

void *gw_VISetPostRetraceCallback(void *cb) {
  void *old = (void *)gw_post_retrace_cb;
  gw_post_retrace_cb = (gw_retrace_cb)cb;
  return old;
}

/* Fields alternate; nothing in the port renders per-field, so report a stable value. */
uint32_t gw_VIGetNextField(void) { return (gw_retrace_count & 1u) != 0u ? 1u : 0u; }

/* Returning 0 keeps boot out of the progressive-scan prompt. */
uint32_t gw_VIGetDTVStatus(void) { return 0; }

/* VI_NTSC */
uint32_t gw_VIGetTvFormat(void) { return 0; }
