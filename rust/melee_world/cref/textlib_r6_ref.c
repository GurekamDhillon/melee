/* Driver for the night-3 round-6 DevText oracle (gen_textlib_r6_ref.sh). Each row, all hex:
 *   T op a0 a1 a2 | <state before> | <state after> ret
 * state = x y w h cx cy sx sy bg c0 c1 c2 c3 id lw flags b27 buf(bytes, no spaces)
 * ops: 0 EraseFirstLine, 1 SetCursorXY(a0,a1), 2 SetCursorX(a0), 3..8 the six flag fns,
 *   9 SetScale(a0,a1 bits), 10 SetXY(a0,a1), 11 StoreColorIndex(a0), 12 SetTextColor(a0 rgba),
 *   13 SetBGColor(a0 rgba), 14 Erase, 15 AdvanceLine, 16 Print(string id a0, a1 = len),
 *   17 PrintInt(a0).  ret is the return value (colours as 0xRRGGBBAA), else 0.
 *   P op string bytes follow in the S rows: S id | bytes  (id 0 = NULL).
 *   A rgba | rgba'              textlib_1 adjust
 *   M x y z m | x' y' z'        grpstadium mul_vec
 * tests/textlib_r6_vs_c.rs replays them. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct { uint8_t r, g, b, a; } GXColor;
typedef struct { float x, y, z; } Vec;
typedef struct DevText {
    int16_t x, y; uint8_t w, h, cursor_x, cursor_y; float scale_x, scale_y;
    GXColor bg_color; GXColor text_colors[4]; int8_t id; uint8_t line_width, flags;
    uint8_t unk : 6; uint8_t current_color : 2;
    char* buf; struct DevText *prev, *next;
} DevText;
void DevText_EraseFirstLine(DevText*);
void DevText_SetCursorXY(DevText*, int, int);
void DevText_SetCursorX(DevText*, int);
void DevText_HideCursor(DevText*);
void DevText_80302AC0(DevText*);
void DevText_ShowBackground(DevText*);
void DevText_HideBackground(DevText*);
void DevText_ShowText(DevText*);
void DevText_HideText(DevText*);
void DevText_SetScale(DevText*, float, float);
void DevText_SetXY(DevText*, int, int);
uint8_t DevText_StoreColorIndex(DevText*, uint8_t);
GXColor DevText_SetTextColor(DevText*, GXColor);
GXColor DevText_SetBGColor(DevText*, GXColor);
void DevText_Erase(DevText*);
void DevText_Print(DevText*, char*);
void DevText_PrintInt(DevText*, int);
GXColor r6_adjust(GXColor);
void r6_mul_vec(Vec*, float);
void r6_advance(DevText*);
float __frsqrte(float x) { return x; }

static uint32_t rng = 0x9E3779B9u;
static uint32_t nx(void) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; return rng; }
static uint32_t fbits(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float bf(uint32_t u) { float f; memcpy(&f, &u, 4); return f; }
static uint32_t cbits(GXColor c) { return (uint32_t) c.r << 24 | c.g << 16 | c.b << 8 | c.a; }
static GXColor bc(uint32_t u) { GXColor c = { u >> 24, u >> 16, u >> 8, u }; return c; }
static int pint(int lo, int hi)
{
    switch (nx() % 10) {
    case 0: return (int) nx();
    case 1: return (int) 0x80000000u;
    case 2: return 0x7FFFFFFF;
    case 3: return 0x100 + (int) (nx() % 8);
    case 4: return -0x100 - (int) (nx() % 8);
    default: return lo + (int) (nx() % (uint32_t) (hi - lo + 1));
    }
}
#define BUFN 256
static char buf[BUFN];
static void dump(DevText* t)
{
    int i, n = 2 * t->w * t->h;
    uint8_t b27; memcpy(&b27, (char*) t + 0x27, 1);
    if (n < 2 * t->w) n = 2 * t->w;
    printf("%x %x %x %x %x %x %08x %08x %08x %08x %08x %08x %08x %x %x %x %x ", (uint16_t) t->x, (uint16_t) t->y,
           t->w, t->h, t->cursor_x, t->cursor_y, fbits(t->scale_x), fbits(t->scale_y), cbits(t->bg_color),
           cbits(t->text_colors[0]), cbits(t->text_colors[1]), cbits(t->text_colors[2]), cbits(t->text_colors[3]),
           (uint8_t) t->id, t->line_width, t->flags, b27);
    for (i = 0; i < n; i++) printf("%02x", (uint8_t) buf[i]);
}
static const char charset[] = "AZaz09 !~\n\n\x7f\x80\xff";
static char strs[64][32];
int main(void)
{
    int n, i, s;
    _Static_assert(sizeof(DevText) == 0x28 + 3 * sizeof(void*), "layout");
    for (s = 1; s < 64; s++) {
        int len = nx() % 31;
        for (i = 0; i < len; i++) strs[s][i] = (nx() % 4) ? 'A' + nx() % 26 : charset[nx() % (sizeof charset - 1)];
        strs[s][len] = 0;
        printf("S %x |", s);
        for (i = 0; i <= len; i++) printf(" %02x", (uint8_t) strs[s][i]);
        printf("\n");
    }
    for (n = 0; n < 60000; n++) {
        DevText t; int op = nx() % 18, a0 = 0, a1 = 0, a2 = 0; uint32_t ret = 0;
        memset(&t, 0, sizeof t);
        t.x = nx(); t.y = nx();
        t.w = (nx() % 16 == 0) ? 0 : 1 + nx() % 10;
        t.h = (nx() % 16 == 0) ? 0 : 1 + nx() % 6;
        if ((op == 15 || op == 16 || op == 17) && (t.w == 0 || t.h == 0)) { t.w = 1 + nx() % 10; t.h = 1 + nx() % 6; }
        t.cursor_x = t.w ? nx() % t.w : 0;
        t.cursor_y = t.h ? nx() % t.h : 0;
        t.scale_x = bf(nx()); t.scale_y = bf(nx());
        t.bg_color = bc(nx());
        for (i = 0; i < 4; i++) t.text_colors[i] = bc(nx());
        t.id = nx(); t.line_width = nx(); t.flags = nx();
        { uint8_t b = nx(); memcpy((char*) &t + 0x27, &b, 1); }
        for (i = 0; i < BUFN; i++) buf[i] = nx();
        t.buf = buf;
        switch (op) {
        case 1: case 10: a0 = pint(-3, 12); a1 = pint(-3, 12); break;
        case 2: a0 = pint(-3, 12); break;
        case 9: a0 = nx(); a1 = nx(); break;
        case 11: a0 = nx() & 0xFF; break;
        case 12: case 13: a0 = nx(); break;
        case 16: a0 = nx() % 64; break;
        case 17: a0 = pint(-100000, 100000); break;
        }
        printf("T %x %x %x %x | ", op, (uint32_t) a0, (uint32_t) a1, (uint32_t) a2);
        dump(&t);
        switch (op) {
        case 0: DevText_EraseFirstLine(&t); break;
        case 1: DevText_SetCursorXY(&t, a0, a1); break;
        case 2: DevText_SetCursorX(&t, a0); break;
        case 3: DevText_HideCursor(&t); break;
        case 4: DevText_80302AC0(&t); break;
        case 5: DevText_ShowBackground(&t); break;
        case 6: DevText_HideBackground(&t); break;
        case 7: DevText_ShowText(&t); break;
        case 8: DevText_HideText(&t); break;
        case 9: DevText_SetScale(&t, bf(a0), bf(a1)); break;
        case 10: DevText_SetXY(&t, a0, a1); break;
        case 11: ret = DevText_StoreColorIndex(&t, (uint8_t) a0); break;
        case 12: ret = cbits(DevText_SetTextColor(&t, bc(a0))); break;
        case 13: ret = cbits(DevText_SetBGColor(&t, bc(a0))); break;
        case 14: DevText_Erase(&t); break;
        case 15: r6_advance(&t); break;
        case 16: DevText_Print(&t, a0 ? strs[a0] : NULL); break;
        case 17: DevText_PrintInt(&t, a0); break;
        }
        printf(" | ");
        dump(&t);
        printf(" %x\n", ret);
    }
    for (n = 0; n < 4096; n++) {
        uint32_t c = nx(); if (n < 256) c = n * 0x01010101u;
        printf("A %08x | %08x\n", c, cbits(r6_adjust(bc(c))));
    }
    for (n = 0; n < 20000; n++) {
        Vec v = { bf(nx()), bf(nx()), bf(nx()) }; float m = bf(nx());
        if (n % 2) { v.x = (float) (int) (nx() % 2000) - 1000; v.y = 0.37f * (n % 91); v.z = -1.5f; m = 0.01f * (n % 300); }
        printf("M %08x %08x %08x %08x | ", fbits(v.x), fbits(v.y), fbits(v.z), fbits(m));
        r6_mul_vec(&v, m);
        printf("%08x %08x %08x\n", fbits(v.x), fbits(v.y), fbits(v.z));
    }
    return 0;
}
