/* Core runtime for the game world: startup fixups, memory regions, logging. */
#include "gw.h"

#include "shim_gx.h"
#include "shim_os.h"
#include "shim_vi.h"

#include <intrin.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

/* ---- link-time pointer fixups -------------------------------------------------------------
 * gwtool stores scalars in global initializers pre-swapped, but a slot holding the address of
 * another symbol is filled in by the linker and so cannot be swapped ahead of time. Each object
 * file lists those slots in section .gwfix$m; the markers below bracket the merged list. */
#pragma section(".gwfix$a", read)
#pragma section(".gwfix$z", read)
__declspec(allocate(".gwfix$a")) static void *const gw_fixups_start[1] = {0};
__declspec(allocate(".gwfix$z")) static void *const gw_fixups_end[1] = {0};

void gw_apply_fixups(void) {
  uint32_t **p = (uint32_t **)(gw_fixups_start + 1);
  uint32_t **end = (uint32_t **)gw_fixups_end;
  size_t count = 0;
  for (; p < end; ++p) {
    if (*p == NULL) {
      continue; /* alignment padding between object-file contributions */
    }
    **p = gw_bswap32(**p);
    ++count;
  }
  gw_log("gw: byte-swapped %zu link-time pointers in game globals", count);
}

/* ---- memory regions ---------------------------------------------------------------------- */

unsigned char *gw_mem1;
uint32_t gw_mem1_size;
unsigned char *gw_aram;
uint32_t gw_aram_size;

#define GW_MEM1_BASE ((void *)0x80000000u)
#define GW_MEM1_SIZE (24u * 1024u * 1024u)
#define GW_ARAM_SIZE (16u * 1024u * 1024u)

bool gw_mem_init(void) {
  gw_mem1 = (unsigned char *)VirtualAlloc(GW_MEM1_BASE, GW_MEM1_SIZE, MEM_RESERVE | MEM_COMMIT,
                                          PAGE_READWRITE);
  if (gw_mem1 == NULL) {
    /* Fatal, not a fallback. Game code tells main memory from ARAM by comparing against
     * 0x80000000 (lbmemory.c:68), and so does the port (gw_ar_addr, shim_gx.c's array bounds).
     * Running from any other address inverts every one of those tests and corrupts silently, so
     * a wrong answer here is worse than not starting. Needs /LARGEADDRESSAWARE to succeed. */
    gw_panic("could not map MEM1 at 0x80000000 (error %lu). The port cannot run from another "
             "address: check that melee-pc.exe is linked /LARGEADDRESSAWARE.",
             GetLastError());
  }
  gw_mem1_size = GW_MEM1_SIZE;
  gw_aram = (unsigned char *)calloc(1, GW_ARAM_SIZE);
  if (gw_aram == NULL) {
    return false;
  }
  gw_aram_size = GW_ARAM_SIZE;
  gw_init_lomem();
  gw_log("gw: MEM1 %u MB at %p, ARAM %u MB at %p", gw_mem1_size >> 20, (void *)gw_mem1,
         gw_aram_size >> 20, (void *)gw_aram);
  return true;
}

/* ---- low-memory OS globals ------------------------------------------------------------------
 * A handful of values live at fixed addresses in the bottom of MEM1, written by the console's IPL
 * before the game ever runs. dolphin/os.h reads them straight through pointers -- __OSBusClock is
 * literally *(u32*)0x800000F8 -- so on PC they are simply zero unless something fills them in.
 *
 * That is not cosmetic. OS_TIMER_CLOCK is OS_BUS_CLOCK / 4, so a zero bus clock makes every
 * OSSecondsToTicks and OSMillisecondsToTicks evaluate to zero. lb_0195.c:87 then computes a pad
 * sampling period of 0, finds it already equals the stored period, returns early, and never arms
 * the pad alarm -- so the pad queue stays empty and the scene loop in gmscene.c:292 spins on
 * lb_80019894() forever without ever drawing a frame.
 *
 * Written big-endian: game code reads these through gwtool-swapped loads. */
