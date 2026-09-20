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

/* This is a TOGGLE, not an assignment: it byte-swaps each pointer in place, so calling it twice
 * puts every game global back to its unswapped link-time value. Nothing ever wants that, and it
 * is invisible until something dereferences one - so the second call is refused rather than
 * trusted.
 *
 * It mattered. gw_test_isolate_end restores MEM1 and then called this again on the theory that
 * the restore had undone it. It had not: game globals are not in MEM1 (melee-pc.map puts them at
 * 0x106Fxxxx, in the exe's own data section) and the restore never touches them. So every test
 * flipped all 19781 pointers, and from then on the game globals alternated between correct and
 * byte-swapped with the PARITY OF THE TEST INDEX. Tests that touch no game global never noticed;
 * one that did passed or faulted purely according to how many tests were registered before it,
 * which is how adding an unrelated test to the suite broke a different one. */
static int gw_fixups_applied;

void gw_apply_fixups(void) {
  uint32_t **p = (uint32_t **)(gw_fixups_start + 1);
  uint32_t **end = (uint32_t **)gw_fixups_end;
  size_t count = 0;
  if (gw_fixups_applied) {
    return;
  }
  for (; p < end; ++p) {
    if (*p == NULL) {
      continue; /* alignment padding between object-file contributions */
    }
    **p = gw_bswap32(**p);
    ++count;
  }
  gw_fixups_applied = 1;
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
  char msg[1024];
  va_start(ap, fmt);
  vsnprintf(msg, sizeof msg, fmt, ap);
  va_end(ap);
  {
    /* Under the test runner a panic fails the current test instead of killing the process. */
    extern int gw_test_active(void);
    extern void gw_test_panic_hit(const char *);
    if (gw_test_active()) {
      gw_test_panic_hit(msg);
    }
  }
  fputs("gw: PANIC ", stdout);
  gw_log("%s", msg);
  gw_archive_crash_log(msg);
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

/* Copies the session log to crashlogs/crash-<timestamp>.log before melee-pc.log is next truncated. */
void gw_archive_crash_log(const char *reason) {
  SYSTEMTIME st;
  char dst[MAX_PATH];
  char header[256];
  FILE *in;
  FILE *out;

  GetLocalTime(&st);
  CreateDirectoryA("crashlogs", NULL);
  snprintf(dst, sizeof dst, "crashlogs\\crash-%04u%02u%02u-%02u%02u%02u.log", st.wYear,
           st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond);

  if (gw_log_file != NULL) {
    fflush(gw_log_file);
  }

  out = fopen(dst, "wb");
  if (out == NULL) {
    return;
  }

  {
    int n = snprintf(header, sizeof header, "==== crash %04u-%02u-%02u %02u:%02u:%02u  %s ====\n",
                     st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond,
                     reason != NULL ? reason : "(no reason)");
    if (n > 0) {
      fwrite(header, 1, (size_t)n, out);
    }
  }

  in = fopen("melee-pc.log", "rb");
  if (in != NULL) {
    char buf[4096];
    size_t r;
    while ((r = fread(buf, 1, sizeof buf, in)) > 0) {
      fwrite(buf, 1, r, out);
    }
    fclose(in);
  }
  fclose(out);
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
  gw_archive_crash_log("unhandled exception");
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
  gw_archive_crash_log("CRT invalid parameter");
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

static bool gw_watch_enabled(void) {
  static int state = -1;
  if (state < 0) {
    const char *v = getenv("MELEE_WATCH");
    state = (v != NULL && v[0] != '\0' && v[0] != '0') ? 1 : 0;
  }
  return state != 0;
}

void gw_watch_page(void *addr, size_t size) {
  const uintptr_t page = 0x1000;
  uintptr_t start;
  uintptr_t end;
  DWORD old;
  if (!gw_watch_enabled()) {
    return;
  }
  start = (uintptr_t)addr & ~(page - 1);
  end = ((uintptr_t)addr + size + page - 1) & ~(page - 1);
  gw_watch_addr = (void *)start;
  gw_watch_len = end - start;
  if (VirtualProtect(gw_watch_addr, gw_watch_len, PAGE_READWRITE | PAGE_GUARD, &old)) {
    gw_watch_armed = 1;
  }
}

void gw_watch_tick(void) {
  if (!gw_watch_enabled()) {
    return;
  }
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
/* How long the same PC has to persist before the watchdog calls it a spin. Ten samples a second.
 *
 * Four seconds is right for one game with the machine to itself. It is WRONG when several runs
 * share the CPU: the sweep runs four at a time now, and a stage that loaded fine on its own
 * started reporting "spinning" at frame 240 purely because the load took longer than four
 * seconds of wall clock. That is a false fault, and a false fault in an unattended sweep is
 * worse than no check at all. MELEE_SPIN_SECONDS lets the harness scale it with -Parallel. */
static int gw_spin_ticks(void) {
  static int ticks = -1;
  if (ticks < 0) {
    const char *v = getenv("MELEE_SPIN_SECONDS");
    long secs = (v != NULL) ? strtol(v, NULL, 10) : 0;
    if (secs < 1 || secs > 120) {
      secs = 4;
    }
    ticks = (int)(secs * 10);
  }
  return ticks;
}



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
      } else if (same == gw_spin_ticks()) {
        gw_log("gw: still at %s after %ds - spinning", where, gw_spin_ticks() / 10);
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

/* ---- data-driven Target Test layouts (mods/targettest/<name>.tt) ---------------------------
 * Phase 1: custom target layouts that reuse each character's existing Target Test geometry. A
 * <name>.tt file is read once, lazily, on first query; the game calls gw_TTMod_ForCharacter for
 * the Target Test character and, on a hit, spawns the mod's targets at bare world coordinates.
 *
 * The directory is resolved next to the executable (the same GetModuleFileNameA dance shim_card.c
 * uses for "card/"). Coordinates are stored as native floats and marshalled to big-endian guest
 * memory by gw_TTMod_Target via gw_wf32. No JSON dependency: the format is plain line-based text. */

#define TT_MAX_LEVELS 32
#define TT_MAX_TARGETS 21
#define TT_MAX_PLATFORMS 16
#define TT_NAME_MAX 64

typedef struct {
  char name[TT_NAME_MAX];
  int ckind;
  int target_count;
  float targets[TT_MAX_TARGETS][3];
  /* Phase 2: custom platform geometry (top surface at cy, X extent w, Z
   * extent d, in world/stage units). */
  int platform_count;
  float platforms[TT_MAX_PLATFORMS][5];
} TTLevel;

static TTLevel tt_levels[TT_MAX_LEVELS];
static int tt_level_count;
static int tt_loaded;

static void tt_base_dir(char *path, size_t cap) {
  DWORD n = GetModuleFileNameA(NULL, path, (DWORD)cap);
  if (n > 0 && n < (DWORD)cap) {
    char *slash = strrchr(path, '\\');
    if (slash != NULL) {
      slash[1] = '\0';
      strncat(path, "mods\\targettest", cap - strlen(path) - 1);
    }
  } else {
    strcpy(path, "mods\\targettest");
  }
}

static int tt_ieq(const char *a, const char *b) {
  while (*a != '\0' && *b != '\0') {
    char ca = *a, cb = *b;
    if (ca >= 'A' && ca <= 'Z') ca = (char)(ca + ('a' - 'A'));
    if (cb >= 'A' && cb <= 'Z') cb = (char)(cb + ('a' - 'A'));
    if (ca != cb) return 0;
    a++;
    b++;
  }
  return *a == *b;
}

/* A case-insensitive character NAME -> the decomp's CharacterKind enum (ft/forward.h), or -1.
 * Names are the one character grammar that cannot be read in the wrong index space. */
static int tt_lookup_ckind_name(const char *v) {
  static const struct {
    const char *name;
    int ckind;
  } table[] = {
      {"captain", 0},    {"falcon", 0},   {"donkey", 1},     {"dk", 1},
      {"fox", 2},        {"gamewatch", 3}, {"gw", 3},        {"kirby", 4},
      {"koopa", 5},      {"bowser", 5},   {"link", 6},       {"luigi", 7},
      {"mario", 8},      {"marth", 9},    {"mars", 9},       {"mewtwo", 10},
      {"ness", 11},      {"peach", 12},   {"pikachu", 13},   {"iceclimbers", 14},
      {"popo", 14},      {"nana", 14},    {"jigglypuff", 15}, {"purin", 15},
      {"samus", 16},     {"yoshi", 17},   {"zelda", 18},     {"sheik", 19},
      {"seak", 19},      {"falco", 20},   {"clink", 21},     {"younglink", 21},
      {"drmario", 22},   {"emblem", 23},  {"roy", 23},       {"pichu", 24},
      {"ganon", 25},     {"ganondorf", 25},
  };
  size_t i;
  for (i = 0; i < sizeof table / sizeof table[0]; ++i) {
    if (tt_ieq(v, table[i].name)) return table[i].ckind;
  }
  return -1;
}

/* The legacy "integer or name" grammar, kept for MELEE_TARGET_TEST / MELEE_TRAINING and the
 * .tt mod files: a bare integer here is a CharacterKind. MELEE_SCENE deliberately does NOT
 * accept a bare integer - see the SCENE LAUNCH block at the end of this file for why. */
static int tt_parse_ckind(const char *v) {
  char *end;
  long n = strtol(v, &end, 0);
  if (end != v && *end == '\0') {
    return (int)n;
  }
  return tt_lookup_ckind_name(v);
}

static void tt_parse_file(const char *path, const char *fname) {
  FILE *f = fopen(path, "rb");
  TTLevel lvl;
  char line[512];
  if (f == NULL) return;
  memset(&lvl, 0, sizeof lvl);
  lvl.ckind = -1;
  while (fgets(line, sizeof line, f) != NULL) {
    char *s = line;
    char key[64];
    char *val;
    int k = 0;
    while (*s == ' ' || *s == '\t') s++;
    if (*s == '#' || *s == '\0' || *s == '\n' || *s == '\r') continue;
    while (*s != '\0' && *s != ' ' && *s != '\t' && *s != '\n' && *s != '\r') {
      if (k < (int)sizeof key - 1) key[k++] = *s;
      s++;
    }
    key[k] = '\0';
    while (*s == ' ' || *s == '\t') s++;
    val = s;
    {
      char *e = val + strlen(val);
      while (e > val && (e[-1] == '\n' || e[-1] == '\r' || e[-1] == ' ' || e[-1] == '\t')) {
        *--e = '\0';
      }
    }
    if (strcmp(key, "name") == 0) {
      snprintf(lvl.name, sizeof lvl.name, "%s", val);
    } else if (strcmp(key, "character") == 0) {
      lvl.ckind = tt_parse_ckind(val);
    } else if (strcmp(key, "basestage") == 0) {
      /* Phase 2 (custom geometry): accepted for forward compatibility, ignored here. */
    } else if (strcmp(key, "target") == 0) {
      if (lvl.target_count < TT_MAX_TARGETS) {
        float x, y, z;
        if (sscanf(val, "%f %f %f", &x, &y, &z) == 3) {
          lvl.targets[lvl.target_count][0] = x;
          lvl.targets[lvl.target_count][1] = y;
          lvl.targets[lvl.target_count][2] = z;
          lvl.target_count++;
        }
      }
    } else if (strcmp(key, "platform") == 0) {
      if (lvl.platform_count < TT_MAX_PLATFORMS) {
        float cx, cy, cz, w, d;
        if (sscanf(val, "%f %f %f %f %f", &cx, &cy, &cz, &w, &d) == 5) {
          lvl.platforms[lvl.platform_count][0] = cx;
          lvl.platforms[lvl.platform_count][1] = cy;
          lvl.platforms[lvl.platform_count][2] = cz;
          lvl.platforms[lvl.platform_count][3] = w;
          lvl.platforms[lvl.platform_count][4] = d;
          lvl.platform_count++;
        }
      }
    }
  }
  fclose(f);
  if (lvl.ckind < 0 || lvl.target_count == 0) {
    gw_log("gw: targettest: skipping %s (missing character or targets)", fname);
    return;
  }
  {
    int i;
    for (i = 0; i < tt_level_count; ++i) {
      if (tt_levels[i].ckind == lvl.ckind) {
        gw_log("gw: targettest: ignoring %s: character already claimed by %s", fname,
               tt_levels[i].name);
        return;
      }
    }
  }
  if (tt_level_count >= TT_MAX_LEVELS) {
    gw_log("gw: targettest: level table full, ignoring %s", fname);
    return;
  }
  tt_levels[tt_level_count++] = lvl;
}

static void tt_load(void) {
  char dir[MAX_PATH];
  char pattern[MAX_PATH];
  WIN32_FIND_DATAA fd;
  HANDLE h;
  if (tt_loaded) return;
  tt_loaded = 1;
  tt_base_dir(dir, sizeof dir);
  snprintf(pattern, sizeof pattern, "%s\\*.tt", dir);
  h = FindFirstFileA(pattern, &fd);
  if (h == INVALID_HANDLE_VALUE) {
    gw_log("gw: targettest: no mods found in %s", dir);
    return;
  }
  do {
    char path[MAX_PATH];
    if ((fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0) continue;
    snprintf(path, sizeof path, "%s\\%s", dir, fd.cFileName);
    tt_parse_file(path, fd.cFileName);
  } while (FindNextFileA(h, &fd) != 0);
  FindClose(h);
  gw_log("gw: targettest: loaded %d mods from %s", tt_level_count, dir);
}

int gw_TTMod_Count(void) {
  tt_load();
  return tt_level_count;
}

/* Dev/debug hook: MELEE_TARGET_TEST=<ckind int or name like mario/fox/zelda> boots the game
 * straight into Target Test with that character, skipping menus and the CSS. Game code calls the
 * unprefixed `TestTargetTestCKind` (gwtool maps it to this symbol) and treats a negative return as
 * "not set / not a known character". Read once and cached; returns the CharacterKind (ft/forward.h)
 * or -1. Reuses tt_parse_ckind so the integer/name grammar matches the .tt mod files. */
int gw_TestTargetTestCKind(void) {
  extern int gw_SceneLaunch_TargetTestCKind(void);
  /* Not cached in a static any more: the scene config is the single source of truth, it is
   * itself read once, and a test can replace it without restarting the process. */
  return gw_SceneLaunch_TargetTestCKind();
}

/* MELEE_TRAINING and MELEE_STAGE used to have hooks of their own here. They are now read by
 * the scene config (gw_sl_load_legacy, at the end of this file) and folded into the same
 * seeding path as MELEE_SCENE, so there is exactly one place that decides what boots. */

/* Dev/debug hook: MELEE_CSS_CURSOR_SCALE=0 disables the m-ex CSS cursor scaling. Default on.
 * Game code calls the unprefixed `Mex_CssCursorScaleEnabled`. Read once. */
int gw_Mex_CssCursorScaleEnabled(void) {
  static int cached = -1;
  if (cached < 0) {
    const char *v = getenv("MELEE_CSS_CURSOR_SCALE");
    cached = (v != NULL && v[0] == '0') ? 0 : 1;
    if (!cached) {
      gw_log("gw: MELEE_CSS_CURSOR_SCALE=0 - m-ex CSS cursor scaling disabled");
    }
  }
  return cached;
}

/* Dev/debug hook: MELEE_CONTENT_PROBE=<file.dat> loads that file through the game's own HSD archive
 * loader at boot and logs whether it parsed. This is the m-ex content-pipeline proof of life: the
 * file comes from the disc FST and is parsed by lbArchive_LoadArchive, so a Sonic file loading here
 * means the port consumes m-ex-produced content. Game code calls the unprefixed `ContentProbeName`
 * and `ContentProbeResult`. Read once. */
const char *gw_ContentProbeName(void) {
  static const char *cached;
  static int read;
  if (!read) {
    const char *v = getenv("MELEE_CONTENT_PROBE");
    cached = (v != NULL && v[0] != '\0') ? v : NULL;
    read = 1;
  }
  return cached;
}

/* MELEE_LOG_MOTION=1 turns on the per-action-state trace in Fighter_ChangeMotionState. The
 * game TU asks here rather than calling getenv itself: gwtool renames every game symbol,
 * so a bare getenv there links as gw_getenv and fails. */
int gw_PcTraceMotionEnabled(void) {
  const char *v = getenv("MELEE_LOG_MOTION");
  return (v != NULL && v[0] != '0') ? 1 : 0;
}

void gw_ContentProbeResult(const char *name, void *archive) {
  gw_log("gw: content probe: %s -> %s", name != NULL ? name : "(null)",
         archive != NULL ? "parsed by lbArchive_LoadArchive"
                         : "FAILED (loader returned NULL)");
}

int gw_TTMod_ForCharacter(int ckind) {
  int i;
  tt_load();
  for (i = 0; i < tt_level_count; ++i) {
    if (tt_levels[i].ckind == ckind) return i;
  }
  return -1;
}

int gw_TTMod_TargetCount(int level) {
  tt_load();
  if (level < 0 || level >= tt_level_count) return 0;
  return tt_levels[level].target_count;
}

void gw_TTMod_Target(int level, int i, float *x, float *y, float *z) {
  tt_load();
  if (level < 0 || level >= tt_level_count || i < 0 || i >= tt_levels[level].target_count) {
    if (x != NULL) gw_wf32(x, 0.0f);
    if (y != NULL) gw_wf32(y, 0.0f);
    if (z != NULL) gw_wf32(z, 0.0f);
    return;
  }
  gw_wf32(x, tt_levels[level].targets[i][0]);
  gw_wf32(y, tt_levels[level].targets[i][1]);
  gw_wf32(z, tt_levels[level].targets[i][2]);
}

int gw_TTMod_PlatformCount(int level) {
  tt_load();
  if (level < 0 || level >= tt_level_count) return 0;
  return tt_levels[level].platform_count;
}

void gw_TTMod_Platform(int level, int i, float *cx, float *cy, float *cz, float *w, float *d) {
  tt_load();
  if (level < 0 || level >= tt_level_count || i < 0 || i >= tt_levels[level].platform_count) {
    if (cx != NULL) gw_wf32(cx, 0.0f);
    if (cy != NULL) gw_wf32(cy, 0.0f);
    if (cz != NULL) gw_wf32(cz, 0.0f);
    if (w != NULL) gw_wf32(w, 0.0f);
    if (d != NULL) gw_wf32(d, 0.0f);
    return;
  }
  gw_wf32(cx, tt_levels[level].platforms[i][0]);
  gw_wf32(cy, tt_levels[level].platforms[i][1]);
  gw_wf32(cz, tt_levels[level].platforms[i][2]);
  gw_wf32(w, tt_levels[level].platforms[i][3]);
  gw_wf32(d, tt_levels[level].platforms[i][4]);
}

/* m-ex feature registry. Behaviors ported from akaneia/m-ex (https://github.com/akaneia/m-ex) are
 * each gated on a named feature so a default build behaves exactly as before. Features are enabled
 * by the MELEE_MEX environment variable (comma- or space-separated) and/or a `mods\mex.txt` file
 * beside the executable (one feature per line, `#` starts a comment). Game code calls the
 * unprefixed `Mex_Enabled`, which gwtool maps onto gw_Mex_Enabled. */

#define GW_MEX_MAX_FEATURES 64
#define GW_MEX_NAME_MAX 48

static char gw_mex_features[GW_MEX_MAX_FEATURES][GW_MEX_NAME_MAX];
static int gw_mex_feature_count = -1;

static void gw_mex_add(const char *name, size_t len) {
  size_t i;
  if (len == 0 || len >= GW_MEX_NAME_MAX) return;
  if (gw_mex_feature_count >= GW_MEX_MAX_FEATURES) return;
  for (i = 0; i < len; ++i) {
    char c = name[i];
    gw_mex_features[gw_mex_feature_count][i] =
        (c >= 'A' && c <= 'Z') ? (char)(c + ('a' - 'A')) : c;
  }
  gw_mex_features[gw_mex_feature_count][len] = '\0';
  gw_mex_feature_count++;
}

static void gw_mex_parse_list(const char *s) {
  while (*s != '\0') {
    size_t len = 0;
    while (*s == ',' || *s == ' ' || *s == '\t' || *s == ';') s++;
    while (s[len] != '\0' && s[len] != ',' && s[len] != ' ' && s[len] != '\t' &&
           s[len] != ';' && s[len] != '\n' && s[len] != '\r') {
      len++;
    }
    gw_mex_add(s, len);
    s += len;
  }
}

static void gw_mex_load_file(const char *path) {
  FILE *f = fopen(path, "rb");
  char line[256];
  if (f == NULL) return;
  while (fgets(line, sizeof line, f) != NULL) {
    char *s = line;
    char *end;
    while (*s == ' ' || *s == '\t') s++;
    if (*s == '#' || *s == '\n' || *s == '\r' || *s == '\0') continue;
    end = s + strlen(s);
    while (end > s && (end[-1] == '\n' || end[-1] == '\r' || end[-1] == ' ' || end[-1] == '\t')) {
      end--;
    }
    *end = '\0';
    gw_mex_add(s, strlen(s));
  }
  fclose(f);
}

static void gw_mex_demo_register(void);

static void gw_mex_load(void) {
  char path[MAX_PATH];
  const char *env;
  DWORD n;
  if (gw_mex_feature_count >= 0) return;
  gw_mex_feature_count = 0;
  env = getenv("MELEE_MEX");
  if (env != NULL) {
    gw_mex_parse_list(env);
  }
  n = GetModuleFileNameA(NULL, path, (DWORD)sizeof path);
  if (n > 0 && n < (DWORD)sizeof path) {
    char *slash = strrchr(path, '\\');
    if (slash != NULL) {
      slash[1] = '\0';
      strncat(path, "mods\\mex.txt", sizeof path - strlen(path) - 1);
      gw_mex_load_file(path);
    }
  }
  if (gw_mex_feature_count > 0) {
    int i;
    gw_log("gw: mex: %d feature(s) enabled", gw_mex_feature_count);
    for (i = 0; i < gw_mex_feature_count; ++i) {
      gw_log("gw: mex:   %s", gw_mex_features[i]);
    }
  }
  gw_mex_demo_register();
}

int gw_Mex_Enabled(const char *name) {
  int i;
  if (name == NULL) {
    return 0;
  }
  gw_mex_load();
  for (i = 0; i < gw_mex_feature_count; ++i) {
    if (tt_ieq(name, gw_mex_features[i])) return 1;
  }
  return 0;
}

/* ---- m-ex Tier C fighter hook surface (native re-expression) ----------------------------
 * Ported from m-ex (https://github.com/akaneia/m-ex): the "Fighter On*" patches each replace one
 * `addi` computing a per-character callback-table base, and the decomp already calls
 * `ftData_<X>[fp->kind](gobj)`. A hook is a table-slot override: a flat per-(event,kind) array of
 * native function pointers, all NULL by default. NULL means "no override, run vanilla", so
 * clearing a slot restores vanilla behaviour; there is no chain (last registration wins), matching
 * m-ex. See _research/mex-tier-c-hooks.md. */

static gwmex_gobj_fn gw_mex_gobj_hooks[GW_MEX_EVENT_COUNT][GW_MEX_KIND_MAX];
static gwmex_gobj_fn2 gw_mex_gobj_hooks2[GW_MEX_EVENT_COUNT][GW_MEX_KIND_MAX];
typedef void (*gwmex_gobj_fn3)(void*, void*, void*);
static gwmex_gobj_fn3 gw_mex_gobj_hooks3[GW_MEX_EVENT_COUNT][GW_MEX_KIND_MAX];
static gwmex_gobj_pred gw_mex_pred_hooks[GW_MEX_EVENT_COUNT][GW_MEX_KIND_MAX];

/* Hook re-entry. An m-ex override is written as a REPLACEMENT for the engine function it hooks,
 * and it commonly performs the real work by calling that same engine function - Sonic's
 * onDoubleJump calls ftCo_800CBAC4, the very function whose dispatch site invoked it. Dispatching
 * the hook again there is an infinite loop (hook -> guest -> native -> hook -> ...), which for an
 * interpreted guest override burns the native stack and dies as 0xC00000FD at gw_ppc_call+0x3.
 *
 * So a hook already on the stack for this (event, kind) is skipped and the vanilla path runs
 * instead: the guest's call into the engine function gets the engine function's real behaviour,
 * which is exactly what the override asked for. This is the standard detour/trampoline rule and
 * it costs nothing when no hook is active. Nesting is never legitimate here - the engine drives
 * these events one fighter at a time, not recursively. */
static unsigned char gw_mex_hook_active[GW_MEX_EVENT_COUNT][GW_MEX_KIND_MAX];

int gw_Mex_HookRegister(int event, int kind, gwmex_gobj_fn fn) {
  if ((unsigned)event >= GW_MEX_EVENT_COUNT || (unsigned)kind >= GW_MEX_KIND_MAX) {
    return 0;
  }
  gw_mex_gobj_hooks[event][kind] = fn;
  return 1;
}

int gw_Mex_HookRegister2(int event, int kind, gwmex_gobj_fn2 fn) {
  if ((unsigned)event >= GW_MEX_EVENT_COUNT || (unsigned)kind >= GW_MEX_KIND_MAX) {
    return 0;
  }
  gw_mex_gobj_hooks2[event][kind] = fn;
  return 1;
}

int gw_Mex_HookRegister3(int event, int kind, void (*fn)(void*, void*, void*)) {
  if ((unsigned)event >= GW_MEX_EVENT_COUNT || (unsigned)kind >= GW_MEX_KIND_MAX) {
    return 0;
  }
  gw_mex_gobj_hooks3[event][kind] = (gwmex_gobj_fn3)fn;
  return 1;
}

int gw_Mex_HasHook(int event, int kind) {
  if ((unsigned)event >= GW_MEX_EVENT_COUNT || (unsigned)kind >= GW_MEX_KIND_MAX) {
    return 0;
  }
  return gw_mex_gobj_hooks[event][kind] != NULL ||
         gw_mex_gobj_hooks2[event][kind] != NULL ||
         gw_mex_gobj_hooks3[event][kind] != NULL;
}

int gw_Mex_PredicateRegister(int event, int kind, gwmex_gobj_pred fn) {
  if ((unsigned)event >= GW_MEX_EVENT_COUNT || (unsigned)kind >= GW_MEX_KIND_MAX) {
    return 0;
  }
  gw_mex_pred_hooks[event][kind] = fn;
  return 1;
}

/* The m-ex runtime serves several fighters; a hook runs with its fighter selected
 * (gw_mex_ftfunction_runtime.c), restored afterwards - hooks nest across fighters. */
extern void *gw_Mex_SelectKind(int kind);
extern void gw_Mex_RestoreKind(void *prev);

void gw_Mex_GObjDispatch(int event, int kind, void *gobj, void *vanilla) {
  gwmex_gobj_fn fn = NULL;
  int in_range = ((unsigned)event < GW_MEX_EVENT_COUNT && (unsigned)kind < GW_MEX_KIND_MAX);
  if (in_range && !gw_mex_hook_active[event][kind]) {
    fn = gw_mex_gobj_hooks[event][kind];
  }
  if (fn != NULL) {
    void *prev = gw_Mex_SelectKind(kind);
    gw_mex_hook_active[event][kind] = 1;
    fn(gobj);
    gw_mex_hook_active[event][kind] = 0;
    gw_Mex_RestoreKind(prev);
  } else if (vanilla != NULL) {
    ((gwmex_gobj_fn)vanilla)(gobj);
  }
}

void gw_Mex_GObjDispatch2(int event, int kind, void *gobj, void *arg1, void *vanilla) {
  gwmex_gobj_fn2 fn = NULL;
  int in_range = ((unsigned)event < GW_MEX_EVENT_COUNT && (unsigned)kind < GW_MEX_KIND_MAX);
  if (in_range && !gw_mex_hook_active[event][kind]) {
    fn = gw_mex_gobj_hooks2[event][kind];
  }
  if (fn != NULL) {
    void *prev = gw_Mex_SelectKind(kind);
    gw_mex_hook_active[event][kind] = 1;
    fn(gobj, arg1);
    gw_mex_hook_active[event][kind] = 0;
    gw_Mex_RestoreKind(prev);
  } else if (vanilla != NULL) {
    ((gwmex_gobj_fn2)vanilla)(gobj, arg1);
  }
}

/* Three-argument dispatch: onModelRender's table entry is called as (gobj, flag_index, mtx). */
void gw_Mex_GObjDispatch3(int event, int kind, void *gobj, void *arg1, void *arg2,
                          void *vanilla) {
  gwmex_gobj_fn3 fn = NULL;
  int in_range = ((unsigned)event < GW_MEX_EVENT_COUNT && (unsigned)kind < GW_MEX_KIND_MAX);
  if (in_range && !gw_mex_hook_active[event][kind]) {
    fn = gw_mex_gobj_hooks3[event][kind];
  }
  if (fn != NULL) {
    void *prev = gw_Mex_SelectKind(kind);
    gw_mex_hook_active[event][kind] = 1;
    fn(gobj, arg1, arg2);
    gw_mex_hook_active[event][kind] = 0;
    gw_Mex_RestoreKind(prev);
  } else if (vanilla != NULL) {
    ((gwmex_gobj_fn3)vanilla)(gobj, arg1, arg2);
  }
}

int gw_Mex_GObjPredDispatch(int event, int kind, void *gobj, void *vanilla) {
  gwmex_gobj_pred fn = NULL;
  int in_range = ((unsigned)event < GW_MEX_EVENT_COUNT && (unsigned)kind < GW_MEX_KIND_MAX);
  if (in_range && !gw_mex_hook_active[event][kind]) {
    fn = gw_mex_pred_hooks[event][kind];
  }
  if (fn != NULL) {
    int r;
    void *prev = gw_Mex_SelectKind(kind);
    gw_mex_hook_active[event][kind] = 1;
    r = fn(gobj);
    gw_mex_hook_active[event][kind] = 0;
    gw_Mex_RestoreKind(prev);
    return r;
  }
  if (vanilla != NULL) {
    return ((gwmex_gobj_pred)vanilla)(gobj);
  }
  return 0;
}

void gw_Mex_OnLoadDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_LOAD, kind, gobj, vanilla);
}
void gw_Mex_OnDeathDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_DEATH, kind, gobj, vanilla);
}
void gw_Mex_OnDestroyDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_DESTROY, kind, gobj, vanilla);
}
void gw_Mex_OnFrameDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_FRAME, kind, gobj, vanilla);
}
void gw_Mex_OnAbsorbDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_ABSORB, kind, gobj, vanilla);
}
void gw_Mex_OnApplyHeadItemDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_APPLY_HEAD_ITEM, kind, gobj, vanilla);
}
void gw_Mex_OnRemoveHeadItemDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_REMOVE_HEAD_ITEM, kind, gobj, vanilla);
}
void gw_Mex_OnItemInvisibleDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_ITEM_INVISIBLE, kind, gobj, vanilla);
}
void gw_Mex_OnItemVisibleDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_ITEM_VISIBLE, kind, gobj, vanilla);
}
void gw_Mex_OnKnockbackEnterDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_KNOCKBACK_ENTER, kind, gobj, vanilla);
}
void gw_Mex_OnKnockbackExitDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_KNOCKBACK_EXIT, kind, gobj, vanilla);
}
void gw_Mex_OnActionStateChangeDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_ACTION_STATE_CHANGE, kind, gobj, vanilla);
}
void gw_Mex_OnReapplyAttrDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_REAPPLY_ATTR, kind, gobj, vanilla);
}
void gw_Mex_SpecialNDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_N, kind, gobj, vanilla);
}
void gw_Mex_SpecialNAirDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_N_AIR, kind, gobj, vanilla);
}
void gw_Mex_SpecialSDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_S, kind, gobj, vanilla);
}
void gw_Mex_SpecialSAirDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_S_AIR, kind, gobj, vanilla);
}
void gw_Mex_SpecialHiDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_HI, kind, gobj, vanilla);
}
void gw_Mex_SpecialHiAirDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_HI_AIR, kind, gobj, vanilla);
}
void gw_Mex_SpecialLwDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_LW, kind, gobj, vanilla);
}
void gw_Mex_SpecialLwAirDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_SPECIAL_LW_AIR, kind, gobj, vanilla);
}
void gw_Mex_MoveLogicDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_MOVE_LOGIC, kind, gobj, vanilla);
}
void gw_Mex_OnDoubleJumpDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_DOUBLE_JUMP, kind, gobj, vanilla);
}
/* ---- m-ex Category 2 ------------------------------------------------------------------
 * These have no vanilla per-kind table, so every call site passes vanilla = NULL: the hook is an
 * addition to what the engine already does, not a replacement for a table entry. The two sites
 * where m-ex's injected code DOES skip vanilla work ask gw_Mex_HasHook first and do the skipping
 * themselves, in the decomp, where it is readable. */
