/* Entry point for the Melee PC port.
 *
 * Aurora owns the real main() (aurora/main.h redefines main to aurora_main), so this runs once
 * Aurora's application layer is up. It brings the game world online in the order melee's own boot
 * code expects, then calls the game's main(), which never returns: from there the port is driven
 * entirely from inside the SDK shims, above all the VI retrace shim.
 */
#include "gw.h"
#include "shim_gx.h"
#include "shim_vi.h"
#include "gw_profiler.h"
#include "gw_hang.h"
#include "gc_adapter_policy.h"

#include <aurora/aurora.h>
#include <dolphin/gx/GXAurora.h>
#include <aurora/event.h>
#include <aurora/gfx.h>
#include <aurora/main.h>

#include <SDL3/SDL_hints.h>

#ifndef _WIN32
#include <SDL3/SDL_video.h>
#endif

#ifdef _WIN32
#include <windows.h>
#else
#include "gw_compat_linux.h"
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char gw_iso_path_buf[1024];

/* Delayed worker/GPU measurements stay native and cannot affect snapshot state.
 * detail identifies the pass; CPU worker samples overlap game-thread zones. */
static void gw_aurora_profiler_sink(const char *name, uint64_t frame, uint64_t ns, void *data) {
  unsigned id, detail = 2166136261u;
  const unsigned char *p = (const unsigned char *)name;
  (void)data;
  if (!gw_prof_active()) return;
  if (!gw_prof_enabled()) { /* summary only: the worker's busy time and pipeline compiles, nothing else */
    if (!strcmp(name, "cpu.render_worker")) gw_prof_cpu_completed(GW_PROF_RENDER, 0, (double)ns * 0.000001);
    else if (!strncmp(name, "cpu.pipeline_compile.", 21) || !strcmp(name, "cpu.pipeline_wait"))
      gw_prof_cpu_completed(GW_PROF_PIPELINE_COMPILE, 0, (double)ns * 0.000001);
    return;
  }
  if (!strcmp(name, "cpu.pipeline_skip")) { gw_prof_counter(GW_PROF_PIPELINE_SKIPS, 1); return; }
  if (!strcmp(name, "cpu.pipeline_wait")) { gw_prof_counter(GW_PROF_PIPELINE_WAITS, 1); }
  for (; *p; ++p) detail = (detail ^ *p) * 16777619u;
  if (strcmp(name, "cpu.render_worker") == 0) id = GW_PROF_RENDER;
  else if (strcmp(name, "cpu.present") == 0) id = GW_PROF_PRESENT;
  else if (strncmp(name, "cpu.pipeline_compile.", 21) == 0 || strcmp(name, "cpu.pipeline_wait") == 0) id = GW_PROF_PIPELINE_COMPILE;
  else id = strcmp(name, "gpu.frame") == 0 ? GW_PROF_GPU : GW_PROF_GPU_PASS;
  gw_prof_detail_name(detail, name);
  if (strncmp(name, "cpu.", 4) == 0) gw_prof_cpu_completed(id, detail, (double)ns * 0.000001);
  else gw_prof_gpu_completed(id, detail, (double)ns * 0.000001, frame);
}

/* Window placement, for running the port without it landing on the user's main display - a
 * second monitor, off-screen entirely, or hidden. Aurora only honours non-negative positions
 * (window.cpp treats a negative x or y as "undefined" and centres the window), so a position on
 * a monitor left of or above the primary has to be applied after the window exists; everything
 * else goes through AuroraConfig and never flashes on the wrong display.
 *
 *   MELEE_WINDOW_X / MELEE_WINDOW_Y  window position, virtual-desktop coordinates (may be
 *                                    negative; large values such as 30000 park it off-screen)
 *   MELEE_WINDOW_W / MELEE_WINDOW_H  window size (Aurora clamps to at least 640x480)
 *   MELEE_WINDOW_HIDE=1              hide the window entirely; the game still runs and renders
 */
static bool gw_env_int(const char *name, int *out) {
  const char *v = getenv(name);
  if (v == NULL || *v == 0) {
    return false;
  }
  *out = atoi(v);
  return true;
}

/* Apply whatever AuroraConfig could not: a negative position, and hiding. Runs immediately after
 * aurora_initialize, before the first frame is drawn. */
