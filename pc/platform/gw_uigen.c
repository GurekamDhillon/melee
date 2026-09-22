/* gw_uigen.c - runtime generation of UI art: byte-exact GX texture rasterizers plus a
 * content-enumeration surface over the m-ex runtime.
 *
 * WHY THIS EXISTS
 *   The menu art (CSS portraits, SSS icons) is currently prebuilt offline by pc/tools/png2gx.py
 *   and shipped as .gxtex files. The next step is to GENERATE that art at runtime from the m-ex
 *   tables, so a custom build's roster shows without anyone prebuilding PNGs. This file is the
 *   foundation: it re-implements png2gx.py's two mask/lossless formats in C, byte-for-byte, and
 *   enumerates what a disc actually offers (fighters, stages) so the generator knows what to draw.
 *
 * WHY BYTE-EXACTNESS MATTERS
 *   The GX hardware (via Aurora/Dawn) samples the tiled bytes directly, and the offline pipeline
 *   already shipped art in this exact byte order. A runtime rasterizer that differed by even one
 *   byte (tiling order, plane order, the I4 coverage rounding) would produce textures that either
 *   look wrong or disagree with the prebuilt set for the same source image. png2gx.py is the
 *   ORACLE: these rasterizers are a line-for-line port of its tile_rgba8 / encode_i4 / _mask_level,
 *   and the tests below memcmp against bytes that png2gx.py itself emitted (see the golden arrays).
 *
 * This module caches nothing: every call re-resolves through the m-ex runtime (gw_Mex_*), so it
 * can never hold a pointer into a MEM1 region the test harness has since restored - the same
 * discipline as gw_mex_sss.c.
 *
 * This is NATIVE platform code (compiled directly with the i686 clang, not through gwtool).
 */
#define _CRT_SECURE_NO_WARNINGS
#include "gw.h"
#include "gw_test.h"
#include "gw_uigen.h"
#include "gw_mex_sss.h"
#include "gw_mex_grfunction.h"

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/* ---- m-ex runtime accessors (defined in gw_mex_ftfunction_runtime.c / shim_dvd.c, not in a
 * ---- public header - declared here exactly as gw_tests_core.c does). */
extern int gw_Mex_CssIconCount(void);
extern void *gw_Mex_CssIconTable(void);
extern int gw_Mex_ExtToPortCKind(int ext);
extern int gw_Mex_PortCKindToExt(int ckind);
extern int gw_Mex_InternalForExt(int ext);
extern const char *gw_Mex_FtPlFile(int k);
extern int gw_DVDConvertPathToEntrynum(const char *path);

/* ---- tiling (ported verbatim from png2gx.py) --------------------------------------------- */

/* CSSIcon (mn/types.h) stride and the m-ex EXTERNAL id byte (+0x01). */
#define GW_UI_CSS_STRIDE 0x1Cu
#define GW_UI_CSS_EXT_OFF 0x01u
/* SSS icon row stride (0x20, m-ex's widened row) and the external id word (word 7, a big-endian
 * s32 at +0x1C). */
#define GW_UI_SSS_STRIDE 0x20u
#define GW_UI_SSS_EXT_OFF 0x1Cu

/* png2gx.py _mask_level: a mask texel's coverage. Art ships masks as white + alpha, so coverage is
 * alpha; a mask drawn opaque grey-on-black carries it in luma. min() of the two reads either kind.
 * When a == 255 the min() is redundant but the expression is kept verbatim. Integer division, all
 * operands non-negative, so it matches Python's // exactly. */
static int uigen_mask_level(int r, int g, int b, int a) {
    if (a != 255) {
        return a;
    }
    return (a < (r * 299 + g * 587 + b * 114) / 1000) ? a : (r * 299 + g * 587 + b * 114) / 1000;
}

/* png2gx.py encode_i4's per-texel coverage: (level * 15 + 127) / 255. */
static int uigen_i4_level(const uint8_t *p) {
    return (uigen_mask_level((int) p[0], (int) p[1], (int) p[2], (int) p[3]) * 15 + 127) / 255;
}

static int uigen_tile_ok(int w, int h, int tw, int th) {
    return w > 0 && h > 0 && (w % tw) == 0 && (h % th) == 0;
}

int gw_UI_Rgba8Size(int w, int h) {
    if (!uigen_tile_ok(w, h, 4, 4)) {
        return 0;
    }
    return (w / 4) * (h / 4) * 64;
}