void gw_Mex_OnModelRenderDispatch(int kind, void *gobj, void *arg1, void *mtx, void *vanilla) {
  gw_Mex_GObjDispatch3(GW_MEX_EVENT_ON_MODEL_RENDER, kind, gobj, arg1, mtx, vanilla);
}
int gw_Mex_HasZairHook(int kind) { return gw_Mex_HasHook(GW_MEX_EVENT_ON_ZAIR, kind); }
int gw_Mex_HasFSmashHook(int kind) { return gw_Mex_HasHook(GW_MEX_EVENT_ON_FSMASH, kind); }

void gw_Mex_OnZairDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_ZAIR, kind, gobj, vanilla);
}
void gw_Mex_OnLandingDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_LANDING, kind, gobj, vanilla);
}
void gw_Mex_OnFSmashDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_FSMASH, kind, gobj, vanilla);
}
void gw_Mex_OnIntroLDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_INTRO_L, kind, gobj, vanilla);
}
void gw_Mex_OnTauntDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_TAUNT, kind, gobj, vanilla);
}
void gw_Mex_OnCatchDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_CATCH, kind, gobj, vanilla);
}

void gw_Mex_OnUSmashDispatch(int kind, void *gobj, void *vanilla) {
  gw_Mex_GObjDispatch(GW_MEX_EVENT_ON_USMASH, kind, gobj, vanilla);
}
void gw_Mex_OnItemPickupDispatch(int kind, void *gobj, void *arg1, void *vanilla) {
  gw_Mex_GObjDispatch2(GW_MEX_EVENT_ON_ITEM_PICKUP, kind, gobj, arg1, vanilla);
}

