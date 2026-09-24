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
#include "gw_overlay.h"

#include <aurora/aurora.h>
#include <aurora/event.h>
#include <aurora/gfx.h> /* AuroraStats: MELEE_PROFILE_FRAMES */
#include <dolphin/gx.h>
#include <dolphin/vi.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <timeapi.h> /* timeBeginPeriod: see gw_pace_field */
#include <tlhelp32.h> /* the sampling profiler's thread list */

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
static uint64_t gw_last_alarm_tick; /* gw_wait_idle's alarm gate, OS ticks */

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

void gw_Snap_AsyncBegin(void);
void gw_Snap_AsyncEnd(void);

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
  gw_Snap_AsyncBegin(); /* gw_snap.c: what deferred work writes is the async world's, not the render pass's */
  for (int i = 0; i < n; ++i) {
    gw_deferred[i].fn(gw_deferred[i].a, gw_deferred[i].b, gw_deferred[i].c);
  }
  gw_Snap_AsyncEnd();
  int left = gw_deferred_count - n;
  for (int i = 0; i < left; ++i) {
    gw_deferred[i] = gw_deferred[n + i];
  }
  gw_deferred_count = left;
  running = false;
}

/* ---- frame driver ------------------------------------------------------------------------- */

/* ---- clean exit ---------------------------------------------------------------------------
 * Closing the window used to end in an access violation inside webgpu_dawn.dll (+0x363548, "Device
 * lost: Device was destroyed") and a crash log every time: Aurora's teardown releases the queue,
 * the surface and the device in an order Dawn's D3D11 backend faults on. Everything worth keeping
 * is already safe by the time that runs - the memory card is written synchronously, the log is
 * flushed per line, and gfx::shutdown (which stops the pipeline-cache writer) comes before the
 * WebGPU part of aurora_shutdown. So the teardown is run under a structured-exception guard, and
 * the process then ends with TerminateProcess(0): the CRT's exit path would run static destructors
 * that release the same Dawn objects a second time. */
static void gw_aurora_shutdown_guarded(void) {
  __try {
    aurora_shutdown();
  } __except (EXCEPTION_EXECUTE_HANDLER) {
    gw_log("melee-pc: aurora teardown faulted inside Dawn (0x%08lX); ignored at exit",
           (unsigned long)GetExceptionCode());
  }
}

void gw_exit_clean(int code) {
  gw_aurora_shutdown_guarded();
  gw_log("melee-pc: exit %d", code);
  fflush(NULL);
  TerminateProcess(GetCurrentProcess(), (UINT)code);
}

extern void gw_pad_focus_event(int focused); /* shim_pad.c */
extern void gw_mouse_event(const void *sdl_event); /* gw_console.cpp: menus and gd.mouse */
extern void gw_pad_focus_tick(void);

static void gw_handle_events(void) {
  const AuroraEvent *event = aurora_update();
  while (event != NULL && event->type != AURORA_NONE) {
    switch (event->type) {
    case AURORA_EXIT:
      gw_exiting = true;
      break;
    case AURORA_SDL_EVENT:
      gw_mouse_event(&event->sdl);
      /* focus: shim_pad.c releases the GC adapter in the background (debounced there) */
      if (event->sdl.type == SDL_EVENT_WINDOW_FOCUS_LOST) {
        gw_pad_focus_event(0);
      } else if (event->sdl.type == SDL_EVENT_WINDOW_FOCUS_GAINED) {
        gw_pad_focus_event(1);
      }
      break;
    default:
      break;
    }
    ++event;
  }
  gw_pad_focus_tick();
  if (gw_exiting) {
    gw_log("melee-pc: window closed, shutting down");
    gw_dump_stub_summary();
    if (gw_frame_begun) {
      aurora_end_frame();
      gw_frame_begun = false;
    }
    gw_exit_clean(0);
  }
}

/* ---- video settings ----------------------------------------------------------------------
 * Render scale: the internal resolution as a multiple of the game's own 640x480 EFB, like
 * Dolphin's Internal Resolution (1 = native, 2 = 1280x960, 3 = 1920x1440, fractions allowed),
 * independent of the window. 0 = "Auto": render at the window's own pixel size (what the port has
 * always done). This goes through Aurora's VISetFrameBufferScale, which sizes Aurora's render
 * targets; the game's GXRenderModeObj is never touched, so layout, EFB copies and XFB behave
 * exactly as at native resolution and only the pixel density changes.
 *
 * Sources, strongest first: MELEE_RENDER_SCALE, then video.cfg next to the exe (MELEE_VIDEO_CFG
 * overrides the path), then 0. gw_Video_SetRenderScale changes it live and saves video.cfg, for the
 * frontend's menu row. */
static float gw_video_scale;
static int gw_video_loaded;
/* Frame rate: 60 = the game's own rate, one presented frame per simulated frame (default).
 * 0 = uncapped and N > 60 = at most N presents a second: the simulation stays at exactly 60 Hz and
 * the frames in between are the last game frame drawn again with interpolated transforms
 * (gw_uncap_*, aurora_frame_replay). */
static int gw_video_fps = 60;
static int gw_video_vsync = 1;
static int gw_video_show_fps;

static const char *gw_video_cfg_path(void) {
  static char buf[MAX_PATH];
  const char *env = getenv("MELEE_VIDEO_CFG");
  char *slash;
  if (env != NULL && env[0] != '\0') {
    return env;
  }
  if (buf[0] == '\0') {
    DWORD n = GetModuleFileNameA(NULL, buf, (DWORD)sizeof buf);
    if (n == 0 || n >= sizeof buf || (slash = strrchr(buf, '\\')) == NULL) {
      strcpy(buf, "video.cfg");
    } else {
      strcpy(slash + 1, "video.cfg");
    }
  }
  return buf;
}

static float gw_video_clamp_scale(float v) {
  if (!(v > 0.0f)) {
    return 0.0f; /* 0, negative or NaN: Auto */
  }
  if (v < 0.5f) {
    v = 0.5f;
  }
  if (v > 8.0f) {
    v = 8.0f; /* 5120x3840: the largest a D3D11 texture this format comfortably allows */
  }
  return v;
}

static void gw_video_load(void) {
  FILE *f;
  char line[128];
  const char *env;
  if (gw_video_loaded) {
    return;
  }
  gw_video_loaded = 1;
  f = fopen(gw_video_cfg_path(), "r");
  if (f != NULL) {
    while (fgets(line, sizeof line, f) != NULL) {
      float v;
      int iv;
      if (sscanf(line, " render_scale = %f", &v) == 1) {
        gw_video_scale = gw_video_clamp_scale(v);
      } else if (sscanf(line, " fps = %d", &iv) == 1) {
        gw_video_fps = iv;
      } else if (sscanf(line, " vsync = %d", &iv) == 1) {
        gw_video_vsync = iv != 0;
      } else if (sscanf(line, " show_fps = %d", &iv) == 1) {
        gw_video_show_fps = iv != 0;
      }
    }
    fclose(f);
  }
  env = getenv("MELEE_RENDER_SCALE");
  if (env != NULL && env[0] != '\0') {
    gw_video_scale = (env[0] == 'a' || env[0] == 'A') ? 0.0f : gw_video_clamp_scale((float)atof(env));
  }
  env = getenv("MELEE_FPS");
  if (env != NULL && env[0] != '\0') {
    gw_video_fps = (env[0] == 'u' || env[0] == 'U') ? 0 : atoi(env);
  }
  if (gw_video_fps != 0 && gw_video_fps <= 60) {
    gw_video_fps = 60;
  }
  env = getenv("MELEE_VSYNC");
  if (env != NULL && env[0] != '\0') {
    gw_video_vsync = atoi(env) != 0;
  }
  env = getenv("MELEE_SHOW_FPS");
  if (env != NULL && env[0] != '\0') {
    gw_video_show_fps = atoi(env) != 0;
  }
}