void gw_init_lomem(void) {
  if (gw_mem1 == NULL) {
    return;
  }
  /* Retail GameCube: 162 MHz bus, 486 MHz core. The port's own virtual timer runs at
   * GW_TIMER_CLOCK = 40.5 MHz in shim_vi.c, which is exactly this bus clock over four. */
  gw_w32(gw_mem1 + 0x00F8, 162000000u); /* __OSBusClock  */
  gw_w32(gw_mem1 + 0x00FC, 486000000u); /* __OSCoreClock */
  gw_w32(gw_mem1 + 0x0028, gw_mem1_size); /* __OSPhysicalMemSize  */
  gw_w32(gw_mem1 + 0x00F0, gw_mem1_size); /* __OSSimulatedMemSize */
  gw_w32(gw_mem1 + 0x00CC, 0u);           /* __OSTVMode: 0 = NTSC */
  gw_log("gw: lomem BusClock=%u CoreClock=%u PhysMem=%u (read back big-endian)",
         gw_r32(gw_mem1 + 0x00F8), gw_r32(gw_mem1 + 0x00FC), gw_r32(gw_mem1 + 0x0028));
}

/* ---- float and matrix marshalling --------------------------------------------------------- */

void gw_read_f32v(float *dst, const void *src, int count) {
  const unsigned char *s = (const unsigned char *)src;
  for (int i = 0; i < count; ++i) {
    dst[i] = gw_rf32(s + 4 * i);
  }
}

void gw_write_f32v(void *dst, const float *src, int count) {
  unsigned char *d = (unsigned char *)dst;
  for (int i = 0; i < count; ++i) {
    gw_wf32(d + 4 * i, src[i]);
  }
}

void gw_read_mtx(float dst[3][4], const void *src) { gw_read_f32v(&dst[0][0], src, 12); }
void gw_write_mtx(void *dst, const float src[3][4]) { gw_write_f32v(dst, &src[0][0], 12); }
void gw_read_mtx44(float dst[4][4], const void *src) { gw_read_f32v(&dst[0][0], src, 16); }
void gw_write_mtx44(void *dst, const float src[4][4]) { gw_write_f32v(dst, &src[0][0], 16); }

/* ---- logging ------------------------------------------------------------------------------ */

static FILE *gw_log_file;

void gw_logv(const char *fmt, va_list ap) {
  va_list copy;
  va_copy(copy, ap);
  vfprintf(stdout, fmt, ap);
  fputc('\n', stdout);
  fflush(stdout);
  if (gw_log_file == NULL) {
    gw_log_file = fopen("melee-pc.log", "w");
  }
  if (gw_log_file != NULL) {
    vfprintf(gw_log_file, fmt, copy);
    fputc('\n', gw_log_file);
    fflush(gw_log_file);
  }
  va_end(copy);
}

/* Opt-in tracing of every OSReport format string, before it is expanded. Set
 * MELEE_PC_TRACE_OSREPORT=1 when chasing a crash inside the formatter. */
bool gw_trace_osreport(void) {
  static int state = -1;
  if (state < 0) {
    const char *v = getenv("MELEE_PC_TRACE_OSREPORT");
    state = (v != NULL && v[0] != '\0' && v[0] != '0') ? 1 : 0;
  }
  return state != 0;
}

/* Writes two strings verbatim, with no format expansion, for use when the thing being reported
 * is itself a format string that may be about to crash the formatter. */
void gw_log_raw(const char *prefix, const char *text) {
  fputs(prefix, stdout);
  fputs(text != NULL ? text : "(null)", stdout);
  fputc('\n', stdout);
  fflush(stdout);
  if (gw_log_file == NULL) {
    gw_log_file = fopen("melee-pc.log", "w");
  }
  if (gw_log_file != NULL) {
    fputs(prefix, gw_log_file);
    fputs(text != NULL ? text : "(null)", gw_log_file);
    fputc('\n', gw_log_file);
    fflush(gw_log_file);
  }
}

void gw_log(const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  gw_logv(fmt, ap);
  va_end(ap);
}

void gw_panic(const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  fputs("gw: PANIC ", stdout);
  gw_logv(fmt, ap);
  va_end(ap);
  gw_dump_stub_summary();
  abort();
}

#define GW_MAX_STUBS 512
static struct {
  const char *name;
  unsigned long count;
} gw_stubs[GW_MAX_STUBS];
static int gw_stub_count;