/* The three item slots m-ex names OnItemRelease / OnItemCatch / onUnknownItemRelated (16/17/18).
 * Each replaces one vanilla per-kind table whose dispatch site is already in ftcommon.c, and each
 * takes the fighter gobj plus that call site's s32 argument. Leaving them unregistered is not
 * inert: ftdata.c's clone-base fallback then runs the BASE fighter's handler instead. */
void gw_Mex_OnItemDropExtDispatch(int kind, void *gobj, void *arg1, void *vanilla) {
  gw_Mex_GObjDispatch2(GW_MEX_EVENT_ON_ITEM_DROP_EXT, kind, gobj, arg1, vanilla);
}
void gw_Mex_OnItemPickup2Dispatch(int kind, void *gobj, void *arg1, void *vanilla) {
  gw_Mex_GObjDispatch2(GW_MEX_EVENT_ON_ITEM_PICKUP2, kind, gobj, arg1, vanilla);
}
void gw_Mex_OnItemDropDispatch(int kind, void *gobj, void *arg1, void *vanilla) {
  gw_Mex_GObjDispatch2(GW_MEX_EVENT_ON_ITEM_DROP, kind, gobj, arg1, vanilla);
}

/* Demo registration for OnFrame: proves the surface fires without a custom mod. Installed once,
 * from gw_mex_load(), so it rides the existing "read flags once" path. The hook logs only its
 * first invocation to avoid a per-fighter per-frame flood. */
static void gw_mex_demo_onframe(void *gobj) {
  static int logged;
  (void)gobj;
  if (!logged) {
    logged = 1;
    gw_log("mex: OnFrame demo hook fired (gobj=%p)", gobj);
  }
}

static void gw_mex_demo_register(void) {
  int k;
  if (!gw_Mex_Enabled("mex_onframe")) {
    return;
  }
  for (k = 0; k < GW_MEX_KIND_MAX; ++k) {
    gw_Mex_HookRegister(GW_MEX_EVENT_ON_FRAME, k, gw_mex_demo_onframe);
  }
}

/* Game code reads a debug switch through this (gwtool prefixes every game symbol, so game code
 * cannot call the CRT's getenv): 1 when environment variable `name` starts with '1'. */
int gw_Env1(const char *name) {
  const char *v = getenv(name);
  return v != NULL && v[0] == '1';
}