static void gw_video_save(void) {
  FILE *f = fopen(gw_video_cfg_path(), "w");
  if (f == NULL) {
    gw_log("gw: video: cannot write %s", gw_video_cfg_path());
    return;
  }
  fprintf(f, "# melee-pc video settings (the game rewrites this file)\n");
  fprintf(f, "# render_scale: internal resolution x 640x480; 0 = match the window\n");
  fprintf(f, "render_scale = %g\n", gw_video_scale);
  fprintf(f, "# fps: 60 = one frame per game frame; 0 = uncapped; N = at most N (interpolated)\n");
  fprintf(f, "fps = %d\n", gw_video_fps);
  fprintf(f, "vsync = %d\n", gw_video_vsync);
  fprintf(f, "show_fps = %d\n", gw_video_show_fps);
  fclose(f);
}

static void gw_video_apply_scale(void) {
  VISetFrameBufferScale(gw_video_scale);
  /* Supersampled (scale >= 2, larger than a usual window) is shrunk to the window with an area
   * filter - bilinear only reads the 2x2 texels nearest each output pixel, so fine texture lines
   * alias into moire stripes at 3x. Upscaling (1x, fractional) stays bilinear. */
  aurora_set_resampler(gw_video_scale >= 2.0f ? SAMPLER_AREA : SAMPLER_BILINEAR);
  if (gw_video_scale > 0.0f) {
    gw_log("gw: video: render scale %gx (%dx%d internal)", gw_video_scale,
           (int)(640.0f * gw_video_scale + 0.5f), (int)(480.0f * gw_video_scale + 0.5f));
  } else {
    gw_log("gw: video: render scale Auto (the window's pixel size)");
  }
}

/* Aurora entry points added by this port's Aurora patch (PORT_PATCHES.md group 5). An older
 * aurora_gx.lib resolves them to these instead: no replay is ever available, so frame
 * interpolation quietly stays off and the build still links. */
void gw_aurora_frame_replay_enable_none(bool v) { (void)v; }
bool gw_aurora_frame_replay_available_none(void) { return false; }
void gw_aurora_frame_set_alpha_none(float v) { (void)v; }
void gw_aurora_frame_replay_mark_none(bool v) { (void)v; }
bool gw_aurora_frame_replay_none(float v) { (void)v; return false; }
bool gw_aurora_frame_slot_available_none(void) { return true; }
void gw_aurora_frame_interp_stats_none(uint32_t *b, uint32_t *r, uint32_t *m) { *b = *r = *m = 0; }
#pragma comment(linker, "/alternatename:_aurora_frame_interp_stats=_gw_aurora_frame_interp_stats_none")
#pragma comment(linker, "/alternatename:_aurora_frame_replay_enable=_gw_aurora_frame_replay_enable_none")
#pragma comment(linker, "/alternatename:_aurora_frame_replay_available=_gw_aurora_frame_replay_available_none")
#pragma comment(linker, "/alternatename:_aurora_frame_set_alpha=_gw_aurora_frame_set_alpha_none")
#pragma comment(linker, "/alternatename:_aurora_frame_replay_mark=_gw_aurora_frame_replay_mark_none")
#pragma comment(linker, "/alternatename:_aurora_frame_replay=_gw_aurora_frame_replay_none")
#pragma comment(linker, "/alternatename:_aurora_frame_slot_available=_gw_aurora_frame_slot_available_none")

static int gw_uncap_on(void) { return gw_video_fps != 60; }

static void gw_video_apply_rate(void) {
  const int on = gw_uncap_on();
  aurora_frame_replay_enable(on);
  /* The game's own frame is drawn at the previous frame's pose (blend 0); the replays that follow
   * move it toward its own pose as the field goes by. Off: no blending at all. */
  aurora_frame_set_alpha(on ? 0.0f : 1.0f);
  if (on && gw_video_fps == 0) {
    gw_log("gw: video: frame rate uncapped (simulation stays 60 Hz; in-between frames interpolated)");
  } else if (on) {
    gw_log("gw: video: frame rate up to %d (simulation stays 60 Hz; in-between frames interpolated)",
           gw_video_fps);
  }
}

/* Frame rate: 60 = the game's own (default); 0 = uncapped; N > 60 = at most N presents a second. */
int gw_Video_FrameRate(void) {
  gw_video_load();
  return gw_video_fps;
}

void gw_Video_SetFrameRate(int fps) {
  gw_video_load();
  gw_video_fps = (fps != 0 && fps <= 60) ? 60 : fps;
  gw_video_apply_rate();
  gw_video_save();
}

int gw_Video_Vsync(void) {
  gw_video_load();
  return gw_video_vsync;
}

void gw_Video_SetVsync(int on) {
  gw_video_load();
  gw_video_vsync = on != 0;
  aurora_enable_vsync(gw_video_vsync != 0);
  gw_video_save();
}

int gw_Video_ShowFps(void) {
  gw_video_load();
  return gw_video_show_fps;
}

void gw_Video_SetShowFps(int on) {
  gw_video_load();
  gw_video_show_fps = on != 0;
  gw_video_save();
}

/* ---- screenshots ----------------------------------------------------------------------------
 * gw_Screenshot(path) writes the next presented frame to a PNG: the final image at the render
 * scale, without the host ImGui overlay (the game's own DevText overlay is part of the frame).
 * Aurora copies it into a readback buffer inside the frame's own command buffer and a worker thread
 * writes the file, so no frame waits; the file appears a few frames later.
 * MELEE_SHOT_AT="<frame>:<path>[,<frame>:<path>...]" takes shots unattended; <frame> counts game
 * frames presented since boot (gw_presented_count, the number the heartbeat log prints). */
void gw_aurora_request_screenshot_none(const char *path) {
  gw_log("gw: screenshot: this aurora_gx.lib has no screenshot support (%s not written)", path);
}
#pragma comment(linker, "/alternatename:_aurora_request_screenshot=_gw_aurora_request_screenshot_none")

void gw_Screenshot(const char *path) {
  if (path == NULL || path[0] == '\0') {
    return;
  }
  gw_log("gw: screenshot requested: %s", path);
  aurora_request_screenshot(path);
}

#define GW_SHOT_MAX 32
static struct {
  uint32_t frame;
  char path[MAX_PATH];
} gw_shots[GW_SHOT_MAX];
static int gw_shot_count, gw_shot_next, gw_shot_loaded;

static void gw_shots_load(void) {
  const char *env = getenv("MELEE_SHOT_AT");
  gw_shot_loaded = 1;
  while (env != NULL && *env != '\0' && gw_shot_count < GW_SHOT_MAX) {
    const char *colon = strchr(env, ':');
    const char *comma;
    size_t n;
    if (colon == NULL) {
      break;
    }
    /* a Windows drive letter is part of the path: the frame number is everything before the
     * first colon, and the path runs to the next comma */
    comma = strchr(colon + 1, ',');
    n = comma != NULL ? (size_t)(comma - (colon + 1)) : strlen(colon + 1);
    if (n >= MAX_PATH) {
      n = MAX_PATH - 1;
    }
    gw_shots[gw_shot_count].frame = (uint32_t)strtoul(env, NULL, 10);
    memcpy(gw_shots[gw_shot_count].path, colon + 1, n);
    gw_shots[gw_shot_count].path[n] = '\0';
    ++gw_shot_count;
    env = comma != NULL ? comma + 1 : NULL;
  }
  if (gw_shot_count > 0) {
    gw_log("gw: MELEE_SHOT_AT: %d screenshot(s) scheduled", gw_shot_count);
  }
}

/* Once per presented game frame, before aurora_end_frame. */
static void gw_shots_tick(uint32_t frame) {
  int i;
  if (!gw_shot_loaded) {
    gw_shots_load();
  }
  for (i = 0; i < gw_shot_count; ++i) {
    if (gw_shots[i].frame == frame) {
      gw_Screenshot(gw_shots[i].path);
    }
  }
  (void)gw_shot_next;
}

/* The current render scale; 0 = Auto (window resolution). */
float gw_Video_RenderScale(void) {
  gw_video_load();
  return gw_video_scale;
}

/* Change the render scale now (Aurora resizes its targets at the next frame boundary) and remember
 * it in video.cfg. 0 = Auto; otherwise clamped to 0.5..8. */
void gw_Video_SetRenderScale(float scale) {
  gw_video_load();
  gw_video_scale = gw_video_clamp_scale(scale);
  gw_video_apply_scale();
  gw_video_save();
}

