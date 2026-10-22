/* Shared harness for gen/cref/tu/<stem>.c (included after the TU). Mirrored line for line by
 * rust/melee_modes/tests/mtu_harness/mod.rs: same xorshift, same fill draws, same trace hash,
 * same pointer / function ids. */
extern int printf(const char*, ...);
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
void OSReport(char* f, ...) { (void) f; }

typedef unsigned long long u64_t_;
static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void T64(double d) { union { double d; u64_t_ u; } c; c.d = d; T((u32) c.u); T((u32) (c.u >> 32)); }
static u32 H;
static void HM(u32 v) { H = H * 0x01000193u ^ v; }
static void HM64(double d) { union { double d; u64_t_ u; } c; c.d = d; HM((u32) c.u); HM((u32) (c.u >> 32)); }

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
/* integer draw: small values most often (so the game's own comparisons are taken both ways) */
static u32 fv(void)
{
    u32 r = rnd();
    switch (r & 15) {
    case 0: case 1: return 0;
    case 2: case 3: return 1;
    case 4: return 2;
    case 5: return 3;
    case 6: return (r >> 8) & 7;
    case 7: return (r >> 8) & 0xF;
    case 8: return (r >> 8) & 0xFF;
    case 9: return (r >> 8) & 0x7F;
    case 10: return 0xFF;
    case 11: return 0x21;
    default: return r >> 5;
    }
}

static u32 SRV[16];
static u32 chk(int id) { return (SRV[id & 15] >> (id & 7)) & 1; }

/* pointer ids: region index + 1 in the high half, byte offset in the low half */
#define MAXR 160
static const unsigned char* RB[MAXR];
static unsigned RZ[MAXR];
static int NR;
static void preg(const void* p, unsigned n) { RB[NR] = (const unsigned char*) p; RZ[NR] = n; NR++; }
static u32 PI(const void* p)
{
    int i;
    unsigned long long a = (unsigned long long) p;
    if (p == 0) return 0;
    for (i = 0; i < NR; i++)
        if (a >= (unsigned long long) RB[i] && a < (unsigned long long) RB[i] + RZ[i])
            return ((u32) (i + 1) << 16) | (u32) (a - (unsigned long long) RB[i]);
    return 0xFFFFFFFFu;
}
#define MAXF 512
static const void* FP[MAXF];
static int NF;
static void freg(const void* f) { FP[NF++] = f; }
static u32 FI(const void* f)
{
    int i;
    if (f == 0) return 0;
    for (i = 0; i < NF; i++) if (FP[i] == f) return (u32) (i + 1);
    return 0xFFFFFFFEu;
}
static u32 SH(const char* s)
{
    u32 h = 0x811C9DC5u;
    if (s == 0) return 0;
    while (*s) { h = (h ^ (unsigned char) *s++) * 0x01000193u; }
    return h;
}
static u32 RH(const void* p, unsigned n)
{
    u32 h = 0x811C9DC5u;
    const unsigned char* b = (const unsigned char*) p;
    while (n--) h = (h ^ *b++) * 0x01000193u;
    return h;
}
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static void tu_case(int op, int k)
{
    int i;
    TR = 0;
    H = 0x811C9DC5u;
    rs = 0x9059u + (u32) op * 7919u + (u32) k * 104729u;
    for (i = 0; i < 16; i++) SRV[i] = rnd();
}