/* ================================================================================================
 * SCENE LAUNCH -- boot straight into any screen, in any configuration
 * ================================================================================================
 *
 * The full grammar lives in _research/scene-launch.md. The short version:
 *
 *   MELEE_SCENE="mode=training;at=match;p1=fox;p2=ck:38/cpu9;stage=ext:293"
 *   MELEE_SCENE_FILE=<path>     the same text, one `key value` or `key=value` per line
 *
 * THE RULE THAT MATTERS: a number in a character or stage field MUST name its index space.
 * There are four fighter index spaces in this port and two stage ones, and a bare integer is
 * right in one of them and silently wrong in the others. `p1=37` is therefore REJECTED; write
 * `p1=ck:37` (port CharacterKind, which is Lucas) or `p1=fk:37` (port FighterKind, which is
 * Sonic) and say what you meant. Names (`p1=fox`) are always unambiguous and always accepted.
 *
 *   ck:N      port CharacterKind   -- what PlayerInitData::ckind holds. m-ex slot i = 0x22 + i.
 *   fk:N      port FighterKind     -- what Fighter::kind holds.        m-ex slot i = 0x21 + i.
 *   mex:N     m-ex INTERNAL id     -- MxDt.dat's own numbering (Akaneia's added seven are 27..33).
 *   mexext:N  m-ex EXTERNAL id     -- MxDt.dat's CSS/ui numbering (Akaneia's added seven 26..32).
 *   ext:N     external StKind      -- what StartMeleeRules::stkind holds. Meta Crystal = 293.
 *   int:N     internal GrKind      -- the port's own stage id.          Meta Crystal = 76.
 *
 * The legacy single-purpose variables MELEE_TRAINING / MELEE_TARGET_TEST / MELEE_STAGE still
 * work and are translated into an equivalent scene config, so there is exactly one seeding path.
 * They keep their historical "a bare integer is a CharacterKind / an external StKind" grammar,
 * and now say so in the log.
 *
 * Parsing is a pure function of a string (gw_sl_parse), which is what makes it testable: the
 * environment is read once into gw_sl_cfg, and a test can build any other config without
 * touching the process environment. gw_SceneLaunch_LoadForTest() is that entry point.
 */

#define GW_SL_SLOTS 4

/* Gm_PKind (src/melee/pl/forward.h). */
#define GW_SL_PK_HUMAN 0
#define GW_SL_PK_CPU 1
#define GW_SL_PK_DEMO 2
#define GW_SL_PK_NA 3

#define GW_SL_CK_NONE 0x21 /* ChKind_None */
#define GW_SL_CK_MEX0 0x22 /* ChKind_Mex0 */
#define GW_SL_FK_MEX0 0x21 /* Ft_Kind_Mex0 */
#define GW_SL_CK_PLAYABLE 26 /* CKind_Playable_Count */

/* Entry state ids. Both GM_VS and GM_TRAINING number their states CSS 0, SSS 1, match 2. */
#define GW_SL_AT_CSS 0
#define GW_SL_AT_SSS 1
#define GW_SL_AT_MATCH 2

typedef struct {
  int ckind;     /* port CharacterKind, or -1 for "slot not configured" */
  int random;    /* pick a random playable CharacterKind when the scene is seeded */
  int slot_type; /* Gm_PKind, or -1 to let the seeder choose */
  int color;
  int cpu_kind;
  int cpu_level;
  int handicap;
  int team;
  int stocks;
  int nametag;
} GwSlPlayer;

typedef struct {
  int configured;   /* a scene was requested at all */
  int game_mode;    /* GameModeKind to boot into, or -1 */
  int vs_mode;      /* GmVsMode index into gmMainLib_804D3EE0->modes.table, or -1 */
  int entry_state;  /* GW_SL_AT_*, or -1 to leave the state machine alone */
  int stage_ext;    /* external StKind, or -1 */
  int tt_ckind;     /* Target Test character (CharacterKind), or -1 */
  int skip_memcard; /* -1 = auto (skip whenever a scene is configured) */
  int teams;        /* -1 = leave alone */
  int time_limit;   /* seconds, -1 = leave alone */
  int item_freq;    /* -1 = leave alone */
  int errors;       /* count of rejected fields; a config with errors is still used */
  GwSlPlayer p[GW_SL_SLOTS];
} GwSceneConfig;

static GwSceneConfig gw_sl_cfg;
static int gw_sl_loaded;

/* CharacterKind -> FighterKind for the retail cast, transcribed from ftMapping_list
 * (src/melee/pl/player.c). The two orders are DIFFERENT permutations - CKind_Fox is 2 and
 * Ft_Kind_Fox is 1 - which is why this table exists at all and why the test
 * scene_ckind_fkind_table checks every row against the game's own ftMapping_list. */
static const signed char gw_sl_ck_to_fk[GW_SL_CK_NONE] = {
    /* 00 Captain   */ 2,  /* 01 Donkey    */ 3,  /* 02 Fox       */ 1,
    /* 03 GameWatch */ 24, /* 04 Kirby     */ 4,  /* 05 Koopa     */ 5,
    /* 06 Link      */ 6,  /* 07 Luigi     */ 17, /* 08 Mario     */ 0,
    /* 09 Mars      */ 18, /* 0A Mewtwo    */ 16, /* 0B Ness      */ 8,
    /* 0C Peach     */ 9,  /* 0D Pikachu   */ 12, /* 0E PopoNana  */ 10,
    /* 0F Purin     */ 15, /* 10 Samus     */ 13, /* 11 Yoshi     */ 14,
    /* 12 Zelda     */ 19, /* 13 Seak      */ 7,  /* 14 Falco     */ 22,
    /* 15 CLink     */ 20, /* 16 DrMario   */ 21, /* 17 Emblem    */ 26,
    /* 18 Pichu     */ 23, /* 19 Ganon     */ 25, /* 1A MasterH   */ 27,
    /* 1B Boy       */ 29, /* 1C Girl      */ 30, /* 1D GKoops    */ 31,
    /* 1E CrezyH    */ 28, /* 1F Sandbag   */ 32, /* 20 Popo      */ 10,
};

/* Retail stage names -> external StKind (gr/forward.h). Aliases first-come-first-served. */
static const struct {
  const char *name;
  int stkind;
} gw_sl_stage_names[] = {
    {"izumi", 2},        {"fountain", 2},   {"fod", 2},        {"pstadium", 3},
    {"stadium", 3},      {"ps", 3},         {"castle", 4},     {"kongo", 5},
    {"jungle", 5},       {"zebes", 6},      {"brinstar", 6},   {"corneria", 7},
    {"story", 8},        {"yoshistory", 8}, {"ys", 8},         {"onett", 9},
    {"mutecity", 10},    {"rcruise", 11},   {"garden", 12},    {"kingdom2", 12},
    {"greatbay", 13},    {"shrine", 14},    {"temple", 14},    {"kraid", 15},
    {"yoster", 16},      {"yoshiisland", 16}, {"greens", 17},  {"dreamland", 17},
    {"dl", 17},          {"fourside", 18},  {"inishie1", 19},  {"inishie2", 20},
    {"akaneia", 21},     {"venom", 22},     {"pura", 23},      {"pokefloats", 23},
    {"bigblue", 24},     {"icemt", 25},     {"icetop", 26},    {"flatzone", 27},
    {"oldpupupu", 28},   {"dreamland64", 28}, {"oldyoshi", 29}, {"oldkongo", 30},
    {"battle", 31},      {"battlefield", 31}, {"bf", 31},      {"final", 32},
    {"fd", 32},          {"finaldestination", 32},
};

/* Scene "mode" keywords -> {GameModeKind, GmVsMode}. GmVsMode is the index of the VsModeData row
 * the mode seeds from; -1 means the mode has no VsModeData of its own. */
static const struct {
  const char *name;
  int game_mode; /* GameModeKind */
  int vs_mode;   /* GmVsMode, or -1 */
} gw_sl_modes[] = {
    {"training", 0x1C, 6},  /* GM_TRAINING, GmVsMode_Training */
    {"vs", 0x02, 0},        /* GM_VS,       GmVsMode_Melee */
    {"melee", 0x02, 0},
    {"targettest", 0x0F, -1}, /* GM_TARGET_TEST */
    {"tt", 0x0F, -1},
    {"title", 0x00, -1},    /* GM_TITLE */
    {"menu", 0x01, -1},     /* GM_MENU */
    {"tiny", 0x1D, 7},      /* GM_TINY_VS,    GmVsMode_Tiny */
    {"giant", 0x1E, 8},     /* GM_GIANT_VS,   GmVsMode_Giant */
    {"stamina", 0x1F, 9},   /* GM_STAMINA_VS, GmVsMode_Stamina */
    {"camera", 0x2A, 3},    /* GM_CAMERA_VS,  GmVsMode_Camera */
    {"ssd", 0x10, 1},       /* GM_SUPER_SUDDEN_DEATH_VS */
    {"invisible", 0x11, 2}, /* GM_INVISIBLE_VS */
    {"slomo", 0x12, 10},    /* GM_SLOMO_VS */
    {"lightning", 0x13, 11},/* GM_LIGHTNING_VS */
    /* The three trophy modes. None has a VsModeData row, so they seed nothing and simply boot:
     * the gallery (view a trophy in 3D), the lottery (coins -> trophies) and the collection
     * (the grid of everything unlocked). GS_TOY_* run tylist/toy/tyfigupon/tydisplay. */
    {"trophygallery", 0x0B, -1},    /* GM_TOY_GALLERY */
    {"tygallery", 0x0B, -1},
    {"trophylottery", 0x0C, -1},    /* GM_TOY_LOTTERY */
    {"tylottery", 0x0C, -1},
    {"lottery", 0x0C, -1},
    {"trophycollection", 0x0D, -1}, /* GM_TOY_COLLECTION */
    {"tycollection", 0x0D, -1},
    {"trophies", 0x0D, -1},
};

static void gw_sl_player_init(GwSlPlayer *p) {
  p->ckind = -1;
  p->random = 0;
  p->slot_type = -1;
  p->color = -1;
  p->cpu_kind = -1;
  p->cpu_level = -1;
  p->handicap = -1;
  p->team = -1;
  p->stocks = -1;
  p->nametag = -1;
}

static void gw_sl_config_init(GwSceneConfig *c) {
  int i;
  memset(c, 0, sizeof *c);
  c->game_mode = -1;
  c->vs_mode = -1;
  c->entry_state = -1;
  c->stage_ext = -1;
  c->tt_ckind = -1;
  c->skip_memcard = -1;
  c->teams = -1;
  c->time_limit = -1;
  c->item_freq = -1;
  for (i = 0; i < GW_SL_SLOTS; ++i) {
    gw_sl_player_init(&c->p[i]);
  }
}

/* Case-insensitive prefix test; returns the rest of `s` past `prefix`, or NULL. */
static const char *gw_sl_after(const char *s, const char *prefix) {
  size_t i;
  for (i = 0; prefix[i] != '\0'; ++i) {
    char a = s[i], b = prefix[i];
    if (a >= 'A' && a <= 'Z') a = (char)(a + ('a' - 'A'));
    if (a != b) return NULL;
  }
  return s + i;
}

static int gw_sl_all_digits(const char *s) {
  if (*s == '-' || *s == '+') s++;
  if (*s == '\0') return 0;
  for (; *s != '\0'; ++s) {
    if (*s < '0' || *s > '9') return 0;
  }
  return 1;
}

/* Port FighterKind -> port CharacterKind. Retail kinds go through the inverse of
 * gw_sl_ck_to_fk; m-ex slot i is Ft_Kind_Mex0 + i on one side and ChKind_Mex0 + i on the
 * other, so the conversion there is +1 - and THAT is the whole "Sonic is 37 or 38" confusion:
 * he is FighterKind 37 and CharacterKind 38, and both statements are true. */
int gw_SceneLaunch_FKindToCKind(int fk) {
  int i;
  if (fk >= GW_SL_FK_MEX0 && fk < GW_SL_FK_MEX0 + 31) {
    return GW_SL_CK_MEX0 + (fk - GW_SL_FK_MEX0);
  }
  for (i = 0; i < GW_SL_CK_NONE; ++i) {
    /* Popo appears twice (CKind_PopoNana and ChKind_Popo); the first, playable row wins. */
    if (gw_sl_ck_to_fk[i] == fk) return i;
  }
  return -1;
}

int gw_SceneLaunch_CKindToFKind(int ck) {
  if (ck >= GW_SL_CK_MEX0 && ck < GW_SL_CK_MEX0 + 31) {
    return GW_SL_FK_MEX0 + (ck - GW_SL_CK_MEX0);
  }
  return (ck >= 0 && ck < GW_SL_CK_NONE) ? gw_sl_ck_to_fk[ck] : -1;
}

/* m-ex INTERNAL id -> port CharacterKind, via the fighter runtime's slot table. */
static int gw_sl_mexint_to_ck(int internal) {
  extern int gw_Mex_PortKindForInternal(int k);
  int fk = gw_Mex_PortKindForInternal(internal);
  return fk >= 0 ? gw_SceneLaunch_FKindToCKind(fk) : -1;
}