void gw_stub_hit(const char *name) {
  for (int i = 0; i < gw_stub_count; ++i) {
    if (gw_stubs[i].name == name || strcmp(gw_stubs[i].name, name) == 0) {
      ++gw_stubs[i].count;
      return;
    }
  }
  if (gw_stub_count < GW_MAX_STUBS) {
    gw_stubs[gw_stub_count].name = name;
    gw_stubs[gw_stub_count].count = 1;
    ++gw_stub_count;
  }
  gw_log("gw: stub %s", name);
}

void gw_dump_stub_summary(void) {
  gw_log("gw: %d distinct stubs were called:", gw_stub_count);
  for (int i = 0; i < gw_stub_count; ++i) {
    gw_log("gw:   %-32s %lu", gw_stubs[i].name, gw_stubs[i].count);
  }
}

/* ---- crash reporting -----------------------------------------------------------------------
 * Without this a fault just stops the log mid-line, which is indistinguishable from a hang. The
 * addresses are reported as .map RVAs (image base + offset) so a fault can be turned into a
 * function name with a single grep of melee-pc.map, rather than going through the Windows event
 * log. db_SetupCrashHandler is game-side and expects OS exception handling the shims do not
 * provide, so this is the only crash reporting the port has. */

/* The port's fixed image base (/BASE in build_melee_pc.bat). Because the image is also linked
 * /DYNAMICBASE:NO, a runtime address equals the third column of melee-pc.map exactly. */
#define GW_MAP_IMAGE_BASE 0x10000000u

static uintptr_t gw_image_base;

/* Describes a code address as "melee-pc.map rva 0x...", or as "module+offset" when the fault is
 * inside Dawn/SDL3/the CRT instead. Without the distinction a DLL address gets reported as a map
 * RVA that resolves to an unrelated game function. Returns dst. */
static const char *gw_describe_code_addr(uintptr_t addr, char *dst, size_t dstlen) {
  HMODULE mod = NULL;
  if (GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
                             GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                         (LPCWSTR)addr, &mod) == 0 ||
      mod == NULL) {
    snprintf(dst, dstlen, "0x%08X (no module)", (uint32_t)addr);
    return dst;
  }
  if ((uintptr_t)mod == gw_image_base) {
    snprintf(dst, dstlen, "melee-pc.map rva 0x%08X",
             (uint32_t)(addr - gw_image_base + GW_MAP_IMAGE_BASE));
    return dst;
  }
  {
    wchar_t path[MAX_PATH];
    const wchar_t *name = L"?";
    if (GetModuleFileNameW(mod, path, MAX_PATH) != 0) {
      const wchar_t *slash = wcsrchr(path, L'\\');
      name = slash != NULL ? slash + 1 : path;
    }
    snprintf(dst, dstlen, "%ls+0x%X", name, (unsigned)(addr - (uintptr_t)mod));
  }
  return dst;
}

static const char *gw_exception_name(DWORD code) {
  switch (code) {
  case EXCEPTION_ACCESS_VIOLATION:
    return "ACCESS_VIOLATION";
  case EXCEPTION_ARRAY_BOUNDS_EXCEEDED:
    return "ARRAY_BOUNDS_EXCEEDED";
  case EXCEPTION_DATATYPE_MISALIGNMENT:
    return "DATATYPE_MISALIGNMENT";
  case EXCEPTION_FLT_DIVIDE_BY_ZERO:
    return "FLT_DIVIDE_BY_ZERO";
  case EXCEPTION_FLT_INVALID_OPERATION:
    return "FLT_INVALID_OPERATION";
  case EXCEPTION_ILLEGAL_INSTRUCTION:
    return "ILLEGAL_INSTRUCTION";
  case EXCEPTION_IN_PAGE_ERROR:
    return "IN_PAGE_ERROR";
  case EXCEPTION_INT_DIVIDE_BY_ZERO:
    return "INT_DIVIDE_BY_ZERO";
  case EXCEPTION_PRIV_INSTRUCTION:
    return "PRIV_INSTRUCTION";
  case EXCEPTION_STACK_OVERFLOW:
    return "STACK_OVERFLOW";
  default:
    return "unknown";
  }
}

