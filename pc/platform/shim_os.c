/* OS shims.
 *
 * Aurora provides the SDK's memory and calendar-time pieces, but not interrupts, alarms, threads or
 * contexts. It also owns no arena in this port: main.c maps MEM1 at 0x80000000 itself (Aurora is
 * configured with mem1Size = 0), so the arena getters/setters live here. The heap calls still
 * forward to Aurora, whose allocator works on whatever range OSInitAlloc is handed -- always inside
 * this port's MEM1, so every block it returns keeps the game's ">= 0x80000000 means main memory"
 * invariant.
 *
 * The clock is the port's virtual GameCube clock (shim_vi.h). OSGetTime returning it -- rather than
 * Aurora's wall-clock-based time -- is what keeps alarm scheduling coherent with the frame driver:
 * gw_os_run_alarms compares deadlines against the same counter.
 *
 * Game code reads __OSCurrHeap directly, and gwtool byte-swaps every access, so its storage here is
 * big-endian like any other game-visible data. */
#include "shim_os.h"
#include "shim_vi.h"

#include <dolphin/os.h>

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ---- game-visible globals ----------------------------------------------------------------- */

/* -1 (0xFFFFFFFF) in big-endian byte order. */
unsigned char gw___OSCurrHeap[4] = {0xFF, 0xFF, 0xFF, 0xFF};

static int gw_cur_heap(void) { return (int)(int32_t)gw_r32(gw___OSCurrHeap); }
static void gw_set_cur_heap(int heap) { gw_w32(gw___OSCurrHeap, (uint32_t)heap); }

/* ---- arena --------------------------------------------------------------------------------
 * The game carves framebuffers, the FIFO, the audio heap and the main heap out of the arena
 * during boot, and reports what is left as "system" memory.
 *
 * The arena must not start at the very bottom of MEM1. A real GameCube reserves the low pages for
 * the OS globals, the exception vectors and the disc header, and OSInit leaves __OSArenaLo above
 * all of that. Starting at 0x80000000 instead put the game's first allocation straight over the
 * globals gw_init_lomem writes, which zeroed __OSBusClock at 0x800000F8. Every OSSecondsToTicks
 * then evaluated to zero, lb_0195.c never armed the pad alarm, and the scene loop spun on an
 * empty pad queue forever without drawing -- a black window with no error anywhere. */

/* Matches the console's post-OSInit __OSArenaLo: past the globals, the 0x100-0x3000 exception
 * vectors and the boot info block. */
#define GW_ARENA_LO_OFFSET 0x3100u

static uintptr_t gw_arena_lo;
static uintptr_t gw_arena_hi;

static void gw_arena_ensure(void) {
  if (gw_arena_lo == 0) {
    gw_arena_lo = (uintptr_t)gw_mem1 + GW_ARENA_LO_OFFSET;
    gw_arena_hi = (uintptr_t)gw_mem1 + gw_mem1_size;
  }
}

void *gw_OSGetArenaLo(void) {
  gw_arena_ensure();
  return (void *)gw_arena_lo;
}

void *gw_OSGetArenaHi(void) {
  gw_arena_ensure();
  return (void *)gw_arena_hi;
}

void gw_OSSetArenaLo(void *lo) {
  gw_arena_ensure();
  gw_arena_lo = (uintptr_t)lo;
}

static uintptr_t gw_align_up(uintptr_t value, u32 align) {
  if (align < 2) {
    return value;
  }
  return (value + (align - 1)) & ~(uintptr_t)(align - 1);
}

void *gw_OSAllocFromArenaLo(u32 size, u32 align) {
  gw_arena_ensure();
  uintptr_t p = gw_align_up(gw_arena_lo, align);
  gw_arena_lo = p + size;
  return (void *)p;
}

void *gw_OSAllocFromArenaHi(u32 size, u32 align) {
  gw_arena_ensure();
  uintptr_t p = gw_align_up(gw_arena_hi - size, align);
  gw_arena_hi = p;
  return (void *)p;
}

/* ---- heap --------------------------------------------------------------------------------- */

void *gw_OSInitAlloc(void *arena_start, void *arena_end, int max_heaps) {
  gw_set_cur_heap(-1);
  return OSInitAlloc(arena_start, arena_end, max_heaps);
}

int gw_OSCreateHeap(void *start, void *end) { return OSCreateHeap(start, end); }

void gw_OSDestroyHeap(int heap) {
  OSDestroyHeap(heap);
  if (gw_cur_heap() == heap) {
    gw_set_cur_heap(-1);
  }
}

void *gw_OSAllocFromHeap(int heap, u32 size) { return OSAllocFromHeap(heap, size); }

void gw_OSFreeToHeap(int heap, void *ptr) { OSFreeToHeap(heap, ptr); }

int gw_OSCheckHeap(int heap) { return OSCheckHeap(heap); }

int gw_OSSetCurrentHeap(int heap) {
  int old = gw_cur_heap();
  gw_set_cur_heap(heap);
  OSSetCurrentHeap(heap);
  return old;
}