/* Parses a character reference into a CharacterKind. Returns 0 on success.
 * `*random` is set for the "random" keyword, which is resolved when the scene is seeded. */
static int gw_sl_parse_char(const char *v, int *ck_out, int *random_out) {
  extern int gw_Mex_ExtToPortCKind(int ext);
  const char *rest;
  int n;
  *random_out = 0;
  if (gw_sl_after(v, "random") != NULL && v[6] == '\0') {
    *ck_out = -1;
    *random_out = 1;
    return 0;
  }
  if (gw_sl_after(v, "none") != NULL && v[4] == '\0') {
    *ck_out = GW_SL_CK_NONE;
    return 0;
  }
  if ((rest = gw_sl_after(v, "ck:")) != NULL && gw_sl_all_digits(rest)) {
    *ck_out = atoi(rest);
    return 0;
  }
  if ((rest = gw_sl_after(v, "fk:")) != NULL && gw_sl_all_digits(rest)) {
    n = gw_SceneLaunch_FKindToCKind(atoi(rest));
    if (n < 0) return -1;
    *ck_out = n;
    return 0;
  }
  if ((rest = gw_sl_after(v, "mexext:")) != NULL && gw_sl_all_digits(rest)) {
    n = gw_Mex_ExtToPortCKind(atoi(rest));
    if (n < 0) return -1;
    *ck_out = n;
    return 0;
  }
  if ((rest = gw_sl_after(v, "mexint:")) == NULL) {
    rest = gw_sl_after(v, "mex:");
  }
  if (rest != NULL && gw_sl_all_digits(rest)) {
    n = gw_sl_mexint_to_ck(atoi(rest));
    if (n < 0) return -1;
    *ck_out = n;
    return 0;
  }
  n = tt_lookup_ckind_name(v);
  if (n >= 0) {
    *ck_out = n;
    return 0;
  }
  return -1;
}

/* Parses a stage reference into an external StKind. Returns 0 on success. */
static int gw_sl_parse_stage(const char *v, int *ext_out) {
  extern int gw_Mex_GrKindForExt(int ext);
  extern int gw_Mex_GrExternalCount(void);
  const char *rest;
  size_t i;
  if ((rest = gw_sl_after(v, "ext:")) != NULL && gw_sl_all_digits(rest)) {
    *ext_out = atoi(rest);
    return 0;
  }
  if ((rest = gw_sl_after(v, "int:")) != NULL && gw_sl_all_digits(rest)) {
    /* Only mexData knows the internal <-> external stage map, and it exposes only the forward
     * direction, so walk it. Without mexData (a vanilla disc) there is no map at all and the two
     * spaces coincide for the retail stages, which is why the fallback is the identity. */
    int want = atoi(rest), e, n = gw_Mex_GrExternalCount();
    for (e = 0; e < n; ++e) {
      if (gw_Mex_GrKindForExt(e) == want) {
        *ext_out = e;
        return 0;
      }
    }
    *ext_out = want;
    return 0;
  }
  for (i = 0; i < sizeof gw_sl_stage_names / sizeof gw_sl_stage_names[0]; ++i) {
    if (tt_ieq(v, gw_sl_stage_names[i].name)) {
      *ext_out = gw_sl_stage_names[i].stkind;
      return 0;
    }
  }
  return -1;
}

/* One `pN=` value: `<charref>[/<opt>]...`. Returns 0 on success. */
static int gw_sl_parse_player(const char *v, GwSlPlayer *p) {
  char buf[128];
  char *tok, *next;
  size_t n = strlen(v);
  if (n >= sizeof buf) return -1;
  memcpy(buf, v, n + 1);
  tok = buf;
  next = strchr(tok, '/');
  if (next != NULL) *next++ = '\0';
  if (gw_sl_parse_char(tok, &p->ckind, &p->random) != 0) return -1;
  while (next != NULL) {
    const char *rest;
    tok = next;
    next = strchr(tok, '/');
    if (next != NULL) *next++ = '\0';
    if ((rest = gw_sl_after(tok, "c")) != NULL && gw_sl_all_digits(rest)) {
      /* A costume id has a hard ceiling that does not depend on the disc: every per-costume
       * runtime array the port rebuilds for an m-ex disc has sixteen rows, and
       * ftData_MexInitKinds clamps a fighter's count to that. Refuse anything above it HERE -
       * a bad costume used to travel all the way into the animation path and fault there
       * (ftAnim_80070200), which is a miserable way to learn you typed c9 for a six-costume
       * fighter. The per-fighter count is not known at parse time (it comes from MxDt.dat,
       * which is not mounted yet); ftData_80085820 reports that one. */
      int c = atoi(rest);
      if (c < 0 || c >= 16) {
        gw_log("gw: scene: rejected \"%s\" -- a costume id must be 0..15", tok);
        return -1;
      }
      p->color = c;
    } else if ((rest = gw_sl_after(tok, "cpu")) != NULL && gw_sl_all_digits(rest)) {
      p->slot_type = GW_SL_PK_CPU;
      p->cpu_level = atoi(rest);
    } else if (tt_ieq(tok, "cpu")) {
      p->slot_type = GW_SL_PK_CPU;
    } else if (tt_ieq(tok, "human") || tt_ieq(tok, "hu")) {
      p->slot_type = GW_SL_PK_HUMAN;
    } else if (tt_ieq(tok, "demo") || tt_ieq(tok, "dummy")) {
      p->slot_type = GW_SL_PK_DEMO;
    } else if (tt_ieq(tok, "off") || tt_ieq(tok, "na")) {
      p->slot_type = GW_SL_PK_NA;
    } else if ((rest = gw_sl_after(tok, "hmp")) != NULL && gw_sl_all_digits(rest)) {
      p->handicap = atoi(rest);
    } else if ((rest = gw_sl_after(tok, "team")) != NULL && gw_sl_all_digits(rest)) {
      p->team = atoi(rest);
    } else if ((rest = gw_sl_after(tok, "stocks")) != NULL && gw_sl_all_digits(rest)) {
      p->stocks = atoi(rest);
    } else if ((rest = gw_sl_after(tok, "kind")) != NULL && gw_sl_all_digits(rest)) {
      p->cpu_kind = atoi(rest);
    } else if ((rest = gw_sl_after(tok, "nametag")) != NULL && gw_sl_all_digits(rest)) {
      p->nametag = atoi(rest);
    } else {
      return -1;
    }
  }
  return 0;
}

/* One `key=value` (or `key value`) pair. Returns 0 on success, -1 when the field is rejected. */
static int gw_sl_apply(GwSceneConfig *c, const char *key, const char *val) {
  size_t i;
  if (tt_ieq(key, "mode")) {
    for (i = 0; i < sizeof gw_sl_modes / sizeof gw_sl_modes[0]; ++i) {
      if (tt_ieq(val, gw_sl_modes[i].name)) {
        c->game_mode = gw_sl_modes[i].game_mode;
        c->vs_mode = gw_sl_modes[i].vs_mode;
        c->configured = 1;
        return 0;
      }
    }
    return -1;
  }
  if (tt_ieq(key, "at") || tt_ieq(key, "screen")) {
    if (tt_ieq(val, "css") || tt_ieq(val, "chars")) {
      c->entry_state = GW_SL_AT_CSS;
    } else if (tt_ieq(val, "sss") || tt_ieq(val, "stages")) {
      c->entry_state = GW_SL_AT_SSS;
    } else if (tt_ieq(val, "match") || tt_ieq(val, "game") || tt_ieq(val, "play")) {
      c->entry_state = GW_SL_AT_MATCH;
    } else {
      return -1;
    }
    return 0;
  }
  if (tt_ieq(key, "stage")) {
    return gw_sl_parse_stage(val, &c->stage_ext);
  }
  if (key[0] == 'p' && key[1] >= '1' && key[1] <= '4' && key[2] == '\0') {
    return gw_sl_parse_player(val, &c->p[key[1] - '1']);
  }
  if (tt_ieq(key, "skipmemcard")) {
    c->skip_memcard = (val[0] == '1');
    return 0;
  }
  if (tt_ieq(key, "teams")) {
    c->teams = (val[0] == '1');
    return 0;
  }
  if (tt_ieq(key, "time")) {
    if (!gw_sl_all_digits(val)) return -1;
    c->time_limit = atoi(val);
    return 0;
  }
  if (tt_ieq(key, "items")) {
    if (!gw_sl_all_digits(val)) return -1;
    c->item_freq = atoi(val);
    return 0;
  }
  return -1;
}

/* Parses a whole config string. Separators: ';', ',' or newline. `#` comments to end of line.
 * A rejected field is logged and counted but does not throw the rest of the config away. */
static void gw_sl_parse(GwSceneConfig *c, const char *text, const char *source) {
  const char *s = text;
  gw_sl_config_init(c);
  while (*s != '\0') {
    char key[64], val[160];
    const char *start;
    size_t klen = 0, vlen = 0;
    while (*s == ' ' || *s == '\t' || *s == ';' || *s == ',' || *s == '\n' || *s == '\r') s++;
    if (*s == '#') {
      while (*s != '\0' && *s != '\n') s++;
      continue;
    }
    if (*s == '\0') break;
    start = s;
    while (*s != '\0' && *s != '=' && *s != ' ' && *s != '\t' && *s != ';' && *s != ',' &&
           *s != '\n' && *s != '\r') {
      s++;
    }
    klen = (size_t)(s - start);
    if (klen >= sizeof key) klen = sizeof key - 1;
    memcpy(key, start, klen);
    key[klen] = '\0';
    while (*s == '=' || *s == ' ' || *s == '\t') s++;
    start = s;
    while (*s != '\0' && *s != ';' && *s != ',' && *s != '\n' && *s != '\r' && *s != '#') s++;
    vlen = (size_t)(s - start);
    while (vlen > 0 && (start[vlen - 1] == ' ' || start[vlen - 1] == '\t')) vlen--;
    if (vlen >= sizeof val) vlen = sizeof val - 1;
    memcpy(val, start, vlen);
    val[vlen] = '\0';
    if (key[0] == '\0') continue;
    if (gw_sl_apply(c, key, val) != 0) {
      c->errors++;
      gw_log("gw: scene: %s: rejected \"%s=%s\" -- a number in a character or stage field must "
             "name its index space (ck:/fk:/mex:/mexext:, ext:/int:)",
             source, key, val);
    }
  }
}

/* The legacy single-purpose variables, folded into the same config so there is one seeding path. */
static void gw_sl_load_legacy(GwSceneConfig *c) {
  const char *v;
  if ((v = getenv("MELEE_TRAINING")) != NULL && v[0] != '\0') {
    int ck = tt_parse_ckind(v);
    if (ck >= 0) {
      c->configured = 1;
      c->game_mode = 0x1C; /* GM_TRAINING */
      c->vs_mode = 6;      /* GmVsMode_Training */
      c->entry_state = GW_SL_AT_MATCH;
      c->p[0].ckind = ck;
      gw_log("gw: MELEE_TRAINING=\"%s\" -> CharacterKind %d (= FighterKind %d). Training Mode.",
             v, ck, gw_SceneLaunch_CKindToFKind(ck));
    } else {
      gw_log("gw: MELEE_TRAINING=\"%s\" is not a CharacterKind or a known name -- ignored", v);
    }
  }
  if ((v = getenv("MELEE_TARGET_TEST")) != NULL && v[0] != '\0') {
    int ck = tt_parse_ckind(v);
    if (ck >= 0) {
      c->configured = 1;
      c->game_mode = 0x0F; /* GM_TARGET_TEST */
      c->vs_mode = -1;
      c->tt_ckind = ck;
      gw_log("gw: MELEE_TARGET_TEST=\"%s\" -> CharacterKind %d (= FighterKind %d). Target Test.",
             v, ck, gw_SceneLaunch_CKindToFKind(ck));
    }
  }
  if ((v = getenv("MELEE_STAGE")) != NULL && v[0] != '\0') {
    if (gw_sl_parse_stage(v, &c->stage_ext) != 0 && gw_sl_all_digits(v)) {
      c->stage_ext = atoi(v); /* historical grammar: a bare integer is an external StKind */
    }
    gw_log("gw: MELEE_STAGE=\"%s\" -> external StKind %d", v, c->stage_ext);
  }
}

