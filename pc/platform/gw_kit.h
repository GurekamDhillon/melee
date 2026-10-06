/* gw_kit.h - the frontend kit for scripts (gd.kit in docs/scripting.md).
 *
 * The port's own menus are drawn from the art pipeline's kit (menu/ -> _build/ui): the font atlas
 * (font_manifest.json + font_<role>_latin_0.gxtex), the palette and section colours (kit.json),
 * the list row template (list_layout.json) and the art pack's textures (frame_*, ico_*, glyph_*).
 * The game-side kit (src/melee/gm/gmfrontend_kit.inc) only runs inside the frontend scenes. This
 * is the same kit read on the host side, so a script can draw it over any scene - a match
 * included - through the ImGui overlay pass that already draws gd.text / gd.fill:
 *
 *   - the same files, the same text setting rules (kerning, '?' fallback, caps-only roles, the
 *     fit rule: step down to the next smaller role of the face, then truncate with an ellipsis),
 *     the same italic shear (x' = x + (baseline - y) * 0.25);
 *   - the same TEV: an I4/I8/IA4/IA8 texture is a MASK (RGB from the tint, alpha = texture alpha
 *     x tint alpha), anything else is modulated by the tint;
 *   - the same 640x480 virtual screen as every other gd draw call.
 *
 * Nothing here touches game memory or the game heap: textures are decoded into host memory and
 * handed to ImGui, so drawing is local-only and cannot affect netplay or rollback.
 *
 * TEXTURE SEARCH (gw_Kit_Tex): the calling script mod's own ui/ folder first (`mod_ui`, see
 * gw_script.c), then MELEE_MENUTEX_DIR, then <exe>/ui, <exe>/../../ui, <exe>/../../../../ui -
 * the frontend's own search (gw_GxTex_OpenUI). A mod's ui/ may also carry *_ui.json manifests:
 * "textures": [{name, size_1x, tint}] give a texture's default draw size and tint, and
 * "palette" entries (a "#rrggbb" string, or an object with "hex" / "u32") become colour tokens
 * for that mod's scripts.
 */
#ifndef GW_KIT_H
#define GW_KIT_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* One quad of the frame's kit draw list, in the 640x480 virtual screen (corners in order
 * top-left, top-right, bottom-right, bottom-left before any shear). tex < 0: a flat quad. */
typedef struct {
    float x[4], y[4];
    float u[4], v[4];
    uint32_t rgba; /* 0xRRGGBBAA: the tint (mask colour, or the modulation) */
    int tex;       /* gw_Kit_TexInfo index, -1 = flat */
    int flags;     /* GW_KITQ_*; 0 for everything but script models */
    uint32_t vcol[4]; /* GW_KITQ_VCOL: a colour per corner (Gouraud) instead of rgba */
    float clip[4]; /* GW_KITQ_CLIP: x0, y0, x1, y1 in the same 640x480 space */
} GwKitQuad;
/* GW_KITQ_TRI: only corners 0..2 are drawn (a model triangle) */
enum { GW_KITQ_TRI = 1, GW_KITQ_VCOL = 2, GW_KITQ_CLIP = 4 };

enum { GW_KIT_ALIGN_LEFT = 0, GW_KIT_ALIGN_CENTER = 1, GW_KIT_ALIGN_RIGHT = 2 };
enum { GW_KIT_ROW_NG = 0, GW_KIT_ROW_SEL = 1, GW_KIT_ROW_DISABLED = 2 };
enum { GW_KIT_FLIP_X = 1, GW_KIT_FLIP_Y = 2 };

/* 1 when the kit's font manifest and kit.json were found and read (read once, on first use). */
int gw_Kit_Available(void);
const char *gw_Kit_Why(void); /* why not, when it is not */

/* ---- text ---------------------------------------------------------------------------------- */
int gw_Kit_RoleCount(void);
const char *gw_Kit_RoleName(int role);
int gw_Kit_Role(const char *name); /* -1 when unknown */
/* size, ascent, descent, cap height, line height of a role (1x px); 0 when unknown */
int gw_Kit_RoleMetrics(int role, float *size, float *ascent, float *descent, float *cap, float *line);
float gw_Kit_TextWidth(int role, const char *s);
/* The fit rule into `out` (the ellipsis is byte 0x01); returns the role it fits in. */
int gw_Kit_Fit(int role, const char *s, float max_w, char *out, int cap);

/* ---- colours ------------------------------------------------------------------------------- */
/* A colour token: "#rrggbb" / "#rrggbbaa", a kit palette name (ink, bone, gold, ...), a section
 * colour "@face" / "@bg" / "@band" / "@face_hi" (the Versus section) or "<section>.<which>"
 * ("solo.face"), a port "p1".."p4" / "cpu" (or "port:p1"), or a name from the mod's *_ui.json
 * palette (`mod_ui` may be NULL). Returns 1 and sets *rgba (alpha ff unless given). */
int gw_Kit_Colour(const char *tok, const char *mod_ui, uint32_t *rgba);
int gw_Kit_PaletteCount(void);
const char *gw_Kit_PaletteName(int i);
uint32_t gw_Kit_PaletteRGBA(int i);
int gw_Kit_SectionCount(void);
const char *gw_Kit_SectionName(int i);
/* which: 0 face, 1 bg, 2 band, 3 face_hi */
uint32_t gw_Kit_SectionRGBA(int section, int which);