static void gw_apply_window_env(void) {
  int x, y, hide = 0;
  bool have_x = gw_env_int("MELEE_WINDOW_X", &x);
  bool have_y = gw_env_int("MELEE_WINDOW_Y", &y);

  (void)gw_env_int("MELEE_WINDOW_HIDE", &hide);
  if (!hide && !((have_x && x < 0) || (have_y && y < 0))) {
    return; /* the config path already placed it */
  }
#ifdef _WIN32
  HWND hwnd = FindWindowA(NULL, "Melee PC");
  if (hwnd == NULL) {
    gw_log("melee-pc: window placement: could not find the window");
    return;
  }
  if ((have_x && x < 0) || (have_y && y < 0)) {
    SetWindowPos(hwnd, NULL, have_x ? x : 0, have_y ? y : 0, 0, 0,
                 SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE);
    gw_log("melee-pc: window moved to %d,%d", have_x ? x : 0, have_y ? y : 0);
  }
  if (hide) {
    ShowWindow(hwnd, SW_HIDE);
    gw_log("melee-pc: window hidden (MELEE_WINDOW_HIDE=1)");
  }
#else
  SDL_Window *win = (SDL_Window *)gw_get_window();
  if (win == NULL) {
    gw_log("melee-pc: window placement: no window");
    return;
  }
  if ((have_x && x < 0) || (have_y && y < 0)) {
    SDL_SetWindowPosition(win, have_x ? x : 0, have_y ? y : 0);
    gw_log("melee-pc: window moved to %d,%d", have_x ? x : 0, have_y ? y : 0);
  }
  if (hide) {
    SDL_HideWindow(win);
    gw_log("melee-pc: window hidden (MELEE_WINDOW_HIDE=1)");
  }
#endif
}

const char *gw_iso_path(void) { return gw_iso_path_buf[0] != '\0' ? gw_iso_path_buf : NULL; }

static void gw_aurora_log(AuroraLogLevel level, const char *module, const char *message,
                          unsigned int len) {
  (void)len;
  static const char *const names[] = {"debug", "info", "warning", "error", "fatal"};
  static int verbose = -1;
  if (verbose < 0) {
    const char *v = getenv("MELEE_AURORA_VERBOSE");
    verbose = (v != NULL && v[0] == '1') ? 1 : 0;
  }
  if (level >= LOG_WARNING || (verbose && level >= LOG_INFO)) {
    gw_log("aurora[%s] %s: %s", names[level], module, message);
  }
  if (level == LOG_FATAL) {
    gw_panic("aurora fatal: %s", message);
  }
}

/* The disc image supplies every asset the game loads, so it has to be found before boot. */
static bool gw_find_iso(int argc, char **argv) {
  for (int i = 1; i < argc; ++i) {
    if (strcmp(argv[i], "--iso") == 0 && i + 1 < argc) {
      snprintf(gw_iso_path_buf, sizeof gw_iso_path_buf, "%s", argv[i + 1]);
      return true;
    }
    if (strstr(argv[i], ".iso") != NULL || strstr(argv[i], ".gcm") != NULL) {
      snprintf(gw_iso_path_buf, sizeof gw_iso_path_buf, "%s", argv[i]);
      return true;
    }
  }
  const char *env = getenv("MELEE_ISO");
  if (env != NULL && env[0] != '\0') {
    snprintf(gw_iso_path_buf, sizeof gw_iso_path_buf, "%s", env);
    return true;
  }
  static const char *const candidates[] = {
      "melee.iso",
      "Super Smash Bros. Melee (USA) (En,Ja) (v1.02).iso",
      "C:\\iso\\Super Smash Bros. Melee (USA) (En,Ja) (v1.02).iso",
  };
  for (size_t i = 0; i < sizeof candidates / sizeof candidates[0]; ++i) {
    FILE *f = fopen(candidates[i], "rb");
    if (f != NULL) {
      fclose(f);
      snprintf(gw_iso_path_buf, sizeof gw_iso_path_buf, "%s", candidates[i]);
      return true;
    }
  }
  return false;
}