/* ---- memory sizes ------------------------------------------------------------------------- */

/* Aurora reports 0 here because the port disabled its MEM1; the game uses this value for its
 * memory report, so answer for the region the port actually mapped. */
u32 gw_OSGetPhysicalMemSize(void) { return gw_mem1_size; }

/* Retail GameCube. Reporting 48 MB would send gmmain.c down the devkit path and reserve 24 MB. */
u32 gw_OSGetConsoleSimulatedMemSize(void) { return 24u * 1024u * 1024u; }

/* ---- time --------------------------------------------------------------------------------- */

int64_t gw_OSGetTime(void) { return (int64_t)gw_time_ticks(); }

int gw_OSGetTick(void) { return (int)(int32_t)(uint32_t)gw_time_ticks(); }

void gw_OSTicksToCalendarTime(int64_t ticks, void *td) {
  OSTicksToCalendarTime((OSTime)ticks, (OSCalendarTime *)td);
}

/* ---- interrupts --------------------------------------------------------------------------- */

/* Nothing in the port is actually interrupt-driven; the frame driver is the only async source, and
 * it runs on the main thread. */
int gw_OSDisableInterrupts(void) { return 0; }

int gw_OSRestoreInterrupts(int level) {
  (void)level;
  return 0;
}

/* ---- alarms -------------------------------------------------------------------------------
 * The GameCube raises timer interrupts; the port fires these from gw_frame_tick instead. An alarm
 * is tracked by the address of the OSAlarm the game owns, which is opaque here. */

#define GW_MAX_ALARMS 16

typedef struct {
  void *handle;
  uint64_t fire_at;
  uint64_t period; /* 0 = one-shot */
  void *handler;
} gw_alarm;

static gw_alarm gw_alarms[GW_MAX_ALARMS];

static gw_alarm *gw_alarm_find(void *handle) {
  gw_alarm *free_slot = NULL;
  if (handle == NULL) {
    return NULL;
  }
  for (int i = 0; i < GW_MAX_ALARMS; ++i) {
    if (gw_alarms[i].handle == handle) {
      return &gw_alarms[i];
    }
    if (gw_alarms[i].handle == NULL && free_slot == NULL) {
      free_slot = &gw_alarms[i];
    }
  }
  return free_slot;
}

void gw_OSInitAlarm(void) {}

void gw_OSCreateAlarm(void *alarm) {
  gw_alarm *a = gw_alarm_find(alarm);
  if (a != NULL) {
    a->handler = NULL;
    a->period = 0;
    a->fire_at = 0;
  }
}

void gw_OSSetAlarm(void *alarm, int64_t ticks, void *handler) {
  gw_alarm *a = gw_alarm_find(alarm);
  if (a == NULL) {
    gw_log("gw: out of alarm slots for OSSetAlarm");
    return;
  }
  a->handle = alarm;
  a->handler = handler;
  a->period = 0;
  a->fire_at = gw_time_ticks() + (uint64_t)ticks;
}

void gw_OSSetPeriodicAlarm(void *alarm, int64_t start, int64_t period, void *handler) {
  gw_alarm *a = gw_alarm_find(alarm);
  if (a == NULL) {
    gw_log("gw: out of alarm slots for OSSetPeriodicAlarm");
    return;
  }
  a->handle = alarm;
  a->handler = handler;
  a->period = (uint64_t)period;
  a->fire_at = gw_time_ticks() + (uint64_t)start;
  gw_log("gw: periodic alarm armed handle=%p start=%lld period=%lld", alarm, (long long)start,
         (long long)period);
}

void gw_OSCancelAlarm(void *alarm) {
  gw_alarm *a = gw_alarm_find(alarm);
  if (a != NULL && a->handle == alarm) {
    a->handle = NULL;
    a->handler = NULL;
    a->period = 0;
  }
}

static uint32_t gw_alarm_fire_count;

void gw_os_alarm_stats(uint32_t *active, uint32_t *fired) {
  uint32_t n = 0;
  for (int i = 0; i < GW_MAX_ALARMS; ++i) {
    if (gw_alarms[i].handle != NULL && gw_alarms[i].handler != NULL) {
      ++n;
    }
  }
  *active = n;
  *fired = gw_alarm_fire_count;
}

