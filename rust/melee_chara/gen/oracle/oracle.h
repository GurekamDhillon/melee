/* State-oracle prelude (rust/melee_chara/gen/oracle/oracle.py includes it after the game headers).
 *
 * The C side of a state oracle: the game's own .c (with the shipping float contraction, see
 * oracle.py) runs on a synthetic game state; every call it makes to code outside the TU hits a
 * generated stub that appends a text line (name, arguments, return value) to LG and answers from the
 * same seeded generator the Rust test uses. After the call the state is hashed (pointers
 * normalised to region id + offset, bitfield bytes masked and compared by value) together with the
 * log, the return value and the out-params. tests/oracle_common/mod.rs repeats every line of this
 * file; the expected hashes (tests/data/oracle_<stem>.txt) are computed from C source only.
 */
#ifndef ORACLE_H
#define ORACLE_H

typedef unsigned long long u64_;
int printf(const char*, ...);
int strcmp(const char*, const char*);
int _flushall(void);

/* ---------------------------------------------------------------- generator */
static u32 rs;
static u32 rnd(void)
{
    rs ^= rs << 13;
    rs ^= rs >> 17;
    rs ^= rs << 5;
    return rs;
}
static float rf(void)
{
    u32 k = rnd() % 16;
    int v;
    if (k == 0) return 0.0f;
    if (k == 1) return -0.0f;
    if (k == 2) return 1.0f;
    if (k == 3) return -1.0f;
    v = (int) (rnd() % 2001) - 1000;
    if (k < 10) return (float) v / 1000.0f;
    return (float) v / 7.0f;
}
/* a random integer field value: mostly small, 0 half the time-ish; negative only if signed */
static u32 iv(int sgn)
{
    u32 k = rnd() % 8;
    u32 v;
    if (k < 2) v = 0;
    else if (k < 5) v = rnd() % 4;
    else if (k < 7) v = rnd() % 400;
    else v = rnd() & 0xFFFF;
    if (sgn) {
        u32 n = rnd() % 6;
        if (n == 0) v = (u32) (-(int) v);
    }
    return v;
}
/* answers of stubbed callees */
static int gen_i(void)
{
    u32 k = rnd() % 4;
    if (k == 0) return 0;
    if (k == 1) return 1;
    if (k == 2) return 2;
    return -1;
}
static int gen_b(void) { return (int) (rnd() & 1); }

/* ---------------------------------------------------------------- hashing */
static u64_ fnv(u64_ h, const void* p, unsigned n)
{
    const unsigned char* b = (const unsigned char*) p;
    while (n--) {
        h ^= *b++;
        h *= 0x100000001B3ull;
    }
    return h;
}
#define FNV0 0xCBF29CE484222325ull
static u32 B_(float f)
{
    union { float f; u32 u; } c;
    c.f = f;
    return c.u;
}
static u64_ D_(double f)
{
    union { double f; u64_ u; } c;
    c.f = f;
    return c.u;
}

/* ---------------------------------------------------------------- regions, pointer names */
typedef struct { const unsigned char* b; u64_ n; } OReg;
static OReg OREGS[64];
static int NOREGS;
static const void* OFNS[256];
static int NOFNS;
static void oreg(const void* b, u64_ n)
{
    OREGS[NOREGS].b = (const unsigned char*) b;
    OREGS[NOREGS].n = n;
    NOREGS++;
}
static void ofn(const void* p) { OFNS[NOFNS++] = p; }
static u64_ normptr(const void* p)
{
    int i;
    const unsigned char* q = (const unsigned char*) p;
    if (!p) return 0;
    for (i = 0; i < NOREGS; i++)
        if (q >= OREGS[i].b && q < OREGS[i].b + OREGS[i].n)
            return ((u64_) (i + 1) << 32) | (u64_) (q - OREGS[i].b);
    for (i = 0; i < NOFNS; i++)
        if (OFNS[i] == p) return (u64_) (0x1000 + i) << 32;
    return (0xFFFFull << 32) | 1;
}
/* a pointer a stub returns: NULL a quarter of the time, else one of 4 scratch objects */
static unsigned char OSCR[4][64];
static void* gen_p(void)
{
    u32 k = rnd() % 5;
    if (k == 0) return 0;
    return OSCR[k - 1];
}

