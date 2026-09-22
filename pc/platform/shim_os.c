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

/* The top GW_MEX_PERSIST_SIZE bytes of MEM1 are withheld from the game's arena and belong to the
 * m-ex runtime (gw_mex_persist_alloc in gw_mex_ftfunction_runtime.c): its SHARED, process-lifetime
 * guest state - mexData (MxDt.dat), the interpreter's guest stack, the Arch_FighterFunc holder.
 * Scene-scoped heaps cannot hold these: allocated from HSD_MemAlloc they were freed at the end of
 * the first match and the second VS match jumped into overwritten memory (user-found:
 * "unimplemented opcode at 0x807A77E0 (OnLoad)"). m-ex does the same with a persistent heap
 * (asm/m-ex/Persistent Heap Expansion/). Per-fighter CODE is not here: it is relocated in place
 * inside the fighter's own loaded file, so it costs nothing per fighter (gw_Mex_FtFunctionInstall).
 *
 * Taking it off the top is the safe place: lbheap carves its fixed heaps downward from the arena's
 * top and gives the MAIN heap whatever remains (lbheap.c), so this only shrinks the main heap,
 * measured at ~11.3 MB with little in use. 384 KB holds the shared state (~260 KB on ACE,
 * whose MxDt.dat is 164 KB). */
#define GW_MEX_PERSIST_SIZE 0x60000u

static uintptr_t gw_arena_lo;
static uintptr_t gw_arena_hi;

static void gw_arena_ensure(void) {
  if (gw_arena_lo == 0) {
    gw_arena_lo = (uintptr_t)gw_mem1 + GW_ARENA_LO_OFFSET;
    gw_arena_hi = (uintptr_t)gw_mem1 + gw_mem1_size - GW_MEX_PERSIST_SIZE;
  }
}

/* The last GW_GUEST_SCRATCH_SIZE bytes of the withheld region are not the m-ex runtime's; see
 * gw_guest_scratch below. Everything above still belongs to gw_mex_persist_alloc. */
#define GW_GUEST_SCRATCH_SIZE 0x100u
#define GW_GUEST_SCRATCH_SLOT 0x20u

/* The withheld region: [base, base + size). Never touched by the game. */
void gw_mex_persist_region(uint32_t *base, uint32_t *size) {
  *base = (uint32_t)((uintptr_t)gw_mem1 + gw_mem1_size - GW_MEX_PERSIST_SIZE);
  *size = GW_MEX_PERSIST_SIZE - GW_GUEST_SCRATCH_SIZE;
}

/* ---- guest scratch for pointer arguments handed to GUEST callbacks -------------------------
 * A native function that passes `&local` to a callback is fine as long as the callee is native
 * too. It is NOT fine when the callee is an interpreted m-ex blob: a host stack address is
 * outside MEM1, so the interpreter's first load through it is a guest access violation - or,
 * before the interpreter got its bounds check, a fault deep inside VCRUNTIME140's memcpy.
 * sysdolphin hits this wherever an animation update function comes from the disc rather than
 * from the engine: HSD_ObjUpdateFunc in fobj.c/robj.c, reached from an m-ex stage's grFunction.
 *
 * This hands out small, permanently mapped GUEST buffers instead. It is a round robin, not an
 * allocator: nothing is freed and slots are reused, which is correct because every caller uses
 * its slot for exactly the duration of one call and the few slots cover the nesting that one
 * callback re-entering the animation system can produce. Contents are undefined on entry; the
 * caller writes the value it wants the callee to see. Living inside the withheld top of MEM1
 * means it survives a scene change and the test harness's MEM1 restore, and needs no
 * invalidation hook. */
void *gw_guest_scratch(uint32_t size) {
  static uint32_t next;
  uint32_t base;
  if (size > GW_GUEST_SCRATCH_SLOT) {
    gw_panic("gw_guest_scratch: %u bytes requested, slot is %u", size,
             (uint32_t)GW_GUEST_SCRATCH_SLOT);
  }
  base = (uint32_t)((uintptr_t)gw_mem1 + gw_mem1_size - GW_GUEST_SCRATCH_SIZE);
  base += (next % (GW_GUEST_SCRATCH_SIZE / GW_GUEST_SCRATCH_SLOT)) * GW_GUEST_SCRATCH_SLOT;
  ++next;
  return (void *)(uintptr_t)base;
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

/* Aurora writes each OSCalendarTime field natively (little-endian) into the game's buffer, but the
 * game reads them big-endian, so the save-description date comes out garbage (DEVLOG §13.6.5).
 * Swap every 32-bit field in place, one by one, so the swap stays correct even if the struct's
 * layout ever gains padding. All ten fields are 32-bit: sec/min/hour/mday/mon/year/wday/yday/
 * msec/usec. */
static void gw_calendar_time_bswap(OSCalendarTime *td) {
  td->sec = (int)gw_bswap32((uint32_t)td->sec);
  td->min = (int)gw_bswap32((uint32_t)td->min);
  td->hour = (int)gw_bswap32((uint32_t)td->hour);
  td->mday = (int)gw_bswap32((uint32_t)td->mday);
  td->mon = (int)gw_bswap32((uint32_t)td->mon);
  td->year = (int)gw_bswap32((uint32_t)td->year);
  td->wday = (int)gw_bswap32((uint32_t)td->wday);
  td->yday = (int)gw_bswap32((uint32_t)td->yday);
  td->msec = (int)gw_bswap32((uint32_t)td->msec);
  td->usec = (int)gw_bswap32((uint32_t)td->usec);
}

void gw_OSTicksToCalendarTime(int64_t ticks, void *td) {
  OSCalendarTime *cal = (OSCalendarTime *)td;
  OSTicksToCalendarTime((OSTime)ticks, cal);
  gw_calendar_time_bswap(cal);
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
uint64_t gw_os_alarm_late_ticks; /* lateness of the alarm now firing, in OS ticks */

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

int gw_os_pad_alarm_deadline(uint64_t period_ticks, uint64_t *fire_at) {
  for (int i = 0; i < GW_MAX_ALARMS; ++i) {
    const gw_alarm *a = &gw_alarms[i];
    if (a->handle != NULL && a->handler != NULL && a->period != 0 &&
        a->period * 10u >= period_ticks * 9u && a->period * 10u <= period_ticks * 11u) {
      *fire_at = a->fire_at;
      return 1;
    }
  }
  return 0;
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
      /* MELEE_INPUT_PROFILE (shim_vi.c): how late this firing is against its deadline. The pad
       * alarm's lateness is exactly the extra age of the controller sample it takes. */
      gw_os_alarm_late_ticks = gw_time_ticks() - fire_at;
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