int gw_UI_I4Size(int w, int h) {
    if (!uigen_tile_ok(w, h, 8, 8)) {
        return 0;
    }
    return (w / 8) * (h / 8) * 32;
}

/* png2gx.py tile_rgba8: 4x4 tiles iterated ty then tx; within a tile, texels y-outer/x-inner.
 * Per tile: a 32-byte AR plane (A,R per texel) then a 32-byte GB plane (G,B). All bytes, no
 * byte-swap. */
void gw_UI_RasterRgba8(int w, int h, const uint8_t *rgba, void *dst) {
    uint8_t *out = (uint8_t *) dst;
    int tx, ty, y, x;
    if (rgba == NULL || dst == NULL || !uigen_tile_ok(w, h, 4, 4)) {
        return;
    }
    for (ty = 0; ty < h; ty += 4) {
        for (tx = 0; tx < w; tx += 4) {
            for (y = ty; y < ty + 4; ++y) {
                for (x = tx; x < tx + 4; ++x) {
                    const uint8_t *p = rgba + ((size_t) y * (size_t) w + (size_t) x) * 4u;
                    *out++ = p[3]; /* A */
                    *out++ = p[0]; /* R */
                }
            }
            for (y = ty; y < ty + 4; ++y) {
                for (x = tx; x < tx + 4; ++x) {
                    const uint8_t *p = rgba + ((size_t) y * (size_t) w + (size_t) x) * 4u;
                    *out++ = p[1]; /* G */
                    *out++ = p[2]; /* B */
                }
            }
        }
    }
}

/* png2gx.py encode_i4: 8x8 tiles iterated ty then tx; within a tile, y-outer, x step 2. One byte
 * per texel pair = (level(x) << 4) | level(x+1), high nibble first. */
void gw_UI_RasterI4(int w, int h, const uint8_t *rgba, void *dst) {
    uint8_t *out = (uint8_t *) dst;
    int tx, ty, y, x;
    if (rgba == NULL || dst == NULL || !uigen_tile_ok(w, h, 8, 8)) {
        return;
    }
    for (ty = 0; ty < h; ty += 8) {
        for (tx = 0; tx < w; tx += 8) {
            for (y = ty; y < ty + 8; ++y) {
                for (x = tx; x < tx + 8; x += 2) {
                    const uint8_t *hi = rgba + ((size_t) y * (size_t) w + (size_t) x) * 4u;
                    const uint8_t *lo = rgba + ((size_t) y * (size_t) w + (size_t) (x + 1)) * 4u;
                    *out++ = (uint8_t) ((uigen_i4_level(hi) << 4) | uigen_i4_level(lo));
                }
            }
        }
    }
}

/* Guest-pointer variants. The guest buffer is a MEM1 address the game allocates (HSD_MemAlloc) and
 * hands to GX; the bytes themselves are byte-order-neutral, so the exact same tiling applies and
 * the write is plain byte stores - never gw_w* (which would byte-swap) and never a reinterpret of
 * the guest pointer. Delegating to the host rasterizers keeps one source of truth for the tiling
 * (png2gx.py's), so the golden tests above remain the only oracle. */
void gw_UI_RasterRgba8Guest(int w, int h, const uint8_t *rgba_host, void *guest_dst) {
    gw_UI_RasterRgba8(w, h, rgba_host, guest_dst);
}

void gw_UI_RasterI4Guest(int w, int h, const uint8_t *rgba_host, void *guest_dst) {
    gw_UI_RasterI4(w, h, rgba_host, guest_dst);
}

/* The runtime UI-generation experiment flag. Game code cannot call getenv (no gw_getenv shim), so
 * gmGenUI_Active() in the game TU delegates here. */
int gw_GenUI_Active(void) {
    const char *v = getenv("MELEE_UIGEN");
    return v != NULL && strcmp(v, "0") != 0;
}

/* MELEE_NATIVE_CSS=1: VS mode keeps the native character / stage select instead of the port's
 * own (gmfrontend_select.inc) - for comparison while the new ones prove themselves. */
int gw_Frontend_NativeSelect(void) {
    const char *v = getenv("MELEE_NATIVE_CSS");
    return v != NULL && v[0] != '\0' && strcmp(v, "0") != 0;
}

/* ---- content enumeration ------------------------------------------------------------------ */

