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
#include "shim_ax.h"
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

/* The clock free-runs off the host's high-resolution counter rather than being stepped by the
 * frame driver. Stepping it only from gw_frame_tick and gw_wait_idle made game time stop
 * whenever the game was computing instead of waiting -- a 14-second run advanced about two
 * seconds of game time -- so every duration the game measured was short by however busy the
 * frame had been, and alarms bunched up behind long stretches of work. A GameCube's time base
 * runs unconditionally, and so does this. */
static uint64_t gw_qpc_freq;
static uint64_t gw_qpc_base;
static uint64_t gw_last_advance_ms;

uint64_t gw_time_ticks(void) {
  LARGE_INTEGER now;
  uint64_t elapsed;
  if (gw_qpc_freq == 0) {
    LARGE_INTEGER freq;
    QueryPerformanceFrequency(&freq);
    gw_qpc_freq = (uint64_t)freq.QuadPart;
    QueryPerformanceCounter(&now);
    gw_qpc_base = (uint64_t)now.QuadPart;
    return 0;
  }
  QueryPerformanceCounter(&now);
  elapsed = (uint64_t)now.QuadPart - gw_qpc_base;
  /* Split the conversion so a long session cannot overflow: elapsed * 40.5e6 would wrap after
   * about half a day at a 10 MHz counter. */
  return (elapsed / gw_qpc_freq) * GW_TIMER_CLOCK +
         ((elapsed % gw_qpc_freq) * GW_TIMER_CLOCK) / gw_qpc_freq;
}

/* Kept because the frame driver still marks field boundaries, but the clock no longer depends on
 * being told about them. */
void gw_time_advance_field(void) {}

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


/* ---- frame profiler ---------------------------------------------------------------------- */

/* Off unless MELEE_PROFILE=1. The port is not as smooth as Dolphin and the cause is not obvious,
 * so this measures the frame rather than guessing at it. The distinction that matters is between
 * a uniformly slow frame and an occasional long one: those have completely different causes and
 * an average hides both. Hence percentiles and a histogram, not a mean.
 *
 * The frame is split at the points gw_frame_tick already has:
 *   game    - everything between the end of the last tick and the start of this one, i.e. game
 *             logic plus the FIFO writes it makes
 *   present - aurora_end_frame(), which only enqueues to Aurora's render worker and returns, so
 *             this is submit cost, not a vsync wait; the real Present()/vsync block happens on
 *             that worker thread, and the game thread's only render back-pressure is the frame
 *             slot acquired in begin
 *   events  - gw_handle_events()
 *   begin   - aurora_begin_frame()
 *
 * "empty" counts ticks that presented nothing and took the Sleep(1) path: the game polling
 * retrace more than once per frame. Those cost a millisecond each and are a smoothness suspect
 * in their own right. */
static int gw_prof_on(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_PROFILE");
    cached = (v != NULL && v[0] == '1') ? 1 : 0;
  }
  return cached;
}

#define GW_PROF_FRAMES 600
#define GW_PROF_BUCKETS 8

static double gw_prof_freq;
static long long gw_prof_last_end;
static long long gw_prof_last_cpu = -1;
static double gw_prof_samples[GW_PROF_FRAMES];
static int gw_prof_count;
static int gw_prof_head;
static double gw_prof_sum_game, gw_prof_sum_present, gw_prof_sum_events, gw_prof_sum_begin;
static double gw_prof_sum_cpu;
static uint32_t gw_prof_frames, gw_prof_empty;
static uint32_t gw_prof_hist[GW_PROF_BUCKETS];
static int gw_prof_spikes;

/* Upper bounds in ms. 16.67 is the target; the buckets either side of it are what matter. */
static const double gw_prof_edges[GW_PROF_BUCKETS] = { 14.0, 16.0, 17.5, 20.0, 25.0, 34.0, 50.0, 1e9 };

static long long gw_prof_now(void) {
  LARGE_INTEGER t;
  QueryPerformanceCounter(&t);
  return (long long)t.QuadPart;
}

static double gw_prof_ms(long long a, long long b) {
  if (gw_prof_freq <= 0.0) {
    LARGE_INTEGER f;
    QueryPerformanceFrequency(&f);
    gw_prof_freq = (double)f.QuadPart;
  }
  return ((double)(b - a) * 1000.0) / gw_prof_freq;
}

/* Kernel+user CPU time of the calling thread, in 100 ns units. The wall split cannot tell real
 * game work from the pad-wait spin, and those call for opposite fixes (optimise the work vs.
 * yield the spin); this is the number that separates them. */
