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

#ifdef __cplusplus
}
#endif
#endif