static void gw_sl_log_config(const GwSceneConfig *c) {
  int i;
  if (!c->configured) {
    gw_log("gw: scene: no scene requested (MELEE_SCENE / MELEE_SCENE_FILE / MELEE_TRAINING / "
           "MELEE_TARGET_TEST unset) -- booting normally");
    return;
  }
  gw_log("gw: scene: mode=%d vsmode=%d at=%d stage=ext:%d skip_memcard=%d errors=%d",
         c->game_mode, c->vs_mode, c->entry_state, c->stage_ext,
         c->skip_memcard < 0 ? 1 : c->skip_memcard, c->errors);
  for (i = 0; i < GW_SL_SLOTS; ++i) {
    const GwSlPlayer *p = &c->p[i];
    if (p->ckind < 0 && !p->random) continue;
    gw_log("gw: scene:   p%d ck=%d (fk=%d) random=%d type=%d color=%d cpu=%d/%d hmp=%d team=%d",
           i + 1, p->ckind, p->ckind >= 0 ? gw_SceneLaunch_CKindToFKind(p->ckind) : -1, p->random,
           p->slot_type, p->color, p->cpu_kind, p->cpu_level, p->handicap, p->team);
  }
}

static void gw_sl_load(void) {
  const char *text;
  char filebuf[4096];
  if (gw_sl_loaded) return;
  gw_sl_loaded = 1;
  gw_sl_config_init(&gw_sl_cfg);
  text = getenv("MELEE_SCENE");
  if (text == NULL || text[0] == '\0') {
    const char *path = getenv("MELEE_SCENE_FILE");
    if (path != NULL && path[0] != '\0') {
      FILE *f = fopen(path, "rb");
      size_t n = 0;
      if (f != NULL) {
        n = fread(filebuf, 1, sizeof filebuf - 1, f);
        fclose(f);
      } else {
        gw_log("gw: scene: cannot open MELEE_SCENE_FILE=\"%s\"", path);
      }
      filebuf[n] = '\0';
      text = filebuf;
      if (n != 0) {
        gw_log("gw: scene: MELEE_SCENE_FILE=\"%s\" (%u bytes)", path, (unsigned)n);
      }
    }
  } else {
    gw_log("gw: scene: MELEE_SCENE=\"%s\"", text);
  }
  if (text != NULL && text[0] != '\0') {
    gw_sl_parse(&gw_sl_cfg, text, "MELEE_SCENE");
  }
  gw_sl_load_legacy(&gw_sl_cfg);
  /* Target Test has no VsModeData; gmmultiman.c asks for the character directly, so lift it
   * out of player 1 when the config spelled it that way. */
  if (gw_sl_cfg.game_mode == 0x0F && gw_sl_cfg.tt_ckind < 0) {
    gw_sl_cfg.tt_ckind = gw_sl_cfg.p[0].ckind;
  }
  gw_sl_log_config(&gw_sl_cfg);
}

/* Test entry point: replace the live config with one parsed from `text`, bypassing the
 * environment entirely. Passing NULL restores "nothing configured". This is what makes every
 * getenv-backed switch here testable more than once per process - the thing the old
 * read-once-into-a-static hooks could not do. */
void gw_SceneLaunch_LoadForTest(const char *text) {
  gw_sl_loaded = 1;
  gw_sl_config_init(&gw_sl_cfg);
  if (text != NULL) {
    gw_sl_parse(&gw_sl_cfg, text, "test");
  }
}

const void *gw_SceneLaunch_ConfigForTest(void) {
  gw_sl_load();
  return &gw_sl_cfg;
}

/* ---- the surface game code calls (gwtool maps `SceneLaunch_X` to `gw_SceneLaunch_X`) ------- */

int gw_SceneLaunch_Active(void) {
  gw_sl_load();
  return gw_sl_cfg.configured;
}

/* The GameModeKind the boot scene should hand over to, or -1 to boot normally. */
int gw_SceneLaunch_BootGameMode(void) {
  gw_sl_load();
  return gw_sl_cfg.configured ? gw_sl_cfg.game_mode : -1;
}

/* GmVsMode index of the VsModeData row this scene seeds, or -1. */
int gw_SceneLaunch_VsModeIndex(void) {
  gw_sl_load();
  return gw_sl_cfg.configured ? gw_sl_cfg.vs_mode : -1;
}

/* 0 = CSS, 1 = SSS, 2 = match; -1 leaves the mode's own state machine alone. Defaults to the
 * match, because "boot into a screen" without further qualification means the playable one. */
int gw_SceneLaunch_EntryStateId(void) {
  gw_sl_load();
  if (!gw_sl_cfg.configured || gw_sl_cfg.vs_mode < 0) return -1;
  return gw_sl_cfg.entry_state < 0 ? GW_SL_AT_MATCH : gw_sl_cfg.entry_state;
}

int gw_SceneLaunch_StageExternal(void) {
  gw_sl_load();
  return gw_sl_cfg.stage_ext;
}

int gw_SceneLaunch_TargetTestCKind(void) {
  gw_sl_load();
  return gw_sl_cfg.tt_ckind;
}

int gw_SceneLaunch_Teams(void) {
  gw_sl_load();
  return gw_sl_cfg.teams;
}

int gw_SceneLaunch_TimeLimit(void) {
  gw_sl_load();
  return gw_sl_cfg.time_limit;
}

int gw_SceneLaunch_ItemFreq(void) {
  gw_sl_load();
  return gw_sl_cfg.item_freq;
}

/* The boot memory-card prompt blocks forever without input, and it runs BEFORE the boot scene's
 * exit handler - which is where the scene hand-over happens. That is exactly why three scripted
 * launches produced no scene at all: the game was sitting on "there is no save data, create
 * one?" with Yes highlighted. A scene launch therefore skips the prompt (and disables saving)
 * by default; `skipmemcard=0` opts back in, and MELEE_SKIP_MEMCARD=1 skips it with no scene. */
int gw_SceneLaunch_SkipMemcard(void) {
  gw_sl_load();
  if (gw_sl_cfg.skip_memcard >= 0) return gw_sl_cfg.skip_memcard;
  if (gw_sl_cfg.configured) return 1;
  return gw_Env1("MELEE_SKIP_MEMCARD");
}

/* Resolves slot `n`'s CharacterKind, drawing a random playable one if the config asked for it.
 * Called once per slot per seed, so `random` really is random per launch. */
int gw_SceneLaunch_PlayerCKind(int n) {
  gw_sl_load();
  if (n < 0 || n >= GW_SL_SLOTS) return -1;
  if (gw_sl_cfg.p[n].random) {
    return (int)(((unsigned)rand()) % GW_SL_CK_PLAYABLE);
  }
  return gw_sl_cfg.p[n].ckind;
}

int gw_SceneLaunch_PlayerSlotType(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].slot_type : -1;
}
int gw_SceneLaunch_PlayerColor(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].color : -1;
}
int gw_SceneLaunch_PlayerCpuKind(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].cpu_kind : -1;
}
int gw_SceneLaunch_PlayerCpuLevel(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].cpu_level : -1;
}
int gw_SceneLaunch_PlayerHandicap(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].handicap : -1;
}
int gw_SceneLaunch_PlayerTeam(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].team : -1;
}
int gw_SceneLaunch_PlayerStocks(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].stocks : -1;
}
int gw_SceneLaunch_PlayerNametag(int n) {
  gw_sl_load();
  return (n >= 0 && n < GW_SL_SLOTS) ? gw_sl_cfg.p[n].nametag : -1;
}

/* ================================================================================================
 * SCENE REPORT -- "what screen am I on, and what is highlighted?"
 * ================================================================================================
 * An unattended run has no eyes on it, so the log has to say where the game is. Every mode/state
 * transition is reported, and so is every change to a cursor a scene chooses to publish. Both are
 * edge-triggered: a static screen costs one line, not one line a frame. MELEE_SCENE_TRACE=0
 * turns it off.
 */

static int gw_sr_enabled = -1;

int gw_SceneReport_Enabled(void) {
  if (gw_sr_enabled < 0) {
    const char *v = getenv("MELEE_SCENE_TRACE");
    gw_sr_enabled = (v == NULL || v[0] != '0');
  }
  return gw_sr_enabled;
}

static const char *gw_sr_mode_name(int m) {
  static const char *const names[] = {
      "GM_TITLE", "GM_MENU", "GM_VS", "GM_CLASSIC", "GM_ADVENTURE", "GM_ALLSTAR", "GM_DEBUG",
      "GM_DEBUG_SOUND_TEST", "GM_HANYU_CSS", "GM_HANYU_SSS", "GM_CAMERA_MODE", "GM_TOY_GALLERY",
      "GM_TOY_LOTTERY", "GM_TOY_COLLECTION", "GM_DEBUG_VS", "GM_TARGET_TEST",
      "GM_SUPER_SUDDEN_DEATH_VS", "GM_INVISIBLE_VS", "GM_SLOMO_VS", "GM_LIGHTNING_VS",
      "GM_CHALLENGER_APPROACH", "GM_CLASSIC_GOVER", "GM_ADVENTURE_GOVER", "GM_ALLSTAR_GOVER",
      "GM_OPENING_MV", "GM_DEBUG_CUTSCENE", "GM_DEBUG_GOVER", "GM_TOURNAMENT", "GM_TRAINING",
      "GM_TINY_VS", "GM_GIANT_VS", "GM_STAMINA_VS", "GM_HOME_RUN_CONTEST", "GM_10MAN_VS",
      "GM_100MAN_VS", "GM_3MIN_VS", "GM_15MIN_VS", "GM_ENDLESS_VS", "GM_CRUEL_VS",
      "GM_PROGRESSIVE_SCAN", "GM_BOOT", "GM_MEMCARD", "GM_CAMERA_VS", "GM_EVENT",
      "GM_SINGLE_BUTTON_VS"};
  return (m >= 0 && m < (int)(sizeof names / sizeof names[0])) ? names[m] : "GM_?";
}

static const char *gw_sr_scene_name(int s) {
  static const char *const names[] = {
      "GS_TITLE", "GS_MENU", "GS_VS", "GS_SUDDEN_DEATH", "GS_TRAINING", "GS_RESULTS", "GS_0x6",
      "GS_DEBUG_MENU", "GS_CSS", "GS_SSS", "GS_UNK10", "GS_TOY_GALLERY", "GS_TOY_LOTTERY",
      "GS_TOY_COLLECTION", "GS_INTRO_NORMAL", "GS_REGEND_TOYFALL", "GS_REGEND_CONGRATS",
      "GS_CUTSCENE_LUIGI", "GS_CUTSCENE_BRINSTAR", "GS_CUTSCENE_EXPLOSION", "GS_CUTSCENE_3KIRBYS",
      "GS_CUTSCENE_GIANTKIRBY", "GS_CUTSCENE_STARFOX", "GS_CUTSCENE_FZERO", "GS_CUTSCENE_METAL",
      "GS_CUTSCENE_BOWSERTOY", "GS_CUTSCENE_GIGATRANSFORM", "GS_CUTSCENE_GIGADEFEATED",
      "GS_MOVIE_OPENING", "GS_MOVIE_END", "GS_MOVIE_HOWTO", "GS_MOVIE_OMAKE15", "GS_INTRO_EASY",
      "GS_INTRO_ALLSTAR", "GS_GAMEOVER", "GS_COMING_SOON", "GS_TOU_SETUP", "GS_TOU_BRACKET",
      "GS_TOU_ALT", "GS_PRIZE_INTERFACE", "GS_PROG_SCAN", "GS_APPROACH", "GS_MEMCARD",
      "GS_STAFFROLL", "GS_CAMERA_VS"};
  return (s >= 0 && s < (int)(sizeof names / sizeof names[0])) ? names[s] : "GS_?";
}

/* Called from the mode state machine as each scene is entered and left. `phase` is 0 for enter
 * and 1 for leave. */
void gw_SceneReport_State(int phase, int mode, int state_id, int scene_kind) {
  if (!gw_SceneReport_Enabled()) return;
  gw_log("scene: %s mode=%s(%d) state=%d screen=%s(%d)", phase == 0 ? "enter" : "leave ",
         gw_sr_mode_name(mode), mode, state_id, gw_sr_scene_name(scene_kind), scene_kind);
}

/* Edge-triggered cursor/selection report. `what` names the screen ("memcard", "menu", ...) and
 * `a`/`b` are whatever that screen's own state is - a highlighted index, a sub-state. Only a
 * change is logged, so a screen that sits still costs nothing. */