/* Which region an address falls in. MEM1/ARAM are the interesting cases: a fault inside them is a
 * game-data bug, a fault just below 0x80000000 is usually a NULL-plus-offset dereference. */
static const char *gw_region_of(uintptr_t addr) {
  if (gw_mem1 != NULL && addr >= (uintptr_t)gw_mem1 &&
      addr < (uintptr_t)gw_mem1 + gw_mem1_size) {
    return " (in MEM1)";
  }
  if (gw_aram != NULL && addr >= (uintptr_t)gw_aram &&
      addr < (uintptr_t)gw_aram + gw_aram_size) {
    return " (in ARAM)";
  }
  if (addr < 0x10000u) {
    return " (near NULL)";
  }
  return "";
}

static LONG WINAPI gw_unhandled_exception(EXCEPTION_POINTERS *ep) {
  const EXCEPTION_RECORD *er = ep->ExceptionRecord;
  const CONTEXT *ctx = ep->ContextRecord;
  const uintptr_t pc = (uintptr_t)er->ExceptionAddress;
  char where[MAX_PATH + 64];

  gw_log("gw: FATAL %s (0x%08lX) at %p  %s%s", gw_exception_name(er->ExceptionCode),
         (unsigned long)er->ExceptionCode, er->ExceptionAddress,
         gw_describe_code_addr(pc, where, sizeof where), gw_region_of(pc));

  if (er->ExceptionCode == EXCEPTION_ACCESS_VIOLATION ||
      er->ExceptionCode == EXCEPTION_IN_PAGE_ERROR) {
    const ULONG_PTR kind = er->ExceptionInformation[0];
    const uintptr_t target = (uintptr_t)er->ExceptionInformation[1];
    const char *what = kind == 0 ? "read of" : kind == 1 ? "write to" : "execute of";
    gw_log("gw:   %s 0x%08X%s", what, (uint32_t)target, gw_region_of(target));
  }

  gw_log("gw:   eip=%08lX esp=%08lX ebp=%08lX eax=%08lX ebx=%08lX ecx=%08lX edx=%08lX",
         ctx->Eip, ctx->Esp, ctx->Ebp, ctx->Eax, ctx->Ebx, ctx->Ecx, ctx->Edx);

  /* Frame-pointer walk. The game world is built at -O2, so clang may omit frame pointers and this
   * can miss or invent frames -- it is a hint for reading the map, not a real unwind. */
  gw_log("gw:   frames (innermost first):");
  gw_log("gw:     %s", gw_describe_code_addr(pc, where, sizeof where));
  {
    uintptr_t *frame = (uintptr_t *)(uintptr_t)ctx->Ebp;
    for (int depth = 0; depth < 24; ++depth) {
      if (frame == NULL || IsBadReadPtr(frame, 2 * sizeof(uintptr_t)) != 0) {
        break;
      }
      const uintptr_t next = frame[0];
      const uintptr_t ret = frame[1];
      if (ret == 0) {
        break;
      }
      gw_log("gw:     %s", gw_describe_code_addr(ret, where, sizeof where));
      if (next <= (uintptr_t)frame) {
        break; /* stacks grow down: a non-increasing link means the chain is junk */
      }
      frame = (uintptr_t *)next;
    }
  }

  /* The FP walk fails whenever -O2 omits frame pointers (most game/shim code). Scan the raw stack
   * for words that fall inside the game image, which recovers the return-address chain. */
  gw_log("gw:   stack scan (image addresses):");
  {
    const uintptr_t *sp = (const uintptr_t *)(uintptr_t)ctx->Esp;
    uintptr_t base = gw_image_base;
    int found = 0;
    int i;
    if (base == 0) {
      base = (uintptr_t)GetModuleHandleW(NULL);
      gw_image_base = base;
    }
    for (i = 1; i < 2048 && found < 48; ++i) {
      uintptr_t w;
      if (IsBadReadPtr(sp + i, sizeof(uintptr_t)) != 0) {
        break;
      }
      w = sp[i];
      if (w >= base && w < base + 0x1000000u) {
        gw_log("gw:     [esp+%04X] %s", (unsigned)(i * 4),
               gw_describe_code_addr(w, where, sizeof where));
        ++found;
      }
    }
  }

  gw_log("gw:   for a map rva, take the melee-pc.map entry with the greatest address <= it");
  gw_dump_stub_summary();
  return EXCEPTION_EXECUTE_HANDLER;
}