bool gw_frame_init(void) {
  gw_video_load();
  gw_video_apply_scale();
  gw_video_apply_rate();
  if (!gw_video_vsync) {
    aurora_enable_vsync(false);
  }
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
static double gw_prof_sum_pace, gw_prof_sum_submit; /* present = overlay + pace + submit */
static long long gw_prof_t_pace0, gw_prof_t_pace1;
static double gw_prof_sum_pace_after; /* pacing after the submit (the default order) */
static uint32_t gw_prof_frames, gw_prof_empty;
static uint32_t gw_prof_hist[GW_PROF_BUCKETS];
static int gw_prof_spikes;

/* Upper bounds in ms. 16.67 is the target; the buckets either side of it are what matter. */
static const double gw_prof_edges[GW_PROF_BUCKETS] = { 14.0, 16.0, 17.5, 20.0, 25.0, 34.0, 50.0, 1e9 };

static uint32_t gw_wait_idle_count;

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
  {
    extern uint32_t gw_gx_texobj_inits;
    static uint32_t last_inits;
    gw_log("gw: PROF texobj   inits/frame=%.1f (each a new aurora cache entry kept 600 frames)",
           (double)(gw_gx_texobj_inits - last_inits) / gw_prof_frames);
    last_inits = gw_gx_texobj_inits;
  }
  gw_log("gw: PROF present   pace_wait=%.2f submit=%.2f (aurora_end_frame; >1 ms = render back-pressure)"
         "  of which pacing after submit=%.2f",
         gw_prof_sum_pace / gw_prof_frames, gw_prof_sum_submit / gw_prof_frames,
         gw_prof_sum_pace_after / gw_prof_frames);
  gw_log("gw: PROF cpu ms    game_cpu=%.2f  of_game_wall=%.2f  wait=%.2f",
         gw_prof_sum_cpu / gw_prof_frames, gw_prof_sum_game / gw_prof_frames,
         (gw_prof_sum_game - gw_prof_sum_cpu) / gw_prof_frames);
  gw_log("gw: PROF hist      <14:%u  <16:%u  <17.5:%u  <20:%u  <25:%u  <34:%u  <50:%u  50+:%u",
         gw_prof_hist[0], gw_prof_hist[1], gw_prof_hist[2], gw_prof_hist[3], gw_prof_hist[4],
         gw_prof_hist[5], gw_prof_hist[6], gw_prof_hist[7]);

  gw_prof_sum_game = gw_prof_sum_present = gw_prof_sum_events = gw_prof_sum_begin = 0.0;
  gw_prof_sum_cpu = 0.0;
  gw_prof_sum_pace = gw_prof_sum_submit = gw_prof_sum_pace_after = 0.0;
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


/* ---- input-latency profiler (MELEE_INPUT_PROFILE=1) ---------------------------------------
 * Measures the path from a controller report to the frame that shows it, in the port's own
 * terms, every 600 presented frames:
 *   adapter ivl   spacing of the GC adapter's reports (gc_adapter.c; 8 ms stock, 1 ms overclocked)
 *   report age    how old the newest adapter report is when the game polls (gw_PADRead)
 *   alarm late    how late the 60 Hz pad alarm fired against its deadline (shim_os.c) - the extra
 *                 age of the sample on top of the report age
 *   sdl age       time since SDL last pumped events (aurora_update): how stale Aurora's SDL pads
 *                 are at the poll
 *   poll->drawn   from the newest poll the frame's logic consumed to the game finishing that
 *                 frame's GX commands (aurora_end_frame is called)
 *   end_frame     aurora_end_frame itself: mostly waiting for Aurora's FIFO thread to finish
 *                 turning the frame's GX commands into draws
 *   poll->queued  poll to aurora_end_frame returning: the frame is with the render worker, which
 *                 encodes, acquires the swapchain image, blits and presents asynchronously
 *   poll->present poll to the render worker returning from Present() for that frame; the image
 *                 then waits for the display's next refresh under vsync (<= 6.9 ms at 144 Hz)
 *   submit ivl    frame-to-frame spacing of aurora_end_frame calls
 *   pace over     how far past the 60 Hz boundary the pacing wait woke
 *   begin wait    aurora_begin_frame: frame-slot back-pressure from the render worker/swapchain
 *   polls/frame   pad polls between two presents (anything but 1 is a doubled or dropped input frame)
 *   queue         the raw pad queue depth (HSD_PadLibData.qcount) when a present returns to the game:
 *                 every entry beyond one is a whole frame of added input latency */
#define GW_INPROF_N 600
typedef struct {
  float v[GW_INPROF_N];
  int n;
} gw_inprof_ring;

static int gw_inprof_on(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_INPUT_PROFILE");
    cached = (v != NULL && v[0] == '1') ? 1 : 0;
  }
  return cached;
}

static gw_inprof_ring gw_ip_age, gw_ip_late, gw_ip_sdl, gw_ip_p2s, gw_ip_ivl, gw_ip_over, gw_ip_begin;
static gw_inprof_ring gw_ip_p2q, gw_ip_endf, gw_ip_p2p;
static uint32_t gw_presented_count; /* defined with the frame driver below */
static uint32_t gw_end_frames;      /* every aurora_end_frame, interpolated replays included */
static void gw_ip_push(gw_inprof_ring *r, double v);
static long long gw_ip_submit_cand;

/* Present times come from Aurora's render worker (aurora_get_last_present_ns, added by this port).
 * An aurora_gx.lib from before that addition resolves them to these instead, which report nothing,
 * so the shim still links against either. */
int64_t gw_aurora_last_present_ns_none(void) { return 0; }
uint32_t gw_aurora_present_count_none(void) { return 0; }
#pragma comment(linker, "/alternatename:_aurora_get_last_present_ns=_gw_aurora_last_present_ns_none")
#pragma comment(linker, "/alternatename:_aurora_get_present_count=_gw_aurora_present_count_none")
#define GW_IP_PRESENT_RING 64
static long long gw_ip_ring_poll[GW_IP_PRESENT_RING];
static uint32_t gw_ip_ring_id[GW_IP_PRESENT_RING];
static uint32_t gw_ip_presents_seen;

/* QPC -> the nanosecond scale of MSVC's steady_clock, which is what Aurora stamps presents with. */
static long long gw_ip_qpc_ns(long long c) {
  static long long f;
  if (f == 0) {
    LARGE_INTEGER q;
    QueryPerformanceFrequency(&q);
    f = q.QuadPart;
  }
  return (c / f) * 1000000000ll + ((c % f) * 1000000000ll) / f;
}

/* Called once a frame: attribute a newly completed Present() to the frame it showed. The n-th
 * present is the n-th aurora_end_frame, barring a frame Aurora could not present. */
static void gw_ip_check_present(void) {
  const uint32_t n = aurora_get_present_count();
  if (n == 0 || n == gw_ip_presents_seen) {
    return;
  }
  gw_ip_presents_seen = n;
  if (gw_ip_ring_id[n % GW_IP_PRESENT_RING] == n) {
    const long long pres = aurora_get_last_present_ns();
    const long long poll = gw_ip_qpc_ns(gw_ip_ring_poll[n % GW_IP_PRESENT_RING]);
    if (pres > poll) {
      gw_ip_push(&gw_ip_p2p, (double)(pres - poll) / 1e6);
    }
  }
}
static uint32_t gw_ip_polls_hist[4], gw_ip_queue_hist[4];
static uint32_t gw_ip_polls_this_frame, gw_ip_late_frames;
static long long gw_ip_last_poll, gw_ip_cand, gw_ip_last_submit, gw_ip_last_events;
static int gw_ip_have_cand, gw_ip_waiting_first;
extern uint64_t gw_os_alarm_late_ticks;
extern unsigned char gw_HSD_PadLibData[];

static void gw_ip_push(gw_inprof_ring *r, double v) {
  if (r->n < GW_INPROF_N) {
    r->v[r->n++] = (float)v;
  }
}

static int gw_ip_cmpf(const void *a, const void *b) {
  const float x = *(const float *)a, y = *(const float *)b;
  return (x < y) ? -1 : ((x > y) ? 1 : 0);
}