int gw_UI_FighterCount(void) {
    return gw_Mex_CssIconCount();
}

int gw_UI_FighterAt(int i, GwUiFighter *out) {
    const uint8_t *icons;
    int n, ext, internal;
    const char *pl;

    if (out == NULL) {
        return 0;
    }
    out->external_id = -1;
    out->kind = -1;
    out->has_file = 0;

    n = gw_Mex_CssIconCount();
    icons = (const uint8_t *) gw_Mex_CssIconTable();
    if (icons == NULL || i < 0 || i >= n) {
        return 0;
    }
    ext = (int) icons[(uint32_t) i * GW_UI_CSS_STRIDE + GW_UI_CSS_EXT_OFF];
    out->external_id = ext;
    out->kind = gw_Mex_ExtToPortCKind(ext);
    /* has_file: the fighter's character file (PlXx.dat) is present on this disc. A fighter the
     * port has a CharacterKind for but whose file the disc does not carry is not drawable. */
    internal = gw_Mex_InternalForExt(ext);
    if (internal >= 0 && (pl = gw_Mex_FtPlFile(internal)) != NULL) {
        out->has_file = gw_DVDConvertPathToEntrynum(pl) >= 0;
    }
    return 1;
}

int gw_UI_StageCount(void) {
    return gw_Mex_SssIconCount();
}

/* A vanilla stage (grkind < GW_MEX_GR_FIRST_NEW) ships on every retail disc, so its file is
 * always present; an m-ex-added stage is present only when gw_Mex_GrIsMex finds its file. */
static int uigen_stage_has_file(int grkind) {
    if (grkind < 0) {
        return 0;
    }
    if (grkind < GW_MEX_GR_FIRST_NEW) {
        return 1;
    }
    return gw_Mex_GrIsMex(grkind);
}

int gw_UI_StageAt(int i, GwUiStage *out) {
    const uint8_t *tbl;
    int n, ext;

    if (out == NULL) {
        return 0;
    }
    out->external_id = -1;
    out->grkind = -1;
    out->has_file = 0;

    n = gw_Mex_SssIconCount();
    tbl = (const uint8_t *) gw_Mex_SssTable();
    if (tbl == NULL || i < 0 || i >= n) {
        return 0;
    }
    ext = (int32_t) gw_r32(tbl + (uint32_t) i * GW_UI_SSS_STRIDE + GW_UI_SSS_EXT_OFF);
    out->external_id = ext;
    out->grkind = gw_Mex_GrKindForExt(ext);
    out->has_file = uigen_stage_has_file(out->grkind);
    return 1;
}

/* ---- tests ------------------------------------------------------------------------------- */

/* The golden pattern, defined once here and reproduced in BOTH languages by the generator script
 * (python3 + PIL -> png2gx.py) that produced kGoldRgba8/kGoldI4 below:
 *
 *     r = (x * 37) & 255
 *     g = (y * 53) & 255
 *     b = ((x + y) * 29) & 255
 *     a = ((x * 3 + y * 5) % 4 == 0) ? 255 : ((x * 16 + y * 8) & 255)
 *
 * It exercises both mask paths (alpha < 255 -> mask via alpha; alpha == 255 -> mask via luma)
 * and, at 8x8, crosses the RGBA8 4x4 tile boundary. */
static void uigen_gold_fill(uint8_t rgba[8 * 8 * 4]) {
    int x, y;
    for (y = 0; y < 8; ++y) {
        for (x = 0; x < 8; ++x) {
            uint8_t *p = &rgba[(y * 8 + x) * 4];
            p[0] = (uint8_t) ((x * 37) & 255);
            p[1] = (uint8_t) ((y * 53) & 255);
            p[2] = (uint8_t) (((x + y) * 29) & 255);
            p[3] = (((x * 3 + y * 5) % 4) == 0) ? 255 : (uint8_t) ((x * 16 + y * 8) & 255);
        }
    }
}

/* Golden RGBA8 payload for the 8x8 pattern above, emitted by the real png2gx.py:
 *
 *   python3 pc/tools/png2gx.py gold.png gold_rgba8.gxtex --format rgba8
 *
 * Image bytes are the .gxtex payload (header 64 bytes; image starts at offset 64, length at
 * header offset 28 big-endian). fmt=6, 8x8, 256 bytes. */
