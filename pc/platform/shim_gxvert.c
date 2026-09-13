/* Immediate-mode vertex shims.
 *
 * Melee's GXVert.h normally writes vertex attributes straight into the GameCube's write-gather
 * pipe at 0xCC008000. Under TARGET_PC the header declares the entry points instead, and these
 * wrappers forward to Aurora's real implementations, which append to its own big-endian FIFO.
 * Only the entry points the game actually calls are defined here; the rest of the declared set
 * can be added the same way if new call sites appear. */
#include "gw.h"

#include <dolphin/gx.h>

void gw_GXPosition3f32(f32 x, f32 y, f32 z) { GXPosition3f32(x, y, z); }
void gw_GXPosition2f32(f32 x, f32 y) { GXPosition2f32(x, y); }
void gw_GXPosition2u8(u8 x, u8 y) { GXPosition2u8(x, y); }

void gw_GXNormal3f32(f32 x, f32 y, f32 z) { GXNormal3f32(x, y, z); }

void gw_GXColor4u8(u8 r, u8 g, u8 b, u8 a) { GXColor4u8(r, g, b, a); }
void gw_GXColor3u8(u8 r, u8 g, u8 b) { GXColor3u8(r, g, b); }
void gw_GXColor1u16(u16 clr) { GXColor1u16(clr); }
void gw_GXColor1x16(u16 index) { GXColor1x16(index); }
void gw_GXColor1x8(u8 index) { GXColor1x8(index); }

void gw_GXTexCoord2f32(f32 s, f32 t) { GXTexCoord2f32(s, t); }
void gw_GXTexCoord2u8(u8 s, u8 t) { GXTexCoord2u8(s, t); }
void gw_GXTexCoord1u8(u8 s) { GXTexCoord1u8(s); }
void gw_GXTexCoord1x16(u16 index) { GXTexCoord1x16(index); }
void gw_GXTexCoord1x8(u8 index) { GXTexCoord1x8(index); }