static void gw_ip_line(const char *name, gw_inprof_ring *r) {
  if (r->n == 0) {
    gw_log("gw: INPROF %-12s (no samples)", name);
    return;
  }
  {
    const int n = r->n;
    double sum = 0.0;
    int i;
    for (i = 0; i < n; ++i) {
      sum += r->v[i];
    }
    qsort(r->v, (size_t)n, sizeof(float), gw_ip_cmpf);
    gw_log("gw: INPROF %-12s ms  mean=%.2f p50=%.2f p95=%.2f p99=%.2f max=%.2f min=%.2f (n=%d)", name,
           sum / n, r->v[(int)(0.50 * (n - 1))], r->v[(int)(0.95 * (n - 1))],
           r->v[(int)(0.99 * (n - 1))], r->v[n - 1], r->v[0], n);
  }
  r->n = 0;
}

static void gw_inprof_report(void) {
  extern int gw_gc_adapter_present(void);
  extern void gw_gc_adapter_take_stats(unsigned *hist9, unsigned *count, unsigned *changed,
                                       double *min_ms, double *max_ms);
  if (gw_gc_adapter_present()) {
    unsigned h[9], count, changed;
    double mn, mx;
    gw_gc_adapter_take_stats(h, &count, &changed, &mn, &mx);
    gw_log("gw: INPROF adapter ivl ms  <0.75:%u <1.5:%u <3:%u <5:%u <7:%u <9:%u <12:%u <20:%u 20+:%u"
           "  min=%.2f max=%.2f reports=%u changed=%u",
           h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7], h[8], mn, mx, count, changed);
  } else {
    gw_log("gw: INPROF adapter      (no GC adapter open)");
  }
  gw_ip_line("report age", &gw_ip_age);
  gw_ip_line("alarm late", &gw_ip_late);
  gw_ip_line("sdl age", &gw_ip_sdl);
  gw_ip_line("poll->drawn", &gw_ip_p2s);
  gw_ip_line("end_frame", &gw_ip_endf);
  gw_ip_line("poll->queued", &gw_ip_p2q);
  gw_ip_line("poll->present", &gw_ip_p2p);
  gw_ip_line("submit ivl", &gw_ip_ivl);
  gw_ip_line("pace over", &gw_ip_over);
  gw_ip_line("begin wait", &gw_ip_begin);
  gw_log("gw: INPROF polls/frame  0:%u 1:%u 2:%u 3+:%u   queue at return 0:%u 1:%u 2:%u 3+:%u"
         "   frames already late at pacing: %u",
         gw_ip_polls_hist[0], gw_ip_polls_hist[1], gw_ip_polls_hist[2], gw_ip_polls_hist[3],
         gw_ip_queue_hist[0], gw_ip_queue_hist[1], gw_ip_queue_hist[2], gw_ip_queue_hist[3],
         gw_ip_late_frames);
  memset(gw_ip_polls_hist, 0, sizeof gw_ip_polls_hist);
  memset(gw_ip_queue_hist, 0, sizeof gw_ip_queue_hist);
  gw_ip_late_frames = 0;
}

/* gw_PADRead, once per poll, after the adapter has been read (always called: the pacer counts
 * polls; the profiling below is MELEE_INPUT_PROFILE only). */
static uint32_t gw_polls_since_pace; /* gw_pace_field: pad polls since the last wait ended */
static uint64_t gw_last_poll_tick;    /* when the newest game frame's pad sample was taken */

void gw_inprof_poll(void) {
  extern long long gw_gc_adapter_report_qpc(void);
  long long now, rep;
  ++gw_polls_since_pace;
  gw_last_poll_tick = gw_time_ticks();
  if (!gw_inprof_on()) {
    return;
  }
  now = gw_prof_now();
  rep = gw_gc_adapter_report_qpc();
  if (rep != 0) {
    gw_ip_push(&gw_ip_age, gw_prof_ms(rep, now));
  }
  gw_ip_push(&gw_ip_late, (double)gw_os_alarm_late_ticks / (GW_TIMER_CLOCK / 1000.0));
  if (gw_ip_last_events != 0) {
    gw_ip_push(&gw_ip_sdl, gw_prof_ms(gw_ip_last_events, now));
  }
  ++gw_ip_polls_this_frame;
  gw_ip_last_poll = now;
  if (gw_ip_waiting_first) {
    gw_ip_cand = now;
    gw_ip_have_cand = 1;
    gw_ip_waiting_first = 0;
  }
}

/* frame tick, immediately before aurora_end_frame. */
static void gw_inprof_submit(void) {
  const long long now = gw_prof_now();
  gw_ip_check_present();
  if (gw_ip_last_submit != 0) {
    gw_ip_push(&gw_ip_ivl, gw_prof_ms(gw_ip_last_submit, now));
  }
  gw_ip_last_submit = now;
  gw_ip_submit_cand = 0;
  if (gw_ip_have_cand) {
    gw_ip_push(&gw_ip_p2s, gw_prof_ms(gw_ip_cand, now));
    gw_ip_submit_cand = gw_ip_cand;
    gw_ip_have_cand = 0;
  }
  ++gw_ip_polls_hist[gw_ip_polls_this_frame < 3 ? gw_ip_polls_this_frame : 3];
  gw_ip_polls_this_frame = 0;
}

/* Right after aurora_end_frame returns: the GX FIFO has been drained (Aurora's FIFO thread has
 * turned every command into draws) and the frame is queued to the render worker, which only has
 * to encode, acquire the swapchain image, blit and present. */
static void gw_inprof_submitted(void) {
  const long long now = gw_prof_now();
  gw_ip_push(&gw_ip_endf, gw_prof_ms(gw_ip_last_submit, now));
  if (gw_ip_submit_cand != 0) {
    const uint32_t id = gw_end_frames; /* this end_frame's 1-based number, replays included */
    gw_ip_push(&gw_ip_p2q, gw_prof_ms(gw_ip_submit_cand, now));
    gw_ip_ring_poll[id % GW_IP_PRESENT_RING] = gw_ip_submit_cand;
    gw_ip_ring_id[id % GW_IP_PRESENT_RING] = id;
  }
}

/* End of a presenting frame tick, as control returns to the game loop: the next logic frame
 * consumes whatever is queued now, or else the first poll to arrive. */
static void gw_inprof_return(void) {
  const unsigned q = gw_HSD_PadLibData[3]; /* qcount, a byte: no swap */
  ++gw_ip_queue_hist[q < 3 ? q : 3];
  if (q > 0 && gw_ip_last_poll != 0) {
    gw_ip_cand = gw_ip_last_poll;
    gw_ip_have_cand = 1;
    gw_ip_waiting_first = 0;
  } else {
    gw_ip_have_cand = 0;
    gw_ip_waiting_first = 1;
  }
  if (gw_ip_ivl.n >= GW_INPROF_N) {
    gw_inprof_report();
  }
}

static uint32_t gw_presented_count;

#define GW_PACE_SPIN_TICKS (3u * (GW_TIMER_CLOCK / 1000u)) /* Sleep(1) fallback: spin the final ~3 ms */
#define GW_PACE_SPIN_TICKS_HR (GW_TIMER_CLOCK / 1000u)     /* high-resolution timer: spin ~1 ms */

static uint64_t gw_last_field_tick;
static bool gw_pace_timer_res_set;
static HANDLE gw_pace_timer;

#ifndef CREATE_WAITABLE_TIMER_HIGH_RESOLUTION
#define CREATE_WAITABLE_TIMER_HIGH_RESOLUTION 0x00000002
#endif

/* MELEE_PACE=legacy restores the old frame order (render -> wait for the boundary -> submit), for
 * A/B measurement with MELEE_INPUT_PROFILE. */
static int gw_pace_legacy(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_PACE");
    cached = (v != NULL && v[0] == 'l') ? 1 : 0;
    if (cached) {
      gw_log("gw: pacing: legacy order (wait before submit)");
    }
  }
  return cached;
}

/* Sleep for about `ticks` (at most ~1 ms). A high-resolution waitable timer (Windows 10 1803+)
 * wakes within a few hundred microseconds; Sleep(1) under timeBeginPeriod(1) can overshoot by a
 * millisecond or more, which is why the spin that follows it has to be longer. */
