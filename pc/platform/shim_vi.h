/* Frame driving and the VI side of the port. */
#ifndef GW_SHIM_VI_H
#define GW_SHIM_VI_H

#include "gw.h"

#ifdef __cplusplus
extern "C" {
#endif

bool gw_frame_init(void);

/* One video field: presents a finished frame, pumps window events, fires the game's retrace
 * callbacks, advances the virtual clock and runs anything that came due (alarms, async I/O).
 * Every blocking wait in the game funnels into here. */
void gw_frame_tick(void);

/* Called by the GX shims: the game finished drawing into the EFB and copied it out, so the
 * Aurora frame in progress is complete and should be presented at the next tick. */
void gw_frame_mark_content(void);

/* GX draw-done notification, owned here because it drives HSD's XFB state machine. */
void gw_gx_set_draw_done(void);
void gw_gx_wait_draw_done(void);
void *gw_gx_set_draw_done_callback(void *cb);

/* The virtual GameCube clock, in OS ticks (bus clock / 4). */
uint64_t gw_time_ticks(void);
void gw_time_advance_field(void);

/* One idle step for blocking-wait loops that never reach VIWaitForRetrace (melee's pad-update
 * spin goes through lb_800195D0, which calls no VI entry point). Advances the virtual clock at
 * roughly one field per real frame period, then runs due alarms and deferred callbacks, so the
 * periodic pad alarm and queued DVD/ARQ completions still make progress. */
void gw_wait_idle(void);
/* Write the next presented frame to a PNG at `path` (render resolution, no host overlay).
 * Asynchronous: the file appears a few frames later. See also MELEE_SHOT_AT in shim_vi.c. */
void gw_Screenshot(const char *path);
/* Diagnostics: retraces counted, frames actually presented, gw_wait_idle calls. */
void gw_frame_stats(uint32_t *retrace, uint32_t *presented, uint32_t *waits);

/* Queued work the pump runs at a safe point, used by the DVD and ARQ shims to complete
 * "asynchronous" transfers the way interrupt callbacks used to. */
typedef void (*gw_deferred_fn)(void *a, void *b, uint32_t c);
void gw_defer(gw_deferred_fn fn, void *a, void *b, uint32_t c);
void gw_run_deferred(void);

#ifdef __cplusplus
}
#endif
#endif