/* ---- textures ------------------------------------------------------------------------------ */
/* Load (once) and return a texture index, -1 when no search directory has `name`.gxtex. */
int gw_Kit_Tex(const char *name, const char *mod_ui);
/* Texel size, 1x draw size (the mod manifest's size_1x, else half the texels: the kit is
 * authored at 2x), whether it is a mask, its default tint token ("" when none). */
int gw_Kit_TexInfo(int tex, int *w, int *h, float *w1x, float *h1x, int *mask, const char **tint);
/* RGBA8 pixels (w*h*4, straight alpha, masks white) for the renderer's upload. */
const uint8_t *gw_Kit_TexPixels(int tex);
int gw_Kit_TexCount(void);
/* A script model's GX RGBA8 atlas (+ optional glow atlas, added in) as a texture, cached by key;
 * -1 when it cannot be decoded or the 64 model slots are used. */
int gw_Kit_TexAddGX(const char *key, const uint8_t *img, size_t img_size, int w, int h,
                    const uint8_t *glow, size_t glow_size, int gw, int gh, int keep_alpha);

/* ---- the frame's draw list ------------------------------------------------------------------ */
void gw_Kit_BeginFrame(void); /* empties the quad list (gw_Script_Tick, before on_tick) */
int gw_Kit_QuadCount(void);
/* Drops the quads from n on: the host's own overlay (gw_console.cpp) draws after the scripts'
 * list and takes its quads back the same frame. */
void gw_Kit_TruncateQuads(int n);
const GwKitQuad *gw_Kit_QuadAt(int i);
/* the list being built (scripts draw into it) and the finished one the overlay shows; the
 * script engine swaps them when its draw list flips */
void gw_Kit_SwapBanks(void);
const GwKitQuad *gw_Kit_ShownQuadAt(int i);

/* Drawing appends quads and returns how many it added (the script's draw list records the
 * range, so kit and plain gd draws keep their order). `shear` is applied about `y` for text (the
 * baseline) and about each element's own middle otherwise. */
int gw_Kit_DrawText(float x, float y, const char *s, int role, uint32_t rgba, int align,
                    float max_w, float shear, float *out_w);
/* Wrapped text: breaks at spaces and '\n' to lines no wider than max_w, at the role's line
 * height. An overlong single word goes through the fit rule. Returns lines drawn; *out_h gets
 * the block's height. */
int gw_Kit_DrawParagraph(float x, float y, float max_w, const char *s, int role, uint32_t rgba,
                         float shear, float *out_h);
int gw_Kit_DrawImage(int tex, float x, float y, float w, float h, uint32_t rgba, int flip,
                     float shear);
int gw_Kit_DrawTri(int tex, const float x[3], const float y[3], const float u[3], const float v[3],
                   const uint32_t col[3], const float *clip);
int gw_Kit_QuadRoom(void); /* quads the bank can still take this frame */
int gw_Kit_DrawFlat(float x, float y, float w, float h, uint32_t rgba, float shear);
/* One flat quad with four arbitrary corners (top-left, top-right, bottom-right, bottom-left): the Atlas plates
 * and glyphs are built from these. Returns 1 when added, 0 when the list is full or the colour is transparent. */
int gw_Kit_DrawPoly4(const float x[4], const float y[4], uint32_t rgba);
/* Letter-spacing in px added after every glyph by gw_Kit_TextWidth, gw_Kit_Fit, gw_Kit_DrawText and the paragraph
 * (0 = none, the default). A caller sets it around its own text and resets it to 0. */
void gw_Kit_SetTracking(float px);
/* A 9-slice panel from <prefix>_corner_tl/_tr/_bl/_br, <prefix>_edge_h (top; flipped vertically
 * for the bottom), <prefix>_edge_v (left; flipped horizontally for the right) and an optional
 * <prefix>_fill. `piece` <= 0: the corner texture's 1x size (clamped to half the panel). The
 * fill: the fill texture tinted by fill_rgba when there is one, else a flat fill_rgba (skipped
 * when its alpha is 0). Missing pieces are skipped. */
int gw_Kit_DrawPanel(float x, float y, float w, float h, const char *prefix, const char *mod_ui,
                     float piece, uint32_t tint, uint32_t fill_rgba, float shear);
/* The kit's list row (list_layout.json's template): face, the gold_dk plate and lift when
 * selected, the label in the row role, an optional right-aligned value. h <= 0: the kit's
 * row height. `section` picks @face (-1: Versus). */
int gw_Kit_DrawRow(float x, float y, float w, float h, const char *label, const char *value,
                   int state, int section, float shear);
/* The kit's row metrics: height, pitch, label x and baseline, selected lift. */
void gw_Kit_RowMetrics(float *h, float *pitch, float *label_x, float *label_base, float *lift);
float gw_Kit_Shear(void); /* the kit's own shear factor (list_layout.json "shear".S) */

void gw_kit_tests_register(void);

#ifdef __cplusplus
}
#endif
#endif /* GW_KIT_H */