static void gw_pace_nap(uint64_t ticks) {
  if (gw_pace_timer != NULL) {
    LARGE_INTEGER due;
    due.QuadPart = -(LONGLONG)((ticks * 10000000ull) / GW_TIMER_CLOCK); /* 100 ns units, relative */
    if (due.QuadPart < -10000) {
      due.QuadPart = -10000;
    }
    if (due.QuadPart > -1) {
      due.QuadPart = -1;
    }
    if (SetWaitableTimer(gw_pace_timer, &due, 0, NULL, NULL, FALSE)) {
      WaitForSingleObject(gw_pace_timer, 5);
      return;
    }
  }
  Sleep(1);
}

/* Field pacing: hold the game to 60 Hz. A GameCube's VI gives the game a hard 16.667 ms boundary;
 * here the present is asynchronous (aurora_end_frame only enqueues to the render worker) and a
 * VRR display lets Present() return without back-pressure, so without this nothing holds the game
 * to 60 Hz.
 *
 * WHERE the wait sits is what decides input latency. It used to sit between the render and the
 * submit: the game polled the pad, ran its logic and rendered in ~1-2 ms, and then the finished
 * frame waited out the rest of the field (~14 ms) before aurora_end_frame. Measured with
 * MELEE_INPUT_PROFILE on ACE: poll -> submit 17.4-18.5 ms, every frame. Now the frame is submitted
 * the moment it is drawn, and the wait happens BEFORE the next frame instead - and it waits for
 * the deadline of the game's own 60 Hz pad alarm (lb_80019628), so the controllers are sampled
 * the instant the wait ends and logic, render and submit follow at once. Nothing about the
 * simulation changes: the same alarm still produces exactly one pad sample per logic frame.
 *
 * If a sample was already taken mid-frame (a slow frame let the deadline pass), there is nothing
 * to wait for - the game should consume it now rather than sit on a stale input. Without an armed pad
 * alarm (boot, some scene transitions) the old free-running 60 Hz grid applies.
 *
 * The wait naps on a high-resolution waitable timer in <=1 ms steps, pumping alarms and deferred
 * work between naps as before, and spins only the final ~1 ms (3 ms with the Sleep(1) fallback),
 * so the core is released for almost all of the field. */
/* ---- uncapped frame rate: interpolated in-between frames -----------------------------------
 * The simulation never runs faster than the 60 Hz pad alarm. While gw_pace_field waits for the
 * next pad deadline, the last game frame is drawn again (aurora_frame_replay) with every position/
 * normal matrix and projection blended between the previous game frame's value and its own, by how
 * far into the field the replay starts. The game's own frame is drawn at blend 0, so what is on
 * screen trails the simulation by one field (16.7 ms) - the price of interpolation, which is why
 * it is opt-in. A replay only starts if it can finish before the deadline and the render worker
 * has a free frame slot, so the pad sample, and so input latency of the game itself, is never
 * delayed by it. */
static uint64_t gw_uncap_cost = 6u * (GW_TIMER_CLOCK / 1000u); /* EMA of one replay's game-thread cost */
static uint64_t gw_uncap_next;                                 /* earliest start of the next present (cap) */
static uint32_t gw_stat_presents, gw_stat_sims, gw_stat_replays;
static uint64_t gw_stat_t0;
static char gw_stat_text[96];

static void gw_stats_note_present(int replay) {
  const uint64_t now = gw_time_ticks();
  ++gw_stat_presents;
  if (replay) {
    ++gw_stat_replays;
  } else {
    ++gw_stat_sims;
  }
  if (gw_stat_t0 == 0) {
    gw_stat_t0 = now;
  } else if (now - gw_stat_t0 >= GW_TIMER_CLOCK / 2u) {
    const double secs = (double)(now - gw_stat_t0) / GW_TIMER_CLOCK;
    snprintf(gw_stat_text, sizeof gw_stat_text, "%.0f fps  %.2f ms   game %.0f Hz%s", gw_stat_presents / secs,
             gw_stat_presents ? secs * 1000.0 / gw_stat_presents : 0.0, gw_stat_sims / secs,
             gw_uncap_on() ? "  interp" : "");
    {
      static int windows;
      if (gw_uncap_on() && (++windows % 20) == 0) {
        uint32_t b, r, m;
        aurora_frame_interp_stats(&b, &r, &m);
        gw_log("gw: interp: %s | matrix loads blended %u, implausible pairing %u, unmatched %u"
               " | replay cost %.2f ms",
               gw_stat_text, b, r, m, (double)gw_uncap_cost * 1000.0 / GW_TIMER_CLOCK);
      }
    }
    gw_stat_presents = gw_stat_sims = gw_stat_replays = 0;
    gw_stat_t0 = now;
  }
}

static void gw_stats_draw(void) {
  /* Geno LAB (private): no plain debug text over a LAB match; the Lab draws its own kit HUD */
  extern int gw_GenoLab_InMatch(void);
  if (gw_video_show_fps && gw_stat_text[0] != '\0' && !gw_GenoLab_InMatch()) {
    gw_Overlay_DrawStats(gw_stat_text);
  }
}

static void gw_uncap_replay(uint64_t now) {
  const uint64_t t0 = now;
  float alpha = (float)((double)(now - gw_last_poll_tick) / (double)GW_TICKS_PER_FIELD);
  if (alpha < 0.0f) {
    alpha = 0.0f;
  }
  if (alpha > 1.0f) {
    alpha = 1.0f;
  }
  aurora_frame_replay_mark(true);
  if (aurora_begin_frame()) {
    gw_Overlay_Draw();
    gw_stats_draw();
    aurora_frame_replay(alpha);
    aurora_end_frame();
    ++gw_end_frames;
    gw_stats_note_present(1);
  }
  aurora_frame_replay_mark(false);
  now = gw_time_ticks();
  gw_uncap_cost = (gw_uncap_cost * 7u + (now - t0)) / 8u;
  gw_uncap_next = gw_video_fps > 60 ? t0 + GW_TIMER_CLOCK / (uint64_t)gw_video_fps : 0;
}

/* Called from the pacing wait: draw an in-between frame if there is time for one. */
static int gw_uncap_try(uint64_t now, uint64_t target) {
  if (!gw_uncap_on() || gw_pace_legacy() || !aurora_frame_replay_available()) {
    return 0;
  }
  if (target <= now || target - now < gw_uncap_cost + GW_TIMER_CLOCK / 1000u) {
    return 0;
  }
  if (gw_uncap_next != 0 && now < gw_uncap_next) {
    return 0;
  }
  if (!aurora_frame_slot_available()) {
    return 0;
  }
  gw_uncap_replay(now);
  return 1;
}

static void gw_pace_field(void) {
  uint64_t now = gw_time_ticks();
  uint64_t last_pump;
  uint64_t spin = GW_PACE_SPIN_TICKS;
  uint64_t target;
  uint64_t deadline;
  if (gw_last_field_tick == 0) {
    gw_last_field_tick = now;
    return;
  }
  if (!gw_pace_timer_res_set) {
    /* Raise the OS timer resolution for the life of the process, so the Sleep(1) fallback wakes
     * in ~1-2 ms rather than the ~15.6 ms default tick. Windows resets a process's timer
     * resolution request automatically on exit, so there is no matching timeEndPeriod. */
    timeBeginPeriod(1);
    gw_pace_timer = CreateWaitableTimerExW(NULL, NULL, CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                                           TIMER_ALL_ACCESS);
    gw_log("gw: pacing: %s", gw_pace_timer != NULL ? "high-resolution waitable timer"
                                                   : "Sleep(1) (no high-resolution timer)");
    gw_pace_timer_res_set = true;
  }
  if (gw_pace_timer != NULL) {
    spin = GW_PACE_SPIN_TICKS_HR;
  }
  target = gw_last_field_tick + GW_TICKS_PER_FIELD;
  if (!gw_pace_legacy() && gw_os_pad_alarm_deadline(GW_TICKS_PER_FIELD, &deadline)) {
    target = deadline;
    /* Normally exactly one poll happened since the last wait ended: the one right after it, which
     * this frame's logic consumed. A second one means the frame ran long, the deadline passed and
     * the alarm fired mid-frame: that sample is waiting, so go now. (Not "the pad queue is
     * non-empty": a rollback stall or time-sync tick leaves the queue alone on purpose, and those
     * ticks must still take one field each.) */
    if (gw_polls_since_pace >= 2u || target > now + 2u * GW_TICKS_PER_FIELD) {
      target = now;
    }
  }
  {
    if (gw_inprof_on() && now >= target) {
      ++gw_ip_late_frames;
    }
    last_pump = now;
    while ((now = gw_time_ticks()) < target) {
      if (now - last_pump >= GW_TIMER_CLOCK / 1000u) {
        last_pump = now;
        gw_os_run_alarms(now);
        gw_run_deferred();
      }
      if (gw_uncap_try(now, target)) {
        continue;
      }
      if (target - now > spin) {
        uint64_t nap = target - now - spin;
        gw_pace_nap(nap < GW_TIMER_CLOCK / 1000u ? nap : GW_TIMER_CLOCK / 1000u);
      } else {
        YieldProcessor();
      }
    }
    if (gw_inprof_on() && now >= target) {
      gw_ip_push(&gw_ip_over, (double)(now - target) / (GW_TIMER_CLOCK / 1000.0));
    }
  }
  gw_last_field_tick = now;
  gw_polls_since_pace = 0;
}