/* Backend override so D3D11 and D3D12 can be compared without a rebuild:
 *   MELEE_BACKEND=d3d12   (or d3d11, auto, vulkan)
 * Defaults to D3D11 for the reason documented at .desiredBackend below. */
#ifdef _WIN32
static AuroraBackend gw_desired_backend(void) {
  const char *env = getenv("MELEE_BACKEND");
  if (env == NULL) {
    return BACKEND_D3D11;
  }
  if (_stricmp(env, "d3d12") == 0) {
    gw_log("melee-pc: MELEE_BACKEND=d3d12");
    return BACKEND_D3D12;
  }
  if (_stricmp(env, "auto") == 0) {
    gw_log("melee-pc: MELEE_BACKEND=auto");
    return BACKEND_AUTO;
  }
  if (_stricmp(env, "vulkan") == 0) {
    gw_log("melee-pc: MELEE_BACKEND=vulkan");
    return BACKEND_VULKAN;
  }
  return BACKEND_D3D11;
}
#else
/* No D3D11/D3D12 on Linux - Vulkan is the only backend, so MELEE_BACKEND only chooses between
 * an explicit request and Aurora's own auto-detection. */
static AuroraBackend gw_desired_backend(void) {
  const char *env = getenv("MELEE_BACKEND");
  if (env != NULL && _stricmp(env, "auto") == 0) {
    gw_log("melee-pc: MELEE_BACKEND=auto");
    return BACKEND_AUTO;
  }
  if (env != NULL && _stricmp(env, "d3d11") != 0 && _stricmp(env, "d3d12") != 0 &&
      _stricmp(env, "vulkan") != 0) {
    gw_log("melee-pc: MELEE_BACKEND=%s not recognized, using vulkan", env);
  } else if (env != NULL && _stricmp(env, "vulkan") != 0) {
    gw_log("melee-pc: MELEE_BACKEND=%s not available on Linux, using vulkan", env);
  }
  return BACKEND_VULKAN;
}
#endif

/* Dawn's compiled-pipeline cache, kept OUT of the shared per-user prefs directory.
 *
 * aurora defaults cachePath to SDL_GetPrefPath("Melee PC"), so EVERY melee-pc process on the
 * machine writes one SQLite database - whatever sandbox, build or agent it belongs to. Two runs
 * at once corrupt it, and from then on every later run reads the poisoned entry and dies at its
 * first draw with "aurora::gfx::gx: unmapped vtx attr 13", long before any game code is
 * involved. It cost hours: a whole sweep silently produced frame-0 failures, a binary that had
 * rendered fifteen minutes earlier stopped rendering, and it looked exactly like a GPU or driver
 * fault - I recommended a reboot, which would not have helped.
 *
 * run.sh and selftest.ps1 already isolate the exe, the log, the memory card and the mods dir per
 * sandbox; this was the one shared mutable file left. Default it next to the executable, which is
 * per-sandbox by construction, and let MELEE_CACHE_DIR override.
 */
static const char *gw_cache_path(void) {
  static char buf[MAX_PATH];
  const char *env = getenv("MELEE_CACHE_DIR");
  DWORD n;
  char *slash;
  if (env != NULL && env[0] != '\0') {
    return env;
  }
  n = GetModuleFileNameA(NULL, buf, (DWORD)sizeof buf);
  if (n == 0 || n >= sizeof buf) {
    return NULL; /* let aurora fall back to its default */
  }
#ifdef _WIN32
  slash = gw_path_separator(buf);
#else
  slash = strrchr(buf, '/');
#endif
  if (slash == NULL) {
    return NULL;
  }
  slash[1] = '\0';
  return buf;
}

#include "gw_pcache_guard.inc"

/* This executable's folder, with a trailing separator - where the pipeline seed sits. */
static const char *gw_exe_dir(void) {
  static char buf[MAX_PATH];
  DWORD n = GetModuleFileNameA(NULL, buf, (DWORD)sizeof buf);
  char *slash;
#ifdef _WIN32
  slash = (n == 0 || n >= sizeof buf) ? NULL : gw_path_separator(buf);
#else
  slash = (n == 0 || n >= sizeof buf) ? NULL : strrchr(buf, '/');
#endif
  if (slash == NULL) {
    return NULL;
  }
  slash[1] = '\0';
  return buf;
}

