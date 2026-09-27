/* Host-only frame diagnostics. Never included in a rollback snapshot. */
#ifndef GW_PERF_H
#define GW_PERF_H
#include <stdint.h>
#define GW_PERF_HISTORY 240
typedef struct {
  float total_ms, logic_ms, gx_ms, submit_ms, worker_ms;
  uint32_t draw_calls, vertices, fx_particles;
} GwPerfFrame;
/* Oldest to newest; a read renews the short sampling lease for a script overlay. */
int gw_perf_snapshot(GwPerfFrame *out, int cap, float *fps, int *target);
#endif
