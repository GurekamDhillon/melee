/* TU: src/melee/if/textlib.c */
/* C oracle for src/tu/textlib.rs: the whole TU included verbatim; every callee out of it is a
 * recording stub, except the libc-like memcpy / memzero / vsnprintf, which are real. Each line:
 * op case ret state-hash trace-hash. The glyph byte DevTextGlyph { color:2; unk:6 } is compared in
 * the GameCube bit order (color on top) that the Rust port uses; the native build keeps it low. */
#define memcpy mtu_memcpy
#define vsnprintf mtu_vsnprintf
#include TU_SRC
#include "mtu_common.h"
DevText* devtext_drawlist;
DevText* devtext_poolhead;
struct un_80304138_objalloc_t* un_804D6E44;
static DevText P[8];
static char BUFS[8][192];
static char STR[128];
static char LONGS[64];
static struct un_80304138_objalloc_t MENU;
static int cbx(enum soundtest_callback_arg0 a) { (void) a; return 0; }
void* memcpy(void* d, const void* s, size_t n)
{
    unsigned char* a = d;
    const unsigned char* b = s;
    size_t i;
    for (i = 0; i < n; i++) a[i] = b[i];
    return d;
}
void memzero(void* p, int n)
{
    unsigned char* a = p;
    int i;
    for (i = 0; i < n; i++) a[i] = 0;
}
extern int __stdio_common_vsprintf(unsigned long long, char*, unsigned long long, const char*, void*, va_list);
int mtu_vsnprintf(char* b, size_t n, const char* f, va_list a)
{
    return __stdio_common_vsprintf(2ull, b, (unsigned long long) n, f, 0, a);
}
#include "textlib_gen.h"

static u32 gcb(u32 b) { return ((b & 3) << 6) | (b >> 2); }

static void setup(void)
{
    int i, j, nd, nf, perm[8];
    DevText* prev;
    zero(P, sizeof P);
    for (i = 0; i < 8; i++) {
        fill_DevText(&P[i]);
        P[i].w = 1 + rnd() % 10;
        P[i].h = 1 + rnd() % 6;
        P[i].cursor_x = rnd() % P[i].w;
        P[i].cursor_y = rnd() % P[i].h;
        P[i].id = (s8) ((int) (rnd() % 7) - 2);
        P[i].buf = (DevTextGlyph*) BUFS[i];
        for (j = 0; j < 192; j++) BUFS[i][j] = (rnd() % 4 == 0) ? 0 : (char) rnd();
        P[i].prev = 0;
        P[i].next = 0;
    }
    for (i = 0; i < 128; i++) STR[i] = (rnd() % 9 == 0) ? 0 : (char) (rnd() % 94 + 33);
    STR[127] = 0;
    for (i = 0; i < 40; i++) STR[i] = (rnd() % 5 == 0) ? '\n' : STR[i];
    for (i = 0; i < 50; i++) LONGS[i] = (char) ('a' + i % 26);
    LONGS[50] = 0;
    for (i = 0; i < 8; i++) perm[i] = i;
    for (i = 7; i > 0; i--) { j = (int) (rnd() % (u32) (i + 1)); { int t = perm[i]; perm[i] = perm[j]; perm[j] = t; } }
    nd = (int) (rnd() % 4);
    nf = 1 + (int) (rnd() % 3);
    devtext_drawlist = 0;
    devtext_poolhead = 0;
    prev = 0;
    for (i = 0; i < nd; i++) {
        DevText* t = &P[perm[i]];
        t->prev = prev;
        if (prev) prev->next = t; else devtext_drawlist = t;
        prev = t;
    }
    prev = 0;
    for (i = 0; i < nf; i++) {
        DevText* t = &P[perm[nd + i]];
        t->prev = prev;
        if (prev) prev->next = t; else devtext_poolhead = t;
        prev = t;
    }
    zero(&MENU, sizeof MENU);
    un_804D6E44 = (rnd() & 1) ? &MENU : 0;
}

static void hash_all(void)
{
    int i, j;
    for (i = 0; i < 8; i++) {
        hash_DevText(&P[i]);
        for (j = 0; j < 192; j++) HM(((j & 1) ? gcb((unsigned char) BUFS[i][j]) : (unsigned char) BUFS[i][j]));
    }
    HM(PI(devtext_drawlist)); HM(PI(devtext_poolhead)); HM(PI(un_804D6E44));
    HM(FI((const void*) MENU.xC));
    HM(RH(STR, sizeof STR));
}

