/* Shared harness for gen/cref/tu/<stem>.c (included after the TU). Mirrored by
 * tests/tu_gm_harness/mod.rs: same xorshift, same fill order, same hashes. */
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; for (;;) {} }
int printf(const char*, ...);

static u32 TR;
static void T(u32 v) { TR = TR * 31u + v; }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
/* byte-wise FNV-1a over a buffer, folded into the trace */
static u32 HBUF(const void* p, unsigned n)
{
    const unsigned char* b = p;
    u32 h = 0x811C9DC5u;
    while (n--) { h ^= *b++; h *= 0x01000193u; }
    return h;
}
static void TB(const void* p, unsigned n) { T(HBUF(p, n)); }

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
/* every byte random (pointer fields must be re-zeroed/set by the caller) */
static void fillb(void* p, unsigned n)
{
    unsigned char* b = p;
    while (n--) *b++ = (unsigned char) (rnd() >> 11);
}
static void zero(void* p, unsigned n) { unsigned char* b = p; while (n--) *b++ = 0; }
static u32 SRV[16];
static u32 SR(int i) { return (SRV[i & 15] >> (i >> 4)) & 1; }
static void tu_case(int op, int k)
{
    int i;
    TR = 0;
    rs = 0x9059u + (u32) op * 7919u + (u32) k * 104729u;
    for (i = 0; i < 16; i++) SRV[i] = rnd();
}