/* ---- sampling profiler (MELEE_PROFILE_SAMPLE=<start seconds>) ------------------------------
 * A background thread suspends the frame thread about once a millisecond, reads its EIP and
 * counts it. Every 5 s it rewrites melee-pc.samples (in the working directory) with the running
 * totals as `count module offset` lines; tools/port/prof_report.py resolves the exe's offsets
 * against melee-pc.map. The start delay skips boot and menus so a scripted match is what gets
 * measured. Samples that land in Sleep (frame pacing) show up under ntdll/KERNELBASE and are
 * idle, not work. Off unless the variable is set. */
#define GW_SAMPLE_SLOTS 131072u
#define GW_SAMPLE_THREADS 128
static DWORD gw_sample_main_tid;
static struct { DWORD tid; HANDLE h; } gw_sample_thr[GW_SAMPLE_THREADS];
static int gw_sample_nthr;
static struct { uint32_t eip, caller, tidx, count; } gw_sample_tab[GW_SAMPLE_SLOTS];
static uint32_t gw_sample_text_lo, gw_sample_text_hi; /* melee-pc.exe's code section */
static uint32_t gw_sample_total[GW_SAMPLE_THREADS];

/* MELEE_PROFILE_SPIKE=<ms>: the frame thread's samples also go into a timestamped ring, and when a
 * frame's game time exceeds <ms> the frame's window of that ring is copied into a spike-only table
 * (melee-pc.spike.samples, same format as melee-pc.samples, thread 0). An ordinary sampled
 * profile is an average over all frames and hides a tail that is 5% of them; this one is the
 * profile of only the slow frames. */
#define GW_SPIKE_RING 8192u
#define GW_SPIKE_SLOTS 16384u
#define GW_CHAIN 8
static struct { long long qpc; uint32_t eip, caller, chain[GW_CHAIN]; } gw_spike_ring[GW_SPIKE_RING];
static struct { uint32_t chain[GW_CHAIN], count; } gw_chain_tab[GW_SPIKE_SLOTS];
static volatile uint32_t gw_spike_ring_head;
static struct { uint32_t eip, caller, count; } gw_spike_tab[GW_SPIKE_SLOTS];
static uint32_t gw_spike_total, gw_spike_frames;
static double gw_spike_threshold_ms;

/* Every thread of the process except the sampler, found again at each dump so threads the
 * renderer starts late are picked up. Thread 0 in the output is always the game (frame) thread. */
static void gw_sample_enum_threads(void) {
  HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
  THREADENTRY32 te;
  if (snap == INVALID_HANDLE_VALUE) {
    return;
  }
  te.dwSize = sizeof te;
  if (Thread32First(snap, &te)) {
    do {
      int i, known = 0;
      if (te.th32OwnerProcessID != GetCurrentProcessId() || te.th32ThreadID == GetCurrentThreadId()) {
        continue;
      }
      for (i = 0; i < gw_sample_nthr; ++i) {
        known |= gw_sample_thr[i].tid == te.th32ThreadID;
      }
      if (!known && gw_sample_nthr < GW_SAMPLE_THREADS) {
        HANDLE h = OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT, FALSE, te.th32ThreadID);
        if (h != NULL) {
          gw_sample_thr[gw_sample_nthr].tid = te.th32ThreadID;
          gw_sample_thr[gw_sample_nthr].h = h;
          ++gw_sample_nthr;
        }
      }
    } while (Thread32Next(snap, &te));
  }
  CloseHandle(snap);
}

static void gw_sample_dump(void) {
  FILE *f = fopen("melee-pc.samples.tmp", "w");
  uint32_t i;
  int t;
  if (f == NULL) {
    return;
  }
  for (t = 0; t < gw_sample_nthr; ++t) {
    fprintf(f, "# thread %d tid %lu total %u\n", t, (unsigned long)gw_sample_thr[t].tid,
            gw_sample_total[t]);
  }
  for (i = 0; i < GW_SAMPLE_SLOTS; ++i) {
    HMODULE mod = NULL;
    char path[MAX_PATH], *base;
    if (gw_sample_tab[i].count == 0u) {
      continue;
    }
    path[0] = '\0';
    if (GetModuleHandleExA(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
                               GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                           (LPCSTR)(uintptr_t)gw_sample_tab[i].eip, &mod) &&
        GetModuleFileNameA(mod, path, sizeof path) != 0) {
      base = strrchr(path, '\\');
      base = (base != NULL) ? base + 1 : path;
    } else {
      base = "?";
      mod = NULL;
    }
    fprintf(f, "%u %u %s %08X %08X\n", gw_sample_tab[i].tidx, gw_sample_tab[i].count, base,
            gw_sample_tab[i].eip - (uint32_t)(uintptr_t)mod,
            gw_sample_tab[i].caller != 0u
                ? gw_sample_tab[i].caller - (uint32_t)(uintptr_t)GetModuleHandleA(NULL)
                : 0u);
  }
  fclose(f);
  MoveFileExA("melee-pc.samples.tmp", "melee-pc.samples", MOVEFILE_REPLACE_EXISTING);
}

