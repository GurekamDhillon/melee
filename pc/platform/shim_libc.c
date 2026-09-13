/* libc/MSL shims.
 *
 * Most of the game's C library calls are byte-oriented or pure math, so the CRT implementations
 * are correct and fast; these wrappers exist only because gwtool renames every external symbol to
 * gw_<name>. The exceptions are the Gekko-shaped ones: sqrtf keeps the Newton-Raphson refinement
 * over the port's __frsqrte (pc/gameworld/gekko_fp.c), matching src/MSL/math_ppc.h.
 *
 * Not yet implemented here (see HANDOFF.md): __va_arg (the port's MSL/stdarg.h routes va_arg
 * through it) and the game-visible data tables __files and MSL_TrigF_80400770/774 (the latter are
 * .sdata2 constants whose values must be recovered from the original DOL).
 *
 * __setjmp/__longjmp are aliased straight to the CRT rather than wrapped: a wrapper would save the
 * shim's own frame instead of the game caller's, which is not what setjmp means. */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"

#include <intrin.h>
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ---- memory ------------------------------------------------------------------------------- */

void *gw_memcpy(void *dst, const void *src, size_t n) { return memcpy(dst, src, n); }
void *gw_memset(void *dst, int val, size_t n) { return memset(dst, val, n); }
int gw_memcmp(const void *a, const void *b, size_t n) { return memcmp(a, b, n); }

/* ---- string ------------------------------------------------------------------------------- */

size_t gw_strlen(const char *s) { return strlen(s); }
char *gw_strcpy(char *dst, const char *src) { return strcpy(dst, src); }
char *gw_strncpy(char *dst, const char *src, size_t n) { return strncpy(dst, src, n); }
int gw_strcmp(const char *a, const char *b) { return strcmp(a, b); }
int gw_strncmp(const char *a, const char *b, size_t n) { return strncmp(a, b, n); }
unsigned long gw_strtoul(const char *s, char **end, int base) { return strtoul(s, end, base); }

/* ---- stdio -------------------------------------------------------------------------------- */

int gw_printf(const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  const int r = vprintf(fmt, ap);
  va_end(ap);
  return r;
}

int gw_sprintf(char *s, const char *fmt, ...) {
  va_list ap;
  va_start(ap, fmt);
  const int r = vsprintf(s, fmt, ap);
  va_end(ap);
  return r;
}

int gw_vsnprintf(char *s, size_t n, const char *fmt, va_list ap) {
  return vsnprintf(s, n, fmt, ap);
}

/* Clang emits this as a marker when the port's MSL stdarg.h initialises a va_list; it has no
 * work to do at runtime. */
void gw___builtin_va_info(void *info) { (void)info; }

/* ---- math --------------------------------------------------------------------------------- */

/* Kept in lockstep with src/MSL/math_ppc.h: same refinement, same __frsqrte estimate. */
extern double gw___frsqrte(double);

float gw_sqrtf(float x) {
  volatile float y;
  if (x > 0.0f) {
    double guess = gw___frsqrte((double)x);
    guess = 0.5 * guess * (3.0 - guess * guess * x);
    guess = 0.5 * guess * (3.0 - guess * guess * x);
    guess = 0.5 * guess * (3.0 - guess * guess * x);
    y = (float)(x * guess);
    return y;
  }
  return x;
}

float gw_sinf(float x) { return sinf(x); }
float gw_cosf(float x) { return cosf(x); }
float gw_tanf(float x) { return tanf(x); }
float gw_atanf(float x) { return atanf(x); }
float gw_logf(float x) { return logf(x); }
double gw_fabs(double x) { return fabs(x); }
float gw_fabsf(float x) { return fabsf(x); }

/* Metrowerks' double -> unsigned long long helper. */
uint64_t gw___cvt_dbl_usll(double x) { return (uint64_t)x; }

/* ---- stack bounds -------------------------------------------------------------------------
 * Linker-defined on the GameCube; only used by a debug report that prints the size of the stack
 * region. Declaring them in this order makes the addresses the right way round on MSVC. */
unsigned char gw__stack_end[0x10000];
unsigned char gw__stack_addr[4];

/* ---- setjmp -------------------------------------------------------------------------------
 * The game's jmp_buf (src/Runtime/Gecko_setjmp.h) is larger than the CRT's and is only ever
 * touched by these two functions, so the CRT's save/restore is a drop-in. Aliasing at link time
 * matters: setjmp saves the calling function's frame, so a C wrapper would be wrong. */