static long long gw_prof_thread_cpu(void) {
  FILETIME create, exit, kernel, user;
  ULARGE_INTEGER k, u;
  if (!GetThreadTimes(GetCurrentThread(), &create, &exit, &kernel, &user)) {
    return 0;
  }
  k.LowPart = kernel.dwLowDateTime;
  k.HighPart = kernel.dwHighDateTime;
  u.LowPart = user.dwLowDateTime;
  u.HighPart = user.dwHighDateTime;
  return (long long)(k.QuadPart + u.QuadPart);
}

static int gw_prof_cmp(const void *a, const void *b) {
  const double x = *(const double *)a;
  const double y = *(const double *)b;
  return (x < y) ? -1 : ((x > y) ? 1 : 0);
}

static double gw_prof_pct(const double *sorted, int n, double p) {
  int i = (int)(p * (double)(n - 1) + 0.5);
  if (i < 0) { i = 0; }
  if (i >= n) { i = n - 1; }
  return sorted[i];
}

static void gw_prof_report(void) {
  double sorted[GW_PROF_FRAMES];
  int n = gw_prof_count;
  int i;

  if (n < 2 || gw_prof_frames == 0u) {
    return;
  }
  for (i = 0; i < n; ++i) {
    sorted[i] = gw_prof_samples[i];
  }
  qsort(sorted, (size_t)n, sizeof(sorted[0]), gw_prof_cmp);

  gw_log("gw: PROF frame ms  p50=%.2f p95=%.2f p99=%.2f max=%.2f  (%d frames)",
         gw_prof_pct(sorted, n, 0.50), gw_prof_pct(sorted, n, 0.95), gw_prof_pct(sorted, n, 0.99),
         sorted[n - 1], n);
  gw_log("gw: PROF split ms  game=%.2f present=%.2f events=%.2f begin=%.2f  empty_ticks=%u",
         gw_prof_sum_game / gw_prof_frames, gw_prof_sum_present / gw_prof_frames,
         gw_prof_sum_events / gw_prof_frames, gw_prof_sum_begin / gw_prof_frames, gw_prof_empty);
  gw_log("gw: PROF cpu ms    game_cpu=%.2f  of_game_wall=%.2f  wait=%.2f",
         gw_prof_sum_cpu / gw_prof_frames, gw_prof_sum_game / gw_prof_frames,
         (gw_prof_sum_game - gw_prof_sum_cpu) / gw_prof_frames);
  gw_log("gw: PROF hist      <14:%u  <16:%u  <17.5:%u  <20:%u  <25:%u  <34:%u  <50:%u  50+:%u",
         gw_prof_hist[0], gw_prof_hist[1], gw_prof_hist[2], gw_prof_hist[3], gw_prof_hist[4],
         gw_prof_hist[5], gw_prof_hist[6], gw_prof_hist[7]);

  gw_prof_sum_game = gw_prof_sum_present = gw_prof_sum_events = gw_prof_sum_begin = 0.0;
  gw_prof_sum_cpu = 0.0;
  gw_prof_frames = 0u;
  gw_prof_empty = 0u;
  gw_prof_spikes = 0;
  for (i = 0; i < GW_PROF_BUCKETS; ++i) {
    gw_prof_hist[i] = 0u;
  }
}

static void gw_prof_record(double total) {
  int b;
  for (b = 0; b < GW_PROF_BUCKETS; ++b) {
    if (total < gw_prof_edges[b]) {
      ++gw_prof_hist[b];
      break;
    }
  }
  gw_prof_samples[gw_prof_head] = total;
  gw_prof_head = (gw_prof_head + 1) % GW_PROF_FRAMES;
  if (gw_prof_count < GW_PROF_FRAMES) {
    ++gw_prof_count;
  }
}

static uint32_t gw_presented_count;

/* Field pacing. A GameCube's VI gives the game a hard 16.667 ms boundary. Here the present is
 * asynchronous (aurora_end_frame only enqueues to the render worker) and a VRR display lets
 * Present() return without back-pressure, so nothing holds the game to 60 Hz and it free-runs at
 * its own frame cost (~15.6 ms). The 1/60 pad alarm is only a soft reference, so every ~13
 * free-run frames the pad queue drains and the game stalls a whole period -- the 15.6/30.6 ms
 * bimodal judder. Wait out the boundary here against the free-running virtual clock, with
 * catch-up so a long frame (map load) does not accumulate a deficit. */
static uint64_t gw_last_field_tick;

static void gw_pace_field(void) {
  uint64_t now = gw_time_ticks();
  uint64_t last_pump;
  if (gw_last_field_tick == 0) {
    gw_last_field_tick = now;
    return;
  }
  const uint64_t target = gw_last_field_tick + GW_TICKS_PER_FIELD;
  if (now < target) {
    last_pump = now;
    while ((now = gw_time_ticks()) < target) {
      if (now - last_pump >= GW_TIMER_CLOCK / 1000u) {
        last_pump = now;
        gw_os_run_alarms(now);
        gw_run_deferred();
      }
      YieldProcessor();
    }
  }
  gw_last_field_tick = now;
}