void gw_log_code_addr(const char *label, const void *addr) {
  char where[MAX_PATH + 64];
  if (gw_image_base == 0) {
    gw_image_base = (uintptr_t)GetModuleHandleW(NULL);
  }
  gw_log("gw: %s %s", label, gw_describe_code_addr((uintptr_t)addr, where, sizeof where));
}

/* The CRT does not raise a catchable exception when it rejects an argument -- it calls __fastfail,
 * which bypasses SEH, so gw_unhandled_exception never runs and the process simply disappears with
 * STATUS_STACK_BUFFER_OVERRUN (0xC0000409) in the Windows event log. Intercepting the handler is
 * the only way to get a message and a stack out of it. In a release CRT the expression, function
 * and file are all NULL, so the frame walk is the informative part. */
static void gw_invalid_parameter(const wchar_t *expr, const wchar_t *func, const wchar_t *file,
                                 unsigned int line, uintptr_t reserved) {
  char where[MAX_PATH + 64];
  (void)reserved;
  gw_log("gw: FATAL CRT rejected an argument: expr=%ls func=%ls file=%ls line=%u",
         expr != NULL ? expr : L"(release CRT: unavailable)", func != NULL ? func : L"?",
         file != NULL ? file : L"?", line);
  gw_log("gw:   caller: %s", gw_describe_code_addr((uintptr_t)_ReturnAddress(), where,
                                                   sizeof where));
  {
    uintptr_t *frame = (uintptr_t *)_AddressOfReturnAddress() - 1;
    for (int depth = 0; depth < 24; ++depth) {
      uintptr_t next, ret;
      if (frame == NULL || IsBadReadPtr(frame, 2 * sizeof(uintptr_t)) != 0) {
        break;
      }
      next = frame[0];
      ret = frame[1];
      if (ret == 0) {
        break;
      }
      gw_log("gw:     %s", gw_describe_code_addr(ret, where, sizeof where));
      if (next <= (uintptr_t)frame) {
        break;
      }
      frame = (uintptr_t *)next;
    }
  }
  gw_dump_stub_summary();
  _exit(3);
}

/* ---- write watchdog --------------------------------------------------------------------------
 * PAGE_GUARD a region and log the faulting instruction of every access, to catch what overwrites a
 * field whose value changes without a deterministic trigger. The CPU clears the guard on the first
 * access, so the handler logs and disarms; gw_watch_tick re-arms once per frame to catch the next
 * writer. Only the first access per frame is reported, which is enough to spot an unexpected one. */
static void *gw_watch_addr;
static size_t gw_watch_len;
static int gw_watch_armed;

static LONG CALLBACK gw_vectored_exception(EXCEPTION_POINTERS *ep) {
  if (ep->ExceptionRecord->ExceptionCode == STATUS_GUARD_PAGE_VIOLATION) {
    char where[MAX_PATH + 64];
    const uintptr_t pc = (uintptr_t)ep->ExceptionRecord->ExceptionAddress;
    const uintptr_t target = (uintptr_t)ep->ExceptionRecord->ExceptionInformation[1];
    gw_log("gw: GUARD access by %s at %p", gw_describe_code_addr(pc, where, sizeof where),
           (void *)target);
    gw_watch_armed = 0;
    if (gw_watch_addr != NULL) {
      DWORD old;
      VirtualProtect(gw_watch_addr, gw_watch_len, PAGE_READWRITE, &old);
    }
    return EXCEPTION_CONTINUE_EXECUTION;
  }
  return EXCEPTION_CONTINUE_SEARCH;
}

void gw_watch_page(void *addr, size_t size) {
  const uintptr_t page = 0x1000;
  uintptr_t start = (uintptr_t)addr & ~(page - 1);
  uintptr_t end = ((uintptr_t)addr + size + page - 1) & ~(page - 1);
  DWORD old;
  gw_watch_addr = (void *)start;
  gw_watch_len = end - start;
  if (VirtualProtect(gw_watch_addr, gw_watch_len, PAGE_READWRITE | PAGE_GUARD, &old)) {
    gw_watch_armed = 1;
  }
}

