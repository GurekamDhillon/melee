/* Shared logging and seeded stream for the group-6 gm TU drivers (gen/turef/tu_gm*_ref.c).
 * Mirrored by the Rust tests (rust/melee_modes/tests/tu_gm*.rs): the same xorshift32 stream and the
 * same %d/%u/%x/%08x/%s formatter (the game's libc has no vsnprintf). */
#ifndef TU_GM_COMMON_H
#define TU_GM_COMMON_H
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

static char g_log[8192];
static size_t g_loglen;
static void app_c(char c)
{
    if (g_loglen + 1 < sizeof g_log) {
        g_log[g_loglen++] = c;
        g_log[g_loglen] = 0;
    }
}
static void app_u(unsigned long v, unsigned base, int width, int zeropad)
{
    char t[40];
    int n = 0;
    do {
        t[n++] = "0123456789abcdef"[v % base];
        v /= base;
    } while (v);
    while (n < width) {
        t[n++] = zeropad ? '0' : ' ';
    }
    while (n) {
        app_c(t[--n]);
    }
}
static void lg(const char* fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    while (*fmt) {
        int width = 0;
        int zp = 0;
        if (*fmt != '%') {
            app_c(*fmt++);
            continue;
        }
        fmt++;
        if (*fmt == '0') {
            zp = 1;
            fmt++;
        }
        while (*fmt >= '0' && *fmt <= '9') {
            width = width * 10 + (*fmt++ - '0');
        }
        switch (*fmt++) {
        case 'd': {
            long v = (long) va_arg(ap, int);
            if (v < 0) {
                app_c('-');
                app_u((unsigned long) -v, 10, 0, 0);
            } else {
                app_u((unsigned long) v, 10, 0, 0);
            }
            break;
        }
        case 'u': app_u((unsigned long) va_arg(ap, unsigned), 10, 0, 0); break;
        case 'x': app_u((unsigned long) va_arg(ap, unsigned), 16, width, zp); break;
        case 's': {
            const char* p = va_arg(ap, const char*);
            while (*p) {
                app_c(*p++);
            }
            break;
        }
        }
    }
    va_end(ap);
}
static u32 g_s = 0x2468ACEu;
static u32 rnd(void)
{
    g_s ^= g_s << 13;
    g_s ^= g_s >> 17;
    g_s ^= g_s << 5;
    return g_s;
}
static float rf(void)
{
    switch (rnd() % 8) {
    case 0: return 0.0f;
    case 1: return -0.0f;
    case 2: return 1.0f;
    case 3: return -1.0f;
    default: return (float) ((int) (rnd() % 4001) - 2000) / 997.0f;
    }
}
static u32 bits(float f)
{
    union {
        float f;
        u32 u;
    } c;
    c.f = f;
    return c.u;
}

#endif