int main(int argc, char *argv[]) {
  gw_install_crash_handler();
  gw_log("melee-pc: starting");
  gw_hang_start();

  /* Only the raw adapter backend implements foreground ownership. SDL's GameCube
   * driver must not claim it behind that policy, even with MELEE_SDL_GAMECUBE=1. */
  _putenv("SDL_JOYSTICK_HIDAPI_GAMECUBE=0");

  if (!gw_find_iso(argc, argv)) {
    gw_log("melee-pc: no disc image found. Pass --iso <path to a GALE01 v1.02 image>, set"
           " MELEE_ISO, or put melee.iso next to the executable.");
    gw_hang_final("no disc image", 1);
    return 1;
  }
  gw_log("melee-pc: disc image <disc>");
  gw_turbo_configure(argc, argv);

  /* Scan the Target Test mod directory now so the loader reports what it found at boot, instead of
   * only on the first Target Test query. */
  {
    extern int gw_TTMod_Count(void);
    (void)gw_TTMod_Count();
  }

  /* --test runs the in-engine suite and exits: no window, no GPU, no frame driver. It still runs
   * the fixups and MEM1 reservation above the game's own main(), because those are what make guest
   * memory valid for the retargeted objects the tests link against. See
   * _research/engine-test-suite-plan.md. */
  {
    extern bool gw_test_requested(int argc, char **argv);
    extern int gw_test_run_all(void);
    if (gw_test_requested(argc, argv)) {
      gw_hang_pause(4); /* headless suite intentionally has no logic/presentation driver */
      gw_apply_fixups();
      if (!gw_mem_init()) {
        gw_log("melee-pc: tests: could not reserve MEM1/ARAM");
        gw_hang_final("tests MEM1/ARAM reservation failed", 1);
        return 1;
      }
      {
        int failures = gw_test_run_all();
        gw_log("melee-pc: tests complete, failures=%d", failures);
        gw_hang_final("tests complete", failures != 0);
        return failures != 0;
      }
    }
  }

  /* MELEE_VSYNC=0 presents without waiting for the display's refresh (Aurora picks Mailbox, else
   * Immediate): the lowest present latency, at the cost of tearing with Immediate. The game still
   * runs at exactly 60 Hz either way (gw_pace_field); vsync only decides when a finished frame
   * reaches the screen. Default on. */
  int vsync = 1;
  (void)gw_env_int("MELEE_VSYNC", &vsync);
  if (gw_turbo_enabled()) vsync = 0;
  int msaa = 1;
  (void)gw_env_int("MELEE_MSAA", &msaa);
  if (msaa != 1 && msaa != 4) {
    gw_log("gw: video: MELEE_MSAA must be 1 or 4; using 1");
    msaa = 1;
  }
  int win_x = 0, win_y = 0, win_w = 1280, win_h = 960;
  bool have_x = gw_env_int("MELEE_WINDOW_X", &win_x);
  bool have_y = gw_env_int("MELEE_WINDOW_Y", &win_y);
  (void)gw_env_int("MELEE_WINDOW_W", &win_w);
  (void)gw_env_int("MELEE_WINDOW_H", &win_h);

  const AuroraConfig config = {
      .appName = "Melee PC",
      .cachePath = gw_cache_path(),
      /* The exe's own folder: aurora loads its pipeline seed, initial_pipeline_cache.db, from
       * here and warms every pipeline in it in the background. The seed is built from a full
       * sweep (tools/port/build_pipeline_seed.py), so the loading screen - which holds until the
       * warm-up queue is empty - releases into a match whose pipelines already exist. */
      .resourcesPath = gw_exe_dir(),
      /* Dawn's D3D12 backend (v20260807.225922, 32-bit x86) crashes a few seconds into
       * first-frame rendering: wgpuSurfaceGetCurrentTexture -> d3d12::Queue::WaitForSerial
       * dereferences a queue-serial value as a pointer (near-NULL read, webgpu_dawn.dll
       * +0x363548). D3D11's queue/serial path does not, so pin it until Dawn is fixed. */
      .desiredBackend = gw_desired_backend(),
      .vsync = vsync != 0,
      .msaa = (uint32_t)msaa,
      /* Aurora reads a negative x or y as "undefined"; those are applied after init instead
       * (gw_apply_window_env), so pass -1 to keep its centring default in that case. */
      .windowPosX = (have_x && win_x >= 0) ? win_x : -1,
      .windowPosY = (have_y && win_y >= 0) ? win_y : -1,
      .windowWidth = (uint32_t)win_w,
      .windowHeight = (uint32_t)win_h,
      .logCallback = &gw_aurora_log,
      .logLevel = LOG_INFO,
      /* The port manages MEM1 and ARAM itself, because game code tells main memory from ARAM by
       * comparing addresses against 0x80000000. */
      .mem1Size = 0,
      .mem2Size = 0,
  };
  /* Boot black screen: Aurora opens SDL's joystick subsystem on the first aurora_update, before the first
   * present, and SDL's DirectInput enumeration asks every HID device for its product string. A slow
   * device kept the main thread in HidD_GetProductString for 4.2 s in 2 of 7 boots (cdb stack: dinput8
   * CDIObj_EnumDevicesW), window black. XInput, HIDAPI and RawInput still find ordinary pads, and the GC
   * adapter has its own path (gc_adapter.c). MELEE_DIRECTINPUT=1 brings DirectInput back for an old pad
   * that only it can see. */
  {
    int dinput = 0;
    (void)gw_env_int("MELEE_DIRECTINPUT", &dinput);
    SDL_SetHint(SDL_HINT_JOYSTICK_DIRECTINPUT, dinput ? "1" : "0");
    gw_log("melee-pc: DirectInput joysticks %s (MELEE_DIRECTINPUT)", dinput ? "on" : "off");
  }
  gw_prof_init();
  aurora_profiler_set_sink(gw_aurora_profiler_sink, NULL);
  aurora_profiler_enable(gw_prof_active() != 0);
  /* Aurora replays every persisted pipeline at boot and aborts on one it cannot build: drop such rows first. */
  (void)gw_pcache_guard_run(config.cachePath);
  AuroraInfo info = aurora_initialize(argc, argv, &config);
  gw_log("prof: GPU timestamps %s (D3D11 has no Dawn timestamp feature; opt into D3D12 for supported adapters)",
         aurora_profiler_gpu_available() ? "available" : "unavailable");

#ifdef _WIN32
  if (!gw_window_drag_install(info.window)) {
    gw_panic("could not install nonmodal window drag");
  }
#endif
  gw_log("melee-pc: aurora backend %d, window %ux%u", (int)info.backend, info.windowSize.width,
         info.windowSize.height);
#ifndef _WIN32
  gw_window_drag_install(info.window);
  gw_set_window(info.window);
#endif
  gw_apply_window_env();

  /* Letterbox rather than stretch when the window is not 4:3.
   *
   * Aurora's two halves disagree out of the box: the GX side defaults its policy to
   * AURORA_VIEWPORT_FIT (gx.hpp), but the window side defaults g_frameBufferAspectFit to false
   * (window.cpp), and the window only learns the policy when AuroraSetViewportPolicy is called.
   * With nobody calling it, GX believes it is fitting while the window stretches. Setting it
   * explicitly syncs the two and makes resizing preserve the game's 4:3 aspect. */
  AuroraSetViewportPolicy(AURORA_VIEWPORT_FIT);

  /* Order matters: the fixups rewrite pointers inside game globals, so nothing may read a game
   * global before this runs. */
  gw_apply_fixups();
  if (!gw_mem_init()) {
    gw_panic("could not reserve MEM1/ARAM");
  }
  gw_gx_init_render_modes();
  if (!gw_frame_init()) {
    gw_panic("could not start the frame driver");
  }
  /* From here there is a renderer whose pipelines compile lazily, so the game's loading screen
   * has something to wait for. A --test run has already returned above and never reaches this. */
  {
    extern void gw_Gfx_SetLive(int live);
    gw_Gfx_SetLive(1);
  }

  gw_log("melee-pc: entering game main()");
  gw_start_watchdog();
  gw_main();

  gw_log("melee-pc: game main() returned");
  gw_dump_stub_summary();
  {
    extern void gw_exit_clean(int code); /* shim_vi.c: guarded teardown, then TerminateProcess */
    gw_exit_clean(0);
  }
  return 0;
}