int main(void)
{
    int op, k, i;
    for (i = 0; i < 8; i++) preg(&P[i], sizeof(DevText));
    preg(&MENU, sizeof MENU);
    freg_all();
    freg((const void*) cbx);
    for (op = 0; op < 21; op++) {
        for (k = 0; k < 1200; k++) {
            u32 ret = 0;
            DevText* t;
            tu_case(op, k);
            setup();
            t = &P[rnd() % 8];
            switch (op) {
            case 0: {
                int id = (int) (rnd() % 8) - 3, x = (int) fv(), y = (int) fv(), w = 1 + (int) (rnd() % 10), h = 1 + (int) (rnd() % 6);
                DevText* r = DevText_Create((char) id, x, y, w, h, BUFS[rnd() % 8]);
                ret = PI(r);
                break;
            }
            case 1: DevText_EraseFirstLine(t); break;
            case 2: { int x = (int) (rnd() % 30) - 8, y = (int) (rnd() % 20) - 6; DevText_SetCursorXY(t, x, y); break; }
            case 3: { int x = (int) (rnd() % 30) - 8; DevText_SetCursorX(t, x); break; }
            case 4: DevText_HideCursor(t); break;
            case 5: DevText_80302AC0(t); break;
            case 6: DevText_ShowBackground(t); break;
            case 7: DevText_HideBackground(t); break;
            case 8: DevText_ShowText(t); break;
            case 9: DevText_HideText(t); break;
            case 10: { float x = rw(), y = rw(); DevText_SetScale(t, x, y); break; }
            case 11: { int x = (int) fv() - 20, y = (int) fv() - 20; DevText_SetXY(t, x, y); break; }
            case 12: ret = DevText_StoreColorIndex(t, (u8) rnd()); break;
            case 13: {
                GXColor c, o;
                c.r = (u8) rnd(); c.g = (u8) rnd(); c.b = (u8) rnd(); c.a = (u8) rnd();
                o = DevText_SetTextColor(t, c);
                ret = ((u32) o.r << 24) | ((u32) o.g << 16) | ((u32) o.b << 8) | o.a;
                break;
            }
            case 14: {
                GXColor c, o;
                c.r = (u8) rnd(); c.g = (u8) rnd(); c.b = (u8) rnd(); c.a = (u8) rnd();
                o = DevText_SetBGColor(t, c);
                ret = ((u32) o.r << 24) | ((u32) o.g << 16) | ((u32) o.b << 8) | o.a;
                break;
            }
            case 15: DevText_Erase(t); break;
            case 16: {
                char* s = (rnd() % 8 == 0) ? 0 : STR + rnd() % 10;
                t->flags = (rnd() & 1) ? (t->flags | DEVTEXT_FLAG_NOWRAP) : (t->flags & ~DEVTEXT_FLAG_NOWRAP);
                DevText_Print(t, s);
                break;
            }
            case 17: DevText_PrintInt(t, (int) fv()); break;
            case 18: {
                int a = (int) fv();
                int n1 = (int) (rnd() % 20001) - 10000;
                float f = (float) n1 / 4.0f;
                switch (rnd() % 10) {
                case 0: DevText_Printf(t, "%d", a); break;
                case 1: DevText_Printf(t, "%02x", (int) (u8) a); break;
                case 2: DevText_Printf(t, "%04x", (int) (u16) a); break;
                case 3: DevText_Printf(t, "%08x", a); break;
                case 4: DevText_Printf(t, "%3.2f", (double) f); break;
                case 5: DevText_Printf(t, "[%s|%5d|%-4x|%c]", STR + 1, a, a, 'A' + (int) (rnd() % 26)); break;
                case 6: DevText_Printf(t, "%s%s", LONGS, LONGS); break;
                case 7: DevText_Printf(t, "%+d %u %X %o %#x", a, a, a, a, a); break;
                case 8: DevText_Printf(t, "%10.3f|%-9.1f|%.0f %%", (double) ((float) n1 / 8.0f), (double) ((float) (n1 / 4) / 2.0f), (double) (float) (n1 / 8)); break;
                case 9: DevText_Printf(t, "%.3s|%05d|%5.2d|% d", LONGS, a, a, a); break;
                }
                break;
            }
            case 19: ret = PI(un_80302DF0()); break;
            case 20: { soundtest_callback cbn = (rnd() & 1) ? cbx : 0; un_80302DF8(&MENU, cbn); } break;
            }
            hash_all();
            printf("%d %d %08X %08X %08X\n", op, k, ret, H, TR);
        }
    }
    return 0;
}
