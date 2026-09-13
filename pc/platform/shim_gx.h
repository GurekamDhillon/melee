/* The parts of the GX shim other shims need: the render-mode images the game links against,
 * and the conversion from one of them into a native GXRenderModeObj. */
#ifndef GW_SHIM_GX_H
#define GW_SHIM_GX_H

#include <dolphin/gx/GXStruct.h>

#include "gw.h"

#ifdef __cplusplus
extern "C" {
#endif

/* GXRenderModeObj as melee lays it out: u32 viTVmode; u16 fbWidth, efbHeight, xfbHeight,
 * viXOrigin, viYOrigin, viWidth, viHeight; two bytes of padding; u32 xFBmode at 0x14;
 * u8 field_rendering, aa; u8 sample_pattern[12][2] at 0x1A; u8 vfilter[7] at 0x32. */
#define GW_RENDER_MODE_SIZE 60

/* Game code takes the address of these and reads their fields itself, so they are game-visible
 * data: every multi-byte field is big-endian. gw_gx_init_render_modes() fills them in; nothing
 * may read them before that runs. */
extern unsigned char gw_GXNtsc480Int[GW_RENDER_MODE_SIZE];
extern unsigned char gw_GXNtsc480IntDf[GW_RENDER_MODE_SIZE];
extern unsigned char gw_GXNtsc480Prog[GW_RENDER_MODE_SIZE];

void gw_gx_init_render_modes(void);

/* Draw activity since startup: finished EFB copies, GXBegin primitives, display lists called.
 * The frame driver logs these periodically so a blank window can be told apart from a game that
 * is drawing but not presenting. */
void gw_gx_get_stats(uint32_t *copies, uint32_t *prims, uint32_t *dlists);

/* Convert a big-endian GXRenderModeObj in game memory (one of the three above, or one the game
 * built itself) into a native one Aurora can read. */
void gw_read_render_mode(GXRenderModeObj *dst, const void *src_be);

#ifdef __cplusplus
}
#endif
#endif
