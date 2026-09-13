/* AX shim internal interface shared with the other platform shims.
 *
 * The AX mixer lives entirely in shim_ax.c and runs on the game thread; shim_vi.c drives it once
 * per video field and shim_misc.c feeds it the AI master-volume/sample-rate state the game sets.
 */
#ifndef GW_SHIM_AX_H
#define GW_SHIM_AX_H

#include "gw.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Generate zero or more 5 ms AX sub-frames (callback + voice mix) on the game thread, keyed off
 * the virtual clock. Called from gw_frame_tick. */
void gw_ax_frame_tick(void);

/* AI state tracked by shim_misc.c, applied by the mixer in shim_ax.c. */
extern uint8_t gw_ai_stream_vol_left;
extern uint8_t gw_ai_stream_vol_right;
extern uint32_t gw_ai_dsp_sample_rate;

#ifdef __cplusplus
}
#endif
#endif /* GW_SHIM_AX_H */
