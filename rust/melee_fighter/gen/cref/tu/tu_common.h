/* Shared harness for gen/cref/tu/<stem>.c (included after the TU). Mirrored by
 * tests/tu_common/mod.rs: same xorshift, same fill order. */
#include <sysdolphin/baselib/gobj.h>
ftCommonData* p_ftCommonData;
static int asrt_hit;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; asrt_hit = 1; }
void OSReport(char* f, ...) { (void) f; }
int printf(const char*, ...);

static u32 TR;
static int dbgT;
static void T(u32 v) { TR = TR * 31u + v; if (dbgT) { printf("T %u", v); printf("%c", 10); } }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static u32 H;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }

static u32 rs;
static u32 rnd(void) { rs ^= rs << 13; rs ^= rs >> 17; rs ^= rs << 5; return rs; }
static float rg(void)
{
    union { float f; u32 u; } c;
    switch (rnd() % 16) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: c.u = 0x7FC00000u; return c.f;
    case 3: return 1.0f;
    case 4: return -1.0f;
    default: return (float) ((int) (rnd() % 41) - 20) / 20.0f;
    }
}
static float rw(void)
{
    static const float k[4] = { 1.0f, 13.0f, 1000.0f, 100000.0f };
    float a = rg();
    return a * k[rnd() % 4];
}

static Fighter F;
static Fighter F2;
static HSD_GObj GO, GO2, IGO;
static ftCommonData CD;
static u32 SRV[16];
static u32 SR(int i) { return (SRV[i & 15] >> (i >> 4)) & 1; }

static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static void tu_init(void) { GO.user_data = &F; GO2.user_data = &F2; p_ftCommonData = &CD; }
static void tu_case(int op, int k)
{
    int i;
    zero(&F, sizeof F); zero(&F2, sizeof F2); zero(&CD, sizeof CD);
    TR = 0;
    rs = 0x9059u + (u32) op * 7919u + (u32) k * 104729u;
    for (i = 0; i < 16; i++) SRV[i] = rnd();
}
