#ifndef GW_SURFACE_H
#define GW_SURFACE_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Native-only API; floats here are native, never game memory. */
uint32_t gw_surface_register(const char *source, const char *label, char *error, unsigned error_size);
int gw_surface_select(unsigned slot, uint32_t program, unsigned owner, const float *params16);
void gw_surface_release(unsigned owner);
/* Cumulative load calls, FIFO commands, active selections. */
void gw_surface_stats(uint64_t out[3]);
/* Scalar game boundary. 1..6 = player slot, 7 = original stage, 0 = unrelated. */
void gw_SurfaceBegin(int slot);
void gw_SurfaceEnd(void);
#ifdef __cplusplus
}
#endif
#endif