void gw_os_run_alarms(uint64_t ticks) {
  /* Not re-entrant, for the same reason gw_run_deferred is not: a handler is free to call
   * something that ends up back in gw_wait_idle -- DVDGetDriveStatus does -- and on hardware a
   * timer interrupt cannot preempt itself. Without this the handler recurses until it deadlocks
   * on the CRT's lock inside a nested log call. */
  static bool running;
  if (running) {
    return;
  }
  running = true;
  for (int i = 0; i < GW_MAX_ALARMS; ++i) {
    gw_alarm *a = &gw_alarms[i];
    while (a->handle != NULL && a->handler != NULL && ticks >= a->fire_at) {
      void *handler = a->handler;
      void *handle = a->handle;
      uint64_t fire_at = a->fire_at;
      /* Handlers are void(void) functions cast to OSAlarmHandler; the extra arguments are
       * ignored on both ABIs. */
      ++gw_alarm_fire_count;
      ((void (*)(void *, void *))handler)(handle, NULL);
      /* A handler may cancel the alarm (OSCancelAlarm clears handler), or re-arm a one-shot by
       * calling OSSetAlarm with the same handler -- lbMemory's chunked memcpy (fn_80015184) does
       * exactly that, rescheduling itself 3ms out. OSSetAlarm bumps fire_at, so treating
       * "handler unchanged" as "still idle" cancelled the re-armed alarm after its first chunk
       * and left the copy (and the preload heap compaction that drives it) stalled forever.
       * Compare fire_at too, so a re-armed one-shot survives. */
      if (a->handler != handler || a->fire_at != fire_at) {
        break; /* cancelled or re-armed inside the handler */
      }
      if (a->period == 0) {
        a->handle = NULL;
        a->handler = NULL;
        break;
      }
      a->fire_at += a->period;
    }
  }
  running = false;
}

/* ---- threads and contexts -----------------------------------------------------------------
 * The game is single-threaded on retail (only the debug console creates a thread, and it is
 * DbLevel-gated), so contexts are inert storage and thread creation reports failure. */

static unsigned char gw_dummy_context[0x400];

void *gw_OSGetCurrentContext(void) { return gw_dummy_context; }

void gw_OSSetCurrentContext(void *context) { (void)context; }

void gw_OSClearContext(void *context) {
  if (context != NULL) {
    memset(context, 0, 0x2C8);
  }
}

int gw_OSSaveContext(void *context) {
  (void)context;
  return 0;
}

void gw_OSSaveFPUContext(void *context) { (void)context; }

void gw_OSLoadFPUContext(void *context) { (void)context; }

int gw_OSCreateThread(void *thread, void *func, void *arg, void *stack, u32 stack_size, int prio,
                      u16 attr) {
  (void)thread;
  (void)func;
  (void)arg;
  (void)stack;
  (void)stack_size;
  (void)prio;
  (void)attr;
  return 0;
}

int gw_OSResumeThread(void *thread) {
  (void)thread;
  return 0;
}

int gw_OSCheckActiveThreads(void) { return 0; }

/* ---- reset and modes ---------------------------------------------------------------------- */

/* gmmain_lib.c reads this to set skip_intro: 0x80000000 is the "rebooted from the IPL" code,
 * which skips the opening movie, and anything else plays it. This used to be hardcoded to skip,
 * because THP decode was stubbed out and the movie would only have drawn garbage. The decoder is
 * real now (extern/dolphin/src/dolphin/thp/THPDec.c), so report a cold boot - which is what
 * launching the executable actually is - and let MvOpen.mth play, Start-skippable exactly as on
 * console. MELEE_SKIP_INTRO=1 restores the old straight-to-title behaviour. */
int gw_OSGetResetCode(void) {
  return getenv("MELEE_SKIP_INTRO") != NULL ? (int32_t)0x80000000u : 0;
}

int gw_OSGetResetSwitchState(void) { return 0; }

void gw_OSResetSystem(int reset, u32 code, int force) {
  gw_log("gw: OSResetSystem(%d, 0x%08X, %d)", reset, code, force);
  gw_dump_stub_summary();
  exit(0);
}

/* 0 keeps boot out of the progressive-scan prompt (gm_1A3F.c checks this against HSD_PAD_B). */
int gw_OSGetProgressiveMode(void) { return 0; }

void gw_OSSetProgressiveMode(int mode) { (void)mode; }

int gw_OSGetSoundMode(void) { return 0; }

void gw_OSSetSoundMode(int mode) { (void)mode; }

void *gw_OSSetErrorHandler(u16 type, void *handler) {
  (void)type;
  (void)handler;
  return NULL;
}

/* ---- diagnostics -------------------------------------------------------------------------- */

void gw_OSReport(const char *fmt, ...) {
  va_list ap;
  /* With MELEE_PC_TRACE_OSREPORT set, the raw format goes out before anything is expanded.
   * Formatting game-supplied arguments is one place a bad pointer can take the CRT down, and
   * __fastfail bypasses SEH -- so the last line would otherwise be whatever printed fine,
   * not the call that died. Off by default: it doubles the log. */
  if (gw_trace_osreport()) {
    gw_log_raw("OSReport fmt: ", fmt);
  }
  va_start(ap, fmt);
  gw_logv(fmt, ap);
  va_end(ap);
}

void gw_OSPanic(const char *file, int line, const char *fmt, ...) {
  char buf[1024];
  va_list ap;
  va_start(ap, fmt);
  vsnprintf(buf, sizeof buf, fmt, ap);
  va_end(ap);
  gw_panic("%s:%d: %s", file != NULL ? file : "?", line, buf);
}

void gw_OSInit(void) { OSInit(); }
