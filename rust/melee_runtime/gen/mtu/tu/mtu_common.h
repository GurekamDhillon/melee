/* Shared harness for gen/mtu/tu/<stem>.c (included after the TU). Mirrored line for line by
 * rust/melee_world/tests/mtu_harness/mod.rs: same xorshift, same fill draws, same trace hash,
 * same pointer / function ids. */
extern int printf(const char*, ...);
extern int fflush(void*);
/* A failed HSD_ASSERT / HSD_Panic / OSPanic ends the case: the driver prints "<op> <k> ABORT" and the Rust test expects
 * its port to panic on the same case (catch_unwind). */
typedef struct { unsigned long long b[40]; } mtu_jb_t __attribute__((aligned(16)));
extern int _setjmp(mtu_jb_t*, void*);
extern void longjmp(mtu_jb_t*, int);
static mtu_jb_t AJB;
#define MTU_SETJMP() _setjmp(&AJB, __builtin_frame_address(0))
static int ARMED;
void __assert(const char* file, u32 line, const char* cond) { (void) file; (void) line; (void) cond; if (ARMED) longjmp(&AJB, 1); for (;;) {} }
__attribute__((noreturn)) void HSD_Panic(const char* file, u32 line, const char* msg) { (void) file; (void) line; (void) msg; if (ARMED) longjmp(&AJB, 1); for (;;) {} }
__attribute__((noreturn)) void OSPanic(const char* file, int line, const char* msg, ...) { (void) file; (void) line; (void) msg; if (ARMED) longjmp(&AJB, 1); for (;;) {} }
void OSReport(char* f, ...) { (void) f; }

typedef unsigned long long u64_t_;
static u32 TR;
#ifndef DBG_OP
#define DBG_OP -1
#define DBG_K -1
#endif
static int TRACEON;
static int NCALLS;
static void T(u32 v) { if (++NCALLS > 20000 && ARMED) longjmp(&AJB, 1); TR = TR * 31u + v; if (TRACEON) printf("T %08X%c", v, 10); }
static u32 B(float f) { union { float f; u32 u; } c; c.f = f; return c.u; }
static void T64(double d) { union { double d; u64_t_ u; } c; c.d = d; T((u32) c.u); T((u32) (c.u >> 32)); }
static u32 H;
static int HDBG;
static void HM(u32 v) { H = H * 0x01000193u ^ v; if (HDBG) printf("H %08X%c", v, 10); }
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

static u64_t_ fv64(void) { u64_t_ hi = fv(); u64_t_ lo = fv(); return (hi << 32) | lo; }

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
/* a data address truncated to 32 bits ((u32) ptr): named by the registered region it falls in */
static u32 PI32(u32 v)
{
    int i;
    if (v == 0) return 0;
    for (i = 0; i < NR; i++) {
        u32 b = (u32) (unsigned long long) RB[i];
        if (v >= b && v - b < RZ[i]) return ((u32) (i + 1) << 16) | (v - b);
    }
    return 0xFFFFFFFCu;
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
/* a pointer slot inside a union that other views overwrite byte-wise: only an exact region base counts */
static u32 PIB(const void* p) { u32 v = PI(p); return (v == 0 || v == 0xFFFFFFFFu || (v & 0xFFFFu) == 0) ? v : 0xFFFFFFFEu; }
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
    NCALLS = 0;
    TRACEON = (op == DBG_OP && k == DBG_K);
    HDBG = TRACEON;
    H = 0x811C9DC5u;
    rs = 0x9059u + (u32) op * 7919u + (u32) k * 104729u;
    for (i = 0; i < 16; i++) SRV[i] = rnd();
}

/* sprintf for the oracle drivers (no libc link): %s %c %d %i %u %x %X %%, flags 0 and -, width. */
int sprintf(char* out, const char* fmt, ...)
{
    __builtin_va_list ap;
    char* o = out;
    __builtin_va_start(ap, fmt);
    while (*fmt) {
        if (*fmt != '%') { *o++ = *fmt++; continue; }
        {
            int zero = 0, left = 0, width = 0, base = 10, neg = 0, len = 0, i;
            char tmp[40];
            const char* str = 0;
            unsigned long long v = 0;
            fmt++;
            for (;; fmt++) { if (*fmt == '0') zero = 1; else if (*fmt == '-') left = 1; else break; }
            while (*fmt >= '0' && *fmt <= '9') width = width * 10 + (*fmt++ - '0');
            if (*fmt == 'l') fmt++;
            switch (*fmt) {
            case '%': *o++ = '%'; fmt++; continue;
            case 'c': tmp[0] = (char) __builtin_va_arg(ap, int); len = 1; str = tmp; break;
            case 's': str = __builtin_va_arg(ap, const char*); while (str[len]) len++; break;
            case 'd': case 'i': { int x = __builtin_va_arg(ap, int); if (x < 0) { neg = 1; v = (unsigned long long) (-(long long) x); } else v = (unsigned) x; } goto num;
            case 'u': v = __builtin_va_arg(ap, unsigned); goto num;
            case 'x': case 'X': v = __builtin_va_arg(ap, unsigned); base = 16;
            num: {
                    int n = 0;
                    if (v == 0) tmp[n++] = '0';
                    while (v) { int d = (int) (v % base); tmp[n++] = (char) (d < 10 ? '0' + d : (*fmt == 'X' ? 'A' : 'a') + d - 10); v /= base; }
                    if (neg) tmp[n++] = '-';
                    for (i = 0; i < n / 2; i++) { char t = tmp[i]; tmp[i] = tmp[n - 1 - i]; tmp[n - 1 - i] = t; }
                    str = tmp; len = n;
                    if (zero && !left && neg && len < width) { /* sign before the zeros */
                        *o++ = '-'; str++; len--; width--;
                    }
                }
                break;
            default: *o++ = *fmt; fmt++; continue;
            }
            fmt++;
            if (!left) for (i = len; i < width; i++) *o++ = zero && *(fmt - 1) != 's' && *(fmt - 1) != 'c' ? '0' : ' ';
            for (i = 0; i < len; i++) *o++ = str[i];
            if (left) for (i = len; i < width; i++) *o++ = ' ';
        }
    }
    *o = 0;
    __builtin_va_end(ap);
    return (int) (o - out);
}
