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

#include <aurora/aurora.h>
#include <dolphin/gx/GXAurora.h>
#include <aurora/event.h>
#include <aurora/main.h>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char gw_iso_path_buf[1024];

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

int main(int argc, char *argv[]) {
  gw_install_crash_handler();
  gw_log("melee-pc: starting");

  /* Keep SDL's HIDAPI GameCube driver off the adapter. The port reads the WUP-028 directly over
   * WinUSB (pc/platform/gc_adapter.c) to get the console's own analog ranges and button layout;
   * if SDL opens the device first, that open fails and the pad falls back to SDL's remapped,
   * deadzoned view of the same hardware. SDL reads its hints from the environment, so setting
   * this before Aurora starts is enough -- no SDL linkage needed here. Set
   * MELEE_SDL_GAMECUBE=1 to hand the adapter back to SDL instead. */
  {
    const char *prefer_sdl = getenv("MELEE_SDL_GAMECUBE");
    if (prefer_sdl == NULL || prefer_sdl[0] != '1') {
      _putenv("SDL_JOYSTICK_HIDAPI_GAMECUBE=0");
    }
  }

  if (!gw_find_iso(argc, argv)) {
    gw_log("melee-pc: no disc image found. Pass --iso <path to a GALE01 v1.02 image>, set"
           " MELEE_ISO, or put melee.iso next to the executable.");
    return 1;
  }
  gw_log("melee-pc: disc image %s", gw_iso_path_buf);

  const AuroraConfig config = {
      .appName = "Melee PC",
      /* Dawn's D3D12 backend (v20260807.225922, 32-bit x86) crashes a few seconds into
       * first-frame rendering: wgpuSurfaceGetCurrentTexture -> d3d12::Queue::WaitForSerial
       * dereferences a queue-serial value as a pointer (near-NULL read, webgpu_dawn.dll
       * +0x363548). D3D11's queue/serial path does not, so pin it until Dawn is fixed. */
      .desiredBackend = gw_desired_backend(),
      .vsync = true,
      .windowWidth = 1280,
      .windowHeight = 960,
      .logCallback = &gw_aurora_log,
      .logLevel = LOG_INFO,
      /* The port manages MEM1 and ARAM itself, because game code tells main memory from ARAM by
       * comparing addresses against 0x80000000. */
      .mem1Size = 0,
      .mem2Size = 0,
  };
  AuroraInfo info = aurora_initialize(argc, argv, &config);
  gw_log("melee-pc: aurora backend %d, window %ux%u", (int)info.backend, info.windowSize.width,
         info.windowSize.height);

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

  gw_log("melee-pc: entering game main()");
  gw_start_watchdog();
  gw_main();

  gw_log("melee-pc: game main() returned");
  gw_dump_stub_summary();
  aurora_shutdown();
  return 0;
}