void gw_watch_tick(void) {
  if (!gw_watch_armed && gw_watch_addr != NULL) {
    DWORD old;
    if (VirtualProtect(gw_watch_addr, gw_watch_len, PAGE_READWRITE | PAGE_GUARD, &old)) {
      gw_watch_armed = 1;
    }
  }
}

void gw_install_crash_handler(void) {
  gw_image_base = (uintptr_t)GetModuleHandleW(NULL);
  SetUnhandledExceptionFilter(&gw_unhandled_exception);
  _set_invalid_parameter_handler(&gw_invalid_parameter);
  AddVectoredExceptionHandler(1, &gw_vectored_exception);
}

/* ---- watchdog -------------------------------------------------------------------------------
 * Not every failure is a fault. A game loop that never reaches VIWaitForRetrace presents nothing
 * and logs nothing, and from outside is indistinguishable from a hang -- the window just stays
 * black. This samples the game thread's instruction pointer every couple of seconds and names it,
 * which turns "stuck somewhere" into a line in melee-pc.map.
 *
 * The sample is taken with the thread suspended but logged after it resumes: gw_log goes through
 * stdio, and formatting while the game thread holds the CRT's lock would deadlock. */

/* Fast enough that the last sample lands within a tenth of a second of a crash. The port dies on
 * __fastfail paths that bypass SEH, so the last breadcrumb here is sometimes the only evidence
 * of where it was. Only a move to a different function is logged, so a spin stays quiet. */
#define GW_WATCHDOG_INTERVAL_MS 100
#define GW_WATCHDOG_STATS_EVERY 20 /* one full stats line every ~2s */

static HANDLE gw_game_thread;

static DWORD WINAPI gw_watchdog(LPVOID unused) {
  uintptr_t last_pc = 0;
  unsigned same = 0;
  unsigned stats_tick = 0;
  char last_where[MAX_PATH + 64] = "";
  (void)unused;
  for (;;) {
    CONTEXT ctx;
    uintptr_t pc = 0;
    Sleep(GW_WATCHDOG_INTERVAL_MS);
    ctx.ContextFlags = CONTEXT_CONTROL;
    if (SuspendThread(gw_game_thread) == (DWORD)-1) {
      continue;
    }
    if (GetThreadContext(gw_game_thread, &ctx)) {
      pc = (uintptr_t)ctx.Eip;
    }
    ResumeThread(gw_game_thread);
    if (pc == 0) {
      continue;
    }
    same = (pc == last_pc) ? same + 1 : 0;
    last_pc = pc;
    {
      char where[MAX_PATH + 64];
      gw_describe_code_addr(pc, where, sizeof where);
      /* Only a move to a different function is worth a line; a spin would otherwise produce ten
       * identical lines a second. The point is that the last line before a silent death names
       * where the game was. */
      if (strcmp(where, last_where) != 0) {
        gw_log("gw: at %s", where);
        snprintf(last_where, sizeof last_where, "%s", where);
      } else if (same == 40) {
        gw_log("gw: still at %s after 4s - spinning", where);
      }
      if (++stats_tick >= GW_WATCHDOG_STATS_EVERY) {
        uint32_t retrace, presented, waits, alarms_active, alarms_fired;
        uint32_t copies, prims, dlists;
        stats_tick = 0;
        gw_frame_stats(&retrace, &presented, &waits);
        gw_os_alarm_stats(&alarms_active, &alarms_fired);
        gw_gx_get_stats(&copies, &prims, &dlists);
        gw_log("gw:   retrace=%u presented=%u waitidle=%u  alarms armed=%u fired=%u  "
               "gx copydisp=%u prim=%u dlist=%u",
               retrace, presented, waits, alarms_active, alarms_fired, copies, prims, dlists);
      }
    }  }
}

void gw_start_watchdog(void) {
  if (!DuplicateHandle(GetCurrentProcess(), GetCurrentThread(), GetCurrentProcess(),
                       &gw_game_thread, 0, FALSE, DUPLICATE_SAME_ACCESS)) {
    gw_log("gw: could not start the watchdog (DuplicateHandle failed %lu)", GetLastError());
    return;
  }
  CloseHandle(CreateThread(NULL, 0, &gw_watchdog, NULL, 0, NULL));
}