/* ---------------------------------------------------------------- the call log */
static char LG[32768];
static u32 LGN;
static void lg_c(char c)
{
    if (LGN < sizeof LG - 1) LG[LGN++] = c;
}
static void lg_s(const char* s)
{
    while (*s) lg_c(*s++);
}
static void lg_u(u64_ v)
{
    char t[24];
    int n = 0;
    if (!v) {
        lg_c('0');
        return;
    }
    while (v) {
        t[n++] = (char) ('0' + (int) (v % 10));
        v /= 10;
    }
    while (n) lg_c(t[--n]);
}
static void lg_hx(u64_ v, int digits) /* fixed width, lowercase */
{
    int i;
    for (i = digits - 1; i >= 0; i--) lg_c("0123456789abcdef"[(v >> (4 * i)) & 15]);
}
static void lg_hmin(u64_ v) /* minimal width */
{
    int n = 1;
    u64_ t = v >> 4;
    while (t) {
        n++;
        t >>= 4;
    }
    lg_hx(v, n);
}
static void lg_int(u32 v) { lg_u(v); lg_c(','); }
static void lg_f(float f) { lg_c('f'); lg_hx(B_(f), 8); lg_c(','); }
static void lg_d(double f) { lg_c('d'); lg_hx(D_(f), 16); lg_c(','); }
static void lg_ptr(const void* p) { lg_c('@'); lg_hmin(normptr(p)); lg_c(','); }
static void lg_v3(const Vec3* p)
{
    lg_c('@');
    lg_hmin(normptr(p));
    if (p) {
        lg_c('[');
        lg_hx(B_(p->x), 8);
        lg_c(' ');
        lg_hx(B_(p->y), 8);
        lg_c(' ');
        lg_hx(B_(p->z), 8);
        lg_c(']');
    }
    lg_c(',');
}
static void lg_v2(const Vec2* p)
{
    lg_c('@');
    lg_hmin(normptr(p));
    if (p) {
        lg_c('[');
        lg_hx(B_(p->x), 8);
        lg_c(' ');
        lg_hx(B_(p->y), 8);
        lg_c(']');
    }
    lg_c(',');
}
static void lg_pf(const float* p)
{
    lg_c('@');
    lg_hmin(normptr(p));
    if (p) {
        lg_c('[');
        lg_hx(B_(*p), 8);
        lg_c(']');
    }
    lg_c(',');
}
static void lg_pi(const int* p)
{
    lg_c('@');
    lg_hmin(normptr(p));
    if (p) {
        lg_c('[');
        lg_u((u32) *p);
        lg_c(']');
    }
    lg_c(',');
}
static void lg_oV3(Vec3* p)
{
    if (p) {
        p->x = rf();
        p->y = rf();
        p->z = rf();
        lg_c('<');
        lg_hx(B_(p->x), 8);
        lg_c(' ');
        lg_hx(B_(p->y), 8);
        lg_c(' ');
        lg_hx(B_(p->z), 8);
        lg_c('>');
    }
}
static void lg_oV2(Vec2* p)
{
    if (p) {
        p->x = rf();
        p->y = rf();
        lg_c('<');
        lg_hx(B_(p->x), 8);
        lg_c(' ');
        lg_hx(B_(p->y), 8);
        lg_c('>');
    }
}
static void lg_oPF(float* p)
{
    if (p) {
        *p = rf();
        lg_c('<');
        lg_hx(B_(*p), 8);
        lg_c('>');
    }
}
static void lg_oPI(int* p)
{
    if (p) {
        *p = gen_i();
        lg_c('<');
        lg_u((u32) *p);
        lg_c('>');
    }
}
static void lg_retu(u32 v) { lg_c('='); lg_u(v); }
static void lg_retf(float f) { lg_c('='); lg_c('f'); lg_hx(B_(f), 8); }
static void lg_retd(double f) { lg_c('='); lg_c('d'); lg_hx(D_(f), 16); }
static void lg_retp(const void* p) { lg_c('='); lg_c('@'); lg_hmin(normptr(p)); }