void gw_SceneReport_Cursor(const char *what, int a, int b) {
  static const char *last_what;
  static int last_a = -0x7FFFFFFF, last_b = -0x7FFFFFFF;
  if (!gw_SceneReport_Enabled()) return;
  if (what == last_what && a == last_a && b == last_b) return;
  last_what = what;
  last_a = a;
  last_b = b;
  gw_log("scene: cursor %s a=%d b=%d", what != NULL ? what : "?", a, b);
}

/* The memory-card prompt, named so an agent reading the log can tell the blocking screen from a
 * passing one. `decision` is gmscmemcard.c's tickDecision; `option` is 0 = left/Yes, 1 =
 * right/No. Decision 2/3/5 and 11 are the ones that wait for a button. */
void gw_SceneReport_Memcard(int decision, int option) {
  static int last_d = -1, last_o = -1;
  if (!gw_SceneReport_Enabled()) return;
  if (decision == last_d && option == last_o) return;
  last_d = decision;
  last_o = option;
  gw_log("scene: cursor memcard decision=%d option=%d (%s) -- %s", decision, option,
         option == 0 ? "Yes/left" : "No/right",
         (decision == 2 || decision == 3 || decision == 5 || decision == 11)
             ? "WAITING FOR A BUTTON"
             : "working");
}

/* The main menu tree. `kind` is MenuKind, `hovered`/`confirmed` are MenuFlow's own fields. */
void gw_SceneReport_Menu(int kind, int hovered, int confirmed) {
  static int last_k = -1, last_h = -1, last_c = -1;
  if (!gw_SceneReport_Enabled()) return;
  if (kind == last_k && hovered == last_h && confirmed == last_c) return;
  last_k = kind;
  last_h = hovered;
  last_c = confirmed;
  gw_log("scene: cursor menu kind=%d hovered=%d confirmed=%d", kind, hovered, confirmed);
}

/* ---- scene-launch tests -------------------------------------------------------------------
 * Headless (`--test`), so they parallelise freely. They cover the two things that actually go
 * wrong: the index-space conversions, and the grammar's refusal to guess which space a bare
 * number is in. Parsing is a pure function of a string, so every case here runs in one process -
 * the thing the old read-once-getenv-into-a-static hooks made impossible. */

#include "gw_test.h"

/* The game's own CharacterKind -> FighterKind table, in the exe's data section (not MEM1, so the
 * per-test snapshot restore does not touch it). {s8 internal_id, s8 extra_internal_id, s8
 * has_transformation}, stride 3. Bytes, so no endian accessor is needed. */
extern const signed char gw_ftMapping_list[];

static int test_scene_ckind_fkind_table(void) {
  int ck;
  for (ck = 0; ck < GW_SL_CK_NONE; ++ck) {
    int want = gw_ftMapping_list[ck * 3];
    if (gw_sl_ck_to_fk[ck] != want) {
      gw_test_fail("ck %d: table says FighterKind %d, ftMapping_list says %d", ck,
                   gw_sl_ck_to_fk[ck], want);
      return 1;
    }
    if (gw_SceneLaunch_CKindToFKind(ck) != want) {
      gw_test_fail("CKindToFKind(%d) = %d, want %d", ck, gw_SceneLaunch_CKindToFKind(ck), want);
      return 1;
    }
  }
  /* The permutation really is a permutation, not the identity: Fox is CharacterKind 2 and
   * FighterKind 1. If this ever stops holding, every "is it 37 or 38" answer changes. */
  if (gw_SceneLaunch_CKindToFKind(2) != 1 || gw_SceneLaunch_FKindToCKind(1) != 2) {
    gw_test_fail("Fox: ck 2 <-> fk 1 broken");
    return 1;
  }
  /* m-ex slot i: FighterKind 0x21+i, CharacterKind 0x22+i. Sonic is Akaneia's slot 4, so he is
   * FighterKind 37 AND CharacterKind 38, and MELEE_TRAINING takes the 38. */
  if (gw_SceneLaunch_FKindToCKind(37) != 38 || gw_SceneLaunch_CKindToFKind(38) != 37) {
    gw_test_fail("m-ex slot 4: fk 37 <-> ck 38 broken (got %d / %d)",
                 gw_SceneLaunch_FKindToCKind(37), gw_SceneLaunch_CKindToFKind(38));
    return 1;
  }
  if (gw_SceneLaunch_FKindToCKind(0x21) != 0x22) {
    gw_test_fail("m-ex slot 0: fk 0x21 -> ck %d, want 0x22", gw_SceneLaunch_FKindToCKind(0x21));
    return 1;
  }
  return 0;
}

static int test_scene_parse_training(void) {
  const GwSceneConfig *c;
  gw_SceneLaunch_LoadForTest("mode=training;p1=fox");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (!c->configured || c->game_mode != 0x1C || c->vs_mode != 6) {
    gw_test_fail("mode=training gave mode %d vsmode %d", c->game_mode, c->vs_mode);
    return 1;
  }
  if (c->p[0].ckind != 2) {
    gw_test_fail("p1=fox gave CharacterKind %d, want 2", c->p[0].ckind);
    return 1;
  }
  if (c->errors != 0) {
    gw_test_fail("clean config reported %d errors", c->errors);
    return 1;
  }
  /* No `at=`: the default screen is the playable one, state 2 in both GM_VS and GM_TRAINING. */
  if (gw_SceneLaunch_EntryStateId() != 2) {
    gw_test_fail("default entry state is %d, want 2", gw_SceneLaunch_EntryStateId());
    return 1;
  }
  gw_SceneLaunch_LoadForTest(NULL);
  return 0;
}

static int test_scene_index_spaces(void) {
  const GwSceneConfig *c;
  /* ck: and fk: name DIFFERENT characters for the same number, which is the whole point. */
  gw_SceneLaunch_LoadForTest("mode=training;p1=ck:38;p2=fk:37;p3=ck:37");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->p[0].ckind != 38 || c->p[1].ckind != 38) {
    gw_test_fail("ck:38 -> %d and fk:37 -> %d, both should be CharacterKind 38", c->p[0].ckind,
                 c->p[1].ckind);
    return 1;
  }
  if (c->p[2].ckind != 37) {
    gw_test_fail("ck:37 -> %d, want 37 (a DIFFERENT fighter from fk:37)", c->p[2].ckind);
    return 1;
  }
  /* A bare integer is refused rather than guessed. This is the bug this whole mechanism exists
   * to make impossible: 37 is Sonic's FighterKind and Lucas's CharacterKind. */
  gw_SceneLaunch_LoadForTest("mode=training;p1=37");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->errors != 1 || c->p[0].ckind != -1) {
    gw_test_fail("bare `p1=37` was accepted (errors=%d ckind=%d)", c->errors, c->p[0].ckind);
    return 1;
  }
  /* Names never need a space. */
  gw_SceneLaunch_LoadForTest("mode=vs;p1=falco;p2=Ganondorf");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->p[0].ckind != 20 || c->p[1].ckind != 25) {
    gw_test_fail("names: falco -> %d (want 20), ganondorf -> %d (want 25)", c->p[0].ckind,
                 c->p[1].ckind);
    return 1;
  }
  gw_SceneLaunch_LoadForTest(NULL);
  return 0;
}

static int test_scene_parse_vs_four(void) {
  const GwSceneConfig *c;
  gw_SceneLaunch_LoadForTest(
      "mode=vs;at=css;p1=fox/c1/hu;p2=falco/cpu5;p3=random/cpu3/team2;p4=marth/cpu9/hmp4;"
      "stage=fd;teams=1;time=180");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->game_mode != 0x02 || c->vs_mode != 0) {
    gw_test_fail("mode=vs gave mode %d vsmode %d", c->game_mode, c->vs_mode);
    return 1;
  }
  if (c->entry_state != 0 || gw_SceneLaunch_EntryStateId() != 0) {
    gw_test_fail("at=css gave entry state %d", c->entry_state);
    return 1;
  }
  if (c->p[0].ckind != 2 || c->p[0].color != 1 || c->p[0].slot_type != GW_SL_PK_HUMAN) {
    gw_test_fail("p1=fox/c1/hu -> ck %d color %d type %d", c->p[0].ckind, c->p[0].color,
                 c->p[0].slot_type);
    return 1;
  }
  if (c->p[1].slot_type != GW_SL_PK_CPU || c->p[1].cpu_level != 5) {
    gw_test_fail("p2=falco/cpu5 -> type %d level %d", c->p[1].slot_type, c->p[1].cpu_level);
    return 1;
  }
  if (!c->p[2].random || c->p[2].team != 2) {
    gw_test_fail("p3=random/cpu3/team2 -> random %d team %d", c->p[2].random, c->p[2].team);
    return 1;
  }
  if (c->p[3].ckind != 9 || c->p[3].handicap != 4 || c->p[3].cpu_level != 9) {
    gw_test_fail("p4=marth/cpu9/hmp4 -> ck %d hmp %d level %d", c->p[3].ckind, c->p[3].handicap,
                 c->p[3].cpu_level);
    return 1;
  }
  if (c->stage_ext != 32 || c->teams != 1 || c->time_limit != 180) {
    gw_test_fail("rules: stage %d teams %d time %d", c->stage_ext, c->teams, c->time_limit);
    return 1;
  }
  /* A `random` slot resolves to a playable CharacterKind, fresh on each read. */
  {
    int i, k;
    for (i = 0; i < 64; ++i) {
      k = gw_SceneLaunch_PlayerCKind(2);
      if (k < 0 || k >= GW_SL_CK_PLAYABLE) {
        gw_test_fail("random slot produced CharacterKind %d, outside 0..%d", k,
                     GW_SL_CK_PLAYABLE - 1);
        return 1;
      }
    }
  }
  if (c->errors != 0) {
    gw_test_fail("clean 4-player config reported %d errors", c->errors);
    return 1;
  }
  gw_SceneLaunch_LoadForTest(NULL);
  return 0;
}

static int test_scene_parse_stage(void) {
  const GwSceneConfig *c;
  gw_SceneLaunch_LoadForTest("mode=training;p1=fox;stage=ext:293");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->stage_ext != 293) {
    gw_test_fail("stage=ext:293 -> %d", c->stage_ext);
    return 1;
  }
  gw_SceneLaunch_LoadForTest("mode=training;p1=fox;stage=battlefield");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->stage_ext != 31) {
    gw_test_fail("stage=battlefield -> %d, want 31", c->stage_ext);
    return 1;
  }
  /* Stages have two spaces too (internal GrKind vs external StKind), so a bare number is
   * refused here for the same reason it is for characters. */
  gw_SceneLaunch_LoadForTest("mode=training;p1=fox;stage=293");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->errors != 1 || c->stage_ext != -1) {
    gw_test_fail("bare `stage=293` was accepted (errors=%d stage=%d)", c->errors, c->stage_ext);
    return 1;
  }
  gw_SceneLaunch_LoadForTest(NULL);
  return 0;
}

static int test_scene_memcard_default(void) {
  /* The boot memory-card prompt blocks a headless run forever, so a configured scene skips it
   * unless the config says otherwise. With no scene, nothing changes. */
  gw_SceneLaunch_LoadForTest("mode=training;p1=fox");
  if (!gw_SceneLaunch_SkipMemcard()) {
    gw_test_fail("a configured scene does not skip the memcard prompt");
    return 1;
  }
  gw_SceneLaunch_LoadForTest("mode=training;p1=fox;skipmemcard=0");
  if (gw_SceneLaunch_SkipMemcard()) {
    gw_test_fail("skipmemcard=0 did not opt back into the prompt");
    return 1;
  }
  gw_SceneLaunch_LoadForTest(NULL);
  if (gw_SceneLaunch_Active()) {
    gw_test_fail("an empty config still reports a scene");
    return 1;
  }
  return 0;
}

static int test_scene_parse_file_form(void) {
  /* MELEE_SCENE_FILE uses the same grammar with newlines and `#` comments, and `key value` is
   * accepted alongside `key=value` so a config file reads like the .tt mod files. */
  const GwSceneConfig *c;
  gw_SceneLaunch_LoadForTest("# a scene file\nmode training\np1 ck:38\nstage ext:293\n"
                             "at match   # trailing comment\n");
  c = (const GwSceneConfig *)gw_SceneLaunch_ConfigForTest();
  if (c->game_mode != 0x1C || c->p[0].ckind != 38 || c->stage_ext != 293 ||
      c->entry_state != 2 || c->errors != 0)
  {
    gw_test_fail("file form: mode %d ck %d stage %d at %d errors %d", c->game_mode,
                 c->p[0].ckind, c->stage_ext, c->entry_state, c->errors);
    return 1;
  }
  gw_SceneLaunch_LoadForTest(NULL);
  return 0;
}