static DWORD WINAPI gw_sample_thread(LPVOID arg) {
  const DWORD start_ms = (DWORD)(uintptr_t)arg;
  DWORD last_dump;
  timeBeginPeriod(1);
  {
    const uint8_t *base = (const uint8_t *)GetModuleHandleA(NULL);
    const IMAGE_NT_HEADERS *nt =
        (const IMAGE_NT_HEADERS *)(base + ((const IMAGE_DOS_HEADER *)base)->e_lfanew);
    const IMAGE_SECTION_HEADER *sec = IMAGE_FIRST_SECTION(nt);
    unsigned i;
    for (i = 0; i < nt->FileHeader.NumberOfSections; ++i, ++sec) {
      if (sec->Characteristics & IMAGE_SCN_CNT_CODE) {
        gw_sample_text_lo = (uint32_t)(uintptr_t)(base + sec->VirtualAddress);
        gw_sample_text_hi = gw_sample_text_lo + sec->Misc.VirtualSize;
        break;
      }
    }
  }
  Sleep(start_ms);
  gw_sample_enum_threads();
  last_dump = GetTickCount();
  gw_log("gw: PROF sampler started (%d threads)", gw_sample_nthr);
  for (;;) {
    int t;
    Sleep(1);
    for (t = 0; t < gw_sample_nthr; ++t) {
      CONTEXT ctx;
      if (SuspendThread(gw_sample_thr[t].h) == (DWORD)-1) {
        continue;
      }
      ctx.ContextFlags = CONTEXT_CONTROL;
      if (GetThreadContext(gw_sample_thr[t].h, &ctx)) {
        uint32_t eip = (uint32_t)ctx.Eip;
        uint32_t caller = 0u;
        uint32_t h;
        if (eip < gw_sample_text_lo || eip >= gw_sample_text_hi) {
          /* Outside our code (a wait in the OS, a driver, the CRT): the nearest return address
           * into melee-pc.exe on the stack says who is waiting. A heuristic stack scan - no frame
           * pointers to walk - but the first hit is almost always the true caller. */
          MEMORY_BASIC_INFORMATION mbi;
          const uint32_t esp = (uint32_t)ctx.Esp;
          if (VirtualQuery((LPCVOID)(uintptr_t)esp, &mbi, sizeof mbi) != 0) {
            const uint32_t top = (uint32_t)(uintptr_t)mbi.BaseAddress + (uint32_t)mbi.RegionSize;
            uint32_t p;
            for (p = esp; p + 4u <= top && p < esp + 8192u; p += 4u) {
              const uint32_t v = *(const uint32_t *)(uintptr_t)p;
              if (v >= gw_sample_text_lo && v < gw_sample_text_hi) {
                caller = v;
                break;
              }
            }
          }
        }
        if (t == 0 && gw_spike_threshold_ms > 0.0) {
          const uint32_t r = gw_spike_ring_head & (GW_SPIKE_RING - 1u);
          gw_spike_ring[r].qpc = gw_prof_now();
          gw_spike_ring[r].eip = eip;
          gw_spike_ring[r].caller = caller;
          {
            MEMORY_BASIC_INFORMATION cm;
            const uint32_t sp0 = (uint32_t)ctx.Esp;
            int nc = 0, k;
            for (k = 0; k < GW_CHAIN; ++k) {
              gw_spike_ring[r].chain[k] = 0u;
            }
            if (VirtualQuery((LPCVOID)(uintptr_t)sp0, &cm, sizeof cm) != 0) {
              const uint32_t ctop = (uint32_t)(uintptr_t)cm.BaseAddress + (uint32_t)cm.RegionSize;
              uint32_t q;
              for (q = sp0; q + 4u <= ctop && q < sp0 + 16384u && nc < GW_CHAIN; q += 4u) {
                const uint32_t v = *(const uint32_t *)(uintptr_t)q;
                if (v >= gw_sample_text_lo + 6u && v < gw_sample_text_hi) {
                  const uint8_t *b = (const uint8_t *)(uintptr_t)v;
                  if (b[-5] == 0xE8u || b[-6] == 0xFFu || b[-2] == 0xFFu || b[-3] == 0xFFu) {
                    gw_spike_ring[r].chain[nc++] = v - gw_sample_text_lo;
                  }
                }
              }
            }
          }
          ++gw_spike_ring_head;
        }
        h = ((eip ^ caller ^ ((uint32_t)t << 27)) * 2654435761u) >> 15;
        while (gw_sample_tab[h].count != 0u &&
               (gw_sample_tab[h].eip != eip || gw_sample_tab[h].tidx != (uint32_t)t ||
                gw_sample_tab[h].caller != caller)) {
          h = (h + 1u) & (GW_SAMPLE_SLOTS - 1u);
        }
        gw_sample_tab[h].eip = eip;
        gw_sample_tab[h].caller = caller;
        gw_sample_tab[h].tidx = (uint32_t)t;
        ++gw_sample_tab[h].count;
        ++gw_sample_total[t];
      }
      ResumeThread(gw_sample_thr[t].h);
    }
    if (GetTickCount() - last_dump >= 5000u) {
      last_dump = GetTickCount();
      gw_sample_dump();
      gw_sample_enum_threads();
    }
  }
}

static void gw_sample_maybe_start(void) {
  static int done;
  const char *v;
  if (done) {
    return;
  }
  done = 1;
  v = getenv("MELEE_PROFILE_SAMPLE");
  if (v == NULL || v[0] == '\0') {
    return;
  }
  gw_sample_main_tid = GetCurrentThreadId();
  {
    const char *sp = getenv("MELEE_PROFILE_SPIKE");
    gw_spike_threshold_ms = (sp != NULL && sp[0] != '\0') ? atof(sp) : 0.0;
  }
  /* the frame thread first, so it is thread 0 */
  gw_sample_thr[0].tid = gw_sample_main_tid;
  gw_sample_thr[0].h = OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT, FALSE,
                                  gw_sample_main_tid);
  gw_sample_nthr = gw_sample_thr[0].h != NULL ? 1 : 0;
  CreateThread(NULL, 0, gw_sample_thread, (LPVOID)(uintptr_t)(atoi(v) * 1000), 0, NULL);
}

/* Copy the ring samples that fall in (t0, t1] into the spike table. */
static void gw_spike_collect(long long t0, long long t1) {
  const uint32_t head = gw_spike_ring_head;
  uint32_t n = head < GW_SPIKE_RING ? head : GW_SPIKE_RING;
  uint32_t i;
  ++gw_spike_frames;
  for (i = 0; i < n; ++i) {
    const uint32_t r = (head - 1u - i) & (GW_SPIKE_RING - 1u);
    uint32_t h;
    if (gw_spike_ring[r].qpc <= t0) {
      break;
    }
    if (gw_spike_ring[r].qpc > t1) {
      continue;
    }
    {
      uint32_t ch = 2166136261u, kk;
      for (kk = 0; kk < 5; ++kk) {
        ch = (ch ^ gw_spike_ring[r].chain[kk]) * 16777619u;
      }
      ch = (ch >> 18) & (GW_SPIKE_SLOTS - 1u);
      while (gw_chain_tab[ch].count != 0u &&
             memcmp(gw_chain_tab[ch].chain, gw_spike_ring[r].chain, 5 * sizeof(uint32_t)) != 0) {
        ch = (ch + 1u) & (GW_SPIKE_SLOTS - 1u);
      }
      memcpy(gw_chain_tab[ch].chain, gw_spike_ring[r].chain, 5 * sizeof(uint32_t));
      ++gw_chain_tab[ch].count;
    }
    h = ((gw_spike_ring[r].eip ^ gw_spike_ring[r].caller) * 2654435761u) >> 18;
    while (gw_spike_tab[h].count != 0u && (gw_spike_tab[h].eip != gw_spike_ring[r].eip ||
                                            gw_spike_tab[h].caller != gw_spike_ring[r].caller)) {
      h = (h + 1u) & (GW_SPIKE_SLOTS - 1u);
    }
    gw_spike_tab[h].eip = gw_spike_ring[r].eip;
    gw_spike_tab[h].caller = gw_spike_ring[r].caller;
    ++gw_spike_tab[h].count;
    ++gw_spike_total;
  }
}

static void gw_spike_dump(void) {
  FILE *f = fopen("melee-pc.spike.samples.tmp", "w");
  uint32_t i;
  if (f == NULL) {
    return;
  }
  fprintf(f, "# thread 0 tid 0 total %u\n", gw_spike_total);
  for (i = 0; i < GW_SPIKE_SLOTS; ++i) {
    HMODULE mod = NULL;
    char path[MAX_PATH], *base;
    if (gw_spike_tab[i].count == 0u) {
      continue;
    }
    path[0] = '\0';
    if (GetModuleHandleExA(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
                               GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                           (LPCSTR)(uintptr_t)gw_spike_tab[i].eip, &mod) &&
        GetModuleFileNameA(mod, path, sizeof path) != 0) {
      base = strrchr(path, '\\');
      base = (base != NULL) ? base + 1 : path;
    } else {
      base = "?";
      mod = NULL;
    }
    fprintf(f, "0 %u %s %08X %08X\n", gw_spike_tab[i].count, base,
            gw_spike_tab[i].eip - (uint32_t)(uintptr_t)mod,
            gw_spike_tab[i].caller != 0u
                ? gw_spike_tab[i].caller - (uint32_t)(uintptr_t)GetModuleHandleA(NULL)
                : 0u);
  }
  fclose(f);
  MoveFileExA("melee-pc.spike.samples.tmp", "melee-pc.spike.samples", MOVEFILE_REPLACE_EXISTING);
  {
    FILE *cf = fopen("melee-pc.spike.chains", "w");
    if (cf != NULL) {
      for (i = 0; i < GW_SPIKE_SLOTS; ++i) {
        if (gw_chain_tab[i].count != 0u) {
          fprintf(cf, "%u %X %X %X %X %X\n", gw_chain_tab[i].count, gw_chain_tab[i].chain[0],
                  gw_chain_tab[i].chain[1], gw_chain_tab[i].chain[2], gw_chain_tab[i].chain[3],
                  gw_chain_tab[i].chain[4]);
        }
      }
      fclose(cf);
    }
  }
  gw_log("gw: PROF spike profile: %u spike frames, %u samples -> melee-pc.spike.samples",
         gw_spike_frames, gw_spike_total);
}