/* ---------------------------------------------------------------- struct fill by field kind */
typedef struct { u32 off; u32 size; char kind; } ODesc;
static void put32(unsigned char* p, u32 v)
{
    p[0] = (unsigned char) v;
    p[1] = (unsigned char) (v >> 8);
    p[2] = (unsigned char) (v >> 16);
    p[3] = (unsigned char) (v >> 24);
}
static void ofill(void* base, const ODesc* d, int n)
{
    int i;
    for (i = 0; i < n; i++) {
        unsigned char* p = (unsigned char*) base + d[i].off;
        u32 v, w;
        u32 j;
        switch (d[i].kind) {
        case 'f': put32(p, B_(rf())); break;
        case 'd': {
            float a = rf();
            u64_ b = D_((double) a);
            put32(p, (u32) b);
            put32(p + 4, (u32) (b >> 32));
            break;
        }
        case 'i': put32(p, iv(1)); break;
        case 'u': put32(p, iv(0)); break;
        case 's':
            v = iv(1);
            p[0] = (unsigned char) v;
            p[1] = (unsigned char) (v >> 8);
            break;
        case 'S':
            v = iv(0);
            p[0] = (unsigned char) v;
            p[1] = (unsigned char) (v >> 8);
            break;
        case 'c':
            v = iv(1);
            p[0] = (unsigned char) v;
            break;
        case 'C':
            v = iv(0);
            p[0] = (unsigned char) v;
            break;
        case 'q':
            v = iv(0);
            w = iv(0);
            put32(p, v);
            put32(p + 4, w & 0xF);
            break;
        case 'p':
            for (j = 0; j < 8; j++) p[j] = 0;
            break;
        case 'b':
            for (j = 0; j < d[i].size; j++) p[j] = (unsigned char) rnd();
            break;
        default: break;
        }
    }
}
/* normalise the pointer slots of a copy (kind 'p') in place */
static void onorm(unsigned char* img, const ODesc* d, int n)
{
    int i;
    for (i = 0; i < n; i++)
        if (d[i].kind == 'p') {
            const void* v = *(const void* const*) (img + d[i].off);
            u64_ k = normptr(v);
            unsigned j;
            for (j = 0; j < 8; j++) img[d[i].off + j] = (unsigned char) (k >> (8 * j));
        }
}

/* ---------------------------------------------------------------- efSync_Spawn (variadic) */
/* the pointee shapes of the trailing arguments, by effect id (tests/oracle_common repeats it):
 * 'V' = Vec3*, 'f' = float*, 'p' = anything else (name only) */
static const char* efsync_shape(int gfx)
{
    switch (gfx) {
    case 0x406: return "Vf";
    case 0x4CF: return "Vf";
    case 0x3FF: return "Vff";
    case 1285: return "V";
    default: return "pppp";
    }
}
#define OPP_NARG_(_1, _2, _3, _4, _5, _6, N, ...) N
#define OPP_NARG(...) OPP_NARG_(__VA_ARGS__, 6, 5, 4, 3, 2, 1, 0)
static void* oracle_efsync(int nextra, int gfx, void* gobj, ...)
{
    __builtin_va_list ap;
    const char* sh = efsync_shape(gfx);
    int i;
    void* r;
    lg_s("efSync_Spawn(");
    lg_int((u32) gfx);
    lg_ptr(gobj);
    __builtin_va_start(ap, gobj);
    for (i = 0; i < nextra; i++) {
        void* a = __builtin_va_arg(ap, void*);
        if (sh[i] == 'V') lg_v3((const Vec3*) a);
        else if (sh[i] == 'f') lg_pf((const float*) a);
        else lg_ptr(a);
    }
    __builtin_va_end(ap);
    lg_s(")");
    r = gen_p();
    lg_retp(r);
    lg_c(';');
    return r;
}

#endif
