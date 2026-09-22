/* OS side of the port: the clock alarms run on and the entry points Aurora does not provide. */
#ifndef GW_SHIM_OS_H
#define GW_SHIM_OS_H

#include "gw.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Fires every alarm whose deadline has passed against the port's virtual clock. The frame driver
 * calls this once per field (see shim_vi.h), which is where the GameCube's timer interrupt would
 * have landed. */
void gw_os_run_alarms(uint64_t ticks);
/* Diagnostics: how many alarms are armed, and how many have fired since startup. */
void gw_os_alarm_stats(uint32_t *active, uint32_t *fired);
/* The next deadline of the game's periodic pad alarm (the only periodic alarm whose period is
 * one 60 Hz field: lb_80019628 arms it to sample the controllers). Returns 0 when none is armed.
 * The frame driver paces against it, so the controllers are sampled right after the wait. */
int gw_os_pad_alarm_deadline(uint64_t period_ticks, uint64_t *fire_at);

#ifdef __cplusplus
}
#endif
#endif