static const unsigned char kGoldRgba8[] = {
    0xFF,0x00,0x10,0x25,0x20,0x4A,0x30,0x6F,0x08,0x00,0xFF,0x25,
    0x28,0x4A,0x38,0x6F,0x10,0x00,0x20,0x25,0xFF,0x4A,0x40,0x6F,
    0x18,0x00,0x28,0x25,0x38,0x4A,0xFF,0x6F,0x00,0x00,0x00,0x1D,
    0x00,0x3A,0x00,0x57,0x35,0x1D,0x35,0x3A,0x35,0x57,0x35,0x74,
    0x6A,0x3A,0x6A,0x57,0x6A,0x74,0x6A,0x91,0x9F,0x57,0x9F,0x74,
    0x9F,0x91,0x9F,0xAE,0xFF,0x94,0x50,0xB9,0x60,0xDE,0x70,0x03,
    0x48,0x94,0xFF,0xB9,0x68,0xDE,0x78,0x03,0x50,0x94,0x60,0xB9,
    0xFF,0xDE,0x80,0x03,0x58,0x94,0x68,0xB9,0x78,0xDE,0xFF,0x03,
    0x00,0x74,0x00,0x91,0x00,0xAE,0x00,0xCB,0x35,0x91,0x35,0xAE,
    0x35,0xCB,0x35,0xE8,0x6A,0xAE,0x6A,0xCB,0x6A,0xE8,0x6A,0x05,
    0x9F,0xCB,0x9F,0xE8,0x9F,0x05,0x9F,0x22,0xFF,0x00,0x30,0x25,
    0x40,0x4A,0x50,0x6F,0x28,0x00,0xFF,0x25,0x48,0x4A,0x58,0x6F,
    0x30,0x00,0x40,0x25,0xFF,0x4A,0x60,0x6F,0x38,0x00,0x48,0x25,
    0x58,0x4A,0xFF,0x6F,0xD4,0x74,0xD4,0x91,0xD4,0xAE,0xD4,0xCB,
    0x09,0x91,0x09,0xAE,0x09,0xCB,0x09,0xE8,0x3E,0xAE,0x3E,0xCB,
    0x3E,0xE8,0x3E,0x05,0x73,0xCB,0x73,0xE8,0x73,0x05,0x73,0x22,
    0xFF,0x94,0x70,0xB9,0x80,0xDE,0x90,0x03,0x68,0x94,0xFF,0xB9,
    0x88,0xDE,0x98,0x03,0x70,0x94,0x80,0xB9,0xFF,0xDE,0xA0,0x03,
    0x78,0x94,0x88,0xB9,0x98,0xDE,0xFF,0x03,0xD4,0xE8,0xD4,0x05,
    0xD4,0x22,0xD4,0x3F,0x09,0x05,0x09,0x22,0x09,0x3F,0x09,0x5C,
    0x3E,0x22,0x3E,0x3F,0x3E,0x5C,0x3E,0x79,0x73,0x3F,0x73,0x5C,
    0x73,0x79,0x73,0x96
};

/* Golden I4 payload for the same 8x8 pattern, emitted by:
 *
 *   python3 pc/tools/png2gx.py gold.png gold_i4.gxtex --format i4
 *
 * fmt=0, 8x8, 32 bytes (one 8x8 tile). */
static const unsigned char kGoldI4[] = {
    0x01,0x23,0x35,0x67,0x03,0x23,0x46,0x67,0x12,0x64,0x56,0x98,
    0x12,0x39,0x56,0x76,0x83,0x45,0xB7,0x88,0x22,0x45,0x64,0x89,
    0x34,0x56,0x78,0x79,0x34,0x56,0x78,0x95
};

static int uigen_memcmp_report(const unsigned char *got, const unsigned char *want, size_t n,
                               const char *what) {
    size_t i;
    for (i = 0; i < n; ++i) {
        if (got[i] != want[i]) {
            gw_test_fail("%s: first differing byte at offset %u - raster=0x%02X golden=0x%02X",
                         what, (unsigned) i, (unsigned) got[i], (unsigned) want[i]);
            return 1;
        }
    }
    return 0;
}

