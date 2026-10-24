/* Shared scaffolding of the lb* TU drivers (shard 833-840): the pointer registry (pointers never print
 * raw: registered ranges as name+offset, registered functions as f:name), OSReport formatting, and
 * __assert as a longjmp so a driver can run the assert paths. Mirrored by rust/melee_modes/tests/lbtest/mod.rs. */
#ifndef TU_LB_COMMON_H
#define TU_LB_COMMON_H
#include "tu_gm_common.h"
#include <stdarg.h>

#define LG(...) lg(__VA_ARGS__)

/* a 1 MiB zeroed arena below 4 GB at a fixed address (the game casts pointers through u32) */
extern void* VirtualAlloc(void*, unsigned long long, unsigned long, unsigned long);
static unsigned char* ARENA_P;
static unsigned long long ARENA_USED;
static void arena_init(void)
{
    if (!ARENA_P) ARENA_P = (unsigned char*) VirtualAlloc((void*) 0x20000000ull, 1ull << 20, 0x3000, 4);
    if (!ARENA_P) { printf("arena_init failed"); for (;;) {} }
    memset(ARENA_P, 0, 1u << 20);
    ARENA_USED = 64;
}
static void* AZ(unsigned long long n)
{
    void* p = ARENA_P + ARENA_USED;
    ARENA_USED += (n + 15) & ~15ull;
    return p;
}
/* main RAM window: 0x80000000 + 33 MiB (the bus clock at 0x800000F8, heap memory from 0x80100000) */
static unsigned char* MEM2_P;
static void mem2_init(void)
{
    if (!MEM2_P) MEM2_P = (unsigned char*) VirtualAlloc((void*) 0x80000000ull, 0x2100000ull, 0x3000, 4);
    if (!MEM2_P) { printf("mem2_init failed"); for (;;) {} }
    memset(MEM2_P, 0, 0x2100000);
    *(unsigned*) (MEM2_P + 0xF8) = 162000000u;
}
typedef struct { const char* name; unsigned long long base, n; } PReg;
static PReg PREGS[64];
static int NPREG;
static void preg(const char* name, const void* p, unsigned long long n)
{
    PREGS[NPREG].name = name;
    PREGS[NPREG].base = (unsigned long long) (size_t) p;
    PREGS[NPREG].n = n;
    NPREG++;
}
static void preg_clear(void) { NPREG = 0; }
typedef struct { const char* name; unsigned long long f; } PFn;
static PFn PFNS[32];
static int NPFN;
static void pfreg(const char* name, const void* f)
{
    PFNS[NPFN].name = name;
    PFNS[NPFN].f = (unsigned long long) (size_t) f;
    NPFN++;
}
static char g_pidbuf[16][48];
static int g_pidk;
static const char* pidx(unsigned long long v, int allow_num)
{
    char* b = g_pidbuf[g_pidk++ & 15];
    int i;
    for (i = 0; i < NPREG; i++) {
        if (v >= PREGS[i].base && v < PREGS[i].base + PREGS[i].n) {
            unsigned long long o = v - PREGS[i].base;
            char t[24];
            int n = 0, k = 0;
            const char* s = PREGS[i].name;
            while (*s) b[k++] = *s++;
            b[k++] = '+';
            do { t[n++] = (char) ('0' + o % 10); o /= 10; } while (o);
            while (n) b[k++] = t[--n];
            b[k] = 0;
            return b;
        }
    }
    for (i = 0; i < NPFN; i++) {
        if (PFNS[i].f == v) {
            const char* s = PFNS[i].name;
            int k = 0;
            b[k++] = 'f'; b[k++] = ':';
            while (*s) b[k++] = *s++;
            b[k] = 0;
            return b;
        }
    }
    {
        char t[24];
        int n = 0, k = 0;
        unsigned long long o = v;
        if (v == 0) { b[0] = 'n'; b[1] = 'u'; b[2] = 'l'; b[3] = 'l'; b[4] = 0; return b; }
        if (!allow_num || v > 0xFFFFFFFFull) { b[0] = '?'; b[1] = 0; return b; }
        b[k++] = 'n';
        do { t[n++] = (char) ('0' + o % 10); o /= 10; } while (o);
        while (n) b[k++] = t[--n];
        b[k] = 0;
        return b;
    }
}
static const char* pidu(unsigned long long v) { return pidx(v, 1); }
static const char* pid(const void* p) { return pidx((unsigned long long) (size_t) p, 0); }
/* a C string: its content in quotes (never an address) */
static const char* pids(const char* s)
{
    char* b = g_pidbuf[g_pidk++ & 15];
    int k = 0;
    if (s == NULL) return "null";
    b[k++] = '"';
    while (*s && k < 40) b[k++] = *s++;
    b[k++] = '"';
    b[k] = 0;
    return b;
}

/* OSReport: "%[0][w]d|u|x|s" */
static char g_rep[512];
static int g_repn;
static void rep_c(char c) { if (g_repn < 510) { g_rep[g_repn++] = c; g_rep[g_repn] = 0; } }
static int g_upper;
static void rep_u(unsigned long v, unsigned base, int width, int zp)
{
    char t[40];
    int n = 0;
    do { t[n++] = (g_upper ? "0123456789ABCDEF" : "0123456789abcdef")[v % base]; v /= base; } while (v);
    while (n < width) t[n++] = zp ? '0' : ' ';
    while (n) rep_c(t[--n]);
}
static void os_report_va(const char* fmt, va_list ap)
{
    g_repn = 0; g_rep[0] = 0;
    while (*fmt) {
        int width = 0, zp = 0;
        if (*fmt != '%') { rep_c(*fmt++); continue; }
        fmt++;
        if (*fmt == '0') { zp = 1; fmt++; }
        while (*fmt >= '0' && *fmt <= '9') width = width * 10 + (*fmt++ - '0');
        switch (*fmt++) {
        case 'd': {
            long v = (long) va_arg(ap, int);
            if (v < 0) { rep_c('-'); rep_u((unsigned long) -v, 10, width ? width - 1 : 0, zp); }
            else rep_u((unsigned long) v, 10, width, zp);
            break;
        }
        case 'u': rep_u((unsigned long) va_arg(ap, unsigned), 10, width, zp); break;
        case 'x': rep_u((unsigned long) va_arg(ap, unsigned), 16, width, zp); break;
        case 'X': g_upper = 1; rep_u((unsigned long) va_arg(ap, unsigned), 16, width, zp); g_upper = 0; break;
        case 's': { const char* p = va_arg(ap, const char*); while (*p) rep_c(*p++); break; }
        }
    }
    LG("OSReport(%s);", g_rep);
}
void OSReport(char* fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    os_report_va(fmt, ap);
    va_end(ap);
}

/* __assert: log and jump back to TRY */
extern int _setjmp(void*, void*);
extern void longjmp(void*, int);
static unsigned long long g_jb[64] __attribute__((aligned(16)));
static int g_asserted;
void __assert(const char* f, u32 l, const char* c)
{
    (void) f;
    LG("assert(%u,%s);", (unsigned) l, c);
    longjmp(g_jb, 1);
}
#define TRY(stmt) \
    do { \
        g_asserted = 0; \
        if (_setjmp(g_jb, (void*) 0) == 0) { \
            stmt; \
        } else { \
            g_asserted = 1; \
        } \
    } while (0)
#endif
