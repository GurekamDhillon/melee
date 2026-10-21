/* Game-side TU for the night-3 round-6 DevText oracle (gen_textlib_r6_ref.sh). Compiled with
 * the game's flags (contraction on, then lower.py) and textlib_r6_body.inc: bodies cut verbatim
 * from src/melee/if/textlib.c, textdraw.c, textlib_1.c and gr/grpstadium.c, `static` /
 * `static inline` dropped. struct DevText and the DEVTEXT_FLAG_* values are copied from
 * src/melee/if/types.h; DevTextGlyph comes from the cut (it sits between two cut functions). */
#include <Runtime/platform.h>
#include <string.h>

typedef struct { f32 x, y, z; } Vec;
typedef struct { u8 r, g, b, a; } GXColor;
#define DEVTEXT_FLAG_HIDETEXT (0x80)
#define DEVTEXT_FLAG_HIDEBACKGROUND (0x40)
#define DEVTEXT_FLAG_NOWRAP (0x20)
#define DEVTEXT_FLAG_SHOWCURSOR (0x10)
typedef struct DevText {
    /*  +0 */ s16 x;
    /*  +2 */ s16 y;
    /*  +4 */ u8 w;
    /*  +5 */ u8 h;
    /*  +6 */ u8 cursor_x;
    /*  +7 */ u8 cursor_y;
    /*  +8 */ f32 scale_x;
    /*  +C */ f32 scale_y;
    /* +10 */ GXColor bg_color;
    /* +14 */ GXColor text_colors[4];
    /* +24 */ s8 id;
    /* +25 */ u8 line_width;
    /* +26 */ u8 flags;
    /* +27 */ u8 unk : 6;
    /* +27 */ u8 current_color : 2;
    /* +28 */ char* buf;
    /* +2C */ struct DevText* prev;
    /* +30 */ struct DevText* next;
} DevText;
#define memzero(p, n) memset((p), 0, (n))
void DevText_NumToStr(int, char*);
#include "textlib_r6_body.inc"
/* Exported wrappers for the inlines whose cut name is file-local in the driver. */
GXColor r6_adjust(GXColor c) { return adjust(c); }
void r6_mul_vec(Vec* v, float m) { mul_vec(v, m); }
void r6_advance(DevText* t) { DevText_AdvanceLine(t); }