/* MELEE_PROFILE_FRAMES=<file>: one CSV row per presented frame - the split, texture inits, GX
 * counters and aurora's pipeline counters - so a spike can be lined up with what the frame did. */
static FILE *gw_prof_csv;
static int gw_prof_csv_tried;

void gw_frame_tick(void) {
  const int prof = gw_prof_on();

  gw_sample_maybe_start();
  long long t_enter = 0, t_present = 0, t_events = 0, t_begin = 0;
  int presented = 0;

  gw_watch_tick();

  if (prof) {
    t_enter = gw_prof_now();
  }

  if (gw_frame_has_content && gw_frame_begun) {
    /* Composited over the game's output by Aurora's ImGui pass, which is why this must happen
     * before end_frame: aurora::end_frame() is what freezes the ImGui draw data. */
    gw_Overlay_Draw();
    gw_Overlay_DrawPanel();
    gw_stats_draw();
    gw_stats_note_present(0);
    gw_shots_tick(gw_presented_count + 1u);
    if (gw_video_fps > 60) {
      gw_uncap_next = gw_time_ticks() + GW_TIMER_CLOCK / (uint64_t)gw_video_fps;
    }
    if (prof) {
      gw_prof_t_pace0 = gw_prof_now();
    }
    if (gw_pace_legacy()) {
      gw_pace_field();
    }
    if (prof) {
      gw_prof_t_pace1 = gw_prof_now();
    }
    if (gw_inprof_on()) {
      gw_inprof_submit();
    }
    aurora_end_frame(); /* enqueues to the render worker; the real Present() is async */
    ++gw_end_frames;
    if (gw_inprof_on()) {
      gw_inprof_submitted();
    }
    gw_frame_begun = false;
    gw_frame_has_content = false;
    ++gw_presented_count;
    presented = 1;
    if (!gw_pace_legacy()) {
      /* submitted as soon as it was drawn; now wait for the next pad sample (gw_pace_field) */
      long long tp = prof ? gw_prof_now() : 0;
      gw_pace_field();
      if (prof) {
        gw_prof_sum_pace_after += gw_prof_ms(tp, gw_prof_now());
      }
    }
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
  if (gw_inprof_on()) {
    gw_ip_last_events = gw_prof_now();
  }

  if (!gw_frame_begun) {
    gw_frame_begun = aurora_begin_frame();
  }
  if (gw_inprof_on() && presented) {
    gw_ip_push(&gw_ip_begin, gw_prof_ms(gw_ip_last_events, gw_prof_now()));
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
        gw_prof_sum_pace += gw_prof_ms(gw_prof_t_pace0, gw_prof_t_pace1);
        gw_prof_sum_submit += gw_prof_ms(gw_prof_t_pace1, t_present);
        gw_prof_sum_events += gw_prof_ms(t_present, t_events);
        gw_prof_sum_begin += gw_prof_ms(t_events, t_begin);
        ++gw_prof_frames;
        gw_prof_record(total);
        if (!gw_prof_csv_tried) {
          const char *cv = getenv("MELEE_PROFILE_FRAMES");
          gw_prof_csv_tried = 1;
          if (cv != NULL && cv[0] != '\0') {
            gw_prof_csv = fopen(cv, "w");
            if (gw_prof_csv != NULL) {
              fprintf(gw_prof_csv,
                      "frame,total_ms,game_ms,present_ms,texobj_inits,prims,dlists,queued_pipes,"
                      "created_pipes,urgent_pipes,drawcalls,vert_kb,storage_kb,texupload_kb,pad_calls,pad_aurora_ms,"
                      "pad_adapter_ms,pad_rest_ms,wait_idle_calls,pipe_wait_hits,pipe_wait_ms\n");
            }
          }
        }
        if (gw_prof_csv != NULL) {
          extern uint32_t gw_gx_texobj_inits;
          static uint32_t last_inits;
          uint32_t copies, prims, dlists;
          const AuroraStats *as = aurora_get_stats();
          gw_gx_get_stats(&copies, &prims, &dlists);
          extern double gw_pad_prof_aurora_ms, gw_pad_prof_adapter_ms, gw_pad_prof_rest_ms;
          extern uint32_t gw_pad_prof_calls;
          static uint32_t last_waits;
          fprintf(gw_prof_csv, "%u,%.3f,%.3f,%.3f,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%.3f,%.3f,%.3f,%u,%u,%.3f\n",
                  gw_presented_count, total, game, gw_prof_ms(t_enter, t_present),
                  gw_gx_texobj_inits - last_inits, prims, dlists,
                  as != NULL ? as->queuedPipelines : 0u, as != NULL ? as->createdPipelines : 0u,
                  as != NULL ? as->urgentPipelinesPending : 0u,
                  as != NULL ? as->drawCallCount : 0u, as != NULL ? as->lastVertSize / 1024u : 0u,
                  as != NULL ? as->lastStorageSize / 1024u : 0u,
                  as != NULL ? as->lastTextureUploadSize / 1024u : 0u, gw_pad_prof_calls,
                  gw_pad_prof_aurora_ms, gw_pad_prof_adapter_ms, gw_pad_prof_rest_ms,
                  gw_wait_idle_count - last_waits,
                  /* cumulative: draws that blocked on an unbuilt pipeline (a warm-up gap) */
                  as != NULL ? as->pipelineWaitHits : 0u,
                  as != NULL ? as->pipelineWaitUs / 1000.0 : 0.0);
          last_waits = gw_wait_idle_count;
          gw_pad_prof_aurora_ms = gw_pad_prof_adapter_ms = gw_pad_prof_rest_ms = 0.0;
          gw_pad_prof_calls = 0u;
          last_inits = gw_gx_texobj_inits;
          {
            static uint32_t rows;
            if ((++rows % 60u) == 0u) {
              fflush(gw_prof_csv);
            }
          }
        }
        if (gw_spike_threshold_ms > 0.0 && game > gw_spike_threshold_ms) {
          gw_spike_collect(gw_prof_last_end, t_enter);
        }
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
          if (gw_spike_threshold_ms > 0.0) {
            gw_spike_dump();
          }
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

  if (presented && gw_inprof_on()) {
    gw_inprof_return();
  }
}

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
  /* A queued completion is already DUE: the DVD/ARQ shims do the transfer inline and only defer
   * the callback out of the starter's stack (gw_defer). The game's synchronous waits are
   * `while (state != DONE) wait_idle();` (lbArq_80014BD0 above all - every fighter action-state
   * change loads its animation from ARAM this way), and nothing but this call can set DONE, so
   * gating it on the clock made each such load wait for GetTickCount64 to change. That is a
   * millisecond at best and a whole 15.6 ms system tick when the timer resolution is coarse: in a
   * real 4-player fight, several motion changes a frame stacked into 20-90 ms game-thread spins
   * (~10 million wait_idle calls) and a p99 of 45 ms. Run the queue at once; the clock gate stays
   * for alarms, which are genuinely time-driven. */
  if (gw_deferred_count != 0) {
    gw_run_deferred();
  }
  /* Alarms are checked on the high-resolution clock, at most every quarter millisecond.
   * GetTickCount64 only moves once per system tick (~15.6 ms, whatever timeBeginPeriod says), and
   * gating on it let the pad alarm fire up to a whole tick late whenever the game was waiting in
   * its pad-queue spin (MELEE_INPUT_PROFILE, loading into a match: alarm lateness p95 11.9 ms,
   * max 24 ms). */
  (void)now;
  {
    const uint64_t t = gw_time_ticks();
    if (t - gw_last_alarm_tick < GW_TIMER_CLOCK / 4000u) {
      return;
    }
    gw_last_alarm_tick = t;
    gw_os_run_alarms(t);
  }
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
