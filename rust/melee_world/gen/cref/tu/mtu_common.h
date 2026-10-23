/* Shared harness for gen/cref/tu/<stem>.c (included after the TU). Mirrored line for line by
 * rust/melee_modes/tests/mtu_harness/mod.rs: same xorshift, same fill draws, same trace hash,
 * same pointer / function ids. */
extern int printf(const char*, ...);
extern int fflush(void*);
extern void exit(int);
void __assert(const char* file, u32 line, const char* cond) { printf("ASSERT %s:%u %s\n", file, line, cond); fflush(0); exit(3); }
void OSReport(char* f, ...) { (void) f; }

typedef unsigned long long u64_t_;
static u32 TR;
static int USEL; /* active view of the unions the spec names (gen_tu.py `unions`) */
#ifndef DBG_OP
#define DBG_OP -1
#define DBG_K -1
#endif
static int TRACEON;
static void T(u32 v) { TR = TR * 31u + v; if (TRACEON) printf("T %08X%c", v, 10); }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void T64(double d) { union { double d; u64_t_ u; } c; c.d = d; T((u32) c.u); T((u32) (c.u >> 32)); }
static u32 H;
static void HN(const char* s) { if (TRACEON) printf("N %s%c", s, 10); }
static void HM(u32 v) { H = H * 0x01000193u ^ v; if (TRACEON) printf("H %08X %08X%c", v, H, 10); }
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

/* 64-bit draw: low word first (C leaves the order of two calls in one expression open) */
static u64_t_ fv64(void) { u32 lo = fv(); u32 hi = fv(); return ((u64_t_) hi << 32) | lo; }

static u32 SRV[16];
static u32 chk(int id) { return (SRV[id & 15] >> (id & 7)) & 1; }

/* pointer ids: region index + 1 in the high half, byte offset in the low half */
#define MAXR 160
static const unsigned char* RB[MAXR];
static unsigned RZ[MAXR];
static int NR;
static void preg(const void* p, unsigned n) { RB[NR] = (const unsigned char*) p; RZ[NR] = n; NR++; }
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
/* a function address truncated to 32 bits ((s32) fnptr): named by the registered function it is */
static u32 FI32(u32 v)
{
    int i;
    if (v == 0) return 0;
    for (i = 0; i < NF; i++) if ((u32) (unsigned long long) FP[i] == v) return (u32) (i + 1);
    return 0xFFFFFFFDu;
}
/* a data pointer that is really a function address (UNK_T slots) gets the function's id */
static u32 PI(const void* p)
{
    int i;
    unsigned long long a = (unsigned long long) p;
    if (p == 0) return 0;
    for (i = 0; i < NR; i++)
        if (a >= (unsigned long long) RB[i] && a < (unsigned long long) RB[i] + RZ[i])
            return ((u32) (i + 1) << 16) | (u32) (a - (unsigned long long) RB[i]);
    for (i = 0; i < NF; i++) if (FP[i] == p) return 0xF0000000u | (u32) (i + 1);
    return 0xFFFFFFFFu;
}
/* a data pointer truncated to 32 bits ((int) ptr): named by the region it points into (low 32 bits compared) */
static u32 PI32(u32 v)
{
    int i;
    if (v == 0) return 0;
    for (i = 0; i < NR; i++) {
        u32 b = (u32) (unsigned long long) RB[i];
        if (v >= b && v < b + RZ[i]) return ((u32) (i + 1) << 16) | (v - b);
    }
    return 0xFFFFFFFFu;
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
    TRACEON = (op == DBG_OP && k == DBG_K);
    H = 0x811C9DC5u;
    rs = 0x9059u + (u32) op * 7919u + (u32) k * 104729u;
    for (i = 0; i < 16; i++) SRV[i] = rnd();
}
