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
  if (level >= LOG_WARNING) {
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

int main(int argc, char *argv[]) {
  gw_install_crash_handler();
  gw_log("melee-pc: starting");

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
      .desiredBackend = BACKEND_D3D11,
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