void gw_frame_tick(void) {
  const int prof = gw_prof_on();
  long long t_enter = 0, t_present = 0, t_events = 0, t_begin = 0;
  int presented = 0;

  if (prof) {
    t_enter = gw_prof_now();
  }

  if (gw_frame_has_content && gw_frame_begun) {
    gw_pace_field();
    aurora_end_frame(); /* enqueues to the render worker; the real Present() is async */
    gw_frame_begun = false;
    gw_frame_has_content = false;
    ++gw_presented_count;
    presented = 1;
  } else {
    /* A wait that produced no new frame: don't burn a core spinning. */
    Sleep(1);
    if (prof) {
      ++gw_prof_empty;
    }
  }
  if (prof) {
    t_present = gw_prof_now();
  }

  gw_handle_events();
  if (prof) {
    t_events = gw_prof_now();
  }

  if (!gw_frame_begun) {
    gw_frame_begun = aurora_begin_frame();
  }

  if (prof) {
    t_begin = gw_prof_now();
    if (presented) {
      /* "game" is the span since the previous tick finished: the game's own work plus the FIFO
       * writes it made, none of which happens inside this function. */
      if (gw_prof_last_end != 0) {
        const double game = gw_prof_ms(gw_prof_last_end, t_enter);
        const double total = gw_prof_ms(gw_prof_last_end, t_begin);
        const long long cpu_now = gw_prof_thread_cpu();
        const double cpu =
            (gw_prof_last_cpu >= 0) ? (double)(cpu_now - gw_prof_last_cpu) / 10000.0 : 0.0;
        gw_prof_last_cpu = cpu_now;
        gw_prof_sum_game += game;
        gw_prof_sum_cpu += cpu;
        gw_prof_sum_present += gw_prof_ms(t_enter, t_present);
        gw_prof_sum_events += gw_prof_ms(t_present, t_events);
        gw_prof_sum_begin += gw_prof_ms(t_events, t_begin);
        ++gw_prof_frames;
        gw_prof_record(total);
        /* The per-component averages cannot explain a bimodal distribution: they describe the
         * typical frame, while the judder lives in the tail. Attribute the slow frames
         * individually so it is clear whether a 33 ms frame is game work or a stalled present. */
        if (total > 25.0 && gw_prof_spikes < 12) {
          ++gw_prof_spikes;
          gw_log("gw: PROF spike %.2f ms  game=%.2f cpu=%.2f present=%.2f events=%.2f begin=%.2f",
                 total, game, cpu, gw_prof_ms(t_enter, t_present), gw_prof_ms(t_present, t_events),
                 gw_prof_ms(t_events, t_begin));
        }
        if ((gw_prof_frames % 180u) == 0u) {
          gw_prof_report();
        }
      }
      gw_prof_last_end = t_begin;
    }
  }

  ++gw_retrace_count;

  /* Heartbeat, roughly every two seconds of game time. Cheap, and it is the only way to tell a
   * hung frame loop from one that is running but drawing nothing. */
  if ((gw_retrace_count % 120u) == 0u) {
    uint32_t copies, prims, dlists;
    gw_gx_get_stats(&copies, &prims, &dlists);
    gw_log("gw: heartbeat retrace=%u presented=%u  gx: copydisp=%u prim=%u dlist=%u",
           gw_retrace_count, gw_presented_count, copies, prims, dlists);
  }

  gw_last_advance_ms = GetTickCount64();
  gw_os_run_alarms(gw_time_ticks());
  gw_run_deferred();

  if (gw_pre_retrace_cb != NULL) {
    gw_pre_retrace_cb(gw_retrace_count);
  }
  if (gw_post_retrace_cb != NULL) {
    gw_post_retrace_cb(gw_retrace_count);
  }

  /* Audio: generate zero or more 5 ms AX sub-frames on the game thread (the AX mixer lives in
   * shim_ax.c and drives HSD_SynthCallback + voice mixing). */
  gw_ax_frame_tick();
}

static uint32_t gw_wait_idle_count;

void gw_frame_stats(uint32_t *retrace, uint32_t *presented, uint32_t *waits) {
  *retrace = gw_retrace_count;
  *presented = gw_presented_count;
  *waits = gw_wait_idle_count;
}

/* Called from the shims a blocking game loop reaches (DVDGetDriveStatus, above all). The clock
 * runs on its own now, so this only has to give alarms and completions a chance to run; the
 * once-per-millisecond gate keeps a tight spin from calling them millions of times a second. */
void gw_wait_idle(void) {
  const uint64_t now = GetTickCount64();
  ++gw_wait_idle_count;
  if (now == gw_last_advance_ms) {
    return;
  }
  gw_last_advance_ms = now;
  gw_os_run_alarms(gw_time_ticks());
  gw_run_deferred();
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
