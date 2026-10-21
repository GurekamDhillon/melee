/* Driver for the night-3 round-2 leaf oracle (gen_leaves_r2_ref.sh). Output, all hex:
 *   G x y maxX minX maxY minY | greens oldpupupu     grGreens_80213AB4 / grOldPupupu_8021128C
 *   S v | r                                           grStadium_ScaleColor
 *   D ax ay az bx by bz | r                           grMc_DistanceSquared
 *   F a b | a-b fabs(a)                               grZebes_Subtract / grInishie1 fabsf_inline
 *   K xC6 | kongo oldkongo                            grKongo_801D8270 / grOldKongo_802105AC
 *   N num | strlen bytes...                           DevText_NumToStr + DevText_StrLen
 *   C val max | r                                     DevText_Clamp
 *   I coins pos | count divisor digit                 ifStock_802F89F8_{count,divisor,digit}
 *   Q f | r                                           ifnametag inlineA1
 *   O flag a b | r flag'                              ifstatus foo
 * tests/leaves_r2_vs_c.rs replays them. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct { float x, y, z; } Vec3;
typedef struct { uint8_t pad[0x28]; void* user_data; } HSD_GObj;
typedef struct { uint8_t pad[0xC4]; int16_t xC4; int16_t xC6; } Ground;
extern int8_t ifStatus_804D6D61;
int grGreens_80213AB4(Vec3*, float, float, float, float);
int grOldPupupu_8021128C(Vec3*, float, float, float, float);
uint8_t grStadium_ScaleColor(uint8_t);
float grMc_DistanceSquared(const Vec3*, const Vec3*);
float grZebes_Subtract(float, float);
float fabsf_inline(float);
void grKongo_801D8270(HSD_GObj*);
void grOldKongo_802105AC(HSD_GObj*);
int DevText_StrLen(char*);
void DevText_NumToStr(int, char*);
int DevText_Clamp(int, int);
int ifStock_802F89F8_count(int);
int ifStock_802F89F8_divisor(int);
int ifStock_802F89F8_digit(int, int);
float inlineA1(float);
float foo(float, float);
/* math_ppc.h sqrtf references it; nothing here calls sqrtf. */
float __frsqrte(float x) { return x; }

static uint32_t rng = 0x2545F491u;
static uint32_t nx(void) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; return rng; }
static float rf(float lo, float hi) { return lo + (hi - lo) * (float) (nx() >> 8) / 16777216.0f; }
static uint32_t b(float f) { uint32_t u; memcpy(&u, &f, 4); return u; }
static float fb(uint32_t u) { float f; memcpy(&f, &u, 4); return f; }
static float pick(float lo, float hi)
{
    switch (nx() % 24) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return fb(0x7FC00000u);
    case 3: return fb(0x7F800000u);
    case 4: return fb(0xFF800000u);
    case 5: return fb(nx() & 0x807FFFFFu); /* denormal */
    case 6: return fb(nx());
    case 7: return (float) (int) rf(lo, hi);
    default: return rf(lo, hi);
    }
}
static int pint(void)
{
    switch (nx() % 8) {
    case 0: return 0;
    case 1: return (int) 0x80000000u;
    case 2: return 0x7FFFFFFF;
    case 3: return (int) nx();
    case 4: return -(int) (nx() % 1000);
    default: return (int) (nx() % 100000);
    }
}
int main(void)
{
    int n, i;
    for (n = 0; n < 20000; n++) {
        Vec3 v;
        float a = pick(-200, 200), c = pick(-200, 200), d = pick(-200, 200), e = pick(-200, 200);
        v.x = (n % 5 == 0) ? a : pick(-200, 200);
        v.y = (n % 7 == 0) ? e : pick(-200, 200);
        printf("G %08x %08x %08x %08x %08x %08x | %x %x\n", b(v.x), b(v.y), b(a), b(c), b(d), b(e),
               grGreens_80213AB4(&v, a, c, d, e), grOldPupupu_8021128C(&v, a, c, d, e));
    }
    for (n = 0; n < 256; n++)
        printf("S %x | %x\n", n, grStadium_ScaleColor((uint8_t) n));
    for (n = 0; n < 20000; n++) {
        Vec3 p = { pick(-1e4f, 1e4f), pick(-1e4f, 1e4f), pick(-1e4f, 1e4f) };
        Vec3 q = { pick(-1e4f, 1e4f), pick(-1e4f, 1e4f), pick(-1e4f, 1e4f) };
        if (n % 3 == 0) q.x = p.x + rf(-1, 1), q.y = p.y + rf(-1, 1), q.z = p.z + rf(-1, 1);
        printf("D %08x %08x %08x %08x %08x %08x | %08x\n", b(p.x), b(p.y), b(p.z), b(q.x), b(q.y),
               b(q.z), b(grMc_DistanceSquared(&p, &q)));
    }
    for (n = 0; n < 5000; n++) {
        float x = pick(-1e3f, 1e3f), y = pick(-1e3f, 1e3f);
        printf("F %08x %08x | %08x %08x\n", b(x), b(y), b(grZebes_Subtract(x, y)), b(fabsf_inline(x)));
    }
    for (n = -4; n < 8; n++) {
        Ground g1, g2;
        HSD_GObj o1, o2;
        int16_t k = (int16_t) (n < 6 ? n : (int) nx());
        memset(&g1, 0, sizeof g1), memset(&g2, 0, sizeof g2);
        g1.xC6 = g2.xC6 = k;
        o1.user_data = &g1, o2.user_data = &g2;
        grKongo_801D8270(&o1);
        grOldKongo_802105AC(&o2);
        printf("K %04x | %04x %04x\n", (uint16_t) k, (uint16_t) g1.xC6, (uint16_t) g2.xC6);
    }
    for (n = 0; n < 5000; n++) {
        char s[16];
        int num = pint(), len;
        memset(s, 0x55, sizeof s);
        DevText_NumToStr(num, s);
        len = DevText_StrLen(s);
        printf("N %08x | %x", (uint32_t) num, len);
        for (i = 0; i < 13; i++) printf(" %02x", (uint8_t) s[i]);
        printf("\n");
    }
    printf("N - | %x\n", DevText_StrLen(NULL));
    for (n = 0; n < 5000; n++) {
        int val = pint(), max = pint();
        printf("C %08x %08x | %08x\n", (uint32_t) val, (uint32_t) max, (uint32_t) DevText_Clamp(val, max));
    }
    for (n = 0; n < 5000; n++) {
        int coins = pint(), pos = (int) (nx() % 32);
        printf("I %08x %08x | %08x %08x %08x\n", (uint32_t) coins, (uint32_t) pos,
               (uint32_t) ifStock_802F89F8_count(coins), (uint32_t) ifStock_802F89F8_divisor(pos),
               (uint32_t) ifStock_802F89F8_digit(coins, pos));
    }
    for (n = 0; n < 5000; n++) {
        float f = (n < 64) ? (float) (n - 16) + 0.5f : pick(-3e9f, 3e9f);
        printf("Q %08x | %08x\n", b(f), b(inlineA1(f)));
    }
    for (n = 0; n < 5000; n++) {
        int8_t fl = (int8_t) (n % 4 == 3 ? (int) nx() : n % 2);
        float x = pick(-1e3f, 1e3f), y = pick(-1e3f, 1e3f), r;
        ifStatus_804D6D61 = fl;
        r = foo(x, y);
        printf("O %02x %08x %08x | %08x %02x\n", (uint8_t) fl, b(x), b(y), b(r), (uint8_t) ifStatus_804D6D61);
    }
    return 0;
}
