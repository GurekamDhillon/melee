/* Oracle stand-in for pc/platform/gw.h: only what shim_ax.c uses. The byte-order helpers are the
 * real ones (copied); logging, timing and profiling are inert. */
#pragma once
#include <stdint.h>
#include <stdbool.h>
#include <string.h>
static inline uint16_t gw_bswap16(uint16_t v) { return (uint16_t)((v >> 8) | (v << 8)); }
static inline uint32_t gw_bswap32(uint32_t v) {
  return (v >> 24) | ((v >> 8) & 0xFF00u) | ((v << 8) & 0xFF0000u) | (v << 24);
}
static inline uint16_t gw_r16(const void *p) { uint16_t v; memcpy(&v, p, 2); return gw_bswap16(v); }
static inline void gw_w16(void *p, uint16_t v) { v = gw_bswap16(v); memcpy(p, &v, 2); }
static inline uint32_t gw_r32(const void *p) { uint32_t v; memcpy(&v, p, 4); return gw_bswap32(v); }
static inline void gw_w32(void *p, uint32_t v) { v = gw_bswap32(v); memcpy(p, &v, 4); }
static inline float gw_rf32(const void *p) { uint32_t v = gw_r32(p); float f; memcpy(&f, &v, 4); return f; }
static inline void *gw_rptr(const void *p) { return (void *)(uintptr_t)gw_r32(p); }
static inline void gw_wptr(void *p, const void *v) { gw_w32(p, (uint32_t)(uintptr_t)v); }
static inline void gw_log(const char *fmt, ...) { (void)fmt; }
static inline int gw_turbo_enabled(void) { return 0; }
static inline uint64_t gw_time_ticks(void) { return 0; }
static inline char *gw_path_separator(char *s) { return strrchr(s, 92); }
extern unsigned char *gw_aram;
extern uint32_t gw_aram_size;