static int test_uigen_rgba8_golden(void) {
    uint8_t rgba[8 * 8 * 4];
    uint8_t out[sizeof kGoldRgba8];
    uigen_gold_fill(rgba);
    if (gw_UI_Rgba8Size(8, 8) != (int) sizeof kGoldRgba8) {
        gw_test_fail("gw_UI_Rgba8Size(8,8)=%d, expected %d", gw_UI_Rgba8Size(8, 8),
                     (int) sizeof kGoldRgba8);
        return 1;
    }
    gw_UI_RasterRgba8(8, 8, rgba, out);
    return uigen_memcmp_report(out, kGoldRgba8, sizeof kGoldRgba8, "uigen_rgba8");
}

static int test_uigen_i4_golden(void) {
    uint8_t rgba[8 * 8 * 4];
    uint8_t out[sizeof kGoldI4];
    uigen_gold_fill(rgba);
    if (gw_UI_I4Size(8, 8) != (int) sizeof kGoldI4) {
        gw_test_fail("gw_UI_I4Size(8,8)=%d, expected %d", gw_UI_I4Size(8, 8),
                     (int) sizeof kGoldI4);
        return 1;
    }
    gw_UI_RasterI4(8, 8, rgba, out);
    return uigen_memcmp_report(out, kGoldI4, sizeof kGoldI4, "uigen_i4");
}

/* Disc-dependent: the enumeration must agree with the m-ex runtime's own counts and every entry
 * must round-trip its index spaces. Skips cleanly with no ISO (or a retail disc). */
static int test_uigen_enumeration(void) {
    int n_stages, n_fighters, i, reachable;

    if (gw_iso_path() == NULL) {
        gw_log("uigen: no disc - skipping content enumeration");
        return 0;
    }
    n_stages = gw_UI_StageCount();
    n_fighters = gw_UI_FighterCount();
    if (n_stages == 0 && n_fighters == 0) {
        gw_log("uigen: no m-ex CSS/SSS on this disc - skipping content enumeration");
        return 0;
    }

    if (n_stages != gw_Mex_SssIconCount()) {
        gw_test_fail("gw_UI_StageCount()=%d but gw_Mex_SssIconCount()=%d", n_stages,
                     gw_Mex_SssIconCount());
        return 1;
    }
    if (n_stages < 30) {
        gw_test_fail("sss icon count %d is fewer than retail's 30", n_stages);
        return 1;
    }
    reachable = 0;
    for (i = 0; i < n_stages; ++i) {
        GwUiStage s;
        if (!gw_UI_StageAt(i, &s)) {
            gw_test_fail("gw_UI_StageAt(%d) failed", i);
            return 1;
        }
        if (s.external_id < 0) {
            gw_test_fail("stage %d has external_id %d", i, s.external_id);
            return 1;
        }
        if (s.grkind >= 0 && s.has_file) {
            reachable++;
        }
    }
    if (reachable < 30) {
        gw_test_fail("only %d of %d stage icons resolve to grkind>=0 with a file", reachable,
                     n_stages);
        return 1;
    }

    if (n_fighters != gw_Mex_CssIconCount()) {
        gw_test_fail("gw_UI_FighterCount()=%d but gw_Mex_CssIconCount()=%d", n_fighters,
                     gw_Mex_CssIconCount());
        return 1;
    }
    for (i = 0; i < n_fighters; ++i) {
        GwUiFighter f;
        if (!gw_UI_FighterAt(i, &f)) {
            gw_test_fail("gw_UI_FighterAt(%d) failed", i);
            return 1;
        }
        if (f.kind != gw_Mex_ExtToPortCKind(f.external_id)) {
            gw_test_fail("fighter %d kind %d != gw_Mex_ExtToPortCKind(%d)=%d", i, f.kind,
                         f.external_id, gw_Mex_ExtToPortCKind(f.external_id));
            return 1;
        }
        if (f.kind >= 0 && gw_Mex_PortCKindToExt(f.kind) < 0) {
            gw_test_fail("fighter %d external %d -> kind %d, but gw_Mex_PortCKindToExt(%d) is -1",
                         i, f.external_id, f.kind, f.kind);
            return 1;
        }
    }
    gw_log("test uigen_enumeration: %d fighters, %d stages (%d with a file)", n_fighters,
           n_stages, reachable);
    return 0;
}

void gw_uigen_tests_register(void) {
    gw_test_register("uigen_rgba8_golden", test_uigen_rgba8_golden);
    gw_test_register("uigen_i4_golden", test_uigen_i4_golden);
    gw_test_register("uigen_enumeration", test_uigen_enumeration);
}