/* ---- .gxtex: host-side GX textures for custom menus ---------------------------------------
 *
 * Melee's own art arrives inside HSD archives on the disc. Anything the port adds cannot, so
 * this reads a `.gxtex` written by pc/tools/png2gx.py: a 64-byte big-endian header followed by
 * GX-tiled image bytes and, for a colour-indexed format, its TLUT.
 *
 * WHY THE GAME NEVER SEES THE FILE. gwtool byte-swaps every memory access in a game TU, so a
 * game-side reader would swap the texels on the way in and hand GX a shuffled texture. Here the
 * payload is moved with memcpy, which is byte-for-byte in either world, into a buffer the game
 * allocated - and GX texture data is big-endian to begin with, which is exactly what survives
 * that copy. Everything else crosses the boundary as a return value rather than through an
 * out-pointer, for the same reason.
 *
 * MELEE_MENUTEX_DIR names the directory. Unset, GxTex_Open always fails and every caller is a
 * no-op, which is how this stays off in a normal run. */
#define GW_GXTEX_MAGIC 0x47585458u /* "GXTX" */
#define GW_GXTEX_VERSION 1
#define GW_GXTEX_MAX 8

typedef struct {
  unsigned char *blob;
  uint32_t size;
  uint32_t format, width, height;
  uint32_t tlut_fmt, tlut_entries;
  uint32_t image_off, image_size;
  uint32_t tlut_off, tlut_size;
} GwGxTex;

static GwGxTex gw_gxtex[GW_GXTEX_MAX];

static GwGxTex *gw_gxtex_get(int handle) {
  if (handle < 0 || handle >= GW_GXTEX_MAX || gw_gxtex[handle].blob == NULL) {
    return NULL;
  }
  return &gw_gxtex[handle];
}

/* The header is big-endian like the payload, so the whole file is one byte order. */
static uint32_t gw_gxtex_be32(const unsigned char *p) {
  return ((uint32_t)p[0] << 24) | ((uint32_t)p[1] << 16) | ((uint32_t)p[2] << 8) | p[3];
}

int gw_GxTex_Open(const char *name) {
  const char *dir = getenv("MELEE_MENUTEX_DIR");
  char path[1024];
  FILE *f;
  long len;
  unsigned char *blob;
  GwGxTex t;
  int i;

  if (dir == NULL || *dir == '\0' || name == NULL) {
    return -1;
  }
  for (i = 0; i < GW_GXTEX_MAX; i++) {
    if (gw_gxtex[i].blob == NULL) {
      break;
    }
  }
  if (i == GW_GXTEX_MAX) {
    gw_log("gxtex: no free slot for '%s' (%d open)", name, GW_GXTEX_MAX);
    return -1;
  }
  if (snprintf(path, sizeof path, "%s/%s.gxtex", dir, name) >= (int)sizeof path) {
    gw_log("gxtex: path for '%s' is too long", name);
    return -1;
  }
  f = fopen(path, "rb");
  if (f == NULL) {
    gw_log("gxtex: cannot open %s", path);
    return -1;
  }
  fseek(f, 0, SEEK_END);
  len = ftell(f);
  fseek(f, 0, SEEK_SET);
  if (len < 64) {
    gw_log("gxtex: %s is %ld bytes, too short for a header", path, len);
    fclose(f);
    return -1;
  }
  blob = (unsigned char *)malloc((size_t)len);
  if (blob == NULL || fread(blob, 1, (size_t)len, f) != (size_t)len) {
    gw_log("gxtex: short read on %s", path);
    free(blob);
    fclose(f);
    return -1;
  }
  fclose(f);

  memset(&t, 0, sizeof t);
  if (gw_gxtex_be32(blob) != GW_GXTEX_MAGIC || gw_gxtex_be32(blob + 4) != GW_GXTEX_VERSION) {
    gw_log("gxtex: %s is not a v%d .gxtex", path, GW_GXTEX_VERSION);
    free(blob);
    return -1;
  }
  t.format = gw_gxtex_be32(blob + 8);
  t.width = gw_gxtex_be32(blob + 12);
  t.height = gw_gxtex_be32(blob + 16);
  t.tlut_fmt = gw_gxtex_be32(blob + 20);
  t.tlut_entries = gw_gxtex_be32(blob + 24);
  t.image_size = gw_gxtex_be32(blob + 28);
  t.tlut_size = gw_gxtex_be32(blob + 32);
  t.image_off = gw_gxtex_be32(blob + 36);
  t.tlut_off = gw_gxtex_be32(blob + 40);
  /* Bounds-check before anything can copy out of the blob: a truncated or hand-edited file
     would otherwise read past the allocation inside GxTex_CopyImage, far from here. */
  if ((uint64_t)t.image_off + t.image_size > (uint64_t)len ||
      (t.tlut_size != 0 && (uint64_t)t.tlut_off + t.tlut_size > (uint64_t)len))
  {
    gw_log("gxtex: %s: image %u@%u tlut %u@%u do not fit in %ld bytes", path,
           (unsigned)t.image_size, (unsigned)t.image_off, (unsigned)t.tlut_size,
           (unsigned)t.tlut_off, len);
    free(blob);
    return -1;
  }
  t.blob = blob;
  t.size = (uint32_t)len;
  gw_gxtex[i] = t;
  gw_log("gxtex: %s -> handle %d: %ux%u fmt %u, image %u B, tlut %u entries fmt %u", path, i,
         (unsigned)t.width, (unsigned)t.height, (unsigned)t.format, (unsigned)t.image_size,
         (unsigned)t.tlut_entries, (unsigned)t.tlut_fmt);
  return i;
}

/* MELEE_MENUTEX names the element. The game asks for "the one the environment names" rather
   than passing a string, so no game TU ever reads host bytes through a swapped access. */
int gw_GxTex_OpenEnv(void) {
  const char *name = getenv("MELEE_MENUTEX");
  return (name == NULL || *name == '\0') ? -1 : gw_GxTex_Open(name);
}

int gw_GxTex_Width(int h) { GwGxTex *t = gw_gxtex_get(h); return t ? (int)t->width : 0; }
int gw_GxTex_Height(int h) { GwGxTex *t = gw_gxtex_get(h); return t ? (int)t->height : 0; }
int gw_GxTex_Format(int h) { GwGxTex *t = gw_gxtex_get(h); return t ? (int)t->format : -1; }
int gw_GxTex_ImageSize(int h) { GwGxTex *t = gw_gxtex_get(h); return t ? (int)t->image_size : 0; }
int gw_GxTex_TlutSize(int h) { GwGxTex *t = gw_gxtex_get(h); return t ? (int)t->tlut_size : 0; }

int gw_GxTex_TlutFormat(int h) {
  GwGxTex *t = gw_gxtex_get(h);
  /* 0xFFFFFFFF in the file means "no TLUT"; say so as -1, which is what C code tests. */
  return (t == NULL || t->tlut_size == 0) ? -1 : (int)t->tlut_fmt;
}

int gw_GxTex_TlutEntries(int h) {
  GwGxTex *t = gw_gxtex_get(h);
  return (t == NULL || t->tlut_size == 0) ? 0 : (int)t->tlut_entries;
}

/* `dst` is a game-heap pointer. A byte copy is the whole point: GX texture data is already in
   the byte order the hardware (and Aurora's decoder) wants, so nothing must reinterpret it. */
void gw_GxTex_CopyImage(int h, void *dst) {
  GwGxTex *t = gw_gxtex_get(h);
  if (t != NULL && dst != NULL) {
    memcpy(dst, t->blob + t->image_off, t->image_size);
  }
}

void gw_GxTex_CopyTlut(int h, void *dst) {
  GwGxTex *t = gw_gxtex_get(h);
  if (t != NULL && dst != NULL && t->tlut_size != 0) {
    memcpy(dst, t->blob + t->tlut_off, t->tlut_size);
  }
}

void gw_GxTex_Close(int h) {
  GwGxTex *t = gw_gxtex_get(h);
  if (t != NULL) {
    free(t->blob);
    memset(t, 0, sizeof *t);
  }
}

static int test_gxtex_header_and_copy(void) {
  /* Build a two-texel-tile .gxtex in memory, write it, read it back through the real loader.
     This is the boundary that matters: the bytes a game TU ends up holding must be the bytes
     in the file, in that order, with nothing swapped on the way. */
  static const unsigned char image[32] = {
      0x80, 0x01, 0x80, 0x02, 0x80, 0x03, 0x80, 0x04, 0x80, 0x05, 0x80,
      0x06, 0x80, 0x07, 0x80, 0x08, 0x80, 0x09, 0x80, 0x0A, 0x80, 0x0B,
      0x80, 0x0C, 0x80, 0x0D, 0x80, 0x0E, 0x80, 0x0F, 0x80, 0x10,
  };
  unsigned char file[64 + 32];
  unsigned char got[32];
  char dir[1024];
  char path[1024];
  const char *tmp = getenv("TEMP");
  const char *saved = getenv("MELEE_MENUTEX_DIR");
  char restore[1024];
  FILE *f;
  int h, rc = 0;
  unsigned i;
  static const uint32_t header[] = { GW_GXTEX_MAGIC, GW_GXTEX_VERSION, 5, 4, 4, 0xFFFFFFFFu,
                                     0, 32, 0, 64, 96 };

  if (tmp == NULL) {
    return 0; /* no writable scratch: nothing to prove, and nothing to fail either */
  }
  snprintf(restore, sizeof restore, "MELEE_MENUTEX_DIR=%s", saved ? saved : "");
  snprintf(dir, sizeof dir, "%s", tmp);
  snprintf(path, sizeof path, "%s/gw_gxtex_test.gxtex", dir);

  memset(file, 0, sizeof file);
  for (i = 0; i < sizeof header / sizeof header[0]; i++) {
    file[i * 4 + 0] = (unsigned char)(header[i] >> 24);
    file[i * 4 + 1] = (unsigned char)(header[i] >> 16);
    file[i * 4 + 2] = (unsigned char)(header[i] >> 8);
    file[i * 4 + 3] = (unsigned char)header[i];
  }
  memcpy(file + 64, image, 32);
  f = fopen(path, "wb");
  if (f == NULL) {
    gw_test_fail("gxtex: cannot write %s", path);
    return 1;
  }
  fwrite(file, 1, sizeof file, f);
  fclose(f);

  _putenv_s("MELEE_MENUTEX_DIR", dir);
  h = gw_GxTex_Open("gw_gxtex_test");
  if (h < 0) {
    gw_test_fail("gxtex: GxTex_Open failed on %s", path);
    rc = 1;
  } else {
    if (gw_GxTex_Width(h) != 4 || gw_GxTex_Height(h) != 4 || gw_GxTex_Format(h) != 5 ||
        gw_GxTex_ImageSize(h) != 32 || gw_GxTex_TlutFormat(h) != -1 ||
        gw_GxTex_TlutEntries(h) != 0)
    {
      gw_test_fail("gxtex: header read back as %dx%d fmt %d image %d tlut %d/%d",
                   gw_GxTex_Width(h), gw_GxTex_Height(h), gw_GxTex_Format(h),
                   gw_GxTex_ImageSize(h), gw_GxTex_TlutFormat(h), gw_GxTex_TlutEntries(h));
      rc = 1;
    }
    memset(got, 0, sizeof got);
    gw_GxTex_CopyImage(h, got);
    if (memcmp(got, image, sizeof image) != 0) {
      gw_test_fail("gxtex: image bytes changed in the copy: %02X %02X vs %02X %02X", got[0],
                   got[1], image[0], image[1]);
      rc = 1;
    }
    gw_GxTex_Close(h);
    if (gw_GxTex_Width(h) != 0) {
      gw_test_fail("gxtex: handle %d still live after Close", h);
      rc = 1;
    }
  }
  _putenv(restore);
  remove(path);
  return rc;
}

void gw_scene_tests_register(void) {
  gw_test_register("gxtex_header_and_copy", test_gxtex_header_and_copy);
  gw_test_register("scene_ckind_fkind_table", test_scene_ckind_fkind_table);
  gw_test_register("scene_parse_training", test_scene_parse_training);
  gw_test_register("scene_index_spaces", test_scene_index_spaces);
  gw_test_register("scene_parse_vs_four", test_scene_parse_vs_four);
  gw_test_register("scene_parse_stage", test_scene_parse_stage);
  gw_test_register("scene_memcard_default", test_scene_memcard_default);
  gw_test_register("scene_parse_file_form", test_scene_parse_file_form);
}