#pragma comment(linker, "/alternatename:_gw___setjmp=_setjmp")
#pragma comment(linker, "/alternatename:_gw___longjmp=_longjmp")

/* ---- ctype --------------------------------------------------------------------------------
 * The flags must match src/MSL/ctype.h: control 0x01, motion 0x02, space 0x04, punctuation 0x08,
 * digit 0x10, hex 0x20, lower 0x40, upper 0x80. C locale, indexed by byte value. */
const unsigned char gw___ctype_map[256] = {
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x03, 0x03, 0x03, 0x03, 0x03, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
    0x04, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
    0x30, 0x30, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x08, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0xA0, 0x80,
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80,
    0x80, 0x80, 0x80, 0x08, 0x08, 0x08, 0x08, 0x08,
    0x08, 0x60, 0x60, 0x60, 0x60, 0x60, 0x60, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40,
    0x40, 0x40, 0x40, 0x08, 0x08, 0x08, 0x08, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
};

/* ---- variadic game code -------------------------------------------------------------------
 * clang lowers the port's MSL stdarg.h to calls to __builtin_va_info/__va_arg, and the contract
 * (va_start passes only the address of the local va_list) cannot recover the caller's arguments on
 * this target, so general varargs are not available yet. The boot-critical caller was moved to
 * explicit parameters (HSD_SetInitParameterU32/Ptr), so anything reaching here is a non-boot path:
 * returning zeroed storage keeps it from faulting, at the cost of seeing zero arguments. */
/* Steps a va_list that llvm.va_start filled with x86 semantics. See src/MSL/stdarg.h for why the
 * game's own va_list machinery cannot work here and why this has to live on the native side.
 *
 * Two conversions happen:
 *   - the va_list slot at offset 0 is a native pointer, read and advanced natively;
 *   - the value itself is copied out byte-swapped, because the caller pushed it natively but the
 *     game will dereference the returned pointer with a gwtool-swapped load.
 * Returning a pointer into a static also matches what the game expects: MSL's va_arg immediately
 * dereferences it, so the storage only has to outlive the expression. */
#define GW_VA_FLOAT 0x10000u

void *gw___va_arg(void *v_list, unsigned info) {
  static union {
    unsigned char b[8];
    float f;
  } slot;
  char **cursor = (char **)v_list;
  const unsigned size = info & 0xFFFFu;
  char *p;
  unsigned step;

  if (cursor == NULL || *cursor == NULL) {
    gw_log("gw: __va_arg on an uninitialised va_list");
    memset(&slot, 0, sizeof slot);
    return slot.b;
  }

  p = *cursor;
  /* Variadic arguments are promoted: floats arrive as doubles, anything narrower than int as int,
   * and every slot is 4-byte aligned on x86. */
  step = (info & GW_VA_FLOAT) ? 8u : ((size < 4u ? 4u : size) + 3u) & ~3u;
  *cursor = p + step;

  if ((info & GW_VA_FLOAT) && size == sizeof(float)) {
    double d;
    memcpy(&d, p, sizeof d);
    slot.f = (float)d; /* narrow first, then hand it over byte-swapped below */
    gw_wf32(slot.b, slot.f);
    return slot.b;
  }

  switch (size) {
  case 1:
    slot.b[0] = (unsigned char)*p;
    break;
  case 2:
    gw_w16(slot.b, (uint16_t) * (const uint16_t *)p);
    break;
  case 8:
    gw_w64(slot.b, *(const uint64_t *)p);
    break;
  case 4:
  default:
    gw_w32(slot.b, *(const uint32_t *)p);
    break;
  }
  return slot.b;
}

/* ---- MSL console I/O ----------------------------------------------------------------------
 * Referenced by src/MSL/ansi_files.c, whose FILE table the debug code manipulates directly.
 * MetroTRK console output is not part of the port. */
int gw___write_console(uint32_t file, uint8_t *buf, uint32_t *n, void *f) {
  (void)file;
  (void)buf;
  (void)n;
  (void)f;
  return 0;
}

int gw___read_console(uint32_t file, uint8_t *buf, uint32_t *n, void *f) {
  (void)file;
  (void)buf;
  (void)n;
  (void)f;
  return 0;
}

int gw___close_console(uint32_t file) {
  (void)file;
  return 0;
}
