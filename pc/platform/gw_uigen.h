/* gw_uigen.h - runtime generation of UI art: byte-exact GX texture rasterizers and a
 * content-enumeration surface over the m-ex runtime.
 *
 * The menu art (CSS portraits, SSS icons) is currently prebuilt offline by pc/tools/png2gx.py and
 * shipped as .gxtex files. The point of this module is to let the port GENERATE that art at
 * runtime from the m-ex tables instead, so a custom build's roster renders without anyone
 * prebuilding PNGs.
 *
 * Two halves:
 *
 *  1. Rasterizers. gw_UI_RasterRgba8 / gw_UI_RasterI4 turn a straight-alpha, row-major RGBA
 *     buffer into GX-tiled texture bytes, byte-for-byte identical to what png2gx.py's
 *     tile_rgba8 / encode_i4 produce (RGBA8 = GX_TF_RGBA8, 4x4 tiles, AR plane then GB plane;
 *     I4 = GX_TF_I4, 8x8 tiles, high nibble first, luma/alpha coverage). png2gx.py is the
 *     oracle; see gw_uigen.c for the exact expressions.
 *
 *  2. Enumeration. gw_UI_FighterCount/At and gw_UI_StageCount/At expose what a mounted m-ex disc
 *     actually offers, resolved through the existing m-ex runtime accessors (gw_Mex_CssIcon* /
 *     gw_Mex_SssIcon*, gw_Mex_ExtToPortCKind, gw_Mex_GrKindForExt, ...). They re-resolve on every
 *     call and hold no pointers, so they cannot hand back a stale guest address.
 *
 * All ints, no bools, for ABI simplicity at the native boundary (game code's MSL bool is int).
 *
 * NOT vendored from m-ex: original C written from the published behaviour of the m-ex tables,
 * same as gw_mex_sss.c / gw_mex_ftfunction_runtime.c.
 */
#ifndef GW_UIGEN_H
#define GW_UIGEN_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* One CSS icon: an m-ex EXTERNAL fighter id and what the port makes of it. `kind` is the port
 * CharacterKind (gw_Mex_ExtToPortCKind), -1 when the port has no such fighter; `has_file` is
 * non-zero when that fighter's character file is present on this disc. */
typedef struct GwUiFighter {
    int external_id;
    int kind;
    int has_file;
} GwUiFighter;

/* One SSS icon: an m-ex EXTERNAL stage id and what the port makes of it. `grkind` is the port
 * GrKind (gw_Mex_GrKindForExt), -1 when the id does not resolve; `has_file` is non-zero when the
 * stage's data file is present on this disc. */
typedef struct GwUiStage {
    int external_id;
    int grkind;
    int has_file;
} GwUiStage;

/* ---- rasterizers ------------------------------------------------------------------------- */

/* Size in bytes of the GX-tiled payload for a w x h image, or 0 for an invalid size. RGBA8 tiles
 * are 4x4 (64 bytes each); I4 tiles are 8x8 (32 bytes each). w,h must be positive multiples of
 * the tile size. */
int gw_UI_Rgba8Size(int w, int h);
int gw_UI_I4Size(int w, int h);

/* Rasterize `rgba` (w*h*4 bytes, row-major, straight alpha: R,G,B,A per texel) into `dst` as
 * GX-tiled texture data matching png2gx.py exactly. On a bad pointer or an invalid size the call
 * returns without writing. Callers size `dst` with the matching gw_UI_*Size. */
void gw_UI_RasterRgba8(int w, int h, const uint8_t *rgba, void *dst);
void gw_UI_RasterI4(int w, int h, const uint8_t *rgba, void *dst);

/* Guest-pointer variants: identical tiling, but `guest_dst` is a pointer into game-visible memory
 * (a MEM1 address the game allocates with HSD_MemAlloc and hands to GX). Texture bytes are
 * byte-order-neutral, so these write straight through with plain byte stores - never gw_w* (which
 * would byte-swap) and never a reinterpret of the guest pointer. They exist as a named seam so a
 * game TU (gmgenui.c) can prove native writes into a guest pointer arrive intact
 * (gmgenui_guest_raster). */
void gw_UI_RasterRgba8Guest(int w, int h, const uint8_t *rgba_host, void *guest_dst);
void gw_UI_RasterI4Guest(int w, int h, const uint8_t *rgba_host, void *guest_dst);

/* Draw ASCII `text` centred into an I4 texture of w x h texels (style 0 = the results screen's
 * per-player name, 1 = its winner banner); `dst` holds gw_UI_I4Size(w, h rounded up to 8) bytes.
 * 1 = drawn, 0 = failed (dst blank). Game code declares it as UI_TextI4. */
int gw_UI_TextI4(const char *text, int w, int h, int style, void *dst);

/* True when the runtime UI-generation experiment is enabled: MELEE_UIGEN is set and not "0".
 * The game-side gmGenUI_Active() (gmgenui.c) calls this; game code cannot call getenv directly
 * (there is no gw_getenv shim), so the env read lives here on the native side. */
int gw_GenUI_Active(void);

/* ---- content enumeration ------------------------------------------------------------------ */

/* Number of CSS icons on this disc, or 0 when there is no usable m-ex CSS (a retail disc). */
int gw_UI_FighterCount(void);

/* Fill *out with CSS icon i's external id, port CharacterKind and file presence. Returns 1 on
 * success, 0 when i is out of range or the table is unavailable. */
int gw_UI_FighterAt(int i, GwUiFighter *out);

/* Number of SSS icons on this disc, or 0 when there is no usable m-ex SSS (a retail disc). */
int gw_UI_StageCount(void);

/* Fill *out with SSS icon i's external id, port GrKind and file presence. Returns 1 on success,
 * 0 when i is out of range or the table is unavailable. */
int gw_UI_StageAt(int i, GwUiStage *out);

/* Registers this module's self-contained tests with the in-engine suite. */
void gw_uigen_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_UIGEN_H */
