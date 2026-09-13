/* gw.h - the boundary between melee's big-endian "game world" and native Windows code.
 *
 * Game code is compiled by clang for PowerPC and then rewritten by gwtool so that every memory
 * access is byte-swapped: values live big-endian in memory (exactly as on a GameCube) and native
 * in registers. Pointers are 32-bit and mean the same addresses natively, so a game pointer can
 * be dereferenced directly here -- but every multi-byte field behind it is big-endian, so it must
 * be read and written with the accessors below.
 *
 * Every SDK/libc function the game calls is an undefined symbol named gw_<name>; the shims in
 * this directory define them. Rules for writing a shim, derived from the PowerPC ABI clang used:
 *   - scalars (integers, floats, pointers) are passed natively: declare them normally;
 *   - a struct passed by value arrives as a byte-for-byte copy, so its fields are big-endian
 *     (GXColor is four bytes, so it is unaffected; GXColorS10 is four s16 and is not);
 *   - a struct return of <= 8 bytes comes back packed into an integer in big-endian order;
 *   - a larger struct return uses a hidden first pointer to a big-endian destination (sret).
 */
#ifndef GW_H
#define GW_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ---- byte-swapped access to game memory ------------------------------------------------- */

static inline uint16_t gw_bswap16(uint16_t v) { return (uint16_t)((v >> 8) | (v << 8)); }
static inline uint32_t gw_bswap32(uint32_t v) {
  return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24);
}
static inline uint64_t gw_bswap64(uint64_t v) {
  return ((uint64_t)gw_bswap32((uint32_t)v) << 32) | gw_bswap32((uint32_t)(v >> 32));
}

static inline uint8_t gw_r8(const void *p) { return *(const uint8_t *)p; }
static inline void gw_w8(void *p, uint8_t v) { *(uint8_t *)p = v; }

static inline uint16_t gw_r16(const void *p) {
  uint16_t v;
  memcpy(&v, p, 2);
  return gw_bswap16(v);
}
static inline void gw_w16(void *p, uint16_t v) {
  v = gw_bswap16(v);
  memcpy(p, &v, 2);
}
static inline uint32_t gw_r32(const void *p) {
  uint32_t v;
  memcpy(&v, p, 4);
  return gw_bswap32(v);
}
static inline void gw_w32(void *p, uint32_t v) {
  v = gw_bswap32(v);
  memcpy(p, &v, 4);
}
static inline uint64_t gw_r64(const void *p) {
  uint64_t v;
  memcpy(&v, p, 8);
  return gw_bswap64(v);
}
static inline void gw_w64(void *p, uint64_t v) {
  v = gw_bswap64(v);
  memcpy(p, &v, 8);
}

static inline float gw_rf32(const void *p) {
  uint32_t v = gw_r32(p);
  float f;
  memcpy(&f, &v, 4);
  return f;
}
static inline void gw_wf32(void *p, float f) {
  uint32_t v;
  memcpy(&v, &f, 4);
  gw_w32(p, v);
}
static inline double gw_rf64(const void *p) {
  uint64_t v = gw_r64(p);
  double d;
  memcpy(&d, &v, 8);
  return d;
}
static inline void gw_wf64(void *p, double d) {
  uint64_t v;
  memcpy(&v, &d, 8);
  gw_w64(p, v);
}

/* A pointer stored in game memory is a big-endian 32-bit value naming the same address space. */
static inline void *gw_rptr(const void *p) { return (void *)(uintptr_t)gw_r32(p); }
static inline void gw_wptr(void *p, const void *v) { gw_w32(p, (uint32_t)(uintptr_t)v); }

/* Bulk helpers for the float arrays the GX API passes around (matrices, viewports, ...). */
void gw_read_f32v(float *dst, const void *src, int count);
void gw_write_f32v(void *dst, const float *src, int count);
void gw_read_mtx(float dst[3][4], const void *src);
void gw_write_mtx(void *dst, const float src[3][4]);
void gw_read_mtx44(float dst[4][4], const void *src);
void gw_write_mtx44(void *dst, const float src[4][4]);

/* ---- diagnostics ------------------------------------------------------------------------ */

void gw_log(const char *fmt, ...);
void gw_logv(const char *fmt, va_list ap);
/* Logs two strings with no format expansion; safe for printing a suspect format string. */
void gw_log_raw(const char *prefix, const char *text);
/* True when MELEE_PC_TRACE_OSREPORT is set; gates the OSReport format trace. */
bool gw_trace_osreport(void);
/* Logs the first call of an unimplemented entry point, then counts silently. Whatever appears
 * during boot is the real to-do list, in the order the game needs it. */
void gw_stub_hit(const char *name);
#define GW_STUB() gw_stub_hit(__FUNCTION__)
void gw_panic(const char *fmt, ...);
void gw_dump_stub_summary(void);
/* Turns a fault from "the log stops" into a logged address, .map RVA and frame list. Install
 * before anything else so faults during startup are reported too. */
void gw_install_crash_handler(void);
/* Logs a code address as a melee-pc.map rva, or as module+offset when it is not in the exe. */
void gw_log_code_addr(const char *label, const void *addr);
/* Samples the game thread's pc every few seconds. Finds loops that never present a frame,
 * which produce no log output and look exactly like a hang from outside. */
void gw_start_watchdog(void);

/* ---- process-wide runtime ---------------------------------------------------------------- */

/* MEM1: the game's 24 MB main memory, mapped at 0x80000000 when the OS allows it, because game
 * code tells main memory from ARAM by comparing addresses against 0x80000000. */
extern unsigned char *gw_mem1;
extern uint32_t gw_mem1_size;
extern unsigned char *gw_aram;
extern uint32_t gw_aram_size;

bool gw_mem_init(void);
/* Fills the OS globals at the bottom of MEM1 that the console's IPL would have written; without
 * them OS_BUS_CLOCK is zero and every tick conversion collapses to zero. Called by gw_mem_init. */
void gw_init_lomem(void);
void gw_apply_fixups(void); /* swap link-time pointers in game globals; must run first */
const char *gw_iso_path(void);

/* melee's own main(), renamed by gwtool. */
void gw_main(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_H */
